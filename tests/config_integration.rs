use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Minimal .Loxone XML fixture with rooms, categories, a page with a control,
/// and a Tree device with child sensors/actuators.
const FIXTURE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="r-0001-0001-0001-ffff000000000001" Title="Room1" WF="16384"/>
  <C Type="Place" V="175" U="r-0002-0002-0002-ffff000000000002" Title="Room2" WF="16384"/>
  <C Type="Category" V="175" U="c-0001-0001-0001-ffff000000000001" Title="Beleuchtung" WF="16384"/>
  <C Type="Page" V="175" U="p-0001-0001-0001-ffff000000000001" Title="Page1" WF="16384">
    <C Type="And" V="175" U="a-0001-0001-0001-ffff000000000001" Title="TestAnd" WF="16384" Nc="2" Nio="3">
      <Co K="I1" U="i-0001-0001-0001-ffff000000000001"/>
      <Co K="I2" U="i-0002-0002-0002-ffff000000000002"/>
      <Co K="Q" U="q-0001-0001-0001-ffff000000000001"/>
      <IoData Cr="c-0001-0001-0001-ffff000000000001" Pr="r-0001-0001-0001-ffff000000000001"/>
    </C>
  </C>
  <C Type="TreeDevice" V="175" U="td-001-0001-0001-ffff000000000001" Title="Touch Tree" WF="16384" Serial="B056A424" SubType="32771">
    <IoData Pr="r-0001-0001-0001-ffff000000000001"/>
    <C Type="TreeAsensor" IName="AI1" V="175" U="ts-001-0001-0001-ffff000000000001" Title="Temperatur" Nio="1" WF="20480">
      <Co K="AQ" U="tq-001-0001-0001-ffff000000000001"/>
      <IoData Cr="c-0001-0001-0001-ffff000000000001" Pr="r-0001-0001-0001-ffff000000000001"/>
    </C>
    <C Type="TreeAactor" IName="AQ1" V="175" U="ta-001-0001-0001-ffff000000000001" Title="Dimmer" Nio="1" WF="20480">
      <Co K="I" U="ti-001-0001-0001-ffff000000000001"/>
      <IoData Cr="c-0001-0001-0001-ffff000000000001" Pr="r-0001-0001-0001-ffff000000000001"/>
    </C>
  </C>
</ControlList>
"#;

fn lox() -> Command {
    Command::cargo_bin("lox").unwrap()
}

/// Write the fixture XML to a temp dir and return (dir_handle, path).
fn write_fixture(dir: &TempDir) -> String {
    let path = dir.path().join("test.Loxone");
    fs::write(&path, FIXTURE_XML).unwrap();
    path.to_str().unwrap().to_string()
}

// ── config rooms ────────────────────────────────────────────────────────────

#[test]
fn config_rooms_lists_rooms() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "rooms", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("Room1"))
        .stdout(predicate::str::contains("Room2"))
        .stdout(predicate::str::contains("2 rooms"));
}

// ── config controls ─────────────────────────────────────────────────────────

#[test]
fn config_controls_lists_controls() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "controls", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("TestAnd"))
        .stdout(predicate::str::contains("And"));
}

// ── config devices ──────────────────────────────────────────────────────────

#[test]
fn config_devices_lists_devices() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "devices", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("Touch Tree"))
        .stdout(predicate::str::contains("B056A424"));
}

#[test]
fn config_devices_ports_flag() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "devices", &path, "--ports"])
        .assert()
        .success()
        // --ports uses ConfigEditor::list_device_ports(); output varies but should succeed
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn config_devices_summary_flag_stderr_only() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["-o", "json", "config", "devices", &path, "--summary"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "1 devices, 1 with bus_address, 0 standalone, 0 low_confidence",
        ));
}

// ── config validate ─────────────────────────────────────────────────────────

#[test]
fn config_validate_runs() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "validate", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ── config describe ─────────────────────────────────────────────────────────

#[test]
fn config_describe_full() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "describe", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("Room1"))
        .stdout(predicate::str::contains("controls"));
}

#[test]
fn config_describe_filtered_by_room() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "describe", &path, "--room", "Room1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Room1"));
}

// ── config add ──────────────────────────────────────────────────────────────

#[test]
fn config_add_block() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);
    let out = dir.path().join("added.Loxone");
    let out_str = out.to_str().unwrap();

    lox()
        .args([
            "config",
            "add",
            &path,
            "--type",
            "And",
            "--title",
            "NewAnd",
            "--save-as",
            out_str,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added"));

    // The output file should exist and contain the new block
    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("NewAnd"),
        "Output file should contain the new block"
    );
}

// ── config set-param / get-params ───────────────────────────────────────────

#[test]
fn config_set_param_and_get_params() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    // Set a default value on I1 of TestAnd
    lox()
        .args(["config", "set-param", &path, "TestAnd", "I1", "42"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set TestAnd.I1"));

    // Read back the params
    lox()
        .args(["config", "get-params", &path, "TestAnd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("I1"))
        .stdout(predicate::str::contains("42"));
}

// ── config template ─────────────────────────────────────────────────────────

#[test]
fn config_template_bedroom() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);
    let out = dir.path().join("templated.Loxone");
    let out_str = out.to_str().unwrap();

    lox()
        .args([
            "config",
            "template",
            &path,
            "bedroom",
            "--room",
            "Room1",
            "--save-as",
            out_str,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Template"))
        .stdout(predicate::str::contains("bedroom"))
        .stdout(predicate::str::contains("Room1"));

    assert!(out.exists(), "Template output file should exist");
}

// ── config room add ─────────────────────────────────────────────────────────

#[test]
fn config_room_add() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);
    let out = dir.path().join("room_added.Loxone");
    let out_str = out.to_str().unwrap();

    lox()
        .args([
            "config",
            "room",
            "add",
            &path,
            "NewRoom",
            "--save-as",
            out_str,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added room"))
        .stdout(predicate::str::contains("NewRoom"));

    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("NewRoom"),
        "Output file should contain the new room"
    );
}

// ── config diff ─────────────────────────────────────────────────────────────

#[test]
fn config_diff_identical_files() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);
    // Copy to a second file for diff
    let path2 = dir.path().join("test2.Loxone");
    fs::copy(&path, &path2).unwrap();
    let path2_str = path2.to_str().unwrap();

    lox()
        .args(["config", "diff", &path, path2_str])
        .assert()
        .success()
        .stdout(predicate::str::contains("Config version"));
}

// ── config diff with modifications ──────────────────────────────────────────

#[test]
fn config_diff_shows_changes() {
    let dir = TempDir::new().unwrap();
    let path1 = write_fixture(&dir);

    // Create a modified version (add a room via CLI)
    let path2 = dir.path().join("modified.Loxone");
    fs::copy(&path1, &path2).unwrap();
    let path2_str = path2.to_str().unwrap();

    lox()
        .args(["config", "room", "add", path2_str, "Room3"])
        .assert()
        .success();

    lox()
        .args(["config", "diff", &path1, path2_str])
        .assert()
        .success()
        .stdout(predicate::str::contains("Config version"));
}

// ── error cases ─────────────────────────────────────────────────────────────

#[test]
fn config_rooms_rejects_zip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.zip");
    fs::write(&path, b"not a zip").unwrap();

    lox()
        .args(["config", "rooms", path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Expected a .Loxone XML file"));
}

#[test]
fn config_rooms_missing_file() {
    lox()
        .args(["config", "rooms", "/tmp/nonexistent_lox_test_file.Loxone"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot read"));
}

// ── config controls with filters ────────────────────────────────────────────

#[test]
fn config_controls_filter_by_type() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "controls", &path, "--type", "And"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TestAnd"));
}

#[test]
fn config_controls_filter_by_room() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["config", "controls", &path, "--room", "Room1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TestAnd"));
}

// ── mutating commands preserve original when using --save-as ─────────────────

#[test]
fn config_add_preserves_original() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);
    let original = fs::read_to_string(&path).unwrap();

    let out = dir.path().join("other.Loxone");
    lox()
        .args([
            "config",
            "add",
            &path,
            "--type",
            "Or",
            "--title",
            "TestOr",
            "--save-as",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Original should be unchanged
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(original, after, "Original file should be unchanged");
    // New file should have the new block
    let new_content = fs::read_to_string(&out).unwrap();
    assert!(new_content.contains("TestOr"));
}

// ── config rooms JSON output ────────────────────────────────────────────────

#[test]
fn config_rooms_json_output() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    lox()
        .args(["--output", "json", "config", "rooms", &path])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Room1\""))
        .stdout(predicate::str::contains("\"Room2\""));
}

// ── config add idempotency ──────────────────────────────────────────────────

#[test]
fn config_add_idempotent_existing_block() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    // "TestAnd" of type "And" already exists in the fixture
    lox()
        .args([
            "config", "add", &path, "--type", "And", "--title", "TestAnd",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Already exists"))
        .stdout(predicate::str::contains("TestAnd"));

    // File should NOT be modified (no save)
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, FIXTURE_XML, "File should not be modified");
}

#[test]
fn config_add_idempotent_json_output() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);

    // JSON mode should return existing: true
    let output = lox()
        .args([
            "--output", "json", "config", "add", &path, "--type", "And", "--title", "TestAnd",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["existing"], true);
    assert_eq!(json["type"], "And");
    assert_eq!(json["title"], "TestAnd");
    assert!(!json["uuid"].as_str().unwrap().is_empty());
}

#[test]
fn config_add_idempotent_different_type_creates_new() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir);
    let out = dir.path().join("new.Loxone");
    let out_str = out.to_str().unwrap();

    // Same title "TestAnd" but different type "Or" should create a new block
    lox()
        .args([
            "config",
            "add",
            &path,
            "--type",
            "Or",
            "--title",
            "TestAnd",
            "--save-as",
            out_str,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added"));

    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("TestAnd"), "New block should be created");
}
