use std::fs;

use serde_json::Value;

fn fixture() -> String {
    let loads: String = (0..12)
        .map(|index| {
            format!(
                "<Load{} nm=\"Consumer {index}\" pwr=\"2.5\" maxOff=\"123\" minOn=\"45\" Unknown=\"keep-{index}\"/>",
                if index == 0 { String::new() } else { format!(" ioIdx=\"{index}\"") }
            )
        })
        .collect();
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"LoadShed\" U=\"manager\" Title=\"Load Manager\" WF=\"16384\"><Co K=\"AQl1\" U=\"connector\"/><IoData Pr=\"room\"/>{loads}<Unknown U=\"keep\"/></C><C Type=\"Place\" U=\"room\" Title=\"Office\"/></C></Loxone>\r\n"
    )
}

#[test]
fn load_manager_cli_saves_priority_with_identity_and_reloads() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().into_bytes();
    fs::write(&input, &original).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["Load Manager", "0", "down", "--room", "Office", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["load_manager"]["loads"][1]["io_index"], 0);
    assert_eq!(
        result["load_manager"]["loads"][1]["attributes"]["Unknown"],
        "keep-0"
    );
    assert!(
        result["load_manager"]["loads"][1]["attributes"]
            .get("ioIdx")
            .is_none()
    );
    assert_eq!(fs::read(&input).unwrap(), original);
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let xml = String::from_utf8(saved).unwrap();
    assert!(xml.contains("U=\"connector\""));
    assert!(xml.contains("WF=\"16384\""));
    assert!(!xml.contains("DateS=\"1\""));
    let inspection = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&output)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspection: Value = serde_json::from_slice(&inspection).unwrap();
    assert_eq!(inspection["load_manager"], result["load_manager"]);
    assert!(inspection["saved_to"].is_null());
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&output)
        .args(["uuid:manager", "0", "up", "-o", "json"])
        .assert()
        .success();
    let inspection = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&output)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspection: Value = serde_json::from_slice(&inspection).unwrap();
    assert_eq!(inspection["load_manager"]["loads"][0]["io_index"], 0);
    assert!(
        inspection["load_manager"]["loads"][0]["attributes"]
            .get("ioIdx")
            .is_none()
    );
}

#[test]
fn load_manager_cli_errors_do_not_touch_input_or_existing_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().into_bytes();
    fs::write(&input, &original).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for (selector, index, direction) in [
        ("uuid:manager", "0", "up"),
        ("uuid:manager", "11", "down"),
        ("uuid:manager", "12", "up"),
        ("uuid:room", "4", "up"),
        ("missing", "4", "down"),
    ] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "move"])
            .arg(&input)
            .args([selector, index, direction, "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    let malformed = fixture().replacen("ioIdx=\"1\"", "ioIdx=\"0\"", 1);
    fs::write(&input, malformed.as_bytes()).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["uuid:manager", "4", "up", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("duplicate ioIdx"));
    assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
}
