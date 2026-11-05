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

mod energy {
    use super::*;

    const FIRST_ID: &str = "74000000-0000-0000-0000-000000000001";
    const SECOND_ID: &str = "74000000-0000-0000-0000-000000000002";
    const XML: &str = include_str!("ux/energy-manager2/cli-synthetic.xml");

    fn fixture() -> (TempDir, String) {
        let dir = tempfile::Builder::new()
            .prefix(".energy-cli-")
            .tempdir_in(env!("CARGO_MANIFEST_DIR"))
            .unwrap();
        let path = dir.path().join("energy.Loxone");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(XML.replace("\r\n", "\n").replace('\n', "\r\n").as_bytes());
        fs::write(&path, bytes).unwrap();
        (dir, path.to_str().unwrap().into())
    }

    fn command(dir: &TempDir) -> Command {
        let mut command = lox();
        command
            .env("LOX_TELEMETRY", "0")
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .arg("--quiet");
        command
    }

    fn inspect(dir: &TempDir, file: &str) -> serde_json::Value {
        let output = command(dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "inspect",
                file,
                "CLI Energy",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        serde_json::from_slice::<serde_json::Value>(&output).unwrap()["energy"].clone()
    }

    #[test]
    fn config_energy_inspect_reports_storage_defaults_and_native_only_choices() {
        let (dir, file) = fixture();
        let original = fs::read(&file).unwrap();
        let inspection = inspect(&dir, &file);
        assert_eq!(inspection["scalar_parameters"][0]["direction"], "parameter");
        assert!(inspection["scalar_parameters"][0]["value"]["stored"].is_null());
        assert_eq!(
            inspection["scalar_parameters"][0]["value"]["effective"],
            0.0
        );
        assert_eq!(inspection["scalar_parameters"][1]["value"]["stored"], "0");
        assert_eq!(inspection["loads"].as_array().unwrap().len(), 12);
        assert_eq!(inspection["loads"][0]["used"]["effective"], true);
        assert_eq!(
            inspection["loads"][0]["preparation_seconds"]["effective"],
            60
        );
        assert_eq!(inspection["loads"][1]["deadline_label"], "24:00");
        assert_eq!(inspection["loads"][1]["deadline_gui_verified"], false);
        assert_eq!(inspection["loads"][0]["attributes"]["Future"], "opaque");
        assert_eq!(inspection["loads"][0]["child_elements"][0], "Extension");
        assert!(
            inspection["verification"]
                .as_str()
                .unwrap()
                .contains("not_complete")
        );
        command(&dir)
            .args([
                "config",
                "energy",
                "inspect",
                &file,
                "CLI Energy",
                "--room",
                "Pilot Room",
                "--no-header",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("PRIORITY").not())
            .stdout(predicate::str::contains("PARAMETER").not());
        command(&dir)
            .args([
                "--output",
                "csv",
                "config",
                "energy",
                "inspect",
                &file,
                "CLI Energy",
            ])
            .assert()
            .success()
            .stdout(predicate::str::starts_with(
                "KIND,PRIORITY,ID,FIELD,STORED,EFFECTIVE,UNIT",
            ));
        assert_eq!(fs::read(&file).unwrap(), original);
    }

    #[test]
    fn config_energy_signed_scalar_save_as_and_parameter_direction() {
        let (dir, file) = fixture();
        let original = fs::read(&file).unwrap();
        let output = dir.path().join("saved.Loxone");
        command(&dir)
            .args([
                "--json",
                "config",
                "energy",
                "set",
                &file,
                "CLI Energy",
                "O",
                "-1.25",
                "--save-as",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
        assert_eq!(fs::read(&file).unwrap(), original);
        let inspection = inspect(&dir, output.to_str().unwrap());
        assert_eq!(
            inspection["scalar_parameters"][0]["value"]["stored"],
            "-1.25"
        );
        assert_eq!(inspection["loads"][0]["attributes"]["Future"], "opaque");
        assert!(inspection["loads"][0]["switch_on_power"]["stored"].is_null());
        assert!(inspection["loads"][0]["preparation_seconds"]["stored"].is_null());
        let bytes = fs::read(&output).unwrap();
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("12hTF=\"true\""));
        assert!(text.contains("<Unknown value=\"nested\"/>"));
        assert!(text.contains("Input=\"73000000-0000-0000-0000-000000000001\""));
        assert!(text.contains("not a vendor project"));
        assert!(!text.replace("\r\n", "").contains('\n'));
        let result = command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "get-params",
                output.to_str().unwrap(),
                "CLI Energy",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let value: serde_json::Value = serde_json::from_slice(&result).unwrap();
        let offset = value["connectors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|connector| connector["key"] == "O")
            .unwrap();
        assert_eq!(offset["direction"], "P");
    }

    #[test]
    fn config_energy_load_patch_converts_minutes_and_preserves_connected_zero_power() {
        let (dir, file) = fixture();
        command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "load",
                "set",
                &file,
                "CLI Energy",
                FIRST_ID,
                "--name",
                "CLI & renamed",
                "--mode",
                "analog",
                "--preparation-seconds",
                "30",
                "--min-runtime-minutes",
                "-1",
                "--daily-minutes",
                "1440",
                "--deadline",
                "dusk",
                "--rated-power",
                "2.5",
                "--switch-on-power",
                "0",
            ])
            .assert()
            .success();
        let inspection = inspect(&dir, &file);
        assert_eq!(
            inspection["loads"][0]["min_runtime_seconds"]["effective"],
            -60
        );
        assert_eq!(inspection["loads"][0]["daily_seconds"]["effective"], 86400);
        assert_eq!(inspection["loads"][0]["switch_on_power"]["effective"], 2.5);
        assert_eq!(inspection["loads"][0]["name_customized"]["effective"], true);
        assert_eq!(inspection["loads"][0]["mode_customized"]["effective"], true);
        assert_eq!(inspection["loads"][0]["attributes"]["Future"], "opaque");
        assert_eq!(
            inspection["loads"][0]["icon"],
            "75000000-0000-0000-0000-000000000001"
        );
        command(&dir)
            .args([
                "config",
                "energy",
                "load",
                "set",
                &file,
                "CLI Energy",
                FIRST_ID,
                "--switch-on-power",
                "2.6",
                "--rated-power",
                "0",
            ])
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
        let inspection = inspect(&dir, &file);
        assert_eq!(inspection["loads"][0]["switch_on_power"]["effective"], 2.6);
        assert!(inspection["loads"][0]["rated_power"]["stored"].is_null());
        assert_eq!(inspection["loads"][0]["used"]["stored"], "true");
        assert_eq!(inspection["loads"][0]["unused_feedback"], true);
    }

    #[test]
    fn config_energy_move_keeps_record_identity_association_and_unknown_storage() {
        let (dir, file) = fixture();
        let before = inspect(&dir, &file);
        command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "load",
                "move",
                &file,
                "CLI Energy",
                FIRST_ID,
                "2",
            ])
            .assert()
            .success();
        let after = inspect(&dir, &file);
        assert_eq!(after["loads"][0]["id"], SECOND_ID);
        assert_eq!(after["loads"][1]["id"], FIRST_ID);
        assert_eq!(after["loads"][1]["output"], "L1");
        assert_eq!(
            after["loads"][1]["attributes"],
            before["loads"][0]["attributes"]
        );
        assert_eq!(
            after["loads"][1]["child_elements"],
            before["loads"][0]["child_elements"]
        );
        assert_eq!(
            after["loads"][0]["attributes"],
            before["loads"][1]["attributes"]
        );
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.contains("Input=\"73000000-0000-0000-0000-000000000001\""));
        assert!(text.contains("<Interleaved sentinel=\"position\"/>"));
        assert!(text.contains("<Unknown value=\"nested\"/>"));
    }

    #[test]
    fn config_energy_failure_and_dry_run_do_not_write_source_or_output() {
        let (dir, file) = fixture();
        let original = fs::read(&file).unwrap();
        let output = dir.path().join("must-not-exist.Loxone");
        for (parameter, value) in [
            ("MinSoc", "101"),
            ("MaxSpwr", "-1"),
            ("O", "NaN"),
            ("O", "1e3"),
        ] {
            command(&dir)
                .args([
                    "--json",
                    "config",
                    "energy",
                    "set",
                    &file,
                    "CLI Energy",
                    parameter,
                    value,
                    "--save-as",
                    output.to_str().unwrap(),
                ])
                .assert()
                .failure()
                .stdout(predicate::str::contains("\"diagnostics\""))
                .stdout(predicate::str::contains("\"ok\": false"));
            assert_eq!(fs::read(&file).unwrap(), original);
            assert!(!output.exists());
        }
        command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "load",
                "set",
                &file,
                "CLI Energy",
                FIRST_ID,
                "--name",
                "discard",
                "--rated-power",
                "8",
                "--switch-on-power",
                "-1",
                "--save-as",
                output.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stdout(predicate::str::contains("out_of_range"));
        for tail in [
            vec!["set", file.as_str(), "CLI Energy", "O", "-1.25"],
            vec![
                "load",
                "set",
                file.as_str(),
                "CLI Energy",
                FIRST_ID,
                "--min-runtime-minutes",
                "-1",
            ],
            vec!["load", "move", file.as_str(), "CLI Energy", FIRST_ID, "12"],
        ] {
            command(&dir)
                .args(["--dry-run", "-o", "json", "config", "energy"])
                .args(tail)
                .args(["--save-as", output.to_str().unwrap()])
                .assert()
                .success()
                .stdout(predicate::str::contains("\"dry_run\": true"))
                .stdout(predicate::str::contains("\"saved_to\": null"));
            assert_eq!(fs::read(&file).unwrap(), original);
            assert!(!output.exists());
        }
        command(&dir)
            .args([
                "config",
                "energy",
                "load",
                "set",
                &file,
                "CLI Energy",
                FIRST_ID,
                "--deadline",
                "sunrise",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid value"));
        assert_eq!(fs::read(&file).unwrap(), original);
        assert!(!output.exists());
        let tiny = format!("0.{}1", "0".repeat(399));
        for value in [&tiny, &format!("-{tiny}")] {
            for parameter in ["O", "MinSoc", "MaxSpwr"] {
                command(&dir)
                    .args([
                        "--json",
                        "config",
                        "energy",
                        "set",
                        &file,
                        "CLI Energy",
                        parameter,
                        value,
                        "--save-as",
                        output.to_str().unwrap(),
                    ])
                    .assert()
                    .failure()
                    .stdout(predicate::str::contains("numeric_underflow"));
                assert_eq!(fs::read(&file).unwrap(), original);
                assert!(!output.exists());
            }
            command(&dir)
                .args([
                    "--json",
                    "config",
                    "energy",
                    "load",
                    "set",
                    &file,
                    "CLI Energy",
                    FIRST_ID,
                    "--name",
                    "discard",
                    "--rated-power",
                    "8",
                    "--switch-on-power",
                    value,
                    "--save-as",
                    output.to_str().unwrap(),
                ])
                .assert()
                .failure()
                .stdout(predicate::str::contains("numeric_underflow"));
            assert_eq!(fs::read(&file).unwrap(), original);
            assert!(!output.exists());
        }
    }

    #[test]
    fn config_energy_native_fixture_edit_reorder_preserves_native_ids() {
        let (dir, file) = fixture();
        let native_xml = include_bytes!("ux/energy-manager2/synthetic.Loxone");
        fs::write(&file, native_xml).unwrap();
        let native = "30000000-0000-0000-0000000000000001";
        let rfc = "30000000-0000-0000-0000-000000000001";
        let edited = command(&dir)
            .args([
                "--json",
                "config",
                "energy",
                "load",
                "set",
                &file,
                "Synthetic Energy",
                native,
                "--name",
                "Native CLI",
                "--min-runtime-minutes",
                "-1",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let edited: serde_json::Value = serde_json::from_slice(&edited).unwrap();
        assert_eq!(edited["energy"]["loads"][0]["id"], native);
        assert_eq!(
            edited["energy"]["loads"][0]["min_runtime_seconds"]["effective"],
            -60
        );
        let moved = command(&dir)
            .args([
                "--json",
                "config",
                "energy",
                "load",
                "move",
                &file,
                "Synthetic Energy",
                rfc,
                "2",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let moved: serde_json::Value = serde_json::from_slice(&moved).unwrap();
        assert_eq!(moved["energy"]["loads"][1]["id"], native);
        assert_eq!(
            moved["energy"]["loads"][1]["attributes"],
            edited["energy"]["loads"][0]["attributes"]
        );
        assert_eq!(moved["energy"]["loads"][1]["output"], "L1");
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.contains("Input=\"23000000-0000-0000-0000000000000001\""));
        assert!(text.contains("PilotUnknown=\"retain\""));
        let before = fs::read(&file).unwrap();
        command(&dir)
            .args([
                "--json",
                "config",
                "energy",
                "load",
                "move",
                &file,
                "Synthetic Energy",
                "3000000-00000-0000-0000000000000001",
                "2",
            ])
            .assert()
            .failure()
            .stdout(predicate::str::contains("invalid_load_selector"));
        assert_eq!(fs::read(&file).unwrap(), before);
    }

    #[test]
    fn config_energy_unsupported_shape_wrong_type_and_duplicate_ids_are_explicit() {
        let (dir, file) = fixture();
        command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "inspect",
                &file,
                "CLI Actor",
            ])
            .assert()
            .failure()
            .stdout(predicate::str::contains("wrong_block_type"));
        let original = fs::read(&file).unwrap();
        let duplicate = String::from_utf8(original.clone())
            .unwrap()
            .replace(SECOND_ID, FIRST_ID);
        fs::write(&file, duplicate.as_bytes()).unwrap();
        command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "load",
                "move",
                &file,
                "CLI Energy",
                FIRST_ID,
                "2",
            ])
            .assert()
            .failure()
            .stdout(predicate::str::contains("ambiguous_load_id"));
        assert_eq!(fs::read(&file).unwrap(), duplicate.as_bytes());
        let sparse = XML.replace(
            "<Load Id=\"74000000-0000-0000-0000-00000000000c\" OIdx=\"11\"/>",
            "",
        );
        fs::write(&file, sparse.as_bytes()).unwrap();
        let inspection = inspect(&dir, &file);
        assert_eq!(inspection["load_edit_supported"], false);
        assert_eq!(inspection["loads"].as_array().unwrap().len(), 11);
        command(&dir)
            .args([
                "-o",
                "json",
                "config",
                "energy",
                "load",
                "set",
                &file,
                "CLI Energy",
                FIRST_ID,
                "--rated-power",
                "1",
            ])
            .assert()
            .failure()
            .stdout(predicate::str::contains("unsupported_load_shape"));
        assert_eq!(fs::read(&file).unwrap(), sparse.as_bytes());
    }
}

#[test]
fn config_extract_raw_loxcc_and_resources() {
    let dir = TempDir::new().unwrap();
    let compressed = lz4_flex::block::compress(FIXTURE_XML.as_bytes());
    let mut archive = Vec::new();
    archive.extend_from_slice(&0xaabbccee_u32.to_le_bytes());
    archive.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    archive.extend_from_slice(&(FIXTURE_XML.len() as u32).to_le_bytes());
    archive.extend_from_slice(&crc32fast::hash(FIXTURE_XML.as_bytes()).to_le_bytes());
    archive.extend_from_slice(&compressed);
    for extension in ["LoxCC", "LxRes"] {
        let source = dir.path().join(format!("test.{extension}"));
        fs::write(&source, &archive).unwrap();
        lox()
            .args(["config", "extract", source.to_str().unwrap()])
            .assert()
            .success();
        assert_eq!(fs::read(&source).unwrap(), archive);
        assert_eq!(
            fs::read_to_string(source.with_file_name(format!("test.{extension}.Loxone"))).unwrap(),
            FIXTURE_XML
        );
    }
}

#[test]
fn config_surfaces_reads_native_attributes_without_exposing_values_or_modifying_source() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("native.Loxone");
    let data = br#"<ControlList><C Type="LightController2" Title="DoNotExpose" 12hTF="true">
        <Co K="Q" Def="123456"/>
        <LightscenesC><LightsceneC Name="PrivateMood" Q1="987654"/></LightscenesC>
        <C Type="LightscenesC"><C Type="LightsceneC" U="PrivateUuid"/></C>
        <SET><option v="PrivateValue"/></SET>
        </C></ControlList>"#;
    fs::write(&source, data).unwrap();
    let output = lox()
        .args(["-o", "json", "config", "surfaces", source.to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output_text = String::from_utf8(output.clone()).unwrap();
    for private_value in [
        "DoNotExpose",
        "PrivateMood",
        "PrivateValue",
        "PrivateUuid",
        "987654",
        "123456",
    ] {
        assert!(!output_text.contains(private_value));
    }
    let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let surface = &result["surfaces"][0];
    assert_eq!(surface["element_type"], "LightController2");
    let attributes = surface["attributes"].as_array().unwrap();
    assert!(attributes.iter().any(|value| value == "12hTF"));
    assert!(!attributes.iter().any(|value| value == "_12hTF"));
    let paths = surface["storage_paths"].as_array().unwrap();
    assert!(
        paths
            .iter()
            .any(|value| value == "LightscenesC/LightsceneC/@Q1")
    );
    assert!(
        paths
            .iter()
            .any(|value| value == "C[Type=LightscenesC]/C[Type=LightsceneC]/@U")
    );
    assert!(paths.iter().any(|value| value == "SET/option/@v"));
    assert!(
        !paths
            .iter()
            .any(|value| value.as_str().unwrap().starts_with("Co"))
    );
    assert_eq!(fs::read(&source).unwrap(), data);
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
