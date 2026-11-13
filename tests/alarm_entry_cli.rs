use std::{fs, path::Path};

use serde_json::Value;
use xmltree::Element;

const STANDARD: &str = "00000000-0000-0001-1500000000000000";
const SELECTED: &str = "00000000-0000-0004-1500000000000000";
const RFC_SELECTED: &str = "00000000-0000-0004-1500-000000000000";
const LAST: &str = "00000000-0000-0005-1500000000000000";

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"Place\" U=\"bed\" Title=\"Bedroom\"/><C Type=\"Place\" U=\"other-room\" Title=\"Other\"/><C Type=\"AlarmClock\" V=\"175\" U=\"clock\" Title=\"Clock\" N=\"3\" Dev=\"device\" Unknown=\"keep\"><IoData Pr=\"bed\"/><AlarmClock U=\"{STANDARD}\" Int=\"true\" Time=\"21600\" Modes=\"1023\" Name=\"Standard\"/><Co K=\"AQ\" U=\"wire\"><In Input=\"source\"/></Co><AlarmClock U=\"{SELECTED}\" Id=\"7\" Time=\"25200\" Modes=\"248\" Active=\"true\" Sa=\"true\" Name=\"Same name\" Marker=\"selected\"/><Unknown Preserve=\"yes\"/><AlarmClock U=\"{LAST}\" Id=\"12\" Time=\"28800\" Modes=\"1023\" Name=\"Same name\" Marker=\"survivor\"/></C><C Type=\"AlarmClock\" V=\"175\" U=\"other-clock\" Title=\"Clock\" N=\"1\"><IoData Pr=\"other-room\"/><AlarmClock U=\"other-standard\" Int=\"true\" Time=\"21600\" Modes=\"1023\"/></C></C></Loxone>\r\n"
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
        node.as_element()
            .is_none_or(|entry| entry.attributes.get("U").map(String::as_str) != Some(SELECTED))
    });
    target.attributes.insert("N".into(), "2".into());
    let mut actual = parse(&saved);
    for root in [&mut actual, &mut expected] {
        let document = root.get_mut_child("C").unwrap();
        document.attributes.shift_remove("Date");
        document.attributes.shift_remove("DateS");
    }
    assert_eq!(actual, expected);
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
        (source.replace(LAST, RFC_SELECTED), "uuid:clock", SELECTED),
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
