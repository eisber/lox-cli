use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn fixture(xml: &[u8]) -> (tempfile::TempDir, String) {
    let dir = tempfile::Builder::new()
        .prefix(".device-inventory-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("devices.Loxone");
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
fn device_inventory_cli_raw_table_group_limits_and_preservation() {
    let xml = br#"<Root>
<C Type="NetworkDevice" Title="Network" MAC="aa" Addr="host"/>
<C Type="Page"><C Type="LoxAIRDevice" Title="Air" SubType="19"/>
<C Type="TreeDevice" Title="Z&amp;&#65;" Serial="S1" SubType="32780">
<C Type="TreeDevice" Title="A" Serial="" SubType="bogus"/></C>
<C Type="TreeAsensor" Title="Not an inventoried device"/>
</C></Root>"#;
    let (dir, path) = fixture(xml);
    for dry_run in [false, true] {
        let mut command = lox(&dir);
        if dry_run {
            command.arg("--dry-run");
        }
        command
            .args(["config", "devices", &path, "--limit", "2"])
            .assert()
            .success()
            .stdout(format!(
                "  Tree devices (2)\n  {:<30} {:<12} Type\n  {:<30} {:<12} Dimmer\n  {:<30} {:<12} Unknown\n\n4 devices total\n",
                "Name", "Serial", "Z&amp;&#65;", "S1", "A", ""
            ))
            .stderr("Showing 2 of 4. Use --limit to see more.\n");
    }
    lox(&dir)
        .args(["config", "devices", &path, "--limit", "0"])
        .assert()
        .success()
        .stdout("\n4 devices total\n")
        .stderr("Showing 0 of 4. Use --limit to see more.\n");
    lox(&dir)
        .args(["config", "devices", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("LoxAIR devices (1)"))
        .stdout(predicate::str::contains("Smoke detector"))
        .stdout(predicate::str::contains("Network devices (1)"))
        .stdout(predicate::str::contains("host"))
        .stderr("");
    assert_eq!(fs::read(&path).unwrap(), xml);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn device_inventory_cli_dom_routes_and_reader_error_envelope_stay_distinct() {
    let xml = br#"<Root><C Type="TreeDevice" U="device" Title="A&amp;B" Serial="S" SubType="32780"><IoData/></C></Root>"#;
    let (dir, path) = fixture(xml);
    lox(&dir)
        .args(["-o", "json", "config", "devices", &path, "--limit", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"derived_label\": \"A&B\""))
        .stdout(predicate::str::contains("\"stable_device_key\""))
        .stdout(predicate::str::contains("\"truncated\"").not());
    lox(&dir)
        .args(["config", "devices", &path, "--room", "missing"])
        .assert()
        .success()
        .stdout("No devices found.\n");
    lox(&dir)
        .args(["config", "devices", &path, "--summary"])
        .assert()
        .success()
        .stdout("")
        .stderr("1 devices, 1 with bus_address, 0 standalone, 0 low_confidence\n");
    lox(&dir)
        .args(["config", "devices", &path, "--ports", "--summary"])
        .assert()
        .success()
        .stderr("");
    fs::write(&path, b"<Root><C></Root>").unwrap();
    lox(&dir)
        .args(["config", "devices", &path])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse XML"))
        .stderr(predicate::str::contains(
            "ill-formed document: expected `</C>`, but `</Root>` was found",
        ));
    lox(&dir)
        .args(["config", "devices", "missing.zip"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Expected a .Loxone XML file. Run `lox config extract missing.zip` first.",
        ));
}
