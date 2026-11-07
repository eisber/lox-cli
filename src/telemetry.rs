//! Anonymous usage telemetry and explicit issue reporting for improving `lox`.
//!
//! **Default: ON** — opt out via `lox telemetry disable` or `LOX_TELEMETRY=0`.
//!
//! Usage events exclude private data. Explicit feedback reports additionally
//! include the supplied issue title and body. See TELEMETRY.md for details.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::OnceLock;

/// PostHog capture endpoint.
const POSTHOG_ENDPOINT: &str = "https://eu.i.posthog.com/capture/";
/// PostHog project API key (public — safe to embed in open-source code).
const POSTHOG_API_KEY: &str = "phc_oH60DjJc0VEuuFQZYDv6b7KrVQzGTk3JDBPmiHVccpG";

/// Maximum time to wait for a telemetry POST before giving up.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

// ── Session ID ──────────────────────────────────────────────────────────────

/// Return a daily-rotated anonymous session ID.
///
/// The ID is a random UUID stored in `~/.lox/.telemetry_session`.  It is
/// regenerated when the date (UTC) changes so that events within a single day
/// can be correlated without providing a persistent identifier.
fn daily_session_id() -> String {
    static CACHED: OnceLock<String> = OnceLock::new();
    CACHED
        .get_or_init(|| {
            let dir = session_file_dir();
            let path = dir.join(".telemetry_session");
            let today = Utc::now().format("%Y-%m-%d").to_string();

            // Try to reuse today's session ID.
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut lines = content.lines();
                if let (Some(date), Some(id)) = (lines.next(), lines.next())
                    && date == today
                {
                    return id.to_string();
                }
            }

            // Generate a new session ID for today.
            let id = uuid::Uuid::new_v4().to_string();
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&path, format!("{}\n{}", today, id));
            id
        })
        .clone()
}

fn session_file_dir() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".lox"))
        .unwrap_or_else(|| PathBuf::from(".lox"))
}

// ── Telemetry ───────────────────────────────────────────────────────────────

/// Lightweight anonymous telemetry client.
pub struct Telemetry {
    enabled: bool,
}

#[allow(dead_code)]
impl Telemetry {
    /// Create a new telemetry instance.
    ///
    /// Resolution order (first match wins):
    /// 1. `LOX_TELEMETRY` env var: `"0"` / `"false"` → disabled; `"1"` / `"true"` → enabled.
    /// 2. `DO_NOT_TRACK` disables telemetry.
    /// 3. Config file `telemetry:` field.
    /// 4. Default: **enabled**.
    pub fn from_config(telemetry_opt: Option<bool>) -> Self {
        let enabled = resolve_enabled(telemetry_opt);
        Self { enabled }
    }

    /// Is telemetry active for this session?
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Track a named event with arbitrary properties.
    ///
    /// This is fire-and-forget: errors are silently ignored so that telemetry
    /// never interferes with CLI operation.
    pub fn track(&self, event: &str, extra: Value) {
        if !self.enabled {
            return;
        }

        let mut properties = base_properties();
        if let Value::Object(map) = extra {
            for (k, v) in map {
                properties
                    .as_object_mut()
                    .expect("base_properties is Object")
                    .insert(k, v);
            }
        }

        let body = json!({
            "api_key": POSTHOG_API_KEY,
            "event": event,
            "properties": properties,
            "timestamp": Utc::now().format("%Y-%m-%dT00:00:00Z").to_string(),
        });

        // Fire-and-forget in a background thread so the CLI never blocks.
        std::thread::spawn(move || {
            let _ = send_event(&body);
        });
    }

    /// Submit explicit feedback synchronously so delivery failures reach the caller.
    pub fn report_issue(&self, report_id: &str, title: &str, body: &str) -> Result<()> {
        self.report_issue_to(POSTHOG_ENDPOINT, report_id, title, body)
    }

    fn report_issue_to(
        &self,
        endpoint: &str,
        report_id: &str,
        title: &str,
        body: &str,
    ) -> Result<()> {
        if !self.enabled {
            bail!(
                "Telemetry is disabled. Enable it with `lox telemetry enable` \
                 and remove any telemetry opt-out environment override to allow fallback reports"
            );
        }
        let mut properties = base_properties();
        properties["report_id"] = json!(report_id);
        properties["title"] = json!(title);
        properties["body"] = json!(body);
        send_event_to(
            endpoint,
            &json!({
                "api_key": POSTHOG_API_KEY,
                "event": "issue_report",
                "properties": properties,
                "timestamp": Utc::now().format("%Y-%m-%dT00:00:00Z").to_string(),
            }),
        )
        .context("PostHog did not accept the issue report")
    }

    /// Track which top-level command was invoked (e.g. `"config"`, `"sim"`).
    pub fn track_command(&self, command: &str) {
        self.track("command", json!({ "command": command }));
    }

    /// Track anonymised config statistics.
    pub fn track_config_stats(&self, rooms: usize, blocks: usize, wires: usize) {
        self.track(
            "config_stats",
            json!({
                "rooms": rooms,
                "blocks": blocks,
                "wires": wires,
            }),
        );
    }

    /// Track the result of `lox config check`.
    pub fn track_check_result(&self, ok: usize, warnings: usize, errors: usize) {
        self.track(
            "config_check",
            json!({
                "ok": ok,
                "warnings": warnings,
                "errors": errors,
            }),
        );
    }

    /// Track which block type was created (e.g. `"And"`, `"LightController2"`).
    pub fn track_block_created(&self, block_type: &str) {
        self.track("block_created", json!({ "block_type": block_type }));
    }

    /// Track scan results (clean / issue count).
    pub fn track_scan_result(&self, issues: usize, warnings: usize) {
        self.track(
            "config_scan",
            json!({
                "issues": issues,
                "warnings": warnings,
            }),
        );
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Resolve whether telemetry should be enabled.
fn resolve_enabled(config_opt: Option<bool>) -> bool {
    // 1. Environment variable overrides everything.
    if let Ok(val) = std::env::var("LOX_TELEMETRY") {
        return matches!(val.as_str(), "1" | "true" | "yes");
    }
    // 2. DO_NOT_TRACK standard (https://consented.dev/)
    if std::env::var("DO_NOT_TRACK").is_ok() {
        return false;
    }
    // 3. Config value.
    // 4. Default: on (transparent — see TELEMETRY.md).
    config_opt.unwrap_or(true)
}

/// Common properties sent with every event (no PII).
fn base_properties() -> Value {
    json!({
        "distinct_id": daily_session_id(),
        "cli_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
}

/// POST a single event to PostHog.
fn send_event(body: &Value) -> Result<()> {
    send_event_to(POSTHOG_ENDPOINT, body)
}

fn send_event_to(endpoint: &str, body: &Value) -> Result<()> {
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let client = reqwest::blocking::Client::builder()
        .use_preconfigured_tls(tls)
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .json(body)
        .send()?;
    if !response.status().is_success() {
        bail!("PostHog returned HTTP {}", response.status());
    }
    Ok(())
}

// ── First-run notice ────────────────────────────────────────────────────────

/// Print a one-time notice about telemetry on first run.
///
/// The notice is shown when `~/.lox/.telemetry_noticed` does not exist.
/// It creates the marker file so the notice is only shown once.
pub fn maybe_show_first_run_notice() {
    let dir = session_file_dir();
    let marker = dir.join(".telemetry_noticed");
    if marker.exists() {
        return;
    }

    eprintln!(
        "\n\
         📊 Anonymous usage analytics help improve lox.\n\
         \x20  What's collected: command names, block types, config stats (no PII).\n\
         \x20  Full details: https://github.com/eisber/lox-cli/blob/main/TELEMETRY.md\n\
         \n\
         \x20  To enable:  lox telemetry enable\n\
         \x20  To disable: lox telemetry disable  (or LOX_TELEMETRY=0)\n\
         \n\
         \x20  Telemetry is ON by default.\n"
    );

    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(&marker, "shown\n");
}

// ── CLI command handlers ────────────────────────────────────────────────────

/// `lox telemetry enable`
pub fn cmd_telemetry_enable() -> anyhow::Result<()> {
    set_telemetry_preference(true)?;
    println!("✓ Telemetry enabled. Thank you for helping improve lox!");
    Ok(())
}

/// `lox telemetry disable`
pub fn cmd_telemetry_disable() -> anyhow::Result<()> {
    set_telemetry_preference(false)?;
    println!("✓ Telemetry disabled.");
    Ok(())
}

/// `lox telemetry status`
pub fn cmd_telemetry_status() -> anyhow::Result<()> {
    use crate::config::Config;

    let config_val = Config::telemetry_preference()?;

    let env_override = std::env::var("LOX_TELEMETRY").ok();
    let effective = resolve_enabled(config_val);

    println!("Telemetry status:");
    if let Some(ref env_val) = env_override {
        println!("  LOX_TELEMETRY env: {}", env_val);
    }
    match config_val {
        Some(true) => println!("  Config file:      enabled"),
        Some(false) => println!("  Config file:      disabled"),
        None => println!("  Config file:      not set (default: enabled)"),
    }
    println!(
        "  Effective:        {}",
        if effective { "enabled" } else { "disabled" }
    );
    Ok(())
}

/// Persist the telemetry preference to the config file.
fn set_telemetry_preference(enabled: bool) -> anyhow::Result<()> {
    use crate::config::{Config, GlobalConfig};

    let path = Config::path();

    // Try to load existing config.
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let value: serde_yaml::Value = serde_yaml::from_str(&content)?;

        if value.get("contexts").is_some() {
            // Multi-context format — store telemetry at root level.
            let mut global: GlobalConfig = serde_yaml::from_str(&content)?;
            global.telemetry = Some(enabled);
            global.save()?;
        } else {
            // Flat format.
            let mut cfg: Config = serde_yaml::from_str(&content)?;
            cfg.telemetry = Some(enabled);
            // Preserve data_dir for save()
            cfg.data_dir = Config::dir();
            cfg.save()?;
        }
    } else {
        // No config exists yet — create a minimal flat config.
        let dir = Config::dir();
        let _ = std::fs::create_dir_all(&dir);
        let cfg = Config {
            telemetry: Some(enabled),
            data_dir: dir,
            ..Default::default()
        };
        cfg.save()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_is_eu() {
        assert!(
            POSTHOG_ENDPOINT.contains("eu.i.posthog.com"),
            "PostHog endpoint must use EU region: {}",
            POSTHOG_ENDPOINT
        );
    }

    #[test]
    fn test_api_key_format() {
        assert!(
            POSTHOG_API_KEY.starts_with("phc_"),
            "API key must start with phc_"
        );
        assert!(POSTHOG_API_KEY.len() > 20, "API key too short");
    }

    #[test]
    fn test_resolve_enabled_default_on() {
        assert!(resolve_enabled(None));
    }

    #[test]
    fn test_resolve_enabled_config_true() {
        assert!(resolve_enabled(Some(true)));
    }

    #[test]
    fn test_resolve_enabled_config_false() {
        assert!(!resolve_enabled(Some(false)));
    }

    #[test]
    fn test_base_properties_has_required_fields() {
        let props = base_properties();
        assert!(props.get("distinct_id").is_some());
        assert!(props.get("cli_version").is_some());
        assert!(props.get("os").is_some());
        assert!(props.get("arch").is_some());
    }

    #[test]
    fn test_base_properties_no_pii() {
        let props = base_properties();
        let s = props.to_string();
        // Must not contain home dir, username, or hostname
        assert!(!s.contains("/home/"), "Properties must not leak home dir");
        assert!(!s.contains("hostname"), "Properties must not leak hostname");
    }

    #[test]
    fn test_track_noop_when_disabled() {
        let t = Telemetry { enabled: false };
        // Should not panic or make network calls
        t.track("test_event", json!({"key": "value"}));
        t.track_command("test");
        t.track_config_stats(5, 100, 200);
        t.track_check_result(10, 2, 0);
        t.track_block_created("And");
        t.track_scan_result(0, 1);
    }

    #[test]
    fn issue_report_waits_for_acceptance_and_includes_only_explicit_content() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/capture/").json_body_partial(
                json!({
                    "event": "issue_report",
                    "properties": {
                        "report_id": "synthetic-report",
                        "title": "Synthetic issue",
                        "body": "Expected 1, got 0",
                        "cli_version": env!("CARGO_PKG_VERSION"),
                    },
                })
                .to_string(),
            );
            then.status(200).json_body(json!({"status": 1}));
        });
        Telemetry { enabled: true }
            .report_issue_to(
                &server.url("/capture/"),
                "synthetic-report",
                "Synthetic issue",
                "Expected 1, got 0",
            )
            .unwrap();
        mock.assert();
    }

    #[test]
    fn issue_report_respects_opt_out_without_network_request() {
        let error = Telemetry { enabled: false }
            .report_issue_to("not a URL", "id", "title", "body")
            .unwrap_err();
        assert!(error.to_string().contains("Telemetry is disabled"));
    }

    #[test]
    fn issue_report_rejects_http_errors_and_redirects() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        for status in [302, 400, 503] {
            let path = format!("/{status}");
            let mock = server.mock(|when, then| {
                when.method(POST).path(&path);
                then.status(status)
                    .header("Location", server.url("/unexpected"));
            });
            let error = Telemetry { enabled: true }
                .report_issue_to(&server.url(&path), "id", "title", "body")
                .unwrap_err();
            assert!(format!("{error:#}").contains(&status.to_string()));
            mock.assert();
        }
    }

    #[test]
    fn test_event_body_structure() {
        let props = base_properties();
        let body = json!({
            "api_key": POSTHOG_API_KEY,
            "event": "test_event",
            "properties": props,
            "timestamp": Utc::now().format("%Y-%m-%dT00:00:00Z").to_string(),
        });
        assert_eq!(body["api_key"], POSTHOG_API_KEY);
        assert_eq!(body["event"], "test_event");
        assert!(body["properties"]["distinct_id"].is_string());
        // Timestamp should be date-only (privacy: no precise time)
        let ts = body["timestamp"].as_str().unwrap();
        assert!(
            ts.ends_with("T00:00:00Z"),
            "Timestamp must be date-only for privacy: {ts}"
        );
    }

    #[test]
    fn test_timeout_is_reasonable() {
        assert!(
            TIMEOUT.as_secs() <= 5,
            "Telemetry timeout must be ≤5s to avoid blocking CLI"
        );
    }
}
