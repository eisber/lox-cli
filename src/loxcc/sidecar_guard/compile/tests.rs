//! Sidecar compilation against synthetic native layouts.
//!
//! `fixtures/` holds an invented V178-style project and catalogue. The
//! permission records below mirror the native layouts the compiler verifies:
//! selector (11 tokens), status control (8 tokens) and energy meter (16 tokens).

use super::{display_format, native_uuid};
use crate::loxcc::{
    compress_loxcc, extract_and_decompress, repack_zip, validate_archive_deployment,
};
use serde_json::{Value, json};
use std::io::{Cursor, Read, Write};

const PROJECT: &str = include_str!("fixtures/project.xml");
const CATALOGUE: &str = include_str!("fixtures/catalogue.json");

const DATE: &str = "Date=\"2026-01-01 00:00:00\"";
const NEW_DATE: &str = "Date=\"2026-02-01 00:00:00\"";
const PAGE_END: &str = "  </C>\n</ControlList>";
const ROOM: &str = "1b000000-0000-0001-ffff000000000000";
const CATEGORY: &str = "1c000000-0000-0001-ffff000000000000";
const ROOM_RIGHTS: &str = "1a000000-0000-0001-ffff000000000000";
const CATEGORY_RIGHTS: &str = "1a000000-0000-0002-ffff000000000000";

const NEW_SELECTOR: &str = r#"    <C Type="PushButton2Sel" V="178" U="2b000000-0000-0001-ffff000000000000" Title="New level">
      <Co K="Reset" U="2b000000-0000-0002-ffff000000000000"/>
      <Co K="Min" U="2b000000-0000-0003-ffff000000000000"/>
      <Co K="Max" U="2b000000-0000-0004-ffff000000000000"/>
      <Co K="Step" U="2b000000-0000-0005-ffff000000000000"/>
      <Co K="AQ" U="2b000000-0000-0006-ffff000000000000"/>
      <IoData Visu="true" Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/>
      <Display Unit="Level &lt;v.2&gt; %"/>
    </C>
"#;
const NEW_STATUS: &str = r#"    <C Type="State" V="178" U="3b000000-0000-0001-ffff000000000000" Title="New status" linkC="4b000000-0000-0001-ffff000000000000">
      <Co K="I1" U="3b000000-0000-0002-ffff000000000000"/>
      <IoData Visu="true" Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/>
    </C>
    <C Type="PButtonT" V="178" U="4b000000-0000-0001-ffff000000000000" Title="New button">
      <Co K="Reset" U="4b000000-0000-0002-ffff000000000000"/>
      <Co K="Q" U="4b000000-0000-0003-ffff000000000000"/>
      <IoData Visu="true" Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/>
    </C>
"#;
const NEW_METER: &str = r#"    <C Type="MeterAbsUni" V="178" U="6b000000-0000-0001-ffff000000000000" Title="New meter" Rec="5a000000-0000-0001-ffff000000000000" PwrN="3" Un1="W" Un2="Wh" StatsType="1">
      <Co K="R" U="6b000000-0000-0002-ffff000000000000"/>
      <Co K="OPf" U="6b000000-0000-0003-ffff000000000000"/>
      <Co K="OMr" U="6b000000-0000-0004-ffff000000000000"/>
      <Co K="ORd" U="6b000000-0000-0005-ffff000000000000"/>
      <Co K="ORw" U="6b000000-0000-0006-ffff000000000000"/>
      <Co K="ORm" U="6b000000-0000-0007-ffff000000000000"/>
      <Co K="ORy" U="6b000000-0000-0008-ffff000000000000"/>
      <IoData Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/>
      <statGrp T="1" Def="1"/>
    </C>
"#;
const EXISTING_NODE: &str = r#"<nd u="5b000000-0000-0002-ffff000000000000" tit="Existing meter" t="1" ctrlUuid="6a000000-0000-0001-ffff000000000000"/>"#;
const NEW_NODE: &str = r#"<nd u="5b000000-0000-0003-ffff000000000000" tit="New meter" t="1" ctrlUuid="6b000000-0000-0001-ffff000000000000"/>"#;

/// Independent encoding of a Loxone UUID as stored in `permissions.bin`.
fn token(uuid: &str) -> Vec<u8> {
    let hex: String = uuid.chars().filter(|c| *c != '-').collect();
    let bytes: Vec<u8> = (0..16)
        .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
        .collect();
    [3, 2, 1, 0, 5, 4, 7, 6, 8, 9, 10, 11, 12, 13, 14, 15]
        .iter()
        .map(|i| bytes[*i])
        .collect()
}

fn record(action: &str, states: &[&str], groups: &[&[&str]]) -> Vec<u8> {
    let mut record = token(action);
    record.extend([0xfe; 16]);
    for state in states {
        record.extend(token(state));
    }
    record.extend([0xfc; 16]);
    for (index, owners) in groups.iter().enumerate() {
        if index > 0 {
            record.extend([0xfb; 16]);
        }
        for owner in *owners {
            record.extend(token(owner));
        }
    }
    record.extend([0xff; 16]);
    record
}

fn uuid(prefix: &str, index: u8) -> String {
    format!("{prefix}000000-0000-{index:04x}-ffff000000000000")
}

fn rights() -> [&'static str; 2] {
    [ROOM_RIGHTS, CATEGORY_RIGHTS]
}

fn source_permissions() -> Vec<u8> {
    let [room, category] = rights();
    let mut data = record(
        &uuid("2a", 1),
        &[
            "aa000000-0000-0002-ffff000000000000",
            &uuid("2a", 3),
            &uuid("2a", 4),
            &uuid("2a", 5),
            &uuid("2a", 6),
        ],
        &[&[room, category]],
    );
    data.extend(record(
        &uuid("3a", 1),
        &[&uuid("3a", 1), &uuid("3a", 2)],
        &[&[room, category]],
    ));
    data.extend(record(
        &uuid("4a", 1),
        &["ca000000-0000-0002-ffff000000000000", &uuid("4a", 3)],
        &[&[room, category]],
    ));
    data.extend(meter_record("6a"));
    data
}

fn meter_record(prefix: &str) -> Vec<u8> {
    let [room, category] = rights();
    let locked = format!(
        "{}{}",
        if prefix == "6a" { "ea" } else { "eb" },
        &uuid(prefix, 2)[2..]
    );
    let states: Vec<String> = std::iter::once(locked)
        .chain((3..=8).map(|index| uuid(prefix, index)))
        .collect();
    let states: Vec<&str> = states.iter().map(String::as_str).collect();
    record(
        &uuid(prefix, 1),
        &states,
        &[&[room, category], &[room, category]],
    )
}

fn archive(xml: &str, app: &str, permissions: &[u8]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .last_modified_time(zip::DateTime::from_date_and_time(2026, 1, 1, 0, 0, 0).unwrap())
        .unix_permissions(0o640);
    writer.start_file("sps0.LoxCC", options).unwrap();
    writer.write_all(&compress_loxcc(xml.as_bytes())).unwrap();
    writer.start_file("LoxAPP3.json", options).unwrap();
    writer.write_all(app.as_bytes()).unwrap();
    writer.start_file("permissions.bin", options).unwrap();
    writer.write_all(permissions).unwrap();
    writer.start_file("unrelated.bin", options).unwrap();
    writer.write_all(b"\x00unchanged").unwrap();
    writer.finish().unwrap().into_inner()
}

fn source() -> Vec<u8> {
    archive(PROJECT, CATALOGUE, &source_permissions())
}

fn member(archive: &[u8], name: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(archive)).unwrap();
    let mut data = Vec::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_end(&mut data)
        .unwrap();
    data
}

fn app(archive: &[u8]) -> Value {
    serde_json::from_slice(&member(archive, "LoxAPP3.json")).unwrap()
}

/// The source project with `additions` appended to its page and a newer save date.
fn edited(additions: &str) -> String {
    PROJECT
        .replace(DATE, NEW_DATE)
        .replace(PAGE_END, &format!("{additions}{PAGE_END}"))
}

fn deploy(xml: &str) -> Vec<u8> {
    let repacked = repack_zip(&source(), xml.as_bytes()).unwrap();
    assert_eq!(extract_and_decompress(&repacked).unwrap(), xml.as_bytes());
    assert_eq!(member(&repacked, "unrelated.bin"), b"\x00unchanged");
    // A raw upload must carry exactly the sidecars lox compiles.
    validate_archive_deployment(&source(), &repacked, xml.as_bytes()).unwrap();
    // Recompiling the deployed archive is a no-op.
    assert_eq!(repack_zip(&repacked, xml.as_bytes()).unwrap(), repacked);
    repacked
}

fn rejection(xml: &str) -> String {
    rejection_from(&source(), xml)
}

fn rejection_from(source: &[u8], xml: &str) -> String {
    let error = repack_zip(source, xml.as_bytes()).unwrap_err().to_string();
    assert!(error.contains("Loxone Config"), "{error}");
    assert!(error.contains("--force cannot bypass"), "{error}");
    error
}

/// Every source entry except `changed` is untouched and keeps its position.
fn assert_preserved(source: &Value, result: &Value, changed: &[&str]) {
    let source = source["controls"].as_object().unwrap();
    let result = result["controls"].as_object().unwrap();
    let keys: Vec<&String> = result.keys().take(source.len()).collect();
    assert_eq!(keys, source.keys().collect::<Vec<_>>());
    for (key, entry) in source {
        if !changed.contains(&key.as_str()) {
            assert_eq!(&result[key], entry, "{key}");
        }
    }
}

#[test]
fn native_uuid_matches_permission_encoding() {
    let uuid = "12345678-1234-1234-1234567812345678";
    assert_eq!(
        native_uuid(uuid).unwrap().to_vec(),
        [
            0x78, 0x56, 0x34, 0x12, 0x34, 0x12, 0x34, 0x12, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34,
            0x56, 0x78
        ]
    );
    assert_eq!(native_uuid(uuid).unwrap().to_vec(), token(uuid));
    for invalid in [
        "selector",
        "12345678-1234-1234-123456781234567g",
        "1234567-81234-1234-1234567812345678",
    ] {
        assert!(native_uuid(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn display_units_convert_to_catalogue_formats() {
    let format = |unit: &str| {
        let xml = format!(r#"<C><Display Unit="{unit}"/></C>"#);
        display_format(&xmltree::Element::parse(xml.as_bytes()).unwrap())
    };
    assert_eq!(format("&lt;v&gt;").unwrap(), "%.0f");
    assert_eq!(format("&lt;v.1&gt; %").unwrap(), "%.1f %%");
    assert_eq!(format("Level &lt;v.6&gt;°C").unwrap(), "Level %.6f°C");
    for unsupported in ["&lt;v.7&gt;", "&lt;v.1&gt;&lt;v&gt;", "plain", "&lt;x&gt;"] {
        assert!(format(unsupported).is_err(), "{unsupported}");
    }
    let without = xmltree::Element::parse(b"<C/>".as_slice()).unwrap();
    assert_eq!(display_format(&without).unwrap(), "%.0f");
}

#[test]
fn new_visualized_selector_copies_the_native_layout() {
    let repacked = deploy(&edited(NEW_SELECTOR));
    let source: Value = serde_json::from_str(CATALOGUE).unwrap();
    let result = app(&repacked);
    assert_preserved(&source, &result, &[]);
    assert_eq!(result["lastModified"], "2026-02-01 00:00:00");
    assert_eq!(result["msInfo"], source["msInfo"]);
    let entry = &result["controls"]["U:2b000000-0000-0001-ffff000000000000"];
    assert_eq!(
        entry,
        &json!({
            "name": "New level",
            "type": "ValueSelector",
            "uuidAction": uuid("2b", 1),
            "room": ROOM,
            "cat": CATEGORY,
            "defaultRating": 0,
            "isFavorite": false,
            "isSecured": false,
            "defaultIcon": "category.svg",
            "restrictions": 0,
            "details": {"increaseOnly": false, "format": "Level %.2f %%"},
            "states": {
                "jLocked": "ab000000-0000-0002-ffff000000000000",
                "min": uuid("2b", 3),
                "max": uuid("2b", 4),
                "step": uuid("2b", 5),
                "value": uuid("2b", 6),
            },
        })
    );
    // Template keys keep their native order; native template links are not copied.
    assert_eq!(
        entry.as_object().unwrap().keys().collect::<Vec<_>>(),
        source["controls"]["U:2a000000-0000-0001-ffff000000000000"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|key| *key != "links")
            .collect::<Vec<_>>()
    );
    let [room, category] = rights();
    let mut expected = source_permissions();
    expected.extend(record(
        &uuid("2b", 1),
        &[
            "ab000000-0000-0002-ffff000000000000",
            &uuid("2b", 3),
            &uuid("2b", 4),
            &uuid("2b", 5),
            &uuid("2b", 6),
        ],
        &[&[room, category]],
    ));
    assert_eq!(member(&repacked, "permissions.bin"), expected);
    assert_eq!(member(&repacked, "LoxAPP3.json").last(), Some(&b'\n'));
}

#[test]
fn renames_update_only_the_catalogue_name() {
    let xml = PROJECT
        .replace(DATE, NEW_DATE)
        .replace("Title=\"Existing level\"", "Title=\"Renamed level\"")
        .replace("Title=\"Existing status\"", "Title=\"Renamed status\"")
        .replace("Title=\"Existing button\"", "Title=\"Renamed button\"")
        .replace(
            "Title=\"Existing meter\" Rec=",
            "Title=\"Renamed meter\" Rec=",
        );
    let repacked = deploy(&xml);
    let source: Value = serde_json::from_str(CATALOGUE).unwrap();
    let result = app(&repacked);
    let mut expected = source.clone();
    expected["lastModified"] = json!("2026-02-01 00:00:00");
    for (prefix, name) in [
        ("2a", "Renamed level"),
        ("3a", "Renamed status"),
        ("4a", "Renamed button"),
        ("6a", "Renamed meter"),
    ] {
        expected["controls"][format!("U:{}", uuid(prefix, 1))]["name"] = json!(name);
    }
    assert_eq!(result, expected);
    // Native key order and every unrenamed field survive.
    assert_eq!(
        serde_json::to_string(&result).unwrap(),
        serde_json::to_string(&expected).unwrap()
    );
    // Renames need no permission changes, so the native file is copied verbatim.
    assert_eq!(member(&repacked, "permissions.bin"), source_permissions());
}

#[test]
fn new_status_controls_and_meter_extend_catalogue_permissions_and_energy_tree() {
    let xml = edited(&format!("{NEW_STATUS}{NEW_METER}")).replace(
        EXISTING_NODE,
        &format!("{EXISTING_NODE}\n          {NEW_NODE}"),
    );
    let repacked = deploy(&xml);
    let source: Value = serde_json::from_str(CATALOGUE).unwrap();
    let result = app(&repacked);
    assert_preserved(&source, &result, &["U:5a000000-0000-0001-ffff000000000000"]);
    let controls = result["controls"].as_object().unwrap();
    // Meters render before status controls, each in document order.
    assert_eq!(
        controls
            .keys()
            .skip(source["controls"].as_object().unwrap().len())
            .collect::<Vec<_>>(),
        [
            "U:6b000000-0000-0001-ffff000000000000",
            "U:3b000000-0000-0001-ffff000000000000",
            "U:4b000000-0000-0001-ffff000000000000"
        ]
    );
    let meter = &controls["U:6b000000-0000-0001-ffff000000000000"];
    let template = &controls["U:6a000000-0000-0001-ffff000000000000"];
    assert_eq!(meter["name"], "New meter");
    assert_eq!(meter["restrictions"], 17);
    assert_eq!(meter["details"], template["details"]);
    assert_eq!(meter["statisticV2"], template["statisticV2"]);
    assert_eq!(meter["defaultIcon"], "category.svg");
    assert_eq!(
        meter["states"]["jLocked"],
        "eb000000-0000-0002-ffff000000000000"
    );
    assert_eq!(meter["states"]["totalYear"], uuid("6b", 8));
    let status = &controls["U:3b000000-0000-0001-ffff000000000000"];
    assert_eq!(
        status,
        &json!({
            "name": "New status",
            "type": "TextState",
            "uuidAction": uuid("3b", 1),
            "room": ROOM,
            "cat": CATEGORY,
            "defaultRating": 0,
            "isFavorite": false,
            "isSecured": false,
            "defaultIcon": "category.svg",
            "restrictions": 0,
            "states": {"textAndIcon": uuid("3b", 1), "iconAndColor": uuid("3b", 2)},
            "links": [uuid("4b", 1)],
        })
    );
    let button = &controls["U:4b000000-0000-0001-ffff000000000000"];
    assert_eq!(button["type"], "Pushbutton");
    assert_eq!(button["details"], json!({"jLockable": true}));
    assert_eq!(
        button["states"],
        json!({"jLocked": "cb000000-0000-0002-ffff000000000000", "active": uuid("4b", 3)})
    );
    let group = &controls["U:5a000000-0000-0001-ffff000000000000"]["details"]["nodes"][0];
    assert_eq!(
        group["nodes"][0],
        source["controls"]["U:5a000000-0000-0001-ffff000000000000"]["details"]["nodes"][0]["nodes"]
            [0]
    );
    assert_eq!(
        group["nodes"][1],
        json!({
            "uuid": "5b000000-0000-0003-ffff000000000000",
            "title": "New meter",
            "icon": "category.svg",
            "ctrlUuid": uuid("6b", 1),
            "nodeType": "Load",
        })
    );
    let [room, category] = rights();
    let mut expected = source_permissions();
    expected.extend(meter_record("6b"));
    expected.extend(record(
        &uuid("3b", 1),
        &[&uuid("3b", 1), &uuid("3b", 2)],
        &[&[room, category]],
    ));
    expected.extend(record(
        &uuid("4b", 1),
        &["cb000000-0000-0002-ffff000000000000", &uuid("4b", 3)],
        &[&[room, category]],
    ));
    assert_eq!(member(&repacked, "permissions.bin"), expected);
}

#[test]
fn intercom_labels_and_links_refresh_without_losing_native_settings() {
    let xml = PROJECT.replace(DATE, NEW_DATE).replace(
        "OutName1=\"Lock\" OutName2=\"Unlock\" OutName3=\"Open\"",
        "OutName1=\"Bolt\" OutName2=\"Release\" OutName3=\"Latch\" linkC=\"3a000000-0000-0001-ffff000000000000\"",
    );
    let repacked = deploy(&xml);
    let result = app(&repacked);
    let door = &result["controls"]["U:7a000000-0000-0001-ffff000000000000"];
    assert_eq!(
        door["details"]["outputs"],
        json!([{"id": 0, "name": "Bolt"}, {"id": 1, "name": "Release"}, {"id": 2, "name": "Latch"}])
    );
    assert_eq!(
        door["details"]["videoInfo"]["url"],
        "https://camera.example/video"
    );
    assert_eq!(
        door["subControls"]["7a000000-0000-0001-ffff000000000000/3"]["name"],
        "Latch"
    );
    assert_eq!(door["links"], json!([uuid("3a", 1)]));
    assert_eq!(member(&repacked, "permissions.bin"), source_permissions());
}

#[test]
fn new_runtime_only_logic_needs_no_sidecar_changes() {
    let xml = edited(
        r#"    <C Type="Or" U="8b000000-0000-0001-ffff000000000000" Title="New gate">
      <Co K="I1" U="8b000000-0000-0002-ffff000000000000"><In Input="2a000000-0000-0006-ffff000000000000"/></Co>
    </C>
"#,
    );
    let repacked = deploy(&xml);
    assert_eq!(member(&repacked, "LoxAPP3.json"), CATALOGUE.as_bytes());
    assert_eq!(member(&repacked, "permissions.bin"), source_permissions());
    let removed = PROJECT.replace(
        "    <C Type=\"And\" U=\"8a000000-0000-0001-ffff000000000000\" Title=\"Gate\">\n      <Co K=\"I1\" U=\"8a000000-0000-0002-ffff000000000000\"/>\n    </C>\n",
        "",
    );
    assert_ne!(removed, PROJECT);
    assert!(repack_zip(&source(), removed.as_bytes()).is_ok());
}

#[test]
fn raw_uploads_must_carry_exactly_the_compiled_sidecars() {
    let xml = edited(NEW_SELECTOR);
    let compiled = repack_zip(&source(), xml.as_bytes()).unwrap();
    let stale = repack_zip(&source(), PROJECT.as_bytes()).unwrap();
    let stale = {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        writer.start_file("sps0.LoxCC", options).unwrap();
        writer.write_all(&compress_loxcc(xml.as_bytes())).unwrap();
        for name in ["LoxAPP3.json", "permissions.bin", "unrelated.bin"] {
            writer.start_file(name, options).unwrap();
            writer.write_all(&member(&stale, name)).unwrap();
        }
        writer.finish().unwrap().into_inner()
    };
    assert!(validate_archive_deployment(&source(), &compiled, xml.as_bytes()).is_ok());
    let error = validate_archive_deployment(&source(), &stale, xml.as_bytes())
        .unwrap_err()
        .to_string();
    assert!(error.contains("replacement sidecars"), "{error}");
}

#[test]
fn unsupported_or_unverifiable_edits_fail_closed() {
    let selector = edited(NEW_SELECTOR);
    let cases = [
        // Not a supported addition.
        (
            edited(r#"    <C Type="Switch" U="a1000000-0000-0001-ffff000000000000" Title="Switch"><IoData Visu="true" Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/></C>
"#),
            "adding Switch",
        ),
        // Edits beyond the reconciled attributes.
        (
            PROJECT.replace(DATE, NEW_DATE).replace(
                "<Display Unit=\"&lt;v.1&gt; %\"/>",
                "<Display Unit=\"&lt;v.2&gt; %\"/>",
            ),
            "beyond its Title",
        ),
        (
            PROJECT.replace(DATE, NEW_DATE).replace("Title=\"Door\"", "Title=\"Gate door\""),
            "beyond its OutName1",
        ),
        (
            PROJECT.replace(EXISTING_NODE, ""),
            "removing compiled energy nodes",
        ),
        (
            PROJECT.replace("<Co K=\"Q\" U=\"4a000000-0000-0003-ffff000000000000\"/>\n", ""),
            "changing control 4a000000",
        ),
        (
            PROJECT.replace("Title=\"Room\"", "Title=\"Renamed room\""),
            "changing control 1b000000",
        ),
        // Removing or moving a visualized control.
        (
            PROJECT.replace("<C Type=\"Doorcontroller\"", "<C Type=\"Unknown\""),
            "changing control 7a000000",
        ),
        // Unverifiable metadata.
        (selector.replace("&lt;v.2&gt;", "&lt;v.9&gt;"), "display format"),
        (
            selector.replace(
                "Visu=\"true\" Pr=\"1b000000-0000-0001-ffff000000000000\" Cr=\"1c000000-0000-0001-ffff000000000000\"/>\n      <Display Unit=\"Level",
                "Visu=\"true\" Pr=\"1d000000-0000-0002-ffff000000000000\" Cr=\"1c000000-0000-0001-ffff000000000000\"/>\n      <Display Unit=\"Level",
            ),
            "existing app room",
        ),
        (
            selector.replace(
                "Title=\"New level\"",
                "Title=\"New level\" RGR=\"1a000000-0000-0001-ffff000000000000\"",
            ),
            "permission overrides",
        ),
        (
            selector.replace("<Co K=\"Step\" U=\"2b000000-0000-0005-ffff000000000000\"/>\n", ""),
            "Step state connector",
        ),
        (selector.replace(NEW_DATE, DATE), "advance the project Document Date"),
        (
            edited(&NEW_SELECTOR.replace("V=\"178\"", "V=\"179\"")),
            "ValueSelector catalogue template of the same block version",
        ),
        (
            edited(&NEW_STATUS.replace("V=\"178\"", "V=\"179\"")),
            "TextState catalogue template of the same block version",
        ),
        (
            edited(&NEW_METER.replace("V=\"178\"", "V=\"179\"")),
            "energy meter template",
        ),
        (
            edited(&NEW_METER.replace("StatsType=\"1\"", "StatsType=\"2\"")),
            "energy meter template",
        ),
        (
            edited(&NEW_METER.replace(
                "<IoData Pr",
                "<IoData Visu=\"true\" Pr",
            )),
            "linked energy meters",
        ),
        (
            edited(NEW_METER).replace(
                EXISTING_NODE,
                &format!("{EXISTING_NODE}{}", NEW_NODE.replace("t=\"1\"", "t=\"2\"")),
            ),
            "new load nodes",
        ),
    ];
    for (xml, expected) in cases {
        let error = rejection(&xml);
        assert!(error.contains(expected), "{expected}: {error}");
    }
}

#[test]
fn new_controls_need_matching_native_templates_and_no_stale_records() {
    let xml = edited(NEW_SELECTOR);
    let mut data: Value = serde_json::from_str(CATALOGUE).unwrap();
    // Without a compiled ValueSelector there is nothing to copy from.
    let mut no_template = data.clone();
    no_template["controls"]["U:2a000000-0000-0001-ffff000000000000"]["type"] = json!("Switch");
    let source = archive(PROJECT, &no_template.to_string(), &source_permissions());
    assert!(rejection_from(&source, &xml).contains("ValueSelector catalogue template"));
    // The compiled selector's permission record must have the verified layout.
    let mut permissions = source_permissions();
    permissions[2 * 16] ^= 1;
    let source = archive(PROJECT, CATALOGUE, &permissions);
    assert!(rejection_from(&source, &xml).contains("ValueSelector permission template"));
    let source = archive(PROJECT, CATALOGUE, &permissions[..permissions.len() - 1]);
    assert!(rejection_from(&source, &xml).contains("alignment"));
    // Stale identities for a control that is new in the XML are never reused.
    data["controls"]["U:2b000000-0000-0001-ffff000000000000"] = json!({"type": "ValueSelector"});
    let source = archive(PROJECT, &data.to_string(), &source_permissions());
    assert!(rejection_from(&source, &xml).contains("stale entry"));
    let mut permissions = source_permissions();
    permissions.extend(record(&uuid("2b", 1), &[], &[&[]]));
    let source = archive(PROJECT, CATALOGUE, &permissions);
    assert!(rejection_from(&source, &xml).contains("stale record"));
}

fn unchanged(xml: &str) {
    let repacked = deploy(xml);
    assert_eq!(member(&repacked, "LoxAPP3.json"), CATALOGUE.as_bytes());
    assert_eq!(member(&repacked, "permissions.bin"), source_permissions());
}

#[test]
fn plumbing_outside_the_app_deploys_without_sidecar_changes() {
    // Additions, edits and removals of logic and I/O plumbing, as in real projects.
    unchanged(&edited(
        r#"    <C Type="InputRef" V="178" U="9d000000-0000-0001-ffff000000000000" Ref="2a000000-0000-0006-ffff000000000000">
      <Co K="AQ" U="9d000000-0000-0002-ffff000000000000"/>
    </C>
    <C Type="OutputRef" V="178" U="9e000000-0000-0001-ffff000000000000">
      <Co K="AI" U="9e000000-0000-0002-ffff000000000000"><In Input="9d000000-0000-0002-ffff000000000000"/></Co>
    </C>
    <C Type="VirtualOutCmd" V="178" U="9f000000-0000-0001-ffff000000000000" Title="New command" CmdOn="/new">
      <IoData Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/>
    </C>
"#,
    ));
    unchanged(
        &PROJECT
            .replace("CmdOn=\"/on\"", "CmdOn=\"/changed\" CmdOnPost=\"x\"")
            .replace("Desc=\"I1*2\"", "Desc=\"I1*3\""),
    );
    unchanged(&PROJECT.replace(
        "    <C Type=\"Formula\" V=\"178\" U=\"9c000000-0000-0001-ffff000000000000\" Title=\"Formula\" Desc=\"I1*2\">\n      <Co K=\"I1\" U=\"9c000000-0000-0002-ffff000000000000\"/>\n    </C>\n",
        "",
    ));
    // Controls that are not visualized or linked stay outside the app.
    unchanged(&edited(&NEW_SELECTOR.replace("Visu=\"true\" ", "")));
    // Save counters are metadata.
    unchanged(&PROJECT.replace(
        "Title=\"Synthetic project\"",
        "Title=\"Synthetic project\" NumO=\"42\"",
    ));
}

#[test]
fn catalogue_neutral_edits_of_visualized_controls_need_no_sidecar_changes() {
    // Input wiring and connector parameters of compiled control types.
    unchanged(&PROJECT.replace(
        "<Co K=\"Min\" U=\"2a000000-0000-0003-ffff000000000000\"/>",
        "<Co K=\"Min\" U=\"2a000000-0000-0003-ffff000000000000\" V=\"5\"><In Input=\"9c000000-0000-0002-ffff000000000000\"/></Co>",
    ));
    unchanged(&PROJECT.replace(
        "<Co K=\"Q\" U=\"4a000000-0000-0003-ffff000000000000\"/>",
        "<Co K=\"Q\" U=\"4a000000-0000-0003-ffff000000000000\" Nc=\"1\"/>",
    ));
    // Status texts and light moods are runtime states.
    unchanged(&PROJECT.replace(
        "<Co K=\"I1\" U=\"3a000000-0000-0002-ffff000000000000\"/>",
        "<Co K=\"I1\" U=\"3a000000-0000-0002-ffff000000000000\"/>\n      <StateTexts><St T=\"Open\"/></StateTexts>",
    ));
    unchanged(&PROJECT.replace(
        "<Ls Id=\"1\" Name=\"Bright\"/>",
        "<Ls Id=\"1\" Name=\"Bright\"/><Ls Id=\"2\" Name=\"Candle\"/>",
    ));
    unchanged(&PROJECT.replace(
        "<Co K=\"Mv\" U=\"9a000000-0000-0002-ffff000000000000\"/>",
        "<Co K=\"Mv\" U=\"9a000000-0000-0002-ffff000000000000\"><In Input=\"9b000000-0000-0002-ffff000000000000\"/></Co>",
    ));
}

#[test]
fn links_and_renames_update_existing_entries() {
    let xml = PROJECT
        .replace(DATE, NEW_DATE)
        .replace(
            "Title=\"Existing status\"",
            "Title=\"Existing status\" linkC=\"4a000000-0000-0001-ffff000000000000,9a000000-0000-0001-ffff000000000000\"",
        )
        .replace("Title=\"Lights\"", "Title=\"Ceiling lights\"");
    let repacked = deploy(&xml);
    let result = app(&repacked);
    let mut expected: Value = serde_json::from_str(CATALOGUE).unwrap();
    expected["lastModified"] = json!("2026-02-01 00:00:00");
    expected["controls"]["U:3a000000-0000-0001-ffff000000000000"]["links"] =
        json!([uuid("4a", 1), uuid("9a", 1)]);
    expected["controls"]["U:9a000000-0000-0001-ffff000000000000"]["name"] = json!("Ceiling lights");
    assert_eq!(result, expected);
    assert_eq!(member(&repacked, "permissions.bin"), source_permissions());
    // Removing every link removes the key.
    let unlinked = xml
        .replace(NEW_DATE, "Date=\"2026-03-01 00:00:00\"")
        .replace(
            " linkC=\"4a000000-0000-0001-ffff000000000000,9a000000-0000-0001-ffff000000000000\"",
            "",
        );
    let result = app(&repack_zip(&repacked, unlinked.as_bytes()).unwrap());
    assert!(
        result["controls"]["U:3a000000-0000-0001-ffff000000000000"]
            .get("links")
            .is_none()
    );
}

#[test]
fn edits_that_reach_the_app_stay_protected() {
    let cases = [
        // A link to a control outside the catalogue needs a linked-only entry.
        (
            PROJECT.replace(DATE, NEW_DATE).replace(
                "Title=\"Existing status\"",
                "Title=\"Existing status\" linkC=\"9b000000-0000-0001-ffff000000000000\"",
            ),
            "linked control 9b000000",
        ),
        // Plumbing referenced by another control's links joins the app.
        (
            edited(r#"    <C Type="VirtualIn" V="178" U="9d000000-0000-0001-ffff000000000000" Title="Linked"/>
"#)
            .replace(
                "Title=\"Existing status\"",
                "Title=\"Existing status\" linkC=\"9d000000-0000-0001-ffff000000000000\"",
            ),
            "adding VirtualIn",
        ),
        // New visualized plumbing, and new blocks of types outside the verified set.
        (
            edited(r#"    <C Type="VirtualIn" V="178" U="9d000000-0000-0001-ffff000000000000" Title="Visible"><IoData Visu="true" Pr="1b000000-0000-0001-ffff000000000000" Cr="1c000000-0000-0001-ffff000000000000"/></C>
"#),
            "adding VirtualIn",
        ),
        (
            edited(r#"    <C Type="Mode" V="178" U="9d000000-0000-0001-ffff000000000000" Title="Mode"/>
"#),
            "adding Mode",
        ),
        // Wiring of light controllers is neutral, but their circuits are not verified.
        (
            PROJECT.replace(
                "<Co K=\"Mv\" U=\"9a000000-0000-0002-ffff000000000000\"/>",
                "<Co K=\"Mv\" U=\"9a000000-0000-0002-ffff000000000000\"/><Co K=\"AQ1\" U=\"9a000000-0000-0004-ffff000000000000\"/>",
            ),
            "changing control 9a000000",
        ),
        // Visualizing existing plumbing.
        (
            PROJECT.replace(
                "Title=\"Command\" CmdOn=\"/on\">\n      <Co K=\"I\" U=\"9b000000-0000-0002-ffff000000000000\"/>\n      <IoData Pr",
                "Title=\"Command\" CmdOn=\"/on\">\n      <Co K=\"I\" U=\"9b000000-0000-0002-ffff000000000000\"/>\n      <IoData Visu=\"true\" Pr",
            ),
            "changing control 9b000000",
        ),
    ];
    for (xml, expected) in cases {
        assert_ne!(xml, PROJECT);
        let error = rejection(&xml);
        assert!(error.contains(expected), "{expected}: {error}");
    }
}
