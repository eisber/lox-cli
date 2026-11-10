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
