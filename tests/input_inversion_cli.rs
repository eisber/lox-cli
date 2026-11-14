use std::{fs, path::Path};

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

#[test]
fn input_inversion_changes_only_selected_inv_and_document_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for key in ["I1", "I2"] {
        let source = if key == "I2" {
            fixture().replace("V=\"121\"", "V=\"90\"")
        } else {
            fixture()
        };
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
        source.replace("Type=\"And\"", "Type=\"Or\""),
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
    for xml in cases {
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
