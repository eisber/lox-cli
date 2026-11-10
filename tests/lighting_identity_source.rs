use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

const XML: &str = r#"<ControlList><C Type="LightController2" U="lighting">
  <LightscenesC Num="99" Outputs="0002" Future="retain">
    <LightsceneC Name="First" UUID="first" SID="2" CID="9" Outputs="2"
      Q1="opaque-selected" Q2="1610612778"><SeqConf Future="retain"/></LightsceneC>
    <LightsceneC Name="Second" UUID="second" SID="7" CID="15" Outputs="1" Q1="0"/>
  </LightscenesC>
</C></ControlList>
"#;

fn fixture_bytes(xml: &str) -> Vec<u8> {
    let mut bytes = vec![0xef, 0xbb, 0xbf];
    bytes.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
    bytes
}

fn add_mood(input: &std::path::Path) -> assert_cmd::Command {
    let mut command = cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .env("LOX_CONFIG", input.with_extension("absent.yaml"))
        .args(["config", "add-mood"])
        .arg(input)
        .args([
            "uuid:lighting",
            "--name",
            "New",
            "--color",
            "hsv(120,100,10)",
            "-o",
            "json",
            "--quiet",
        ]);
    command
}

#[test]
fn lighting_identity_source_cli_overflow_is_an_atomic_error_not_a_panic_or_wrap() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-identity-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (field, original) in [("SID", "7"), ("CID", "15")] {
        let xml = XML.replace(
            &format!("{field}=\"{original}\""),
            &format!("{field}=\"{}\"", i64::MAX),
        );
        let before = fixture_bytes(&xml);
        for save_as in [false, true] {
            fs::write(&input, &before).unwrap();
            fs::write(&output, b"existing destination").unwrap();
            let mut command = add_mood(&input);
            command
                .arg(if field == "SID" { "--cid" } else { "--sid" })
                .arg("100");
            if save_as {
                command.arg("--save-as").arg(&output);
            }
            let result = command.assert().failure().get_output().clone();
            let error: serde_json::Value = serde_json::from_slice(&result.stdout)
                .unwrap_or_else(|_| panic!("expected diagnostic, not panic: {result:?}"));
            assert_eq!(error["ok"], false);
            assert_eq!(error["error"], "mood_identity_exhausted");
            assert!(String::from_utf8_lossy(&result.stdout).contains(field));
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing destination");
        }
    }
}

fn xml_with_scenes(scenes: &str) -> String {
    format!(
        r#"<ControlList><C Type="LightController2" U="lighting"><LightscenesC Num="99" Outputs="0002" Future="retain">{scenes}</LightscenesC></C></ControlList>"#
    )
}

#[test]
fn lighting_identity_source_cli_native_defaults_fill_gaps_without_searching_other_cids() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let occupied = (1..=8)
        .map(|sid| format!(r#"<LightsceneC SID="{sid}" CID="opaque-unused" Name="Old{sid}"/>"#))
        .collect::<String>();
    for (scenes, sid, cid) in [
        ("", 1, 12),
        (
            r#"<LightsceneC SID="777" CID="22"/><LightsceneC SID="778" CID="11"/>"#,
            1,
            12,
        ),
        (
            r#"<LightsceneC SID="1" CID="12"/><LightsceneC SID="3" CID="14"/>"#,
            2,
            13,
        ),
        (
            r#"<LightsceneC SID="1" CID="12"/><LightsceneC SID="50" CID="13"/>"#,
            2,
            0,
        ),
        (occupied.as_str(), 9, 0),
        (
            r#"<Unknown><LightsceneC SID="1" CID="12" Name="Nested"/></Unknown><LightsceneC Name="Missing IDs"/>"#,
            1,
            12,
        ),
    ] {
        let xml = xml_with_scenes(scenes);
        fs::write(&input, fixture_bytes(&xml)).unwrap();
        add_mood(&input).assert().success();
        let bytes = fs::read(&input).unwrap();
        let actual = xmltree::Element::parse(&bytes[3..]).unwrap();
        let collection = actual
            .get_child("C")
            .unwrap()
            .get_child("LightscenesC")
            .unwrap();
        let original = xmltree::Element::parse(xml.as_bytes()).unwrap();
        let old_collection = original
            .get_child("C")
            .unwrap()
            .get_child("LightscenesC")
            .unwrap();
        assert_eq!(
            &collection.children[..old_collection.children.len()],
            &old_collection.children
        );
        assert_eq!(collection.attributes["Outputs"], "0002");
        let new = collection.children.last().unwrap().as_element().unwrap();
        assert_eq!(new.attributes["SID"], sid.to_string(), "{scenes}");
        assert_eq!(new.attributes["CID"], cid.to_string(), "{scenes}");
        assert_eq!(new.attributes["Q1"], "1610622736");
    }
}

#[test]
fn lighting_identity_source_cli_exhaustion_and_unsupported_identities_never_write() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let occupied = (1..=99)
        .map(|sid| format!(r#"<LightsceneC SID="{sid}" CID="0"/>"#))
        .collect::<String>();
    for (xml, code) in [
        (xml_with_scenes(&occupied), "mood_identity_exhausted"),
        (
            XML.replace("SID=\"7\"", "SID=\"opaque\""),
            "unsupported_mood_identity_value",
        ),
        (
            XML.replace("SID=\"7\"", "SID=\"2147483648\""),
            "unsupported_mood_identity_value",
        ),
        (
            XML.replace("CID=\"15\"", "CID=\"opaque\""),
            "unsupported_mood_identity_value",
        ),
        (
            XML.replace("CID=\"15\"", "CID=\"2147483648\""),
            "unsupported_mood_identity_value",
        ),
    ] {
        let before = fixture_bytes(&xml);
        for save_as in [false, true] {
            fs::write(&input, &before).unwrap();
            fs::write(&output, b"existing destination").unwrap();
            let mut command = add_mood(&input);
            if save_as {
                command.arg("--save-as").arg(&output);
            }
            let result = command.assert().failure().get_output().clone();
            let error: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(error["error"], code);
            assert_eq!(fs::read(&input).unwrap(), before);
            assert_eq!(fs::read(&output).unwrap(), b"existing destination");
        }
    }
}

#[test]
fn lighting_identity_source_cli_one_override_keeps_legacy_partner_allocation() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.Loxone");
    for (flag, value, sid, cid) in [("--sid", "100", 100, 16), ("--cid", "9", 8, 9)] {
        fs::write(&input, fixture_bytes(XML)).unwrap();
        add_mood(&input).args([flag, value]).assert().success();
        let bytes = fs::read(&input).unwrap();
        let document = xmltree::Element::parse(&bytes[3..]).unwrap();
        let scene = document
            .get_child("C")
            .unwrap()
            .get_child("LightscenesC")
            .unwrap()
            .children
            .last()
            .unwrap()
            .as_element()
            .unwrap();
        assert_eq!(scene.attributes["SID"], sid.to_string());
        assert_eq!(scene.attributes["CID"], cid.to_string());
    }
}

#[test]
fn lighting_identity_source_cli_overrides_keep_existing_ids_and_output_defaults() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-identity-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let xml = XML
        .replace("SID=\"7\"", &format!("SID=\"{}\"", i64::MAX))
        .replace("CID=\"15\"", &format!("CID=\"{}\"", i64::MAX));
    let before = fixture_bytes(&xml);
    fs::write(&input, &before).unwrap();
    add_mood(&input)
        .args(["--sid", "100", "--cid", "9", "--save-as"])
        .arg(&output)
        .assert()
        .success();
    assert_eq!(fs::read(&input).unwrap(), before);
    let after = fs::read(&output).unwrap();
    assert!(after.starts_with(&[0xef, 0xbb, 0xbf]));
    let text = std::str::from_utf8(&after[3..]).unwrap();
    assert!(!text.replace("\r\n", "").contains('\n'));
    let actual = xmltree::Element::parse(&after[3..]).unwrap();
    let collection = actual
        .get_child("C")
        .unwrap()
        .get_child("LightscenesC")
        .unwrap();
    let scenes: Vec<_> = collection
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .collect();
    let original = xmltree::Element::parse(xml.as_bytes()).unwrap();
    let old_scenes: Vec<_> = original
        .get_child("C")
        .unwrap()
        .get_child("LightscenesC")
        .unwrap()
        .children
        .iter()
        .filter_map(|node| node.as_element())
        .collect();
    assert_eq!(&scenes[..2], &old_scenes);
    assert_eq!(collection.attributes["Outputs"], "0002");
    assert_eq!(collection.attributes["Num"], "3");
    assert_eq!(scenes[2].attributes["SID"], "100");
    assert_eq!(scenes[2].attributes["CID"], "9");
    assert_eq!(scenes[2].attributes["Outputs"], "18");
    assert_eq!(scenes[2].attributes["Q1"], "1610622736");
    assert_eq!(scenes[2].attributes["Q2"], "1610612736");
    assert_eq!(scenes[2].attributes["Q18"], "0");
}

#[test]
fn lighting_identity_source_cli_duplicate_sid_remains_an_atomic_error() {
    let dir = tempfile::Builder::new()
        .prefix(".lighting-identity-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let before = fixture_bytes(XML);
    fs::write(&input, &before).unwrap();
    fs::write(&output, b"existing destination").unwrap();
    let result = add_mood(&input)
        .args(["--sid", "2", "--cid", "9", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(String::from_utf8_lossy(&result.stdout).contains("SID 2 is already in use"));
    assert_eq!(fs::read(&input).unwrap(), before);
    assert_eq!(fs::read(&output).unwrap(), b"existing destination");
}
