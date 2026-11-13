use std::{fs, path::Path};

use serde_json::Value;
use xmltree::Element;

const STANDARD: &str = "00000000-0000-0001-1500000000000000";
const SELECTED: &str = "00000000-0000-0004-1500000000000000";
const RFC_SELECTED: &str = "00000000-0000-0004-1500-000000000000";
const LAST: &str = "00000000-0000-0005-1500000000000000";
const RFC_LAST: &str = "00000000-0000-0005-1500-000000000000";
const RFC_STANDARD: &str = "00000000-0000-0001-1500-000000000000";

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"Place\" U=\"bed\" Title=\"Bedroom\"/><C Type=\"Place\" U=\"other-room\" Title=\"Other\"/><C Type=\"AlarmClock\" V=\"175\" U=\"clock\" Title=\"Clock\" N=\"3\" Dev=\"device\" Ode=\"7\" Le=\"7\" Unknown=\"keep\"><IoData Pr=\"bed\"/><AlarmClock U=\"{STANDARD}\" Int=\"true\" Time=\"21600\" Modes=\"1023\" Name=\"Standard\"/><Co K=\"AQ\" U=\"wire\"><In Input=\"source\"/></Co><AlarmClock U=\"{SELECTED}\" Id=\"7\" Time=\"25200\" Modes=\"248\" Active=\"true\" Sa=\"true\" Name=\"Same name\" Marker=\"selected\"/><Entry U=\"{SELECTED}\" Marker=\"auxiliary-reference\"/><Unknown Preserve=\"yes\"/><AlarmClock U=\"{LAST}\" Id=\"12\" Time=\"28800\" Modes=\"1023\" Name=\"Same name\" Marker=\"survivor\"/></C><C Type=\"AlarmClock\" V=\"175\" U=\"other-clock\" Title=\"Clock\" N=\"1\"><IoData Pr=\"other-room\"/><AlarmClock U=\"other-standard\" Int=\"true\" Time=\"21600\" Modes=\"1023\"/></C></C></Loxone>\r\n"
    )
}

fn command(input: &Path, selector: &str, uuid: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-remove"])
        .arg(input)
        .args([selector, "--entry-uuid", uuid]);
    command
}

fn active_command(input: &Path, selector: &str, uuid: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-set-active"])
        .arg(input)
        .args([selector, "--entry-uuid", uuid]);
    command
}

fn mode_command(input: &Path, selector: &str, uuid: &str, bit: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-set-mode"])
        .arg(input)
        .args([selector, "--entry-uuid", uuid, "--mode-bit", bit]);
    command
}

fn add_command(input: &Path, selector: &str, name: &str, time: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-add"])
        .arg(input)
        .args([selector, "--name", name, "--time", time]);
    command
}

fn time_command(input: &Path, selector: &str, uuid: &str, time: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-set-time"])
        .arg(input)
        .args([selector, "--entry-uuid", uuid, "--time", time]);
    command
}

fn name_command(input: &Path, selector: &str, uuid: &str, name: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-set-name"])
        .arg(input)
        .args([selector, "--entry-uuid", uuid, "--name", name]);
    command
}

fn parse(bytes: &[u8]) -> Element {
    Element::parse(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes)).unwrap()
}

fn clock(root: &mut Element) -> &mut Element {
    root.get_mut_child("C")
        .unwrap()
        .children
        .iter_mut()
        .filter_map(|node| node.as_mut_element())
        .find(|element| element.attributes.get("U").map(String::as_str) == Some("clock"))
        .unwrap()
}

#[test]
fn alarm_name_edit_preserves_identity_order_and_all_other_data_and_allows_duplicate_unicode_names()
{
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (old, name, crlf) in [
        ("Different", "Same name", true),
        ("Same name", " früh & \"Spät\" <🚀> ", false),
        ("Same name", "same name", true),
        ("Cafe\u{301}", "Café", false),
    ] {
        let mut source = fixture()
            .replace("Time=\"25200\"", "Time=\"+00025200\"")
            .replace("Active=\"true\"", "Active=\"FaLsE\"")
            .replace(
                "<Unknown Preserve=\"yes\"/>",
                "<Unknown Preserve='yes' Numeric=\"&#65;\" Literal=\"line&#xA;break\" Duplicate=\"same\" Duplicate=\"same\"/>",
            )
            .replacen(
                "Name=\"Same name\" Marker=\"selected\"",
                &format!("Name=\"{old}\" Marker=\"selected\""),
                1,
            );
        if !crlf {
            source = source
                .replace("\r\n", "\n")
                .trim_start_matches('\u{feff}')
                .into();
        }
        fs::write(&input, &source).unwrap();
        let result = name_command(&input, "Clock", RFC_SELECTED, name)
            .args(["--room", "Bedroom", "--save-as"])
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["name_change"]["entry_uuid"], RFC_SELECTED);
        assert_eq!(result["name_change"]["stored_entry_uuid"], SELECTED);
        assert_eq!(result["name_change"]["previous_name"], old);
        assert_eq!(result["name_change"]["name"], name);
        assert_eq!(result["name_change"]["changed"], true);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let saved = fs::read(&output).unwrap();
        assert_eq!(saved.starts_with(&[0xef, 0xbb, 0xbf]), crlf);
        assert_eq!(saved.windows(2).any(|bytes| bytes == b"\r\n"), crlf);
        let mut expected = parse(
            source
                .replace(
                    " Duplicate=\"same\" Duplicate=\"same\"",
                    " Duplicate=\"same\"",
                )
                .as_bytes(),
        );
        let actual = parse(
            String::from_utf8_lossy(&saved)
                .replace(
                    " Duplicate=\"same\" Duplicate=\"same\"",
                    " Duplicate=\"same\"",
                )
                .as_bytes(),
        );
        let expected_doc = expected.get_mut_child("C").unwrap();
        let actual_doc = actual.get_child("C").unwrap();
        assert_ne!(
            expected_doc.attributes["DateS"],
            actual_doc.attributes["DateS"]
        );
        for key in ["Date", "DateS"] {
            expected_doc
                .attributes
                .insert(key.into(), actual_doc.attributes[key].clone());
        }
        let expected_bytes = source
            .replacen(
                &format!("Name=\"{old}\" Marker=\"selected\""),
                &format!(
                    "Name=\"{}\" Marker=\"selected\"",
                    quick_xml::escape::escape(name)
                ),
                1,
            )
            .replace(
                "Date=\"2020-01-01 00:00:00\"",
                &format!("Date=\"{}\"", actual_doc.attributes["Date"]),
            )
            .replace(
                "DateS=\"1\"",
                &format!("DateS=\"{}\"", actual_doc.attributes["DateS"]),
            );
        assert_eq!(saved, expected_bytes.as_bytes());
        clock(&mut expected)
            .children
            .iter_mut()
            .filter_map(|node| node.as_mut_element())
            .find(|entry| entry.attributes.get("U").map(String::as_str) == Some(SELECTED))
            .unwrap()
            .attributes
            .insert("Name".into(), name.into());
        assert_eq!(actual, expected);
        name_command(&output, "uuid:clock", SELECTED, name)
            .arg("-q")
            .assert()
            .success();
        assert_eq!(fs::read(&output).unwrap(), saved);
        time_command(&output, "uuid:clock", SELECTED, "07:15")
            .arg("-q")
            .assert()
            .success();
        active_command(&output, "uuid:clock", SELECTED)
            .args(["--no-active", "-q"])
            .assert()
            .success();
        mode_command(&output, "uuid:clock", SELECTED, "3")
            .args(["--no-enabled", "-q"])
            .assert()
            .success();
        add_command(&output, "uuid:clock", "Additional", "10:00")
            .arg("-q")
            .assert()
            .success();
        command(&output, "uuid:clock", SELECTED)
            .arg("-q")
            .assert()
            .success();
    }
}

#[test]
fn alarm_name_named_entity_noops_copy_exact_bytes_and_missing_or_empty_names_have_no_add_default() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let name = " Früh & \"Spät\", <🚀>' &#65; ";
    let encoded = " Früh &amp; &quot;Spät&quot;, &lt;🚀&gt;&apos; &amp;#65; ";
    let source = fixture().replacen(
        "Name=\"Same name\" Marker=\"selected\"",
        &format!("Name='{encoded}' Marker=\"selected\""),
        1,
    );
    fs::write(&input, &source).unwrap();
    let result = name_command(&input, "Clock [Bedroom]", SELECTED, name)
        .arg("--save-as")
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["name_change"]["changed"], false);
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    let csv = name_command(&input, "uuid:clock", SELECTED, name)
        .args(["-o", "csv", "--no-header"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let csv = String::from_utf8(csv).unwrap();
    assert!(!csv.contains("BLOCK_UUID"));
    assert!(csv.contains("\"\"Spät\"\""));
    assert!(csv.trim_end().ends_with(",false"));
    name_command(&input, "uuid:clock", SELECTED, name)
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    name_command(&input, "uuid:clock", SELECTED, name)
        .assert()
        .success()
        .stdout(predicates::str::contains("unchanged"));
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());

    for stored in [None, Some("")] {
        let replacement = stored.map_or(String::new(), |value| format!("Name=\"{value}\" "));
        let mut source = fixture().replacen(
            "Name=\"Same name\" Marker=\"selected\"",
            &format!("{replacement}Marker=\"selected\""),
            1,
        );
        if stored.is_none() {
            source = source.replace(" Date=\"2020-01-01 00:00:00\" DateS=\"1\"", "");
        }
        fs::write(&input, &source).unwrap();
        let result = name_command(&input, "uuid:clock", SELECTED, "New name")
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(
            result["name_change"]["previous_name"],
            serde_json::json!(stored)
        );
        assert_eq!(result["name_change"]["changed"], true);
        let saved = fs::read(&input).unwrap();
        let saved_text = String::from_utf8(saved.clone()).unwrap();
        assert!(saved_text.contains("Name=\"New name\""));
        assert_ne!(
            parse(&saved).get_child("C").unwrap().attributes["DateS"],
            "1"
        );
    }
}

#[test]
fn alarm_name_uses_original_attribute_for_digit_equals_names_and_exact_noops() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for previous in ["Wake 7am=ok", "Alarm 1=2", "Shift 12h=on"] {
        let changed_name = previous.replacen(' ', " _", 1);
        for requested in [changed_name.as_str(), previous] {
            let source = fixture().replacen(
                "Name=\"Same name\" Marker=\"selected\"",
                &format!("Name=\"{previous}\" Marker=\"selected\""),
                1,
            );
            fs::write(&input, &source).unwrap();
            let result = name_command(&input, "uuid:clock", SELECTED, requested)
                .arg("--save-as")
                .arg(&output)
                .args(["-o", "json"])
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();
            let result: Value = serde_json::from_slice(&result).unwrap();
            let changed = requested != previous;
            assert_eq!(result["name_change"]["previous_name"], previous);
            assert_eq!(result["name_change"]["changed"], changed);
            assert_eq!(result["name_change"]["name"], requested);
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            let saved = fs::read(&output).unwrap();
            let mut actual = parse(&saved);
            let document = actual.get_child("C").unwrap();
            let date = document.attributes["Date"].clone();
            let seconds = document.attributes["DateS"].clone();
            let entry = clock(&mut actual)
                .children
                .iter()
                .filter_map(|node| node.as_element())
                .find(|entry| entry.attributes.get("U").map(String::as_str) == Some(SELECTED))
                .unwrap();
            assert_eq!(entry.attributes["Name"], requested);
            if changed {
                assert_ne!(seconds, "1");
                assert_ne!(date, "2020-01-01 00:00:00");
                let expected = source
                    .replacen(
                        &format!("Name=\"{previous}\" Marker=\"selected\""),
                        &format!("Name=\"{requested}\" Marker=\"selected\""),
                        1,
                    )
                    .replace("DateS=\"1\"", &format!("DateS=\"{seconds}\""))
                    .replace("Date=\"2020-01-01 00:00:00\"", &format!("Date=\"{date}\""));
                assert_eq!(saved, expected.as_bytes());
            } else {
                assert_eq!(seconds, "1");
                assert_eq!(date, "2020-01-01 00:00:00");
                assert_eq!(saved, source.as_bytes());
            }
        }
    }
}

#[test]
fn alarm_name_rejects_numeric_identity_and_stamp_metadata_without_touching_destinations() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for source in [
        fixture().replacen(
            &format!("<AlarmClock U=\"{SELECTED}\""),
            &format!("<AlarmClock U=\"{}\"", SELECTED.replacen('0', "&#48;", 1)),
            1,
        ),
        fixture().replacen("U=\"clock\"", "U=\"clo&#99;k\"", 1),
        fixture().replacen("Type=\"AlarmClock\"", "Type=\"AlarmClo&#99;k\"", 1),
        fixture()
            .replace("Type=\"Document\"", "Type=\"Docu&#109;ent\"")
            .replace(
                "</Loxone>",
                "<C Type=\"Document\" U=\"decoy\" DateS=\"99\"/></Loxone>",
            ),
        fixture().replace("DateS=\"1\"", "DateS=\"&#49;\""),
    ] {
        fs::write(&input, &source).unwrap();
        for existing_destination in [true, false] {
            if existing_destination {
                fs::write(&output, b"preserve destination").unwrap();
            } else {
                fs::remove_file(&output).unwrap();
            }
            name_command(&input, "uuid:clock", SELECTED, "Renamed")
                .arg("--save-as")
                .arg(&output)
                .assert()
                .failure()
                .stderr(predicates::str::contains("target-name-storage"));
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            if existing_destination {
                assert_eq!(fs::read(&output).unwrap(), b"preserve destination");
            } else {
                assert!(!output.exists());
            }
        }
    }
}

#[test]
fn alarm_name_guards_and_storage_ambiguities_are_atomic_including_save_as_and_noops() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (source, uuid, name, error) in [
        (fixture(), SELECTED, "", "this CLI supports nonempty"),
        (fixture(), SELECTED, " \t ", "this CLI supports nonempty"),
        (fixture(), SELECTED, "A\nB", "single-line"),
        (fixture(), SELECTED, "A\u{2028}B", "single-line"),
        (fixture(), SELECTED, "A\u{ffff}B", "single-line"),
        (fixture(), STANDARD, "Renamed", "ordinary alarms only"),
        (fixture(), "7", "Renamed", "UUID"),
        (
            fixture().replace("V=\"175\"", "V=\"174\""),
            SELECTED,
            "Same name",
            "V=175",
        ),
        (
            fixture().replace("Time=\"25200\"", "Time=\"21600\""),
            SELECTED,
            "Same name",
            "strictly increasing",
        ),
        (
            fixture().replacen("Name=\"Same name\"", "Name=\"S&#97;me name\"", 1),
            SELECTED,
            "Same name",
            "target-name-storage",
        ),
        (
            fixture().replacen("Name=\"Same name\"", "Name=\"S&#x61;me name\"", 1),
            SELECTED,
            "Renamed",
            "target-name-storage",
        ),
        (
            fixture().replacen("Name=\"Same name\"", "Name=\"Same\tname\"", 1),
            SELECTED,
            "Same name",
            "target-name-storage",
        ),
        (
            fixture().replacen("Name=\"Same name\"", "Name=\"Same\r\nname\"", 1),
            SELECTED,
            "Same name",
            "target-name-storage",
        ),
        (
            fixture().replace("encoding=\"utf-8\"", "encoding=\"iso-8859-1\""),
            SELECTED,
            "Renamed",
            "target-name-storage",
        ),
        (
            fixture().replace("<Loxone>", "<!DOCTYPE Loxone><Loxone>"),
            SELECTED,
            "Renamed",
            "target-name-storage",
        ),
        (
            fixture().replace(
                &format!("<AlarmClock U=\"{SELECTED}\""),
                &format!("<x:AlarmClock xmlns:x=\"urn:alarm\" U=\"{SELECTED}\""),
            ),
            SELECTED,
            "Renamed",
            "target-name-storage",
        ),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        name_command(&input, "uuid:clock", uuid, name)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure()
            .stderr(predicates::str::contains(error));
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    let source = fixture();
    fs::write(&input, &source).unwrap();
    for arguments in [vec!["--room", "Bedroom"], vec!["--dry-run"]] {
        name_command(&input, "Clock [Bedroom]", SELECTED, "New")
            .args(arguments)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
    // An unselected row's unusual Name must not be mistaken for the target.
    let source = fixture().replacen("Name=\"Same name\"", "Name=\"Other&#65;\"", 1);
    fs::write(&input, &source).unwrap();
    name_command(&input, "uuid:clock", LAST, "Last renamed")
        .arg("-q")
        .assert()
        .success();
    let saved = String::from_utf8(fs::read(&input).unwrap()).unwrap();
    assert!(saved.contains("Name=\"Other&#65;\""));
}

#[test]
fn alarm_time_edit_physically_reorders_existing_records_without_changing_identity_or_other_fields()
{
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (time, seconds, order) in [
        ("06:30", 23400, [STANDARD, SELECTED, LAST]),
        ("09:00:01", 32401, [STANDARD, LAST, SELECTED]),
        ("00:00", 0, [SELECTED, STANDARD, LAST]),
        ("23:59:59", 86399, [STANDARD, LAST, SELECTED]),
    ] {
        let source = fixture().replace("Active=\"true\"", "Active=\"FaLsE\"");
        fs::write(&input, &source).unwrap();
        let result = time_command(&input, "Clock", RFC_SELECTED, time)
            .args(["--room", "Bedroom", "--save-as"])
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["time_change"]["entry_uuid"], RFC_SELECTED);
        assert_eq!(result["time_change"]["stored_entry_uuid"], SELECTED);
        assert_eq!(result["time_change"]["previous_time_seconds"], 25200);
        assert_eq!(result["time_change"]["time_seconds"], seconds);
        assert_eq!(result["time_change"]["changed"], true);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let saved = fs::read(&output).unwrap();
        assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            saved.iter().enumerate().all(|(index, byte)| {
                *byte != b'\n' || (index > 0 && saved[index - 1] == b'\r')
            })
        );
        let mut original = parse(source.as_bytes());
        let mut actual = parse(&saved);
        assert_ne!(
            original.get_child("C").unwrap().attributes["DateS"],
            actual.get_child("C").unwrap().attributes["DateS"]
        );
        let expected_clock = clock(&mut original);
        let actual_clock = clock(&mut actual);
        assert_eq!(actual_clock.attributes, expected_clock.attributes);
        let entries: Vec<_> = actual_clock
            .children
            .iter()
            .filter_map(|node| node.as_element().filter(|entry| entry.name == "AlarmClock"))
            .collect();
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.attributes["U"].as_str())
                .collect::<Vec<_>>(),
            order
        );
        for entry in &entries {
            let mut expected = expected_clock
                .children
                .iter()
                .filter_map(|node| node.as_element())
                .find(|other| {
                    other.name == "AlarmClock" && other.attributes["U"] == entry.attributes["U"]
                })
                .unwrap()
                .clone();
            if entry.attributes["U"] == SELECTED {
                if seconds == 0 {
                    expected.attributes.shift_remove("Time");
                } else {
                    expected
                        .attributes
                        .insert("Time".into(), seconds.to_string());
                }
            }
            assert_eq!(**entry, expected);
        }
        for (before, after) in expected_clock.children.iter().zip(&actual_clock.children) {
            if before
                .as_element()
                .is_none_or(|entry| entry.name != "AlarmClock")
            {
                assert_eq!(before, after);
            }
        }
        time_command(&output, "uuid:clock", SELECTED, time)
            .arg("-q")
            .assert()
            .success();
        assert_eq!(fs::read(&output).unwrap(), saved);
        mode_command(&output, "uuid:clock", SELECTED, "3")
            .args(["--enabled", "-q"])
            .assert()
            .success();
        active_command(&output, "uuid:clock", SELECTED)
            .args(["--no-active", "-q"])
            .assert()
            .success();
        add_command(&output, "uuid:clock", "Additional", "10:00")
            .arg("-q")
            .assert()
            .success();
        command(&output, "uuid:clock", SELECTED)
            .arg("-q")
            .assert()
            .success();
    }
}

#[test]
fn alarm_time_semantic_noops_preserve_exact_bytes_and_save_as_with_all_output_formats() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture().replace("Time=\"25200\"", "Time=\"+00025200\"");
    fs::write(&input, &source).unwrap();
    for time in ["07:00", "07:00:00"] {
        let result = time_command(&input, "Clock [Bedroom]", SELECTED, time)
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["time_change"]["changed"], false);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    }
    let csv = time_command(&input, "uuid:clock", RFC_SELECTED, "07:00")
        .args(["-o", "csv", "--no-header"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(csv).unwrap().trim(),
        format!("\"clock\",{RFC_SELECTED},{SELECTED},25200,25200,false")
    );
    time_command(&input, "uuid:clock", SELECTED, "07:00")
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    time_command(&input, "uuid:clock", SELECTED, "07:00")
        .assert()
        .success()
        .stdout(predicates::str::contains("unchanged"));
}

#[test]
fn alarm_time_errors_are_atomic_for_equal_times_invalid_requests_and_unsupported_storage() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let base = fixture();
    let invalid_time_storage = base.replace("Time=\"25200\"", "Time=\"28800\"");
    let dated = base.replace("Id=\"7\"", "Id=\"7\" Year=\"2026\"");
    let rfc_storage = base.replace(SELECTED, RFC_SELECTED);
    for (source, selector, uuid, time) in [
        (base.as_str(), "uuid:clock", SELECTED, "06:00"),
        (base.as_str(), "uuid:clock", SELECTED, "08:00:00"),
        (base.as_str(), "uuid:clock", SELECTED, "24:00"),
        (base.as_str(), "uuid:clock", SELECTED, "07:60"),
        (base.as_str(), "uuid:clock", SELECTED, "07:00:60"),
        (base.as_str(), "uuid:clock", SELECTED, "7:00"),
        (base.as_str(), "uuid:clock", SELECTED, "25200"),
        (base.as_str(), "uuid:clock", STANDARD, "05:00"),
        (base.as_str(), "uuid:clock", "missing", "05:00"),
        (base.as_str(), "Clock", SELECTED, "05:00"),
        (
            invalid_time_storage.as_str(),
            "uuid:clock",
            SELECTED,
            "05:00",
        ),
        (dated.as_str(), "uuid:clock", SELECTED, "05:00"),
        (rfc_storage.as_str(), "uuid:clock", SELECTED, "05:00"),
    ] {
        fs::write(&input, source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        time_command(&input, selector, uuid, time)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
        fs::remove_file(&output).unwrap();
        time_command(&input, selector, uuid, time)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert!(!output.exists());
    }
    fs::write(&input, &base).unwrap();
    time_command(&input, "Clock [Bedroom]", SELECTED, "05:00")
        .args(["--room", "Bedroom"])
        .assert()
        .failure();
    time_command(&input, "uuid:clock", SELECTED, "05:00")
        .arg("--dry-run")
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), base.as_bytes());
}

#[test]
fn alarm_mode_changes_one_native_bit_without_reactivating_or_rewriting_other_fields() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (source, bit, enabled, modes, active) in [
        (fixture(), "0", true, 249, true),
        (
            fixture().replace("Active=\"true\"", "Active=\"FaLsE\""),
            "9",
            true,
            760,
            false,
        ),
        (
            fixture().replace(" Active=\"true\"", ""),
            "3",
            false,
            240,
            false,
        ),
    ] {
        fs::write(&input, &source).unwrap();
        let result = mode_command(&input, "Clock", RFC_SELECTED, bit)
            .args([
                "--room",
                "Bedroom",
                if enabled { "--enabled" } else { "--no-enabled" },
                "--save-as",
            ])
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        let change = &result["mode_change"];
        assert_eq!(change["entry_uuid"], RFC_SELECTED);
        assert_eq!(change["stored_entry_uuid"], SELECTED);
        assert_eq!(change["mode_bit"], bit.parse::<u8>().unwrap());
        assert_eq!(change["previous_modes"], 248);
        assert_eq!(change["modes"], modes);
        assert_eq!(change["previous_enabled"], !enabled);
        assert_eq!(change["enabled"], enabled);
        assert_eq!(change["active"], active);
        assert_eq!(change["changed"], true);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let saved = fs::read(&output).unwrap();
        assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            saved.iter().enumerate().all(|(index, byte)| {
                *byte != b'\n' || (index > 0 && saved[index - 1] == b'\r')
            })
        );
        let mut expected = parse(source.as_bytes());
        clock(&mut expected)
            .children
            .iter_mut()
            .filter_map(|node| node.as_mut_element())
            .find(|entry| {
                entry.name == "AlarmClock"
                    && entry.attributes.get("U").map(String::as_str) == Some(SELECTED)
            })
            .unwrap()
            .attributes
            .insert("Modes".into(), modes.to_string());
        let mut actual = parse(&saved);
        assert_ne!(
            actual.get_child("C").unwrap().attributes["DateS"],
            expected.get_child("C").unwrap().attributes["DateS"]
        );
        for root in [&mut actual, &mut expected] {
            let document = root.get_mut_child("C").unwrap();
            document.attributes.shift_remove("Date");
            document.attributes.shift_remove("DateS");
        }
        assert_eq!(actual, expected);
        active_command(&output, "uuid:clock", SELECTED)
            .args(["--active", "-q"])
            .assert()
            .success();
        command(&output, "uuid:clock", SELECTED)
            .arg("-q")
            .assert()
            .success();
    }
}

#[test]
fn alarm_mode_noops_preserve_raw_mask_boolean_defaults_and_stamp_including_save_as() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (source, active) in [
        (
            fixture().replace("Active=\"true\"", "Active=\"TrUe\""),
            true,
        ),
        (
            fixture().replace("Active=\"true\"", "Active=\"FaLsE\""),
            false,
        ),
        (fixture().replace(" Active=\"true\"", ""), false),
    ] {
        let source = source.replace("Modes=\"248\"", "Modes=\"+000248\"");
        fs::write(&input, &source).unwrap();
        for (bit, flag, enabled) in [("3", "--enabled", true), ("9", "--no-enabled", false)] {
            for save_as in [false, true] {
                let mut command = mode_command(&input, "Clock [Bedroom]", SELECTED, bit);
                command.args([flag, "-o", "csv", "--no-header", "-q"]);
                if save_as {
                    command.arg("--save-as").arg(&output);
                }
                command.assert().success().stdout(format!(
                    "\"clock\",{SELECTED},{SELECTED},{bit},{enabled},{enabled},248,248,{active},false\n"
                ));
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                if save_as {
                    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
                }
            }
        }
    }
    mode_command(&input, "uuid:clock", SELECTED, "3")
        .args(["--enabled", "-q"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn alarm_mode_final_bit_and_unsupported_storage_refuse_before_either_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (bit, modes) in [("0", "1"), ("9", "512")] {
        for active in ["Active=\"true\"", "Active=\"false\"", ""] {
            let source = fixture()
                .replace("Modes=\"248\"", &format!("Modes=\"{modes}\""))
                .replace("Active=\"true\"", active);
            fs::write(&input, &source).unwrap();
            for existing in [false, true] {
                if existing {
                    fs::write(&output, b"sentinel").unwrap();
                }
                mode_command(&input, "uuid:clock", SELECTED, bit)
                    .args(["--no-enabled", "--save-as"])
                    .arg(&output)
                    .assert()
                    .failure()
                    .stderr(predicates::str::contains("final selected mode"));
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                if existing {
                    assert_eq!(fs::read(&output).unwrap(), b"sentinel");
                    fs::remove_file(&output).unwrap();
                } else {
                    assert!(!output.exists());
                }
            }
        }
    }
    for (source, selector, uuid) in [
        (fixture(), "Clock", SELECTED),
        (fixture(), "uuid:clock", STANDARD),
        (fixture(), "uuid:clock", "bad-uuid"),
        (
            fixture().replace(SELECTED, RFC_SELECTED),
            "uuid:clock",
            SELECTED,
        ),
        (
            fixture().replace("Modes=\"248\"", "Modes=\"0\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            fixture().replace("Modes=\"248\"", "Modes=\"1024\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            fixture().replace("N=\"3\"", "N=\"4\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            fixture().replace("Active=\"true\"", "Active=\"1\""),
            "uuid:clock",
            SELECTED,
        ),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"sentinel").unwrap();
        mode_command(&input, selector, uuid, "0")
            .args(["--enabled", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"sentinel");
    }
}

#[test]
fn alarm_mode_requires_one_explicit_boolean_and_native_bit_not_labels_or_columns() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = fixture();
    fs::write(&input, &source).unwrap();
    for flags in [
        vec![],
        vec!["--enabled", "--no-enabled"],
        vec!["--enabled", "false"],
        vec!["--no-enabled", "--room", "Bedroom"],
        vec!["--enabled", "--dry-run"],
    ] {
        mode_command(&input, "Clock [Bedroom]", SELECTED, "3")
            .args(flags)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
    for bit in ["10", "12", "-1", "256", "Monday", "0x200"] {
        mode_command(&input, "uuid:clock", SELECTED, bit)
            .arg("--enabled")
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
}

#[test]
fn alarm_add_allocates_first_hole_native_identity_and_sorted_row_preserving_other_nodes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    let result = add_command(&input, "Clock", "Früh & \"bereit\"", "07:30:01")
        .args(["--room", "Bedroom", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    let uuid = result["added"]["entry_uuid"].as_str().unwrap();
    assert_eq!(uuid.len(), 35);
    assert_eq!(
        uuid.split('-').map(str::len).collect::<Vec<_>>(),
        [8, 4, 4, 16]
    );
    let identity = uuid::Uuid::parse_str(&uuid.replace('-', "")).unwrap();
    assert!(!identity.is_nil());
    assert_ne!(uuid, SELECTED);
    assert_eq!(result["added"]["id"], 1);
    assert_eq!(result["added"]["time_seconds"], 27001);
    assert_eq!(result["added"]["total_entries"], 4);
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
    let target = clock(&mut expected);
    let position = target
        .children
        .iter()
        .position(|node| {
            node.as_element()
                .is_some_and(|entry| entry.attributes.get("U").map(String::as_str) == Some(LAST))
        })
        .unwrap();
    let entry = Element::parse(
        format!(
            "<AlarmClock U=\"{uuid}\" Id=\"1\" Name=\"Früh &amp; &quot;bereit&quot;\" Time=\"27001\" Active=\"true\" Modes=\"1023\"/>"
        )
        .as_bytes(),
    )
    .unwrap();
    target
        .children
        .insert(position, xmltree::XMLNode::Element(entry));
    target.attributes.insert("N".into(), "4".into());
    let mut actual = parse(&saved);
    assert_ne!(
        actual.get_child("C").unwrap().attributes["DateS"],
        expected.get_child("C").unwrap().attributes["DateS"]
    );
    for root in [&mut actual, &mut expected] {
        let document = root.get_mut_child("C").unwrap();
        document.attributes.shift_remove("Date");
        document.attributes.shift_remove("DateS");
    }
    assert_eq!(actual, expected);

    active_command(&output, "uuid:clock", uuid)
        .args(["--no-active", "-q"])
        .assert()
        .success();
    command(&output, "uuid:clock", uuid)
        .arg("-q")
        .assert()
        .success();
    let mut after = parse(&fs::read(&output).unwrap());
    assert_eq!(clock(&mut after).attributes["N"], "3");
}

#[test]
fn alarm_add_repeated_names_reuse_id_holes_and_keep_persisted_uuid_on_next_add() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    fs::write(&input, fixture().replace("Id=\"12\"", "Id=\"1\"")).unwrap();
    let mut uuids = Vec::new();
    for (time, id) in [("00:00", 2), ("23:59:59", 3)] {
        let result = add_command(&input, "Clock [Bedroom]", "Same name", time)
            .args(["-o", "json", "-q"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["added"]["id"], id);
        uuids.push(result["added"]["entry_uuid"].as_str().unwrap().to_owned());
    }
    assert_ne!(uuids[0], uuids[1]);
    let mut saved = parse(&fs::read(&input).unwrap());
    let target = clock(&mut saved);
    let entries: Vec<_> = target
        .children
        .iter()
        .filter_map(|node| node.as_element().filter(|entry| entry.name == "AlarmClock"))
        .collect();
    assert_eq!(target.attributes["N"], "5");
    assert_eq!(entries[0].attributes["U"], uuids[0]);
    assert!(!entries[0].attributes.contains_key("Time"));
    assert_eq!(entries[4].attributes["U"], uuids[1]);
    assert_eq!(entries[4].attributes["Time"], "86399");
}

#[test]
fn alarm_add_rejects_ties_invalid_input_and_unsupported_storage_before_any_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture();
    let cases = [
        (source.clone(), "Clock", "Wake", "07:30"),
        (source.clone(), "uuid:clock", "Wake", "06:00"),
        (source.clone(), "uuid:clock", "Wake", "07:00"),
        (source.clone(), "uuid:clock", "Wake", "24:00"),
        (source.clone(), "uuid:clock", "Wake", "07:60"),
        (source.clone(), "uuid:clock", "Wake", "07:30:60"),
        (source.clone(), "uuid:clock", "Wake", "7:30"),
        (source.clone(), "uuid:clock", "Wake", "07:30:00.1"),
        (source.clone(), "uuid:clock", " ", "07:30"),
        (source.clone(), "uuid:clock", "Wake\nnow", "07:30"),
        (
            source.replace(LAST, RFC_LAST),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("N=\"3\"", "N=\"4\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("V=\"175\"", "V=\"154\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("Int=\"true\"", "Int=\"false\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("Id=\"12\"", "Id=\"7\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("Modes=\"248\"", "Modes=\"0\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("Sa=\"true\"", "Sa=\"1\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
        (
            source.replace("Id=\"12\"", "Id=\"12\" Year=\"2027\""),
            "uuid:clock",
            "Wake",
            "07:30",
        ),
    ];
    for (source, selector, name, time) in cases {
        fs::write(&input, &source).unwrap();
        for existing_destination in [false, true] {
            if existing_destination {
                fs::write(&output, b"sentinel").unwrap();
            }
            add_command(&input, selector, name, time)
                .arg("--save-as")
                .arg(&output)
                .assert()
                .failure();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            if existing_destination {
                assert_eq!(fs::read(&output).unwrap(), b"sentinel");
                fs::remove_file(&output).unwrap();
            } else {
                assert!(!output.exists());
            }
        }
    }
}

#[test]
fn alarm_add_standard_only_and_csv_quiet_and_dry_run_contract() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    fs::write(&input, fixture()).unwrap();
    for uuid in [SELECTED, LAST] {
        command(&input, "uuid:clock", uuid)
            .arg("-q")
            .assert()
            .success();
    }
    add_command(&input, "uuid:clock", "CSV,\"name\"", "07:30")
        .args(["-o", "csv", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            ",1,\"CSV,\"\"name\"\"\",27000,true,1023,2\n",
        ));
    let before = fs::read(&input).unwrap();
    add_command(&input, "uuid:clock", "Dry", "08:30")
        .arg("--dry-run")
        .assert()
        .failure();
    add_command(&input, "Clock [Bedroom]", "Room", "08:30")
        .args(["--room", "Bedroom"])
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), before);
}

#[test]
fn alarm_delete_uses_uuid_alias_and_room_preserving_every_other_node() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    let result = command(&input, "Clock", RFC_SELECTED)
        .args(["--room", "Bedroom", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["removed"]["stored_entry_uuid"], SELECTED);
    assert_eq!(result["removed"]["entry_uuid"], RFC_SELECTED);
    assert_eq!(result["removed"]["removed_attributes"]["Id"], "7");
    assert_eq!(result["removed"]["remaining_entries"], 2);
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
    let target = clock(&mut expected);
    target.children.retain(|node| {
        node.as_element().is_none_or(|entry| {
            entry.name != "AlarmClock"
                || entry.attributes.get("U").map(String::as_str) != Some(SELECTED)
        })
    });
    target.attributes.insert("N".into(), "2".into());
    let mut actual = parse(&saved);
    let target = clock(&mut actual);
    assert_eq!(target.attributes["Ode"], "7");
    assert_eq!(target.attributes["Le"], "7");
    assert_eq!(target.get_child("Entry").unwrap().attributes["U"], SELECTED);
    for root in [&mut actual, &mut expected] {
        let document = root.get_mut_child("C").unwrap();
        document.attributes.shift_remove("Date");
        document.attributes.shift_remove("DateS");
    }
    assert_eq!(actual, expected);
}

#[test]
fn alarm_delete_refuses_stored_rfc_uuids_in_target_survivor_or_standard_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture();
    for (source, request) in [
        (source.replace(SELECTED, RFC_SELECTED), SELECTED),
        (source.replace(SELECTED, RFC_SELECTED), RFC_SELECTED),
        (source.replace(LAST, RFC_LAST), RFC_SELECTED),
        (source.replace(STANDARD, RFC_STANDARD), SELECTED),
    ] {
        fs::write(&input, &source).unwrap();
        for existing_destination in [false, true] {
            if existing_destination {
                fs::write(&output, b"sentinel").unwrap();
            }
            command(&input, "uuid:clock", request)
                .arg("--save-as")
                .arg(&output)
                .assert()
                .failure()
                .stderr(predicates::str::contains(
                    "Stored alarm entry U must use native 35-character",
                ));
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            if existing_destination {
                assert_eq!(fs::read(&output).unwrap(), b"sentinel");
                fs::remove_file(&output).unwrap();
            } else {
                assert!(!output.exists());
            }
        }
    }
}

#[test]
fn alarm_delete_final_ordinary_entry_keeps_original_standard_and_count_one() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    fs::write(&input, fixture()).unwrap();
    for (uuid, expected) in [(SELECTED, 2), (LAST, 1)] {
        command(&input, "Clock [Bedroom]", uuid)
            .args(["-o", "csv", "--no-header", "-q"])
            .assert()
            .success()
            .stdout(format!("\"clock\",{uuid},{uuid},{expected}\n"));
    }
    let mut saved = parse(&fs::read(&input).unwrap());
    let target = clock(&mut saved);
    assert_eq!(target.attributes["N"], "1");
    let standard = target.get_child("AlarmClock").unwrap();
    assert_eq!(standard.attributes["U"], STANDARD);
    assert_eq!(standard.attributes["Time"], "21600");
    assert_eq!(standard.attributes["Int"], "true");
    let before = fs::read(&input).unwrap();
    for uuid in [LAST, STANDARD] {
        command(&input, "uuid:clock", uuid).assert().failure();
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}

#[test]
fn alarm_delete_rejects_ambiguous_malformed_or_migrating_storage_before_either_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    let source = fixture();
    let cases = [
        (source.clone(), "Clock", SELECTED),
        (source.clone(), "uuid:clock", STANDARD),
        (source.clone(), "uuid:clock", "bad-uuid"),
        (source.replace("N=\"3\"", "N=\"4\""), "uuid:clock", SELECTED),
        (
            source.replace("V=\"175\"", "V=\"154\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Int=\"true\"", "Int=\"false\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Id=\"12\"", "Id=\"7\""),
            "uuid:clock",
            SELECTED,
        ),
        (source.replace(LAST, SELECTED), "uuid:clock", SELECTED),
        (
            source.replace("Time=\"28800\"", "Time=\"25200\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Id=\"12\"", "Id=\"12\" Year=\"2027\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Modes=\"248\"", "Modes=\"0\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Sa=\"true\"", "Sa=\"1\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Type=\"AlarmClock\"", "Type=\"DayTimer\""),
            "uuid:clock",
            SELECTED,
        ),
    ];
    for (source, selector, uuid) in cases {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"sentinel").unwrap();
        command(&input, selector, uuid)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"sentinel");
    }
    fs::write(&input, fixture()).unwrap();
    command(&input, "Clock [Bedroom]", SELECTED)
        .args(["--room", "Bedroom"])
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), fixture().as_bytes());
}

#[test]
fn alarm_active_changes_only_selected_boolean_preserving_references_and_other_rows() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    for (uuid, stored_uuid, active) in [(RFC_LAST, LAST, true), (RFC_SELECTED, SELECTED, false)] {
        fs::write(&input, &original).unwrap();
        let result = active_command(&input, "Clock", uuid)
            .args([
                "--room",
                "Bedroom",
                if active { "--active" } else { "--no-active" },
            ])
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["activation"]["stored_entry_uuid"], stored_uuid);
        assert_eq!(result["activation"]["previous_active"], !active);
        assert_eq!(result["activation"]["active"], active);
        assert_eq!(result["activation"]["changed"], true);
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        let saved = fs::read(&output).unwrap();
        assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            saved.iter().enumerate().all(|(index, byte)| {
                *byte != b'\n' || (index > 0 && saved[index - 1] == b'\r')
            })
        );
        let mut expected = parse(original.as_bytes());
        let target = clock(&mut expected)
            .children
            .iter_mut()
            .filter_map(|node| node.as_mut_element())
            .find(|entry| {
                entry.name == "AlarmClock"
                    && entry.attributes.get("U").map(String::as_str) == Some(stored_uuid)
            })
            .unwrap();
        if active {
            target.attributes.insert("Active".into(), "true".into());
        } else {
            target.attributes.shift_remove("Active");
        }
        let mut actual = parse(&saved);
        assert_ne!(
            actual.get_child("C").unwrap().attributes["Date"],
            "2020-01-01 00:00:00"
        );
        for root in [&mut actual, &mut expected] {
            let document = root.get_mut_child("C").unwrap();
            document.attributes.shift_remove("Date");
            document.attributes.shift_remove("DateS");
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn alarm_active_noops_preserve_exact_bytes_spelling_absence_and_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture();
    for (source, uuid, flag, active) in [
        (source.clone(), LAST, "--no-active", false),
        (
            source.replace("Id=\"12\"", "Id=\"12\" Active=\"FaLsE\""),
            LAST,
            "--no-active",
            false,
        ),
        (
            source.replace("Active=\"true\"", "Active=\"TrUe\""),
            SELECTED,
            "--active",
            true,
        ),
    ] {
        fs::write(&input, &source).unwrap();
        for save_as in [false, true] {
            let mut command = active_command(&input, "Clock [Bedroom]", uuid);
            command.args([flag, "-o", "csv", "--no-header", "-q"]);
            if save_as {
                command.arg("--save-as").arg(&output);
            }
            command
                .assert()
                .success()
                .stdout(format!("\"clock\",{uuid},{uuid},{active},{active},false\n"));
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            if save_as {
                assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
            }
        }
    }
    fs::write(&input, fixture()).unwrap();
    active_command(&input, "uuid:clock", SELECTED)
        .args(["--no-active", "-q"])
        .assert()
        .success()
        .stdout("");
    let mut saved = parse(&fs::read(&input).unwrap());
    assert!(
        !clock(&mut saved)
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .find(|entry| entry.attributes.get("U").map(String::as_str) == Some(SELECTED))
            .unwrap()
            .attributes
            .contains_key("Active")
    );
}

#[test]
fn alarm_active_refuses_unsupported_storage_or_arguments_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture();
    let cases = [
        (source.clone(), "Clock", SELECTED),
        (source.clone(), "uuid:clock", STANDARD),
        (source.clone(), "uuid:clock", "bad-uuid"),
        (
            source.replace(SELECTED, RFC_SELECTED),
            "uuid:clock",
            SELECTED,
        ),
        (source.replace(LAST, RFC_LAST), "uuid:clock", SELECTED),
        (
            source.replace(STANDARD, RFC_STANDARD),
            "uuid:clock",
            SELECTED,
        ),
        (source.replace(LAST, SELECTED), "uuid:clock", SELECTED),
        (
            source.replace("Id=\"12\"", "Id=\"7\""),
            "uuid:clock",
            SELECTED,
        ),
        (source.replace("N=\"3\"", "N=\"4\""), "uuid:clock", SELECTED),
        (
            source.replace("V=\"175\"", "V=\"154\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Int=\"true\"", "Int=\"false\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Time=\"28800\"", "Time=\"25200\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Id=\"12\"", "Id=\"12\" Year=\"2027\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Modes=\"248\"", "Modes=\"0\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Modes=\"1023\"", "Modes=\"0\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Active=\"true\"", "Active=\"1\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Sa=\"true\"", "Sa=\"1\""),
            "uuid:clock",
            SELECTED,
        ),
        (
            source.replace("Type=\"AlarmClock\"", "Type=\"DayTimer\""),
            "uuid:clock",
            SELECTED,
        ),
    ];
    for (source, selector, uuid) in cases {
        fs::write(&input, &source).unwrap();
        for active in ["--active", "--no-active"] {
            for existing in [false, true] {
                if existing {
                    fs::write(&output, b"sentinel").unwrap();
                }
                active_command(&input, selector, uuid)
                    .arg(active)
                    .arg("--save-as")
                    .arg(&output)
                    .assert()
                    .failure();
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                if existing {
                    assert_eq!(fs::read(&output).unwrap(), b"sentinel");
                    fs::remove_file(&output).unwrap();
                } else {
                    assert!(!output.exists());
                }
            }
        }
    }
    fs::write(&input, fixture()).unwrap();
    for flags in [
        vec![],
        vec!["--active", "--no-active"],
        vec!["--active", "true"],
        vec!["--active", "--room", "Bedroom"],
        vec!["--active", "--dry-run"],
    ] {
        active_command(&input, "Clock [Bedroom]", SELECTED)
            .args(flags)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), fixture().as_bytes());
        assert!(!output.exists());
    }
}
