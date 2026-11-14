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
        source.replace("Type=\"And\"", "Type=\"Xor\""),
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
