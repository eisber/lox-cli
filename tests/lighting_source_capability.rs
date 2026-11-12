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
fn lighting_identity_inspection_cli_keeps_diagnostics_in_all_formats_without_writes() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = fixture_bytes(&XML.replace("SID=\"3\" CID=\"10\"", "SID=\"+02\" CID=\"+09\""));
    fs::write(&input, &before).unwrap();
    for format in ["json", "table", "csv"] {
        let result = lox()
            .args(["config", "lighting", "inspect"])
            .arg(&input)
            .args(["uuid:lighting", "-o", format, "--no-header", "--quiet"])
            .assert()
            .success()
            .get_output()
            .clone();
        if format == "json" {
            let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            for code in ["duplicate_mood_sid", "duplicate_mood_cid"] {
                assert!(
                    value["diagnostics"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|d| d["code"] == code)
                );
            }
            assert_eq!(value["moods"][1]["attributes"]["SID"], "+02");
        } else {
            let stderr = String::from_utf8(result.stderr).unwrap();
            assert!(stderr.contains("duplicate_mood_sid") && stderr.contains("duplicate_mood_cid"));
        }
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}

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
fn lighting_numeric_level_cli_dry_run_matches_writer_and_preserves_every_destination() {
    let dir = tempfile::Builder::new()
        .prefix("level-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (raw, capability, level, after, changed) in [
        ("1610622736", "", "0", "1610612736", true),
        ("1610622736", "+0032", "100", "1610612836", true),
        ("+01610612773", "+0006", "37", "+01610612773", false),
        ("536870921", "opaque-unused", "37", "536870949", true),
    ] {
        let xml = LEVEL_XML
            .replace("Q2=\"1610622736\"", &format!("Q2=\"{raw}\""))
            .replace(
                "CapAI0=\"7\"",
                &if capability.is_empty() {
                    "CapAI0=\"7\"".to_owned()
                } else {
                    format!("CapAI0=\"7\" CapAI1=\"{capability}\"")
                },
            )
            .replace(
                "SID=\"1\" CID=\"12\"",
                "SID=\"+0001\" CID=\"opaque\" MixMod=\"opaque\" SkipNext=\"future\" SName=\"alias\"",
            );
        let before = fixture_bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut preview = serde_json::Value::Null;
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut cmd = lox();
            cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "lighting", "set-mood-level"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--mood",
                    "Evening",
                    "--output-index",
                    "2",
                    "--level",
                    level,
                    "--json",
                    "--quiet",
                    "--trace-id",
                    "level-preview",
                ]);
            if let Some(destination) = destination {
                cmd.arg("--save-as").arg(destination);
            }
            let result = cmd
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            preview = serde_json::from_slice(&result).unwrap();
            assert_eq!(preview["dry_run"], true);
            assert_eq!(preview["trace_id"], "level-preview");
            assert_eq!(preview["selected_mood"]["uuid"], "mood");
            assert_eq!(preview["selected_mood"]["sid"], "+0001");
            assert_eq!(preview["selected_mood"]["cid"], "opaque");
            assert_eq!(preview["output"], "Q2");
            assert_eq!(preview["output_index"], 2);
            assert_eq!(preview["editor_kind"], "numeric");
            assert_eq!(preview["before"], raw);
            assert_eq!(preview["after"], after);
            assert_eq!(
                preview["before_effective"],
                raw.parse::<i32>().unwrap() & 0x0fff_ffff
            );
            assert_eq!(preview["after_effective"], level.parse::<u8>().unwrap());
            assert_eq!(preview["changed"], changed);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
            assert!(!missing.parent().unwrap().exists());
            assert!(!dir.path().join("absent.yaml").exists());
        }
        let result = lox()
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
                "--json",
                "--quiet",
                "--save-as",
            ])
            .arg(&output)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        for key in [
            "dry_run",
            "trace_id",
            "selected_mood",
            "output_index",
            "editor_kind",
            "before_effective",
            "after_effective",
        ] {
            preview.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(
            preview,
            serde_json::from_slice::<serde_json::Value>(&result).unwrap()
        );
        assert_eq!(
            fs::read(&output).unwrap(),
            fixture_bytes(&xml.replace(&format!("Q2=\"{raw}\""), &format!("Q2=\"{after}\"")))
        );
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}

#[test]
fn lighting_numeric_level_cli_dry_run_noop_exposes_raw_scalar_and_classification_in_all_formats() {
    let dir = tempfile::Builder::new()
        .prefix("level-formats-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let before = fixture_bytes(
        &LEVEL_XML
            .replace("Q2=\"1610622736\"", "Q2=\"+01610612773\"")
            .replace("CapAI0=\"7\">", "CapAI0=\"7\" CapAI1=\"+0006\"><IoData Pr=\"room\"/>")
            .replace("<ControlList>", "<ControlList><C Type=\"Place\" U=\"room\" Title=\"Kitchen\"/><C Type=\"And\" U=\"other\" Title=\"Lamp\"/>"),
    );
    fs::write(&input, &before).unwrap();
    for format in ["table", "csv", "json"] {
        for quiet in [false, true] {
            for no_header in [false, true] {
                let mut cmd = lox();
                let selector = if quiet { "Lamp" } else { "Lamp [Kitchen]" };
                cmd.args(["config", "lighting", "set-mood-level"])
                    .arg(&input)
                    .args([
                        selector,
                        "--mood",
                        "1",
                        "--output-index",
                        "2",
                        "--level",
                        "37",
                        "--dry-run",
                        "-o",
                        format,
                        "--trace-id",
                        "level-formats",
                    ]);
                if quiet {
                    cmd.args(["--quiet", "--room", "Kitchen"]);
                }
                if no_header {
                    cmd.arg("--no-header");
                }
                let result = cmd
                    .assert()
                    .success()
                    .stderr("")
                    .get_output()
                    .stdout
                    .clone();
                let text = String::from_utf8(result).unwrap();
                match format {
                    "json" => {
                        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
                        assert_eq!(value["trace_id"], "level-formats");
                        assert_eq!(value["before"], "+01610612773");
                        assert_eq!(value["after"], "+01610612773");
                        assert_eq!(value["before_effective"], 37);
                        assert_eq!(value["after_effective"], 37);
                        assert_eq!(value["capability"]["stored"], "+0006");
                        assert_eq!(value["capability"]["effective"], 6);
                        assert_eq!(value["changed"], false);
                    }
                    "csv" => {
                        assert_eq!(text.starts_with("MOOD,"), !no_header);
                        assert_eq!(
                            text.lines().last().unwrap(),
                            "\"1\",true,\"mood\",true,\"1\",true,\"12\",Q2,2,numeric,\"+01610612773\",\"+01610612773\",37,37,37,6,true,true,\"+0006\",6,false"
                        );
                        assert!(!text.contains("dry-run wrote") && !text.contains("level-formats"));
                    }
                    _ => {
                        assert_eq!(text.starts_with("MOOD_UUID"), !no_header);
                        assert!(text.contains("Q2/2  numeric"));
                        assert!(text.contains("\"+01610612773\"→\"+01610612773\""));
                        assert!(text.contains("37→37  6  \"+0006\"/6  false"));
                        assert_eq!(text.contains("dry-run wrote no files"), !quiet);
                        assert!(!text.contains("level-formats"));
                    }
                }
                assert_eq!(fs::read(&input).unwrap(), before);
            }
        }
    }
}

#[test]
fn lighting_numeric_level_cli_dry_run_and_writer_share_ordered_errors_without_writes() {
    let dir = tempfile::Builder::new()
        .prefix("level-errors-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (xml, selector, mood, index, level, code) in [
        (
            LEVEL_XML.to_owned(),
            "missing",
            "missing",
            "0",
            "101",
            "mood_level_out_of_range",
        ),
        (
            LEVEL_XML.to_owned(),
            "missing",
            "missing",
            "0",
            "37",
            "unsupported_mood_output_index",
        ),
        (
            LEVEL_XML.replace("LightController2", "Switch"),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "wrong_block_type",
        ),
        (
            LEVEL_XML.replace("</C>", "<LightscenesC/></C>"),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "unsupported_mood_container_shape",
        ),
        (
            LEVEL_XML.to_owned(),
            "uuid:lighting",
            "missing",
            "2",
            "37",
            "ambiguous_or_missing_mood",
        ),
        (
            LEVEL_XML.replace("Name=\"Evening\"", "Name=\"missing\""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "ambiguous_or_missing_mood",
        ),
        (
            LEVEL_XML.replace(
                "</LightscenesC>",
                "<LightsceneC Name=\"1\" SID=\"2\"/></LightscenesC>",
            ),
            "uuid:lighting",
            "1",
            "2",
            "37",
            "ambiguous_or_missing_mood",
        ),
        (
            LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"bad\""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "unsupported_mood_output_width",
        ),
        (
            LEVEL_XML.replace("Q2=\"1610622736\"", ""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "unsupported_mood_output_value",
        ),
        (
            LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"2147483648\""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "unsupported_mood_output_value",
        ),
        (
            LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"1\""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "unsupported_mood_output_width",
        ),
        (
            LEVEL_XML.replace("CapAI0=\"7\"", "CapAI0=\"7\" CapAI1=\"65536\""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "unsupported_mood_capability",
        ),
        (
            LEVEL_XML.to_owned(),
            "uuid:lighting",
            "Evening",
            "1",
            "37",
            "incompatible_mood_output_kind",
        ),
        (
            LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"-1\""),
            "uuid:lighting",
            "Evening",
            "2",
            "37",
            "incompatible_mood_output_kind",
        ),
    ] {
        let before = fixture_bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut expected = None;
        for dry_run in [false, true] {
            for destination in [None, Some(&input), Some(&output), Some(&missing)] {
                let mut cmd = lox();
                cmd.args(["config", "lighting", "set-mood-level"])
                    .arg(&input)
                    .args([
                        selector,
                        "--mood",
                        mood,
                        "--output-index",
                        index,
                        "--level",
                        level,
                        "--json",
                        "--quiet",
                        "--trace-id",
                        "level-error",
                    ]);
                if dry_run {
                    cmd.arg("--dry-run");
                }
                if let Some(destination) = destination {
                    cmd.arg("--save-as").arg(destination);
                }
                let result = cmd.assert().failure().get_output().stdout.clone();
                let error: serde_json::Value = serde_json::from_slice(&result).unwrap();
                assert_eq!(error["error"], code);
                if let Some(expected) = &expected {
                    assert_eq!(&error, expected);
                } else {
                    expected = Some(error);
                }
                assert_eq!(fs::read(&input).unwrap(), before);
                assert_eq!(fs::read(&output).unwrap(), b"existing output");
                assert!(!missing.parent().unwrap().exists());
            }
        }
    }
    for dry_run in [false, true] {
        let mut cmd = lox();
        cmd.args(["config", "lighting", "set-mood-level"])
            .arg(&input)
            .args([
                "Lamp [Room]",
                "--room",
                "Room",
                "--mood",
                "1",
                "--level",
                "37",
                "--json",
            ]);
        if dry_run {
            cmd.arg("--dry-run");
        }
        let result = cmd.assert().failure().get_output().stdout.clone();
        let error: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(error["error"], "conflicting_room_selector");
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
