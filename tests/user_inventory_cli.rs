use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn fixture(xml: &[u8]) -> (tempfile::TempDir, String) {
    let dir = tempfile::Builder::new()
        .prefix(".user-inventory-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("users.Loxone");
    fs::write(&path, xml).unwrap();
    (dir, path.to_str().unwrap().to_string())
}

fn lox(dir: &tempfile::TempDir) -> Command {
    let mut command = Command::cargo_bin("lox").unwrap();
    command
        .env("LOX_TELEMETRY", "0")
        .env("LOX_CONFIG", dir.path().join("absent.yaml"));
    command
}

#[test]
fn user_inventory_cli_table_limits_and_input_preservation() {
    let xml = br#"<Root><C Type="User" Title="Z&amp;B" U="same" Desc="Main_user"/>
<C Type="User" Title="A" NFCArr="0"/><C Type="User"/></Root>"#;
    let (dir, path) = fixture(xml);
    for dry_run in [false, true] {
        let mut command = lox(&dir);
        if dry_run {
            command.arg("--dry-run");
        }
        command
            .args(["config", "users", &path, "--limit", "1"])
            .assert()
            .success()
            .stdout(format!(
                "  {:<26} {:<6} {:<6} DESCRIPTION\n  {:<26} {:<6} {:<6} Main user\n\n3 users (1 with NFC)\n",
                "NAME", "NFC", "DIRECT", "Z&amp;B", "-", 0
            ))
            .stderr("Showing 1 of 3. Use --limit to see more.\n");
    }
    lox(&dir)
        .args(["--no-header", "config", "users", &path, "--limit", "0"])
        .assert()
        .success()
        .stdout("\n3 users (1 with NFC)\n")
        .stderr("Showing 0 of 3. Use --limit to see more.\n");
    assert_eq!(fs::read(&path).unwrap(), xml);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn user_inventory_cli_dom_permissions_duplicates_and_fallback_stay_head_owned() {
    let xml = br#"<Root><C Type="VirtualIn" U="object" Title="A&amp;B"/>
<C Type="User" Title="First" U="same"><PH><PRM REF="object" ACT="3"/></PH></C>
<C Type="User" Title="Last" U="same"><PH><PRM REF="missing" ACT="1" TIM="2"/></PH></C>
<C Type="User" Title="Raw" U="raw&amp;id"><PH><PRM REF="object"/></PH></C></Root>"#;
    let (dir, path) = fixture(xml);
    let output = lox(&dir)
        .args(["-o", "json", "config", "users", &path, "--limit", "2"])
        .assert()
        .success()
        .stderr("Showing 2 of 3. Use --limit to see more.\n")
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["total"], 3);
    assert_eq!(value["shown"], 2);
    assert_eq!(value["truncated"], true);
    assert_eq!(value["users"][0]["name"], "First");
    assert_eq!(value["users"][1]["name"], "Last");
    assert_eq!(
        value["users"][0]["permissions"],
        serde_json::json!([{"uuid":"missing","title":null,"type":null,"act":"1","tim":"2"}])
    );
    assert_eq!(
        value["users"][0]["permissions"],
        value["users"][1]["permissions"]
    );
    let output = lox(&dir)
        .args(["-o", "json", "config", "users", &path])
        .assert()
        .success()
        .stderr("")
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value[2]["uuid"], "raw&amp;id");
    assert_eq!(value[2]["permissions"], serde_json::json!([]));
    fs::write(
        &path,
        br#"<Root><C Type="User" Title="raw&custom;" U="u"><PH><PRM REF="object"/></PH></C></Root>"#,
    )
    .unwrap();
    lox(&dir)
        .args(["-o", "json", "config", "users", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"raw&custom;\""))
        .stdout(predicate::str::contains("\"permissions\": []"))
        .stderr("");
}

#[test]
fn user_inventory_cli_reader_errors_and_zip_guard_keep_envelopes() {
    let (dir, path) = fixture(b"<Root><C Type=\"User\"></Root>");
    lox(&dir)
        .args(["config", "users", &path])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to parse XML: ill-formed document: expected `</C>`, but `</Root>` was found",
        ));
    lox(&dir)
        .args(["-o", "json", "config", "users", &path])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"ok\": false"))
        .stdout(predicate::str::contains("Failed to parse XML"))
        .stderr("");
    lox(&dir)
        .args(["config", "users", "missing.zip"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Expected a .Loxone XML file. Run `lox config extract missing.zip` first.",
        ));
}
