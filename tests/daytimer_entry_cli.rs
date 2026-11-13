use std::fs;

use predicates::prelude::*;
use serde_json::Value;
use xmltree::Element;

const MODE_A: &str = "11111111-1111-1111-1111-111111111111";
const MODE_B: &str = "22222222-2222-2222-2222-222222222222";
const NATIVE_MODE_A: &str = "00000000-0000-0004-1500000000000000";
const NATIVE_MODE_B: &str = "00000000-0000-0005-1500000000000000";
const RFC_MODE_A: &str = "00000000-0000-0004-1500-000000000000";

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"DayTimer\" U=\"timer\" Title=\"Schedule\" Analog=\"false\" DefValue=\"0\" N=\"3\" Modes=\"{MODE_A},{MODE_B}\" WF=\"16384\" Unknown=\"keep\"><Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/><Co K=\"AQ\" U=\"output\"><In Input=\"wire\"/></Co><Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/><Entry From=\"120\" To=\"180\" V=\"5\" Marker=\"selected\"/><Unknown Preserve=\"yes\"/></C><C Type=\"DayTimer\" U=\"other\" Title=\"Other\" N=\"1\"><Entry To=\"300\" V=\"9\"/></C><C Type=\"Mode\" U=\"{MODE_A}\" Title=\"Mode A\"/><C Type=\"Mode\" U=\"{MODE_B}\" Title=\"Mode B\"/></C></Loxone>\r\n"
    )
}

fn command(input: &std::path::Path, from: &str, to: &str) -> assert_cmd::Command {
    command_for_mode(input, MODE_A, from, to)
}

fn command_for_mode(
    input: &std::path::Path,
    mode: &str,
    from: &str,
    to: &str,
) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-remove"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            mode,
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
        .stdout(format!("\"timer\",{MODE_A},{MODE_A},120,180,0\n"));
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

fn native_fixture(stored_mode: &str, object_mode: &str) -> String {
    fixture()
        .replace(
            &format!("Modes=\"{MODE_A},{MODE_B}\""),
            &format!("Modes=\"{stored_mode},{NATIVE_MODE_B}\""),
        )
        .replace(
            &format!("Type=\"Mode\" U=\"{MODE_A}\""),
            &format!("Type=\"Mode\" U=\"{object_mode}\""),
        )
        .replace(
            &format!("Type=\"Mode\" U=\"{MODE_B}\""),
            &format!("Type=\"Mode\" U=\"{NATIVE_MODE_B}\""),
        )
}

#[test]
fn daytimer_native_mode_ids_match_aliases_and_case_without_rewriting_spelling() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (stored, object, request) in [
        (NATIVE_MODE_A, NATIVE_MODE_A, NATIVE_MODE_A),
        (NATIVE_MODE_A, RFC_MODE_A, RFC_MODE_A),
        (
            "ABCDEF01-ABCD-0004-15000000ABCDEF00",
            "abcdef01-abcd-0004-1500-0000abcdef00",
            "aBcDeF01-AbCd-0004-15000000aBcDeF00",
        ),
    ] {
        let source = native_fixture(stored, object);
        fs::write(&input, &source).unwrap();
        let result = command_for_mode(&input, request, "02:00", "03:00")
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["removed"]["entry"]["mode_uuid"], request);
        assert_eq!(result["removed"]["stored_mode_uuid"], stored);
        assert_eq!(
            result["removed"]["removed_attributes"]["Marker"],
            "selected"
        );
        assert_eq!(result["removed"]["remaining_entries"], 2);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let mut expected = parse(source.as_bytes());
        let timer = expected
            .get_mut_child("C")
            .unwrap()
            .get_mut_child("C")
            .unwrap();
        timer.children.retain(|node| {
            node.as_element().is_none_or(|element| {
                element.attributes.get("Marker").map(String::as_str) != Some("selected")
            })
        });
        timer.attributes.insert("N".into(), "2".into());
        let mut actual = parse(&fs::read(&output).unwrap());
        for root in [&mut actual, &mut expected] {
            let document = root.get_mut_child("C").unwrap();
            document.attributes.shift_remove("Date");
            document.attributes.shift_remove("DateS");
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn daytimer_native_last_entry_and_mixed_alias_duplicates_keep_atomic_boundaries() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    let source = native_fixture(NATIVE_MODE_A, NATIVE_MODE_A)
        .replace("N=\"3\"", "N=\"1\"")
        .replace("<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>", "")
        .replace("<Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/>", "");
    fs::write(&input, &source).unwrap();
    command_for_mode(&input, RFC_MODE_A, "02:00", "03:00")
        .args(["-o", "csv", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(format!(
            "\"timer\",{RFC_MODE_A},{NATIVE_MODE_A},120,180,0\n"
        ));
    let saved = parse(&fs::read(&input).unwrap());
    let timer = saved.get_child("C").unwrap().get_child("C").unwrap();
    assert!(!timer.attributes.contains_key("N"));
    assert!(timer.get_child("Entry").is_none());
    assert_eq!(timer.attributes["DefValue"], "0");
    assert_eq!(
        timer.attributes["Modes"],
        format!("{NATIVE_MODE_A},{NATIVE_MODE_B}")
    );
    for invalid in [
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A).replace(
            &format!("Modes=\"{NATIVE_MODE_A},{NATIVE_MODE_B}\""),
            &format!("Modes=\"{NATIVE_MODE_A},{RFC_MODE_A}\""),
        ),
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A).replace(
            &format!("<C Type=\"Mode\" U=\"{NATIVE_MODE_A}\" Title=\"Mode A\"/>"),
            &format!("<C Type=\"Mode\" U=\"{NATIVE_MODE_A}\" Title=\"Mode A\"/><C Type=\"Mode\" U=\"{RFC_MODE_A}\"/>"),
        ),
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A)
            .replace(NATIVE_MODE_B, "00000000-0000-0000-0000000000000000"),
    ] {
        fs::write(&input, &invalid).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        command_for_mode(&input, NATIVE_MODE_A, "02:00", "03:00")
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), invalid.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    let missing = dir.path().join("missing.Loxone");
    for malformed in [
        "00000000000000041500000000000000",
        "urn:uuid:00000000-0000-0004-1500-000000000000",
        "{00000000-0000-0004-1500-000000000000}",
        "00000000-0000-0004-1500000000000000 ",
        "0000000-00000-0004-1500000000000000",
        "00000000-0000-0004-150000000000000g",
        "00000000-0000-0000-0000000000000000",
    ] {
        command_for_mode(&missing, malformed, "02:00", "03:00")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Cannot read").not());
    }
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
