use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

const XML: &str = r#"<ControlList><C Type="LightController2" U="lighting">
  <Co K="AQ2" U="output-2"><In Input="source" FutureWire="retain"/></Co>
  <LightscenesC Num="99" Outputs="0020" Future="retain">
    <LightsceneC Name="First" UUID="first" SID="2" CID="9" Outputs="3"
      Q1="opaque-selected-output" Q2="1610612778" Q3="-1" Q10="future">
      <SeqConf Future="retain"/>
    </LightsceneC>
    <LightsceneC Name="Second" UUID="second" SID="3" CID="10" Q2="0"/>
  </LightscenesC>
</C></ControlList>
"#;

fn fixture_bytes(xml: &str) -> Vec<u8> {
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    bytes
}

#[test]
fn lighting_native_append_cli_persists_type_defaults_and_preserves_unselected_records() {
    let dir = tempfile::tempdir().unwrap();
    for save_as in [false, true] {
        let input = dir.path().join("input.Loxone");
        let output = dir.path().join("output.Loxone");
        let before = fixture_bytes(XML);
        fs::write(&input, &before).unwrap();
        let mut command = cargo_bin_cmd!("lox");
        command
            .env("LOX_TELEMETRY", "0")
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["config", "add-mood"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--name",
                "New",
                "--color",
                "hsv(120,100,10)",
                "-o",
                "json",
                "--quiet",
            ]);
        if save_as {
            command.arg("--save-as").arg(&output);
        }
        command.assert().success();
        let after = fs::read(if save_as { &output } else { &input }).unwrap();
        assert!(after.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            !std::str::from_utf8(&after[3..])
                .unwrap()
                .replace("\r\n", "")
                .contains('\n')
        );
        let document = xmltree::Element::parse(&after[3..]).unwrap();
        let collection = document
            .get_child("C")
            .unwrap()
            .get_child("LightscenesC")
            .unwrap();
        assert_eq!(collection.attributes["Num"], "3");
        assert_eq!(collection.attributes["Outputs"], "0020");
        let scenes: Vec<_> = collection
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .filter(|node| node.name == "LightsceneC")
            .collect();
        let new = scenes[2];
        assert_eq!(new.attributes["Outputs"], "18");
        assert_eq!(new.attributes["Q1"], "1610622736");
        assert_eq!(new.attributes["Q2"], "1610612736");
        assert_eq!(new.attributes["Q3"], "-268435456");
        assert_eq!(new.attributes["Q18"], "0");
        assert!(!new.attributes.contains_key("Q19"));
        let original = xmltree::Element::parse(XML.as_bytes()).unwrap();
        let old_collection = original
            .get_child("C")
            .unwrap()
            .get_child("LightscenesC")
            .unwrap();
        let old_scenes: Vec<_> = old_collection
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .filter(|node| node.name == "LightsceneC")
            .collect();
        assert_eq!(&scenes[..2], &old_scenes);
        if save_as {
            assert_eq!(fs::read(&input).unwrap(), before);
        }
    }
}

#[test]
fn lighting_native_append_cli_unsupported_type_never_writes_either_file() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = fixture_bytes(&XML.replace("Q2=\"1610612778\"", "Q2=\"opaque\""));
    fs::write(&input, &before).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    let result = cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "add-mood"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--name",
            "New",
            "--color",
            "42",
            "--save-as",
        ])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["error"], "unsupported_mood_output_value");
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
}

fn creation_command(input: &std::path::Path) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .env("LOX_CONFIG", input.with_extension("absent.yaml"))
        .args(["config", "add-mood"])
        .arg(input)
        .args([
            "uuid:lighting",
            "--name",
            "New",
            "--color",
            "hsv(120,100,10)",
        ]);
    command
}

#[test]
fn lighting_creation_cli_dry_run_plans_ids_defaults_and_count_without_touching_destinations() {
    let dir = tempfile::Builder::new()
        .prefix("creation-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    let xml = XML.replace(
        "<ControlList>",
        "<ControlList><C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>",
    );
    let before = fixture_bytes(&xml);
    fs::write(&input, &before).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for (overrides, sid, cid) in [
        (vec![], "1", "12"),
        (vec!["--sid", "20"], "20", "11"),
        (vec!["--cid", "99"], "4", "99"),
        (vec!["--sid", "776", "--cid", "9"], "776", "9"),
    ] {
        for destination in [None, Some(&input), Some(&output), Some(&missing)] {
            let mut command = creation_command(&input);
            command.args(&overrides).args(["--dry-run", "--json"]);
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
            let preview: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(preview["dry_run"], true);
            assert_eq!(preview["scope"], "stored_mood_creation");
            assert_eq!(preview["container_created"], false);
            assert_eq!(preview["before_num"], "99");
            assert_eq!(preview["after_num"], "3");
            let scene = &preview["proposed_mood"];
            assert_eq!(scene["Name"], "New");
            assert!(!scene["UUID"].as_str().unwrap().is_empty());
            assert_eq!(scene["SID"], sid);
            assert_eq!(scene["CID"], cid);
            assert_eq!(scene["Outputs"], "18");
            assert_eq!(scene["Q1"], "1610622736");
            assert_eq!(scene["Q2"], "1610612736");
            assert_eq!(scene["Q3"], "-268435456");
            assert_eq!(scene["Q18"], "0");
            assert!(scene.get("Q19").is_none());
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing destination");
            assert!(!missing.parent().unwrap().exists());
        }
    }

    // Normal execution still reports its own generated identity, overwrites and stamps.
    let result = creation_command(&input)
        .args(["--json", "--save-as"])
        .arg(&output)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(
        report,
        serde_json::json!({"ok": true, "mood": "New", "uuid": report["uuid"],
            "sid": 1, "cid": 12, "value": 1610622736})
    );
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(
        !std::str::from_utf8(&saved[3..])
            .unwrap()
            .replace("\r\n", "")
            .contains('\n')
    );
    let saved = xmltree::Element::parse(&saved[3..]).unwrap();
    let document = saved
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .find(|node| node.attributes.get("Type").map(String::as_str) == Some("Document"))
        .unwrap();
    assert_ne!(document.attributes["DateS"], "504921600");
    let controller = saved
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .find(|node| node.attributes.get("U").map(String::as_str) == Some("lighting"))
        .unwrap();
    let scene = controller
        .get_child("LightscenesC")
        .unwrap()
        .children
        .last()
        .unwrap()
        .as_element()
        .unwrap();
    assert_eq!(scene.attributes["UUID"], report["uuid"].as_str().unwrap());
    assert_eq!(fs::read(&input).unwrap(), before);
}

#[test]
fn lighting_creation_cli_dry_run_preserves_presence_escaping_and_output_flags() {
    let dir = tempfile::Builder::new()
        .prefix("creation-formats-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    for (attributes, present) in [("", false), ("Title=\"\"", true)] {
        let num = if present { "Num=\"\"" } else { "" };
        let before = fixture_bytes(&format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\" {attributes}><LightscenesC {num}/></C></ControlList>\n"
        ));
        fs::write(&input, &before).unwrap();
        for format in ["json", "csv", "table"] {
            for quiet in [false, true] {
                for no_header in [false, true] {
                    let mut command = cargo_bin_cmd!("lox");
                    command
                        .env("LOX_TELEMETRY", "0")
                        .env("LOX_CONFIG", input.with_extension("absent.yaml"))
                        .args(["config", "add-mood"])
                        .arg(&input)
                        .args([
                            "uuid:lighting",
                            "--name",
                            "µ,\"B",
                            "--color",
                            "0",
                            "--dry-run",
                            "--trace-id",
                            "creation-formats",
                            "-o",
                            format,
                        ]);
                    if quiet {
                        command.arg("--quiet");
                    }
                    if no_header {
                        command.arg("--no-header");
                    }
                    let text = String::from_utf8(
                        command
                            .assert()
                            .success()
                            .stderr("")
                            .get_output()
                            .stdout
                            .clone(),
                    )
                    .unwrap();
                    match format {
                        "json" => {
                            let result: serde_json::Value = serde_json::from_str(&text).unwrap();
                            assert_eq!(result["trace_id"], "creation-formats");
                            assert_eq!(result["proposed_mood"]["Name"], "µ,\"B");
                            assert_eq!(result["proposed_mood"]["Q18"], "0");
                            assert_eq!(result["controller_title"].is_null(), !present);
                            assert_eq!(result["before_num"].is_null(), !present);
                        }
                        "csv" => {
                            assert_eq!(text.starts_with("CONTROLLER_UUID_PRESENT,"), !no_header);
                            assert!(text.contains("\"µ,\"\"B\""));
                            assert!(text.contains(&format!("true,\"lighting\",{present},\"\",true,\"0\",{present},\"\",true,\"1\"")));
                            assert!(text.ends_with(",false,stored_mood_creation\n"));
                            assert!(
                                !text.contains("dry-run wrote")
                                    && !text.contains("creation-formats")
                            );
                            if !no_header {
                                assert!(text.contains("PROPOSED_UUID_PRESENT,PROPOSED_UUID"));
                                assert!(text.contains("PROPOSED_Q18_PRESENT,PROPOSED_Q18"));
                            }
                        }
                        _ => {
                            assert_eq!(text.starts_with("CREATION PREVIEW"), !no_header);
                            assert!(text.contains(&format!(
                                "BEFORE_NUM  {}",
                                if present { "\"\"" } else { "<absent>" }
                            )));
                            assert!(text.contains("PROPOSED_Name  \"µ,\\\"B\""));
                            assert!(text.contains("PROPOSED_Q18  \"0\""));
                            assert!(text.contains("CONTAINER_CREATED  false"));
                            assert_eq!(text.contains("dry-run wrote no files"), !quiet);
                            assert!(!text.contains("creation-formats"));
                        }
                    }
                    assert_eq!(fs::read(&input).unwrap(), before);
                }
            }
        }
    }
}

#[test]
fn lighting_creation_cli_dry_run_and_writer_keep_read_load_parser_error_order() {
    let dir = tempfile::Builder::new()
        .prefix("creation-errors-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    for (xml, message) in [
        (None, "Cannot read"),
        (Some("<ControlList><C".to_owned()), "XML"),
        (Some(XML.to_owned()), "not hsv"),
    ] {
        if let Some(xml) = xml {
            fs::write(&input, fixture_bytes(&xml)).unwrap();
        }
        let mut errors = Vec::new();
        for dry_run in [false, true] {
            let mut command = cargo_bin_cmd!("lox");
            command
                .env("LOX_TELEMETRY", "0")
                .env("LOX_CONFIG", input.with_extension("absent.yaml"))
                .args(["config", "add-mood"])
                .arg(&input)
                .args(["missing", "--name", "First", "--color", "bad", "--json"]);
            if dry_run {
                command.arg("--dry-run");
            }
            let result = command.assert().failure().get_output().stdout.clone();
            let error: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert!(
                error["message"].as_str().unwrap().contains(message),
                "{error}"
            );
            errors.push(error);
        }
        assert_eq!(errors[0], errors[1]);
    }
}
