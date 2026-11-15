//! Actual executable characterization, run before and after the runtime extraction.
use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use serde_json::{Value, json};
use std::{fs, time::Duration};

const ID: &str = "1fbc668c-005c-7471-ffffed57184a04d2";
const AUTH: &str = "Basic cmVhZDpzeW50aGV0aWMtb25seQ==";

struct Sandbox {
    root: tempfile::TempDir,
    server: MockServer,
}

impl Sandbox {
    fn new() -> Self {
        let sandbox = Self {
            root: tempfile::Builder::new()
                .prefix(".control-read-cli-")
                .tempdir_in(env!("CARGO_MANIFEST_DIR"))
                .unwrap(),
            server: MockServer::start(),
        };
        fs::create_dir(sandbox.root.path().join("home")).unwrap();
        sandbox.config(&sandbox.server.base_url(), ID);
        sandbox
    }

    fn config(&self, host: &str, alias: &str) {
        fs::write(
            self.root.path().join("config.yaml"),
            format!(
                "host: {host}\nuser: read\npass: synthetic-only\nverify_ssl: true\n\
                 telemetry: false\naliases:\n  opaque: '{alias}'\n"
            ),
        )
        .unwrap();
    }

    fn command(&self, query: &str, verbose: bool, json: bool) -> Command {
        let root = self.root.path();
        let mut command = Command::cargo_bin("lox").unwrap();
        command
            .current_dir(root)
            .env("LOX_CONFIG", root.join("config.yaml"))
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
            .timeout(Duration::from_secs(20))
            .args(["--quiet", "live", "get", query]);
        if verbose {
            command.arg("-vv");
        }
        if json {
            command.arg("--json");
        }
        command
    }
}

#[test]
fn all_valid_json_shapes_and_legacy_code_precedence_remain_admitted() {
    let sandbox = Sandbox::new();
    for (wire, value, code) in [
        (json!(null), "", None),
        (json!([]), "", None),
        (json!(false), "", None),
        (json!(42), "", None),
        (json!("root"), "", None),
        (json!({"LL":null}), "", None),
        (json!({"LL":[]}), "", None),
        (json!({"LL":{"value":true,"Code":false}}), "", None),
        (json!({"LL":{"value":{},"Code":[]}}), "", None),
        (json!({"LL":{"value":"","Code":"401"}}), "", Some("401")),
        (
            json!({"LL":{"value":"kept","Code":"403","code":"200"}}),
            "kept",
            Some("403"),
        ),
        (
            json!({"LL":{"value":42,"Code":401,"code":"500"}}),
            "42",
            Some("500"),
        ),
        (
            json!({"LL":{"value":-0.0,"Code":200,"code":500}}),
            "-0",
            Some("200"),
        ),
        (json!({"LL":{"value":1.5,"code":200}}), "1.5", None),
        (json!({"LL":{"value":null,"Code":1.5}}), "", None),
    ] {
        let mut response = sandbox.server.mock(|when, then| {
            when.method(GET)
                .path(format!("/jdev/sps/io/{ID}"))
                .header("authorization", AUTH)
                .header("user-agent", concat!("lox-cli/", env!("CARGO_PKG_VERSION")));
            then.json_body(wire);
        });
        for verbose in [false, true] {
            let assertion = sandbox.command(ID, verbose, true).assert().success();
            let output: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            assert_eq!(output, json!({"uuid":ID,"value":value,"code":code}));
        }
        response.assert_hits(2);
        response.delete();
    }
}

#[test]
fn literal_host_prefix_slash_and_opaque_alias_paths_are_not_hardened() {
    for (suffix, alias, path) in [
        ("", "child/state", "/jdev/sps/io/child/state"),
        ("/", "", "//jdev/sps/io/"),
        ("/prefix/", "x?flag=1", "/prefix//jdev/sps/io/x"),
    ] {
        let sandbox = Sandbox::new();
        sandbox.config(&format!("{}{suffix}", sandbox.server.base_url()), alias);
        let state = sandbox.server.mock(|when, then| {
            when.method(GET).path(path).header("authorization", AUTH);
            then.json_body(json!({"LL":{"value":"opaque","Code":200}}));
        });
        let unexpected = sandbox.server.mock(|when, then| {
            when.any_request();
            then.status(500);
        });
        let assertion = sandbox
            .command("opaque", false, true)
            .assert()
            .success()
            .stderr("");
        let output: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
        assert_eq!(output, json!({"uuid":alias,"value":"opaque","code":"200"}));
        state.assert_hits(1);
        unexpected.assert_hits(0);
    }
}

#[test]
fn status_decode_and_lossy_verbose_failures_keep_categories_and_atomic_output() {
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
        let sandbox = Sandbox::new();
        let state = sandbox.server.mock(|when, then| {
            when.method(GET).path(format!("/jdev/sps/io/{ID}"));
            then.status(status).body(body);
        });
        for json in [false, true] {
            let assertion = sandbox.command(ID, verbose, json).assert().code(1);
            let output = assertion.get_output();
            let message = if json {
                let error: Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(error["error"], category);
                assert_eq!(error["ok"], false);
                assert!(error.get("value").is_none());
                error["message"].as_str().unwrap().to_owned()
            } else {
                assert!(output.stdout.is_empty());
                String::from_utf8_lossy(&output.stderr).into_owned()
            };
            assert!(
                message.contains(&format!("reading state of {ID}")),
                "{message}"
            );
            assert!(message.contains(detail), "{message}");
            if verbose && status != 200 {
                assert!(!String::from_utf8_lossy(&output.stderr).contains("body:"));
            }
        }
        state.assert_hits(hits * 2);
    }
    let sandbox = Sandbox::new();
    let state = sandbox.server.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}"));
        then.body(b"{\"LL\":{\"value\":\"bad-\xff\",\"Code\":200}}");
    });
    let assertion = sandbox.command(ID, true, true).assert().success();
    let output = assertion.get_output();
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["value"], "bad-\u{fffd}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("body:"));
    state.assert_hits(1);
}

#[test]
fn quiet_still_prints_state_and_template_note_is_only_human() {
    let sandbox = Sandbox::new();
    let state = sandbox.server.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}"));
        then.json_body(json!({"LL":{"value":"<v.col> / <v>"}}));
    });
    let assertion = sandbox.command(ID, false, false).assert().success();
    assert_eq!(assertion.get_output().stdout, b"<v.col> / <v>\n");
    assert!(String::from_utf8_lossy(&assertion.get_output().stderr).starts_with("note:"));
    sandbox
        .command(ID, false, true)
        .assert()
        .success()
        .stderr("");
    state.assert_hits(2);
}
