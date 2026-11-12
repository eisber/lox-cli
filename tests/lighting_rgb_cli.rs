use assert_cmd::Command;
use std::fs;

const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room" Title="Kitchen"/>
  <C Type="And" U="other" Title="Lamp"/>
  <C Type="LightController2" U="lighting" Title="Lamp" CapAI1="7" Future="keep">
    <IoData Pr="room"/>
    <LightscenesC Num="2" Outputs="18">
      <LightsceneC UUID="first" Name="First" SID="2" CID="9" Q1="1879048233" Q2="37"><SeqConf Future="keep"/></LightsceneC>
      <LightsceneC UUID="second" Name="Second" SID="+0003" CID="10" Outputs="18" Q1="opaque" Q2="1610622736" Q10="-1" MixMod="true" SkipNext="true" Future="retain"/>
    </LightscenesC>
    <LSConfig OpM="opaque" ScMv="future"/><HCL T="2" FROM="0"/><Unknown Future="retain"/>
  </C>
</ControlList>
"#;

fn lox() -> Command {
    let mut command = Command::cargo_bin("lox").unwrap();
    command.env("LOX_TELEMETRY", "0");
    command
}

fn bytes(xml: &str) -> Vec<u8> {
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    bytes
}

#[test]
fn lighting_rgb_cli_selected_only_save_as_json_csv_and_noop() {
    let dir = tempfile::Builder::new()
        .prefix("lighting-rgb-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = bytes(XML);
    fs::write(&input, &original).unwrap();
    let report = lox()
        .args(["config", "lighting", "set-mood-rgb"])
        .arg(&input)
        .args([
            "Lamp",
            "--room",
            "Kitchen",
            "--mood",
            "Second",
            "--output-index",
            "2",
            "--rgb",
            "1,127,254",
            "--json",
            "--quiet",
            "--trace-id",
            "rgb",
            "--save-as",
        ])
        .arg(&output)
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(report["scope"], "stored_tag6_rgb");
    assert_eq!(report["selected_mood"]["sid"], "+0003");
    assert_eq!(report["selected_mood"]["uuid"], "second");
    assert_eq!(report["percent"], serde_json::json!([0, 50, 100]));
    assert_eq!(report["after"], "1710662736");
    assert_eq!(report["trace_id"], "rgb");
    assert_eq!(fs::read(&input).unwrap(), original);
    let written = fs::read(&output).unwrap();
    assert!(written.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(
        !String::from_utf8_lossy(&written)
            .replace("\r\n", "")
            .contains('\n')
    );
    let expected = XML.replace("Q2=\"1610622736\"", "Q2=\"1710662736\"");
    assert_eq!(
        xmltree::Element::parse(&written[3..]).unwrap(),
        xmltree::Element::parse(expected.as_bytes()).unwrap()
    );
    let unchanged = bytes(&XML.replace("Q2=\"1610622736\"", "Q2=\"+01710662736\""));
    fs::write(&output, &unchanged).unwrap();
    lox()
        .args(["config", "lighting", "set-mood-rgb"])
        .arg(&output)
        .args([
            "uuid:lighting",
            "--mood",
            "+0003",
            "--output-index",
            "2",
            "--rgb",
            "1,127,254",
            "--output",
            "csv",
            "--no-header",
        ])
        .assert()
        .success()
        .stdout("\"+0003\",Q2,\"+01710662736\",\"+01710662736\",1,127,254,false\n");
    assert_eq!(fs::read(&output).unwrap(), unchanged);
}

#[test]
fn lighting_rgb_cli_zero_guard_does_not_change_raw_zero_contract() {
    let dir = tempfile::Builder::new()
        .prefix("lighting-rgb-raw-zero-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML
        .replace("Num=\"2\"", "Num=\"opaque\"")
        .replace("CapAI1=\"7\"", "CapAI1=\"opaque\"")
        .replace("Outputs=\"18\"", "Outputs=\"opaque\"")
        .replace("Q2=\"1610622736\"", "Q2=\"opaque\"");
    let original = bytes(&xml);
    fs::write(&input, &original).unwrap();
    for color in ["0", "rgb(0,0,0)", "rgb(1,1,1)", "1610612736"] {
        let report = lox()
            .args(["config", "set-mood-color"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--mood",
                "Second",
                "--output-index",
                "2",
                "--color",
                color,
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
        let report: serde_json::Value = serde_json::from_slice(&report).unwrap();
        assert_eq!(report["value"], 1_610_612_736_i64);
        assert_eq!(fs::read(&input).unwrap(), original);
        let expected = xml.replace("Q2=\"opaque\"", "Q2=\"1610612736\"");
        let written = fs::read(&output).unwrap();
        assert_eq!(
            xmltree::Element::parse(&written[3..]).unwrap(),
            xmltree::Element::parse(expected.as_bytes()).unwrap()
        );
    }
}

#[test]
fn lighting_rgb_cli_rejects_bad_inputs_branches_and_dry_run_without_writes() {
    let dir = tempfile::Builder::new()
        .prefix("lighting-rgb-reject-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let config = dir.path().join("absent.yaml");
    for (xml, flags) in [
        (XML.to_owned(), vec!["--rgb", "256,0,0"]),
        (XML.to_owned(), vec!["--rgb=1,-1,255"]),
        (XML.to_owned(), vec!["--rgb", "1.5,0,255"]),
        (XML.to_owned(), vec!["--rgb", "0,0,0"]),
        (XML.to_owned(), vec!["--rgb", "255,0"]),
        (XML.to_owned(), vec!["--rgb", "255,0,0,0"]),
        (
            XML.replace("Outputs=\"18\"", "Outputs=\"1\""),
            vec!["--rgb", "255,0,0"],
        ),
        (XML.to_owned(), vec!["--rgb", "255,0,0", "--dry-run"]),
        (
            XML.replace("CapAI1=\"7\"", "CapAI1=\"0\""),
            vec!["--rgb", "255,0,0"],
        ),
        (
            XML.replace("CapAI1=\"7\"", "CapAI1=\"64\""),
            vec!["--rgb", "255,0,0"],
        ),
        (
            XML.replace("Q2=\"1610622736\"", "Q2=\"1810615436\""),
            vec!["--rgb", "255,0,0"],
        ),
        (
            XML.replace("Q2=\"1610622736\"", "Q2=\"1610612736\""),
            vec!["--rgb", "255,0,0"],
        ),
        (
            XML.replace("SID=\"2\"", "SID=\"03\""),
            vec!["--rgb", "255,0,0"],
        ),
    ] {
        let original = bytes(&xml);
        fs::write(&input, &original).unwrap();
        fs::write(&output, b"existing output").unwrap();
        lox()
            .env("LOX_CONFIG", &config)
            .args(["config", "lighting", "set-mood-rgb"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "Second", "--output-index", "2"])
            .args(&flags)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
        assert!(!config.exists());
    }
}
