use std::fs;

use predicates::prelude::*;
use serde_json::Value;
use xmltree::Element;

const MODE_A: &str = "11111111-1111-1111-1111-111111111111";
const MODE_B: &str = "22222222-2222-2222-2222-222222222222";

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"DayTimer\" U=\"timer\" Title=\"Schedule\" Analog=\"false\" DefValue=\"0\" N=\"3\" Modes=\"{MODE_A},{MODE_B}\" WF=\"16384\" Unknown=\"keep\"><Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/><Co K=\"AQ\" U=\"output\"><In Input=\"wire\"/></Co><Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/><Entry From=\"120\" To=\"180\" V=\"5\" Marker=\"selected\"/><Unknown Preserve=\"yes\"/></C><C Type=\"DayTimer\" U=\"other\" Title=\"Other\" N=\"1\"><Entry To=\"300\" V=\"9\"/></C><C Type=\"Mode\" U=\"{MODE_A}\" Title=\"Mode A\"/><C Type=\"Mode\" U=\"{MODE_B}\" Title=\"Mode B\"/></C></Loxone>\r\n"
    )
}

fn command(input: &std::path::Path, from: &str, to: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-remove"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            MODE_A,
            "--from",
            from,
            "--to",
            to,
        ]);
    command
}

fn parse(bytes: &[u8]) -> Element {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    Element::parse(bytes).unwrap()
}

#[test]
fn daytimer_delete_selects_mode_and_interval_not_xml_order_and_preserves_survivors() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    let result = command(&input, "02:00", "03:00")
        .arg("--save-as")
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["removed"]["remaining_entries"], 2);
    assert_eq!(result["removed"]["entry"]["mode_uuid"], MODE_A);
    assert_eq!(
        result["removed"]["removed_attributes"]["Marker"],
        "selected"
    );
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(
        saved
            .iter()
            .enumerate()
            .all(|(index, byte)| { *byte != b'\n' || (index > 0 && saved[index - 1] == b'\r') })
    );
    let mut expected = parse(original.as_bytes());
    let document = expected.get_mut_child("C").unwrap();
    let timer = document.get_mut_child("C").unwrap();
    timer.children.retain(|node| {
        node.as_element().is_none_or(|element| {
            element.attributes.get("Marker").map(String::as_str) != Some("selected")
        })
    });
    timer.attributes.insert("N".into(), "2".into());
    let mut actual = parse(&saved);
    for root in [&mut actual, &mut expected] {
        let doc = root.get_mut_child("C").unwrap();
        doc.attributes.shift_remove("Date");
        doc.attributes.shift_remove("DateS");
    }
    assert_eq!(actual, expected);
    assert!(!String::from_utf8(saved).unwrap().contains("DateS=\"1\""));
}

#[test]
fn daytimer_delete_last_entry_omits_zero_count_and_keeps_default_and_mode_references() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = fixture()
        .replace("N=\"3\"", "N=\"1\"")
        .replace(
            "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
            "",
        )
        .replace(
            "<Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/>",
            "",
        );
    fs::write(&input, &source).unwrap();
    command(&input, "02:00", "03:00")
        .args(["-o", "csv", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(format!("\"timer\",{MODE_A},120,180,0\n"));
    let saved = parse(&fs::read(&input).unwrap());
    let timer = saved.get_child("C").unwrap().get_child("C").unwrap();
    assert!(!timer.attributes.contains_key("N"));
    assert_eq!(timer.attributes["DefValue"], "0");
    assert_eq!(timer.attributes["Modes"], format!("{MODE_A},{MODE_B}"));
    assert!(timer.get_child("Entry").is_none());
    let before = fs::read(&input).unwrap();
    command(&input, "02:00", "03:00").assert().failure();
    assert_eq!(fs::read(&input).unwrap(), before);
}

#[test]
fn daytimer_delete_rejects_unsafe_storage_without_touching_input_or_existing_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    for source in [
        fixture().replace("N=\"3\"", "N=\"2\""),
        fixture().replace("Type=\"DayTimer\" U=\"timer\"", "Type=\"And\" U=\"timer\""),
        fixture().replace("N=\"3\"", "M=\"4\" N=\"3\""),
        fixture().replace("From=\"120\" To=\"180\"", "From=\"600\" To=\"660\""),
        fixture().replace("From=\"120\" To=\"180\"", "From=\"660\" To=\"720\""),
        fixture().replace(
            "V=\"5\" Marker=\"selected\"",
            "Ix=\"2\" V=\"5\" Marker=\"selected\"",
        ),
        fixture().replace(
            &format!("Type=\"Mode\" U=\"{MODE_B}\""),
            &format!("Type=\"And\" U=\"{MODE_B}\""),
        ),
        fixture().replace("Analog=\"false\"", "Analog=\"true\""),
        fixture().replace(" N=\"3\"", ""),
        fixture().replace(
            &format!("Modes=\"{MODE_A},{MODE_B}\""),
            &format!("Modes=\"{MODE_A},{MODE_A}\""),
        ),
        fixture().replace(
            &format!("Modes=\"{MODE_A},{MODE_B}\""),
            &format!("Modes=\"urn:uuid:{MODE_A},{MODE_B}\""),
        ),
        fixture().replace(
            &format!("Type=\"Mode\" U=\"{MODE_A}\""),
            &format!("Type=\"Mode\" U=\"urn:uuid:{MODE_A}\""),
        ),
        fixture().replace(MODE_B, "00000000-0000-0000-0000-000000000000"),
        fixture().replace(" Modes=", " UnsupportedModes="),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        command(&input, "02:00", "03:00")
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
}

#[test]
fn daytimer_delete_uses_omitted_zero_identity_and_dry_run_fails_closed_before_io() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for source in [
        fixture(),
        fixture().replace(" Analog=\"false\" DefValue=\"0\"", ""),
    ] {
        let source = source.replace("From=\"120\" To=\"180\" V=\"5\"", "To=\"180\" V=\"5\"");
        fs::write(&input, source).unwrap();
        command(&input, "00:00", "03:00")
            .arg("-q")
            .assert()
            .success()
            .stdout("");
    }
    let missing = dir.path().join("missing.Loxone");
    command(&missing, "00:00", "03:00")
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("dry-run not supported"))
        .stderr(predicate::str::contains("Cannot read").not());
    for (from, to) in [("2:00", "03:00"), ("23:00", "01:00"), ("24:01", "24:02")] {
        command(&missing, from, to)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Cannot read").not());
    }
}
