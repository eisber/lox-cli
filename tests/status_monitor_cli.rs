use std::fs;

use predicates::prelude::*;
use serde_json::Value;

fn id(index: u8) -> String {
    format!("abcdef01-0002-0003-{index:016x}")
}

fn fixture() -> String {
    let states: String = [4, 2, 0, 1, 3, 5, 6, 7, 8, 9, 10]
        .into_iter()
        .map(|index| {
            let value = if index == 10 {
                String::new()
            } else {
                format!(" Val=\"{index}\"")
            };
            format!(
                "<State Id=\"{}\" OIdx=\"{index}\" Na=\"State {index}\"{value} Cl=\"1,2,3\" Future=\"keep-{index}\"><Unknown keep=\"nested-{index}\"/></State>",
                id(index)
            )
        })
        .collect();
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"StatusMonitor\" U=\"monitor\" Title=\"Status\" WF=\"16384\"><Co K=\"AQ1\" U=\"connector\"/><IoData Pr=\"room\"/>{states}<Unknown keep=\"block\"/></C><C Type=\"Place\" U=\"room\" Title=\"Office\"/><C Type=\"StateV\" U=\"consumer\"><Co K=\"I\"><In Input=\"connector\"/></Co></C></C></Loxone>\r\n"
    )
}

#[test]
fn status_monitor_cli_move_uses_id_saves_and_preserves_output_references() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    let before = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "status-monitor", "inspect"])
        .arg(&input)
        .args(["Status", "--room", "Office", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let before: Value = serde_json::from_slice(&before).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "status-monitor", "move"])
        .arg(&input)
        .args(["Status [Office]", &id(2).to_uppercase(), "up", "--save-as"])
        .arg(&output)
        .args(["-o", "json", "-q"])
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["changed"], true);
    assert_eq!(result["movement"]["from_priority"], 2);
    assert_eq!(result["movement"]["to_priority"], 1);
    assert_eq!(result["status_monitor"]["states"][0]["id"], id(2));
    assert_eq!(result["status_monitor"]["states"][0]["output_index"], 2);
    assert_eq!(
        result["status_monitor"]["states"][0]["attributes"],
        before["status_monitor"]["states"][1]["attributes"]
    );
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let xml = String::from_utf8(saved).unwrap();
    for retained in [
        "Input=\"connector\"",
        "U=\"connector\"",
        "Future=\"keep-2\"",
        "keep=\"nested-2\"",
        "keep=\"block\"",
        "OIdx=\"10\"",
        "WF=\"16384\"",
    ] {
        assert!(xml.contains(retained), "lost {retained}");
    }
    assert!(!xml.contains("DateS=\"1\""));
}

#[test]
fn status_monitor_cli_boundary_noops_are_raw_byte_preserving_including_save_as() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    for (index, direction) in [(4, "up"), (9, "down")] {
        let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "status-monitor", "move"])
            .arg(&input)
            .args(["uuid:monitor", &id(index), direction, "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["changed"], false);
        assert!(result["saved_to"].is_null());
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "status-monitor", "move"])
            .arg(&input)
            .args(["uuid:monitor", &id(index), direction, "--save-as"])
            .arg(&output)
            .args(["-o", "csv", "-q", "--no-header"])
            .assert()
            .success()
            .stdout(
                predicate::str::contains(format!("1,\"{}\",4,\"State 4\"", id(4)))
                    .and(predicate::str::contains("PRIORITY").not()),
            );
        assert_eq!(fs::read(&output).unwrap(), original.as_bytes());
    }
}

#[test]
fn status_monitor_cli_refusals_do_not_touch_input_or_existing_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (source, selection, message) in [
        (fixture(), id(10), "Rest state"),
        (fixture(), id(99), "No Status Monitor state"),
        (
            fixture().replacen("Val=\"2\"", "Val=\"4\"", 1),
            id(2),
            "Colliding",
        ),
        (
            fixture().replacen("Type=\"StateV\"", "Type=\"StateV\" iObjS=\"monitor\"", 1),
            id(2),
            "iObjS",
        ),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, "existing output").unwrap();
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "status-monitor", "move"])
            .arg(&input)
            .args(["uuid:monitor", &selection, "up", "--save-as"])
            .arg(&output)
            .assert()
            .failure()
            .stderr(predicate::str::contains(message));
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read_to_string(&output).unwrap(), "existing output");
    }
}
