//! Current CLI behavior, including permissive LL defaults, not a proposed runtime API.

use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use serde_json::{Value, json};
use std::fs;
use std::sync::Mutex;
use std::time::Duration;
use tempfile::TempDir;

const DATE: &str = "/jdev/sys/date";
const TIME: &str = "/jdev/sys/time";
const AUTH: &str = "Basic Y2xvY2stdGVzdDpzeW50aGV0aWMtb25seQ==";

// Only the ordered-success test installs these matchers; its cases run sequentially.
static REQUEST_ORDER: Mutex<Vec<&str>> = Mutex::new(Vec::new());

struct Sandbox {
    dir: TempDir,
    server: MockServer,
}

impl Sandbox {
    fn new() -> Self {
        let sandbox = Self {
            dir: tempfile::Builder::new()
                .prefix(".live-clock-cli-")
                .tempdir_in(env!("CARGO_MANIFEST_DIR"))
                .unwrap(),
            server: MockServer::start(),
        };
        assert!(
            sandbox
                .server
                .host()
                .parse::<std::net::IpAddr>()
                .unwrap()
                .is_loopback()
        );
        let root = sandbox.dir.path();
        fs::create_dir_all(root.join(".lox").join("cache")).unwrap();
        fs::create_dir(root.join("home")).unwrap();
        fs::write(
            root.join(".lox").join("config.yaml"),
            format!(
                "host: {}\nuser: clock-test\npass: synthetic-only\n\
                 verify_ssl: true\ntelemetry: false\n",
                sandbox.server.base_url()
            ),
        )
        .unwrap();
        sandbox
    }

    fn command(&self, json: bool) -> Command {
        let root = self.dir.path();
        let mut command = Command::cargo_bin("lox").unwrap();
        command
            .current_dir(root)
            .env("LOX_CONFIG", root.join(".lox").join("config.yaml"))
            .env("LOX_HOME", root.join("home"))
            .env("HOME", root.join("home"))
            .env("USERPROFILE", root.join("home"))
            .env_remove("HOMEDRIVE")
            .env_remove("HOMEPATH")
            .env_remove("LOX_HOST")
            .env_remove("LOX_USER")
            .env_remove("LOX_PASS")
            .env_remove("LOX_SERIAL")
            .env("LOX_TELEMETRY", "0")
            .env("HTTP_PROXY", self.server.base_url())
            .env("HTTPS_PROXY", self.server.base_url())
            .env("ALL_PROXY", self.server.base_url())
            .env("NO_PROXY", "127.0.0.1,localhost")
            .timeout(Duration::from_secs(15))
            // Quiet suppresses the first-run telemetry notice, not the clock output.
            .args(["--quiet", "live", "time"]);
        if json {
            command.arg("--json");
        }
        command
    }

    fn assert_no_local_state(&self) {
        let root = self.dir.path();
        assert_eq!(fs::read_dir(root.join("home")).unwrap().count(), 0);
        assert_eq!(
            fs::read_dir(root.join(".lox").join("cache"))
                .unwrap()
                .count(),
            0
        );
        assert_eq!(fs::read_dir(root.join(".lox")).unwrap().count(), 2);
    }
}

fn assert_clock_output(stdout: &[u8], as_json: bool, date: &str, time: &str) {
    if as_json {
        assert_eq!(
            serde_json::from_slice::<Value>(stdout).unwrap(),
            json!({"date": date, "time": time})
        );
        assert_eq!(stdout.iter().filter(|&&byte| byte == b'\n').count(), 1);
    } else {
        assert_eq!(stdout, format!("{date} {time}\n").as_bytes());
    }
}

#[test]
fn clock_reads_date_before_time_and_formats_string_and_numeric_values() {
    for (date, time, expected_date, expected_time) in [
        (
            json!("2026-11-14"),
            json!("22:23:00"),
            "2026-11-14",
            "22:23:00",
        ),
        (json!(1234), json!(56.25), "1234", "56.25"),
    ] {
        for as_json in [false, true] {
            let sandbox = Sandbox::new();
            REQUEST_ORDER.lock().unwrap().clear();
            let date_request = sandbox.server.mock(|when, then| {
                when.method(GET)
                    .path(DATE)
                    .header("authorization", AUTH)
                    .matches(|request| {
                        if request.path == DATE {
                            REQUEST_ORDER.lock().unwrap().push(DATE);
                        }
                        true
                    });
                then.json_body(json!({"LL": {"value": date, "Code": "200"}}));
            });
            let time_request = sandbox.server.mock(|when, then| {
                when.method(GET)
                    .path(TIME)
                    .header("authorization", AUTH)
                    .matches(|request| {
                        if request.path == TIME {
                            REQUEST_ORDER.lock().unwrap().push(TIME);
                        }
                        true
                    });
                then.json_body(json!({"LL": {"value": time, "Code": 200}}));
            });
            let unexpected = sandbox.server.mock(|when, then| {
                when.any_request();
                then.status(500);
            });

            let output = sandbox
                .command(as_json)
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            assert_clock_output(&output, as_json, expected_date, expected_time);
            assert_eq!(*REQUEST_ORDER.lock().unwrap(), [DATE, TIME]);
            date_request.assert_hits(1);
            time_request.assert_hits(1);
            unexpected.assert_hits(0);
            sandbox.assert_no_local_state();
        }
    }
}

#[test]
fn clock_currently_defaults_missing_values_and_ignores_protocol_codes() {
    for (date, time, expected_date, expected_time) in [
        (json!({}), json!({"LL": {"Code": "503"}}), "", ""),
        (
            json!({"LL": {"value": true, "Code": 200}}),
            json!({"LL": {"value": null, "code": "200"}}),
            "",
            "",
        ),
        (
            json!({"LL": {"value": "kept-date", "Code": "401"}}),
            json!({"LL": {"value": 42, "code": "500"}}),
            "kept-date",
            "42",
        ),
        (
            json!({"LL": {"value": 1.5, "Code": 403}}),
            json!({"LL": {"value": "kept-time"}}),
            "1.5",
            "kept-time",
        ),
    ] {
        for as_json in [false, true] {
            let sandbox = Sandbox::new();
            let date_request = sandbox.server.mock(|when, then| {
                when.method(GET).path(DATE).header("authorization", AUTH);
                then.json_body(date.clone());
            });
            let time_request = sandbox.server.mock(|when, then| {
                when.method(GET).path(TIME).header("authorization", AUTH);
                then.json_body(time.clone());
            });
            let unexpected = sandbox.server.mock(|when, then| {
                when.any_request();
                then.status(500);
            });

            let output = sandbox
                .command(as_json)
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            assert_clock_output(&output, as_json, expected_date, expected_time);
            date_request.assert_hits(1);
            time_request.assert_hits(1);
            unexpected.assert_hits(0);
            sandbox.assert_no_local_state();
        }
    }
}

#[test]
fn clock_http_and_json_failures_preserve_retry_counts_and_stop_without_partial_output() {
    for (failing_path, status, body, attempts, category, message) in [
        (DATE, 401, "denied", 1, "unauthorized", "HTTP 401"),
        (DATE, 503, "unavailable", 4, "http_error", "HTTP 503"),
        (DATE, 200, "{", 4, "error", "error decoding response body"),
        (TIME, 403, "denied", 1, "forbidden", "HTTP 403"),
        (TIME, 200, "{", 4, "error", "error decoding response body"),
    ] {
        for as_json in [false, true] {
            let sandbox = Sandbox::new();
            let date_request = sandbox.server.mock(|when, then| {
                when.method(GET).path(DATE).header("authorization", AUTH);
                if failing_path == DATE {
                    then.status(status).body(body);
                } else {
                    then.json_body(json!({"LL": {"value": "2026-11-14"}}));
                }
            });
            let time_request = sandbox.server.mock(|when, then| {
                when.method(GET).path(TIME).header("authorization", AUTH);
                if failing_path == TIME {
                    then.status(status).body(body);
                } else {
                    then.json_body(json!({"LL": {"value": "22:23:00"}}));
                }
            });
            let unexpected = sandbox.server.mock(|when, then| {
                when.any_request();
                then.status(500);
            });

            let assertion = sandbox.command(as_json).assert().code(1);
            let output = assertion.get_output();
            if as_json {
                assert!(output.stderr.is_empty(), "{output:?}");
                let error: Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(error["ok"], false);
                assert_eq!(error["error"], category);
                assert!(error["message"].as_str().unwrap().contains(message));
                assert!(error.get("date").is_none());
                assert!(error.get("time").is_none());
            } else {
                assert!(output.stdout.is_empty(), "{output:?}");
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(stderr.starts_with("Error: "), "{stderr}");
                assert!(stderr.contains(message), "{stderr}");
            }
            if status != 200 {
                let error_output = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(error_output.contains(failing_path), "{error_output}");
            }
            date_request.assert_hits(if failing_path == DATE { attempts } else { 1 });
            time_request.assert_hits(if failing_path == TIME { attempts } else { 0 });
            unexpected.assert_hits(0);
            sandbox.assert_no_local_state();
        }
    }
}
