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

#[test]
fn lighting_raw_color_cli_dry_run_matches_existing_parser_writer_and_destinations() {
    let dir = tempfile::Builder::new()
        .prefix("raw-color-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (attribute, color, index, value, changed) in [
        (
            "Q2=\"future\"",
            "hsv(120,100,10)",
            "2",
            1_610_622_736_i64,
            true,
        ),
        (
            "Q2=\"1610622736\"",
            " +001610622736 ",
            "2",
            1_610_622_736,
            false,
        ),
        (
            "Q2=\"+01610622736\"",
            " HSV(120,100,10) ",
            "2",
            1_610_622_736,
            true,
        ),
        (
            "Q2=\"-2147483648\"",
            "rgb(0,255,0)",
            "2",
            1_610_712_736,
            true,
        ),
        ("", "40100", "99", 1_610_628_775, true),
        ("Q2=\"opaque\"", "2610612736", "2", 2_610_612_736, true),
        ("Q2=\"opaque\"", "5610612736", "2", 5_610_612_736, true),
        ("Q2=\"opaque\"", "-1000000000", "2", 1_610_612_736, true),
    ] {
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList>\n\t<C Type=\"LightController2\" U=\"lighting\" Title=\"Lamp\" CapAI1=\"opaque\">\n\t\t<Unknown><LightsceneC UUID=\"first\" Name=\"A\" SID=\"future\" CID=\"opaque\" Outputs=\"bad\" {attribute}><SeqConf Future=\"keep\"/></LightsceneC></Unknown>\n\t\t<LightscenesC Num=\"bad\"><LightsceneC UUID=\"later\" Name=\"future\" SID=\"A\" Q2=\"other\"/></LightscenesC>\n\t\t<LightscenesC/><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/><HCL T=\"2\"/>\n\t</C>\n</ControlList>\n"
        );
        let before = fixture_bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut preview = serde_json::Value::Null;
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut cmd = lox();
            cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "set-mood-color"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", "--output-index", index])
                .arg(format!("--color={color}"))
                .args(["--json", "--quiet", "--trace-id", "raw-color-preview"]);
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
            assert_eq!(preview["trace_id"], "raw-color-preview");
            assert_eq!(preview["scope"], "raw_packed_edit");
            assert_eq!(preview["controller_uuid"], "lighting");
            assert_eq!(preview["controller_title"], "Lamp");
            assert_eq!(preview["selected_mood"]["uuid"], "first");
            assert_eq!(preview["selected_mood"]["sid"], "future");
            assert_eq!(preview["selected_mood"]["cid"], "opaque");
            assert_eq!(preview["selected_mood_name"], "A");
            assert_eq!(preview["selected_mood_path"], serde_json::json!([0, 0]));
            assert_eq!(preview["output"], format!("Q{index}"));
            assert_eq!(preview["output_index"], index.parse::<usize>().unwrap());
            assert_eq!(preview["value"], value);
            assert_eq!(preview["after"], value.to_string());
            assert_eq!(preview["changed"], changed);
            for key in [
                "editor_kind",
                "capability",
                "before_effective",
                "after_effective",
            ] {
                assert!(preview.get(key).is_none());
            }
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
            assert!(!missing.parent().unwrap().exists());
            assert!(!dir.path().join("absent.yaml").exists());
        }
        let report = lox()
            .args(["config", "set-mood-color"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", "--output-index", index])
            .arg(format!("--color={color}"))
            .args(["--json", "--quiet", "--save-as"])
            .arg(&output)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&report).unwrap(),
            serde_json::json!({"ok": true, "mood": "A", "output": index.parse::<usize>().unwrap(), "value": value})
        );
        let mut expected =
            xmltree::Element::parse(before.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap()).unwrap();
        expected
            .get_mut_child("C")
            .unwrap()
            .get_mut_child("Unknown")
            .unwrap()
            .get_mut_child("LightsceneC")
            .unwrap()
            .attributes
            .insert(
                format!("Q{index}"),
                preview["after"].as_str().unwrap().to_owned(),
            );
        let written = fs::read(&output).unwrap();
        assert!(written.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            !std::str::from_utf8(&written[3..])
                .unwrap()
                .replace("\r\n", "")
                .contains('\n')
        );
        assert_eq!(xmltree::Element::parse(&written[3..]).unwrap(), expected);
        assert_eq!(fs::read(&input).unwrap(), before);
    }
}

#[test]
fn lighting_raw_color_cli_dry_run_presence_escaping_and_noop_formats() {
    let dir = tempfile::Builder::new()
        .prefix("raw-color-formats-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    for attribute in ["Q2=\"1610622736\"", ""] {
        let before = fixture_bytes(&format!(
            "<ControlList><C Type=\"Place\" U=\"room\" Title=\"Kitchen\"/><C Type=\"And\" U=\"other\" Title=\"Lamp\"/><C Type=\"LightController2\" U=\"lighting\" Title=\"Lamp\"><IoData Pr=\"room\"/><Unknown><LightsceneC UUID=\"a,&quot;b\" Name=\"A,&quot;B\" SID=\"+0002\" {attribute}/></Unknown></C></ControlList>\n"
        ));
        fs::write(&input, &before).unwrap();
        for format in ["table", "csv", "json"] {
            for quiet in [false, true] {
                for no_header in [false, true] {
                    let mut cmd = lox();
                    cmd.args(["config", "set-mood-color"]).arg(&input).args([
                        "Lamp [Kitchen]",
                        "--mood",
                        "A,\"B",
                        "--output-index",
                        "2",
                        "--color",
                        "1610622736",
                        "--dry-run",
                        "-o",
                        format,
                        "--trace-id",
                        "raw-formats",
                    ]);
                    if quiet {
                        cmd.arg("--quiet");
                    }
                    if no_header {
                        cmd.arg("--no-header");
                    }
                    let text = String::from_utf8(
                        cmd.assert()
                            .success()
                            .stderr("")
                            .get_output()
                            .stdout
                            .clone(),
                    )
                    .unwrap();
                    match format {
                        "json" => {
                            let value: serde_json::Value = serde_json::from_str(&text).unwrap();
                            assert_eq!(value["trace_id"], "raw-formats");
                            assert_eq!(value["selected_mood"]["uuid"], "a,\"b");
                            assert_eq!(value["selected_mood_name"], "A,\"B");
                            assert!(value["selected_mood"]["cid"].is_null());
                            assert_eq!(
                                value["before"],
                                if attribute.is_empty() {
                                    serde_json::Value::Null
                                } else {
                                    "1610622736".into()
                                }
                            );
                            assert_eq!(value["changed"], attribute.is_empty());
                        }
                        "csv" => {
                            assert_eq!(text.starts_with("CONTROLLER_UUID_PRESENT,"), !no_header);
                            assert!(text.contains("\"a,\"\"b\"") && text.contains("\"A,\"\"B\""));
                            assert!(text.contains("false,\"\",true,\"A,"));
                            assert!(text.ends_with(&format!(
                                ",2,1610622736,{},raw_packed_edit\n",
                                attribute.is_empty()
                            )));
                            assert!(
                                !text.contains("dry-run wrote") && !text.contains("raw-formats")
                            );
                        }
                        _ => {
                            assert_eq!(text.starts_with("RAW PACKED EDIT"), !no_header);
                            assert!(text.contains("CID  <absent>"));
                            assert!(text.contains("OUTPUT  \"Q2\""));
                            assert!(text.contains("SCOPE  raw_packed_edit"));
                            assert_eq!(text.contains("dry-run wrote no files"), !quiet);
                            assert!(!text.contains("raw-formats"));
                        }
                    }
                    assert_eq!(fs::read(&input).unwrap(), before);
                }
            }
        }
    }
}

#[test]
fn lighting_raw_color_cli_dry_run_does_not_skip_existing_noop_save_or_stamp_rules() {
    let dir = tempfile::Builder::new()
        .prefix("raw-color-save-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    let before = fixture_bytes(
        "<ControlList><C Type=\"Document\" U=\"doc\" Date=\"2020-01-01 00:00:00\" DateS=\"347068800\"><C Type=\"LightController2\" U=\"lighting\"><LightsceneC Name=\"A\" Q1=\"1610622736\"/></C></C></ControlList>\n",
    );
    fs::write(&input, &before).unwrap();
    lox()
        .args(["config", "set-mood-color"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--color",
            "1610622736",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"changed\":false"));
    assert_eq!(fs::read(&input).unwrap(), before);
    lox()
        .args(["config", "set-mood-color"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--color",
            "1610622736",
            "--quiet",
            "--save-as",
        ])
        .arg(&output)
        .assert()
        .success();
    let written = fs::read(&output).unwrap();
    let actual = xmltree::Element::parse(&written[3..]).unwrap();
    let document = actual.get_child("C").unwrap();
    assert_eq!(document.attributes["Date"], "2020-01-01 00:00:00");
    assert_eq!(document.attributes["DateS"], "347068800");
    let expected = xmltree::Element::parse(&before[3..]).unwrap();
    assert_eq!(actual, expected);
    for dry_run in [true, false] {
        let mut cmd = lox();
        cmd.args(["config", "set-mood-color"])
            .arg(&output)
            .args([
                "uuid:lighting",
                "--mood",
                "A",
                "--color",
                "1610622736",
                "--quiet",
                "--save-as",
            ])
            .arg(&missing);
        if dry_run {
            cmd.args(["--dry-run", "--json"])
                .assert()
                .success()
                .stderr("")
                .stdout(predicates::str::contains("\"changed\":false"));
        } else {
            cmd.assert()
                .failure()
                .stdout("")
                .stderr(predicates::str::contains("Cannot resolve output directory"));
        }
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), written);
        assert!(!missing.parent().unwrap().exists());
    }
    let changed = std::str::from_utf8(&before[3..])
        .unwrap()
        .replace("Q1=\"1610622736\"", "Q1=\"+01610622736\"");
    let changed = [&[0xef, 0xbb, 0xbf][..], changed.as_bytes()].concat();
    fs::write(&input, &changed).unwrap();
    lox()
        .args(["config", "set-mood-color"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--color",
            "1610622736",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .stderr("")
        .stdout(predicates::str::contains("\"changed\":true"));
    assert_eq!(fs::read(&input).unwrap(), changed);
    lox()
        .args(["config", "set-mood-color"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--color",
            "1610622736",
            "--quiet",
            "--save-as",
        ])
        .arg(&output)
        .assert()
        .success();
    let written = fs::read(&output).unwrap();
    let actual = xmltree::Element::parse(&written[3..]).unwrap();
    let document = actual.get_child("C").unwrap();
    assert_ne!(document.attributes["Date"], "2020-01-01 00:00:00");
    assert!(document.attributes["DateS"].parse::<i64>().unwrap() > 347068800);
    let mut expected = expected;
    for key in ["Date", "DateS"] {
        expected
            .get_mut_child("C")
            .unwrap()
            .attributes
            .insert(key.into(), document.attributes[key].clone());
    }
    assert_eq!(actual, expected);
    assert_eq!(fs::read(&input).unwrap(), changed);
}

#[test]
fn lighting_raw_color_cli_dry_run_and_writer_preserve_existing_ordered_failures() {
    let dir = tempfile::Builder::new()
        .prefix("raw-color-errors-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (xml, selector, mood, index, color, message) in [
        (
            "<ControlList><C".to_owned(),
            "missing",
            "missing",
            "0",
            "bad",
            "XML",
        ),
        (XML.to_owned(), "missing", "missing", "0", "bad", "not hsv"),
        (
            XML.to_owned(),
            "missing",
            "missing",
            "0",
            "4294967295",
            "outside 0..100",
        ),
        (
            XML.to_owned(),
            "missing",
            "missing",
            "0",
            "-1",
            "outside 0..255",
        ),
        (
            XML.to_owned(),
            "missing",
            "missing",
            "0",
            "9223372036854775808",
            "not hsv",
        ),
        (
            XML.to_owned(),
            "uuid:missing",
            "missing",
            "0",
            "0",
            "uuid:missing",
        ),
        (
            XML.replace(
                "</ControlList>",
                "<C Type=\"LightController2\" U=\"other\" Title=\"Lamp\"/></ControlList>",
            ),
            "Lamp",
            "missing",
            "0",
            "0",
            "matches 2 elements",
        ),
        (
            XML.replace("LightController2", "Switch"),
            "uuid:lighting",
            "missing",
            "0",
            "0",
            "not a LightController2",
        ),
        (XML.to_owned(), "uuid:lighting", "missing", "0", "0", "Q0"),
        (
            XML.to_owned(),
            "uuid:lighting",
            "missing",
            "99",
            "0",
            "mood 'missing' not found",
        ),
    ] {
        let before = fixture_bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut errors = Vec::new();
        for dry_run in [false, true] {
            for destination in [&input, &output, &missing] {
                let mut cmd = lox();
                cmd.args(["config", "set-mood-color"])
                    .arg(&input)
                    .args([selector, "--mood", mood, "--output-index", index])
                    .arg(format!("--color={color}"))
                    .args(["--json", "--save-as"])
                    .arg(destination);
                if dry_run {
                    cmd.arg("--dry-run");
                }
                let result = cmd.assert().failure().get_output().stdout.clone();
                let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
                assert_eq!(result["ok"], false);
                assert!(
                    result["message"].as_str().unwrap().contains(message),
                    "{result}"
                );
                errors.push(result);
                assert_eq!(fs::read(&input).unwrap(), before);
                assert_eq!(fs::read(&output).unwrap(), b"existing output");
                assert!(!missing.parent().unwrap().exists());
            }
        }
        assert!(errors.iter().all(|error| error == &errors[0]));
    }
    fs::remove_file(&input).unwrap();
    for dry_run in [true, false] {
        let mut cmd = lox();
        cmd.args(["config", "set-mood-color"]).arg(&input).args([
            "missing",
            "--mood",
            "missing",
            "--color",
            "bad",
            "--output-index",
            "0",
            "--json",
        ]);
        if dry_run {
            cmd.arg("--dry-run");
        }
        let result = cmd.assert().failure().get_output().stdout.clone();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert!(result["message"].as_str().unwrap().contains("Cannot read"));
        assert!(!input.exists());
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
    }
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
fn lighting_recount_cli_dry_run_matches_writer_and_preserves_destinations() {
    let dir = tempfile::Builder::new()
        .prefix(".recount-preview-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (attribute, effective, after, changed) in [
        ("", None, "2", true),
        ("Num=\"\"", None, "2", true),
        ("Num=\"future\"", None, "2", true),
        ("Num=\"2147483648\"", None, "2", true),
        ("Num=\"-1\"", Some(-1), "2", true),
        ("Num=\"99\"", Some(99), "2", true),
        ("Num=\"0002\"", Some(2), "0002", false),
        ("Num=\"+2\"", Some(2), "+2", false),
    ] {
        let xml = XML.replace("Num=\"99\"", attribute).replace(
            "<Unknown Num=\"81\"/>",
            "<Unknown Num=\"81\"><LightsceneC Num=\"90\"/><LightscenesC Num=\"91\"/></Unknown>",
        );
        let before = fixture_bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let modified = fs::metadata(&input).unwrap().modified().unwrap();
        let mut preview = serde_json::Value::Null;
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut command = lox();
            command
                .env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "lighting", "recount-moods"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--json",
                    "--quiet",
                    "--trace-id",
                    "recount-preview",
                ]);
            if let Some(destination) = destination {
                command.arg("--save-as").arg(destination);
            }
            let result = command
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            preview = serde_json::from_slice(&result).unwrap();
            assert_eq!(preview["dry_run"], true);
            assert_eq!(preview["trace_id"], "recount-preview");
            assert_eq!(preview["controller_uuid"], "lighting");
            assert_eq!(preview["controller_title"], "Lamp");
            assert_eq!(preview["before_effective"], serde_json::json!(effective));
            assert_eq!(preview["after_effective"], 2);
            assert_eq!(preview["after"], after);
            assert_eq!(preview["changed"], changed);
            assert_eq!(preview["scenes"], 2);
            let root = xmltree::Element::parse(xml.as_bytes()).unwrap();
            let controller = root.get_child("C").unwrap();
            let index = preview["collection_path"][0].as_u64().unwrap() as usize;
            assert_eq!(
                controller.children[index].as_element().unwrap().name,
                "LightscenesC"
            );
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::metadata(&input).unwrap().modified().unwrap(), modified);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
            assert!(!missing.parent().unwrap().exists());
        }
        for save_as in [false, true] {
            fs::write(&input, &before).unwrap();
            let modified = fs::metadata(&input).unwrap().modified().unwrap();
            let mut command = lox();
            command
                .env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["config", "lighting", "recount-moods"])
                .arg(&input)
                .args(["uuid:lighting", "--json", "--quiet"]);
            if save_as {
                command.arg("--save-as").arg(&output);
            }
            let result = command
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
            let expected_report = serde_json::json!({
                "before": preview["before"],
                "after": preview["after"],
                "scenes": preview["scenes"],
                "changed": preview["changed"],
            });
            assert_eq!(result, expected_report);
            let written = fs::read(if save_as { &output } else { &input }).unwrap();
            assert!(written.starts_with(&[0xef, 0xbb, 0xbf]));
            assert!(
                !std::str::from_utf8(&written[3..])
                    .unwrap()
                    .replace("\r\n", "")
                    .contains('\n')
            );
            let expected = if changed {
                if attribute.is_empty() {
                    xml.replacen("<LightscenesC ", "<LightscenesC Num=\"2\" ", 1)
                } else {
                    xml.replace(attribute, "Num=\"2\"")
                }
            } else {
                xml.clone()
            };
            assert_eq!(
                xmltree::Element::parse(&written[3..]).unwrap(),
                xmltree::Element::parse(expected.as_bytes()).unwrap()
            );
            if save_as || !changed {
                assert_eq!(fs::read(&input).unwrap(), before);
                assert_eq!(fs::metadata(&input).unwrap().modified().unwrap(), modified);
            }
        }
    }
}

#[test]
fn lighting_recount_cli_dry_run_formats_preserve_presence_spelling_and_room_identity() {
    let dir = tempfile::Builder::new()
        .prefix(".recount-formats-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    for (attribute, prior, effective, after) in [
        ("", None, None, "0"),
        (" Num=\"\"", Some(""), None, "0"),
        (" Num=\"µ,&quot;bad&quot;\"", Some("µ,\"bad\""), None, "0"),
        (" Num=\"-1\"", Some("-1"), Some(-1), "0"),
        (" Num=\"+000\"", Some("+000"), Some(0), "+000"),
    ] {
        let xml = format!(
            "<ControlList><C Type=\"Place\" U=\"room\" Title=\"Study\"/><C Type=\"LightController2\" U=\"actual\" Title=\"Lamp, &quot;µ&quot;\"><IoData Pr=\"room\"/><LightscenesC{attribute}/></C><C Type=\"LightController2\" U=\"other\" Title=\"Lamp\"><LightscenesC/></C></ControlList>"
        );
        let before = fixture_bytes(&xml);
        fs::write(&input, &before).unwrap();
        for (format, flags) in [
            (
                "json",
                vec!["--quiet", "--no-header", "--trace-id", "format-preview"],
            ),
            ("csv", vec![]),
            ("csv", vec!["--quiet", "--no-header"]),
            ("table", vec![]),
            ("table", vec!["--quiet", "--no-header"]),
        ] {
            let result = lox()
                .env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "lighting", "recount-moods"])
                .arg(&input)
                .args(["Lamp", "--room", "Study", "-o", format])
                .args(&flags)
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            let text = String::from_utf8(result).unwrap();
            if format == "json" {
                let result: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(result["before"], serde_json::json!(prior));
                assert_eq!(result["before_effective"], serde_json::json!(effective));
                assert_eq!(result["after"], after);
                assert_eq!(result["controller_uuid"], "actual");
                assert_eq!(result["controller_title"], "Lamp, \"µ\"");
                assert_eq!(result["trace_id"], "format-preview");
            } else if format == "csv" {
                assert_eq!(
                    text.starts_with("CONTROLLER_UUID_PRESENT,"),
                    flags.is_empty()
                );
                let data = text.lines().last().unwrap();
                let quote = |value: &str| format!("\"{}\"", value.replace('"', "\"\""));
                assert!(data.contains(&format!(
                    ",{},{},true,{},",
                    prior.is_some(),
                    quote(prior.unwrap_or("")),
                    quote(after)
                )));
                assert!(data.contains("\"Lamp, \"\"µ\"\"\""));
                assert!(data.ends_with(&format!(
                    ",0,0,{},stored_mood_count_repair",
                    effective != Some(0)
                )));
                assert_eq!(text.lines().count(), if flags.is_empty() { 2 } else { 1 });
            } else {
                assert_eq!(text.starts_with("COUNT REPAIR PREVIEW"), flags.is_empty());
                assert!(text.contains(&format!(
                    "BEFORE  {}\nAFTER  {after:?}",
                    prior.map_or_else(|| "<absent>".to_owned(), |value| format!("{value:?}"))
                )));
                assert!(text.contains("CONTROLLER_UUID  \"actual\""));
                assert_eq!(text.contains("dry-run wrote no files"), flags.is_empty());
            }
            assert_eq!(fs::read(&input).unwrap(), before);
        }
    }
}

#[test]
fn lighting_recount_cli_dry_run_and_writer_share_ordered_atomic_errors() {
    let dir = tempfile::Builder::new()
        .prefix(".recount-errors-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("missing").join("output.Loxone");
    for (xml, selector, room, expected) in [
        (
            None,
            "Lamp [Study]",
            Some("Study"),
            "conflicting_room_selector",
        ),
        (None, "Lamp", None, "Cannot read"),
        (Some("<ControlList><C"), "Lamp", None, "XML"),
        (Some("<ControlList/>"), "Lamp", None, "not found"),
        (
            Some("<ControlList><C Title=\"Lamp\"/><C Title=\"Lamp\"/></ControlList>"),
            "Lamp",
            None,
            "matches 2",
        ),
        (
            Some("<ControlList><C Type=\"And\" Title=\"Lamp\"/></ControlList>"),
            "Lamp",
            None,
            "wrong_block_type",
        ),
        (
            Some(
                "<ControlList><C Type=\"LightController2\" Title=\"Lamp\"><Unknown><LightscenesC/></Unknown></C></ControlList>",
            ),
            "Lamp",
            None,
            "unsupported_mood_container_shape",
        ),
        (
            Some(
                "<ControlList><C Type=\"LightController2\" Title=\"Lamp\"><LightscenesC/><LightscenesC/></C></ControlList>",
            ),
            "Lamp",
            None,
            "unsupported_mood_container_shape",
        ),
    ] {
        if let Some(xml) = xml {
            fs::write(&input, fixture_bytes(xml)).unwrap();
        }
        let before = fs::read(&input).ok();
        let mut results = Vec::new();
        for dry_run in [true, false] {
            let mut command = lox();
            command.env("LOX_CONFIG", dir.path().join("absent.yaml"));
            if dry_run {
                command.arg("--dry-run");
            }
            command
                .args(["config", "lighting", "recount-moods"])
                .arg(&input)
                .arg(selector)
                .args(["--save-as"])
                .arg(&output)
                .arg("--json");
            if let Some(room) = room {
                command.args(["--room", room]);
            }
            let result = command.assert().failure().get_output().stdout.clone();
            let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert!(result.to_string().contains(expected), "{result}");
            results.push(result);
            assert_eq!(fs::read(&input).ok(), before);
            assert!(!output.parent().unwrap().exists());
        }
        assert_eq!(results[0], results[1]);
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
