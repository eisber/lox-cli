use std::{fs, path::Path};

use predicates::prelude::PredicateBooleanExt;

const OWNER: &str = "a1111111-2222-3333-4444555566667777";
const INPUT: &str = "b1111111-2222-3333-4444555566667777";

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" U=\"01111111-2222-3333-4444555566667777\" V=\"121\" DateS=\"1\"><C Type=\"Page\" U=\"31111111-2222-3333-4444555566667777\" V=\"121\"><C Type=\"And\" U=\"{OWNER}\" V=\"121\" Nio=\"3\" Title=\"Gate\" Unknown=\"keep\"><Co K=\"I1\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/><Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\"/><Co K=\"Q\" U=\"51111111-2222-3333-4444555566667777\"/></C></C><C Type=\"OutputRef\" U=\"61111111-2222-3333-4444555566667777\"><Co K=\"AI\" U=\"71111111-2222-3333-4444555566667777\" Nc=\"1\"><Ref U=\"51111111-2222-3333-4444555566667777\" Unknown=\"keep-wire\"/></Co></C><SET><Password t=\"15\" v=\"unchanged\"/></SET></C></Loxone>\r\n"
    )
}

fn command(input: &Path, key: &str, invert: bool) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "set-input-inversion"])
        .arg(input)
        .args([
            format!("uuid:{OWNER}").as_str(),
            key,
            if invert {
                "--inverted"
            } else {
                "--no-inverted"
            },
        ]);
    command
}

fn control<'a>(element: &'a xmltree::Element, uuid: &str) -> Option<&'a xmltree::Element> {
    if element.name == "C" && element.attributes.get("U").map(String::as_str) == Some(uuid) {
        return Some(element);
    }
    element
        .children
        .iter()
        .filter_map(xmltree::XMLNode::as_element)
        .find_map(|child| control(child, uuid))
}

fn logic_fixture(owner_type: &str) -> String {
    let mut source = fixture()
        .replace("Type=\"And\"", &format!("Type=\"{owner_type}\""))
        .replace("V=\"121\"", "V=\"121\" WF=\"16384\"")
        .replace(
            "<C Type=\"Page\"",
            "<C Type=\"Program\" U=\"81111111-2222-3333-4444555566667777\" V=\"121\" WF=\"16384\"><C Type=\"Page\"",
        )
        .replace("</C></C><C Type=\"OutputRef\"", "</C></C></C><C Type=\"OutputRef\"");
    if owner_type == "Not" {
        source = source
            .replace("Nio=\"3\"", "Nio=\"2\"")
            .replace("K=\"I1\"", "K=\"I\"")
            .replace(
                "<Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\"/>",
                "",
            );
    }
    source
}

fn up_down_counter_fixture() -> String {
    logic_fixture("UpDownCounter")
        .replace("<Loxone>", "<ControlList>")
        .replace("</Loxone>", "</ControlList>")
        .replace("Nio=\"3\"", "Nio=\"9\"")
        .replace("K=\"I1\"", "K=\"Trigger\"")
        .replace(
            "<Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\"/>",
            "<Co K=\"InputDir\" U=\"41111111-2222-3333-4444555566667777\" Def='1' Inv='true' Nc='1'><Ref U='51111111-2222-3333-4444555566667777' Note='direction wire'/></Co>\
             <Co K='Reset' U='91111111-2222-3333-4444555566667777' Def='1' Inv='false'/>\
             <Co K='OnValue' U='a2222222-2222-3333-4444555566667777' Def='10'/>\
             <Co K='OffValue' U='a3333333-2222-3333-4444555566667777' Def='5'/>\
             <Co K='Remanence' U='a4444444-2222-3333-4444555566667777' Def='0'/>\
             <Co K='StartValue' U='a5555555-2222-3333-4444555566667777' Def='0'/>\
             <Co K='AQ' U='a6666666-2222-3333-4444555566667777' Note='analog output'/>",
        )
        .replace(
            "Title=\"Gate\"",
            "Title=\"Gate\" Pos='120,240' Rights='keep' Unit='pulses'",
        )
}

#[test]
fn up_down_counter_trigger_preserves_full_native_layout_and_atomic_destinations() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("counter.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = up_down_counter_fixture();
    for selected_storage in ["Def=\"0\"", "Inv='false'"] {
        let source = source.replace("Def=\"0\"", selected_storage);
        fs::write(&input, &source).unwrap();
        let result = command(&input, "Trigger", false)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["input_inversion"]["changed"], false);
        assert!(result["saved_to"].is_null());
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        fs::write(&existing, b"preserve until accepted").unwrap();
        for destination in [&existing, &new] {
            command(&input, "Trigger", false)
                .arg("--save-as")
                .arg(destination)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(destination).unwrap(), source.as_bytes());
            if destination == &new {
                fs::remove_file(destination).unwrap();
            }
            command(&input, "Trigger", true)
                .arg("--save-as")
                .arg(destination)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            let saved = fs::read_to_string(destination).unwrap();
            let selected =
                format!("<Co K=\"Trigger\" U=\"{INPUT}\" {selected_storage} Unknown=\"keep\"/>");
            let changed = if selected_storage == "Inv='false'" {
                selected.replace("Inv='false'", "Inv='true'")
            } else {
                selected.replace("/>", " Inv=\"true\"/>")
            };
            let expected = source.replace(&selected, &changed);
            let page_start = source.find("<C Type=\"Program\"").unwrap();
            assert_eq!(
                &saved[saved.find("<C Type=\"Program\"").unwrap()..],
                &expected[page_start..]
            );
            assert!(saved.starts_with("\u{feff}<?xml"));
            command(destination, "Trigger", true)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read_to_string(destination).unwrap(), saved);
            command(destination, "Trigger", false)
                .arg("-q")
                .assert()
                .success();
            let cleared = fs::read_to_string(destination).unwrap();
            assert!(!cleared.contains("Inv=\"true\""));
            assert!(!cleared.contains("Inv='true' Unknown=\"keep\""));
            assert!(cleared.contains("Def='1' Inv='true' Nc='1'"));
        }
        fs::remove_file(&new).unwrap();
    }
}

#[test]
fn up_down_counter_refuses_other_caps_incomplete_layout_and_context_before_all_writes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("counter.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = up_down_counter_fixture();
    let mut cases: Vec<_> = [
        "InputDir",
        "Reset",
        "OnValue",
        "OffValue",
        "Remanence",
        "StartValue",
        "Q",
        "AQ",
        "I1",
        "trigger",
    ]
    .into_iter()
    .map(|key| (source.clone(), key))
    .collect();
    for malformed in [
        source.replace("Nio=\"9\"", "Nio=\"8\""),
        source.replace(
            "<Co K='Remanence' U='a4444444-2222-3333-4444555566667777' Def='0'/>",
            "",
        ),
        source.replace("K='StartValue'", "K='OnValue'"),
        source.replace("Def=\"0\"", "Def=\"1\""),
        source.replace("Def=\"0\"", "Def=\"0\" Nc=\"1\""),
        source.replace(
            "Unknown=\"keep\"/>",
            "Unknown=\"keep\"><Ref U='51111111-2222-3333-4444555566667777'/></Co>",
        ),
        source.replace("Type=\"Program\"", "Type=\"Folder\""),
        source.replace("Type=\"Page\"", "Type=\"Folder\""),
        source.replace("WF=\"16384\"", "WF=\"16385\""),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" IfT='true'"),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" uuidParent='x'"),
        source.replace("DateS=\"1\"", "DateS=\"2147483647\""),
        source.replace("DateS=\"1\"", "DateS=\"+1\""),
        source.replace("Title=\"Gate\"", "Title=\"Noon 12hTF=quoted\""),
        source.replace("Def=\"0\"", "Def=\"0\" Def=\"0\""),
        source.replace(
            "U='a6666666-2222-3333-4444555566667777'",
            &format!("U='{}'", INPUT.to_uppercase()),
        ),
        source.replace(
            "</ControlList>",
            &format!(
                "<C Type='Other' U='{}'/></ControlList>",
                OWNER.to_uppercase()
            ),
        ),
    ] {
        cases.push((malformed, "Trigger"));
    }
    for (xml, key) in cases {
        for invert in [false, true] {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &xml).unwrap();
                fs::write(&existing, b"preserve").unwrap();
                let mut edit = command(&input, key, invert);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                edit.assert().failure();
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                assert!(!new.exists());
            }
        }
    }
}

#[test]
fn up_down_counter_trigger_on_full_golden_project_preserves_every_unselected_byte() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("golden-counter.Loxone");
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs")
        .join("Guest Room Door.Loxone");
    let mut source = fs::read_to_string(golden).unwrap();
    let fixture = up_down_counter_fixture()
        .replace("Def='10'", "Def='37'")
        .replace("Def='5'", "Def='11'")
        .replace(
            "K='StartValue' U='a5555555-2222-3333-4444555566667777' Def='0'",
            "K='StartValue' U='a5555555-2222-3333-4444555566667777' Def='3'",
        )
        .replace(
            "K='Remanence' U='a4444444-2222-3333-4444555566667777' Def='0'",
            "K='Remanence' U='a4444444-2222-3333-4444555566667777' Def='1'",
        );
    let owner_start = fixture.find("<C Type=\"UpDownCounter\"").unwrap();
    let owner_end = owner_start + fixture[owner_start..].find("</C>").unwrap() + 4;
    let page = source
        .find("U=\"1fbf34a5-03b1-83a1-ffff9bb3c1479539\"")
        .unwrap();
    let insertion = page + source[page..].find('>').unwrap() + 1;
    source.insert_str(insertion, &fixture[owner_start..owner_end]);
    fs::write(&input, &source).unwrap();
    command(&input, "Trigger", true)
        .arg("-q")
        .assert()
        .success();
    let saved = fs::read_to_string(&input).unwrap();
    let selected = format!("<Co K=\"Trigger\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/>");
    let expected = source.replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"));
    let program = source.find("<C Type=\"Program\"").unwrap();
    assert_eq!(
        &saved[saved.find("<C Type=\"Program\"").unwrap()..],
        &expected[program..]
    );
    command(&input, "Trigger", true)
        .arg("-q")
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&input).unwrap(), saved);
}

#[test]
fn up_down_counter_owning_root_refuses_before_noop_stamp_or_any_destination_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let stages = [
        dir.path().join("input.Loxone.tmp"),
        dir.path().join("existing.Loxone.tmp"),
        dir.path().join("new.Loxone.tmp"),
    ];
    let source = up_down_counter_fixture();
    for (opening, closing) in [
        (
            "<C Type='Folder' U='c7777777-2222-3333-4444555566667777' V='121' WF='0'>",
            "</C>",
        ),
        (
            "<C Type='Folder' U='c7777777-2222-3333-4444555566667777' V='121' WF='536870912'>",
            "</C>",
        ),
        ("<Loxone>", "</Loxone>"),
        (
            "<ControlList Type='Folder' V='121' WF='16384'>",
            "</ControlList>",
        ),
    ] {
        let xml = source
            .replace("<ControlList>", opening)
            .replace("</ControlList>", closing);
        for inverted in [false, true] {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &xml).unwrap();
                fs::write(&existing, b"preserve destination").unwrap();
                for stage in &stages {
                    fs::write(stage, b"preserve foreign stage").unwrap();
                }
                let mut edit = command(&input, "Trigger", inverted);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                edit.assert().failure().stderr(predicates::str::contains(
                    "UpDownCounter input inversion requires",
                ));
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"preserve destination");
                assert!(!new.exists());
                for stage in &stages {
                    assert_eq!(fs::read(stage).unwrap(), b"preserve foreign stage");
                }
            }
        }
    }
}

#[test]
fn up_down_counter_qualified_owning_roots_preserve_noops_and_existing_native_data() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for document_is_root in [false, true] {
        for with_program in [false, true] {
            let mut source = up_down_counter_fixture();
            if document_is_root {
                source = source
                    .replace("<ControlList>", "")
                    .replace("</ControlList>", "");
            }
            if !with_program {
                let start = source.find("<C Type=\"Program\"").unwrap();
                let end = start + source[start..].find('>').unwrap() + 1;
                source.replace_range(start..end, "");
                source = source.replacen(
                    "</C></C></C><C Type=\"OutputRef\"",
                    "</C></C><C Type=\"OutputRef\"",
                    1,
                );
            }
            fs::write(&input, &source).unwrap();
            let selected = format!("<Co K=\"Trigger\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/>");
            let body_start = source.find("<C Type=\"Page\"").unwrap();
            for inverted in [false, true] {
                for destination in [&existing, &new] {
                    if destination == &existing {
                        fs::write(destination, b"replace only after admission").unwrap();
                    } else if destination.exists() {
                        fs::remove_file(destination).unwrap();
                    }
                    command(&input, "Trigger", inverted)
                        .arg("--save-as")
                        .arg(destination)
                        .arg("-q")
                        .assert()
                        .success();
                    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                    let saved = fs::read_to_string(destination).unwrap();
                    if inverted {
                        assert_eq!(
                            &saved[saved.find("<C Type=\"Page\"").unwrap()..],
                            source[body_start..]
                                .replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"))
                        );
                    } else {
                        assert_eq!(saved, source);
                    }
                    command(destination, "Trigger", inverted)
                        .arg("-q")
                        .assert()
                        .success();
                    assert_eq!(fs::read_to_string(destination).unwrap(), saved);
                }
            }
        }
    }
}

fn step_sel_fixture() -> String {
    let golden =
        include_str!("eval/golden/configs/Fan_Control_-_Presence_and_Temperature_Support.Loxone");
    let start = golden.find("<C Type=\"StepSel\"").unwrap();
    let end = start + golden[start..].find("</C>").unwrap() + 4;
    let mut owner = golden[start..end]
        .replace("1f3ef44e-014f-e285-ffffc074cd9342e0", OWNER)
        .replace("WF=\"64\"", "WF=\"16384\"")
        .replace(
            "Nio=\"42\"",
            "Nio=\"42\" Unknown='12hTF=quoted' Rights='keep'",
        );
    let reset = owner.find("<Co K=\"Reset\"").unwrap();
    let reset_end = reset + owner[reset..].find("</Co>").unwrap() + 5;
    owner.replace_range(
        reset..reset_end,
        "<Co K=\"Reset\" U=\"1f3ef44e-014f-e26e-13ff61ca91a4e69c\" Def='0' Nc='0'/>",
    );
    owner = owner.replace(
        "<Co K=\"Q7\"",
        "<Co Inv='true' Unknown='keep-output' K=\"Q7\"",
    );
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList><C Type=\"Document\" U=\"01111111-2222-3333-4444555566667777\" V=\"178\" DateS=\"2147483600\" Date=\"old\"><C Type=\"Program\" U=\"81111111-2222-3333-4444555566667777\" V=\"178\" WF=\"16384\"><C Type=\"Page\" U=\"31111111-2222-3333-4444555566667777\" V=\"178\" WF=\"16384\">{owner}</C></C><SET><Password t='15' v='untouched'/></SET></C></ControlList>\r\n"
    )
}

fn step_sel_port(source: &str, key: &str) -> String {
    let start = source.find(&format!("<Co K=\"{key}\"")).unwrap();
    let end = start + source[start..].find("/>").unwrap() + 2;
    source[start..end].to_string()
}

#[test]
fn step_sel_native_ports_preserve_full_saved_layout_and_only_stamp_changes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for key in ["InputTriggerP", "InputTriggerM", "Reset"] {
        let source = step_sel_fixture();
        let selected = step_sel_port(&source, key);
        for destination in [None, Some(&output)] {
            fs::write(&input, &source).unwrap();
            let mut edit = command(&input, key, false);
            if let Some(destination) = destination {
                edit.arg("--save-as").arg(destination);
            }
            edit.arg("-q").assert().success();
            let saved = destination.unwrap_or(&input);
            assert_eq!(fs::read(saved).unwrap(), source.as_bytes());
            let mut edit = command(&input, key, true);
            if let Some(destination) = destination {
                edit.arg("--save-as").arg(destination);
            }
            edit.arg("-q").assert().success();
            let changed = fs::read_to_string(saved).unwrap();
            let expected = source.replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"));
            let body = changed.find("<C Type=\"Program\"").unwrap();
            assert_eq!(
                &changed[body..],
                &expected[expected.find("<C Type=\"Program\"").unwrap()..]
            );
            assert!(changed.starts_with("\u{feff}<?xml"));
            assert!(changed.contains("DateS=\"2147483601\""));
            assert!(!changed.contains("Date=\"old\""));
            if destination.is_some() {
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            }
            command(saved, key, true).arg("-q").assert().success();
            assert_eq!(fs::read_to_string(saved).unwrap(), changed);
            command(saved, key, false).arg("-q").assert().success();
            let cleared = fs::read_to_string(saved).unwrap();
            let expected = source.replace(&selected, &selected.replace("/>", " />"));
            let body = cleared.find("<C Type=\"Program\"").unwrap();
            assert_eq!(
                &cleared[body..],
                &expected[expected.find("<C Type=\"Program\"").unwrap()..]
            );
            assert!(cleared.contains("DateS=\"2147483602\""));
        }
    }
}

#[test]
fn step_sel_admits_only_qualified_roots_and_preserves_noop_bytes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for document_is_root in [false, true] {
        for with_program in [false, true] {
            let mut source = step_sel_fixture();
            if document_is_root {
                source = source
                    .replace("<ControlList>", "")
                    .replace("</ControlList>", "");
            }
            if !with_program {
                let start = source.find("<C Type=\"Program\"").unwrap();
                let end = start + source[start..].find('>').unwrap() + 1;
                source.replace_range(start..end, "");
                source = source.replace("</C></C><SET>", "</C><SET>");
            }
            for bom in [false, true] {
                let source = if bom {
                    source.clone()
                } else {
                    source.trim_start_matches('\u{feff}').to_string()
                };
                fs::write(&input, &source).unwrap();
                command(&input, "InputTriggerP", false)
                    .arg("-q")
                    .assert()
                    .success();
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                command(&input, "InputTriggerP", true)
                    .arg("-q")
                    .assert()
                    .success();
                let changed = fs::read_to_string(&input).unwrap();
                let owner = source.find("<C Type=\"StepSel\"").unwrap();
                let selected = step_sel_port(&source, "InputTriggerP");
                assert_eq!(
                    &changed[changed.find("<C Type=\"StepSel\"").unwrap()..],
                    &source[owner..].replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"))
                );
            }
        }
    }
}

#[test]
fn step_sel_capability_identity_layout_and_context_refusals_are_atomic_even_on_noop() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let stages = [
        dir.path().join("input.Loxone.tmp"),
        dir.path().join("existing.Loxone.tmp"),
        dir.path().join("new.Loxone.tmp"),
    ];
    let source = step_sel_fixture();
    let selected = step_sel_port(&source, "InputTriggerP");
    let mut cases: Vec<(String, &str)> = [
        "InputTrigger 1",
        "InputTrigger 16",
        "InputSel",
        "InputDisable",
        "Remanence",
        "Max",
        "Mode",
        "Q1",
        "OutputAPI",
        "inputtriggerp",
    ]
    .map(|key| (source.clone(), key))
    .into();
    for (from, to) in [
        ("Type=\"StepSel\"", "Type=\"Radio2\""),
        ("Nio=\"42\"", "Nio=\"41\""),
        ("K=\"InputTrigger 16\"", "K=\"InputTrigger 15\""),
        ("<ControlList>", "<ControlList Type='Folder'>"),
        ("<ControlList>", "<ControlList xmlns='urn:not-native'>"),
        ("DateS=\"2147483600\"", "DateS=\"2147483647\""),
        ("Unknown='12hTF=quoted'", "Unknown='before 12hTF=quoted'"),
        ("WF=\"16384\" T1=", "WF=\"64\" T1="),
    ] {
        assert!(source.contains(from), "missing refusal setup: {from}");
        cases.push((source.replace(from, to), "InputTriggerP"));
    }
    for replacement in [
        selected.replace("/>", " Nc='1'/>"),
        selected.replace("/>", " Def='1'/>"),
        selected.replace("/>", " Def='00'/>"),
        selected.replace("/>", " Inv='1'/>"),
        selected.replace("/>", "><In Input='keep-wire'/></Co>"),
        selected.replace("InputTriggerP", "InputTrigger&#80;"),
        selected.replace("1f3ef44e-014f-e26b-10ff61ca91a4e69c", OWNER),
        String::new(),
    ] {
        cases.push((source.replace(&selected, &replacement), "InputTriggerP"));
    }
    for (opening, closing) in [
        ("<Loxone>", "</Loxone>"),
        (
            "<C Type='Folder' U='c7777777-2222-3333-4444555566667777' V='178' WF='536870912'>",
            "</C>",
        ),
    ] {
        cases.push((
            source
                .replace("<ControlList>", opening)
                .replace("</ControlList>", closing),
            "InputTriggerP",
        ));
    }
    cases.push((format!("\u{feff}{source}"), "InputTriggerP"));
    cases.push((
        source.replace(
            "<SET>",
            "<C Type='Folder' U='a1111111-2222-3333-4444-555566667777'/><SET>",
        ),
        "InputTriggerP",
    ));
    for (index, (xml, key)) in cases.into_iter().enumerate() {
        for inverted in [false, true] {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &xml).unwrap();
                fs::write(&existing, b"preserve destination").unwrap();
                for stage in &stages {
                    fs::write(stage, b"preserve foreign stage").unwrap();
                }
                let mut edit = command(&input, key, inverted);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                edit.assert().failure().stderr(
                    predicates::str::contains("nothing was changed")
                        .or(predicates::str::contains("Nothing was changed")),
                );
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes(), "case {index}");
                assert_eq!(fs::read(&existing).unwrap(), b"preserve destination");
                assert!(!new.exists(), "case {index}");
                for stage in &stages {
                    assert_eq!(fs::read(stage).unwrap(), b"preserve foreign stage");
                }
            }
        }
    }
}

fn monoflop_fixture() -> String {
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs")
        .join("Guest Room Door.Loxone");
    let mut source = format!(
        "\u{feff}{}",
        fs::read_to_string(golden)
            .unwrap()
            .trim_start_matches('\u{feff}')
    );
    source = source.replacen("?>", "?>\r\n<?audit native-input?>\r\n<!--keep-->", 1);
    let page = source
        .find("U=\"1fbf34a5-03b1-83a1-ffff9bb3c1479539\"")
        .unwrap();
    let insertion = page + source[page..].find('>').unwrap() + 1;
    source.insert_str(
        insertion,
        &format!(
            "\r\n<C Type=\"Monoflop\" V=\"174\" U=\"{OWNER}\" WF=\"16384\" Nio=\"5\" Title=\"Pulse\" Cx=\"440\" Cy=\"280\" Pr=\"keep-rights\" Unknown=\"été\">
<Co K=\"InputTrigger\" U=\"{INPUT}\" Def=\"0\" Nc=\"0\" Unknown='keep-input'/>
<Co K=\"Reset\" U=\"41111111-2222-3333-4444555566667777\" Inv=\"true\" Nc=\"1\"><Ref U=\"d1111111-2222-3333-4444555566667777\" Unknown=\"keep-reset-wire\"/></Co>
<Co K=\"Remanence\" U=\"81111111-2222-3333-4444555566667777\" Def=\"1\" Unknown=\"keep-param\"/>
<Co K=\"Time\" U=\"91111111-2222-3333-4444555566667777\" Def=\"2\"/>
<Co K=\"Q\" U=\"51111111-2222-3333-4444555566667777\" Unknown=\"keep-output\"/>
</C>
<C Type=\"Not\" V=\"174\" U=\"c1111111-2222-3333-4444555566667777\" WF=\"16384\" Nio=\"2\"><Co K=\"I\" U=\"e1111111-2222-3333-4444555566667777\"/><Co K=\"Q\" U=\"d1111111-2222-3333-4444555566667777\"/></C>
<C Type=\"And\" V=\"174\" U=\"61111111-2222-3333-4444555566667777\" WF=\"16384\" Nio=\"3\"><Co K=\"I1\" U=\"71111111-2222-3333-4444555566667777\" Nc=\"1\"><Ref U=\"51111111-2222-3333-4444555566667777\" Unknown=\"keep-output-wire\"/></Co><Co K=\"I2\" U=\"f1111111-2222-3333-4444555566667777\"/><Co K=\"Q\" U=\"a2222222-2222-3333-4444555566667777\"/></C>\r\n"
        ),
    );
    source
}

fn monoflop_expected_stamp(source: &str, saved: &str) -> String {
    let before = xmltree::Element::parse(source.trim_start_matches('\u{feff}').as_bytes()).unwrap();
    let after = xmltree::Element::parse(saved.trim_start_matches('\u{feff}').as_bytes()).unwrap();
    let document = "1fbf3415-030f-bc7d-ffff9bb3c1479539";
    let before = control(&before, document).unwrap();
    let after = control(&after, document).unwrap();
    let mut expected = source.to_string();
    for key in ["Date", "DateS"] {
        expected = expected.replacen(
            &format!("{key}=\"{}\"", before.attributes[key]),
            &format!("{key}=\"{}\"", after.attributes[key]),
            1,
        );
    }
    expected
}

#[test]
fn monoflop_input_trigger_preserves_full_project_defaults_wires_and_atomic_copies() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    for time in ["2", "17.25"] {
        let mut source = monoflop_fixture().replacen("Def=\"2\"", &format!("Def=\"{time}\""), 1);
        if time != "2" {
            source = source.replacen(" Def=\"0\" Nc=\"0\"", " Nc=\"0\"", 1);
        }
        fs::write(&input, &source).unwrap();
        for existing in [false, true] {
            if existing {
                fs::write(&output, b"PRESERVE UNTIL ACCEPTED").unwrap();
            } else if output.exists() {
                fs::remove_file(&output).unwrap();
            }
            command(&input, "InputTrigger", false)
                .arg("--save-as")
                .arg(&output)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
            if !existing {
                fs::remove_file(&output).unwrap();
            }
            command(&input, "InputTrigger", true)
                .arg("--save-as")
                .arg(&output)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            let saved = fs::read_to_string(&output).unwrap();
            let expected = monoflop_expected_stamp(&source, &saved).replacen(
                "Unknown='keep-input'/>",
                "Unknown='keep-input' Inv=\"true\"/>",
                1,
            );
            assert_eq!(saved, expected);
        }
        let result = command(&input, "InputTrigger", true)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["input_inversion"]["connector"], "InputTrigger");
        assert_eq!(result["input_inversion"]["changed"], true);
        let once = fs::read(&input).unwrap();
        command(&input, "InputTrigger", true)
            .arg("-q")
            .assert()
            .success();
        assert_eq!(fs::read(&input).unwrap(), once);
        command(&input, "InputTrigger", false)
            .arg("-q")
            .assert()
            .success();
        let cleared = fs::read_to_string(&input).unwrap();
        assert_eq!(
            cleared,
            monoflop_expected_stamp(&source, &cleared).replacen(
                "Unknown='keep-input'/>",
                "Unknown='keep-input' />",
                1,
            )
        );
    }
}

#[test]
fn monoflop_input_inversion_refuses_unqualified_ports_layout_and_context_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = monoflop_fixture();
    let mut cases: Vec<(String, &str)> = [
        "Reset",
        "Remanence",
        "Time",
        "Q",
        "I1",
        "Trigger",
        "inputtrigger",
    ]
    .map(|key| (source.clone(), key))
    .into();
    for (from, to) in [
        ("Nio=\"5\" Title=\"Pulse\"", "Nio=\"4\" Title=\"Pulse\""),
        ("K=\"Time\"", "K=\"Duration\""),
        ("K=\"Time\"", "K=\"Reset\""),
        (
            "Def=\"0\" Nc=\"0\" Unknown='keep-input'",
            "Def=\"1\" Nc=\"0\"",
        ),
        (
            "Def=\"0\" Nc=\"0\" Unknown='keep-input'",
            "Def=\"0\" Nc=\"1\"",
        ),
        (
            "Unknown='keep-input'/>",
            "Unknown='keep-input'><Ref U=\"d1111111-2222-3333-4444555566667777\"/></Co>",
        ),
        ("Title=\"Pulse\"", "Title=\"Pulse\" IfT=\"false\""),
        ("Title=\"Pulse\"", "Title=\"Pulse\" uuidParent=\"x\""),
        ("WF=\"16384\" Nio=\"5\"", "WF=\"16385\" Nio=\"5\""),
        ("Type=\"Program\"", "Type=\"Folder\""),
        ("Type=\"Document\"", "Type=\"DocumentTemplate\""),
        ("<ControlList ", "<Folder "),
        ("</ControlList>", "</Folder>"),
        (
            "Type=\"Document\" V=",
            "Type=\"Document\" Type=\"Document\" V=",
        ),
        (
            "Unknown='keep-input'",
            "Unknown='keep-input' Inv=\"false\" Inv=\"false\"",
        ),
        ("Unknown='keep-input'", "Unknown='Noon 12hTF=quoted'"),
        ("<?audit native-input?>", "<?1bad?>"),
        ("Title=\"Pulse\"", "xmlns=\"urn:wrong\" Title=\"Pulse\""),
        ("DateS=\"532627050\"", "DateS=\"+1\""),
        ("DateS=\"532627050\"", "DateS=\"2147483647\""),
        (
            "U=\"b1111111-2222-3333-4444555566667777\"",
            "U=\"b1111111-2222-3333-4444-555566667777\"",
        ),
        (
            "U=\"41111111-2222-3333-4444555566667777\"",
            "U=\"B1111111-2222-3333-4444555566667777\"",
        ),
    ] {
        assert!(source.contains(from), "missing refusal fixture {from}");
        cases.push((source.replacen(from, to, 1), "InputTrigger"));
    }
    cases.push((
        source
            .replace("<ControlList ", "<C Type=\"Folder\" ")
            .replace("</ControlList>", "</C>"),
        "InputTrigger",
    ));
    cases.push((
        source.replace(
            "<Co K=\"Time\" U=\"91111111-2222-3333-4444555566667777\" Def=\"2\"/>",
            "",
        ),
        "InputTrigger",
    ));
    cases.push((format!("{source}<C Type=\"Monoflop\"/>"), "InputTrigger"));
    cases.push((format!("\u{feff}{source}"), "InputTrigger"));
    for (case_index, (xml, key)) in cases.into_iter().enumerate() {
        for invert in [false, true] {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &xml).unwrap();
                fs::write(&existing, b"PRESERVE").unwrap();
                let mut edit = command(&input, key, invert);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                let result = edit.output().unwrap();
                assert!(
                    !result.status.success(),
                    "Monoflop refusal case {case_index}, key {key}, inverted {invert}, destination {destination:?}: {}",
                    String::from_utf8_lossy(&result.stdout)
                );
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"PRESERVE");
                assert!(!new.exists());
            }
        }
    }
}

fn delay_fixture(owner_type: &str) -> String {
    let golden = include_str!("eval/golden/configs/Guest Room Door.Loxone");
    let page = golden.find("<C Type=\"Page\"").expect("golden native Page");
    let insertion = page + golden[page..].find('>').unwrap() + 1;
    let (nio, parameters) = match owner_type {
        "OnDelay" => (
            5,
            "<Co K=\"Time\" U=\"81111111-2222-3333-4444555566667777\" Def=\"1\"/>",
        ),
        "OffDelay" => (
            5,
            "<Co K=\"Time\" U=\"81111111-2222-3333-4444555566667777\" Def=\"7.5\"/>",
        ),
        "OnOffDelay" => (
            6,
            "<Co K=\"TimeOn\" U=\"81111111-2222-3333-4444555566667777\" Def=\"1\"/><Co K=\"TimeOff\" U=\"91111111-2222-3333-4444555566667777\" Def=\"9.25\"/>",
        ),
        _ => unreachable!(),
    };
    let blocks = format!(
        r#"
        <C Type="{owner_type}" V="174" U="{OWNER}" WF="16384" Nio="{nio}" Title="Recovered delay input" Pos="490,280" Pr="4" Unknown="keep &amp; preserve">
          <Co K="InputTrigger" U="{INPUT}" Def="0" Inv='false' Unknown="selected"/>
          <Co K="Reset" U="41111111-2222-3333-4444555566667777" Nc="1" Inv="true"><Ref U="71111111-2222-3333-4444555566667777" Unknown="keep reset wire"/></Co>
          <Co K="Remanence" U="61111111-2222-3333-4444555566667777" Def="1"/>
          {parameters}
          <Co K="Q" U="51111111-2222-3333-4444555566667777" Unknown="keep output"/>
        </C>
        <C Type="Not" V="174" U="c1111111-2222-3333-4444555566667777" WF="16384" Nio="2" Pos="220,280"><Co K="I" U="d1111111-2222-3333-4444555566667777"/><Co K="Q" U="71111111-2222-3333-4444555566667777"/></C>
        <C Type="OutputRef" V="174" U="e1111111-2222-3333-4444555566667777" WF="16384" Pos="730,280"><Co K="AI" U="f1111111-2222-3333-4444555566667777" Nc="1"><Ref U="51111111-2222-3333-4444555566667777" Unknown="keep output wire"/></Co></C>
"#
    );
    let source = format!("{}{}{}", &golden[..insertion], blocks, &golden[insertion..]);
    format!(
        "\u{feff}{}<!--preserve--><?audit delay?>\r\n",
        source
            .trim_start_matches('\u{feff}')
            .replace("\r\n", "\n")
            .replace('\n', "\r\n")
    )
}

fn with_saved_stamp(source: &str, saved: &str) -> String {
    let original = xmltree::Element::parse(source.as_bytes()).unwrap();
    let output = xmltree::Element::parse(saved.as_bytes()).unwrap();
    let uuid = "1fbf3415-030f-bc7d-ffff9bb3c1479539";
    let before = control(&original, uuid).unwrap();
    let after = control(&output, uuid).unwrap();
    let mut expected = source.to_string();
    for field in ["Date", "DateS"] {
        expected = expected.replacen(
            &format!(" {field}=\"{}\"", before.attributes[field]),
            &format!(" {field}=\"{}\"", after.attributes[field]),
            1,
        );
    }
    expected
}

#[test]
fn delay_input_inversion_preserves_full_native_project_defaults_and_other_wires() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("delay.Loxone");
    let output = dir.path().join("copy.Loxone");
    for owner_type in ["OnDelay", "OffDelay", "OnOffDelay"] {
        let source = delay_fixture(owner_type);
        fs::write(&input, &source).unwrap();
        let report = command(&input, "InputTrigger", false)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let report: serde_json::Value = serde_json::from_slice(&report).unwrap();
        assert_eq!(report["saved_to"], serde_json::Value::Null);
        assert_eq!(report["input_inversion"]["changed"], false);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        for existing in [false, true] {
            if existing {
                fs::write(&output, b"preserve until accepted").unwrap();
            } else if output.exists() {
                fs::remove_file(&output).unwrap();
            }
            command(&input, "InputTrigger", false)
                .arg("--save-as")
                .arg(&output)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
            if !existing {
                fs::remove_file(&output).unwrap();
            }
            command(&input, "InputTrigger", true)
                .arg("--save-as")
                .arg(&output)
                .arg("-q")
                .assert()
                .success();
            let saved = fs::read_to_string(&output).unwrap();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            assert_eq!(
                saved,
                with_saved_stamp(&source, &saved).replacen("Inv='false'", "Inv='true'", 1)
            );
            command(&output, "InputTrigger", true)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read_to_string(&output).unwrap(), saved);
            command(&output, "InputTrigger", false)
                .arg("-q")
                .assert()
                .success();
            let cleared = fs::read_to_string(&output).unwrap();
            assert_eq!(
                cleared,
                with_saved_stamp(&saved, &cleared).replacen("Inv='true'", "", 1)
            );
        }
    }
}

#[test]
fn delay_input_inversion_refuses_unqualified_ports_layout_and_context_before_any_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("delay.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for owner_type in ["OnDelay", "OffDelay", "OnOffDelay"] {
        let source = delay_fixture(owner_type);
        let mut cases: Vec<(String, &str)> =
            ["Reset", "Remanence", "Time", "TimeOn", "TimeOff", "Q", "I1"]
                .into_iter()
                .map(|key| (source.clone(), key))
                .collect();
        for xml in [
            source.replace("K=\"InputTrigger\"", "K=\"Trigger\""),
            source
                .replace("Nio=\"5\"", "Nio=\"4\"")
                .replace("Nio=\"6\"", "Nio=\"5\""),
            source.replace("<Co K=\"Remanence\"", "<Co K=\"Unknown\""),
            source.replace("Def=\"0\" Inv='false'", "Def=\"1\" Inv='false'"),
            source.replace("Def=\"0\" Inv='false'", "Def=\"0\" Nc=\"1\" Inv='false'"),
            source.replace("Type=\"Program\"", "Type=\"Folder\""),
            source
                .replace("<ControlList ", "<Folder ")
                .replace("</ControlList>", "</Folder>"),
            source.replace(
                "Title=\"Recovered delay input\"",
                "Title=\"Noon 12hTF=quoted\"",
            ),
            source.replace(INPUT, OWNER),
            source.replace("Unknown=\"selected\"", "Unknown=\"selected\" IfT=\"false\""),
            source.replace("DateS=\"532627050\"", "DateS=\"2147483647\""),
            source.replace(
                "K=\"InputTrigger\"",
                "K=\"InputTrigger\" K=\"InputTrigger\"",
            ),
            source
                .replace("Type=\"OnDelay\"", "Type=\"ondelay\"")
                .replace("Type=\"OffDelay\"", "Type=\"offdelay\"")
                .replace("Type=\"OnOffDelay\"", "Type=\"onoffdelay\""),
        ] {
            cases.push((xml, "InputTrigger"));
        }
        for (xml, key) in cases {
            for inverted in [false, true] {
                for destination in [None, Some(&existing), Some(&new)] {
                    fs::write(&input, &xml).unwrap();
                    fs::write(&existing, b"preserve").unwrap();
                    let mut edit = command(&input, key, inverted);
                    if let Some(destination) = destination {
                        edit.arg("--save-as").arg(destination);
                    }
                    edit.assert().failure();
                    assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                    assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                    assert!(!new.exists());
                }
            }
        }
    }
}

fn latch_fragment(owner_type: &str, selected: &str, remanence: u8) -> String {
    let nio = if owner_type == "FlipFlop" { 4 } else { 5 };
    let input = |key: &str, uuid: &str| {
        if key == selected {
            format!("<Co K=\"{key}\" U=\"{uuid}\" Def=\"0\" Pos='11,22' Unknown=\"selected\"/>")
        } else {
            format!(
                "<Co K=\"{key}\" U=\"{uuid}\" Def=\"1\" Inv=\"true\" Nc=\"1\" Pos='33,44'><Ref U=\"d1111111-2222-3333-4444555566667777\" Unknown=\"input-wire\"/></Co>"
            )
        }
    };
    let trigger = if owner_type == "FlipFlop" {
        String::new()
    } else {
        "<Co K=\"InputTrigger\" U=\"c1111111-2222-3333-4444555566667777\" Def=\"0\" Nc=\"1\"><Ref U=\"d1111111-2222-3333-4444555566667777\"/></Co>".into()
    };
    let input_s = input("InputS", INPUT);
    let input_r = input("InputR", "41111111-2222-3333-4444555566667777");
    format!(
        "<C Type=\"And\" U=\"91111111-2222-3333-4444555566667777\" V=\"121\" WF=\"16384\" Nio=\"3\"><Co K=\"I1\" U=\"92111111-2222-3333-4444555566667777\"/><Co K=\"I2\" U=\"93111111-2222-3333-4444555566667777\"/><Co K=\"Q\" U=\"d1111111-2222-3333-4444555566667777\"/></C><C Type=\"{owner_type}\" U=\"{OWNER}\" V=\"121\" WF=\"16384\" Nio=\"{nio}\" Title=\"Latch\" Position=\"400,500\" Rights=\"keep\">{input_s}{trigger}{input_r}<Co K=\"Remanence\" U=\"e1111111-2222-3333-4444555566667777\" Def=\"{remanence}\" Pos='55,66' Unknown=\"parameter\"/><Co K=\"Q\" U=\"51111111-2222-3333-4444555566667777\" Inv=\"true\" Pos='77,88'/></C><C Type=\"OutputRef\" U=\"61111111-2222-3333-4444555566667777\" V=\"121\" WF=\"16384\"><Co K=\"AI\" U=\"71111111-2222-3333-4444555566667777\" Nc=\"1\"><Ref U=\"51111111-2222-3333-4444555566667777\" Unknown=\"output-wire\"/></Co></C>"
    )
}

fn latch_project(owner_type: &str, selected: &str, remanence: u8) -> String {
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs")
        .join("Guest Room Door.Loxone");
    let mut source = fs::read_to_string(golden).unwrap();
    let page_uuid = source
        .find("U=\"1fbf34a5-03b1-83a1-ffff9bb3c1479539\"")
        .unwrap();
    let start = source[..page_uuid].rfind("<C ").unwrap();
    let end = start + source[start..].find('>').unwrap() + 1;
    assert!(source[start..end].contains("Type=\"Page\""));
    source.insert_str(end, &latch_fragment(owner_type, selected, remanence));
    source
}

fn without_document_stamp(source: &str) -> String {
    let mut source = source.to_string();
    let kind = source.find("Type=\"Document\"").unwrap();
    let start = source[..kind].rfind("<C ").unwrap();
    for field in [" DateS=", " Date="] {
        let end = start + source[start..].find('>').unwrap();
        if let Some(position) = source[start..end].find(field) {
            let position = start + position;
            let value = position + field.len();
            let quote = source.as_bytes()[value] as char;
            let last = value + 1 + source[value + 1..].find(quote).unwrap();
            source.replace_range(position..last + 1, "");
        }
    }
    source
}

#[test]
fn latch_inputs_preserve_full_project_unselected_wires_defaults_and_atomic_copies() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("latch.Loxone");
    let output = dir.path().join("copy.Loxone");
    for owner_type in ["FlipFlop", "RSFlipFlop", "SRFlipFlop"] {
        for key in ["InputS", "InputR"] {
            for remanence in [0, 1] {
                let source = latch_project(owner_type, key, remanence);
                let uuid = if key == "InputS" {
                    INPUT
                } else {
                    "41111111-2222-3333-4444555566667777"
                };
                let selected = format!(
                    "<Co K=\"{key}\" U=\"{uuid}\" Def=\"0\" Pos='11,22' Unknown=\"selected\"/>"
                );
                let expected =
                    source.replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"));
                fs::write(&input, &source).unwrap();
                for existing in [false, true] {
                    if existing {
                        fs::write(&output, b"preserve until accepted").unwrap();
                    } else if output.exists() {
                        fs::remove_file(&output).unwrap();
                    }
                    command(&input, key, false)
                        .arg("--save-as")
                        .arg(&output)
                        .arg("-q")
                        .assert()
                        .success();
                    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
                    if !existing {
                        fs::remove_file(&output).unwrap();
                    }
                    let result = command(&input, key, true)
                        .arg("--save-as")
                        .arg(&output)
                        .args(["-o", "json"])
                        .assert()
                        .success()
                        .get_output()
                        .stdout
                        .clone();
                    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
                    assert_eq!(result["input_inversion"]["connector"], key);
                    assert_eq!(result["input_inversion"]["connector_uuid"], uuid);
                    assert_eq!(result["input_inversion"]["changed"], true);
                    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                    assert_eq!(
                        without_document_stamp(&fs::read_to_string(&output).unwrap()),
                        without_document_stamp(&expected)
                    );
                }
                command(&input, key, true).arg("-q").assert().success();
                let once = fs::read(&input).unwrap();
                command(&input, key, true).arg("-q").assert().success();
                assert_eq!(fs::read(&input).unwrap(), once);
                command(&input, key, false).arg("-q").assert().success();
                let expected = source.replace(&selected, &selected.replace("/>", " />"));
                assert_eq!(
                    without_document_stamp(&fs::read_to_string(&input).unwrap()),
                    without_document_stamp(&expected)
                );
            }
        }
    }
}

fn counter_fixture(owner_type: &str) -> (&'static str, String) {
    let (key, nio, extra) = if owner_type == "Counter" {
        (
            "Trigger",
            "7",
            "<Co K=\"Remanence\" U=\"91111111-2222-3333-4444555566667777\" Def=\"1\"/><Co K=\"EndValue\" U=\"a2111111-2222-3333-4444555566667777\" Def=\"1000\"/><Co K=\"Mode\" U=\"a3111111-2222-3333-4444555566667777\" Def=\"0\"/>",
        )
    } else {
        (
            "InputEnable",
            "10",
            "<Co K=\"InputResetAll\" U=\"91111111-2222-3333-4444555566667777\" Inv=\"true\"/><Co K=\"Unit\" U=\"a2111111-2222-3333-4444555566667777\" Def=\"2\"/><Co K=\"Maintenance\" U=\"a3111111-2222-3333-4444555566667777\" Def=\"720\"/><Co K=\"AQl\" U=\"a4111111-2222-3333-4444555566667777\"/><Co K=\"AQr\" U=\"a5111111-2222-3333-4444555566667777\"/><Co K=\"AQs\" U=\"a6111111-2222-3333-4444555566667777\"/>",
        )
    };
    let source = logic_fixture(owner_type)
        .replace("Nio=\"3\"", &format!("Nio=\"{nio}\""))
        .replace("K=\"I1\"", &format!("K=\"{key}\""))
        .replace(
            "<Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\"/>",
            &format!(
                "<Co K=\"Reset\" U=\"41111111-2222-3333-4444555566667777\" Nc=\"1\" Inv=\"true\" Def=\"1\"><Ref U=\"51111111-2222-3333-4444555566667777\" Unknown=\"reset-wire\"/></Co>{extra}<Co K=\"AQ\" U=\"a7111111-2222-3333-4444555566667777\" Unknown=\"analog-output\"/>"
            ),
        );
    (key, source)
}

#[test]
fn counter_inputs_preserve_other_wires_defaults_and_atomic_destinations() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    for owner_type in ["Counter", "HourCounter"] {
        let (key, source) = counter_fixture(owner_type);
        for existing in [false, true] {
            for inverted in [false, true] {
                fs::write(&input, &source).unwrap();
                if existing {
                    fs::write(&output, b"preserve until accepted").unwrap();
                } else if output.exists() {
                    fs::remove_file(&output).unwrap();
                }
                command(&input, key, inverted)
                    .arg("--save-as")
                    .arg(&output)
                    .arg("-q")
                    .assert()
                    .success();
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                let saved = fs::read_to_string(&output).unwrap();
                if inverted {
                    let selected =
                        format!("<Co K=\"{key}\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/>");
                    let expected =
                        source.replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"));
                    assert_eq!(
                        &saved[saved.find("<C Type=\"Program\"").unwrap()..],
                        &expected[expected.find("<C Type=\"Program\"").unwrap()..]
                    );
                } else {
                    assert_eq!(saved, source);
                }
            }
        }
        fs::write(&input, &source).unwrap();
        command(&input, key, false).arg("-q").assert().success();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        command(&input, key, true).arg("-q").assert().success();
        let once = fs::read(&input).unwrap();
        command(&input, key, true).arg("-q").assert().success();
        assert_eq!(fs::read(&input).unwrap(), once);
        command(&input, key, false).arg("-q").assert().success();
        let cleared = fs::read_to_string(&input).unwrap();
        let selected = format!("<Co K=\"{key}\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/>");
        let expected = source.replace(&selected, &selected.replace("/>", " />"));
        assert_eq!(
            &cleared[cleared.find("<C Type=\"Program\"").unwrap()..],
            &expected[expected.find("<C Type=\"Program\"").unwrap()..]
        );
    }
}

#[test]
fn latch_inputs_refuse_other_native_roles_layouts_and_guarded_context_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("latch.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for owner_type in ["FlipFlop", "RSFlipFlop", "SRFlipFlop"] {
        let source = latch_project(owner_type, "InputS", 1);
        let nio = if owner_type == "FlipFlop" { 4 } else { 5 };
        let selected = format!("K=\"InputS\" U=\"{INPUT}\" Def=\"0\"");
        let mut cases = vec![
            (source.clone(), "I1"),
            (source.clone(), "InputTrigger"),
            (source.clone(), "Remanence"),
            (source.clone(), "Q"),
            (source.clone(), "inputs"),
            (source.clone(), "InputR"),
            (
                source.replace(
                    &format!("Nio=\"{nio}\" Title=\"Latch\""),
                    "Nio=\"3\" Title=\"Latch\"",
                ),
                "InputS",
            ),
            (source.replace("K=\"Remanence\"", "K=\"I2\""), "InputS"),
            (source.replace("K=\"Remanence\"", "K=\"InputR\""), "InputS"),
            (
                source.replace(
                    "<Co K=\"Remanence\" U=\"e1111111-2222-3333-4444555566667777\" Def=\"1\" Pos='55,66' Unknown=\"parameter\"/>",
                    "",
                ),
                "InputS",
            ),
            (
                source.replace(&selected, &selected.replace("Def=\"0\"", "Def=\"1\"")),
                "InputS",
            ),
            (
                source.replace(&selected, &format!("{selected} Nc=\"1\"")),
                "InputS",
            ),
            (
                source.replace("Title=\"Latch\"", "Title=\"Latch\" Dev=\"unknown\""),
                "InputS",
            ),
            (
                source.replace("Title=\"Latch\"", "Title=\"Latch\" IfT=\"false\""),
                "InputS",
            ),
            (
                source.replace("Title=\"Latch\"", "Title=\"Latch\" uuidParent=\"unknown\""),
                "InputS",
            ),
            (
                source.replace("Title=\"Latch\"", "Title=\"Noon 12hTF=quoted\""),
                "InputS",
            ),
            (
                source.replace("WF=\"16384\" Nio=", "WF=\"16385\" Nio="),
                "InputS",
            ),
            (
                source.replace("Type=\"Document\"", "Type=\"DocumentTemplate\""),
                "InputS",
            ),
            (
                source.replace("K=\"InputS\"", "K=\"InputS\" K=\"InputS\""),
                "InputS",
            ),
            (
                source.replace(
                    "<Ref U=\"d1111111-2222-3333-4444555566667777\"",
                    &format!("<Ref U=\"{INPUT}\""),
                ),
                "InputS",
            ),
            (
                source.replace("e1111111-2222-3333-4444555566667777", &INPUT.to_uppercase()),
                "InputS",
            ),
            (
                source.replace(
                    "e1111111-2222-3333-4444555566667777",
                    "b1111111-2222-3333-4444-555566667777",
                ),
                "InputS",
            ),
            (format!("{source}<C Type=\"Other\"/>"), "InputS"),
        ];
        if owner_type != "FlipFlop" {
            let neutral_trigger = source.replace(
                "<Co K=\"InputTrigger\" U=\"c1111111-2222-3333-4444555566667777\" Def=\"0\" Nc=\"1\"><Ref U=\"d1111111-2222-3333-4444555566667777\"/></Co>",
                "<Co K=\"InputTrigger\" U=\"c1111111-2222-3333-4444555566667777\" Def=\"0\"/>",
            );
            cases.push((neutral_trigger, "InputTrigger"));
            cases.push((
                source.replace("K=\"InputTrigger\"", "K=\"InputTriggerAlias\""),
                "InputS",
            ));
            cases.push((
                source.replace("K=\"InputTrigger\"", "Inv=\"true\" K=\"InputTrigger\""),
                "InputTrigger",
            ));
        }
        for (case, (xml, key)) in cases.into_iter().enumerate() {
            for inverted in [false, true] {
                for destination in [None, Some(&existing), Some(&new)] {
                    fs::write(&input, &xml).unwrap();
                    fs::write(&existing, b"preserve").unwrap();
                    let mut edit = command(&input, key, inverted);
                    if let Some(destination) = destination {
                        edit.arg("--save-as").arg(destination);
                    }
                    let result = edit.output().unwrap();
                    assert!(
                        !result.status.success(),
                        "{owner_type} case {case}, key {key}, inverted {inverted}: {}",
                        String::from_utf8_lossy(&result.stdout)
                    );
                    assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                    assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                    assert!(!new.exists());
                }
            }
        }
    }
}

#[test]
fn counter_inputs_refuse_other_native_ports_and_incomplete_layouts_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for owner_type in ["Counter", "HourCounter"] {
        let (key, source) = counter_fixture(owner_type);
        let mut cases: Vec<_> = [
            "I1",
            "InputTrigger",
            "Reset",
            "InputResetAll",
            "Remanence",
            "EndValue",
            "Unit",
            "Maintenance",
            "Q",
            "AQ",
            "OutputQ",
            if key == "Trigger" {
                "InputEnable"
            } else {
                "Trigger"
            },
        ]
        .into_iter()
        .map(|wrong| (source.clone(), wrong))
        .collect();
        for xml in [
            source.replace(
                if owner_type == "Counter" {
                    "Nio=\"7\""
                } else {
                    "Nio=\"10\""
                },
                "Nio=\"3\"",
            ),
            source.replace(
                "<Co K=\"AQ\" U=\"a7111111-2222-3333-4444555566667777\" Unknown=\"analog-output\"/>",
                "",
            ),
            source.replace("K=\"Q\"", "K=\"OutputQ\""),
            source.replace("Def=\"0\" Unknown=\"keep\"", "Def=\"1\" Unknown=\"keep\""),
            source.replace("Def=\"0\" Unknown=\"keep\"", "Nc=\"1\" Def=\"0\" Unknown=\"keep\""),
            source.replace("WF=\"16384\"", "WF=\"536870912\""),
            source.replace("DateS=\"1\"", "DateS=\"-1\""),
            source.replace("<Loxone>", "<Loxone><Template/>"),
            source.replace(
                "a7111111-2222-3333-4444555566667777",
                "B1111111-2222-3333-4444555566667777",
            ),
        ] {
            cases.push((xml, key));
        }
        for (xml, requested) in cases {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &xml).unwrap();
                fs::write(&existing, b"preserve").unwrap();
                let mut edit = command(&input, requested, false);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                edit.assert().failure();
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                assert!(!new.exists());
            }
        }
    }
}

#[test]
fn xor_not_inputs_preserve_raw_project_and_atomic_destinations() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    for (owner_type, key) in [("Xor", "I1"), ("Xor", "I2"), ("Not", "I")] {
        let source = logic_fixture(owner_type);
        fs::write(&input, &source).unwrap();
        for existing in [false, true] {
            if existing {
                fs::write(&output, b"preserve until accepted").unwrap();
            } else if output.exists() {
                fs::remove_file(&output).unwrap();
            }
            command(&input, key, false)
                .arg("--save-as")
                .arg(&output)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            if !existing {
                fs::remove_file(&output).unwrap();
            }
            command(&input, key, true)
                .arg("--save-as")
                .arg(&output)
                .arg("-q")
                .assert()
                .success();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            let saved = fs::read_to_string(&output).unwrap();
            let selected = if key == "I2" {
                "<Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\"/>".into()
            } else {
                format!("<Co K=\"{key}\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/>")
            };
            let expected = source.replace(&selected, &selected.replace("/>", " Inv=\"true\"/>"));
            let start = source.find("<C Type=\"Program\"").unwrap();
            assert_eq!(
                &saved[saved.find("<C Type=\"Program\"").unwrap()..],
                &expected[start..]
            );
        }
        let result = command(&input, key, true)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["input_inversion"]["connector"], key);
        assert_eq!(result["input_inversion"]["changed"], true);
        let once = fs::read(&input).unwrap();
        command(&input, key, true).arg("-q").assert().success();
        assert_eq!(fs::read(&input).unwrap(), once);
        command(&input, key, false).arg("-q").assert().success();
        let cleared = fs::read_to_string(&input).unwrap();
        assert!(!cleared.contains("Inv="));
        assert!(cleared.contains("Def=\"0\" Unknown=\"keep\""));
        assert!(cleared.contains("Unknown=\"keep-wire\""));
        assert!(cleared.contains("<Password t=\"15\" v=\"unchanged\"/>"));
    }
}

#[test]
fn xor_not_inputs_on_full_golden_project_preserve_nonzero_wf_and_wires() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("golden.Loxone");
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs")
        .join("Guest Room Door.Loxone");
    for (owner_type, keys) in [("Xor", &["I1", "I2"][..]), ("Not", &["I"][..])] {
        fs::copy(&golden, &input).unwrap();
        let mut add = assert_cmd::cargo::cargo_bin_cmd!("lox");
        let output = add
            .env("LOX_TELEMETRY", "0")
            .args(["config", "add"])
            .arg(&input)
            .args([
                "--type",
                owner_type,
                "--title",
                "Registered input regression",
                "--parent",
                "uuid:1fbf34a5-03b1-83a1-ffff9bb3c1479539",
                "-o",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let added: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let owner = added["uuid"].as_str().unwrap();
        let project = fs::read(&input).unwrap();
        let project = xmltree::Element::parse(project.as_slice()).unwrap();
        let source = control(&project, owner)
            .unwrap()
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .find(|co| co.attributes.get("K").map(String::as_str) == Some("Q"))
            .unwrap()
            .attributes["U"]
            .clone();
        let mut add_sink = assert_cmd::cargo::cargo_bin_cmd!("lox");
        let sink = add_sink
            .env("LOX_TELEMETRY", "0")
            .args(["config", "add"])
            .arg(&input)
            .args([
                "--type",
                "And",
                "--title",
                "Unselected connected sink",
                "--parent",
                "uuid:1fbf34a5-03b1-83a1-ffff9bb3c1479539",
                "-o",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let sink: serde_json::Value = serde_json::from_slice(&sink).unwrap();
        let mut wire = assert_cmd::cargo::cargo_bin_cmd!("lox");
        wire.env("LOX_TELEMETRY", "0")
            .args(["config", "wire-connector"])
            .arg(&input)
            .args([
                format!("uuid:{}.I1", sink["uuid"].as_str().unwrap()).as_str(),
                format!("uuid:{source}").as_str(),
                "-q",
            ])
            .assert()
            .success();
        let before = fs::read_to_string(&input).unwrap();
        let original = xmltree::Element::parse(before.as_bytes()).unwrap();
        assert_eq!(control(&original, owner).unwrap().attributes["WF"], "16384");
        assert!(before.contains("<In "));
        for key in keys {
            for flag in ["--inverted", "--no-inverted"] {
                let mut edit = assert_cmd::cargo::cargo_bin_cmd!("lox");
                edit.env("LOX_TELEMETRY", "0")
                    .args(["config", "set-input-inversion"])
                    .arg(&input)
                    .args([format!("uuid:{owner}").as_str(), key, flag, "-q"])
                    .assert()
                    .success();
            }
        }
        let after = fs::read_to_string(&input).unwrap();
        let saved = xmltree::Element::parse(after.as_bytes()).unwrap();
        for uuid in [
            "1fbf3415-030f-bc7d-ffff9bb3c1479539",
            "1fbf341a-00e9-c0af-ffff9bb3c1479539",
            "1fbf34a5-03b1-83a1-ffff9bb3c1479539",
            owner,
        ] {
            assert_eq!(control(&saved, uuid).unwrap().attributes["WF"], "16384");
        }
        let wires = |text: &str| {
            text.lines()
                .filter(|line| line.contains("<In ") || line.contains("<Ref "))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        assert_eq!(wires(&after), wires(&before));
    }
}

#[test]
fn counter_inputs_on_full_golden_project_preserve_nonzero_wf_and_wires() {
    fn append_owner(element: &mut xmltree::Element, owner: &xmltree::Element) -> bool {
        if element.attributes.get("U").map(String::as_str)
            == Some("1fbf34a5-03b1-83a1-ffff9bb3c1479539")
        {
            element
                .children
                .push(xmltree::XMLNode::Element(owner.clone()));
            return true;
        }
        element
            .children
            .iter_mut()
            .filter_map(xmltree::XMLNode::as_mut_element)
            .any(|child| append_owner(child, owner))
    }

    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("golden.Loxone");
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs")
        .join("Guest Room Door.Loxone");
    for owner_type in ["Counter", "HourCounter"] {
        let (key, source) = counter_fixture(owner_type);
        let source = xmltree::Element::parse(source.as_bytes()).unwrap();
        let mut project = xmltree::Element::parse(fs::read(&golden).unwrap().as_slice()).unwrap();
        assert!(append_owner(&mut project, control(&source, OWNER).unwrap()));
        project
            .write_with_config(
                fs::File::create(&input).unwrap(),
                xmltree::EmitterConfig::new().perform_indent(true),
            )
            .unwrap();
        let owner = OWNER;
        let before = fs::read_to_string(&input).unwrap();
        assert!(before.contains("<In "));
        let original = xmltree::Element::parse(before.as_bytes()).unwrap();
        let owner_before = control(&original, owner).unwrap().clone();
        for flag in ["--inverted", "--no-inverted"] {
            assert_cmd::cargo::cargo_bin_cmd!("lox")
                .env("LOX_TELEMETRY", "0")
                .args(["config", "set-input-inversion"])
                .arg(&input)
                .args([format!("uuid:{owner}").as_str(), key, flag, "-q"])
                .assert()
                .success();
        }
        let after = fs::read_to_string(&input).unwrap();
        let saved = xmltree::Element::parse(after.as_bytes()).unwrap();
        assert_eq!(control(&saved, owner).unwrap(), &owner_before);
        for uuid in [
            "1fbf3415-030f-bc7d-ffff9bb3c1479539",
            "1fbf341a-00e9-c0af-ffff9bb3c1479539",
            "1fbf34a5-03b1-83a1-ffff9bb3c1479539",
            owner,
        ] {
            assert_eq!(control(&saved, uuid).unwrap().attributes["WF"], "16384");
        }
        let wires = |text: &str| {
            text.lines()
                .filter(|line| line.contains("<In ") || line.contains("<Ref "))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        assert_eq!(wires(&after), wires(&before));
    }
}

#[test]
fn xor_not_inputs_refuse_wrong_native_ports_layouts_and_context_before_noop() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for (owner_type, key, wrong_key) in [("Xor", "I1", "I"), ("Not", "I", "I1")] {
        let source = logic_fixture(owner_type);
        let mut cases = vec![
            (source.clone(), wrong_key),
            (source.clone(), "Q"),
            (
                source
                    .replace("Nio=\"2\"", "Nio=\"3\"")
                    .replace("Nio=\"3\"", "Nio=\"4\""),
                key,
            ),
            (source.replace(&format!("K=\"{key}\""), "K=\"Input1\""), key),
            (source.replace("Def=\"0\"", "Def=\"1\""), key),
            (source.replace("Def=\"0\"", "Def=\"0\" Nc=\"1\""), key),
            (
                source.replace(
                    "Unknown=\"keep\"/>",
                    "Unknown=\"keep\"><Ref U=\"51111111-2222-3333-4444555566667777\"/></Co>",
                ),
                key,
            ),
            (source.replace("DateS=\"1\"", "DateS=\"+1\""), key),
            (source.replace("DateS=\"1\"", "DateS=\"١\""), key),
            (source.replace("WF=\"16384\"", "WF=\"16385\""), key),
            (source.replace("Type=\"Program\"", "Type=\"Folder\""), key),
            (
                source.replace("Title=\"Gate\"", "Title=\"Gate\" IfT=\"false\""),
                key,
            ),
            (
                source.replace("Title=\"Gate\"", "Title=\"Gate\" uuidParent=\"x\""),
                key,
            ),
            (
                source.replace("Title=\"Gate\"", "Title=\"Noon 12hTF=quoted\""),
                key,
            ),
            (
                source.replace(INPUT, "b1111111-2222-3333-4444-555566667777"),
                key,
            ),
            (source.replace("Def=\"0\"", "Def=\"0\" Def=\"0\""), key),
            (
                source.replace(
                    "</Loxone>",
                    &format!(
                        "<C Type=\"Other\" U=\"{}\"/></Loxone>",
                        OWNER.to_uppercase()
                    ),
                ),
                key,
            ),
        ];
        if owner_type == "Not" {
            cases.push((source.replace("Nio=\"2\"", "Nio=\"3\""), key));
            cases.push((source.replace("K=\"Q\"", "K=\"I\""), key));
        }
        for (xml, requested) in cases {
            for inverted in [false, true] {
                for destination in [None, Some(&existing), Some(&new)] {
                    fs::write(&input, &xml).unwrap();
                    fs::write(&existing, b"preserve").unwrap();
                    let mut edit = command(&input, requested, inverted);
                    if let Some(destination) = destination {
                        edit.arg("--save-as").arg(destination);
                    }
                    edit.assert().failure();
                    assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                    assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                    assert!(!new.exists());
                }
            }
        }
    }
}

#[test]
fn input_inversion_review_admits_cli_added_and_on_unchanged_golden_page() {
    assert_cli_added_logic_on_unchanged_golden_page("And");
}

#[test]
fn input_inversion_admits_cli_added_or_on_unchanged_golden_page() {
    assert_cli_added_logic_on_unchanged_golden_page("Or");
}

fn assert_cli_added_logic_on_unchanged_golden_page(block_type: &str) {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("golden-with-cli-logic.Loxone");
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs")
        .join("Guest Room Door.Loxone");
    fs::copy(&golden, &input).unwrap();
    let page = "1fbf34a5-03b1-83a1-ffff9bb3c1479539";
    let mut add = assert_cmd::cargo::cargo_bin_cmd!("lox");
    let output = add
        .env("LOX_TELEMETRY", "0")
        .args(["config", "add"])
        .arg(&input)
        .args([
            "--type",
            block_type,
            "--title",
            "Input inversion admission regression",
            "--parent",
            format!("uuid:{page}").as_str(),
            "-o",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let added: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let owner = added["uuid"].as_str().unwrap();
    let before = fs::read(&input).unwrap();
    let original = xmltree::Element::parse(before.as_slice()).unwrap();
    let ancestors = [
        "1fbf3415-030f-bc7d-ffff9bb3c1479539",
        "1fbf341a-00e9-c0af-ffff9bb3c1479539",
        page,
        owner,
    ];
    for uuid in ancestors {
        assert_eq!(control(&original, uuid).unwrap().attributes["WF"], "16384");
    }
    for (key, flag) in [
        ("I1", "--inverted"),
        ("I1", "--no-inverted"),
        ("I2", "--inverted"),
        ("I2", "--no-inverted"),
    ] {
        let mut edit = assert_cmd::cargo::cargo_bin_cmd!("lox");
        edit.env("LOX_TELEMETRY", "0")
            .args(["config", "set-input-inversion"])
            .arg(&input)
            .args([format!("uuid:{owner}").as_str(), key, flag, "-q"])
            .assert()
            .success();
        let after = fs::read(&input).unwrap();
        let saved = xmltree::Element::parse(after.as_slice()).unwrap();
        for uuid in ancestors {
            assert_eq!(control(&saved, uuid).unwrap().attributes["WF"], "16384");
        }
        let selected = control(&saved, owner)
            .unwrap()
            .children
            .iter()
            .filter_map(xmltree::XMLNode::as_element)
            .find(|co| co.attributes.get("K").map(String::as_str) == Some(key))
            .unwrap();
        assert_eq!(
            selected.attributes.get("Inv").map(String::as_str),
            (flag == "--inverted").then_some("true")
        );
    }
}

#[test]
fn input_inversion_review_preserves_unrelated_identical_duplicate_attributes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("duplicates.Loxone");
    let output = dir.path().join("copy.Loxone");
    let duplicate = "<C Type=\"Foo\" U=\"91111111-2222-3333-4444555566667777\" Title=\"x\" Title='x' Note=\"&amp;\" Note=\"&amp;\"/>";
    let source = fixture().replace("</Loxone>", &format!("{duplicate}</Loxone>"));
    fs::write(&input, &source).unwrap();
    fs::write(&output, b"existing output").unwrap();
    command(&input, "I1", false)
        .arg("--save-as")
        .arg(&output)
        .arg("-q")
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    command(&input, "I1", true).arg("-q").assert().success();
    let changed = fs::read(&input).unwrap();
    assert!(std::str::from_utf8(&changed).unwrap().contains(duplicate));
    command(&input, "I1", true).arg("-q").assert().success();
    assert_eq!(fs::read(&input).unwrap(), changed);
    command(&input, "I1", false).arg("-q").assert().success();
    assert!(fs::read_to_string(&input).unwrap().contains(duplicate));
}

#[test]
fn input_inversion_review_refuses_conflicting_or_guarded_duplicates_atomically() {
    let source = fixture();
    let cases = [
        source.replace("</Loxone>", "<C Type=\"Foo\" U=\"91111111-2222-3333-4444555566667777\" Title=\"x\" Title=\"y\"/></Loxone>"),
        source.replace("</Loxone>", "<C Type=\"Foo\" U=\"91111111-2222-3333-4444555566667777\" Title=\"x\" Title=\"&#120;\"/></Loxone>"),
        source.replace("</Loxone>", "<C Type=\"Foo\" Type=\"Foo\" U=\"91111111-2222-3333-4444555566667777\"/></Loxone>"),
        source.replace("</Loxone>", "<C Type=\"Foo\" U=\"91111111-2222-3333-4444555566667777\" U=\"91111111-2222-3333-4444555566667777\"/></Loxone>"),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" Title=\"Gate\""),
        source.replace("Def=\"0\"", "Def=\"0\" Def=\"0\""),
        source.replace("DateS=\"1\"", "DateS=\"1\" DateS=\"1\""),
        source.replace("U=\"51111111-2222-3333-4444555566667777\" Unknown=\"keep-wire\"", "U=\"51111111-2222-3333-4444555566667777\" U=\"51111111-2222-3333-4444555566667777\" Unknown=\"keep-wire\""),
    ];
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for xml in cases {
        for inverted in [false, true] {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &xml).unwrap();
                fs::write(&existing, b"preserve").unwrap();
                let mut edit = command(&input, "I1", inverted);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                edit.assert().failure().stderr(
                    predicates::str::contains("Input inversion")
                        .and(predicates::str::contains("nothing was changed")),
                );
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                assert!(!new.exists());
            }
        }
    }
}

#[test]
fn input_inversion_changes_only_selected_inv_and_document_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for (block_type, key) in [("And", "I1"), ("And", "I2"), ("Or", "I1"), ("Or", "I2")] {
        let source = if key == "I2" {
            fixture().replace("V=\"121\"", "V=\"90\"")
        } else {
            fixture()
        }
        .replace("Type=\"And\"", &format!("Type=\"{block_type}\""));
        fs::write(&input, &source).unwrap();
        let output = command(&input, key, true)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["input_inversion"]["inverted"], true);
        assert_eq!(result["input_inversion"]["changed"], true);
        let saved = fs::read_to_string(&input).unwrap();
        assert!(saved.starts_with('\u{feff}'));
        assert!(saved.ends_with("\r\n"));
        let co = if key == "I1" {
            format!("<Co K=\"I1\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\" Inv=\"true\"/>")
        } else {
            "<Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\" Inv=\"true\"/>".into()
        };
        assert!(saved.contains(&co));
        let page_start = source.find("<C Type=\"Page\"").unwrap();
        let expected = if key == "I1" {
            source[page_start..].replacen(
                &format!("<Co K=\"I1\" U=\"{INPUT}\" Def=\"0\" Unknown=\"keep\"/>"),
                &co,
                1,
            )
        } else {
            source[page_start..].replacen(
                "<Co K=\"I2\" U=\"41111111-2222-3333-4444555566667777\"/>",
                &co,
                1,
            )
        };
        assert_eq!(&saved[saved.find("<C Type=\"Page\"").unwrap()..], expected);
        assert!(saved.contains("<Password t=\"15\" v=\"unchanged\"/>"));
        let once = fs::read(&input).unwrap();
        command(&input, key, true).arg("-q").assert().success();
        assert_eq!(fs::read(&input).unwrap(), once);
        command(&input, key, false).arg("-q").assert().success();
        let cleared = fs::read_to_string(&input).unwrap();
        assert!(!cleared.contains("Inv="));
        assert!(cleared.contains("Def=\"0\" Unknown=\"keep\""));
    }
}

#[test]
fn input_inversion_false_noop_and_save_as_preserve_original_bytes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let destination = dir.path().join("copy.Loxone");
    let source = fixture().replace("Def=\"0\"", "Def=\"0\" Inv=\"false\"");
    fs::write(&input, &source).unwrap();
    command(&input, "I1", false)
        .arg("--save-as")
        .arg(&destination)
        .arg("-q")
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&destination).unwrap(), source.as_bytes());
}

#[test]
fn input_inversion_admission_precedes_noop_and_all_destination_writes() {
    let source = fixture();
    let mut cases = vec![
        source.replace("Type=\"And\"", "Type=\"UnsupportedLogic\""),
        source.replace("Nio=\"3\"", "Nio=\"4\""),
        source.replace("V=\"121\"", "V=\"89\""),
        source.replace("Def=\"0\"", "Def=\"1\""),
        source.replace("Def=\"0\"", "Def=\"0\" Inv=\"invalid\""),
        source.replace("Def=\"0\"", "Def=\"0\" Nc=\"1\""),
        source.replace(
            "Unknown=\"keep\"/>",
            "Unknown=\"keep\"><Ref U=\"51111111-2222-3333-4444555566667777\"/></Co>",
        ),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" WF=\"8192\""),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" WF=\"16385\""),
        source.replace("Type=\"Page\"", "Type=\"Page\" WF=\"32768\""),
        source.replace("Type=\"Document\"", "Type=\"Document\" WF=\"536887296\""),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" Dev=\"device\""),
        source.replace("Title=\"Gate\"", "Title=\" Wake 7am=ok\""),
        source.replace("K=\"I2\"", "K=\"I1\""),
        source.replace(INPUT, "00000000-0000-0000-0000000000000000"),
        source.replace(INPUT, "b1111111-2222-3333-4444-555566667777"),
        source.replace(
            "41111111-2222-3333-4444555566667777",
            "00000000-0000-0000-0000000000000000",
        ),
        source.replace(OWNER, "00000000-0000-0000-0000000000000000"),
        source.replace(
            "</Loxone>",
            "<C Type=\"Or\" U=\"a1111111-2222-3333-4444-555566667777\"/></Loxone>",
        ),
        source.replace(
            "</Loxone>",
            &format!("<C Type=\"Or\" U=\"{OWNER}\"/></Loxone>"),
        ),
        source.replace(
            "</Loxone>",
            &format!("<C Type=\"Or\" U=\"{}\"/></Loxone>", OWNER.to_uppercase()),
        ),
        source.replace(
            "</Loxone>",
            "<C Type=\"Or\" U=\"b1111111-2222-3333-4444-555566667777\"/></Loxone>",
        ),
        source.replace(
            "</Loxone>",
            &format!("<C Type=\"Or\" U=\"{}\"/></Loxone>", INPUT.to_uppercase()),
        ),
        source.replace("K=\"I1\"", "K=\"Input1\""),
        source.replace("Type=\"Page\"", "Type=\"Place\""),
        source.replace("<Loxone>", "<Loxone xmlns=\"urn:not-native\">"),
        source.replace("<Loxone>", "<Loxone xmlns:x=\"urn:not-native\">"),
        source.replace(OWNER, "&#x61;1111111-2222-3333-4444555566667777"),
        source.replace("DateS=\"1\"", "DateS=\"invalid\""),
        source.replace("DateS=\"1\"", "DateS=\"2147483647\""),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" IfT=\"false\""),
        source.replace("Title=\"Gate\"", "Title=\"Gate\" uuidParent=\"parent\""),
    ];
    let program = source
        .replace(
            "<C Type=\"Page\"",
            "<C Type=\"Program\" V=\"121\" U=\"81111111-2222-3333-4444555566667777\" WF=\"16384\"><C Type=\"Page\"",
        )
        .replace("</C></C><C Type=\"OutputRef\"", "</C></C></C><C Type=\"OutputRef\"");
    cases.extend([
        program.replace("WF=\"16384\"", "WF=\"16385\""),
        program.replace("Type=\"Program\"", "Type=\"Folder\""),
        program.replace("Type=\"Program\" V=\"121\"", "Type=\"Program\" V=\"89\""),
        program.replace("WF=\"16384\"", "WF=\"16384\" WF=\"16384\""),
    ]);
    for template in [
        "<Template/>",
        "<Template IfT=\"true\"/>",
        "<Template IfT=\"false\"/>",
    ] {
        cases.push(source.replace("<C Type=\"Page\"", &format!("{template}<C Type=\"Page\"")));
    }
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let or_cases: Vec<_> = cases
        .iter()
        .map(|xml| xml.replace("Type=\"And\"", "Type=\"Or\""))
        .collect();
    for xml in cases.into_iter().chain(or_cases) {
        for destination in [None, Some(&existing), Some(&new)] {
            fs::write(&input, &xml).unwrap();
            fs::write(&existing, b"preserve").unwrap();
            let mut invocation = command(&input, "I1", false);
            if let Some(destination) = destination {
                invocation.arg("--save-as").arg(destination);
            }
            invocation.assert().failure();
            assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"preserve");
            assert!(!new.exists());
        }
    }
}

#[test]
fn or_input_inversion_noop_and_save_as_report_only_actual_writes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("or.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = fixture()
        .replace("Type=\"And\"", "Type=\"Or\" WF=\"16384\"")
        .replace(" DateS=\"1\"", "")
        .replace("Def=\"0\"", "Def=\"0\" Inv='false'")
        .replace(
            "<Co K=\"I2\"",
            "<Co Def='1' Inv='true' Note='keep &amp; preserve' K=\"I2\"",
        )
        .replace("<Loxone>", "<?audit before?>\r\n<!--before--><Loxone>")
        + "<!--after--><?audit after?>\r\n";
    fs::write(&input, &source).unwrap();
    fs::write(&existing, b"preserve until success").unwrap();

    for destination in [None, Some(&existing)] {
        let mut edit = command(&input, "I1", false);
        if let Some(destination) = destination {
            edit.arg("--save-as").arg(destination);
        }
        let output = edit
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["input_inversion"]["changed"], false);
        assert_eq!(
            result["saved_to"],
            serde_json::json!(destination.map(|path| path.to_str().unwrap()))
        );
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
    assert_eq!(fs::read(&existing).unwrap(), source.as_bytes());

    command(&input, "I1", true)
        .arg("--save-as")
        .arg(&new)
        .arg("-q")
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    let saved = fs::read_to_string(&new).unwrap();
    assert!(saved.starts_with("\u{feff}<?xml"));
    assert!(saved.ends_with("<!--after--><?audit after?>\r\n"));
    let page_start = source.find("<C Type=\"Page\"").unwrap();
    assert_eq!(
        &saved[saved.find("<C Type=\"Page\"").unwrap()..],
        source[page_start..].replacen("Inv='false'", "Inv='true'", 1)
    );
    command(&input, "I1", true)
        .arg("--save-as")
        .arg(&existing)
        .arg("-q")
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert!(
        fs::read_to_string(&existing)
            .unwrap()
            .contains("Inv='true'")
    );
}

#[test]
fn input_inversion_refuses_hidden_roots_and_unbalanced_storage_before_any_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for block_type in ["And", "Or"] {
        let source = fixture().replace("Type=\"And\"", &format!("Type=\"{block_type}\""));
        let cases = [
            format!("{source}<C Type=\"Document\"/>"),
            format!("{source}<C Type=\"Or\" U=\"{OWNER}\"/>"),
            format!("{source}<Co U=\"{}\"/>", INPUT.to_uppercase()),
            format!("{source}<Co U=\"b1111111-2222-3333-4444-555566667777\"/>"),
            format!("{source}trailing"),
            format!("{source}&#32;"),
            format!("{source}<![CDATA[ ]]>"),
            format!("{source}<?xml version=\"1.0\"?>"),
            format!("{source}</Loxone>"),
            source.replace("</Loxone>", ""),
            source.replace("<Loxone>", "<!DOCTYPE Loxone><Loxone>"),
        ];
        for xml in cases {
            for inverted in [false, true] {
                for destination in [None, Some(&existing), Some(&new)] {
                    fs::write(&input, &xml).unwrap();
                    fs::write(&existing, b"preserve").unwrap();
                    let mut edit = command(&input, "I1", inverted);
                    if let Some(destination) = destination {
                        edit.arg("--save-as").arg(destination);
                    }
                    edit.assert().failure().stderr(
                        predicates::str::contains("Input inversion")
                            .and(predicates::str::contains("nothing was changed")),
                    );
                    assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                    assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                    assert!(!new.exists());
                }
            }
        }
    }
}

#[test]
fn input_inversion_pi_refuses_malformed_targets_before_noop_or_any_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for (block_type, leading, pi) in [
        ("Or", true, "<? ?>"),
        ("Or", false, "<? ?>"),
        ("And", true, "<?\t \r\n?>"),
        ("And", false, "<?\t \r\n?>"),
        ("Or", true, "<? audit?>"),
        ("Or", false, "<?1audit?>"),
        ("And", true, "<?bad?target?>"),
        ("And", false, "<?XmL data?>"),
    ] {
        let source = fixture().replace("Type=\"And\"", &format!("Type=\"{block_type}\""));
        let source = if leading {
            source.replace("<Loxone>", &format!("{pi}<Loxone>"))
        } else {
            format!("{source}{pi}")
        };
        for inverted in [false, true] {
            for destination in [None, Some(&existing), Some(&new)] {
                fs::write(&input, &source).unwrap();
                fs::write(&existing, b"PRESERVE").unwrap();
                let mut edit = command(&input, "I1", inverted);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                edit.assert().failure().stderr(
                    predicates::str::contains("Input inversion")
                        .and(predicates::str::contains("nothing was changed")),
                );
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"PRESERVE");
                assert!(!new.exists());
            }
        }
    }
}

#[test]
fn input_inversion_pi_preserves_valid_names_data_and_document_bytes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for block_type in ["And", "Or"] {
        let source = fixture()
            .replace("Type=\"And\"", &format!("Type=\"{block_type}\""))
            .replace(
                "<Loxone>",
                "<?audit?>\r\n<!--before--><?xml-stylesheet type='text/xsl'?><?réglage mode='été & <ok>'?><Loxone>",
            )
            .replace(
                "<C Type=\"Page\"",
                "<!--inside--><?測定 値=\"23\"?><?測定?><C Type=\"Page\"",
            )
            + "<!--after--><?audit done?><?fin?>\r\n";
        fs::write(&input, &source).unwrap();
        fs::write(&existing, b"PRESERVE UNTIL SUCCESS").unwrap();
        command(&input, "I1", false).arg("-q").assert().success();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        command(&input, "I1", false)
            .arg("--save-as")
            .arg(&existing)
            .arg("-q")
            .assert()
            .success();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&existing).unwrap(), source.as_bytes());
        command(&input, "I1", true)
            .arg("--save-as")
            .arg(&new)
            .arg("-q")
            .assert()
            .success();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let saved = fs::read_to_string(&new).unwrap();
        let parsed = xmltree::Element::parse(saved.as_bytes()).unwrap();
        let document = control(&parsed, "01111111-2222-3333-4444555566667777").unwrap();
        let expected = source
            .replacen(
                "DateS=\"1\"",
                &format!(
                    "DateS=\"{}\" Date=\"{}\"",
                    document.attributes["DateS"], document.attributes["Date"]
                ),
                1,
            )
            .replacen(
                "Def=\"0\" Unknown=\"keep\"/>",
                "Def=\"0\" Unknown=\"keep\" Inv=\"true\"/>",
                1,
            );
        assert_eq!(saved, expected);
    }
}

#[test]
fn input_inversion_refuses_output_parameter_alias_and_missing_explicit_state() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    fs::write(&input, fixture()).unwrap();
    for key in ["Q", "Remanence", "Input1", "i1"] {
        command(&input, key, true).assert().failure();
    }
    let mut invocation = assert_cmd::cargo::cargo_bin_cmd!("lox");
    invocation
        .args(["config", "set-input-inversion"])
        .arg(&input)
        .args([format!("uuid:{OWNER}").as_str(), "I1"]);
    invocation.assert().failure();
}
