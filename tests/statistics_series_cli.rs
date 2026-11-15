use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use serde_json::{Value, json};
use std::fs;
use std::time::Duration;

const UUID: &str = "0a0b0c0d-0003-0004-bbbb000000000002";

#[test]
fn monthly_statistics_cli_preserves_output_and_failure_envelopes() {
    for (xml, expected) in [
        (
            r#"<Statistics Name="Caf&#xE9; &amp; Bar" StatsGroup="2" Outputs="Power,">
<S T="not an epoch" V2="2e1" V="-0"/></Statistics>"#,
            Some(json!({
                "name": "Café & Bar", "group": 2, "outputs": ["Power", "V2"],
                "rows": [{"t": "not an epoch", "values": {"Power": -0.0, "V2": 20.0}}]
            })),
        ),
        (
            r#"<Statistics Outputs="Power"><S T="first" V="1"/><S T="later" V="warm"/></Statistics>"#,
            None,
        ),
    ] {
        let root = tempfile::Builder::new()
            .prefix(".statistics-series-cli-")
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
        fs::create_dir(root.path().join("home")).unwrap();
        let config = root.path().join("config.yaml");
        fs::write(
            &config,
            format!(
                "host: {}\nuser: test\npass: synthetic-only\ntelemetry: false\n",
                server.base_url()
            ),
        )
        .unwrap();
        let index = server.mock(|when, then| {
            when.method(GET).path("/stats/");
            then.body(format!("<ul><li><a href=\"{UUID}_2.202610.xml\">Kitchen (Statistic Group: 2) (Climate, Kitchen) 202610</a></li></ul>"));
        });
        let monthly = server.mock(|when, then| {
            when.method(GET).path(format!("/stats/{UUID}_2.202610.xml"));
            then.body(xml);
        });
        let unexpected = server.mock(|when, then| {
            when.any_request();
            then.status(500);
        });
        for format in ["json", "csv", "table"] {
            let mut command = Command::cargo_bin("lox").unwrap();
            let assertion = command
                .current_dir(root.path())
                .env("LOX_CONFIG", &config)
                .env("HOME", root.path().join("home"))
                .env("USERPROFILE", root.path().join("home"))
                .env("LOX_HOME", root.path().join("home"))
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
                .args(["--quiet", "-o", format, "stats", "get", "Kitchen"])
                .assert();
            let output = assertion.get_output();
            if let Some(expected) = &expected {
                assert!(output.status.success(), "{output:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
                match format {
                    "json" => assert_eq!(
                        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                        *expected
                    ),
                    "csv" => assert_eq!(output.stdout, b"T,Power,V2\nnot an epoch,-0,20\n"),
                    _ => {
                        let text = String::from_utf8_lossy(&output.stdout);
                        assert!(
                            text.contains("Power")
                                && text.contains("V2")
                                && text.contains("not an epoch"),
                            "{text}"
                        );
                    }
                }
            } else {
                assert_eq!(output.status.code(), Some(1), "{output:?}");
                let message = format!(
                    "parsing {UUID}_2.202610.xml: row later: V='warm' is not a number: invalid float literal"
                );
                if format == "json" {
                    assert!(output.stderr.is_empty(), "{output:?}");
                    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
                    assert_eq!(error["ok"], false);
                    assert_eq!(error["error"], "error");
                    assert_eq!(error["message"], message);
                    assert!(error.get("rows").is_none());
                } else {
                    assert!(output.stdout.is_empty(), "{output:?}");
                    assert!(
                        String::from_utf8_lossy(&output.stderr)
                            .starts_with(&format!("Error: {message}\n")),
                        "{output:?}"
                    );
                }
            }
        }
        index.assert_hits(3);
        monthly.assert_hits(3);
        unexpected.assert_hits(0);
    }
}
