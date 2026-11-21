use std::fs;

use predicates::prelude::*;
use serde_json::Value;

fn fixture() -> String {
    let loads: String = (0..12)
        .map(|index| {
            format!(
                "<Load{} nm=\"Consumer {index}\" pwr=\"2.5\" maxOff=\"123\" minOn=\"45\" Unknown=\"keep-{index}\"/>",
                if index == 0 { String::new() } else { format!(" ioIdx=\"{index}\"") }
            )
        })
        .collect();
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"LoadShed\" U=\"manager\" Title=\"Load Manager\" WF=\"16384\"><Co K=\"AQl1\" U=\"connector\"/><IoData Pr=\"room\"/>{loads}<Unknown U=\"keep\"/></C><C Type=\"Place\" U=\"room\" Title=\"Office\"/></C></Loxone>\r\n"
    )
}

#[test]
fn load_manager_inspect_cli_keeps_exact_json_table_and_csv() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = fixture()
        .replacen(
            r#"nm="Consumer 0" pwr="2.5" maxOff="123" minOn="45""#,
            r#"nm=" A &quot;B&quot; " pwr="malformed" minOn="+0045""#,
            1,
        )
        .replacen(r#" nm="Consumer 1""#, "", 1);
    fs::write(&input, &source).unwrap();
    let inspect = || {
        let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
        command
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "inspect"])
            .arg(&input);
        command
    };
    let mut loads = vec![serde_json::json!({
        "priority": 1,
        "io_index": 0,
        "attributes": {"nm": " A \"B\" ", "pwr": "malformed", "minOn": "+0045", "Unknown": "keep-0"}
    })];
    for index in 1..12 {
        let mut row = serde_json::json!({
            "priority": index + 1,
            "io_index": index,
            "attributes": {"ioIdx": index.to_string(), "pwr": "2.5", "maxOff": "123", "minOn": "45", "Unknown": format!("keep-{index}")}
        });
        if index != 1 {
            row["attributes"]["nm"] = Value::from(format!("Consumer {index}"));
        }
        loads.push(row);
    }
    let expected = serde_json::json!({
        "ok": true,
        "saved_to": null,
        "load_manager": {"uuid": "manager", "title": "Load Manager", "loads": loads},
        "trace_id": "inspect-trace"
    });
    inspect()
        .args([
            "Load Manager",
            "-r",
            "Office",
            "-o",
            "json",
            "--trace-id",
            "inspect-trace",
        ])
        .assert()
        .success()
        .stdout(format!(
            "{}\n",
            serde_json::to_string_pretty(&expected).unwrap()
        ));
    inspect()
        .arg("Manager [Off]")
        .assert()
        .success()
        .stdout(concat!(
            "PRIORITY  IO_INDEX  NAME\n",
            "1         0          A \"B\" \n",
            "2         1         \n",
            "3         2         Consumer 2\n",
            "4         3         Consumer 3\n",
            "5         4         Consumer 4\n",
            "6         5         Consumer 5\n",
            "7         6         Consumer 6\n",
            "8         7         Consumer 7\n",
            "9         8         Consumer 8\n",
            "10        9         Consumer 9\n",
            "11        10        Consumer 10\n",
            "12        11        Consumer 11\n",
            "\n12 loads\n"
        ));
    inspect()
        .args(["uuid:manager", "-o", "csv", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(concat!(
            "1,0,\" A \"\"B\"\" \"\n",
            "2,1,\"\"\n",
            "3,2,\"Consumer 2\"\n",
            "4,3,\"Consumer 3\"\n",
            "5,4,\"Consumer 4\"\n",
            "6,5,\"Consumer 5\"\n",
            "7,6,\"Consumer 6\"\n",
            "8,7,\"Consumer 7\"\n",
            "9,8,\"Consumer 8\"\n",
            "10,9,\"Consumer 9\"\n",
            "11,10,\"Consumer 10\"\n",
            "12,11,\"Consumer 11\"\n"
        ));
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
}

#[test]
fn load_manager_inspect_cli_keeps_admission_and_exact_refusals() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let original = fixture();
    let inspect = |args: &[&str], code: &str, message: &str| {
        let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "inspect"])
            .arg(&input)
            .args(["-o", "json"])
            .args(args)
            .assert()
            .code(1)
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"], code);
        assert_eq!(result["message"], message);
    };
    inspect(
        &["--dry-run", "uuid:manager"],
        "dry_run_not_supported",
        "dry-run not supported for this command: no safe non-mutating plan is available. Nothing was executed and inputs were not fully validated. Remove --dry-run only when you intend to execute the command.",
    );
    inspect(
        &["Manager [Office]", "--room", "Office"],
        "error",
        "Use either a bracket room qualifier or --room, not both.",
    );

    fs::write(&input, &original).unwrap();
    inspect(
        &["Load Managr"],
        "error",
        "Element 'Load Managr' not found. Did you mean: Load Manager?\n  Available: Load Manager, Office",
    );
    for (xml, selector, message) in [
        (
            original.replace(
                r#"U="room" Title="Office"/>"#,
                r#"U="room" Title="Office"/><C Type="Switch" U="other" Title="Load Manager"/>"#,
            ),
            "Load Manager",
            "Selector 'Load Manager' matches 2 elements (expected 1):\n  LoadShed 'Load Manager' (uuid:manager)\n  Switch 'Load Manager' (uuid:other)\nUse uuid:<uuid> for exact match.",
        ),
        (
            original.replace(r#"Type="LoadShed""#, r#"Type="loadshed""#),
            "uuid:manager",
            "'uuid:manager' is not a LoadShed (Load Manager). Use `lox config controls FILE --type LoadShed` to find one.",
        ),
        (
            original.replacen("<Load nm", "<Other nm", 1),
            "uuid:manager",
            "Unsupported Load Manager shape: expected exactly 12 direct Load records, found 11. Sparse/extended lists are not supported; no defaults were fabricated.",
        ),
        (
            original.replacen(r#"ioIdx="1""#, r#"ioIdx="00""#, 1),
            "uuid:manager",
            "Unsupported Load Manager shape: duplicate ioIdx 0; identities must be unique.",
        ),
        (
            original.replacen(r#"ioIdx="1""#, r#"ioIdx="+1""#, 1),
            "uuid:manager",
            "Unsupported Load Manager shape: ioIdx must be absent (native zero) or decimal digits in 0..11. Explicit empty, signed, whitespace and nonnumeric identities are not supported.",
        ),
    ] {
        fs::write(&input, &xml).unwrap();
        inspect(&[selector], "error", message);
        assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
    }
}

#[test]
fn load_manager_move_cli_preserves_save_as_report_and_canonical_empty_nodes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture()
        .replace(r#"DateS="1""#, r#"DateS="2147483647""#)
        .replacen(
            r#"Unknown="keep-0"/><Load ioIdx="1""#,
            r#"Unknown="keep-0"><Empty/></Load><Between/><Load ioIdx="1""#,
            1,
        );
    fs::write(&input, &source).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["Load Manager", "0", "down", "-r", "Office", "--save-as"])
        .arg(&output)
        .args(["-o", "json", "--trace-id", "move-trace"])
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(result["trace_id"], "move-trace");
    assert_eq!(result["saved_to"], output.to_str().unwrap());
    assert!(result.get("changed").is_none());
    let loads = result["load_manager"]["loads"].as_array().unwrap();
    assert_eq!(loads[0]["priority"], 1);
    assert_eq!(loads[0]["io_index"], 1);
    assert_eq!(
        loads[1],
        serde_json::json!({
            "priority": 2, "io_index": 0,
            "attributes": {
                "nm": "Consumer 0", "pwr": "2.5", "maxOff": "123", "minOn": "45",
                "Unknown": "keep-0"
            }
        })
    );
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    let saved = fs::read_to_string(&output).unwrap();
    assert!(saved.starts_with("\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n"));
    assert!(saved.contains(r#"DateS="2147483648""#));
    assert!(saved.contains(r#"<Co K="AQl1" U="connector"/>"#));
    let first = saved.find(r#"Unknown="keep-1"/>"#).unwrap();
    let between = saved.find("<Between></Between>").unwrap();
    let zero = saved.find(r#"Unknown="keep-0">"#).unwrap();
    let empty = saved.find("<Empty></Empty>").unwrap();
    assert!(first < between && between < zero && zero < empty);
}

#[test]
fn load_manager_move_cli_refusals_never_touch_input_or_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    fs::write(&output, "existing output").unwrap();
    let run = |direction: &str, extra: &[&str]| {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "move"])
            .arg(&input)
            .args(["uuid:manager", "0", direction, "--save-as"])
            .arg(&output)
            .args(["-o", "json"])
            .args(extra)
            .assert()
            .code(1)
            .get_output()
            .stdout
            .clone()
    };
    let dry_run: Value = serde_json::from_slice(&run("down", &["--dry-run"])).unwrap();
    assert_eq!(dry_run["error"], "dry_run_not_supported");
    assert!(!input.exists());
    let source = fixture();
    for (xml, direction, message) in [
        (
            source.clone(),
            "up",
            "Load io-index 0 is already at the first priority boundary; nothing was changed.",
        ),
        (
            source.replace(r#"ioIdx="1""#, r#"ioIdx="00""#),
            "down",
            "Unsupported Load Manager shape: duplicate ioIdx 0; identities must be unique.",
        ),
    ] {
        fs::write(&input, &xml).unwrap();
        let failed: Value = serde_json::from_slice(&run(direction, &[])).unwrap();
        assert_eq!(failed["ok"], false);
        assert_eq!(failed["message"], message);
        assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing output");
        assert!(!output.with_extension("Loxone.tmp").exists());
    }
}

#[test]
fn load_manager_move_cli_keeps_legacy_scope_and_late_report() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let run = |selector: &str| {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("lox");
        cmd.env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "move"])
            .arg(&input)
            .args([selector, "0", "down", "--save-as"])
            .arg(&output)
            .args(["-o", "json"]);
        cmd
    };
    let xml = fixture();
    let start = xml.find(r#"<C Type="LoadShed""#).unwrap();
    let end = xml.find(r#"<C Type="Place""#).unwrap();
    let bare = &xml[start..end];
    for source in [
        xml.replace("<Loxone>", r#"<Loxone xml:lang="en">"#),
        xml.replace(r#"DateS="1">"#, r#"DateS="1"><Template IfT="true"/>"#),
        bare.to_owned(),
        format!("<Outer><Inner>{bare}</Inner></Outer>"),
    ] {
        fs::write(&input, &source).unwrap();
        let result = run("uuid:manager").assert().success();
        let value: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
        assert_eq!(value["load_manager"]["loads"][1]["io_index"], 0);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        fs::remove_file(&output).unwrap();
    }

    let source = xml
        .replace(r#"Unknown="keep-0"/>"#, r#"Unknown="keep-0"><C Type="Place" U="r1" Title="Garage"/><C Type="Place" U="r2" Title="Office"/></Load>"#)
        .replace(r#"Unknown="keep-1"/>"#, r#"Unknown="keep-1"><C Type="Place" U="r1" Title="Office"/><C Type="Place" U="r2" Title="Garage"/></Load>"#)
        .replace(r#"<IoData Pr="room"/>"#, r#"<IoData Pr="r1"/>"#)
        .replace(r#"<C Type="Place" U="room""#, r#"<C Type="LoadShed" U="other" Title="Load Manager"><IoData Pr="r2"/></C><C Type="Place" U="room""#);
    fs::write(&input, &source).unwrap();
    fs::write(&output, "existing output").unwrap();
    let result = run("Manager [Office]").assert().code(1);
    let value: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(
        value["message"],
        "Unsupported Load Manager shape: expected exactly 12 direct Load records, found 0. Sparse/extended lists are not supported; no defaults were fabricated."
    );
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), b"existing output");
    assert!(!output.with_extension("Loxone.tmp").exists());
}

#[test]
fn load_manager_cli_min_on_seconds_and_zero_follow_identity_after_move() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture()
        .replacen(" maxOff=\"123\" minOn=\"45\"", "", 1)
        .replacen(" minOn=\"45\"", "", 1);
    fs::write(&input, source).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["uuid:manager", "0", "down", "-q"])
        .assert()
        .success();
    let original = fs::read(&input).unwrap();
    let before = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&input)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mut expected: Value = serde_json::from_slice(&before).unwrap();
    for (request, stored) in [
        ("000600", Some("600")),
        ("2147483647", Some("2147483647")),
        ("0", None),
        ("10", Some("10")),
    ] {
        let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(if output.exists() { &output } else { &input })
            .args([
                "Load Manager",
                "0",
                request,
                "--room",
                "Office",
                "--save-as",
            ])
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        let attributes = expected["load_manager"]["loads"][1]["attributes"]
            .as_object_mut()
            .unwrap();
        match stored {
            Some(value) => {
                attributes.insert("minOn".into(), Value::from(value));
            }
            None => {
                attributes.remove("minOn");
            }
        }
        assert_eq!(result["changed"], true);
        assert_eq!(result["load_manager"], expected["load_manager"]);
        assert_eq!(fs::read(&input).unwrap(), original);
    }
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let xml = String::from_utf8(saved).unwrap();
    assert!(xml.contains("U=\"connector\""));
    assert!(xml.contains("WF=\"16384\""));
    assert!(!xml.contains("DateS=\"1\""));
}

#[test]
fn load_manager_cli_min_on_noops_preserve_zero_omission_spelling_and_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (storage, request) in [
        (None, "0"),
        (Some("+0000"), "000"),
        (Some("+000600"), "00600"),
    ] {
        let original = fixture().replacen(
            " minOn=\"45\"",
            &storage.map_or(String::new(), |raw| format!(" minOn=\"{raw}\"")),
            1,
        );
        fs::write(&input, original.as_bytes()).unwrap();
        let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(&input)
            .args(["uuid:manager", "0", request, "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["changed"], false);
        assert!(result["saved_to"].is_null());
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(&input)
            .args(["uuid:manager", "0", request, "--save-as"])
            .arg(&output)
            .args(["-o", "csv", "-q", "--no-header"])
            .assert()
            .success()
            .stdout(predicates::str::contains("1,0,\"Consumer 0\""));
        let saved = fs::read_to_string(&output).unwrap();
        let root =
            xmltree::Element::parse(saved.trim_start_matches('\u{feff}').as_bytes()).unwrap();
        let block = root.get_child("C").unwrap().get_child("C").unwrap();
        assert_eq!(
            block
                .get_child("Load")
                .unwrap()
                .attributes
                .get("minOn")
                .map(String::as_str),
            storage
        );
        assert!(saved.contains("DateS=\"1\""));
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    }
}

#[test]
fn load_manager_cli_min_on_keeps_legacy_scope_and_presave_report() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture()
        .replacen(r#"WF="16384""#, r#"WF="16384" IfT="false""#, 1)
        .replacen(
            r#"Unknown="keep-0""#,
            r#"Unknown="note&#32;12hTF=value""#,
            1,
        );
    fs::write(&input, &source).unwrap();
    let run = |subcommand| {
        let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
        command
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", subcommand]);
        command
    };
    let report = run("set-min-on")
        .arg(&input)
        .args(["Load Manager [Office]", "0", "600", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(report["changed"], true);
    assert_eq!(report["saved_to"], output.to_str().unwrap());
    assert_eq!(report["load_manager"]["uuid"], "manager");
    let row = &report["load_manager"]["loads"][0];
    assert_eq!(row["io_index"], 0);
    assert_eq!(row["attributes"]["minOn"], "600");
    assert_eq!(row["attributes"]["Unknown"], "note 12hTF=value");

    let after = run("inspect")
        .arg(&output)
        .args(["Load Manager [Office]", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let after: Value = serde_json::from_slice(&after).unwrap();
    let mut expected = report["load_manager"].clone();
    expected["loads"][0]["attributes"]["Unknown"] = Value::from("note _12hTF=value");
    assert_eq!(after["load_manager"], expected);
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
}

#[test]
fn load_manager_cli_min_on_rejections_are_atomic_and_not_generic_parameters() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, original.as_bytes()).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for value in [
        "",
        "1",
        "9",
        "10.0",
        "1e2",
        "+600",
        "-600",
        " 600",
        "600s",
        "2147483648",
        "NaN",
        "inf",
    ] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(&input)
            .args(["uuid:manager", "0", value, "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    for raw in [
        "",
        "-1",
        "-0",
        "600.0",
        "6e2",
        "garbage",
        "2147483648",
        " 600",
    ] {
        let malformed = original.replacen("minOn=\"45\"", &format!("minOn=\"{raw}\""), 1);
        fs::write(&input, malformed.as_bytes()).unwrap();
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(&input)
            .args(["uuid:manager", "0", "0", "--save-as"])
            .arg(&output)
            .assert()
            .failure()
            .stderr(predicates::str::contains("minOn storage"));
        assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    fs::write(&input, original.as_bytes()).unwrap();
    for parameter in ["minOn", "Load.0.minOn"] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "set-param"])
            .arg(&input)
            .args(["uuid:manager", parameter, "600", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    for malformed in [
        original.replacen("ioIdx=\"1\"", "ioIdx=\"0\"", 1),
        original.replacen("ioIdx=\"1\"", "ioIdx=\"12\"", 1),
        original.replacen("<Load ioIdx=\"1\"", "<NotLoad ioIdx=\"1\"", 1),
    ] {
        fs::write(&input, malformed.as_bytes()).unwrap();
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(&input)
            .args(["uuid:manager", "0", "600", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    fs::write(&input, original.as_bytes()).unwrap();
    for (selector, index) in [("uuid:manager", "12"), ("uuid:room", "0"), ("missing", "0")] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-min-on"])
            .arg(&input)
            .args([selector, index, "600", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["--dry-run", "config", "load-manager", "set-min-on"])
        .arg(dir.path().join("does-not-exist.Loxone"))
        .args(["uuid:manager", "0", "600", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("dry-run"))
        .stderr(predicates::str::contains("os error").not());
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
}

#[test]
fn load_manager_cli_max_off_seconds_targets_identity_after_move_and_preserves_other_fields() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture()
        .replacen(" maxOff=\"123\" minOn=\"45\"", "", 1)
        .replacen(" maxOff=\"123\"", "", 1);
    fs::write(&input, source).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["uuid:manager", "0", "down", "-q"])
        .assert()
        .success();
    let original = fs::read(&input).unwrap();
    let before = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&input)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mut expected: Value = serde_json::from_slice(&before).unwrap();
    expected["load_manager"]["loads"][1]["attributes"]["maxOff"] = Value::from("600");
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-max-off"])
        .arg(&input)
        .args([
            "Load Manager",
            "0",
            "000600",
            "--room",
            "Office",
            "--save-as",
        ])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["changed"], true);
    assert_eq!(result["load_manager"], expected["load_manager"]);
    assert_eq!(fs::read(&input).unwrap(), original);
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let xml = String::from_utf8(saved).unwrap();
    assert!(xml.contains("U=\"connector\""));
    assert!(xml.contains("WF=\"16384\""));
    for seconds in ["10", "2147483647"] {
        let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-max-off"])
            .arg(&output)
            .args(["uuid:manager", "0", seconds, "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        expected["load_manager"]["loads"][1]["attributes"]["maxOff"] = Value::from(seconds);
        assert_eq!(result["load_manager"], expected["load_manager"]);
    }
}

#[test]
fn load_manager_cli_max_off_noop_preserves_spelling_and_document_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().replacen("maxOff=\"123\"", "maxOff=\"+000600\"", 1);
    fs::write(&input, original.as_bytes()).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-max-off"])
        .arg(&input)
        .args(["uuid:manager", "0", "00600", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["changed"], false);
    assert!(result["saved_to"].is_null());
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-max-off"])
        .arg(&input)
        .args(["uuid:manager", "0", "600", "--save-as"])
        .arg(&output)
        .args(["-o", "csv", "-q", "--no-header"])
        .assert()
        .success()
        .stdout(predicates::str::contains("1,0,\"Consumer 0\""));
    let saved = fs::read_to_string(&output).unwrap();
    assert!(saved.contains("maxOff=\"+000600\""));
    assert!(saved.contains("DateS=\"1\""));
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-max-off"])
        .arg(&output)
        .args(["uuid:manager", "0", "601", "-q"])
        .assert()
        .success();
    assert!(!fs::read_to_string(&output).unwrap().contains("DateS=\"1\""));
}

#[test]
fn load_manager_cli_max_off_rejections_are_atomic_and_not_generic_parameters() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, original.as_bytes()).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for value in [
        "",
        "0",
        "1",
        "9",
        "10.0",
        "1e2",
        "+600",
        "-600",
        " 600",
        "600s",
        "2147483648",
        "NaN",
        "inf",
    ] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-max-off"])
            .arg(&input)
            .args(["uuid:manager", "0", value, "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    for raw in ["", "-1", "600.0", "6e2", "garbage", "2147483648", " 600"] {
        let malformed = original.replacen("maxOff=\"123\"", &format!("maxOff=\"{raw}\""), 1);
        fs::write(&input, malformed.as_bytes()).unwrap();
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-max-off"])
            .arg(&input)
            .args(["uuid:manager", "0", "600", "--save-as"])
            .arg(&output)
            .assert()
            .failure()
            .stderr(predicates::str::contains("maxOff storage"));
        assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    fs::write(&input, original.as_bytes()).unwrap();
    for parameter in ["maxOff", "Load.0.maxOff"] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "set-param"])
            .arg(&input)
            .args(["uuid:manager", parameter, "600", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    for (selector, index) in [("uuid:manager", "12"), ("uuid:room", "0"), ("missing", "0")] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-max-off"])
            .arg(&input)
            .args([selector, index, "600", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    let malformed = original.replacen("ioIdx=\"1\"", "ioIdx=\"0\"", 1);
    fs::write(&input, malformed.as_bytes()).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-max-off"])
        .arg(&input)
        .args(["uuid:manager", "0", "600", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("duplicate ioIdx"));
    assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["--dry-run", "config", "load-manager", "set-max-off"])
        .arg(dir.path().join("does-not-exist.Loxone"))
        .args(["uuid:manager", "0", "600", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("dry-run"))
        .stderr(predicates::str::contains("os error").not());
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
}

#[test]
fn load_manager_cli_power_uses_kw_and_stable_identity_without_touching_other_settings() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    fs::write(&input, fixture().replacen(" maxOff=\"123\"", "", 1)).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["uuid:manager", "0", "down", "-q"])
        .assert()
        .success();
    let original = fs::read(&input).unwrap();
    let inspection = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&input)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mut expected: Value = serde_json::from_slice(&inspection).unwrap();
    expected["load_manager"]["loads"][1]["attributes"]["pwr"] = Value::from("0.001");
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-power"])
        .arg(&input)
        .args([
            "Load Manager",
            "0",
            "0.001",
            "--room",
            "Office",
            "--save-as",
        ])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["changed"], true);
    assert_eq!(result["load_manager"], expected["load_manager"]);
    assert_eq!(fs::read(&input).unwrap(), original);
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let xml = String::from_utf8(saved).unwrap();
    assert!(xml.contains("U=\"connector\""));
    assert!(xml.contains("WF=\"16384\""));
    assert!(!xml.contains("DateS=\"1\""));
}

#[test]
fn load_manager_cli_power_noop_preserves_file_and_document_stamp_including_save_as() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().replacen("pwr=\"2.5\"", "pwr=\"2.500e0\"", 1);
    fs::write(&input, original.as_bytes()).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-power"])
        .arg(&input)
        .args(["uuid:manager", "0", "2.500", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["changed"], false);
    assert!(result["saved_to"].is_null());
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-power"])
        .arg(&input)
        .args(["uuid:manager", "0", "2.5", "--save-as"])
        .arg(&output)
        .args(["-o", "csv", "-q", "--no-header"])
        .assert()
        .success()
        .stdout(predicates::str::contains("1,0,\"Consumer 0\""));
    let saved = fs::read_to_string(&output).unwrap();
    assert!(saved.contains("pwr=\"2.500e0\""));
    assert!(saved.contains("DateS=\"1\""));
}

#[test]
fn load_manager_cli_power_errors_and_global_dry_run_are_atomic() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, original.as_bytes()).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for (selector, index, value) in [
        ("uuid:manager", "0", "NaN"),
        ("uuid:manager", "0", "1.0001"),
        ("uuid:manager", "0", "9007199254740993"),
        ("uuid:manager", "12", "2.5"),
        ("uuid:room", "0", "2.5"),
        ("missing", "0", "2.5"),
    ] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-power"])
            .arg(&input)
            .args([selector, index, value, "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["--dry-run", "config", "load-manager", "set-power"])
        .arg(&input)
        .args(["uuid:manager", "0", "2.5", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("dry-run"));
    for malformed in [
        original.replacen("ioIdx=\"1\"", "ioIdx=\"0\"", 1),
        original.replacen(" pwr=\"2.5\"", "", 1),
        original.replacen("pwr=\"2.5\"", "pwr=\"garbage\"", 1),
    ] {
        fs::write(&input, malformed.as_bytes()).unwrap();
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-power"])
            .arg(&input)
            .args(["uuid:manager", "0", "0", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
}

#[test]
fn load_manager_cli_saves_priority_with_identity_and_reloads() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().into_bytes();
    fs::write(&input, &original).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["Load Manager", "0", "down", "--room", "Office", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["load_manager"]["loads"][1]["io_index"], 0);
    assert_eq!(
        result["load_manager"]["loads"][1]["attributes"]["Unknown"],
        "keep-0"
    );
    assert!(
        result["load_manager"]["loads"][1]["attributes"]
            .get("ioIdx")
            .is_none()
    );
    assert_eq!(fs::read(&input).unwrap(), original);
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let xml = String::from_utf8(saved).unwrap();
    assert!(xml.contains("U=\"connector\""));
    assert!(xml.contains("WF=\"16384\""));
    assert!(!xml.contains("DateS=\"1\""));
    let inspection = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&output)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspection: Value = serde_json::from_slice(&inspection).unwrap();
    assert_eq!(inspection["load_manager"], result["load_manager"]);
    assert!(inspection["saved_to"].is_null());
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&output)
        .args(["uuid:manager", "0", "up", "-o", "json"])
        .assert()
        .success();
    let inspection = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&output)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspection: Value = serde_json::from_slice(&inspection).unwrap();
    assert_eq!(inspection["load_manager"]["loads"][0]["io_index"], 0);
    assert!(
        inspection["load_manager"]["loads"][0]["attributes"]
            .get("ioIdx")
            .is_none()
    );
}

#[test]
fn load_manager_cli_errors_do_not_touch_input_or_existing_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().into_bytes();
    fs::write(&input, &original).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for (selector, index, direction) in [
        ("uuid:manager", "0", "up"),
        ("uuid:manager", "11", "down"),
        ("uuid:manager", "12", "up"),
        ("uuid:room", "4", "up"),
        ("missing", "4", "down"),
    ] {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "move"])
            .arg(&input)
            .args([selector, index, direction, "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    let malformed = fixture().replacen("ioIdx=\"1\"", "ioIdx=\"0\"", 1);
    fs::write(&input, malformed.as_bytes()).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["uuid:manager", "4", "up", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("duplicate ioIdx"));
    assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
}

#[test]
fn load_manager_cli_sets_name_by_identity_after_move_and_preserves_noop_bytes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    fs::write(&input, fixture()).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "move"])
        .arg(&input)
        .args(["uuid:manager", "0", "down", "-q"])
        .assert()
        .success();
    let original = fs::read(&input).unwrap();
    let before = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "inspect"])
        .arg(&input)
        .args(["uuid:manager", "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let mut expected: Value = serde_json::from_slice(&before).unwrap();
    let name = "  Wärmepumpe 東京 🔌 & \"Haus\"  ";
    expected["load_manager"]["loads"][1]["attributes"]["nm"] = Value::from(name);
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&input)
        .args(["Load Manager", "0", name, "--room", "Office", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["changed"], true);
    assert_eq!(result["load_manager"], expected["load_manager"]);
    assert_eq!(fs::read(&input).unwrap(), original);
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(saved.windows(2).any(|window| window == b"\r\n"));
    let noop = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&output)
        .args(["uuid:manager", "0", name, "-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let noop: Value = serde_json::from_slice(&noop).unwrap();
    assert_eq!(noop["changed"], false);
    assert!(noop["saved_to"].is_null());
    assert_eq!(fs::read(&output).unwrap(), saved);
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&output)
        .args(["uuid:manager", "0", "", "-q", "--no-header", "-o", "csv"])
        .assert()
        .success()
        .stdout(predicates::str::contains("2,0,\"\""))
        .stdout(predicates::str::contains("PRIORITY").not());
    let xml = String::from_utf8(fs::read(&output).unwrap()).unwrap();
    assert!(!xml.contains("nm=\"\""));
    assert!(xml.contains("U=\"connector\""));
    assert!(xml.contains("WF=\"16384\""));
}

#[test]
fn load_manager_cli_name_errors_and_dry_run_are_atomic() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().into_bytes();
    fs::write(&input, &original).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    for (selector, index, name, diagnostic) in [
        (
            "missing",
            "0",
            "one\ttwo",
            Some("Unsupported Load Manager name"),
        ),
        (
            "uuid:manager",
            "0",
            "one\ntwo",
            Some("Unsupported Load Manager name"),
        ),
        ("uuid:manager", "12", "New", None),
        (
            "uuid:room",
            "0",
            "New",
            Some("is not a LoadShed (Load Manager)"),
        ),
    ] {
        let refusal = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", "set-name"])
            .arg(&input)
            .args([selector, index, name, "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        if let Some(diagnostic) = diagnostic {
            refusal.stderr(predicates::str::contains(diagnostic));
        }
        assert_eq!(fs::read(&input).unwrap(), original);
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
        assert!(!dir.path().join("output.Loxone.tmp").exists());
    }
    // Admission must reject before attempting to open even a nonexistent input.
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(dir.path().join("missing.Loxone"))
        .args(["uuid:manager", "0", "New", "--dry-run", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("dry-run not supported"));
    let malformed = fixture().replacen("ioIdx=\"1\"", "ioIdx=\"0\"", 1);
    fs::write(&input, &malformed).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&input)
        .args(["uuid:manager", "0", "New", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("duplicate ioIdx"));
    assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    let malformed = b"<Loxone><C></Loxone>";
    fs::write(&input, malformed).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&input)
        .args(["missing", "0", "one\ttwo", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("Failed to parse Loxone XML"));
    assert_eq!(fs::read(&input).unwrap(), malformed);
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    assert!(!dir.path().join("output.Loxone.tmp").exists());
}

#[test]
fn load_manager_cli_name_keeps_legacy_scope_and_presave_report() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture().replacen(r#"WF="16384""#, r#"WF="16384" IfT="false""#, 1);
    fs::write(&input, &original).unwrap();
    let run = |subcommand| {
        let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
        command
            .env("LOX_TELEMETRY", "0")
            .args(["config", "load-manager", subcommand]);
        command
    };
    let name = "at 7am=on";
    let report = run("set-name")
        .arg(&input)
        .args(["uuid:manager", "0", name, "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(report["changed"], true);
    assert_eq!(report["load_manager"]["loads"][0]["attributes"]["nm"], name);
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    let saved = fs::read_to_string(&output).unwrap();
    assert!(saved.contains(r#"IfT="false""#));
    assert!(saved.contains(r#"nm="at 7am=on""#));
    run("inspect")
        .arg(&output)
        .args(["uuid:manager", "-o", "csv", "-q", "--no-header"])
        .assert()
        .success()
        .stdout(predicates::str::contains("1,0,\"at _7am=on\""));
}

#[test]
fn load_manager_cli_name_noop_save_as_retains_explicit_empty_and_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture().replacen(r#"nm="Consumer 0""#, r#"nm="""#, 1);
    fs::write(&input, &source).unwrap();
    let report = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&input)
        .args(["uuid:manager", "0", "", "--save-as"])
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(report["changed"], false);
    assert_eq!(report["saved_to"], output.to_str().unwrap());
    assert_eq!(report["load_manager"]["loads"][0]["attributes"]["nm"], "");
    let saved = fs::read_to_string(&output).unwrap();
    assert!(saved.contains(r#"Date="2020-01-01 00:00:00" DateS="1""#));
    assert!(saved.contains(r#"<Load nm="" pwr="2.5""#));
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
}

#[test]
fn load_manager_cli_name_keeps_xml_1_1_noop_admission() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let original =
        fixture()
            .replacen("1.0", "1.1", 1)
            .replacen(r#"nm="Consumer 1""#, r#"nm="&#x1;""#, 1);
    fs::write(&input, &original).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&input)
        .args(["uuid:manager", "0", "Consumer 0", "-o", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""changed": false"#));
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
}

#[test]
fn load_manager_cli_name_change_uses_ordinary_document_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    fs::write(&input, fixture()).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "load-manager", "set-name"])
        .arg(&input)
        .args(["uuid:manager", "0", "Consumer 1", "-q", "-o", "json"])
        .assert()
        .success()
        .stderr("");
    let xml = String::from_utf8(fs::read(&input).unwrap()).unwrap();
    assert!(!xml.contains("Date=\"2020-01-01 00:00:00\""));
    assert!(!xml.contains("DateS=\"1\""));
    assert_eq!(xml.matches("nm=\"Consumer 1\"").count(), 2);
}
