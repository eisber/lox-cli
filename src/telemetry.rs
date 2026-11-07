//! Anonymous usage telemetry for improving `lox`.
//!
//! **Default: ON** — opt out with `lox telemetry disable`, `telemetry: false`
//! in the config, or `LOX_TELEMETRY=0`.
//!
//! Only anonymous, aggregated data is collected — never file paths, room names,
//! UUIDs, usernames, or any other PII.  See TELEMETRY.md for full details.

use chrono::Utc;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::thread::JoinHandle;

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

/// Lightweight, opt-in, anonymous telemetry client.
pub struct Telemetry {
    enabled: bool,
    source: &'static str,
    endpoint: String,
    timeout: std::time::Duration,
    pending: Vec<JoinHandle<Result<(), reqwest::Error>>>,
}

impl Telemetry {
    /// Create a new telemetry instance.
    ///
    /// Resolution order (first match wins):
    /// 1. `LOX_TELEMETRY` env var: `"0"` / `"false"` → disabled; `"1"` / `"true"` → enabled.
    /// 2. Config file `telemetry:` field.
    /// 3. `DO_NOT_TRACK` disables telemetry when no explicit override is set.
    /// 4. Default: **enabled**.
    pub fn from_config(telemetry_opt: Option<bool>) -> Self {
        let mut enabled = resolve_enabled(telemetry_opt);
        let source = match std::env::var("LOX_TELEMETRY_SOURCE").ok().as_deref() {
            None | Some("usage") => "usage",
            Some("test") => "test",
            Some("eval") => "eval",
            Some("diagnostic") => "diagnostic",
            Some(_) => {
                if enabled {
                    eprintln!(
                        "Warning: telemetry disabled: LOX_TELEMETRY_SOURCE must be usage, test, eval, or diagnostic."
                    );
                }
                enabled = false;
                "usage"
            }
        };
        Self {
            enabled,
            source,
            endpoint: POSTHOG_ENDPOINT.into(),
            timeout: TIMEOUT,
            pending: Vec::new(),
        }
    }

    /// Track a named event with arbitrary properties.
    ///
    /// Requests run concurrently and are joined on drop, including error exits.
    /// Delivery failures warn on stderr without changing the command's result.
    pub fn track(&mut self, event: &str, extra: Value) {
        if !self.enabled {
            return;
        }

        let mut properties = base_properties(self.source);
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

        let endpoint = self.endpoint.clone();
        let timeout = self.timeout;
        match std::thread::Builder::new()
            .name("lox-telemetry".into())
            .spawn(move || send_event(&endpoint, &body, timeout))
        {
            Ok(worker) => self.pending.push(worker),
            Err(error) => eprintln!("Warning: could not start telemetry sender: {error}"),
        }
    }

    /// Track canonical command names and outcomes, never arguments or error text.
    pub fn track_command(&mut self, command: &str, error_type: Option<&str>) {
        self.track(
            "command",
            json!({
                "command": command,
                "success": error_type.is_none(),
                "error_type": error_type,
            }),
        );
    }

    /// Track anonymised config statistics.
    pub fn track_config_stats(&mut self, rooms: usize, blocks: usize, wires: usize) {
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
    pub fn track_check_result(&mut self, ok: usize, warnings: usize, errors: usize) {
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
    pub fn track_block_created(&mut self, block_type: &str) {
        self.track("block_created", json!({ "block_type": block_type }));
    }

    /// Track scan results (clean / issue count).
    pub fn track_scan_result(&mut self, issues: usize, warnings: usize) {
        self.track(
            "config_scan",
            json!({
                "issues": issues,
                "warnings": warnings,
            }),
        );
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        let mut failure = None;
        for worker in self.pending.drain(..) {
            let result = match worker.join() {
                Ok(result) => result.map_err(anyhow::Error::from),
                Err(_) => Err(anyhow::anyhow!("telemetry sender panicked")),
            };
            if let Err(error) = result
                && failure.is_none()
            {
                failure = Some(error);
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(endpoint: String) -> Self {
        Self {
            enabled: true,
            source: "test",
            endpoint,
            timeout: TIMEOUT,
            pending: Vec::new(),
        }
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            eprintln!("Warning: anonymous telemetry was not delivered: {error:#}");
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Resolve whether telemetry should be enabled.
fn resolve_enabled(config_opt: Option<bool>) -> bool {
    enabled_from_values(
        config_opt,
        std::env::var("LOX_TELEMETRY").ok().as_deref(),
        std::env::var_os("DO_NOT_TRACK").is_some(),
    )
}

fn enabled_from_values(
    config_opt: Option<bool>,
    override_value: Option<&str>,
    do_not_track: bool,
) -> bool {
    // 1. Environment variable overrides everything.
    if let Some(val) = override_value {
        return matches!(val, "1" | "true" | "yes");
    }
    // 2. DO_NOT_TRACK standard (https://consented.dev/)
    if do_not_track {
        return false;
    }
    // 3. Config value.
    // 4. Default: on (transparent — see TELEMETRY.md).
    config_opt.unwrap_or(true)
}

/// Common properties sent with every event (no PII).
fn base_properties(source: &str) -> Value {
    json!({
        "distinct_id": daily_session_id(),
        "cli_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "source": source,
        "$geoip_disable": true,
        "$process_person_profile": false,
    })
}

/// POST a single event to PostHog.
fn send_event(
    endpoint: &str,
    body: &Value,
    timeout: std::time::Duration,
) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()?;
    client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .json(body)
        .send()?
        .error_for_status()?;
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
         \x20  Telemetry is ON by default; opt out at any time.\n"
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

    let config_val = Config::load().ok().and_then(|c| c.telemetry);

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
        assert!(enabled_from_values(None, None, false));
    }

    #[test]
    fn test_resolve_enabled_config_true() {
        assert!(enabled_from_values(Some(true), None, false));
    }

    #[test]
    fn test_resolve_enabled_config_false() {
        assert!(!enabled_from_values(Some(false), None, false));
    }

    #[test]
    fn test_base_properties_has_required_fields() {
        let props = base_properties("test");
        assert!(props.get("distinct_id").is_some());
        assert!(props.get("cli_version").is_some());
        assert!(props.get("os").is_some());
        assert!(props.get("arch").is_some());
    }

    #[test]
    fn test_base_properties_no_pii() {
        let props = base_properties("test");
        let s = props.to_string();
        // Must not contain home dir, username, or hostname
        assert!(!s.contains("/home/"), "Properties must not leak home dir");
        assert!(!s.contains("hostname"), "Properties must not leak hostname");
    }

    #[test]
    fn test_track_noop_when_disabled() {
        let mut t = Telemetry::for_test("http://127.0.0.1:1".into());
        t.enabled = false;
        // Should not panic or make network calls
        t.track("test_event", json!({"key": "value"}));
        t.track_command("test", None);
        t.track_config_stats(5, 100, 200);
        t.track_check_result(10, 2, 0);
        t.track_block_created("And");
        t.track_scan_result(0, 1);
    }

    #[test]
    fn test_event_body_structure() {
        let props = base_properties("test");
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

    #[test]
    fn telemetry_overrides_and_do_not_track_are_respected() {
        assert!(!enabled_from_values(Some(true), Some("0"), false));
        assert!(!enabled_from_values(Some(true), None, true));
        assert!(enabled_from_values(Some(false), Some("1"), false));
    }

    #[test]
    fn drop_delivers_command_before_returning() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let capture = server.mock(|when, then| {
            when.method(POST).path("/capture/").json_body_partial(
                r#"{"event":"command","properties":{"command":"config check","success":false,"error_type":"error","source":"test","$geoip_disable":true,"$process_person_profile":false}}"#,
            );
            then.status(200).body(r#"{"status":"Ok"}"#);
        });
        {
            let mut telemetry = Telemetry::for_test(server.url("/capture/"));
            telemetry.track_command("config check", Some("error"));
        }
        capture.assert_hits(1);
    }

    #[test]
    fn flush_reports_http_rejection() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let capture = server.mock(|when, then| {
            when.method(POST);
            then.status(400).body("invalid event");
        });
        let mut telemetry = Telemetry::for_test(server.url("/capture/"));
        telemetry.track_command("config check", None);
        assert!(telemetry.flush().unwrap_err().to_string().contains("400"));
        capture.assert_hits(1);
    }

    #[test]
    fn flush_times_out_instead_of_waiting_for_server() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST);
            then.status(200)
                .delay(std::time::Duration::from_millis(500));
        });
        let mut telemetry = Telemetry::for_test(server.url("/capture/"));
        telemetry.timeout = std::time::Duration::from_millis(50);
        telemetry.track_command("config check", None);
        let start = std::time::Instant::now();
        let error = telemetry.flush().unwrap_err();
        assert!(error.downcast_ref::<reqwest::Error>().unwrap().is_timeout());
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
    }
}
