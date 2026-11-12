use assert_cmd::Command;
use std::fs;

const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
	<C Type="LightController2" U="lighting" Title="Lamp">
		<LightscenesC Num="2" Outputs="18" Future="keep">
			<LightsceneC Name="A" UUID="a" SID="+0002" CID="0" MixMod="false" Q1="future" SName="alias">
				<SeqConf Future="keep"/>
			</LightsceneC>
			<LightsceneC Name="B" UUID="b" SID="3" CID="20" MixMod="true" Q2="-1"/>
		</LightscenesC>
		<LSConfig OpM="opaque" ScMv="future"/>
		<HCL T="2" FROM="0"/>
	</C>
</ControlList>
"#;

fn bytes(xml: &str) -> Vec<u8> {
    let mut result = vec![0xef, 0xbb, 0xbf];
    result.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    result
}

fn command() -> Command {
    let mut command = Command::cargo_bin("lox").unwrap();
    command.env("LOX_TELEMETRY", "0");
    command
}

#[test]
fn lighting_metadata_cli_boolean_case_retains_noop_bytes_and_does_not_coerce_bad_flags() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML.replacen("MixMod=\"false\"", "MixMod=\"FaLsE\" SkipNext=\"TRUE\"", 1);
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    for (operation, flag) in [("set-mood-mix", "--no-mix"), ("set-mood-skip", "--skip")] {
        let result = command()
            .args(["config", "lighting", operation])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", flag, "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&result).unwrap()["changed"],
            false
        );
        assert_eq!(fs::read(&input).unwrap(), before);
    }
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--mix", "--save-as"])
        .arg(&output)
        .args(["--quiet"])
        .assert()
        .success();
    let expected = bytes(&xml.replacen("MixMod=\"FaLsE\"", "MixMod=\"true\"", 1));
    assert_eq!(fs::read(&output).unwrap(), expected);
    let invalid = bytes(&xml.replacen("MixMod=\"FaLsE\"", "MixMod=\"0\"", 1));
    fs::write(&input, &invalid).unwrap();
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--no-mix", "--save-as"])
        .arg(&output)
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), invalid);
    assert_eq!(fs::read(&output).unwrap(), expected);
}

#[test]
fn lighting_metadata_cli_options_are_readonly_and_expose_projection_limits_in_all_formats() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    for format in ["json", "table", "csv"] {
        let output = command()
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["config", "lighting", "inspect-mood-options"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--mood",
                "A",
                "-o",
                format,
                "--quiet",
                "--no-header",
            ])
            .assert()
            .success()
            .get_output()
            .clone();
        if format == "json" {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            let skip = value["flags"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["key"] == "SkipNext")
                .unwrap();
            assert_eq!(skip["value"]["stored"], serde_json::Value::Null);
            assert_eq!(skip["value"]["effective"], false);
            assert_eq!(skip["checkbox_checked"], true);
            assert_eq!(value["display_name"]["stored"], "alias");
            assert_eq!(value["read_only"], true);
            assert_eq!(value["icon_id"]["stored"], serde_json::Value::Null);
            assert_eq!(value["icon_id"]["effective"], 0);
            assert_eq!(value["icon_resolution"], "unresolved_catalog_mapping");
        } else {
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.contains("SkipNext")
                    && stdout.contains("inverse")
                    && stdout.contains("alias")
            );
            assert!(!stdout.contains("OPTION,") && !stdout.contains("OPTION  "));
            if format == "csv" {
                assert!(stdout.lines().all(|line| line.split(',').count() == 7));
                assert!(
                    stdout.contains(
                        "\"SkipNext\",false,\"\",\"false\",\"false\",\"true\",\"inverse\""
                    )
                );
                assert!(stdout.contains("\"SName\",true,\"alias\""));
                assert!(
                    stdout
                        .contains("\"IID\",false,\"\",\"0\",\"0\",,\"icon_reference_unresolved\"")
                );
            }
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("not loaded-row/import")
            );
        }
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}

#[test]
fn lighting_metadata_cli_display_name_ascii_optin_reports_conversion_and_preserves_noop() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    fs::write(&input, bytes(XML)).unwrap();
    let result = command()
        .args(["config", "lighting", "set-mood-display-name"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--name",
            "Late Evening",
            "--normalize-ascii",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["normalized_from"], "Late Evening");
    assert_eq!(result["after"], "LATE EVENING");
    let after = bytes(&XML.replace("SName=\"alias\"", "SName=\"LATE EVENING\""));
    assert_eq!(fs::read(&input).unwrap(), after);
    command()
        .args(["config", "lighting", "set-mood-display-name"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--name",
            "late evening",
            "--normalize-ascii",
            "--quiet",
        ])
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), after);
    command()
        .args(["config", "lighting", "set-mood-display-name"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--clear",
            "--normalize-ascii",
        ])
        .assert()
        .code(2);
    assert_eq!(fs::read(&input).unwrap(), after);
}

#[test]
fn lighting_metadata_cli_display_name_is_separate_and_clear_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    command()
        .args(["config", "lighting", "set-mood-display-name"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--name",
            "LATE EVENING",
            "--quiet",
        ])
        .assert()
        .success()
        .stdout("");
    assert_eq!(
        fs::read(&input).unwrap(),
        bytes(&XML.replace("SName=\"alias\"", "SName=\"LATE EVENING\""))
    );
    let output = command()
        .args(["config", "lighting", "set-mood-display-name"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--clear", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(output["before"], "LATE EVENING");
    assert_eq!(output["after"], "");
    let cleared = bytes(&XML.replace("SName=\"alias\"", "SName=\"\""));
    assert_eq!(fs::read(&input).unwrap(), cleared);
    command()
        .args(["config", "lighting", "set-mood-display-name"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "B", "--clear", "--quiet"])
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), cleared);
}

#[test]
fn lighting_metadata_cli_display_name_rejects_transformations_and_bad_flag_combinations() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (flags, code) in [
        (vec![], 2),
        (vec!["--name", ""], 2),
        (vec!["--name", "ABC", "--clear"], 2),
        (vec!["--name", "Mixed Case"], 1),
        (vec!["--name", "BAD_NAME"], 1),
        (vec!["--name", " ABC"], 1),
        (vec!["--name", "Ä"], 1),
    ] {
        let before = bytes(XML);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        command()
            .args(["config", "lighting", "set-mood-display-name"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A"])
            .args(flags)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .code(code);
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
    }
}

#[test]
fn lighting_metadata_cli_skip_preserves_absence_then_changes_only_selected_flag() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    let result = command()
        .args(["config", "lighting", "set-mood-skip"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--no-skip", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["before"]["stored"], serde_json::Value::Null);
    assert_eq!(result["before"]["effective"], false);
    assert_eq!(result["after"], serde_json::Value::Null);
    assert_eq!(result["checkbox_checked"], true);
    assert_eq!(result["changed"], false);
    assert_eq!(fs::read(&input).unwrap(), before);
    let result = command()
        .args(["config", "lighting", "set-mood-skip"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--skip", "--save-as"])
        .arg(&output)
        .args(["-o", "json", "--quiet"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["skip"], true);
    assert_eq!(result["checkbox_checked"], false);
    assert_eq!(fs::read(&input).unwrap(), before);
    let expected = XML.replace("SName=\"alias\"", "SName=\"alias\" SkipNext=\"true\"");
    assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
}

#[test]
fn lighting_metadata_cli_skip_requires_exact_state_and_never_defaults_explicit_bad_values() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (xml, flags, code) in [
        (XML.to_string(), vec![], 2),
        (XML.to_string(), vec!["--skip", "--no-skip"], 2),
        (
            XML.replace("SName=\"alias\"", "SName=\"alias\" SkipNext=\"future\""),
            vec!["--skip"],
            1,
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        command()
            .args(["config", "lighting", "set-mood-skip"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A"])
            .args(flags)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .code(code);
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
    }
}

#[test]
fn lighting_metadata_cli_rename_preserves_other_bytes_and_guards_ambiguous_names() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    let result = command()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "rename-mood"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--name", "B", "--save-as"])
        .arg(&output)
        .args(["-o", "json", "--quiet"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["mood"]["sid"], "+0002");
    assert_eq!(result["before"], "A");
    assert_eq!(result["after"], "B");
    let after = bytes(&XML.replace("Name=\"A\"", "Name=\"B\""));
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(fs::read(&output).unwrap(), after);
    command()
        .args(["config", "lighting", "rename-mood"])
        .arg(&output)
        .args([
            "uuid:lighting",
            "--mood",
            "B",
            "--name",
            "New Name",
            "-o",
            "json",
        ])
        .assert()
        .failure();
    assert_eq!(fs::read(&output).unwrap(), after);
}

#[test]
fn lighting_metadata_cli_mix_enable_disable_and_noops_preserve_unselected_storage() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--no-mix", "-o", "csv", "--no-header"])
        .assert()
        .success()
        .stdout("\"a\",\"+0002\",\"0\",\"false\",\"false\",false,false,selected_record_role_independent\n");
    assert_eq!(fs::read(&input).unwrap(), before);
    let result = command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--mix",
            "-o",
            "json",
            "--quiet",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["enabled"], true);
    assert_eq!(result["scope"], "selected_record_role_independent");
    assert_eq!(
        fs::read(&input).unwrap(),
        bytes(&XML.replace("CID=\"0\" MixMod=\"false\"", "CID=\"0\" MixMod=\"true\""))
    );
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--no-mix", "--quiet"])
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), before);
    command()
        .args(["config", "lighting", "rename-mood"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--name", "A", "--quiet"])
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), before);
}

#[test]
fn lighting_metadata_cli_invalid_name_role_and_state_flags_never_write_destinations() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (xml, operation, flags, expected_code) in [
        (
            XML.to_string(),
            "rename-mood",
            vec!["--name", "Bad_Name"],
            1,
        ),
        (
            XML.replace("CID=\"0\"", "CID=\"12\""),
            "set-mood-mix",
            vec!["--mix"],
            1,
        ),
        (
            XML.replace("SID=\"+0002\"", "SID=\"32\""),
            "set-mood-mix",
            vec!["--mix"],
            1,
        ),
        (XML.to_string(), "set-mood-mix", vec![], 2),
        (
            XML.to_string(),
            "set-mood-mix",
            vec!["--mix", "--no-mix"],
            2,
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        for save_as in [false, true] {
            let mut cmd = command();
            cmd.args(["config", "lighting", operation])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A"])
                .args(&flags)
                .args(["-o", "json"]);
            if save_as {
                cmd.arg("--save-as").arg(&output);
            }

            cmd.assert().code(expected_code);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
        }
    }
}

#[test]
fn lighting_metadata_cli_mix_omitted_cid_preserves_storage_and_rejects_bad_explicit_identity() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML.replacen(" CID=\"0\"", "", 1);
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    let result = command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--no-mix", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["mood"]["cid"], serde_json::Value::Null);
    assert_eq!(result["cid_effective"], 0);
    assert_eq!(result["changed"], false);
    assert_eq!(fs::read(&input).unwrap(), before);
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--mix", "--save-as"])
        .arg(&output)
        .args(["--quiet"])
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(
        fs::read(&output).unwrap(),
        bytes(&xml.replacen("MixMod=\"false\"", "MixMod=\"true\"", 1))
    );
    let bad = bytes(&xml.replacen("Name=\"A\"", "Name=\"A\" CID=\"future\"", 1));
    fs::write(&input, &bad).unwrap();
    let saved = fs::read(&output).unwrap();
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--mix", "--save-as"])
        .arg(&output)
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), bad);
    assert_eq!(fs::read(&output).unwrap(), saved);
}

#[test]
fn lighting_metadata_cli_mix_missing_default_noop_and_enable_preserve_raw_omission() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML.replace(" MixMod=\"false\"", "");
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    let result = command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--no-mix", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["before"], serde_json::Value::Null);
    assert_eq!(result["before_effective"], false);
    assert_eq!(result["default"], false);
    assert_eq!(result["after"], serde_json::Value::Null);
    assert_eq!(result["changed"], false);
    assert_eq!(fs::read(&input).unwrap(), before);
    command()
        .args(["config", "lighting", "set-mood-mix"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--mix", "--save-as"])
        .arg(&output)
        .args(["--quiet"])
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(
        fs::read(&output).unwrap(),
        bytes(&xml.replace("SName=\"alias\">", "SName=\"alias\" MixMod=\"true\">"))
    );
}
