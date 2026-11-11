use assert_cmd::Command;
use std::fs;

const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
	<C Type="LightController2" U="lighting" Title="Lamp">
		<LightscenesC Num="2" Outputs="18" Future="keep">
			<LightsceneC Name="A" UUID="a" SID="1" CID="12" Q1="future"/>
			<LightsceneC Name="B" UUID="b" SID="2" CID="13" Q2="-1"/>
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
fn lighting_sid_cli_high_free_cid_clear_is_explicit_atomic_and_reported() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML.replace(
        "OpM=\"opaque\" ScMv=\"future\"",
        "OpM=\"0,1\" ScMv=\"1,2,opaque\"",
    );
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    fs::write(&output, b"existing output").unwrap();
    command()
        .args(["config", "lighting", "set-mood-sid"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--sid", "32", "--save-as"])
        .arg(&output)
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(fs::read(&output).unwrap(), b"existing output");
    let result = command()
        .args(["config", "lighting", "set-mood-sid"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--sid",
            "32",
            "--allow-cid-clear",
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
    assert_eq!(result["cid_cleared"], true);
    assert_eq!(result["identities"][0]["after_cid"], "0");
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(
        fs::read(&output).unwrap(),
        bytes(
            &xml.replace("SID=\"1\" CID=\"12\"", "SID=\"32\" CID=\"0\"")
                .replace("ScMv=\"1,2,opaque\"", "ScMv=\"32,2,opaque\"")
        )
    );
}

#[test]
fn lighting_sid_cli_exchange_updates_ids_and_selection_only() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML.replace(
        "OpM=\"opaque\" ScMv=\"future\"",
        "OpM=\"0,1\" ScMv=\"1,2,opaque\"",
    );
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    let result = command()
        .args(["config", "lighting", "set-mood-sid"])
        .arg(&input)
        .args([
            "uuid:lighting",
            "--mood",
            "A",
            "--sid",
            "2",
            "--swap",
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
    assert_eq!(result["swapped"], true);
    assert_eq!(result["selection_refs"]["after"], "2,1,opaque");
    let expected = xml
        .replace("SID=\"1\" CID=\"12\"", "SID=\"2\" CID=\"12\"")
        .replace("SID=\"2\" CID=\"13\"", "SID=\"1\" CID=\"13\"")
        .replace("ScMv=\"1,2,opaque\"", "ScMv=\"2,1,opaque\"");
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
    let result = command()
        .args(["config", "lighting", "set-mood-sid"])
        .arg(&output)
        .args(["uuid:lighting", "--mood", "A", "--sid", "2", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result).unwrap()["changed"],
        false
    );
    assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
    command().args(["config", "lighting", "set-mood-sid"]).arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--sid", "3", "-o", "csv", "--no-header"])
        .assert().success()
        .stdout("mood,\"a\",\"1\",\"3\",\"12\",\"12\",,\nselection,,,,,,\"1,2,opaque\",\"3,2,opaque\"\n");
}

#[test]
fn lighting_sid_cli_unconfirmed_and_unsupported_edits_preserve_both_destinations() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (sid, flags, error) in [
        ("2", vec![], "mood_sid_occupied"),
        ("32", vec![], "mood_sid_cid_clear_required"),
        ("2147483648", vec![], "unsupported_mood_sid_scope"),
        ("2", vec!["--swap"], "unsupported_lighting_selection_value"),
    ] {
        let before = bytes(XML);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let result = command()
            .args(["config", "lighting", "set-mood-sid"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", "--sid", sid])
            .args(flags)
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&result).unwrap()["error"],
            error
        );
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
    }
}

#[test]
fn lighting_cid_cli_swaps_existing_binding_and_preserves_every_other_byte() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = bytes(XML);
    fs::write(&input, &before).unwrap();
    let result = command()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["config", "lighting", "set-mood-cid"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--cid", "13", "--save-as"])
        .arg(&output)
        .args(["-o", "json", "--quiet"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["identities"].as_array().unwrap().len(), 2);
    assert_eq!(result["identities"][0]["after_cid"], "13");
    assert_eq!(result["identities"][1]["after_cid"], "12");
    assert_eq!(result["selection_remaps"], serde_json::json!([]));
    let expected = XML
        .replace("SID=\"1\" CID=\"12\"", "SID=\"1\" CID=\"13\"")
        .replace("SID=\"2\" CID=\"13\"", "SID=\"2\" CID=\"12\"");
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
}

#[test]
fn lighting_cid_cli_noop_and_unsupported_transaction_preserve_both_destinations() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = bytes(&XML.replace("CID=\"12\"", "CID=\"+00012\""));
    fs::write(&input, &before).unwrap();
    fs::write(&output, b"existing output").unwrap();
    let result = command()
        .args(["config", "lighting", "set-mood-cid"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--cid", "12", "-o", "json"])
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
    for save_as in [false, true] {
        let mut cmd = command();
        cmd.args(["config", "lighting", "set-mood-cid"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", "--cid", "22", "-o", "json"]);
        if save_as {
            cmd.arg("--save-as").arg(&output);
        }
        let result = cmd.assert().failure().get_output().stdout.clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&result).unwrap()["error"],
            "unsupported_lighting_selection_value"
        );
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
    }
}
