use std::fs;

#[test]
fn wiring_catalog_cli_keeps_json_plain_counts_and_read_only_flags() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let path = dir.path().join("wires.Loxone");
    let original = br#"<C Title="Gate"><Co K="AQI" U="raw"/><Co K="I1"/><Co K="output"><In/></Co><Co K="I1" U="00000000-0000-0000-0000000000000000"/></C>"#;
    fs::write(&path, original).unwrap();
    let output = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "control", "wires"])
        .arg(&path)
        .args(["Gate", "-o", "json", "--dry-run"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let wires: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(wires[0]["direction"], "output");
    assert_eq!(wires[0]["target_uuid"], "raw");
    assert_eq!(wires[2]["direction"], "parameter");
    assert_eq!(wires[2]["connected"], true);
    assert_eq!(wires.as_array().unwrap().len(), 4);
    let plain = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "control", "wires"])
        .arg(&path)
        .args(["Gate", "-o", "table"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let plain = String::from_utf8(plain).unwrap();
    assert!(plain.find("Inputs:").unwrap() < plain.find("Outputs:").unwrap());
    assert!(plain.find("Outputs:").unwrap() < plain.find("Parameters:").unwrap());
    assert!(plain.ends_with("4 connectors (2 connected, 2 unconnected)\n"));
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn wiring_catalog_cli_keeps_error_envelope_and_never_writes_input() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let path = dir.path().join("wires.Loxone");
    for (bytes, selector) in [
        (b"<C>".as_slice(), "Gate"),
        (b"<C Title='\xff'/>".as_slice(), "Gate"),
        (b"<C Title='Gate'/>".as_slice(), "missing"),
        (
            b"<R><C Title='Gate'/><C Title='Gate'/></R>".as_slice(),
            "Gate",
        ),
    ] {
        fs::write(&path, bytes).unwrap();
        let output = assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "control", "wires"])
            .arg(&path)
            .args([selector, "-o", "json"])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(envelope["ok"], false);
        assert!(envelope["message"].is_string());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}
