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
