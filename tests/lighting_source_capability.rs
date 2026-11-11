use assert_cmd::Command;
use std::fs;

const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="LightController2" U="lighting" Title="Lamp">
    <Co K="AQ2" U="output-2"><In Input="source" FLG="2" Future="retain"/></Co>
    <LightscenesC Num="99" Outputs="0003" Future="opaque">
      <LightsceneC UUID="first" Name="First" SID="2" CID="9" Outputs="2" Q1="00037" Q10="future">
        <SeqConf Future="retain"><Unknown Q2="nested"/></SeqConf>
      </LightsceneC>
      <Unknown Num="81"/>
      <LightsceneC UUID="second" Name="Second" SID="3" CID="10" Q2="-1"/>
    </LightscenesC>
  </C>
</ControlList>
"#;

fn lox() -> Command {
    let mut command = Command::cargo_bin("lox").unwrap();
    command.env("LOX_TELEMETRY", "0");
    command
}

fn fixture_bytes(xml: &str) -> Vec<u8> {
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    bytes
}

const LEVEL_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
	<C Type="LightController2" U="lighting" Title="Lamp" CapAI0="7">
		<LightscenesC Num="1" Outputs="18" Future="keep">
			<LightsceneC Name="Evening" SID="1" CID="12" UUID="mood" Outputs="18" Q1="1640612766" Q2="1610622736" Q10="1610612736">
				<SeqConf Future="retain"/>
			</LightsceneC>
		</LightscenesC>
		<HCL T="2" TMIN="2000" FROM="0" TO="0"/>
	</C>
</ControlList>
"#;

#[test]
fn lighting_editor_inspection_cli_is_read_only_and_exposes_operation_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = fixture_bytes(LEVEL_XML);
    fs::write(&input, &before).unwrap();
    let result = lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "inspect"])
        .arg(&input)
        .args(["uuid:lighting", "-o", "json", "--quiet"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    let outputs = result["moods"][0]["outputs"].as_array().unwrap();
    let q1 = outputs.iter().find(|q| q["key"] == "Q1").unwrap();
    let q2 = outputs.iter().find(|q| q["key"] == "Q2").unwrap();
    assert_eq!(q1["editor"]["kind"], "color");
    assert_eq!(q2["editor"]["kind"], "numeric");
    assert_eq!(
        q2["editor"]["capability"]["stored"],
        serde_json::Value::Null
    );
    assert_eq!(q2["editor"]["capability"]["effective"], 0);
    assert!(result["diagnostics"].as_array().unwrap().iter().any(|d| {
        d["code"] == "lighting_edit_operation_boundary"
            && d["message"]
                .as_str()
                .unwrap()
                .contains("not a mode-aware equivalent")
    }));
    assert_eq!(fs::read(&input).unwrap(), before);
}

#[test]
fn lighting_editor_inspection_cli_table_csv_and_quiet_keep_kinds_and_raw_edit_limit() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let xml = LEVEL_XML.replace("</C>", "<Co K=\"AQ2\" U=\"output-2\"/></C>");
    let before = fixture_bytes(&xml);
    fs::write(&input, &before).unwrap();
    for format in ["table", "csv"] {
        let result = lox()
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["config", "lighting", "inspect"])
            .arg(&input)
            .args(["uuid:lighting", "-o", format, "--no-header", "--quiet"])
            .assert()
            .success()
            .get_output()
            .clone();
        let stdout = String::from_utf8(result.stdout).unwrap();
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(stdout.contains("numeric") && stdout.contains("color"));
        assert!(!stdout.contains("EDITOR"));
        assert!(
            stderr.contains("raw packed-value edit")
                && stderr.contains("not a mode-aware equivalent")
        );
        if format == "csv" {
            assert!(stdout.lines().all(|line| line.split(',').count() == 10));
            assert!(stdout.contains(",Q2,\"1610622736\",numeric,6,\"\",0,"));
        }
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}

#[test]
fn lighting_numeric_level_cli_matches_native_selected_value_and_preserves_every_other_byte() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = fixture_bytes(LEVEL_XML);
    fs::write(&input, &before).unwrap();
    let result = lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "set-mood-level"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "Evening",
            "--output-index",
            "2",
            "--level",
            "37",
            "--save-as",
        ])
        .arg(&output)
        .args(["-o", "json", "--quiet"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["before"], "1610622736");
    assert_eq!(result["after"], "1610612773");
    assert_eq!(result["type_tag"], 6);
    assert_eq!(result["capability"]["stored"], serde_json::Value::Null);
    assert_eq!(result["capability"]["effective"], 0);
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(
        fs::read(&output).unwrap(),
        fixture_bytes(&LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"1610612773\""))
    );
}

#[test]
fn lighting_numeric_level_cli_noop_preserves_spelling_and_has_csv_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = fixture_bytes(&LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"+01610612773\""));
    fs::write(&input, &before).unwrap();
    lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "set-mood-level"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "1",
            "--output-index",
            "2",
            "--level",
            "37",
            "-o",
            "csv",
            "--no-header",
        ])
        .assert()
        .success()
        .stdout("\"1\",Q2,\"+01610612773\",\"+01610612773\",37,6,false\n");
    assert_eq!(fs::read(&input).unwrap(), before);
}

#[test]
fn lighting_numeric_level_cli_failures_preserve_input_and_existing_save_as() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (xml, level, code) in [
        (LEVEL_XML.to_string(), "101", "mood_level_out_of_range"),
        (
            LEVEL_XML.replace("CapAI0=\"7\"", "CapAI0=\"7\" CapAI1=\"7\""),
            "37",
            "incompatible_mood_output_kind",
        ),
        (
            LEVEL_XML.replace("CapAI0=\"7\"", "CapAI0=\"7\" CapAI1=\"opaque\""),
            "37",
            "unsupported_mood_capability",
        ),
    ] {
        for save_as in [false, true] {
            let before = fixture_bytes(&xml);
            fs::write(&input, &before).unwrap();
            fs::write(&output, b"existing destination").unwrap();
            let mut command = lox();
            command
                .env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["config", "lighting", "set-mood-level"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--mood",
                    "Evening",
                    "--output-index",
                    "2",
                    "--level",
                    level,
                    "-o",
                    "json",
                ]);
            if save_as {
                command.arg("--save-as").arg(&output);
            }
            let result = command.assert().failure().get_output().stdout.clone();
            let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(result["error"], code);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing destination");
        }
    }
}

#[test]
fn lighting_recount_cli_repairs_num_in_place_or_save_as_without_filling_sparse_values() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-recount-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    for save_as in [false, true] {
        let input = dir.path().join("input.Loxone");
        let output = dir.path().join("output.Loxone");
        let before = fixture_bytes(XML);
        fs::write(&input, &before).unwrap();
        let mut command = lox();
        command
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["config", "lighting", "recount-moods"])
            .arg(&input)
            .args(["uuid:lighting", "--quiet", "-o", "json"]);
        if save_as {
            command.arg("--save-as").arg(&output);
        }
        let result = command.assert().success().get_output().stdout.clone();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["before"], "99");
        assert_eq!(result["after"], "2");
        assert_eq!(result["scenes"], 2);
        assert_eq!(result["changed"], true);
        let after = fs::read(if save_as { &output } else { &input }).unwrap();
        assert!(after.starts_with(&[0xef, 0xbb, 0xbf]));
        let text = std::str::from_utf8(&after[3..]).unwrap();
        assert!(!text.replace("\r\n", "").contains('\n'));
        let actual = xmltree::Element::parse(&after[3..]).unwrap();
        let expected = XML.replace("Num=\"99\"", "Num=\"2\"");
        let expected = xmltree::Element::parse(expected.as_bytes()).unwrap();
        assert_eq!(actual, expected);
        if save_as {
            assert_eq!(fs::read(&input).unwrap(), before);
        }
    }
}

#[test]
fn lighting_recount_cli_noop_preserves_original_bytes_and_supports_csv() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-recount-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("input.Loxone");
    let before = fixture_bytes(&XML.replace("Num=\"99\"", "Num=\"0002\""));
    fs::write(&path, &before).unwrap();
    lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "recount-moods"])
        .arg(&path)
        .args(["uuid:lighting", "-o", "csv", "--no-header"])
        .assert()
        .success()
        .stdout("\"0002\",\"0002\",2,false\n");
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn lighting_recount_cli_failure_never_writes_input_or_save_as() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-recount-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = fixture_bytes(&XML.replace(
        "<LightscenesC Num=\"99\"",
        "<LightscenesC Num=\"71\"/><LightscenesC Num=\"99\"",
    ));
    fs::write(&input, &before).unwrap();
    let result = lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "recount-moods"])
        .arg(&input)
        .args(["uuid:lighting", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"], "unsupported_mood_container_shape");
    assert_eq!(fs::read(&input).unwrap(), before);
    assert!(!output.exists());
}
