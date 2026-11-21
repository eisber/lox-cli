use serde_json::{Value, json};
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

#[test]
fn energy_inspect_cli_keeps_storage_defaults_order_and_enrichment() {
    let dir = tempfile::Builder::new()
        .prefix(".energy-projection-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let fixture = dir.path().join("energy.Loxone");
    let source = br#"<ControlList>
  <C Type="Place" U="den" Title="Study"/>
  <C Type="EnergyManager2" U="e" Title="Solar">
    <IoData Pr="den"/>
    <Co K="Soc" Nc="1"><In Input="20000000-0000-0000-0000000000000001"/></Co>
    <Co K="MinSoc" Def="+05.00"/><Co K="O" Def="+01.50"/><Co K="O" Def="7"/><Co K="MaxSpwr"/>
    <Load OIdx="7" Id="30000000-0000-0000-0000000000000001" Na="" U="0" Anc="true" RT="bad"
      ico="00000001-00ff-0000-0000-000000000000" Future="a&quot;b"><Extra/></Load>
    <Unknown><Load OIdx="1"/></Unknown>
    <Load OIdx="02" Id="30000000-0000-0000-0000000000000002" ico="raw-invalid"/>
  </C>
  <C Type="And" U="wrong" Title="Solar"/>
</ControlList>"#;
    fs::write(&fixture, source).unwrap();
    let inspect = |selector: &str, flags: &[&str]| {
        assert_cmd::cargo::cargo_bin_cmd!("lox")
            .env("LOX_TELEMETRY", "0")
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["config", "energy", "inspect"])
            .arg(&fixture)
            .arg(selector)
            .args(flags)
            .assert()
    };

    let output = inspect(
        "Solar",
        &[
            "--room",
            "Study",
            "--json",
            "--dry-run",
            "--trace-id",
            "energy-read",
        ],
    )
    .success();
    let result: Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(result["dry_run"], true);
    assert_eq!(result["trace_id"], "energy-read");
    assert_eq!(result["saved_to"], Value::Null);
    let energy = &result["energy"];
    assert_eq!(energy["uuid"], "e");
    assert_eq!(energy["load_edit_supported"], false);
    assert_eq!(
        energy["scalar_parameters"],
        json!([
            {"key":"O","direction":"parameter","unit":"kW","connector_present":true,
             "value":{"stored":"+01.50","effective":null,"default":0.0}},
            {"key":"MinSoc","direction":"parameter","unit":"%","connector_present":true,
             "value":{"stored":"+05.00","effective":5.0,"default":0.0}},
            {"key":"MaxSpwr","direction":"parameter","unit":"kW","connector_present":true,
             "value":{"stored":null,"effective":0.0,"default":0.0}},
        ])
    );
    assert_eq!(
        energy["storage_power_feedback"],
        json!({"status":"missing"})
    );
    let loads = energy["loads"].as_array().unwrap();
    let [first, second] = loads.as_slice() else {
        panic!("expected two direct loads");
    };
    assert_eq!(first["priority"], 1);
    assert_eq!(first["output"], "L8");
    assert_eq!(
        first["name"],
        json!({"stored":"","effective":"","default":""})
    );
    assert_eq!(first["child_elements"], json!(["Extra"]));
    assert_eq!(first["attributes"]["Future"], "a\"b");
    assert_eq!(first["icon"], "00000001-00ff-0000-0000-000000000000");
    assert_eq!(
        first["icon_resolution"],
        json!({"status":"known_builtin","native_id":"00000001-00ff-0000-0000000000000000",
               "filename":"8-ball.svg"})
    );
    assert_eq!(second["priority"], 2);
    assert_eq!(second["output"], "L3");
    assert_eq!(
        second["name"],
        json!({"stored":null,"effective":"","default":""})
    );
    assert_eq!(
        second["preparation_seconds"],
        json!({"stored":null,"effective":60,"default":60})
    );
    assert_eq!(second["icon_resolution"]["status"], "malformed_id");
    assert_eq!(
        energy["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|issue| (
                issue["code"].as_str().unwrap(),
                issue["path"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        [
            ("unsupported_load_shape", "EnergyManager2/Load"),
            ("ambiguous_scalar_binding", "EnergyManager2/Co[K=O]/@Def"),
            ("missing_storage_power", "EnergyManager2/Co[K=MaxSpwr]/@Def"),
            ("invalid_storage", "Load[1]/@RT"),
        ]
    );

    let csv = inspect("uuid:e", &["-o", "csv", "--no-header", "--quiet"]).success();
    let stdout = std::str::from_utf8(&csv.get_output().stdout).unwrap();
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        [
            r#""parameter","","e","O","+01.50","","kW""#,
            r#""parameter","","e","MinSoc","+05.00","5","%""#,
            r#""parameter","","e","MaxSpwr","","0","kW""#,
            r#""load","1","30000000-0000-0000-0000000000000001","Anc","true","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","Future","a""b","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","Id","30000000-0000-0000-0000000000000001","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","Na","","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","OIdx","7","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","RT","bad","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","U","0","","""#,
            r#""load","1","30000000-0000-0000-0000000000000001","ico","00000001-00ff-0000-0000-000000000000","","""#,
            r#""inheritance","1","30000000-0000-0000-0000000000000001","mode_refresh_gate","","blocked_by_mode_customization","""#,
            r#""inheritance","1","30000000-0000-0000-0000000000000001","name_refresh_gate","","blocked_by_mode_customization","""#,
            r#""load","2","30000000-0000-0000-0000000000000002","Id","30000000-0000-0000-0000000000000002","","""#,
            r#""load","2","30000000-0000-0000-0000000000000002","OIdx","02","","""#,
            r#""load","2","30000000-0000-0000-0000000000000002","ico","raw-invalid","","""#,
            r#""inheritance","2","30000000-0000-0000-0000000000000002","mode_refresh_gate","","source_unresolved","""#,
            r#""inheritance","2","30000000-0000-0000-0000000000000002","name_refresh_gate","","source_unresolved","""#,
        ]
    );
    let stderr = std::str::from_utf8(&csv.get_output().stderr).unwrap();
    assert!(stderr.starts_with("unsupported_load_shape at EnergyManager2/Load:"));
    assert!(!stderr.contains("Built-in icon catalog"));

    let table = inspect("Solar [Study]", &["--no-header", "--quiet"]).success();
    let stdout = std::str::from_utf8(&table.get_output().stdout).unwrap();
    assert!(stdout.find("8-ball.svg").unwrap() < stdout.find("raw-invalid").unwrap());
    assert!(!stdout.contains("PRIORITY"));

    let wrong = inspect("uuid:wrong", &["--json"]).failure().code(1);
    let error: Value = serde_json::from_slice(&wrong.get_output().stdout).unwrap();
    assert_eq!(error["error"], "wrong_block_type");
    assert_eq!(
        error["diagnostics"],
        json!([{"code":"wrong_block_type","path":"uuid:wrong",
            "message":"Selected 'Solar' is type 'And', not EnergyManager2. Use `lox config controls FILE --type EnergyManager2` to find an energy manager."}])
    );
    let ambiguous = inspect("Solar", &["--json"]).failure().code(1);
    let error: Value = serde_json::from_slice(&ambiguous.get_output().stdout).unwrap();
    assert_eq!(
        error["message"],
        "Selector 'Solar' matches 2 elements (expected 1):\n  EnergyManager2 'Solar' (uuid:e)\n  And 'Solar' (uuid:wrong)\nUse uuid:<uuid> for exact match."
    );
    let room = inspect("Solar [Study]", &["--room", "Study", "--json"])
        .failure()
        .code(1);
    let error: Value = serde_json::from_slice(&room.get_output().stdout).unwrap();
    assert_eq!(error["error"], "conflicting_room_selector");
    assert_eq!(fs::read(&fixture).unwrap().as_slice(), &source[..]);
}
