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

fn add_command(input: &Path, selector: &str, name: &str, time: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "alarm-entry-add"])
        .arg(input)
        .args([selector, "--name", name, "--time", time]);
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
