use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{Value, json};
use std::fs;

fn lox() -> Command {
    let mut command = Command::cargo_bin("lox").unwrap();
    command.env("LOX_TELEMETRY", "0");
    command
}

fn color_json(args: &[&str]) -> Value {
    let output = lox()
        .arg("color")
        .args(args)
        .arg("--json")
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn color_encode_rounds_before_scaling_and_keeps_distinct_packings() {
    let rgb = color_json(&["encode", "--rgb", "1.49,127.5,254.5", "--brightness", "50"]);
    assert_eq!(
        rgb,
        json!({"mode":"rgb","rgb":[1,64,128],"hex":"#014080",
            "composite":128064001,"command":"hsv(210,99,50)"})
    );
    let mood = color_json(&[
        "encode",
        "--hsv",
        "360,100,100",
        "--brightness",
        "50",
        "--mood",
    ]);
    assert_eq!(
        mood,
        json!({"mode":"mood","rgb":[128,0,0],"percent":[50,0,0],
            "mood_value":1610612786_i64,"command":"hsv(0,100,50)"})
    );
    for kelvin in ["1000", "15000"] {
        let value = color_json(&["encode", "--kelvin", kelvin, "--brightness=-0"]);
        assert_eq!(value["command"], format!("temp(0,{kelvin})"));
        assert!(value["brightness"].as_f64().unwrap().is_sign_negative());
    }
}

#[test]
fn color_decode_legacy_observations_are_not_encode_validation() {
    for (spec, rgb) in [
        (" HsV(720,100,100) ", json!([255, 0, 0])),
        ("hsv(-120,100,100)", json!([0, 0, 255])),
        ("hsv(NaN,100,100)", json!([255, 0, 0])),
        ("hsv(inf,100,100)", json!([255, 0, 0])),
        ("hsv(0,NaN,100)", json!([0, 0, 0])),
        ("hsv(0,100,200)", json!([255, 0, 0])),
    ] {
        assert_eq!(color_json(&["decode", spec])["rgb"], rgb, "{spec}");
    }
    let temp = color_json(&["decode", " LuMiTeCh(-0,inf) "]);
    assert_eq!(temp["mode"], "temperature");
    assert!(temp["brightness"].as_f64().unwrap().is_sign_negative());
    assert_eq!(temp["kelvin"], Value::Null);
    let mood = color_json(&["decode", "9610622736"]);
    assert_eq!(mood["mood_value"], 9_610_622_736_i64);
    assert_eq!(mood["percent"], json!([0, 10, 0]));
    assert_eq!(mood["command"], "hsv(120,100,10)");
}

#[test]
fn color_exact_human_output_and_existing_root_flags() {
    for flags in [
        vec![],
        vec!["--quiet"],
        vec!["--output", "csv", "--no-header"],
        vec!["--dry-run", "--trace-id", "color"],
    ] {
        lox()
            .args(["color", "encode", "--rgb", "100,40,0"])
            .args(flags)
            .assert()
            .success()
            .stderr("")
            .stdout(
                "RGB color\n  rgb:       100,40,0\n  hex:       #642800\n  composite: 40100   ← wire/set-param this integer (<v.col>)\n  command:   hsv(24,100,39)\n",
            );
    }
}

#[test]
fn color_error_order_and_legacy_messages() {
    for (args, message) in [
        (
            vec!["encode"],
            "specify one of --rgb R,G,B | --hsv H,S,V | --kelvin K",
        ),
        (
            vec!["encode", "--rgb", "bad", "--hsv", "bad"],
            "--rgb, --hsv and --kelvin are mutually exclusive",
        ),
        (
            vec!["encode", "--kelvin", "0", "--mood", "--brightness", "NaN"],
            "--mood applies to RGB/HSV colors, not --kelvin (tunable white)",
        ),
        (
            vec!["encode", "--kelvin", "0", "--brightness", "NaN"],
            "--brightness NaN out of range (0..100)",
        ),
        (
            vec!["encode", "--kelvin", "999"],
            "--kelvin 999 out of range (expected ~2700..6500)",
        ),
        (
            vec!["encode", "--hsv", "720,100,100", "--brightness", "NaN"],
            "--hsv expects H in 0..360, S and V in 0..100",
        ),
        (
            vec!["encode", "--rgb", "NaN,0,0", "--brightness", "NaN"],
            "--rgb channel R=NaN out of range (0..255)",
        ),
        (
            vec!["encode", "--rgb", "0,256,no"],
            "'no' is not a number (R,G,B)",
        ),
        (
            vec!["decode", "hsv(1,2)"],
            "expected three comma-separated values (hsv(H,S,V)), got '1,2'",
        ),
        (
            vec!["decode", "temp(no,0)"],
            "'no' is not a number (temp(brightness,kelvin))",
        ),
        (
            vec!["decode", "999"],
            "composite 999 has red channel 999 outside 0..255 — not a valid RGB value",
        ),
        (
            vec!["decode", "1610812736"],
            "mood value 1610812736 has green channel 200% outside 0..100 — not a valid mood color",
        ),
        (
            vec!["decode", "1000000000"],
            "value 1000000000 is outside the RGB composite range (0..255255255); color-temperature values use the temp(brightness,kelvin) command string, not an integer",
        ),
    ] {
        lox()
            .arg("color")
            .args(args)
            .assert()
            .failure()
            .stdout("")
            .stderr(predicate::str::contains(message));
    }
}

#[test]
fn color_raw_config_mood_parser_remains_raw_and_failure_atomic() {
    let dir = tempfile::Builder::new()
        .prefix(".color-codec-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = br#"<ControlList><C Type="LightController2" U="lc" Title="Lamp" CapAI1="opaque"><LightscenesC Num="opaque" Outputs="opaque"><LightsceneC Name="First" SID="1" Q1="opaque"/></LightscenesC></C></ControlList>"#;
    fs::write(&input, original).unwrap();
    for (spec, expected) in [
        ("hsv(720,100,100)", 1_610_612_836_i64),
        ("rgb(0,26,0)", 1_610_622_736),
        ("1000040100", 1_610_628_775),
        ("9610622736", 9_610_622_736),
        ("hsv(NaN,100,100)", 1_610_612_836),
        ("0", 1_610_612_736),
    ] {
        let report = lox()
            .args(["config", "set-mood-color"])
            .arg(&input)
            .args([
                "uuid:lc",
                "--mood",
                "First",
                "--color",
                spec,
                "--json",
                "--save-as",
            ])
            .arg(&output)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let report: Value = serde_json::from_slice(&report).unwrap();
        assert_eq!(report["value"], expected, "{spec}");
        assert!(
            fs::read_to_string(&output)
                .unwrap()
                .contains(&format!("Q1=\"{expected}\""))
        );
        assert_eq!(fs::read(&input).unwrap(), original);
    }
    for spec in ["rgb(NaN,0,0)", "1610812736", "-1", "temp(10,2700)"] {
        fs::write(&output, b"preserve output").unwrap();
        lox()
            .args(["config", "set-mood-color"])
            .arg(&input)
            .args(["uuid:lc", "--mood", "First", "--color", spec, "--save-as"])
            .arg(&output)
            .assert()
            .failure()
            .stdout("");
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), b"preserve output");
    }
}

#[test]
fn color_add_mood_uses_the_same_legacy_parser_before_creation() {
    let dir = tempfile::Builder::new()
        .prefix(".color-add-mood-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = br#"<ControlList><C Type="LightController2" U="lc" Title="Lamp"><LightscenesC Num="1" Outputs="18"><LightsceneC UUID="first" Name="First" SID="2" CID="3" Q1="1610622736"/></LightscenesC></C></ControlList>"#;
    fs::write(&input, original).unwrap();
    let report = lox()
        .args(["config", "add-mood"])
        .arg(&input)
        .args([
            "uuid:lc",
            "--name",
            "Added",
            "--color",
            "9610622736",
            "--sid",
            "5",
            "--cid",
            "6",
            "--json",
            "--save-as",
        ])
        .arg(&output)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(report["value"], 9_610_622_736_i64);
    assert!(
        fs::read_to_string(&output)
            .unwrap()
            .contains("Q1=\"9610622736\"")
    );
    assert_eq!(fs::read(&input).unwrap(), original);
    fs::write(&output, b"preserve output").unwrap();
    lox()
        .args(["config", "add-mood"])
        .arg(&input)
        .args([
            "uuid:lc",
            "--name",
            "First",
            "--color",
            "rgb(NaN,0,0)",
            "--save-as",
        ])
        .arg(&output)
        .assert()
        .failure()
        .stdout("")
        .stderr(predicate::str::contains(
            "--rgb channel R=NaN out of range (0..255)",
        ));
    assert_eq!(fs::read(&input).unwrap(), original);
    assert_eq!(fs::read(&output).unwrap(), b"preserve output");
}
