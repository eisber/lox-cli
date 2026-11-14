use std::{fs, path::Path};

use serde_json::Value;

const BLOCK: &str = "0f8e1234-1234-5678-90abcdef01234567";

fn fixture() -> String {
    let zones: String = (0..8)
        .map(|index| {
            let name = if index == 3 {
                "Wake 7am=ok &amp; garden"
            } else {
                "North &amp; South"
            };
            format!("<zone ioIdx=\"{index}\" nm=\"{name}\" Unknown=\"keep-{index}\"/>")
        })
        .collect();
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"Irrigation\" U=\"{BLOCK}\" Title=\"Garden\" Unknown=\" Wake 7am=ok\"><Co K=\"Tv4\" U=\"timer\" Def=\"300\"/><Co K=\"Qv4\" U=\"valve\"><In Input=\"physical-output\"/></Co>{zones}<Unknown 7am=\"metadata\" Value=\" Wake 7am=ok\"/></C><C Type=\"OutputRef\" U=\"ref\" Title=\"Old output name\"><Co K=\"AI\" U=\"valve\"/></C><SET><Password t=\"15\" v=\"unchanged\"/></SET></C></Loxone>\r\n"
    )
}

fn command(input: &Path, index: &str, name: &str) -> assert_cmd::Command {
    selected_command(input, &format!("uuid:{BLOCK}"), index, name)
}

fn clear_command(input: &Path, index: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "irrigation-zone-set-name"])
        .arg(input)
        .args([&format!("uuid:{BLOCK}"), "--io-index", index, "--clear"]);
    command
}

fn selected_command(input: &Path, selector: &str, index: &str, name: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "irrigation-zone-set-name"])
        .arg(input)
        .args([selector, "--io-index", index, "--name", name]);
    command
}

fn assert_admission_refusal(xml: &str, selector: &str) {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for name in ["New", "Wake 7am=ok & garden"] {
        for destination in [None, Some(&existing), Some(&new)] {
            fs::write(&input, xml).unwrap();
            fs::write(&existing, b"existing destination").unwrap();
            let mut command = selected_command(&input, selector, "3", name);
            if let Some(destination) = destination {
                command.arg("--save-as").arg(destination);
            }
            command.assert().failure();
            assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"existing destination");
            assert!(!new.exists());
        }
    }
}

#[test]
fn irrigation_zone_name_admission_refuses_every_direct_document_template() {
    for template in [
        "<Template IfT=\"true\"/>",
        "<Template/>",
        "<Template IfT=\"false\"/>",
    ] {
        let xml = fixture().replacen(
            "<C Type=\"Irrigation\"",
            &format!("{template}<C Type=\"Irrigation\""),
            1,
        );
        assert_admission_refusal(&xml, &format!("uuid:{BLOCK}"));
    }
}

#[test]
fn irrigation_zone_name_admission_requires_one_actual_stamp_document_owning_the_block() {
    let source = fixture();
    for xml in [
        source.replace("Type=\"Document\"", "Type=\"Project\""),
        source
            .replace("<Loxone>", "<Loxone><Envelope>")
            .replace("</Loxone>", "</Envelope></Loxone>"),
        source.replace("</Loxone>", "<C Type=\"Document\"/></Loxone>"),
        source
            .replace("<C Type=\"Irrigation\"", "</C><C Type=\"Irrigation\"")
            .replace("</SET></C></Loxone>", "</SET></Loxone>"),
        source.replace(
            "Type=\"Document\"",
            "Type=\"Document\" xmlns=\"urn:not-native\"",
        ),
    ] {
        assert_admission_refusal(&xml, &format!("uuid:{BLOCK}"));
    }
    let root_document = source.replace("<Loxone>", "").replace("</Loxone>", "");
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("root-document.Loxone");
    fs::write(&input, &root_document).unwrap();
    command(&input, "3", "Root garden")
        .arg("-q")
        .assert()
        .success();
    let saved = fs::read_to_string(&input).unwrap();
    assert!(saved.contains("nm=\"Root garden\""));
    assert!(!saved.contains("DateS=\"1\""));
}

#[test]
fn irrigation_zone_name_admission_refuses_duplicate_effective_owner_uuid() {
    for duplicate in [
        BLOCK.to_string(),
        BLOCK.to_uppercase(),
        "0f8e1234-1234-5678-90ab-cdef01234567".into(),
    ] {
        let xml = fixture().replace(
            "</Loxone>",
            &format!("<C Type=\"And\" U=\"{duplicate}\"/></Loxone>"),
        );
        assert_admission_refusal(&xml, &format!("uuid:{BLOCK}"));
    }
}

#[test]
fn irrigation_zone_name_admission_refuses_rfc36_stored_owner_uuid() {
    let rfc = "0f8e1234-1234-5678-90ab-cdef01234567";
    let xml = fixture().replace(BLOCK, rfc);
    assert_admission_refusal(&xml, &format!("uuid:{rfc}"));
    assert_admission_refusal(&xml, &format!("uuid:{BLOCK}"));
}

#[test]
fn irrigation_zone_name_uses_original_text_and_changes_only_selected_name_and_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture();
    fs::write(&input, &source).unwrap();
    let name = " North & \"Süd\" <🌿> ";
    let stdout = command(&input, "3", name)
        .arg("--save-as")
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(result["name_change"]["block_uuid"], BLOCK);
    assert_eq!(result["name_change"]["io_index"], 3);
    assert_eq!(
        result["name_change"]["previous_name"],
        "Wake 7am=ok & garden"
    );
    assert_eq!(result["name_change"]["name"], name);
    assert_eq!(result["name_change"]["changed"], true);
    let saved = fs::read(&output).unwrap();
    let text = std::str::from_utf8(&saved).unwrap();
    let stamp = |key: &str| {
        text.split_once(&format!(" {key}=\""))
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
    };
    assert_ne!(stamp("DateS"), "1");
    let expected = source
        .replace(
            "nm=\"Wake 7am=ok &amp; garden\"",
            &format!("nm=\"{}\"", quick_xml::escape::escape(name)),
        )
        .replace(
            "Date=\"2020-01-01 00:00:00\"",
            &format!("Date=\"{}\"", stamp("Date")),
        )
        .replace("DateS=\"1\"", &format!("DateS=\"{}\"", stamp("DateS")));
    assert_eq!(saved, expected.as_bytes());
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
}

#[test]
fn irrigation_zone_name_noop_save_as_and_duplicate_names_preserve_raw_storage() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    let source = fixture().replace("\r\n", "\n").replace(" ioIdx=\"0\"", "");
    fs::write(&input, &source).unwrap();
    command(&input, "3", "Wake 7am=ok & garden")
        .arg("--save-as")
        .arg(&output)
        .args(["-o", "csv", "--no-header"])
        .assert()
        .success()
        .stdout(format!(
            "\"{BLOCK}\",3,\"Wake 7am=ok & garden\",\"Wake 7am=ok & garden\",false\n"
        ));
    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    command(&input, "3", "Wake 7am=ok & garden")
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    command(&input, "0", "North & South")
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    command(&input, "0", "West")
        .arg("--save-as")
        .arg(&output)
        .arg("-q")
        .assert()
        .success();
    let zero_renamed = fs::read_to_string(&output).unwrap();
    assert!(zero_renamed.contains("<zone nm=\"West\""));
    assert!(!zero_renamed.contains("ioIdx=\"0\""));
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    command(&input, "3", "North & South")
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    let after = fs::read(&input).unwrap();
    command(&input, "3", "North & South")
        .arg("-q")
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), after);
}

#[test]
fn irrigation_zone_name_rejects_unqualified_identity_autofill_and_text_without_writes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("never.Loxone");
    let source = fixture();
    for (xml, index, name) in [
        (source.replace("ioIdx=\"7\"", "ioIdx=\"6\""), "3", "New"),
        (source.replace("ioIdx=\"3\"", "ioIdx=\"03\""), "3", "New"),
        (
            source.replace("nm=\"North &amp; South\"", "nm=\"\""),
            "3",
            "New",
        ),
        (
            source.replace("nm=\"Wake 7am=ok &amp; garden\"", "nm=\"&#65;\""),
            "3",
            "New",
        ),
        (
            source.replace("nm=\"Wake 7am=ok &amp; garden\"", "nm=\"line\nbreak\""),
            "3",
            "New",
        ),
        (
            source.replace("Type=\"Irrigation\"", "Type=\"And\""),
            "3",
            "New",
        ),
        (source.clone(), "8", "New"),
        (source.clone(), "3", " "),
        (source.clone(), "3", "line\nbreak"),
    ] {
        fs::write(&input, &xml).unwrap();
        command(&input, index, name)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
        assert!(!output.exists());
    }

    let mut by_name = assert_cmd::cargo::cargo_bin_cmd!("lox");
    by_name
        .env("LOX_TELEMETRY", "0")
        .args(["config", "irrigation-zone-set-name"])
        .arg(&input)
        .args(["Garden", "--io-index", "3", "--name", "New"])
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    command(&input, "3", "New")
        .arg("--dry-run")
        .arg("--save-as")
        .arg(&output)
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert!(!output.exists());
}

#[test]
fn irrigation_zone_clear_omits_only_selected_name_and_can_restore_it() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("cleared.Loxone");
    let source = fixture();
    fs::write(&input, &source).unwrap();
    let stdout = clear_command(&input, "3")
        .arg("--save-as")
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(
        report["name_change"]["previous_name"],
        "Wake 7am=ok & garden"
    );
    assert_eq!(report["name_change"]["name"], "");
    assert_eq!(report["name_change"]["changed"], true);
    let saved = fs::read_to_string(&output).unwrap();
    let date = |text: &str, key: &str| {
        text.split_once(&format!(" {key}=\""))
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
            .to_string()
    };
    let expected = source
        .replace(" nm=\"Wake 7am=ok &amp; garden\"", "")
        .replace(
            "Date=\"2020-01-01 00:00:00\"",
            &format!("Date=\"{}\"", date(&saved, "Date")),
        )
        .replace(
            "DateS=\"1\"",
            &format!("DateS=\"{}\"", date(&saved, "DateS")),
        );
    assert_eq!(saved, expected);
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    let restored_stdout = command(&output, "3", "Wake 7am=ok & garden")
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let restored_report: Value = serde_json::from_slice(&restored_stdout).unwrap();
    assert_eq!(restored_report["name_change"]["previous_name"], "");
    let restored = fs::read_to_string(&output).unwrap();
    assert!(restored.contains("nm=\"Wake 7am=ok &amp; garden\""));
    assert_eq!(
        restored.replace(
            "<zone ioIdx=\"3\" Unknown=\"keep-3\" nm=\"Wake 7am=ok &amp; garden\"/>",
            "<zone ioIdx=\"3\" nm=\"Wake 7am=ok &amp; garden\" Unknown=\"keep-3\"/>",
        ),
        source
            .replace(
                "Date=\"2020-01-01 00:00:00\"",
                &format!("Date=\"{}\"", date(&restored, "Date")),
            )
            .replace(
                "DateS=\"1\"",
                &format!("DateS=\"{}\"", date(&restored, "DateS")),
            )
    );
}

#[test]
fn irrigation_zone_clear_noop_and_restore_admit_selected_missing_or_empty_name() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    for replacement in ["", " nm=\"\"", " nm = ''"] {
        let source = fixture().replace(" nm=\"Wake 7am=ok &amp; garden\"", replacement);
        fs::write(&input, &source).unwrap();
        clear_command(&input, "3")
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "csv", "--no-header"])
            .assert()
            .success()
            .stdout(format!("\"{BLOCK}\",3,\"\",\"\",false\n"));
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
        command(&input, "3", "Restored 7am=ok")
            .arg("-q")
            .assert()
            .success()
            .stdout("");
        assert!(
            fs::read_to_string(&input)
                .unwrap()
                .contains("Restored 7am=ok")
        );
    }
}

#[test]
fn irrigation_zone_clear_removes_whitespace_name_without_changing_other_fields() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = fixture().replace("nm=\"Wake 7am=ok &amp; garden\"", "nm = '   '");
    fs::write(&input, &source).unwrap();
    clear_command(&input, "3")
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    let saved = fs::read_to_string(&input).unwrap();
    assert!(saved.contains("<zone ioIdx=\"3\" Unknown=\"keep-3\"/>"));
    assert!(saved.contains("<Co K=\"Tv4\" U=\"timer\" Def=\"300\"/>"));
    assert!(saved.contains("<Co K=\"Qv4\" U=\"valve\"><In Input=\"physical-output\"/></Co>"));
    assert!(saved.contains("<C Type=\"OutputRef\" U=\"ref\" Title=\"Old output name\">"));
}

#[test]
fn irrigation_zone_clear_retains_atomic_admission_and_nonblank_name_contract() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = fixture();
    for xml in [
        source.replace("ioIdx=\"7\"", "ioIdx=\"6\""),
        source.replace("nm=\"North &amp; South\"", "nm=\"\""),
        source.replace("nm=\"Wake 7am=ok &amp; garden\"", "nm=\"&#65;\""),
        source.replace("Type=\"Irrigation\"", "Type=\"And\""),
        source.replace(
            "<C Type=\"Irrigation\"",
            "<Template IfT=\"true\"/><C Type=\"Irrigation\"",
        ),
        source.replace(
            "</Loxone>",
            &format!("<C Type=\"And\" U=\"{}\"/></Loxone>", BLOCK.to_uppercase()),
        ),
        source.replace(BLOCK, "00000000-0000-0000-0000000000000000"),
        source.replace("Type=\"Document\"", "Type=\"Project\""),
    ] {
        for destination in [None, Some(&existing), Some(&new)] {
            fs::write(&input, &xml).unwrap();
            fs::write(&existing, b"sentinel").unwrap();
            let mut command = clear_command(&input, "3");
            if let Some(destination) = destination {
                command.arg("--save-as").arg(destination);
            }
            command.assert().failure();
            assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"sentinel");
            assert!(!new.exists());
        }
    }
    fs::write(&input, &source).unwrap();
    for name in ["", " ", "\t"] {
        command(&input, "3", name).assert().failure();
        clear_command(&input, "3")
            .args(["--name", name])
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
    clear_command(&input, "3")
        .args(["--name", "New"])
        .assert()
        .failure();
    let mut missing_action = assert_cmd::cargo::cargo_bin_cmd!("lox");
    missing_action
        .env("LOX_TELEMETRY", "0")
        .args(["config", "irrigation-zone-set-name"])
        .arg(&input)
        .args([&format!("uuid:{BLOCK}"), "--io-index", "3"])
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
}
