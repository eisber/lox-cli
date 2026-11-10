use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn energy_explicit_name_clear_cli_dry_run_keeps_customization_and_source() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("ux")
        .join("energy-manager2")
        .join("synthetic.Loxone");
    let before = fs::read(&fixture).unwrap();
    let output = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "energy", "load", "set"])
        .arg(&fixture)
        .args([
            "Synthetic Energy",
            "30000000-0000-0000-0000000000000001",
            "--no-name",
            "--dry-run",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&output).unwrap();
    let load = &result["energy"]["loads"][0];
    assert_eq!(load["name"]["stored"], "");
    assert_eq!(load["name_customized"]["effective"], true);
    assert_eq!(load["mode_customized"]["stored"], "true");
    assert_eq!(load["analog"]["stored"], "true");
    assert_eq!(load["icon"], "40000000-0000-0000-0000000000000001");
    assert_eq!(load["attributes"]["PilotUnknown"], "retain");
    assert_eq!(result["energy"]["loads"][1]["name"]["stored"], "Pilot Beta");
    assert_eq!(result["dry_run"], true);
    assert!(result["saved_to"].is_null());
    assert_eq!(fs::read(&fixture).unwrap(), before);
}
