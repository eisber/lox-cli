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
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "irrigation-zone-set-name"])
        .arg(input)
        .args([
            &format!("uuid:{BLOCK}"),
            "--io-index",
            index,
            "--name",
            name,
        ]);
    command
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
