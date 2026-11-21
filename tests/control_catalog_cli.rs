use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use serde_json::{Value, json};
use std::fs;
use std::time::Duration;

#[test]
fn catalog_live_get_uses_top_level_projection_and_preserves_bypasses() {
    let root = tempfile::Builder::new()
        .prefix(".control-catalog-cli-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let server = MockServer::start();
    assert!(
        server
            .host()
            .parse::<std::net::IpAddr>()
            .unwrap()
            .is_loopback()
    );
    let uuid = "1fbc668c-005c-7471-ffffed57184a04d2";
    fs::create_dir(root.path().join("home")).unwrap();
    let config = root.path().join("config.yaml");
    fs::write(&config, format!(
        "host: {}\nuser: test\npass: synthetic-only\ntelemetry: false\naliases:\n  exact-alias: {uuid}\n",
        server.base_url()
    )).unwrap();
    let structure = server.mock(|when, then| {
        when.method(GET).path("/data/LoxApp3.json");
        then.json_body(json!({
            "rooms": {"r": {"name": "KÜCHE"}},
            "controls": {
                uuid: {"name": "Licht", "room": "r", "type": false,
                    "subControls": {"nested": {"name": "Licht", "room": "r"}}},
                "other": {"name": "Licht", "room": "unknown"}
            }
        }));
    });
    let state = server.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{uuid}"));
        then.json_body(json!({"LL": {"value": "1", "Code": "200"}}));
    });
    let unexpected = server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (query, expected_hits) in [(uuid, 0), ("exact-alias", 0), ("Licht [küche]", 1)] {
        let output = Command::cargo_bin("lox")
            .unwrap()
            .current_dir(root.path())
            .env("LOX_CONFIG", &config)
            .env("LOX_HOME", root.path().join("home"))
            .env("HOME", root.path().join("home"))
            .env("USERPROFILE", root.path().join("home"))
            .env("LOX_TELEMETRY", "0")
            .env_remove("HOMEDRIVE")
            .env_remove("HOMEPATH")
            .env_remove("LOX_HOST")
            .env_remove("LOX_USER")
            .env_remove("LOX_PASS")
            .env_remove("LOX_SERIAL")
            .env("HTTP_PROXY", server.base_url())
            .env("HTTPS_PROXY", server.base_url())
            .env("ALL_PROXY", server.base_url())
            .env("NO_PROXY", "127.0.0.1,localhost")
            .timeout(Duration::from_secs(15))
            .args(["--quiet", "--json", "live", "get", query])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .stdout
            .clone();
        assert_eq!(
            serde_json::from_slice::<Value>(&output).unwrap(),
            json!({"uuid": uuid, "value": "1", "code": "200"})
        );
        structure.assert_hits(expected_hits);
    }
    state.assert_hits(3);
    unexpected.assert_hits(0);
}
