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
    let mut command = Command::cargo_bin("lox").unwrap();
    command.env("LOX_TELEMETRY", "0");
    command
}

#[test]
fn editor_kernel_set_param_legacy_boundaries() {
    let dir = tempfile::Builder::new()
        .prefix(".editor-kernel-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let source = dir.path().join("source.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<ControlList>\r\n\
\t<C Type=\"VirtualIn\" U=\"a\" Title=\"Control\" Title='Control' 12hTF=\"true\" NTXT=\"first\nsecond\">\r\n\
\t\t<Co K=\"I1\" Def=\"1\"/>\r\n\
\t\t<IoData></IoData>\r\n\
\t\t<Co K=\"Q\" U=\"wire\"><In Input=\"first\"/><In Input=\"second\" FLG=\"2\"/></Co>\r\n\
\t</C>\r\n\
\t<C Type=\"Future\" U=\"untouched\" Title=\"Collateral\" Value=\"&quot; &amp; &lt;\"/>\r\n\
</ControlList>\r\n";
    fs::write(&source, xml).unwrap();
    lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["--json", "config", "set-param"])
        .arg(&source)
        .args(["uuid:a", "I1", "42", "--save-as"])
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"old\":\"1\""))
        .stdout(predicate::str::contains("\"new\":\"42\""));
    assert_eq!(fs::read(&source).unwrap(), xml.as_bytes());
    let revised = fs::read_to_string(&output).unwrap();
    assert!(revised.starts_with('\u{feff}'));
    assert!(revised.contains("12hTF=\"true\""));
    assert!(revised.contains("NTXT=\"first\nsecond\""));
    assert!(revised.contains("<IoData></IoData>"));
    assert!(revised.contains("Input=\"first\""));
    assert!(revised.find("Input=\"first\"").unwrap() < revised.find("Input=\"second\"").unwrap());
    assert!(revised.contains("Value=\"&quot; &amp; &lt;\""));
    assert_eq!(revised.matches("Title=\"Control\"").count(), 1);
    for (selector, param, expected) in [
        ("uuid:a", "MaxVal", "Parameter 'MaxVal' not found"),
        ("Missing", "I1", "Element 'Missing' not found."),
        ("", "I1", "matches 2 elements (expected 1)"),
    ] {
        fs::write(&output, b"sentinel").unwrap();
        lox()
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args(["config", "set-param"])
            .arg(&source)
            .args([selector, param, "9", "--save-as"])
            .arg(&output)
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected));
        assert_eq!(fs::read(&output).unwrap(), b"sentinel");
        assert_eq!(fs::read(&source).unwrap(), xml.as_bytes());
    }
    lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args(["--dry-run", "config", "set-param"])
        .arg(&source)
        .args(["uuid:a", "I1", "42"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("dry-run not supported"));
    assert_eq!(fs::read(&source).unwrap(), xml.as_bytes());
}

/// Write the fixture XML to a temp dir and return (dir_handle, path).
fn write_fixture(dir: &TempDir) -> String {
    let path = dir.path().join("test.Loxone");
    fs::write(&path, FIXTURE_XML).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn config_xml_title_edit_preserves_ordered_wires_and_unrelated_metadata() {
    let dir = tempfile::Builder::new()
        .prefix(".title-edit-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("title.Loxone");
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267" NextObj="73" NextMem="3">
	<C Type="Document" U="10000000-0000-0000-0000-000000000001" Title="Test" Date="2020-01-01 00:00:00" DateS="347068800" NumO="4">
		<C Type="Page" U="10000000-0000-0000-0000-000000000002" Title="Page">
			<C Type="EnergyManager2" U="10000000-0000-0000-0000-000000000003" Title="Test Energy" LtE="300000000" WF="16384">
				<Co K="AI1" U="20000000-0000-0000-0000-000000000001">
					<In Input="20000000-0000-0000-0000-000000000003"/>
					<In Input="20000000-0000-0000-0000-000000000002" FLG="2"/>
					<In Input="20000000-0000-0000-0000-000000000003" FLG="1"/>
				</Co>
				<SET>
					<Future t="11" v="retain"/>
				</SET>
			</C>
			<C Type="Constant" U="10000000-0000-0000-0000-000000000004" Title="Source">
				<Co K="AQ" U="20000000-0000-0000-0000-000000000002"/>
				<Co K="AQ2" U="20000000-0000-0000-0000-000000000003"/>
			</C>
		</C>
	</C>
</ControlList>
"#;
    let mut before = vec![0xEF, 0xBB, 0xBF];
    before.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    fs::write(&path, &before).unwrap();

    lox()
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .args([
            "config",
            "xml",
            "set-attr",
            path.to_str().unwrap(),
            "uuid:10000000-0000-0000-0000-000000000003",
            "Title",
            "Renamed Energy",
        ])
        .assert()
        .success();

    let after = fs::read(&path).unwrap();
    assert!(after.starts_with(&[0xEF, 0xBB, 0xBF]));
    let text = std::str::from_utf8(&after[3..]).unwrap();
    assert!(!text.replace("\r\n", "").contains('\n'));
    let actual = xmltree::Element::parse(&after[3..]).unwrap();
    let actual_doc = actual.get_child("C").unwrap();
    assert!(actual_doc.attributes["DateS"].parse::<i64>().unwrap() > 347068800);
    assert_ne!(actual_doc.attributes["Date"], "2020-01-01 00:00:00");

    // Raw set-attr minimal-change contract, not native Title-edit/LtE parity.
    let mut expected = xmltree::Element::parse(&before[3..]).unwrap();
    let document = expected.get_mut_child("C").unwrap();
    for key in ["Date", "DateS"] {
        document
            .attributes
            .insert(key.to_string(), actual_doc.attributes[key].clone());
    }
    document
        .get_mut_child("C")
        .unwrap()
        .get_mut_child("C")
        .unwrap()
        .attributes
        .insert("Title".to_string(), "Renamed Energy".to_string());
    assert_eq!(actual, expected);
}

#[test]
fn config_set_mood_color_rejects_zero_index_without_writing() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-mood-cli-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("lighting.Loxone");
    let output = dir.path().join("edited.Loxone");
    let mut before = vec![0xEF, 0xBB, 0xBF];
    before.extend_from_slice(
        include_str!("ux/light-controller2/synthetic.Loxone")
            .replace("\r\n", "\n")
            .replace('\n', "\r\n")
            .as_bytes(),
    );
    fs::write(&input, &before).unwrap();

    for save_as in [false, true] {
        let mut command = lox();
        command
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .args([
                "config",
                "set-mood-color",
                input.to_str().unwrap(),
                "uuid:lighting",
                "--mood",
                "First",
                "--color",
                "hsv(120,100,10)",
                "--output-index",
                "0",
            ]);
        if save_as {
            command.args(["--save-as", output.to_str().unwrap()]);
        }
        command
            .assert()
            .failure()
            .stderr(predicate::str::contains("Q0").and(predicate::str::contains("1-based")))
            .stdout(predicate::str::is_empty());
        assert_eq!(fs::read(&input).unwrap(), before);
        assert!(!output.exists());
    }
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
    fn config_energy_icons_lists_exact_members_and_provenance_without_a_project() {
        let (dir, _) = fixture();
        let output = command(&dir)
            .args(["--json", "config", "energy", "icons"])
            .assert()
            .success()
            .stderr(predicate::str::is_empty())
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["count"], 623);
        assert_eq!(result["source"], "builtin");
        assert_eq!(result["catalog"]["version"], "20261009");
        assert_eq!(
            result["catalog"]["archive_sha256"],
            "11656e55212ffbf0153f9075ee21da28da04f8c23d5d5e6efe945f4782391df4"
        );
        let icons = result["icons"].as_array().unwrap();
        let fan = icons
            .iter()
            .find(|icon| icon["filename"] == "electric-fan.svg")
            .unwrap();
        assert_eq!(fan["native_id"], "00000094-00ff-0000-0000000000000000");
        assert!(
            !icons
                .iter()
                .any(|icon| icon["filename"] == "awning-close.svg")
        );
        for flags in [vec![], vec!["--no-header"]] {
            let output = command(&dir)
                .args(["config", "energy", "icons"])
                .args(&flags)
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();
            let text = String::from_utf8(output).unwrap();
            assert_eq!(text.starts_with("NAME"), flags.is_empty());
            assert!(text.contains("electric-fan.svg"));
            assert!(text.contains("623 built-in icons"));
            assert!(text.contains("archive SHA256"));
            let output = command(&dir)
                .args(["--output", "csv", "config", "energy", "icons"])
                .args(&flags)
                .assert()
                .success()
                .stderr(predicate::str::is_empty())
                .get_output()
                .stdout
                .clone();
            let text = String::from_utf8(output).unwrap();
            assert_eq!(
                text.starts_with("SOURCE,NAME,NATIVE_ID,CATALOG_VERSION,ARCHIVE_SHA256"),
                flags.is_empty()
            );
            assert_eq!(text.lines().count(), 623 + usize::from(flags.is_empty()));
            assert!(text.contains("\"builtin\",\"electric-fan.svg\",\"00000094-00ff-0000-0000000000000000\",\"20261009\""));
        }
    }

    #[test]
    fn config_energy_builtin_icon_selection_clear_room_save_as_and_dry_run() {
        let (dir, file) = fixture();
        let original = fs::read(&file).unwrap();
        let baseline = inspect(&dir, &file);
        assert_eq!(
            baseline["loads"][0]["icon_resolution"]["status"],
            "unresolved"
        );
        let output = dir.path().join("selected.Loxone");
        for selector in [
            "electric-fan.svg",
            "00000094-00FF-0000-0000000000000000",
            "00000094-00FF-0000-0000-000000000000",
        ] {
            command(&dir)
                .args([
                    "--json",
                    "--dry-run",
                    "config",
                    "energy",
                    "load",
                    "set",
                    &file,
                    "CLI Energy",
                    FIRST_ID,
                    "--room",
                    "Pilot Room",
                    "--icon",
                    selector,
                    "--save-as",
                    output.to_str().unwrap(),
                ])
                .assert()
                .success()
                .stdout(predicate::str::contains("\"status\": \"known_builtin\""))
                .stdout(predicate::str::contains(
                    "\"filename\": \"electric-fan.svg\"",
                ))
                .stdout(predicate::str::contains("\"saved_to\": null"));
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
                "CLI Energy [Pilot Room]",
                FIRST_ID,
                "--icon",
                "electric-fan.svg",
                "--save-as",
                output.to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"builtin_icon_catalog\""));
        assert_eq!(fs::read(&file).unwrap(), original);
        let selected = inspect(&dir, output.to_str().unwrap());
        assert_eq!(
            selected["loads"][0]["icon"],
            "00000094-00ff-0000-0000000000000000"
        );
        let mut expected = baseline["loads"][0]["attributes"].clone();
        expected["ico"] = "00000094-00ff-0000-0000000000000000".into();
        assert_eq!(selected["loads"][0]["attributes"], expected);
        command(&dir)
            .args([
                "--json",
                "--dry-run",
                "config",
                "energy",
                "load",
                "set",
                output.to_str().unwrap(),
                "CLI Energy",
                FIRST_ID,
                "--no-icon",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"status\": \"none\""));
        assert_eq!(
            inspect(&dir, output.to_str().unwrap())["loads"],
            selected["loads"]
        );
        command(&dir)
            .args([
                "--json",
                "config",
                "energy",
                "load",
                "set",
                output.to_str().unwrap(),
                "CLI Energy",
                FIRST_ID,
                "--no-icon",
            ])
            .assert()
            .success();
        let cleared = inspect(&dir, output.to_str().unwrap());
        expected.as_object_mut().unwrap().remove("ico");
        assert_eq!(cleared["loads"][0]["attributes"], expected);
        assert!(cleared["loads"][0]["icon"].is_null());
        assert_eq!(cleared["loads"][1], baseline["loads"][1]);
        let bytes = fs::read(&output).unwrap();
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.replace("\r\n", "").contains('\n'));
        assert!(text.contains("12hTF=\"true\""));
        assert!(text.contains("<Unknown value=\"nested\"/>"));
        assert!(text.contains("Input=\"73000000-0000-0000-0000-000000000001\""));
    }

    #[test]
    fn config_energy_icon_errors_and_conflicting_clear_flags_do_not_write() {
        let (dir, file) = fixture();
        let original = fs::read(&file).unwrap();
        let output = dir.path().join("must-not-exist.Loxone");
        for selection in [
            "",
            "builtin:",
            "Electric-fan.svg",
            "awning-close.svg",
            "0000001d-00ff-0000-0000000000000000",
            "000000a0-00ff-0000-0000000000000000",
            "0000094-000ff-0000-0000000000000000",
            "00000000-0000-0000-0000000000000000",
            "project:75000000-0000-0000-0000-000000000001",
            "75000000-0000-0000-0000-000000000001",
        ] {
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
                    "--icon",
                    selection,
                    "--save-as",
                    output.to_str().unwrap(),
                ])
                .assert()
                .failure()
                .stdout(predicate::str::contains("\"ok\": false"))
                .stdout(predicate::str::contains("config energy icons"))
                .stdout(predicate::str::contains("--no-icon"));
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
                "--icon",
                "electric-fan.svg",
                "--no-icon",
                "--save-as",
                output.to_str().unwrap(),
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("cannot be used with"));
        assert_eq!(fs::read(&file).unwrap(), original);
        assert!(!output.exists());
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
        assert_eq!(inspection["loads"][1]["deadline_gui_verified"], true);
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
                "00:00",
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

mod archive_boundary {
    use super::*;
    use std::io::{Cursor, Write};

    fn sandbox() -> TempDir {
        tempfile::Builder::new()
            .prefix(".archive-cli-")
            .tempdir_in(env!("CARGO_MANIFEST_DIR"))
            .unwrap()
    }

    fn command(dir: &TempDir) -> Command {
        let mut command = lox();
        command
            .current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"));
        command
    }

    fn zip_entry(name: &str, data: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn config_archive_cli_roundtrip_preserves_raw_bytes_and_emits_crc() {
        let dir = sandbox();
        let mut xml = vec![0xEF, 0xBB, 0xBF];
        let text = FIXTURE_XML
            .replacen(
                "<ControlList Version=\"267\">",
                "<ControlList Version='267' 12hTF='true' NextObj='00073' NextObj='00073'>",
                1,
            )
            .replace("\r\n", "\n")
            .replace('\n', "\r\n")
            .replace("Title=\"Room1\"", "Title='Room&#49;' Note='first\nsecond'");
        xml.extend_from_slice(text.as_bytes());
        fs::write(dir.path().join("source.loxone"), &xml).unwrap();

        command(&dir)
            .args(["config", "compress", "source.loxone"])
            .assert()
            .success()
            .stderr(predicate::str::is_empty());
        let compressed = fs::read(dir.path().join("source.LoxCC")).unwrap();
        assert_eq!(
            u32::from_le_bytes(compressed[12..16].try_into().unwrap()),
            crc32fast::hash(&xml)
        );
        assert_eq!(fs::read(dir.path().join("source.loxone")).unwrap(), xml);

        command(&dir)
            .args([
                "config",
                "compress",
                "source.loxone",
                "--save-as",
                "custom.LxRes",
            ])
            .assert()
            .success()
            .stdout(format!(
                "Compressed {} KB → {} KB → custom.LxRes\n",
                xml.len() / 1024,
                compressed.len() / 1024
            ))
            .stderr(predicate::str::is_empty());
        assert_eq!(
            fs::read(dir.path().join("custom.LxRes")).unwrap(),
            compressed
        );

        let zip = zip_entry("SpS0.lOxCc", &compressed);
        fs::write(dir.path().join("backup.zip"), &zip).unwrap();
        let baseline = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "host": "offline.invalid",
            "serial": "synthetic",
            "xml": std::str::from_utf8(&xml).unwrap(),
        }))
        .unwrap();
        fs::write(dir.path().join("custom.LxRes.lox-base.json"), &baseline).unwrap();
        for (input, output, explicit) in [
            ("custom.LxRes", "raw-output.Loxone", true),
            ("backup.zip", "backup.Loxone", false),
        ] {
            fs::write(dir.path().join(output), b"previous destination").unwrap();
            let output_baseline = dir.path().join(format!("{output}.lox-base.json"));
            fs::write(&output_baseline, b"stale baseline").unwrap();
            let mut cmd = command(&dir);
            cmd.args(["config", "extract", input]);
            if explicit {
                cmd.args(["--save-as", output]);
            }
            cmd.assert()
                .success()
                .stdout(format!(
                    "Decompressed {} KB → {} KB → {output}\n",
                    if explicit {
                        compressed.len()
                    } else {
                        zip.len()
                    } / 1024,
                    xml.len() / 1024
                ))
                .stderr("Extracting config archive...\n");
            assert_eq!(fs::read(dir.path().join(output)).unwrap(), xml);
            if explicit {
                assert_eq!(fs::read(output_baseline).unwrap(), baseline);
            } else {
                assert!(!output_baseline.exists());
            }
        }
        assert_eq!(fs::read(dir.path().join("backup.zip")).unwrap(), zip);
        assert_eq!(
            fs::read(dir.path().join("custom.LxRes")).unwrap(),
            compressed
        );
    }

    #[test]
    fn config_archive_extract_decode_errors_leave_destinations_untouched() {
        let dir = sandbox();
        let mut truncated = 0xaabbccee_u32.to_le_bytes().to_vec();
        truncated.push(0);
        for (name, data, message) in [
            ("invalid.zip", b"not a zip".to_vec(), "Invalid ZIP archive"),
            (
                "missing-entry.zip",
                zip_entry("unrelated.txt", b"retain"),
                "Backup ZIP does not contain sps0.LoxCC",
            ),
            (
                "truncated.LoxCC",
                truncated,
                "LoxCC data too short (5 bytes)",
            ),
        ] {
            fs::write(dir.path().join(name), &data).unwrap();
            for existing in [false, true] {
                let output = if existing {
                    "existing.Loxone"
                } else {
                    "absent.Loxone"
                };
                if existing {
                    fs::write(dir.path().join(output), b"retain destination").unwrap();
                }
                command(&dir)
                    .args(["config", "extract", name, "--save-as", output])
                    .assert()
                    .code(1)
                    .stdout(predicate::str::is_empty())
                    .stderr(
                        predicate::str::starts_with("Extracting config archive...\n")
                            .and(predicate::str::contains(message)),
                    );
                if existing {
                    assert_eq!(
                        fs::read(dir.path().join(output)).unwrap(),
                        b"retain destination"
                    );
                } else {
                    assert!(!dir.path().join(output).exists());
                }
            }
            assert_eq!(fs::read(dir.path().join(name)).unwrap(), data);
        }
    }

    #[test]
    fn config_archive_read_errors_precede_progress_and_do_not_create_output() {
        let dir = sandbox();
        for operation in ["compress", "extract"] {
            command(&dir)
                .args([
                    "config",
                    operation,
                    "missing.Loxone",
                    "--save-as",
                    "output.Loxone",
                ])
                .assert()
                .code(1)
                .stdout(predicate::str::is_empty())
                .stderr(
                    predicate::str::contains("Cannot read missing.Loxone")
                        .and(predicate::str::contains("Extracting config archive").not()),
                );
            assert!(!dir.path().join("output.Loxone").exists());
        }
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
fn config_diff_comparison_contract_human_and_json() {
    let dir = tempfile::Builder::new()
        .prefix(".diff-contract-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let before = br#"<ControlList Version="1">
      <C Type="Document" Date="old"/>
      <C Type="Place" U="r" Title="Before"/>
      <C Type="Category" U="c" Title="Lights"/>
      <C Type="Switch" U="same" Title="Old"><IoData Pr="r"/></C>
      <C Type="Switch" U="gone" Title="Twin"/>
      <C Type="User" Title="Keep"/><C Type="User" Title="Keep"/>
      <C Type="User" Title="Gone"/><C Type="User" Title="Gone"/>
    </ControlList>"#;
    let after = br#"<ControlList Version="2">
      <C Type="Document" Date="new"/>
      <C Type="Place" U="r" Title="After"/>
      <C Type="Category" U="c" Title="Lighting"/>
      <C Type="Dimmer" U="same" Title="New"><IoData Pr="unknown"/></C>
      <C Type="Switch" U="added" Title="Twin"/>
      <C Type="User" Title="Keep"/>
      <C Type="User" Title="Added"/><C Type="User" Title="Added"/>
    </ControlList>"#;
    fs::write(dir.path().join("old.Loxone"), before).unwrap();
    fs::write(dir.path().join("new.Loxone"), after).unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"));
        cmd
    };
    let output = command()
        .args(["--json", "config", "diff", "old.Loxone", "new.Loxone"])
        .assert()
        .success()
        .stderr("")
        .get_output()
        .clone();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!({
            "version_old":"1", "version_new":"2", "date_old":"old", "date_new":"new",
            "controls_added":[{"name":"Twin", "control_type":"Switch"}],
            "controls_removed":[{"name":"Twin", "control_type":"Switch"}],
            "controls_changed":[
                {"name":"New", "field":"name", "old_value":"Old", "new_value":"New"},
                {"name":"New", "field":"type", "old_value":"Switch", "new_value":"Dimmer"},
                {"name":"New", "field":"room", "old_value":"Before", "new_value":"unknown"}
            ],
            "rooms_added":[], "rooms_removed":[],
            "rooms_renamed":[{"old":"Before", "new":"After"}],
            "categories_added":[], "categories_removed":[],
            "categories_renamed":[{"old":"Lights", "new":"Lighting"}],
            "users_added":["Added"], "users_removed":["Gone"]
        })
    );
    command()
        .args(["config", "diff", "old.Loxone", "new.Loxone"])
        .assert()
        .success()
        .stderr("")
        .stdout(concat!(
            "Config version: 1 → 2\nModified: old → new\n\nControls:\n",
            "  + Added: \"Twin\" (Switch)\n",
            "  ~ Changed: \"New\" — name \"Old\" → \"New\"\n",
            "  ~ Changed: \"New\" — type \"Switch\" → \"Dimmer\"\n",
            "  ~ Changed: \"New\" — room \"Before\" → \"unknown\"\n",
            "  - Removed: \"Twin\" (Switch)\n\nRooms:\n",
            "  ~ Renamed: \"Before\" → \"After\"\n\nCategories:\n",
            "  ~ Renamed: \"Lights\" → \"Lighting\"\n\nUsers:\n",
            "  + Added: \"Added\"\n  - Removed: \"Gone\"\n\n9 changes total\n"
        ));
    assert_eq!(fs::read(dir.path().join("old.Loxone")).unwrap(), before);
    assert_eq!(fs::read(dir.path().join("new.Loxone")).unwrap(), after);
}

#[test]
fn config_diff_metadata_and_ignored_xml_are_not_structural_changes() {
    let dir = tempfile::Builder::new()
        .prefix(".diff-ignored-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let before = br#"<ControlList Version="1"><C Type="Document" Date="old"/>
      <C Type="Place" U="r" Title="Before"/>
      <C Type="Switch" U="a" Title="Stable" X="1"><IoData Pr="r"/><P V="1"/></C>
      <C Type="User" Title="Keep"/><C Type="User" Title="Keep"/>
    </ControlList>"#;
    let after = br#"<ControlList Version="2"><C Type="Document" Date="new"/>
      <C Type="Place" U="r" Title="Before"/>
      <C Type="Switch" U="a" Title="Stable" X="99"><IoData Pr="r"/><P V="9"/></C>
      <C Type="User" Title="Keep"/>
    </ControlList>"#;
    fs::write(dir.path().join("old.Loxone"), before).unwrap();
    fs::write(dir.path().join("new.Loxone"), after).unwrap();
    let mut command = lox();
    command
        .current_dir(dir.path())
        .env("LOX_CONFIG", dir.path().join("absent.yaml"))
        .env("LOX_HOME", dir.path().join("home"))
        .args(["config", "diff", "old.Loxone", "new.Loxone"])
        .assert()
        .success()
        .stderr("")
        .stdout("Config version: 1 → 2\nModified: old → new\n\nNo structural changes.\n");
}

#[test]
fn config_diff_room_reference_presentation() {
    let dir = tempfile::Builder::new()
        .prefix(".diff-room-labels-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    for (old_uuid, old_room, new_uuid, new_room, old_label, new_label) in [
        ("", "", "unknown", "", "(none)", "unknown"),
        ("unknown", "", "", "", "unknown", "(none)"),
        (
            "",
            r#"<C Type="Place" U="" Title="Empty ID"/>"#,
            "r",
            r#"<C Type="Place" U="r" Title=""/>"#,
            "Empty ID",
            "",
        ),
    ] {
        for (file, uuid, room) in [
            ("old.Loxone", old_uuid, old_room),
            ("new.Loxone", new_uuid, new_room),
        ] {
            fs::write(
                dir.path().join(file),
                format!(
                    r#"<Root>{room}<C Type="Switch" U="a" Title="Stable"><IoData Pr="{uuid}"/></C></Root>"#
                ),
            )
            .unwrap();
        }
        let output = lox()
            .current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"))
            .args(["--json", "config", "diff", "old.Loxone", "new.Loxone"])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .clone();
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value["controls_changed"],
            serde_json::json!([{
                "name":"Stable", "field":"room", "old_value":old_label, "new_value":new_label
            }])
        );
    }
}

#[test]
fn summary_producer_cli_raw_loader_output_and_zip_dispatch() {
    let dir = tempfile::Builder::new()
        .prefix(".summary-producer-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let old = dir.path().join("old.Loxone");
    let new = dir.path().join("new.ZIP");
    let before = b"\xef\xbb\xbf<ControlList Version=\"1\"><C Type=\"Document\" Date=\"old\"/><C Type=\"Place\" U=\"r1\" Title=\"A&amp;B\"/><C Type=\"Switch\" U=\"a\" Title='has 12hTF=\"x\"'><IoData Pr=\"r1\"/></C></ControlList>";
    let after = "<ControlList Version=\"2\"><C Type=\"Document\" Date=\"new\"/><C Type=\"Place\" U=\"r2\" Title=\"室\"/><C Type=\"Switch\" U=\"a\" Title='has 12hTF=\"x\"'><IoData Pr=\"r2\"/></C></ControlList>".as_bytes();
    fs::write(&old, before).unwrap();
    fs::write(&new, after).unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"));
        cmd
    };
    let expected = serde_json::json!({
        "version_old":"1", "version_new":"2", "date_old":"old", "date_new":"new",
        "controls_added":[], "controls_removed":[],
        "controls_changed":[{"name":"has 12hTF=\"x\"", "field":"room", "old_value":"A&amp;B", "new_value":"室"}],
        "rooms_added":["室"], "rooms_removed":["A&amp;B"], "rooms_renamed":[],
        "categories_added":[], "categories_removed":[], "categories_renamed":[],
        "users_added":[], "users_removed":[]
    });
    for dry_run in [false, true] {
        let mut cmd = command();
        cmd.args(["--json", "config", "diff", "old.Loxone", "new.ZIP"]);
        if dry_run {
            cmd.arg("--dry-run");
        }
        let result = cmd.assert().success().stderr("").get_output().clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap(),
            expected
        );
    }
    command()
        .args(["config", "diff", "old.Loxone", "new.ZIP"])
        .assert()
        .success()
        .stdout("Config version: 1 → 2\nModified: old → new\n\nControls:\n  ~ Changed: \"has 12hTF=\"x\"\" — room \"A&amp;B\" → \"室\"\n\nRooms:\n  + Added: \"室\"\n  - Removed: \"A&amp;B\"\n\n3 changes total\n")
        .stderr("");
    let zip_path = dir.path().join("backup.zip");
    {
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
        zip.start_file("sps0.LoxCC", zip::write::SimpleFileOptions::default())
            .unwrap();
        let archive = lox_core::configuration::archive::compress_loxcc(before);
        std::io::Write::write_all(&mut zip, &archive).unwrap();
        zip.finish().unwrap();
    }
    command()
        .args(["config", "diff", "old.Loxone", "backup.zip"])
        .assert()
        .success()
        .stdout("Config version: 1 → 1\nModified: old → old\n\nNo structural changes.\n")
        .stderr("");
    assert_eq!(fs::read(old).unwrap(), before);
    assert_eq!(fs::read(new).unwrap(), after);
    assert!(!dir.path().join("home").exists());
}

#[test]
fn summary_producer_cli_errors_and_permissive_input() {
    let dir = tempfile::Builder::new()
        .prefix(".summary-producer-errors-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    fs::write(dir.path().join("empty.Loxone"), b"not XML").unwrap();
    fs::write(dir.path().join("open.Loxone"), b"<Root>").unwrap();
    fs::write(dir.path().join("bad.Loxone"), b"<Root><C></Root>").unwrap();
    fs::write(dir.path().join("bad.zip"), b"not a zip").unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"));
        cmd
    };
    command()
        .args(["config", "diff", "empty.Loxone", "open.Loxone"])
        .assert()
        .success()
        .stdout("Config version:  → \nModified:  → \n\nNo structural changes.\n");
    for (file, prefix) in [
        (
            "bad.Loxone",
            "Failed to parse XML: ill-formed document: expected `</C>`, but `</Root>` was found: expected `</C>`, but `</Root>` was found",
        ),
        ("missing.Loxone", "Cannot read missing.Loxone: "),
        ("bad.zip", "Invalid ZIP archive: "),
    ] {
        let output = command()
            .args(["--json", "config", "diff", "empty.Loxone", file])
            .assert()
            .code(1)
            .stderr("")
            .get_output()
            .clone();
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"], "error");
        let message = value["message"].as_str().unwrap();
        assert!(message.starts_with(prefix), "{message}");
        command()
            .args(["config", "diff", "empty.Loxone", file])
            .assert()
            .code(1)
            .stdout("")
            .stderr(format!(
                "Error: {message}\n\n{}\n",
                value["report_hint"].as_str().unwrap()
            ));
    }
}

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
fn control_inventory_cli_filters_limits_formats_and_raw_loader_boundary() {
    let dir = tempfile::Builder::new()
        .prefix(".control-inventory-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("controls.Loxone");
    let mut bytes = b"\xef\xbb\xbf".to_vec();
    bytes.extend_from_slice(
        r#"<Root>
<C Type="Switch" U="b" Title="Z"><IoData Pr="r" Cr="c"/></C>
<C Type="Page" U="p" Title='has 12hTF="x"'/><Other><IoData Pr="raw&amp;id"/></Other>
<C Type="Switch" U="a" Title="A&amp;&#65;"><IoData Pr="r"/></C>
<C Type="Place" U="r" Title="Old"/><C Type="Place" U="r" Title="KÜCHE"/>
<C Type="Category" U="c" Title="Light"/>
</Root>"#
            .as_bytes(),
    );
    fs::write(&path, &bytes).unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"))
            .args(["--quiet", "--trace-id", "control-inventory"]);
        cmd
    };
    let all = serde_json::json!([
        {"control_type":"Page","title":"has 12hTF=\"x\"","uuid":"p","room":"raw&amp;id","category":""},
        {"control_type":"Switch","title":"A&amp;&#65;","uuid":"a","room":"KÜCHE","category":""},
        {"control_type":"Switch","title":"Z","uuid":"b","room":"KÜCHE","category":"Light"}
    ]);
    for (filters, selected) in [
        (vec![], all.clone()),
        (
            vec!["--type", "sWiTcH", "--room", "küc"],
            serde_json::json!(&all.as_array().unwrap()[1..]),
        ),
        (vec!["--type", ""], serde_json::json!([])),
        (vec!["--room", "absent"], serde_json::json!([])),
    ] {
        for limit in [None, Some(0), Some(1), Some(3), Some(8)] {
            let mut cmd = command();
            cmd.args([
                "--json",
                "--dry-run",
                "config",
                "controls",
                "controls.Loxone",
            ])
            .args(&filters);
            if let Some(limit) = limit {
                cmd.args(["--limit", &limit.to_string()]);
            }
            let output = cmd.assert().success().get_output().clone();
            let total = selected.as_array().unwrap().len();
            let shown = limit.unwrap_or(100).min(total);
            let expected = if shown < total {
                serde_json::json!({
                    "controls": &selected.as_array().unwrap()[..shown],
                    "truncated": true, "total": total, "shown": shown
                })
            } else {
                selected.clone()
            };
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
                expected
            );
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                if shown < total {
                    format!(
                        "Showing {shown} of {total}. Use --limit to see more or --type/--room to filter.\n"
                    )
                } else {
                    String::new()
                }
            );
        }
    }
    let expected = format!(
        "  {:<20} {:<30} {:<20} {:<20} UUID\n  {:<20} {:<30} {:<20} {:<20} {}\n  {:<20} {:<30} {:<20} {:<20} p\n\n3 controls\n",
        "Type",
        "Title",
        "Room",
        "Category",
        "─".repeat(20),
        "─".repeat(30),
        "─".repeat(20),
        "─".repeat(20),
        "─".repeat(36),
        "Page",
        "has 12hTF=\"x\"",
        "raw&amp;id",
        ""
    );
    for format in ["table", "csv"] {
        command()
            .args([
                "-o",
                format,
                "--no-header",
                "config",
                "controls",
                "controls.Loxone",
                "--limit",
                "1",
            ])
            .assert()
            .success()
            .stdout(expected.clone())
            .stderr("Showing 1 of 3. Use --limit to see more or --type/--room to filter.\n");
    }
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert!(!dir.path().join("home").exists());
}

#[test]
fn control_inventory_cli_reader_file_errors_and_permissive_eof() {
    let dir = tempfile::Builder::new()
        .prefix(".control-inventory-errors-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"))
            .args(["--quiet", "--dry-run"]);
        cmd
    };
    fs::write(
        dir.path().join("mismatch.Loxone"),
        "<ControlList><C></ControlList>",
    )
    .unwrap();
    for (file, message) in [
        (
            "absent.zip",
            "Expected a .Loxone XML file. Run `lox config extract absent.zip` first.",
        ),
        ("absent.ZIP", "Cannot read absent.ZIP: "),
        (
            "mismatch.Loxone",
            "ill-formed document: expected `</C>`, but `</ControlList>` was found: expected `</C>`, but `</ControlList>` was found",
        ),
    ] {
        let output = command()
            .args(["--json", "config", "controls", file])
            .assert()
            .code(1)
            .stderr("")
            .get_output()
            .clone();
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"], "error");
        let actual = value["message"].as_str().unwrap();
        if file == "absent.ZIP" {
            assert!(actual.starts_with(message), "{actual}");
        } else {
            assert_eq!(actual, message);
        }
        command()
            .args(["config", "controls", file])
            .assert()
            .code(1)
            .stdout("")
            .stderr(format!(
                "Error: {actual}\n\n{}\n",
                value["report_hint"].as_str().unwrap()
            ));
    }
    for (file, xml, expected) in [
        ("text.Loxone", "not XML", serde_json::json!([])),
        (
            "open.ZIP",
            r#"<C Type="Switch" U="u" Title=""><IoData/>"#,
            serde_json::json!([{"control_type":"Switch","title":"","uuid":"u","room":"","category":""}]),
        ),
    ] {
        fs::write(dir.path().join(file), xml).unwrap();
        let output = command()
            .args(["--json", "config", "controls", file])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .stdout
            .clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output).unwrap(),
            expected
        );
    }
}

#[test]
fn room_inventory_cli_formats_limits_dry_run_and_raw_loader_boundary() {
    let dir = tempfile::Builder::new()
        .prefix(".room-inventory-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("rooms.Loxone");
    let mut bytes = b"\xef\xbb\xbf".to_vec();
    bytes.extend_from_slice(
        br#"<Root>
<IoData Pr="z"/><IoData Pr="missing"/>
<C Type="Place" U="z" Title="Z"/><C Type="Place" U="a" Title="A&amp;&#65;"/>
<C Type="Place" U="q" Title='has 12hTF="x"'/><IoData Pr="z"/>
</Root>"#,
    );
    fs::write(&path, &bytes).unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"))
            .args(["--quiet", "--trace-id", "room-inventory"]);
        cmd
    };
    let all = serde_json::json!([
        {"uuid":"a","name":"A&amp;&#65;","item_count":0},
        {"uuid":"z","name":"Z","item_count":2},
        {"uuid":"q","name":"has 12hTF=\"x\"","item_count":0}
    ]);
    for dry_run in [false, true] {
        for limit in [None, Some(0), Some(1), Some(3), Some(8)] {
            let mut cmd = command();
            cmd.args(["--json", "config", "rooms", "rooms.Loxone"]);
            if dry_run {
                cmd.arg("--dry-run");
            }
            if let Some(limit) = limit {
                cmd.args(["--limit", &limit.to_string()]);
            }
            let result = cmd.assert().success().get_output().clone();
            let shown = limit.unwrap_or(3).min(3);
            let expected = if shown < 3 {
                serde_json::json!({
                    "rooms": &all.as_array().unwrap()[..shown],
                    "truncated": true, "total":3, "shown":shown
                })
            } else {
                all.clone()
            };
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap(),
                expected
            );
            assert_eq!(
                String::from_utf8(result.stderr).unwrap(),
                if shown < 3 {
                    format!("Showing {shown} of 3. Use --limit to see more.\n")
                } else {
                    String::new()
                }
            );
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    let expected = format!(
        "  {:<30} {:<6} UUID\n  {:<30} {:<6} {}\n  {:<30} {:<6} a\n\n3 rooms, 2 items total\n",
        "Room",
        "Items",
        "─".repeat(30),
        "─".repeat(6),
        "─".repeat(36),
        "A&amp;&#65;",
        0
    );
    for format in ["table", "csv"] {
        command()
            .args([
                "-o",
                format,
                "--no-header",
                "--dry-run",
                "config",
                "rooms",
                "rooms.Loxone",
                "--limit",
                "1",
            ])
            .assert()
            .success()
            .stdout(expected.clone())
            .stderr("Showing 1 of 3. Use --limit to see more.\n");
    }
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(!dir.path().join("home").exists());
}

#[test]
fn room_inventory_cli_errors_and_permissive_eof_are_unchanged() {
    let dir = tempfile::Builder::new()
        .prefix(".room-inventory-errors-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let command = || {
        let mut cmd = lox();
        cmd.current_dir(dir.path())
            .env("LOX_CONFIG", dir.path().join("absent.yaml"))
            .env("LOX_HOME", dir.path().join("home"))
            .args(["--quiet", "--dry-run"]);
        cmd
    };
    fs::write(
        dir.path().join("mismatch.Loxone"),
        "<ControlList><C></ControlList>",
    )
    .unwrap();
    for (file, message) in [
        (
            "absent.zip",
            "Expected a .Loxone XML file. Run `lox config extract absent.zip` first.",
        ),
        ("absent.Loxone", "Cannot read absent.Loxone"),
        ("mismatch.Loxone", "expected `</C>`"),
    ] {
        let result = command()
            .args(["--json", "config", "rooms", file])
            .assert()
            .code(1)
            .stderr("")
            .get_output()
            .clone();
        let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"], "error");
        let actual_message = value["message"].as_str().unwrap();
        assert!(actual_message.contains(message), "{actual_message}");
        match file {
            "absent.zip" => assert_eq!(actual_message, message),
            "absent.Loxone" => assert!(actual_message.starts_with("Cannot read absent.Loxone: ")),
            "mismatch.Loxone" => assert_eq!(
                actual_message,
                "ill-formed document: expected `</C>`, but `</ControlList>` was found: expected `</C>`, but `</ControlList>` was found"
            ),
            _ => unreachable!(),
        }
        command()
            .args(["config", "rooms", file])
            .assert()
            .code(1)
            .stdout("")
            .stderr(format!(
                "Error: {actual_message}\n\n{}\n",
                value["report_hint"].as_str().unwrap()
            ));
    }
    for (file, bytes, expected) in [
        ("text.Loxone", "not XML", serde_json::json!([])),
        (
            "open.ZIP",
            r#"<C Type="Place" U="r" Title="Open">"#,
            serde_json::json!([{"uuid":"r","name":"Open","item_count":0}]),
        ),
    ] {
        fs::write(dir.path().join(file), bytes).unwrap();
        let result = command()
            .args(["--json", "config", "rooms", file])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .stdout
            .clone();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&result).unwrap(),
            expected
        );
    }
}

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

// ── config user-grant / user-revoke / user-perms ────────────────────────────

/// Loxone-formatted fixture (BOM, CRLF, tab indentation) with a user, a group and
/// two virtual inputs. It round-trips byte-identically through the editor.
fn write_user_fixture() -> (TempDir, String, Vec<u8>) {
    let dir = tempfile::Builder::new()
        .prefix(".user-cli-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<ControlList Version=\"267\">\n\
\t<C Type=\"Document\" V=\"178\" U=\"doc-1\" Title=\"Test\">\n\
\t\t<C Type=\"VirtualInCaption\" V=\"178\" U=\"vic-1\" Title=\"Virtual Inputs\">\n\
\t\t\t<C Type=\"VirtualIn\" V=\"178\" U=\"vi-1-ffff504f94a010d0\" Title=\"UniFi Doorbell\" WF=\"16384\"/>\n\
\t\t\t<C Type=\"VirtualIn\" V=\"178\" U=\"vi-2-ffff504f94a010d0\" Title=\"UniFi Gate\" WF=\"16384\"/>\n\
\t\t</C>\n\
\t\t<C Type=\"UserCaption\" V=\"178\" U=\"uc-1\" Title=\"Users\">\n\
\t\t\t<C Type=\"User\" V=\"178\" U=\"user-1\" Title=\"webhook_api\" WF=\"16384\">\n\
\t\t\t\t<HP H=\"h\" Sc=\"2\" S=\"s\" Alg=\"1\"/>\n\
\t\t\t\t<PH NUM=\"1\">\n\
\t\t\t\t\t<PRM REF=\"vi-2-ffff504f94a010d0\" ACT=\"3\"/>\n\
\t\t\t\t</PH>\n\
\t\t\t</C>\n\
\t\t\t<C Type=\"UserGroup\" V=\"178\" U=\"grp-1\" Title=\"Webhooks\" WF=\"16384\" User=\"\"/>\n\
\t\t</C>\n\
\t</C>\n\
</ControlList>\n";
    let mut data = vec![0xEF, 0xBB, 0xBF];
    data.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    let path = dir.path().join("users.Loxone");
    fs::write(&path, &data).unwrap();
    (dir, path.to_str().unwrap().to_string(), data)
}

fn without_user_save_stamp(data: &[u8]) -> String {
    let mut xml = std::str::from_utf8(data).unwrap().to_string();
    let start = xml.find("<C Type=\"Document\"").unwrap();
    let end = start + xml[start..].find('>').unwrap() + 1;
    let mut document = xml[start..end].to_string();
    for key in ["Date", "DateS"] {
        let prefix = format!(" {key}=\"");
        if let Some(start) = document.find(&prefix) {
            let value = start + prefix.len();
            let end = value + document[value..].find('"').unwrap() + 1;
            document.replace_range(start..end, "");
        }
    }
    xml.replace_range(start..end, &document);
    xml
}

#[test]
fn config_user_grant_then_revoke_only_changes_save_stamp() {
    let (_dir, path, original) = write_user_fixture();

    lox()
        .args([
            "config",
            "user-grant",
            &path,
            "webhook_api",
            "UniFi Doorbell",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added permission entry for user 'webhook_api' on VirtualIn 'UniFi Doorbell'",
        ));
    let granted = fs::read_to_string(&path).unwrap();
    assert!(granted.contains(
        "<PH NUM=\"2\">\r\n\t\t\t\t\t<PRM REF=\"vi-2-ffff504f94a010d0\" ACT=\"3\"/>\r\n\t\t\t\t\t<PRM REF=\"vi-1-ffff504f94a010d0\" ACT=\"3\"/>\r\n"
    ));

    lox()
        .args([
            "config",
            "user-revoke",
            &path,
            "webhook_api",
            "UniFi Doorbell",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed permission entry for user 'webhook_api'",
        ));
    let restored = fs::read(&path).unwrap();
    assert_ne!(restored, original);
    assert_eq!(
        without_user_save_stamp(&restored),
        without_user_save_stamp(&original)
    );
}

#[test]
fn config_user_grant_is_idempotent() {
    let (_dir, path, original) = write_user_fixture();

    let output = lox()
        .args([
            "-o",
            "json",
            "config",
            "user-grant",
            &path,
            "webhook_api",
            "uuid:vi-2-ffff504f94a010d0",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["changed"], false);
    assert_eq!(json["change"], "unchanged");
    assert_eq!(json["object"]["title"], "UniFi Gate");
    assert_eq!(fs::read(&path).unwrap(), original);

    lox()
        .args([
            "config",
            "user-revoke",
            &path,
            "webhook_api",
            "UniFi Doorbell",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("no change"));
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn config_user_grant_group_and_ambiguous_object() {
    let (_dir, path, original) = write_user_fixture();

    // "UniFi" matches several elements → must be rejected, file untouched
    lox()
        .args(["config", "user-grant", &path, "webhook_api", "UniFi"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("expected 1"));
    assert_eq!(fs::read(&path).unwrap(), original);

    lox()
        .args([
            "config",
            "user-grant",
            &path,
            "Webhooks",
            "UniFi Gate",
            "--group",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("user group 'Webhooks'"));
    lox()
        .args([
            "config",
            "user-revoke",
            &path,
            "Webhooks",
            "UniFi Gate",
            "--group",
        ])
        .assert()
        .success();
    assert_eq!(
        without_user_save_stamp(&fs::read(&path).unwrap()),
        without_user_save_stamp(&original)
    );
}

#[test]
fn config_user_perms_and_users_list_permissions() {
    let (_dir, path, _) = write_user_fixture();

    lox()
        .args(["config", "user-perms", &path, "webhook_api"])
        .assert()
        .success()
        .stdout(predicate::str::contains("UniFi Gate"))
        .stdout(predicate::str::contains("VirtualIn"))
        .stdout(predicate::str::contains("1 direct permission(s)"));

    let output = lox()
        .args(["-o", "json", "config", "users", &path])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let perms = &json[0]["permissions"];
    assert_eq!(perms[0]["uuid"], "vi-2-ffff504f94a010d0");
    assert_eq!(perms[0]["title"], "UniFi Gate");
    assert_eq!(perms[0]["type"], "VirtualIn");
}

#[test]
fn config_user_perms_preserves_head_output_and_refusals() {
    let dir = tempfile::Builder::new()
        .prefix(".selected-permissions-cli-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let file = dir.path().join("selected.Loxone");
    let path = file.to_str().unwrap();
    let xml = br#"<Root>
<C Type="User" Title="Sample" U="User-01"/>
<C Type="User" Title="SAMPLE"/>
<C Type="UserGroup" Title="Sample" U="Group-01"><PH>
    <PRM REF="Object" ACT="0" TIM="Schedule"/><PRM/>
</PH></C>
<C Type="Place" U="Object" Title="A &quot;room&quot;"/>
</Root>"#;
    fs::write(&file, xml).unwrap();
    let output = lox()
        .args([
            "-o",
            "json",
            "config",
            "user-perms",
            path,
            "sample",
            "--group",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "kind": "UserGroup", "title": "Sample", "uuid": "Group-01",
            "permissions": [
                {"uuid": "Object", "title": "A \"room\"", "type": "Place", "act": "0", "tim": "Schedule"},
                {"uuid": "", "title": null, "type": null, "act": null},
            ],
        })
    );
    lox()
        .args(["-o", "csv", "config", "user-perms", path, "sample", "--group"])
        .assert()
        .success()
        .stdout("TITLE,TYPE,ACT,UUID\n\"A \"\"room\"\"\",\"Place\",\"0\",\"Object\"\n\"\",\"\",\"\",\"\"\n");

    for (query, expected) in [
        (
            "sample",
            "User name 'sample' matches 2 elements (expected 1):\n  uuid:User-01\n  uuid:\nUse uuid:<uuid> to pick one.",
        ),
        (
            "Sampl",
            "User 'Sampl' not found. Did you mean: SAMPLE, Sample?\n  Available: SAMPLE, Sample",
        ),
    ] {
        lox()
            .args(["config", "user-perms", path, query])
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected));
    }
    assert_eq!(fs::read(&file).unwrap().as_slice(), xml.as_slice());
}

#[test]
fn config_user_add_virtual_input_with_grant() {
    let (_dir, path, _) = write_user_fixture();

    let output = lox()
        .args([
            "-o",
            "json",
            "config",
            "add",
            &path,
            "--type",
            "virtual-input",
            "--title",
            "UniFi Webhook",
            "--grant",
            "webhook_api",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let uuid = json["virtual_in_uuid"].as_str().unwrap().to_string();
    assert_eq!(json["granted"]["title"], "webhook_api");

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("<PH NUM=\"2\">"));
    assert!(content.contains(&format!("<PRM REF=\"{uuid}\" ACT=\"3\"/>")));
    assert!(content.contains(&format!(
        "Type=\"VirtualIn\" IName=\"VI0\" V=\"175\" U=\"{uuid}\""
    )));
}

#[test]
fn config_user_permission_writes_respect_quiet_and_save_failures() {
    let (dir, path, original) = write_user_fixture();
    for action in ["user-grant", "user-revoke"] {
        lox()
            .args([
                "--quiet",
                "config",
                action,
                &path,
                "webhook_api",
                "UniFi Doorbell",
            ])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
    let before_failed_save = fs::read(&path).unwrap();
    assert_eq!(
        without_user_save_stamp(&before_failed_save),
        without_user_save_stamp(&original)
    );

    let invalid_output = dir.path().join("missing").join("out.Loxone");
    let output = lox()
        .args([
            "-o",
            "json",
            "config",
            "user-grant",
            &path,
            "webhook_api",
            "UniFi Doorbell",
            "--save-as",
            invalid_output.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(fs::read(&path).unwrap(), before_failed_save);
}

#[test]
fn config_user_add_virtual_input_grant_is_atomic_and_quiet() {
    for dedicated in [false, true] {
        let (_dir, path, original) = write_user_fixture();
        let args = if dedicated {
            vec!["config", "add-virtual-in", &path, "Webhook"]
        } else {
            vec![
                "config",
                "add",
                &path,
                "--type",
                "virtual-input",
                "--title",
                "Webhook",
            ]
        };
        lox()
            .args(&args)
            .args(["--grant", "missing-user"])
            .assert()
            .failure();
        assert_eq!(fs::read(&path).unwrap(), original);
        lox()
            .args(["--quiet"])
            .args(&args)
            .args(["--grant", "webhook_api"])
            .assert()
            .success()
            .stdout("")
            .stderr("");
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Title=\"Webhook\""));
        assert!(content.contains("<PH NUM=\"2\">"));
    }
}

#[test]
fn config_virtual_input_creators_keep_grant_and_aq_identities_distinct_with_or_without_page() {
    for (command, with_page) in [
        ("add", true),
        ("add-virtual-in", true),
        ("add", false),
        ("add-virtual-in", false),
    ] {
        let (dir, path, original) = write_user_fixture();
        let original = if with_page {
            let xml = String::from_utf8(original).unwrap().replace(
                "\t</C>\r\n</ControlList>",
                "\t\t<C Type=\"Page\" U=\"page-1\"/>\r\n\t</C>\r\n</ControlList>",
            );
            fs::write(&path, &xml).unwrap();
            xml.into_bytes()
        } else {
            original
        };
        let output = dir.path().join("created.Loxone");
        let args = if command == "add" {
            vec![
                "config",
                command,
                &path,
                "--type",
                "virtual-input",
                "--title",
                "New input",
            ]
        } else {
            vec!["config", command, &path, "New input"]
        };
        lox()
            .args(&args)
            .args(["--grant", "missing-user", "--save-as"])
            .arg(&output)
            .assert()
            .failure();
        assert!(!output.exists());
        assert_eq!(fs::read(&path).unwrap(), original);

        let report = lox()
            .arg("--json")
            .args(&args)
            .args(["--grant", "webhook_api", "--save-as"])
            .arg(&output)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&report).unwrap();
        let object = json["virtual_in_uuid"].as_str().unwrap();
        let aq = json["uuid"].as_str().unwrap();
        assert_ne!(object, aq);
        if command == "add-virtual-in" {
            assert_eq!(json["connector_uuid"], aq);
        }
        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains(&format!(r#"<PRM REF="{object}" ACT="3"/>"#)));
        assert_eq!(content.contains("Type=\"InputRef\""), with_page);
        if with_page {
            assert!(content.contains(&format!(r#"K="AQ" U="{aq}""#)));
        } else {
            assert!(!content.contains(aq));
        }
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}

#[test]
fn config_user_perms_csv_quotes_fields_and_respects_no_header() {
    let (_dir, path, _) = write_user_fixture();
    let content = fs::read_to_string(&path)
        .unwrap()
        .replace("UniFi Gate", "Gate, &quot;North&quot;");
    fs::write(&path, content).unwrap();
    for no_header in [false, true] {
        let mut command = lox();
        command.args(["-o", "csv", "config", "user-perms", &path, "webhook_api"]);
        if no_header {
            command.arg("--no-header");
        }
        let header = if no_header {
            ""
        } else {
            "TITLE,TYPE,ACT,UUID\n"
        };
        command.assert().success().stdout(format!(
            "{header}\"Gate, \"\"North\"\"\",\"VirtualIn\",\"3\",\"vi-2-ffff504f94a010d0\"\n"
        ));
    }
}
