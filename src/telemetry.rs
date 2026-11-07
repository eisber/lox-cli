//! Anonymous usage telemetry and explicit issue reporting for improving `lox`.
//!
//! **Default: ON** — opt out with `lox telemetry disable`, `telemetry: false`
//! in the config, or `LOX_TELEMETRY=0`.
//!
//! Usage events exclude private data. Explicit feedback reports additionally
//! include the supplied issue title and body. See TELEMETRY.md for details.

use anyhow::{Context, Result, bail};
use chrono::Utc;
use serde_json::{Value, json};
use std::backtrace::{Backtrace, BacktraceStatus};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

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
    /// 2. `DO_NOT_TRACK` disables telemetry.
    /// 3. Config file `telemetry:` field.
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

    pub fn is_enabled(&self) -> bool {
        self.enabled
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

    /// Submit explicit feedback synchronously so delivery failures reach the caller.
    pub fn report_issue(&self, report_id: &str, title: &str, body: &str) -> Result<()> {
        if !self.enabled {
            bail!(
                "Telemetry is disabled. Enable it with `lox telemetry enable` \
                 and remove any telemetry opt-out environment override to allow fallback reports"
            );
        }
        let mut properties = base_properties(self.source);
        properties["report_id"] = json!(report_id);
        properties["title"] = json!(title);
        properties["body"] = json!(body);
        let response = post_event(
            &self.endpoint,
            &json!({
                "api_key": POSTHOG_API_KEY,
                "event": "issue_report",
                "properties": properties,
                "timestamp": Utc::now().format("%Y-%m-%dT00:00:00Z").to_string(),
            }),
            self.timeout,
        )
        .context("PostHog did not accept the issue report")?;
        if !response.status().is_success() {
            bail!("PostHog returned HTTP {}", response.status());
        }
        Ok(())
    }

    /// Track canonical command names and outcomes, never arguments or error text.
    pub fn track_command(&mut self, command: &str, error_type: Option<&str>, duration: Duration) {
        self.track(
            "command",
            json!({
                "command": command,
                "success": error_type.is_none(),
                "error_type": error_type,
                "duration_ms": duration.as_secs_f64() * 1000.0,
            }),
        );
    }

    pub fn track_exception(
        &mut self,
        command: &str,
        error_type: &str,
        handled: bool,
        backtrace: &Backtrace,
        duration: Duration,
    ) {
        if !self.enabled {
            return;
        }
        let frames = stack_frames(&backtrace.to_string());
        if frames.is_empty() && backtrace.status() == BacktraceStatus::Captured {
            eprintln!("Warning: exception backtrace contained no safe symbol frames.");
        }
        let mut exception = json!({
            "type": if handled { "RustError" } else { "RustPanic" },
            "value": format!("{command}: {error_type}"),
            "mechanism": {
                "type": "generic",
                "handled": handled,
                "synthetic": false,
            },
        });
        if !frames.is_empty() {
            exception["stacktrace"] = json!({ "type": "raw", "frames": frames });
        }
        self.track(
            "$exception",
            json!({
                "command": command,
                "error_type": error_type,
                "duration_ms": duration.as_secs_f64() * 1000.0,
                "$exception_level": "error",
                "$exception_list": [exception],
            }),
        );
    }

    pub fn install_panic_hook(&self, command: &str, started: Instant) {
        if !self.enabled {
            return;
        }
        // Initialize the ID before a hook could reenter its OnceLock initializer.
        daily_session_id();
        let source = self.source;
        let endpoint = self.endpoint.clone();
        let timeout = self.timeout;
        let command = command.to_owned();
        let command_thread = std::thread::current().id();
        let original = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if std::thread::current().name() != Some("lox-telemetry") {
                let duration = started.elapsed();
                let backtrace = Backtrace::force_capture();
                let mut telemetry = Self {
                    enabled: true,
                    source,
                    endpoint: endpoint.clone(),
                    timeout,
                    pending: Vec::new(),
                };
                if std::thread::current().id() == command_thread {
                    telemetry.track_command(&command, Some("panic"), duration);
                }
                telemetry.track_exception(&command, "panic", false, &backtrace, duration);
                // Drop waits for the bounded sends before the original panic continues.
                drop(telemetry);
            }
            original(info);
        }));
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

pub fn prepare_error_backtraces() {
    if std::env::var_os("RUST_LIB_BACKTRACE").is_none() && resolve_enabled(None) {
        // Called before threads/config parsing can create and cache the first backtrace.
        // Upload still uses the resolved config preference, even if it disables telemetry.
        unsafe { std::env::set_var("RUST_LIB_BACKTRACE", "1") };
    }
}

fn stack_frames(backtrace: &str) -> Vec<Value> {
    // Stable Backtrace exposes text, not structured frames. Accept numbered symbol
    // lines only; never transmit source-location lines or the original error text.
    let frames: Vec<_> = backtrace
        .lines()
        .filter_map(|line| {
            let (index, symbol) = line.trim().split_once(':')?;
            index.parse::<usize>().ok()?;
            let mut symbol = symbol.trim();
            if let Some((address, name)) = symbol.split_once(" - ")
                && address.strip_prefix("0x").is_some_and(|value| {
                    !value.is_empty() && value.chars().all(|c| c.is_ascii_hexdigit())
                })
            {
                symbol = name;
            }
            if let Some((name, hash)) = symbol.rsplit_once("::h")
                && hash.len() == 16
                && hash.chars().all(|c| c.is_ascii_hexdigit())
            {
                symbol = name;
            }
            if symbol.is_empty()
                || symbol == "<unknown>"
                || symbol.starts_with("0x")
                || symbol.contains(['/', '\\'])
                || symbol.contains("::telemetry::Telemetry::")
                || symbol.starts_with("std::backtrace")
                || symbol.starts_with("backtrace::")
            {
                return None;
            }
            Some(json!({
                "platform": "custom",
                "lang": "rust",
                "function": symbol.chars().take(512).collect::<String>(),
                "in_app": symbol.starts_with("lox::"),
            }))
        })
        .take(64)
        .collect();
    frames.into_iter().rev().collect()
}

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
        "source": "test",
        "$geoip_disable": true,
        "$process_person_profile": false,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "source": source,
        "geoip_country_only": true,
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
    post_event(endpoint, body, timeout)?.error_for_status()?;
    Ok(())
}

fn post_event(
    endpoint: &str,
    body: &Value,
    timeout: std::time::Duration,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .json(body)
        .send()
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
         \x20  What's collected: commands, timings, sanitized error stacks, country.\n\
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
        t.track_command("test", None, Duration::ZERO);
        t.track_config_stats(5, 100, 200);
        t.track_check_result(10, 2, 0);
        t.track_block_created("And");
        t.track_scan_result(0, 1);
        t.track_exception(
            "test",
            "error",
            true,
            &Backtrace::force_capture(),
            Duration::ZERO,
        );
        assert!(t.pending.is_empty());
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
        Telemetry::for_test(server.url("/capture/"))
            .report_issue("synthetic-report", "Synthetic issue", "Expected 1, got 0")
            .unwrap();
        mock.assert();
    }

    #[test]
    fn issue_report_respects_opt_out_without_network_request() {
        let mut telemetry = Telemetry::for_test("not a URL".into());
        telemetry.enabled = false;
        let error = telemetry.report_issue("id", "title", "body").unwrap_err();
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
            let error = Telemetry::for_test(server.url(&path))
                .report_issue("id", "title", "body")
                .unwrap_err();
            assert!(format!("{error:#}").contains(&status.to_string()));
            mock.assert();
        }
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
                r#"{"event":"command","properties":{"command":"config check","success":false,"error_type":"error","duration_ms":125.0,"source":"test","geoip_country_only":true,"$geoip_disable":true,"$process_person_profile":false}}"#,
            );
            then.status(200).body(r#"{"status":"Ok"}"#);
        });
        {
            let mut telemetry = Telemetry::for_test(server.url("/capture/"));
            telemetry.track_command("config check", Some("error"), Duration::from_millis(125));
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
        telemetry.track_command("config check", None, Duration::ZERO);
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
        telemetry.track_command("config check", None, Duration::ZERO);
        let start = std::time::Instant::now();
        let error = telemetry.flush().unwrap_err();
        assert!(error.downcast_ref::<reqwest::Error>().unwrap().is_timeout());
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
    }

    #[test]
    fn exception_frames_exclude_paths_addresses_and_build_hashes() {
        let frames = stack_frames(
            "0: lox::commands::config_cmd::load::h1234567890abcdef\n\
             at C:\\Users\\PRIVATE\\secret.Loxone:22\n\
             1: 0x1234 - lox::run_command\n\
             2: /home/PRIVATE/project/src/main.rs\n\
             3: C:\\Users\\PRIVATE\\source.rs\n\
             4: lox::telemetry::Telemetry::track_exception\n\
             5: <unknown>\n\
             6: 0x1234\n",
        );
        let names: Vec<_> = frames
            .iter()
            .map(|frame| frame["function"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            ["lox::run_command", "lox::commands::config_cmd::load"]
        );
        let payload = serde_json::to_string(&frames).unwrap();
        assert!(!payload.contains("PRIVATE"));
        assert!(!payload.contains("0x1234"));
        assert!(!payload.contains("h1234567890abcdef"));
    }

    #[test]
    fn exception_events_capture_error_origins_and_panics_without_messages() {
        use httpmock::prelude::*;

        for mode in ["error", "panic", "worker_panic"] {
            let server = MockServer::start();
            let exception = server.mock(|when, then| {
                when.method(POST).matches(|request| {
                    let Some(body) = request.body.as_deref() else {
                        return false;
                    };
                    let Ok(body) = std::str::from_utf8(body) else {
                        return false;
                    };
                    let Ok(event) = serde_json::from_str::<Value>(body) else {
                        return false;
                    };
                    let list = &event["properties"]["$exception_list"];
                    let frames = list[0]["stacktrace"]["frames"].as_array();
                    event["event"] == "$exception"
                        && event["properties"]["source"] == "test"
                        && !body.contains("PRIVATE")
                        && frames.is_some_and(|frames| {
                            frames.iter().any(|frame| {
                                frame["function"]
                                    .as_str()
                                    .is_some_and(|name| name.contains("failure_origin"))
                            })
                        })
                });
                then.status(200);
            });
            let command = server.mock(|when, then| {
                when.method(POST)
                    .json_body_partial(r#"{"event":"command","properties":{"success":false}}"#);
                then.status(200);
            });
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "telemetry::tests::exception_subprocess",
                    "--nocapture",
                ])
                .env("LOX_EXCEPTION_TEST_MODE", mode)
                .env("LOX_EXCEPTION_TEST_ENDPOINT", server.url("/capture/"))
                .env("RUST_LIB_BACKTRACE", "1")
                .env("RUST_BACKTRACE", "0")
                .output()
                .unwrap();
            assert_eq!(
                output.status.success(),
                mode != "panic",
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            exception.assert_hits(1);
            command.assert_hits(1);
        }
    }

    #[test]
    #[ignore = "subprocess fixture for exception transport and panic hooks"]
    fn exception_subprocess() {
        #[inline(never)]
        fn failure_origin(panic: bool) -> Result<()> {
            if panic {
                panic!("PRIVATE C:\\Users\\PRIVATE\\secret.Loxone token=PRIVATE");
            }
            bail!("PRIVATE C:\\Users\\PRIVATE\\secret.Loxone token=PRIVATE")
        }

        let mode = std::env::var("LOX_EXCEPTION_TEST_MODE").unwrap();
        let mut telemetry =
            Telemetry::for_test(std::env::var("LOX_EXCEPTION_TEST_ENDPOINT").unwrap());
        let started = Instant::now();
        if mode == "panic" {
            telemetry.install_panic_hook("config check", started);
            let _ = failure_origin(true);
        } else if mode == "worker_panic" {
            telemetry.install_panic_hook("config check", started);
            assert!(
                std::thread::spawn(|| {
                    let _ = failure_origin(true);
                })
                .join()
                .is_err()
            );
            telemetry.track_command("config check", Some("error"), started.elapsed());
        } else {
            assert_eq!(mode, "error");
            let error = failure_origin(false).unwrap_err();
            let duration = started.elapsed();
            telemetry.track_command("config check", Some("error"), duration);
            telemetry.track_exception("config check", "error", true, error.backtrace(), duration);
        }
    }
}
