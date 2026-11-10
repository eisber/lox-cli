use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn energy_inheritance_cli_inspection_reports_gates_without_resolving_or_writing() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("ux")
        .join("energy-manager2")
        .join("synthetic.Loxone");
    let before = fs::read(&fixture).unwrap();
    let inspect = |format: &str| {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .args(["config", "energy", "inspect"])
            .arg(&fixture)
            .args(["Synthetic Energy", "-o", format])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone()
    };
    let result: Value = serde_json::from_slice(&inspect("json")).unwrap();
    let customized = &result["energy"]["loads"][0];
    assert_eq!(
        customized["inheritance"]["mode_refresh"],
        "blocked_by_mode_customization"
    );
    assert_eq!(
        customized["inheritance"]["name_refresh"],
        "blocked_by_mode_customization"
    );
    assert_eq!(customized["name"]["stored"], "Pilot Alpha");
    assert_eq!(customized["analog"]["stored"], "true");
    let uncustomized = &result["energy"]["loads"][2];
    assert_eq!(
        uncustomized["inheritance"]["mode_refresh"],
        "source_unresolved"
    );
    assert_eq!(
        uncustomized["inheritance"]["name_refresh"],
        "source_unresolved"
    );
    assert!(uncustomized["name_customized"]["stored"].is_null());
    assert!(uncustomized["mode_customized"]["stored"].is_null());
    assert!(result["saved_to"].is_null());

    let text = String::from_utf8(inspect("table")).unwrap();
    assert!(text.contains("not resolved values; restore operation not recovered"));
    assert!(text.contains("blocked_by_mode_customization"));
    assert!(text.contains("source_unresolved"));
    let csv = String::from_utf8(inspect("csv")).unwrap();
    assert!(csv.contains("\"mode_refresh_gate\",\"\",\"blocked_by_mode_customization\""));
    assert!(csv.contains("\"name_refresh_gate\",\"\",\"source_unresolved\""));
    assert_eq!(fs::read(&fixture).unwrap(), before);
}
