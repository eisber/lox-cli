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
        (
            XML.replace("MixMod=\"false\"", ""),
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
