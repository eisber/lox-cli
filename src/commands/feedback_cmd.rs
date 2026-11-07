use anyhow::{Context, Result, bail};
use clap::Subcommand;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::commands::RunContext;
use crate::telemetry::Telemetry;

const REPOSITORY: &str = "eisber/lox-cli";

#[derive(Subcommand)]
pub enum FeedbackCmd {
    /// Report a CLI issue to GitHub, falling back to telemetry when enabled
    #[command(
        after_help = "Only send sanitized issue details. Do not include credentials, private configuration, or personal data. GitHub issues are public; telemetry fallback includes the supplied title and body."
    )]
    Report {
        /// Short issue summary
        #[arg(long)]
        title: String,
        /// Sanitized reproduction steps, expected behavior, and actual behavior
        #[arg(
            long,
            required_unless_present = "body_file",
            conflicts_with = "body_file"
        )]
        body: Option<String>,
        /// Read the report body from a UTF-8 file, or - for stdin
        #[arg(long, required_unless_present = "body", conflicts_with = "body")]
        body_file: Option<PathBuf>,
    },
}

struct Report {
    id: String,
    title: String,
    body: String,
}

impl Report {
    fn new(title: String, body: String) -> Result<Self> {
        if title.trim().is_empty() || body.trim().is_empty() {
            bail!(
                "Issue title and body must not be empty. Supply --title and --body or --body-file."
            );
        }
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            body,
        })
    }

    fn github_body(&self) -> String {
        format!(
            "{}\n\n---\nReported via `lox feedback report`\n\
             lox {} | {} / {}\nReport ID: {}",
            self.body,
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            self.id
        )
    }
}

pub fn cmd_feedback(ctx: &RunContext, action: FeedbackCmd, telemetry: &Telemetry) -> Result<()> {
    let FeedbackCmd::Report {
        title,
        body,
        body_file,
    } = action;
    let body = match (body, body_file) {
        (Some(body), None) => body,
        (None, Some(path)) if path.as_os_str() == "-" => {
            let mut body = String::new();
            std::io::stdin()
                .read_to_string(&mut body)
                .context("Cannot read issue report from stdin")?;
            body
        }
        (None, Some(path)) => std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read issue report from {}", path.display()))?,
        _ => bail!("Supply exactly one of --body or --body-file."),
    };
    let report = Report::new(title, body)?;

    let mut result = if ctx.dry_run {
        json!({
            "ok": true,
            "dry_run": true,
            "repository": REPOSITORY,
            "title": report.title,
            "body": report.github_body(),
            "report_id": report.id,
            "telemetry_fallback_enabled": telemetry.is_enabled(),
        })
    } else {
        deliver_report(&report, create_github_issue, |report| {
            telemetry.report_issue(&report.id, &report.title, &report.body)
        })?
    };
    if let Some(trace_id) = &ctx.trace_id {
        result["trace_id"] = json!(trace_id);
    }

    if ctx.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if !ctx.quiet {
        if ctx.dry_run {
            println!(
                "Would report to {REPOSITORY} (telemetry fallback: {}).\n{}\n\n{}",
                telemetry.is_enabled(),
                report.title,
                report.github_body()
            );
        } else if let Some(url) = result["url"].as_str() {
            println!("GitHub issue created: {url}");
        } else {
            if let Some(error) = result["github_error"].as_str() {
                eprintln!("GitHub unavailable: {error}");
            }
            println!(
                "Issue report accepted by telemetry. Report ID: {}",
                report.id
            );
        }
    }
    Ok(())
}

fn deliver_report(
    report: &Report,
    github: impl FnOnce(&str, &str) -> Result<String>,
    telemetry: impl FnOnce(&Report) -> Result<()>,
) -> Result<Value> {
    match github(&report.title, &report.github_body()) {
        Ok(url) => Ok(json!({
            "ok": true,
            "channel": "github",
            "url": url,
            "report_id": report.id,
        })),
        Err(github_error) => {
            telemetry(report).map_err(|telemetry_error| {
                anyhow::anyhow!(
                    "GitHub issue creation failed: {github_error:#}. \
                     Telemetry fallback failed: {telemetry_error:#}. \
                     No delivery was confirmed. Install gh and run `gh auth login`, \
                     or file the report at https://github.com/{REPOSITORY}/issues/new."
                )
            })?;
            Ok(json!({
                "ok": true,
                "channel": "telemetry",
                "url": null,
                "report_id": report.id,
                "github_error": format!("{github_error:#}"),
            }))
        }
    }
}

pub(crate) fn create_github_issue(title: &str, body: &str) -> Result<String> {
    let mut child = Command::new("gh")
        .args([
            "issue",
            "create",
            "--repo",
            REPOSITORY,
            "--title",
            title,
            "--body-file",
            "-",
        ])
        .env("GH_HOST", "github.com")
        .env("GH_PROMPT_DISABLED", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Cannot run GitHub CLI; install gh and run `gh auth login`")?;
    // Stdin avoids shell quoting and Windows command-line limits for long reports.
    let write_result = child
        .stdin
        .take()
        .context("Cannot open gh stdin")?
        .write_all(body.as_bytes());
    let output = child
        .wait_with_output()
        .context("Cannot wait for GitHub issue creation")?;
    if !output.status.success() {
        bail!(
            "gh issue create exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    write_result.context("Cannot send issue body to gh")?;
    let url = String::from_utf8(output.stdout).context("gh returned a non-UTF-8 issue URL")?;
    let url = url.trim();
    let prefix = format!("https://github.com/{REPOSITORY}/issues/");
    if !url
        .strip_prefix(&prefix)
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|c| c.is_ascii_digit()))
    {
        bail!(
            "gh reported success without an issue URL; check https://github.com/{REPOSITORY}/issues"
        );
    }
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> Report {
        Report::new(
            "Simulator issue".into(),
            "Synthetic reproduction; expected 1, got 0.".into(),
        )
        .unwrap()
    }

    #[test]
    fn github_success_never_sends_telemetry() {
        let report = report();
        let result = deliver_report(
            &report,
            |title, body| {
                assert_eq!(title, report.title);
                assert!(body.starts_with(&report.body));
                assert!(body.contains(&report.id));
                Ok("https://github.com/eisber/lox-cli/issues/123".into())
            },
            |_| panic!("Telemetry must not run after GitHub succeeds"),
        )
        .unwrap();
        assert_eq!(result["channel"], "github");
        assert_eq!(result["report_id"], report.id);
        assert_eq!(
            result["url"],
            "https://github.com/eisber/lox-cli/issues/123"
        );
    }

    #[test]
    fn github_failure_falls_back_with_supplied_content() {
        let report = report();
        let result = deliver_report(
            &report,
            |_, _| bail!("gh not authenticated"),
            |fallback| {
                assert_eq!(fallback.title, report.title);
                assert_eq!(fallback.body, report.body);
                assert_eq!(fallback.id, report.id);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(result["channel"], "telemetry");
        assert!(result["url"].is_null());
        assert_eq!(result["github_error"], "gh not authenticated");
    }

    #[test]
    fn failed_fallback_is_not_reported_as_success() {
        for reason in ["Telemetry is disabled", "HTTP 503", "connection timed out"] {
            let error = deliver_report(
                &report(),
                |_, _| bail!("gh unavailable"),
                |_| bail!("{reason}"),
            )
            .unwrap_err()
            .to_string();
            assert!(error.contains("gh unavailable"));
            assert!(error.contains(reason));
            assert!(error.contains("No delivery was confirmed"));
        }
    }

    #[test]
    fn empty_reports_are_rejected() {
        for (title, body) in [("", "details"), ("summary", " \n"), (" ", "")] {
            assert!(Report::new(title.into(), body.into()).is_err());
        }
    }
}
