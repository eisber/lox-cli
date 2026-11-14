//! `lox live` — thin, read-only-by-default helpers over the Miniserver's
//! `jdev/sps/io` HTTP API.
//!
//! These exist so live verification (does the Virtual Input echo? what is the
//! actor showing? what is the Miniserver clock?) does not require hand-rolled
//! `curl`/`Invoke-WebRequest` with basic-auth and `--SkipCertificateCheck`.
//! Reads are unrestricted; writes are gated behind `--write`.
//!
//! Note: color/composite (`<v.col>`) states return the literal template string
//! over HTTP polling, not a number — perceived color cannot be read back here
//! (it needs the websocket/visu channel), so color verification stays a
//! human-in-the-loop step.

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lox_core::runtime::{Cancellation, ClockDriver, ClockError, ClockOptions, Diagnostic};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::client::{self, HttpStatusError, LoxClient};
use crate::commands::RunContext;
use crate::config::Config;

#[derive(Subcommand)]
pub enum LiveCmd {
    /// Read a control/connector state: GET /jdev/sps/io/<uuid>
    Get {
        /// Control UUID (or name resolvable via the structure cache)
        uuid: String,
    },
    /// Write a value/command to a control: GET /jdev/sps/io/<uuid>/<value>
    Set {
        /// Control UUID (or name resolvable via the structure cache)
        uuid: String,
        /// Value or command string (e.g. "1", "0", "on", "hsv(120,100,30)")
        value: String,
        /// Required to perform the write (live, mutating operation)
        #[arg(long)]
        write: bool,
    },
    /// Show the Miniserver date and time
    Time,
}

pub fn cmd_live(ctx: &RunContext, action: LiveCmd) -> Result<()> {
    let cfg = Config::load()?;
    match action {
        LiveCmd::Get { uuid } => get(ctx, &mut LoxClient::new(cfg)?, &uuid),
        LiveCmd::Set { uuid, value, write } => {
            set(ctx, &mut LoxClient::new(cfg)?, &uuid, &value, write)
        }
        LiveCmd::Time => time(ctx, cfg),
    }
}

/// Extract `(value, code)` from a Loxone `{"LL": {...}}` envelope.
fn parse_ll(v: &Value) -> (String, Option<String>) {
    let ll = &v["LL"];
    let value = ll["value"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| ll["value"].as_f64().map(|f| f.to_string()))
        .unwrap_or_default();
    let code = ll["Code"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| ll["code"].as_str().map(|s| s.to_string()))
        .or_else(|| ll["Code"].as_i64().map(|n| n.to_string()));
    (value, code)
}

fn get(ctx: &RunContext, lox: &mut LoxClient, name_or_uuid: &str) -> Result<()> {
    let uuid = lox.resolve(name_or_uuid)?;
    let resp = lox
        .get_json(&format!("/jdev/sps/io/{}", uuid))
        .with_context(|| format!("reading state of {uuid}"))?;
    let (value, code) = parse_ll(&resp);
    if ctx.json {
        println!("{}", json!({ "uuid": uuid, "value": value, "code": code }));
    } else {
        println!("{value}");
        if value.contains("<v.col>") || value.contains("<v>") {
            eprintln!(
                "note: this is a color/template state — HTTP polling returns the literal \
                 template, not a number. Perceived color is not readable here."
            );
        }
    }
    Ok(())
}

fn set(
    ctx: &RunContext,
    lox: &mut LoxClient,
    name_or_uuid: &str,
    value: &str,
    write: bool,
) -> Result<()> {
    if !write {
        bail!(
            "refusing to write to the live Miniserver without --write \
             (would send '{value}' to '{name_or_uuid}')"
        );
    }
    let uuid = lox.resolve(name_or_uuid)?;
    let resp = lox
        .send_cmd(&uuid, value)
        .with_context(|| format!("writing '{value}' to {uuid}"))?;
    let (ret, code) = parse_ll(&resp);
    if ctx.json {
        println!(
            "{}",
            json!({ "uuid": uuid, "sent": value, "value": ret, "code": code })
        );
    } else if !ctx.quiet {
        println!(
            "✓ {uuid} ← {value}  (Code {})",
            code.as_deref().unwrap_or("?")
        );
        if !ret.is_empty() {
            println!("  value: {ret}");
        }
    }
    Ok(())
}

fn time(ctx: &RunContext, cfg: Config) -> Result<()> {
    let mut options = ClockOptions::new(&cfg.host, &cfg.user, &cfg.pass);
    options.verify_tls = cfg.verify_ssl.unwrap_or(false);
    options.user_agent = client::USER_AGENT.into();
    let verbosity = client::verbose();
    if verbosity >= 1 {
        if !options.verify_tls {
            eprintln!("warning: TLS certificate verification is disabled (verify_ssl not set)");
        }
        let mut host = reqwest::Url::parse(&cfg.host)?;
        let _ = host.set_username("");
        let _ = host.set_password(None);
        let host = client::redact_url(host.as_str());
        options.diagnostics = Some(Arc::new(move |event| match event {
            Diagnostic::Request(endpoint) => {
                eprintln!("GET {}{}", host.trim_end_matches('/'), endpoint.path());
            }
            Diagnostic::Response { status, .. } => {
                let reason = reqwest::StatusCode::from_u16(status)
                    .ok()
                    .and_then(|status| status.canonical_reason())
                    .unwrap_or("");
                eprintln!("  → {status} {reason}");
            }
            Diagnostic::Body { bytes, .. } if verbosity >= 2 => {
                eprintln!("  body: <{bytes} bytes>");
            }
            _ => {}
        }));
    }
    let driver = ClockDriver::new(options)?;
    let result = driver.read_clock(&Cancellation::new());
    // Always close before rendering, including failure, without replacing the
    // operation's original diagnostic with a secondary cleanup error.
    let closed = driver.close();
    let observation = result.map_err(clock_error)?;
    closed?;
    let clock = observation.legacy_text();
    let (date, time) = (clock.date, clock.time);
    if ctx.json {
        println!("{}", json!({ "date": date, "time": time }));
    } else {
        println!("{date} {time}");
    }
    Ok(())
}

fn clock_error(error: ClockError) -> anyhow::Error {
    match error {
        ClockError::Http { endpoint, status } => anyhow::Error::new(HttpStatusError {
            status,
            path: endpoint.path().into(),
        }),
        error => error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};
    use std::time::Duration;

    #[test]
    fn clock_timeout_category_matches_legacy_transport() {
        let server = MockServer::start();
        let date = server.mock(|when, then| {
            when.method(GET).path("/jdev/sys/date");
            then.delay(Duration::from_millis(200)).body("{}");
        });
        let mut legacy = LoxClient::new(Config {
            host: server.base_url(),
            user: "clock-test".into(),
            pass: "synthetic-only".into(),
            ..Config::default()
        })
        .unwrap();
        legacy.client = reqwest::blocking::Client::builder()
            .no_proxy()
            .timeout(Duration::from_millis(20))
            .build()
            .unwrap();
        let legacy_error = legacy.get_json("/jdev/sys/date").unwrap_err();
        let mut options = ClockOptions::new(server.base_url(), "clock-test", "synthetic-only");
        options.use_system_proxy = false;
        options.timeout = Duration::from_millis(20);
        options.retry_delays.clear();
        let driver = ClockDriver::new(options).unwrap();
        let error = clock_error(driver.read_clock(&Cancellation::new()).unwrap_err());
        driver.close().unwrap();
        assert_eq!(
            crate::categorize_error(&error),
            crate::categorize_error(&legacy_error)
        );
        date.assert_hits(5);
    }
}
