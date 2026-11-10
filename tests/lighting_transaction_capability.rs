use assert_cmd::Command;
use std::fs;

#[test]
fn lighting_transaction_diagnostics_are_visible_without_modifying_input_in_every_format() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-transaction-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    for (storage, code) in [
        (
            r#"<LightscenesC Num="0" Outputs="2" Future="retain"/>"#,
            "native_empty_mood_collection_reinitialized",
        ),
        (
            r#"<LightscenesC Num="1"><LightsceneC UUID="mood" SID="000" Outputs="2" Q1="00037"><SeqConf Future="retain"/></LightsceneC></LightscenesC>"#,
            "native_zero_mood_sid_reconciled",
        ),
    ] {
        let before = format!(
            "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList><C Type=\"LightController2\" U=\"lighting\">{storage}</C></ControlList>\r\n"
        )
        .into_bytes();
        fs::write(&input, &before).unwrap();
        for format in ["json", "csv", "table"] {
            let result = Command::cargo_bin("lox")
                .unwrap()
                .env("LOX_TELEMETRY", "0")
                .env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["config", "lighting", "inspect"])
                .arg(&input)
                .args(["uuid:lighting", "--quiet", "-o", format])
                .assert()
                .success()
                .get_output()
                .clone();
            if format == "json" {
                let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
                assert!(
                    value["diagnostics"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|diagnostic| diagnostic["code"].as_str() == Some(code))
                );
                assert_eq!(value["read_only"], true);
                assert_eq!(value["gui_verified"], false);
            } else {
                assert!(String::from_utf8_lossy(&result.stderr).contains(code));
            }
            assert_eq!(fs::read(&input).unwrap(), before);
        }
    }
}
