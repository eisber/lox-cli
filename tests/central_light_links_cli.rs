use std::fs;

use predicates::prelude::*;
use serde_json::Value;

fn id(n: u8) -> String {
    format!("abcdef01-0002-0003-{n:016x}")
}

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList><C Type=\"Document\" U=\"{}\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"CentralLight\" U=\"{}\" Title=\"Central\" WF=\"16384\" linkC=\"{},{}\" Future=\"keep\"><IoData Visu=\"true\" Pr=\"{}\" Rating=\"7\"/><Unknown key=\"first\"/><Co K=\"I\" U=\"{}\"><In Input=\"{}\"/></Co><Unknown key=\"last\"/></C><C Type=\"Not\" U=\"{}\" Title=\"A\" WF=\"268451840\"/><C Type=\"Not\" U=\"{}\" Title=\"B\" WF=\"268435456\"/><C Type=\"Place\" U=\"{}\" Title=\"Office\"/><C Type=\"HVACController\" U=\"{}\" iObjS=\"unrelated\"/></C></ControlList>\r\n",
        id(1),
        id(2),
        id(10),
        id(11),
        id(30),
        id(20),
        id(21),
        id(10),
        id(11),
        id(30),
        id(40)
    )
}

#[test]
fn central_light_links_cli_moves_uuid_with_room_and_preserves_save_contract() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "central-light", "move-link"])
        .arg(&input)
        .args([
            "Central",
            &id(11).to_uppercase(),
            "up",
            "--room",
            "Office",
            "--save-as",
        ])
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
    assert_eq!(result["movement"]["from_position"], 2);
    assert_eq!(result["movement"]["to_position"], 1);
    assert_eq!(result["central_light"]["links"][0]["uuid"], id(11));
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|bytes| bytes == b"\r\n"));
    let saved = String::from_utf8(saved).unwrap();
    for retained in [
        format!("linkC=\"{},{}\"", id(11), id(10)),
        format!("Input=\"{}\"", id(21)),
        "Future=\"keep\"".into(),
        "Rating=\"7\"".into(),
        "WF=\"268451840\"".into(),
        "WF=\"268435456\"".into(),
        "iObjS=\"unrelated\"".into(),
    ] {
        assert!(saved.contains(&retained), "lost {retained}");
    }
    assert!(saved.find("key=\"first\"").unwrap() < saved.find("key=\"last\"").unwrap());
    assert!(!saved.contains("DateS=\"1\""));
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "central-light", "inspect"])
        .arg(&output)
        .args(["Central [Office]", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id(11)).and(predicate::str::contains("POSITION").not()));
}

#[test]
fn central_light_links_cli_boundaries_keep_original_bytes_and_support_csv() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    for (uuid, direction) in [(id(10), "up"), (id(11), "down")] {
        let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "central-light", "move-link"])
            .arg(&input)
            .args(["Central", &uuid, direction, "-o", "json"])
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
            .args(["config", "central-light", "move-link"])
            .arg(&input)
            .args(["Central", &uuid, direction, "--save-as"])
            .arg(&output)
            .args(["-o", "csv", "--no-header", "-q"])
            .assert()
            .success()
            .stderr("")
            .stdout(
                predicate::str::contains(format!("1,\"{}\",\"Not\",\"A\"", id(10)))
                    .and(predicate::str::contains("POSITION").not()),
            );
        assert_eq!(fs::read(&output).unwrap(), original.as_bytes());
    }
}

#[test]
fn central_light_links_cli_refusals_preserve_input_and_existing_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (source, selection, message) in [
        (fixture(), "1".into(), "nonzero native UUID"),
        (
            fixture().replace("DateS=\"1\">", "DateS=\"1\"><Template IfT=\"true\"/>"),
            id(11),
            "Document Template",
        ),
        (fixture().replace("268451840", "16384"), id(11), "WF marker"),
        (
            fixture().replace(
                &format!("linkC=\"{},{}\"", id(10), id(11)),
                &format!("linkC=\"{},{}\"", id(10), id(99)),
            ),
            id(11),
            "exactly one",
        ),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, "existing output").unwrap();
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "central-light", "move-link"])
            .arg(&input)
            .args(["Central", &selection, "up", "--save-as"])
            .arg(&output)
            .assert()
            .failure()
            .stderr(predicate::str::contains(message));
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read_to_string(&output).unwrap(), "existing output");
    }
}
