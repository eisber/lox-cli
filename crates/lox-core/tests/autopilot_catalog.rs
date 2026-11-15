use lox_core::autopilot_catalog::{AutopilotCatalog, AutopilotRule, CatalogDecodeError};
use serde_json::{Value, json};

#[test]
fn permissive_roots_entries_and_independent_defaults() {
    for root in ["null", "true", "1", "\"x\"", "[]", "{}"] {
        assert!(
            AutopilotCatalog::from_loxapp3_json(root.as_bytes())
                .unwrap()
                .rules()
                .is_empty()
        );
    }
    for value in json!([null, false, 1, "", [], {}]).as_array().unwrap() {
        assert!(
            AutopilotCatalog::from_loxapp3_json(
                &serde_json::to_vec(&json!({"autopilot":value})).unwrap()
            )
            .unwrap()
            .rules()
            .is_empty()
        );
        let input = json!({"autopilot":{
            "scalar":value,
            "name":{"name":value},
            "states":{"states":value},
            "changed":{"states":{"changed":value,"history":"history"}},
            "history":{"states":{"changed":"changed","history":value}}
        }});
        let catalog =
            AutopilotCatalog::from_loxapp3_json(&serde_json::to_vec(&input).unwrap()).unwrap();
        assert_eq!(catalog.rules().len(), 5);
        for rule in catalog.rules() {
            assert_eq!(
                rule.name,
                if rule.uuid == "name" {
                    value.as_str().unwrap_or("?")
                } else {
                    "?"
                }
            );
            assert_eq!(
                rule.state_changed,
                if rule.uuid == "history" {
                    "changed"
                } else if rule.uuid == "changed" {
                    value.as_str().unwrap_or("")
                } else {
                    ""
                }
            );
            assert_eq!(
                rule.state_history,
                if rule.uuid == "changed" {
                    "history"
                } else if rule.uuid == "history" {
                    value.as_str().unwrap_or("")
                } else {
                    ""
                }
            );
        }
    }
}

#[test]
fn raw_owned_rows_and_stable_name_only_sort_follow_enabled_map_order() {
    let mut bytes = br#"{"autopilot":{
        "tie-z":{"name":"Same"},
        "tie-a":{"name":"Same"},
        "raw\u0000/e\u0301":{"name":"<C>\u00dc\u0000e\u0301","states":{"changed":"<changed>\u0000","history":"History/ID"}},
        "lower":{"name":"a"},
        "upper":{"name":"A"},
        "empty":{"name":""},
        "scalar":false
    }}"#.to_vec();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let expected_ties: Vec<_> = value["autopilot"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, row)| row["name"] == "Same")
        .map(|(id, _)| id.clone())
        .collect();
    let catalog = AutopilotCatalog::from_loxapp3_json(&bytes).unwrap();
    bytes.fill(0);
    drop(bytes);
    drop(value);
    assert_eq!(
        catalog
            .rules()
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>(),
        ["", "<C>Ü\0e\u{301}", "?", "A", "Same", "Same", "a"]
    );
    assert_eq!(
        catalog
            .rules()
            .iter()
            .filter(|r| r.name == "Same")
            .map(|r| r.uuid.clone())
            .collect::<Vec<_>>(),
        expected_ties
    );
    assert_eq!(
        catalog.rules()[1],
        AutopilotRule {
            name: "<C>Ü\0e\u{301}".into(),
            uuid: "raw\0/e\u{301}".into(),
            state_changed: "<changed>\0".into(),
            state_history: "History/ID".into(),
        }
    );
    let mut detached = catalog.clone().into_rules();
    detached[1].name.clear();
    assert_eq!(catalog.rules()[1].name, "<C>Ü\0e\u{301}");
}

#[test]
fn invalid_syntax_and_utf8_have_shared_payload_free_coordinates() {
    for bytes in [
        b"{\n\"autopilot\":".as_slice(),
        b"{} secret-tail",
        b"\"\xff\"",
    ] {
        let error = AutopilotCatalog::from_loxapp3_json(bytes).unwrap_err();
        let original = serde_json::from_slice::<Value>(bytes).unwrap_err();
        assert_eq!(
            error,
            CatalogDecodeError::InvalidJson {
                line: original.line(),
                column: original.column(),
            }
        );
        assert_eq!(
            error.to_string(),
            format!(
                "invalid LoxApp3 JSON at line {} column {}",
                original.line(),
                original.column()
            )
        );
        let _: &dyn std::error::Error = &error;
    }
}
