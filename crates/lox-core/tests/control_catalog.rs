use lox_core::control::{Control, ControlQuery, PreparedControlQuery};
use lox_core::control_catalog::{CatalogDecodeError, ControlCatalog, ControlFilter};
use std::collections::HashMap;

#[test]
fn owned_snapshot_reuses_controls_filters_and_resolution_without_raw_tree() {
    let bytes = br#"{
        "rooms": {"": {"name": "Default"}, "bad": {"name": 3}},
        "cats": {"": {"name": "Category"}},
        "controls": {
            "opaque / key": {"name": "Zulu", "type": "Switch", "isFavorite": true,
                "isSecured": true, "room": null, "cat": 42,
                "subControls": {"nested": {"name": "Excluded"}}},
            "u2": {"name": "Alpha", "type": "SWITCH", "room": "missing", "cat": "bad",
                "isFavorite": "true", "isSecured": 1},
            "u3": {"name": "Alpha", "room": "bad"},
            "": null
        }
    }"#
    .to_vec();
    let catalog = ControlCatalog::from_loxapp3_json(&bytes).unwrap();
    drop(bytes);
    let all: &[Control] = catalog.controls();
    assert_eq!(
        all.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        ["?", "Alpha", "Alpha", "Zulu"]
    );
    assert_eq!(
        all[0],
        Control {
            name: "?".into(),
            typ: "?".into(),
            uuid: "".into(),
            room: Some("Default".into()),
            cat: Some("Category".into()),
            is_favorite: false,
            is_secured: false,
        }
    );
    let filter = ControlFilter {
        typ: Some("swi".into()),
        room: Some("def".into()),
        category: Some("cat".into()),
        favorites_only: true,
    };
    let mut selected = catalog.select(&filter);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].uuid, "opaque / key");
    assert!(selected[0].is_secured);
    selected[0].name.clear();
    assert_eq!(catalog.controls()[3].name, "Zulu");
    assert_eq!(
        catalog
            .select(&ControlFilter {
                typ: Some("switch".into()),
                room: Some("".into()),
                category: Some("".into()),
                ..Default::default()
            })
            .len(),
        2
    );
    assert!(
        catalog
            .controls()
            .iter()
            .filter(|c| c.name == "Alpha")
            .all(|c| c.room.is_none() && !c.is_favorite && !c.is_secured)
    );
    let PreparedControlQuery::Catalog(query) = ControlQuery::prepare("Zulu", None, &HashMap::new())
    else {
        panic!("name requires catalog");
    };
    assert_eq!(
        query.resolve(catalog.controls()).unwrap().uuid,
        "opaque / key"
    );
}

#[test]
fn valid_json_shapes_are_permissive_but_invalid_bytes_are_typed_errors() {
    for bytes in [
        "null",
        "true",
        "4",
        "\"text\"",
        "[]",
        "{}",
        r#"{"controls": null}"#,
        r#"{"controls": []}"#,
        r#"{"controls": "wrong"}"#,
        r#"{"controls": 4}"#,
    ] {
        assert!(
            ControlCatalog::from_loxapp3_json(bytes.as_bytes())
                .unwrap()
                .controls()
                .is_empty(),
            "{bytes}"
        );
    }
    for bytes in [
        r#"{"rooms":null,"cats":[],"controls":{"key":false}}"#,
        r#"{"rooms":7,"cats":"wrong","controls":{"key":[]}}"#,
        r#"{"rooms":{"":{"name":false}},"cats":{"":{"name":[]}},"controls":{"key":{}}}"#,
    ] {
        let catalog = ControlCatalog::from_loxapp3_json(bytes.as_bytes()).unwrap();
        assert_eq!(catalog.controls().len(), 1);
        assert_eq!(catalog.controls()[0].room, None);
        assert_eq!(catalog.controls()[0].cat, None);
    }
    for bytes in [b"{\n\"controls\":".as_slice(), b"{} trailing", b"\"\xff\""] {
        let error = ControlCatalog::from_loxapp3_json(bytes).unwrap_err();
        let CatalogDecodeError::InvalidJson { line, column } = error;
        assert!(line >= 1 && column >= 1);
        assert!(
            error
                .to_string()
                .starts_with("invalid LoxApp3 JSON at line ")
        );
        let _: &dyn std::error::Error = &error;
    }
}

#[test]
fn unicode_filters_are_lowercase_substrings_not_case_folding_or_trimming() {
    let catalog = ControlCatalog::from_loxapp3_json(
        r#"{
        "rooms":{"r":{"name":"KÜCHE"}},"cats":{"c":{"name":"LÜFTUNG"}},
        "controls":{"u":{"type":"Straße","room":"r","cat":"c","isFavorite":true}}
    }"#
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(
        catalog
            .select(&ControlFilter {
                typ: Some("STRAß".into()),
                room: Some("kü".into()),
                category: Some("lüft".into()),
                favorites_only: true,
            })
            .len(),
        1
    );
    for typ in ["STRASSE", " Straße "] {
        assert!(
            catalog
                .select(&ControlFilter {
                    typ: Some(typ.into()),
                    ..Default::default()
                })
                .is_empty()
        );
    }
}
