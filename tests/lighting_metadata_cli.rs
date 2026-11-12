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
fn lighting_metadata_cli_skip_dry_run_matches_writer_and_preserves_every_destination() {
    let dir = tempfile::Builder::new()
        .prefix("skip-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (raw, skip) in [
        (None, false),
        (None, true),
        (Some("FaLsE"), true),
        (Some("tRuE"), false),
        (Some("tRuE"), true),
    ] {
        let attribute = raw
            .map(|raw| format!(" SkipNext=\"{raw}\""))
            .unwrap_or_default();
        let xml = XML
            .replacen(
                "SID=\"+0002\" CID=\"0\"",
                "SID=\"+000777\" CID=\"opaque\"",
                1,
            )
            .replacen("SName=\"alias\"", &format!("SName=\"alias\"{attribute}"), 1);
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let flag = if skip { "--skip" } else { "--no-skip" };
        let mut preview = serde_json::Value::Null;
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut cmd = command();
            cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "lighting", "set-mood-skip"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--mood",
                    "A",
                    flag,
                    "-o",
                    "json",
                    "--quiet",
                    "--trace-id",
                    "skip-preview",
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
            assert_eq!(preview["trace_id"], "skip-preview");
            assert_eq!(preview["mood"]["sid"], "+000777");
            assert_eq!(preview["mood"]["cid"], "opaque");
            assert_eq!(preview["before"]["stored"], serde_json::json!(raw));
            assert_eq!(preview["skip"], skip);
            assert_eq!(preview["checkbox_checked"], !skip);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
            assert!(!missing.parent().unwrap().exists());
            assert!(!dir.path().join("absent.yaml").exists());
        }
        let result = command()
            .args(["config", "lighting", "set-mood-skip"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--mood",
                "A",
                flag,
                "-o",
                "json",
                "--quiet",
                "--save-as",
            ])
            .arg(&output)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let effective = raw.is_some_and(|raw| raw.eq_ignore_ascii_case("true"));
        let expected = if skip == effective {
            xml
        } else if raw.is_none() {
            xml.replacen("SName=\"alias\"", "SName=\"alias\" SkipNext=\"true\"", 1)
        } else {
            xml.replacen(&attribute, &format!(" SkipNext=\"{skip}\""), 1)
        };
        preview.as_object_mut().unwrap().remove("dry_run");
        preview.as_object_mut().unwrap().remove("trace_id");
        assert_eq!(
            preview,
            serde_json::from_slice::<serde_json::Value>(&result).unwrap()
        );
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
    }
}

#[test]
fn lighting_metadata_cli_skip_dry_run_noop_exposes_raw_and_effective_values_in_all_formats() {
    let dir = tempfile::Builder::new()
        .prefix("skip-preview-formats-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (raw, cid, skip) in [
        (None, None, false),
        (Some("FaLsE"), Some("opaque"), false),
        (Some("tRuE"), Some("+012"), true),
    ] {
        let attribute = raw
            .map(|raw| format!(" SkipNext=\"{raw}\""))
            .unwrap_or_default();
        let cid_attribute = cid.map(|cid| format!(" CID=\"{cid}\"")).unwrap_or_default();
        let xml = XML.replacen("SID=\"+0002\" CID=\"0\"", &format!("SID=\"+000777\"{cid_attribute}{attribute}"), 1)
            .replace("<ControlList>", "<ControlList><C Type=\"Place\" U=\"room\" Title=\"Kitchen\"/><C Type=\"And\" U=\"other\" Title=\"Lamp\"/>")
            .replace("Title=\"Lamp\">", "Title=\"Lamp\"><IoData Pr=\"room\"/>");
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        for (block, room) in [
            ("uuid:lighting", None),
            ("Lamp", Some("Kitchen")),
            ("Lamp [Kitchen]", None),
        ] {
            for format in ["json", "table", "csv"] {
                for (quiet, no_header) in
                    [(false, false), (true, false), (false, true), (true, true)]
                {
                    let mut cmd = command();
                    cmd.args(["--dry-run", "config", "lighting", "set-mood-skip"])
                        .arg(&input)
                        .args([
                            block,
                            "--mood",
                            "+000777",
                            if skip { "--skip" } else { "--no-skip" },
                            "-o",
                            format,
                            "--save-as",
                        ])
                        .arg(&missing);
                    if let Some(room) = room {
                        cmd.args(["--room", room]);
                    }
                    if quiet {
                        cmd.arg("--quiet");
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
                    if format == "json" {
                        let value: serde_json::Value = serde_json::from_slice(&result).unwrap();
                        assert_eq!(value["dry_run"], true);
                        assert_eq!(value["mood"]["sid"], "+000777");
                        assert_eq!(value["mood"]["cid"], serde_json::json!(cid));
                        assert_eq!(value["before"]["stored"], serde_json::json!(raw));
                        assert_eq!(value["before"]["effective"], skip);
                        assert_eq!(value["before"]["default"], false);
                        assert_eq!(value["after"], serde_json::json!(raw));
                        assert_eq!(value["skip"], skip);
                        assert_eq!(value["checkbox_checked"], !skip);
                        assert_eq!(value["changed"], false);
                    } else {
                        let stdout = String::from_utf8(result).unwrap();
                        assert_eq!(stdout.contains("MOOD_UUID"), !no_header);
                        if format == "csv" {
                            let expected = format!(
                                "\"a\",\"+000777\",{},\"{}\",{},\"{}\",{},\"{}\",{skip},{skip},false,{},false\n",
                                cid.is_some(),
                                cid.unwrap_or(""),
                                raw.is_some(),
                                raw.unwrap_or(""),
                                raw.is_some(),
                                raw.unwrap_or(""),
                                !skip
                            );
                            assert!(stdout.ends_with(&expected), "{stdout}");
                        } else {
                            let stored = raw.unwrap_or("<absent>");
                            assert!(stdout.contains(&format!("{stored}→{stored}")), "{stdout}");
                            assert!(stdout.contains(&format!("{skip}→{skip}")), "{stdout}");
                            assert!(
                                stdout
                                    .contains(&format!("+000777  {}  ", cid.unwrap_or("<absent>"))),
                                "{stdout}"
                            );
                            assert_eq!(stdout.contains("\n1 mood;"), !quiet);
                        }
                    }
                    assert_eq!(fs::read(&input).unwrap(), before);
                    assert!(!missing.parent().unwrap().exists());
                }
            }
        }
    }
}

#[test]
fn lighting_metadata_cli_skip_dry_run_and_writer_reject_identical_unsupported_data() {
    let dir = tempfile::Builder::new()
        .prefix("skip-preview-errors-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (xml, block, room, error) in [
        (
            XML.replace("SID=\"+0002\"", "SID=\"0\""),
            "uuid:lighting",
            None,
            "unsupported_mood_skip_scope",
        ),
        (
            XML.replace("SID=\"+0002\"", "SID=\"2147483648\""),
            "uuid:lighting",
            None,
            "unsupported_mood_identity",
        ),
        (
            XML.replace("SID=\"3\"", "SID=\"opaque\""),
            "uuid:lighting",
            None,
            "unsupported_mood_identity",
        ),
        (
            XML.replace("SName=\"alias\"", "SName=\"alias\" SkipNext=\"0\""),
            "uuid:lighting",
            None,
            "unsupported_mood_skip_value",
        ),
        (
            XML.replace("SID=\"3\"", "SID=\"0002\""),
            "uuid:lighting",
            None,
            "ambiguous_mood_sid",
        ),
        (
            XML.replace("Name=\"B\"", "Name=\"A\""),
            "uuid:lighting",
            None,
            "ambiguous_or_missing_mood",
        ),
        (
            XML.replace("Name=\"A\"", "Name=\"missing\""),
            "uuid:lighting",
            None,
            "ambiguous_or_missing_mood",
        ),
        (
            XML.replace("</LightscenesC>", "</LightscenesC><LightscenesC/>"),
            "uuid:lighting",
            None,
            "unsupported_mood_container_shape",
        ),
        (
            XML.replace("Type=\"LightController2\"", "Type=\"And\""),
            "uuid:lighting",
            None,
            "wrong_block_type",
        ),
        (
            XML.to_string(),
            "Lamp [Kitchen]",
            Some("Kitchen"),
            "conflicting_room_selector",
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut errors = Vec::new();
            for dry_run in [true, false] {
                let mut cmd = command();
                cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                    .args(["config", "lighting", "set-mood-skip"])
                    .arg(&input)
                    .args([block, "--mood", "A", "--no-skip", "-o", "json", "--quiet"]);
                if let Some(destination) = destination {
                    cmd.arg("--save-as").arg(destination);
                }
                if dry_run {
                    cmd.arg("--dry-run");
                }
                if let Some(room) = room {
                    cmd.args(["--room", room]);
                }
                let result = cmd.assert().failure().get_output().stdout.clone();
                let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
                assert_eq!(result["error"], error);
                errors.push(result);
                assert_eq!(fs::read(&input).unwrap(), before);
                assert_eq!(fs::read(&output).unwrap(), b"existing output");
                assert!(!missing.parent().unwrap().exists());
                assert!(!dir.path().join("absent.yaml").exists());
            }
            assert_eq!(errors[0], errors[1]);
        }
    }
}

#[test]
fn lighting_metadata_cli_mix_dry_run_matches_writer_and_preserves_every_destination() {
    let dir = tempfile::Builder::new()
        .prefix("mix-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (xml, flag, expected) in [
        (
            XML.to_string(),
            "--mix",
            XML.replacen("MixMod=\"false\"", "MixMod=\"true\"", 1),
        ),
        (
            XML.replacen("MixMod=\"false\"", "MixMod=\"tRuE\"", 1),
            "--no-mix",
            XML.to_string(),
        ),
        (
            XML.replacen(" CID=\"0\" MixMod=\"false\"", "", 1),
            "--mix",
            XML.replacen(" CID=\"0\" MixMod=\"false\"", "", 1).replacen(
                "SName=\"alias\">",
                "SName=\"alias\" MixMod=\"true\">",
                1,
            ),
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut preview = serde_json::Value::Null;
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut cmd = command();
            cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "lighting", "set-mood-mix"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--mood",
                    "A",
                    flag,
                    "-o",
                    "json",
                    "--quiet",
                    "--trace-id",
                    "mix-preview",
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
            assert_eq!(preview["trace_id"], "mix-preview");
            assert_eq!(preview["mood"]["sid"], "+0002");
            assert_eq!(preview["changed"], true);
            assert_eq!(preview["enabled"], flag == "--mix");
            assert_eq!(preview["before_effective"], flag == "--no-mix");
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
            assert!(!missing.parent().unwrap().exists());
            assert!(!dir.path().join("absent.yaml").exists());
        }
        let result = command()
            .args(["config", "lighting", "set-mood-mix"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--mood",
                "A",
                flag,
                "-o",
                "json",
                "--quiet",
                "--save-as",
            ])
            .arg(&output)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        preview.as_object_mut().unwrap().remove("dry_run");
        preview.as_object_mut().unwrap().remove("trace_id");
        assert_eq!(
            preview,
            serde_json::from_slice::<serde_json::Value>(&result).unwrap()
        );
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
    }
}

#[test]
fn lighting_metadata_cli_mix_dry_run_noop_exposes_raw_and_effective_values_in_all_formats() {
    let dir = tempfile::Builder::new()
        .prefix("mix-preview-formats-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (attributes, raw, cid) in [
        ("", None, None),
        (
            " CID=\"+000\" MixMod=\"FaLsE\"",
            Some("FaLsE"),
            Some("+000"),
        ),
    ] {
        let xml = XML.replacen(" CID=\"0\" MixMod=\"false\"", attributes, 1)
            .replace("<ControlList>", "<ControlList><C Type=\"Place\" U=\"room\" Title=\"Kitchen\"/><C Type=\"And\" U=\"other\" Title=\"Lamp\"/>")
            .replace("Title=\"Lamp\">", "Title=\"Lamp\"><IoData Pr=\"room\"/>");
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        for (block, room) in [
            ("uuid:lighting", None),
            ("Lamp", Some("Kitchen")),
            ("Lamp [Kitchen]", None),
        ] {
            for format in ["json", "table", "csv"] {
                for quiet_noheader in [false, true] {
                    let mut cmd = command();
                    cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                        .args(["--dry-run", "config", "lighting", "set-mood-mix"])
                        .arg(&input)
                        .args([
                            block,
                            "--mood",
                            "+0002",
                            "--no-mix",
                            "-o",
                            format,
                            "--save-as",
                        ])
                        .arg(&missing);
                    if let Some(room) = room {
                        cmd.args(["--room", room]);
                    }
                    if quiet_noheader {
                        cmd.args(["--quiet", "--no-header"]);
                    }
                    let result = cmd
                        .assert()
                        .success()
                        .stderr("")
                        .get_output()
                        .stdout
                        .clone();
                    if format == "json" {
                        let value: serde_json::Value = serde_json::from_slice(&result).unwrap();
                        assert_eq!(value["dry_run"], true);
                        assert_eq!(value["mood"]["sid"], "+0002");
                        assert_eq!(value["mood"]["cid"], serde_json::json!(cid));
                        assert_eq!(value["cid_effective"], 0);
                        assert_eq!(value["before"], serde_json::json!(raw));
                        assert_eq!(value["after"], serde_json::json!(raw));
                        assert_eq!(value["before_effective"], false);
                        assert_eq!(value["enabled"], false);
                        assert_eq!(value["default"], false);
                        assert_eq!(value["changed"], false);
                    } else {
                        let stdout = String::from_utf8(result).unwrap();
                        assert_eq!(stdout.contains("MOOD_UUID"), !quiet_noheader);
                        if format == "csv" {
                            let expected = if raw.is_none() {
                                "\"a\",\"+0002\",false,\"\",0,false,\"\",false,\"\",false,false,false,false,selected_record_role_independent\n"
                            } else {
                                "\"a\",\"+0002\",true,\"+000\",0,true,\"FaLsE\",true,\"FaLsE\",false,false,false,false,selected_record_role_independent\n"
                            };
                            assert!(stdout.ends_with(expected), "{stdout}");
                        } else {
                            let stored = raw.unwrap_or("<absent>");
                            assert!(stdout.contains(&format!("{stored}→{stored}")), "{stdout}");
                            assert!(stdout.contains("false→false"), "{stdout}");
                            assert!(
                                stdout.contains(&format!("{}/0", cid.unwrap_or("<absent>"))),
                                "{stdout}"
                            );
                            assert_eq!(stdout.contains("\n1 mood;"), !quiet_noheader);
                        }
                    }
                    assert_eq!(fs::read(&input).unwrap(), before);
                    assert_eq!(fs::read(&output).unwrap(), b"existing output");
                    assert!(!missing.parent().unwrap().exists());
                    assert!(!dir.path().join("absent.yaml").exists());
                }
            }
        }
    }
}

#[test]
fn lighting_metadata_cli_mix_dry_run_and_writer_reject_identical_unsupported_data() {
    let dir = tempfile::Builder::new()
        .prefix("mix-preview-errors-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for (xml, block, room, error) in [
        (
            XML.replace("CID=\"0\"", "CID=\"12\""),
            "uuid:lighting",
            None,
            "role_dependent_mood_mix",
        ),
        (
            XML.replace("CID=\"0\"", "CID=\"opaque\""),
            "uuid:lighting",
            None,
            "unsupported_mood_identity",
        ),
        (
            XML.replace("MixMod=\"false\"", "MixMod=\"0\""),
            "uuid:lighting",
            None,
            "unsupported_mood_mix_value",
        ),
        (
            XML.replace("SID=\"+0002\"", "SID=\"32\""),
            "uuid:lighting",
            None,
            "unsupported_mood_mix_scope",
        ),
        (
            XML.replace("SID=\"3\"", "SID=\"0002\""),
            "uuid:lighting",
            None,
            "ambiguous_mood_sid",
        ),
        (
            XML.replace("Name=\"B\"", "Name=\"A\""),
            "uuid:lighting",
            None,
            "ambiguous_or_missing_mood",
        ),
        (
            XML.replace("Name=\"A\"", "Name=\"missing\""),
            "uuid:lighting",
            None,
            "ambiguous_or_missing_mood",
        ),
        (
            XML.replace("</LightscenesC>", "</LightscenesC><LightscenesC/>"),
            "uuid:lighting",
            None,
            "unsupported_mood_container_shape",
        ),
        (
            XML.replace("Type=\"LightController2\"", "Type=\"And\""),
            "uuid:lighting",
            None,
            "wrong_block_type",
        ),
        (
            XML.to_string(),
            "Lamp [Kitchen]",
            Some("Kitchen"),
            "conflicting_room_selector",
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        for destination in [&input, &output, &missing] {
            let mut errors = Vec::new();
            for dry_run in [true, false] {
                let mut cmd = command();
                cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                    .args(["config", "lighting", "set-mood-mix"])
                    .arg(&input)
                    .args([
                        block,
                        "--mood",
                        "A",
                        "--no-mix",
                        "-o",
                        "json",
                        "--quiet",
                        "--save-as",
                    ])
                    .arg(destination);
                if dry_run {
                    cmd.arg("--dry-run");
                }
                if let Some(room) = room {
                    cmd.args(["--room", room]);
                }
                let result = cmd.assert().failure().get_output().stdout.clone();
                let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
                assert_eq!(result["error"], error);
                errors.push(result);
                assert_eq!(fs::read(&input).unwrap(), before);
                assert_eq!(fs::read(&output).unwrap(), b"existing output");
                assert!(!missing.parent().unwrap().exists());
                assert!(!dir.path().join("absent.yaml").exists());
            }
            assert_eq!(errors[0], errors[1]);
        }
    }
}

#[test]
fn lighting_metadata_cli_references_preserve_exact_bytes_and_formats_with_quiet_noheader() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let xml = XML.replace(
        "OpM=\"opaque\" ScMv=\"future\"",
        "OpM=\" +00 ,1,-02\" ScMv=\" +002 , -1,+2,opaque,,tail\"",
    );
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    for format in ["json", "table", "csv"] {
        for quiet_noheader in [false, true] {
            let mut cmd = command();
            cmd.env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["config", "lighting", "inspect-mood-references"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", "-o", format]);
            if quiet_noheader {
                cmd.args(["--quiet", "--no-header"]);
            }
            let output = cmd.assert().success().get_output().clone();
            if format == "json" {
                let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                assert_eq!(value["read_only"], true);
                assert_eq!(value["mood"]["sid"], "+0002");
                assert_eq!(value["selected_sid"], 2);
                assert_eq!(value["stored_num"], "2");
                assert_eq!(value["status"], "resolved");
                assert_eq!(value["opm"], " +00 ,1,-02");
                assert_eq!(value["scmv"], " +002 , -1,+2,opaque,,tail");
                assert_eq!(value["opm_count"], 3);
                assert_eq!(value["visited"][0]["raw"], " +002 ");
                assert_eq!(value["visited"][0]["value"], 2);
                assert_eq!(value["visited"][0]["matches_selected_sid"], true);
                assert_eq!(value["visited"][1]["matches_selected_sid"], false);
                assert_eq!(value["visited"][2]["matches_selected_sid"], true);
                assert_eq!(value["unvisited_tail"]["start_index"], 3);
                assert_eq!(value["unvisited_tail"]["raw"], "opaque,,tail");
                assert_eq!(
                    value["diagnostics"][0]["code"],
                    "stored_mood_references_only"
                );
            } else {
                let stdout = String::from_utf8(output.stdout).unwrap();
                assert!(stdout.contains("selected_sid") && stdout.contains("+0002"));
                assert!(stdout.contains("opaque,,tail") && stdout.contains("unvisited_tail"));
                assert_eq!(stdout.contains("MATCHES_SELECTED_SID"), !quiet_noheader);
                if format == "csv" {
                    assert!(stdout.contains("\"visited\",0,true,\" +002 \",\"2\",true"));
                    assert!(stdout.contains("\"visited\",1,true,\" -1\",\"-1\",false"));
                    assert!(stdout.contains("\"unvisited_tail\",3,true,\"opaque,,tail\",\"3\","));
                } else {
                    assert_eq!(stdout.contains("\n3 visited references"), !quiet_noheader);
                }
                let stderr = String::from_utf8(output.stderr).unwrap();
                assert!(stderr.contains("stored_mood_references_only"));
                assert!(stderr.contains("Not Num-limited import"));
            }
            assert_eq!(fs::read(&input).unwrap(), before);
        }
    }
}

#[test]
fn lighting_metadata_cli_references_unresolved_diagnostics_are_visible_not_success_empty() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    for format in ["json", "table", "csv"] {
        let output = command()
            .args(["config", "lighting", "inspect-mood-references"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--mood",
                "A",
                "-o",
                format,
                "-q",
                "--no-header",
            ])
            .assert()
            .success()
            .get_output()
            .clone();
        if format == "json" {
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["status"], "unresolved");
            assert!(value["visited"].is_null());
            assert_eq!(value["opm"], "opaque");
            assert_eq!(value["scmv"], "future");
            assert!(
                value["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| { d["code"] == "unsupported_lighting_selection_value" })
            );
        } else {
            assert!(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("unresolved")
            );
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("unsupported_lighting_selection_value")
            );
        }
        assert_eq!(fs::read(&input).unwrap(), before);
    }
    let before = bytes(&XML.replace("OpM=\"opaque\" ScMv=\"future\"", "ScMv=\"opaque,,tail\""));
    fs::write(&input, &before).unwrap();
    let output = command()
        .args(["config", "lighting", "inspect-mood-references"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["status"], "no_visited_references");
    assert!(value["visited"].as_array().unwrap().is_empty());
    assert!(value["opm"].is_null());
    assert_eq!(value["scmv"], "opaque,,tail");
    assert_eq!(fs::read(&input).unwrap(), before);
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
