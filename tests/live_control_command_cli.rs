use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use serde_json::{Value, json};
use std::{fs, time::Duration};

const ID: &str = "1fbc668c-005c-7471-ffffed57184a04d2";

struct Sandbox {
    root: tempfile::TempDir,
    server: MockServer,
}

impl Sandbox {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix(".control-command-cli-")
            .tempdir_in(env!("CARGO_MANIFEST_DIR"))
            .unwrap();
        let server = MockServer::start();
        fs::create_dir(root.path().join("home")).unwrap();
        fs::write(
            root.path().join("config.yaml"),
            format!(
                "host: {}\nuser: command\npass: synthetic-only\nverify_ssl: true\n\
                 telemetry: false\naliases:\n  lamp: '{ID}'\n  invalid: 'child/state'\n",
                server.base_url()
            ),
        )
        .unwrap();
        Self { root, server }
    }

    fn command(&self, query: &str, value: &str, flags: &[&str]) -> Command {
        let mut command = match std::env::var_os("LOX_COMMAND_ORACLE") {
            Some(binary) => Command::new(binary),
            None => Command::cargo_bin("lox").unwrap(),
        };
        command
            .current_dir(self.root.path())
            .env("LOX_CONFIG", self.root.path().join("config.yaml"))
            .env("LOX_HOME", self.root.path().join("home"))
            .env("HOME", self.root.path().join("home"))
            .env("USERPROFILE", self.root.path().join("home"))
            .env_remove("LOX_HOST")
            .env_remove("LOX_USER")
            .env_remove("LOX_PASS")
            .env_remove("LOX_SERIAL")
            .env("LOX_TELEMETRY", "0")
            .env("HTTP_PROXY", self.server.base_url())
            .env("HTTPS_PROXY", self.server.base_url())
            .env("ALL_PROXY", self.server.base_url())
            .env("NO_PROXY", "127.0.0.1,localhost")
            .timeout(Duration::from_secs(20))
            .args(["live", "set", query, value])
            .args(flags);
        command
    }
}

#[test]
fn write_gate_precedes_name_resolution_and_invalid_alias_precedes_send() {
    let sandbox = Sandbox::new();
    let unexpected = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for query in [ID, "lamp", "missing", "invalid"] {
        let assertion = sandbox
            .command(query, "pulse", &["--json"])
            .assert()
            .code(1);
        let error: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
        assert!(
            error["message"]
                .as_str()
                .unwrap()
                .contains("without --write")
        );
    }
    let assertion = sandbox
        .command("invalid", "pulse", &["--write", "--json"])
        .assert()
        .code(1);
    let error: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("Invalid characters in UUID: 'child/state'")
    );
    unexpected.assert_hits(0);
    fs::remove_file(sandbox.root.path().join("config.yaml")).unwrap();
    let assertion = sandbox.command(ID, "pulse", &["--json"]).assert().code(1);
    let error: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
    assert!(
        !error["message"]
            .as_str()
            .unwrap()
            .contains("without --write")
    );
    unexpected.assert_hits(0);
}

#[test]
fn actual_command_keeps_legacy_paths_codes_and_output_modes() {
    let sandbox = Sandbox::new();
    let response = sandbox.server.mock(|when, then| {
        when.method(GET)
            .path(format!("/jdev/sps/io/{ID}/on/child"))
            .query_param("flag", "1")
            .header("authorization", "Basic Y29tbWFuZDpzeW50aGV0aWMtb25seQ==");
        then.json_body(json!({"LL":{"value":42,"Code":401,"code":"odd"}}));
    });
    let sent = "on/child?flag=1#fragment";
    for query in [ID, "lamp"] {
        let assertion = sandbox
            .command(query, sent, &["--write", "--json"])
            .assert()
            .success()
            .stderr("");
        let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
        assert_eq!(
            result,
            json!({"uuid":ID,"sent":sent,"value":"42","code":"odd"})
        );
    }
    sandbox
        .command(ID, sent, &["--write", "--quiet"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    let assertion = sandbox.command(ID, sent, &["--write"]).assert().success();
    assert_eq!(
        String::from_utf8_lossy(&assertion.get_output().stdout),
        format!("✓ {ID} ← {sent}  (Code odd)\n  value: 42\n")
    );
    response.assert_hits(4);
}

#[test]
fn command_suffixes_keep_url_parser_utf8_space_escape_and_empty_behavior() {
    let sandbox = Sandbox::new();
    for (sent, suffix) in [
        ("", ""),
        ("hsv(120,100,30)", "hsv(120,100,30)"),
        ("a b", "a%20b"),
        ("grün", "gr%C3%BCn"),
        ("%2F", "%2F"),
        ("a\\b", "a/b"),
    ] {
        let mut response = sandbox.server.mock(|when, then| {
            when.method(GET).path(format!("/jdev/sps/io/{ID}/{suffix}"));
            then.json_body(json!({"LL":{"value":"ok","Code":200}}));
        });
        sandbox
            .command(ID, sent, &["--write", "--json"])
            .assert()
            .success();
        response.assert_hits(1);
        response.delete();
    }
}

#[test]
fn command_ll_projection_preserves_odd_shapes_and_numeric_code_precedence() {
    let sandbox = Sandbox::new();
    for (wire, value, code) in [
        (json!(null), "", None),
        (json!([]), "", None),
        (json!({"LL":null}), "", None),
        (json!({"LL":{"value":true,"Code":false}}), "", None),
        (json!({"LL":{"value":{},"Code":[]}}), "", None),
        (json!({"LL":{"value":"","Code":"401"}}), "", Some("401")),
        (
            json!({"LL":{"value":-0.0,"Code":200,"code":500}}),
            "-0",
            Some("200"),
        ),
        (json!({"LL":{"value":1.5,"code":200}}), "1.5", None),
        (json!({"LL":{"value":null,"Code":1.5}}), "", None),
        (
            json!({"LL":{"value":9007199254740993_u64,"Code":18446744073709551615_u64}}),
            "9007199254740992",
            None,
        ),
    ] {
        let mut response = sandbox.server.mock(|when, then| {
            when.method(GET).path(format!("/jdev/sps/io/{ID}/pulse"));
            then.json_body(wire);
        });
        for verbose in [false, true] {
            let mut command = sandbox.command(ID, "pulse", &["--write", "--json"]);
            if verbose {
                command.arg("-vv");
            }
            let assertion = command.assert().success();
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            assert_eq!(
                result,
                json!({"uuid":ID,"sent":"pulse","value":value,"code":code})
            );
        }
        response.assert_hits(2);
        response.delete();
    }
}

#[test]
fn real_status_decode_retry_and_verbose_errors_keep_head_categories() {
    let sandbox = Sandbox::new();
    for (status, body, verbose, hits, category, detail) in [
        (401, b"{".as_slice(), false, 1, "unauthorized", "HTTP 401"),
        (403, b"denied", true, 1, "forbidden", "HTTP 403"),
        (404, b"missing", false, 1, "not_found", "HTTP 404"),
        (503, b"unavailable", false, 4, "http_error", "HTTP 503"),
        (200, b"{", false, 4, "error", "error decoding response body"),
        (200, b"{", true, 4, "error", "EOF while parsing an object"),
        (
            200,
            b"{\"LL\":{\"value\":\"bad-\xff\"}}",
            false,
            4,
            "error",
            "error decoding response body",
        ),
    ] {
        let mut response = sandbox.server.mock(|when, then| {
            when.method(GET).path(format!("/jdev/sps/io/{ID}/pulse"));
            then.status(status).body(body);
        });
        for json in [false, true] {
            let mut command = sandbox.command(ID, "pulse", &["--write"]);
            if json {
                command.arg("--json");
            }
            if verbose {
                command.arg("-vv");
            }
            let assertion = command.assert().code(1);
            let output = assertion.get_output();
            let message = if json {
                let error: Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(error["error"], category);
                assert_eq!(error["ok"], false);
                error["message"].as_str().unwrap().to_owned()
            } else {
                assert!(output.stdout.is_empty());
                String::from_utf8_lossy(&output.stderr).into_owned()
            };
            assert!(
                message.contains(&format!("writing 'pulse' to {ID}")),
                "{message}"
            );
            assert!(message.contains(detail), "{message}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            if verbose {
                assert_eq!(stderr.matches("GET ").count(), 1);
                assert_eq!(stderr.matches("  → ").count(), hits);
                assert_eq!(stderr.contains("  body:"), status == 200);
            } else {
                assert!(!stderr.contains("  body:"));
            }
        }
        response.assert_hits(hits * 2);
        response.delete();
    }
}

#[test]
fn actual_cached_name_resolution_remains_head_owned_without_structure_request() {
    let sandbox = Sandbox::new();
    let config = fs::read_to_string(sandbox.root.path().join("config.yaml")).unwrap();
    fs::write(
        sandbox.root.path().join("config.yaml"),
        config.replace("127.0.0.1", "127.1"),
    )
    .unwrap();
    fs::create_dir(sandbox.root.path().join("cache")).unwrap();
    fs::write(
        sandbox.root.path().join("cache").join("structure.json"),
        json!({"controls":{ID:{"name":"Cached Lamp","type":"Switch"}},"rooms":{}}).to_string(),
    )
    .unwrap();
    let response = sandbox.server.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}/pulse"));
        then.json_body(json!({"LL":{"value":"ok","Code":200}}));
    });
    let structure = sandbox.server.mock(|when, then| {
        when.method(GET).path("/data/LoxApp3.json");
        then.status(500);
    });
    sandbox
        .command("Cached Lamp", "pulse", &["--write", "--json"])
        .assert()
        .success();
    response.assert_hits(1);
    structure.assert_hits(0);
}
