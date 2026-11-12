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
fn lighting_sid_cli_dry_run_matches_exchange_and_confirmed_clear_without_writing() {
    let dir = tempfile::Builder::new()
        .prefix("sid-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    let xml = XML
        .replace("SID=\"1\" CID=\"12\"", "SID=\"+0001\" CID=\"+012\"")
        .replace(
            "OpM=\"opaque\" ScMv=\"future\"",
            "OpM=\"0,1,2\" ScMv=\" +01 ,2,99,opaque\"",
        );
    for (sid, flags, expected) in [
        (
            "2",
            vec!["--swap"],
            xml.replace("SID=\"+0001\" CID=\"+012\"", "SID=\"2\" CID=\"+012\"")
                .replace("SID=\"2\" CID=\"13\"", "SID=\"1\" CID=\"13\"")
                .replace("ScMv=\" +01 ,2,99,opaque\"", "ScMv=\" 2 ,1,1,opaque\""),
        ),
        (
            "32",
            vec!["--allow-cid-clear"],
            xml.replace("SID=\"+0001\" CID=\"+012\"", "SID=\"32\" CID=\"0\"")
                .replace("ScMv=\" +01 ,2,99,opaque\"", "ScMv=\" 32 ,2,99,opaque\""),
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut preview = serde_json::Value::Null;
        for destination in [&input, &output, &missing] {
            let result = command()
                .env("LOX_CONFIG", dir.path().join("absent.yaml"))
                .args(["--dry-run", "config", "lighting", "set-mood-sid"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", "--sid", sid])
                .args(&flags)
                .arg("--save-as")
                .arg(destination)
                .args(["-o", "json", "--quiet", "--trace-id", "sid-preview"])
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();
            preview = serde_json::from_slice(&result).unwrap();
            assert_eq!(preview["dry_run"], true);
            assert_eq!(preview["trace_id"], "sid-preview");
            assert_eq!(preview["current_sid"], 1);
            assert_eq!(preview["selected"]["before_sid"], "+0001");
            assert_eq!(preview["selected"]["after_sid"], sid);
            assert_eq!(preview["selected"]["before_cid"], "+012");
            assert_eq!(preview["swapped"], sid == "2");
            assert_eq!(preview["cid_cleared"], sid == "32");
            if sid == "2" {
                assert_eq!(
                    preview["selection_remaps"],
                    serde_json::json!([
                        {"before": 2, "after": 99},
                        {"before": 1, "after": 2},
                        {"before": 99, "after": 1}
                    ])
                );
                assert_eq!(preview["selection_refs"]["after"], " 2 ,1,1,opaque");
                assert_eq!(preview["selected"]["after_cid"], "+012");
            } else {
                assert_eq!(preview["selected"]["after_cid"], "0");
            }
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
            assert!(!missing.parent().unwrap().exists());
            assert!(!dir.path().join("absent.yaml").exists());
        }
        let actual = command()
            .args(["config", "lighting", "set-mood-sid"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", "--sid", sid])
            .args(&flags)
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json", "--quiet"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let actual: serde_json::Value = serde_json::from_slice(&actual).unwrap();
        for field in ["dry_run", "trace_id", "current_sid", "selected"] {
            preview.as_object_mut().unwrap().remove(field);
        }
        assert_eq!(preview, actual);
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), bytes(&expected));
    }
}

#[test]
fn lighting_sid_cli_dry_run_noop_keeps_raw_spelling_and_omission_visible_in_all_formats() {
    let dir = tempfile::Builder::new()
        .prefix("sid-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    for cid in ["", " CID=\"+000\"", " CID=\"opaque\""] {
        let xml = XML.replace("SID=\"1\" CID=\"12\"", &format!("SID=\"+0032\"{cid}"));
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
                "32",
                "--dry-run",
                "--allow-cid-clear",
                "--save-as",
            ])
            .arg(&missing)
            .args(["-o", "json", "--quiet"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["changed"], false);
        assert_eq!(result["cid_cleared"], false);
        assert_eq!(result["identities"], serde_json::json!([]));
        assert_eq!(result["selection_refs"], serde_json::Value::Null);
        assert_eq!(result["current_sid"], 32);
        assert_eq!(result["requested_sid"], 32);
        assert_eq!(result["selected"]["before_sid"], "+0032");
        assert_eq!(result["selected"]["after_sid"], "+0032");
        let stored_cid = match cid {
            "" => serde_json::Value::Null,
            " CID=\"+000\"" => serde_json::json!("+000"),
            _ => serde_json::json!("opaque"),
        };
        assert_eq!(result["selected"]["before_cid"], stored_cid);
        assert_eq!(result["selected"]["after_cid"], stored_cid);
        assert_eq!(fs::read(&input).unwrap(), before);
        assert!(!missing.parent().unwrap().exists());
    }
    let before = bytes(&XML.replace("SID=\"1\" CID=\"12\"", "SID=\"+0032\""));
    fs::write(&input, &before).unwrap();
    for (format, no_header, quiet, expected) in [
        (
            "csv",
            true,
            true,
            "\"selected\",1,\"a\",\"SID\",true,\"+0032\",true,\"+0032\"\n\"selected\",1,\"a\",\"CID\",false,\"\",false,\"\"\n\"selected_effective\",1,\"a\",\"SID\",true,\"32\",true,\"32\"\n",
        ),
        (
            "table",
            true,
            true,
            "selected  1  a  SID  +0032→+0032\nselected  1  a  CID  <absent>→<absent>\nselected_effective  1  a  SID  32→32\n",
        ),
        (
            "table",
            false,
            false,
            "RECORD  INDEX  UUID  FIELD  BEFORE→AFTER\nselected  1  a  SID  +0032→+0032\nselected  1  a  CID  <absent>→<absent>\nselected_effective  1  a  SID  32→32\n\n0 identity edits; selected effective SID 32→32; changed=false; dry-run wrote no files\n",
        ),
    ] {
        let mut cmd = command();
        cmd.args(["--dry-run", "config", "lighting", "set-mood-sid"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", "--sid", "32", "-o", format]);
        if no_header {
            cmd.arg("--no-header");
        }
        if quiet {
            cmd.arg("--quiet");
        }
        cmd.assert().success().stdout(expected);
    }
    assert_eq!(fs::read(&input).unwrap(), before);
    assert!(!missing.parent().unwrap().exists());
}

#[test]
fn lighting_sid_cli_dry_run_and_writer_reject_identical_unsupported_transactions() {
    let dir = tempfile::Builder::new()
        .prefix("sid-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (xml, sid, flags, error) in [
        (XML.to_string(), "32", vec![], "mood_sid_cid_clear_required"),
        (
            XML.to_string(),
            "2",
            vec!["--allow-cid-clear"],
            "mood_sid_occupied",
        ),
        (XML.to_string(), "0", vec![], "unsupported_mood_sid_scope"),
        (XML.to_string(), "91", vec![], "unsupported_mood_sid_scope"),
        (XML.to_string(), "777", vec![], "unsupported_mood_sid_scope"),
        (
            XML.to_string(),
            "2147483648",
            vec![],
            "unsupported_mood_sid_scope",
        ),
        (
            XML.replace("SID=\"1\"", "SID=\"777\""),
            "3",
            vec![],
            "unsupported_mood_sid_scope",
        ),
        (
            XML.replace("SID=\"2\"", "SID=\"+001\""),
            "3",
            vec![],
            "ambiguous_mood_sid",
        ),
        (
            XML.replace("SID=\"2\"", "SID=\"opaque\""),
            "3",
            vec![],
            "unsupported_mood_identity_value",
        ),
        (
            XML.replace("CID=\"12\"", "CID=\"opaque\""),
            "32",
            vec!["--allow-cid-clear"],
            "unsupported_mood_identity_value",
        ),
        (
            XML.replace("Num=\"2\"", "Num=\"3\""),
            "3",
            vec![],
            "unsupported_mood_count",
        ),
        (
            XML.replace("Name=\"B\"", "Name=\"A\""),
            "3",
            vec![],
            "ambiguous_or_missing_mood",
        ),
        (
            XML.replace("Type=\"LightController2\"", "Type=\"And\""),
            "3",
            vec![],
            "wrong_block_type",
        ),
        (
            XML.replace("<HCL", "<LightscenesC/><HCL"),
            "3",
            vec![],
            "unsupported_mood_container_shape",
        ),
        (
            XML.to_string(),
            "2",
            vec!["--swap"],
            "unsupported_lighting_selection_value",
        ),
        (
            XML.replace(
                "OpM=\"opaque\" ScMv=\"future\"",
                "OpM=\"0,1\" ScMv=\"1,opaque\"",
            ),
            "32",
            vec!["--allow-cid-clear"],
            "unsupported_lighting_selection_value",
        ),
        (
            XML.replace("<HCL", "<LSConfig/><HCL"),
            "3",
            vec![],
            "unsupported_lighting_selection_shape",
        ),
        (
            XML.replace("Num=\"2\"", "Num=\"3\"").replace(
                "</LightscenesC>",
                "<LightsceneC SID=\"99\"/></LightscenesC>",
            ),
            "2",
            vec!["--swap"],
            "mood_sid_temporary_collision",
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut errors = Vec::new();
        for dry_run in [true, false] {
            let mut cmd = command();
            cmd.args(["config", "lighting", "set-mood-sid"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", "--sid", sid])
                .args(&flags)
                .arg("--save-as")
                .arg(&output)
                .args(["-o", "json", "--quiet", "--trace-id", "sid-rejected"]);
            if dry_run {
                cmd.arg("--dry-run");
            }
            let result = cmd.assert().failure().get_output().stdout.clone();
            let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(result["error"], error);
            errors.push(result);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
        }
        assert_eq!(errors[0], errors[1]);
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

#[test]
fn lighting_cid_cli_dry_run_reports_the_exact_writer_transaction_without_writing() {
    let dir = tempfile::Builder::new()
        .prefix("cid-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let missing = dir.path().join("missing").join("new.Loxone");
    let xml = XML
        .replace("Num=\"2\"", "Num=\"3\"")
        .replace("SID=\"1\" CID=\"12\"", "SID=\"2\" CID=\"13\"")
        .replace("SID=\"2\" CID=\"13\" Q2", "SID=\"777\" CID=\"22\" Q2")
        .replace(
            "\t\t</LightscenesC>",
            "\t\t\t<LightsceneC SID=\"1\" CID=\"12\"/>\n\t\t</LightscenesC>",
        )
        .replace(
            "OpM=\"opaque\" ScMv=\"future\"",
            "OpM=\"0,1,2\" ScMv=\"2, 777 ,+03,opaque\"",
        );
    let before = bytes(&xml);
    fs::write(&input, &before).unwrap();
    fs::write(&output, b"existing output").unwrap();
    let mut preview = serde_json::Value::Null;
    for destination in [&input, &output, &missing] {
        let result = command()
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["--dry-run", "config", "lighting", "set-mood-cid"])
            .arg(&input)
            .args(["uuid:lighting", "--mood", "A", "--cid", "22", "--save-as"])
            .arg(destination)
            .args(["-o", "json", "--quiet", "--trace-id", "cid-preview"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        preview = serde_json::from_slice(&result).unwrap();
        assert_eq!(preview["dry_run"], true);
        assert_eq!(preview["trace_id"], "cid-preview");
        assert_eq!(preview["current_cid"], 13);
        assert_eq!(preview["requested_cid"], 22);
        assert_eq!(preview["selected"]["before_sid"], "2");
        assert_eq!(preview["selected"]["after_sid"], "777");
        assert_eq!(preview["selection_refs"]["after"], "2, 2 ,2,opaque");
        assert_eq!(fs::read(&input).unwrap(), before);
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
        assert!(!missing.parent().unwrap().exists());
        assert!(!dir.path().join("absent.yaml").exists());
    }
    let actual = command()
        .args(["config", "lighting", "set-mood-cid"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--cid", "22", "--save-as"])
        .arg(&output)
        .args(["-o", "json", "--quiet"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let actual: serde_json::Value = serde_json::from_slice(&actual).unwrap();
    for field in ["dry_run", "trace_id", "current_cid", "selected"] {
        preview.as_object_mut().unwrap().remove(field);
    }
    assert_eq!(preview, actual);
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(
        fs::read(&output).unwrap(),
        bytes(
            &xml.replace("SID=\"2\" CID=\"13\"", "SID=\"777\" CID=\"22\"")
                .replace("SID=\"777\" CID=\"22\" Q2", "SID=\"2\" CID=\"13\" Q2")
                .replace("ScMv=\"2, 777 ,+03,opaque\"", "ScMv=\"2, 2 ,2,opaque\"")
        )
    );
}

#[test]
fn lighting_cid_cli_dry_run_noop_keeps_omission_spelling_and_formats_visible() {
    let dir = tempfile::Builder::new()
        .prefix("cid-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("new.Loxone");
    for (xml, cid, stored, effective) in [
        (XML.replace(" CID=\"12\"", ""), "0", None, 0),
        (
            XML.replace("CID=\"12\"", "CID=\"+00012\""),
            "12",
            Some("+00012"),
            12,
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        let result = command()
            .args(["config", "lighting", "set-mood-cid"])
            .arg(&input)
            .args([
                "uuid:lighting",
                "--mood",
                "A",
                "--cid",
                cid,
                "--dry-run",
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
        assert_eq!(result["changed"], false);
        assert_eq!(result["identities"], serde_json::json!([]));
        assert_eq!(result["current_cid"], effective);
        assert_eq!(result["selected"]["before_cid"], serde_json::json!(stored));
        assert_eq!(result["selected"]["after_cid"], serde_json::json!(stored));
        assert_eq!(fs::read(&input).unwrap(), before);
        assert!(!output.exists());
    }
    fs::write(&input, bytes(&XML.replace(" CID=\"12\"", ""))).unwrap();
    let before = fs::read(&input).unwrap();
    command().args(["--dry-run", "config", "lighting", "set-mood-cid"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--cid", "0", "-o", "csv", "--no-header", "--quiet"])
        .assert().success()
        .stdout("\"selected\",1,\"a\",\"SID\",true,\"1\",true,\"1\"\n\"selected\",1,\"a\",\"CID\",false,\"\",false,\"\"\n\"selected_effective\",1,\"a\",\"CID\",true,\"0\",true,\"0\"\n");
    command().args(["--dry-run", "config", "lighting", "set-mood-cid"])
        .arg(&input)
        .args(["uuid:lighting", "--mood", "A", "--cid", "0", "--no-header", "--quiet"])
        .assert().success()
        .stdout("selected  1  a  SID  1→1\nselected  1  a  CID  <absent>→<absent>\nselected_effective  1  a  CID  0→0\n");
    assert_eq!(fs::read(&input).unwrap(), before);
    assert!(!output.exists());
}

#[test]
fn lighting_cid_cli_dry_run_and_writer_reject_the_same_unsupported_transactions() {
    let dir = tempfile::Builder::new()
        .prefix("cid-preview-")
        .tempdir_in(".")
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (xml, cid, error) in [
        (
            XML.to_string(),
            "22",
            "unsupported_lighting_selection_value",
        ),
        (
            XML.replace("ScMv=\"future\"", "ScMv=\"1,opaque\"")
                .replace("OpM=\"opaque\"", "OpM=\"0,1\""),
            "22",
            "unsupported_lighting_selection_value",
        ),
        (
            XML.replace("Name=\"B\"", "Name=\"A\""),
            "13",
            "ambiguous_or_missing_mood",
        ),
        (
            XML.replace("SID=\"2\"", "SID=\"+001\""),
            "13",
            "ambiguous_mood_sid",
        ),
        (
            XML.replace("CID=\"13\"", "CID=\"12\""),
            "13",
            "ambiguous_mood_cid",
        ),
        (
            XML.replace("CID=\"13\"", "CID=\"opaque\""),
            "13",
            "unsupported_mood_identity_value",
        ),
        (
            XML.replace("Num=\"2\"", "Num=\"3\""),
            "13",
            "unsupported_mood_count",
        ),
        (XML.to_string(), "2147483648", "mood_cid_out_of_range"),
        (
            XML.replace("<HCL", "<LSConfig/><HCL"),
            "22",
            "unsupported_lighting_selection_shape",
        ),
    ] {
        let before = bytes(&xml);
        fs::write(&input, &before).unwrap();
        fs::write(&output, b"existing output").unwrap();
        let mut results = Vec::new();
        for dry_run in [true, false] {
            let mut cmd = command();
            if dry_run {
                cmd.arg("--dry-run");
            }
            let result = cmd
                .args(["config", "lighting", "set-mood-cid"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", "--cid", cid, "--save-as"])
                .arg(&output)
                .args(["-o", "json"])
                .assert()
                .failure()
                .get_output()
                .stdout
                .clone();
            let result: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(result["error"], error);
            assert_eq!(result["ok"], false);
            results.push(result);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing output");
        }
        assert_eq!(results[0], results[1]);
    }
}
