use lox_core::autopilot_catalog::{
    AutopilotCatalog, AutopilotRule, CatalogDecodeError, RuleResolutionError, resolve_rule,
};
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
fn rule(uuid: &str, name: &str) -> AutopilotRule {
    AutopilotRule {
        uuid: uuid.into(),
        name: name.into(),
        state_changed: "raw/\0".into(),
        state_history: "<history>".into(),
    }
}

#[test]
fn selector_uuid_branch_is_loose_raw_and_first_row_wins() {
    for uuid in [
        "abcdef01-2345-6789-abcdef0123456789",
        "abcdef01-2345-6789-abcd-ef0123456789",
        "---------------------",
        "a----------------------------------------------",
    ] {
        let rows = [
            rule(uuid, "first"),
            rule(uuid, "second"),
            rule("other", uuid),
        ];
        assert!(std::ptr::eq(resolve_rule(&rows, uuid).unwrap(), &rows[0]));
    }
    let lower = "abcdef01-2345-6789-abcdef0123456789";
    let upper = lower.to_uppercase();
    let rows = [rule(lower, &upper)];
    assert_eq!(
        resolve_rule(&rows, &upper),
        Err(RuleResolutionError::UuidNotFound)
    );
    for length in [0, 1, 19, 20, 21, 22, 35, 36, 80] {
        let query = "-".repeat(length);
        let rows = [rule("opaque", &query)];
        if length > 20 {
            assert_eq!(
                resolve_rule(&rows, &query),
                Err(RuleResolutionError::UuidNotFound)
            );
        } else {
            assert_eq!(resolve_rule(&rows, &query), Ok(&rows[0]));
        }
    }
    for query in [
        "gggggggg-gggg-gggg-gggggggggggggggg",
        "abcdef0123456789abcdef0123456789",
    ] {
        let rows = [rule("opaque", query)];
        assert_eq!(resolve_rule(&rows, query), Ok(&rows[0]));
    }
}

#[test]
fn selector_names_are_substrings_not_identity_ranking_or_distinct_ids() {
    let rows = [
        rule("same", "Kitchen Extra"),
        rule("same", "Kitchen"),
        rule("third", "Kitchen"),
        rule("Kitchen", "Unrelated"),
    ];
    assert_eq!(
        resolve_rule(&rows, "Kitchen"),
        Err(RuleResolutionError::Ambiguous {
            candidates: vec![&rows[0], &rows[1], &rows[2]]
        })
    );
    assert_eq!(
        resolve_rule(&rows, "Kitchen [Room]"),
        Err(RuleResolutionError::NotFound)
    );
    assert_eq!(
        resolve_rule(&rows, "same"),
        Err(RuleResolutionError::NotFound)
    );
    assert_eq!(
        resolve_rule(&rows, " Kitchen"),
        Err(RuleResolutionError::NotFound)
    );
    let reversed = [rows[2].clone(), rows[1].clone(), rows[0].clone()];
    assert_eq!(
        resolve_rule(&reversed, "Kitchen"),
        Err(RuleResolutionError::Ambiguous {
            candidates: reversed.iter().collect()
        })
    );
}

#[test]
fn selector_empty_placeholders_and_unicode_retain_lowercase_only_semantics() {
    assert_eq!(resolve_rule(&[], ""), Err(RuleResolutionError::NotFound));
    let rows = [
        rule("raw/\0", ""),
        rule("?", "?"),
        rule("unicode", "Ü İ Σ Straße é"),
    ];
    assert_eq!(resolve_rule(&rows[..1], ""), Ok(&rows[0]));
    assert_eq!(
        resolve_rule(&rows, ""),
        Err(RuleResolutionError::Ambiguous {
            candidates: rows.iter().collect()
        })
    );
    assert_eq!(resolve_rule(&rows, "?"), Ok(&rows[1]));
    for query in ["ü", "i\u{307}", "σ", "straße", "é"] {
        assert_eq!(resolve_rule(&rows, query), Ok(&rows[2]));
    }
    for query in ["STRASSE", "e\u{301}", "Ｕ", " ü", "ς"] {
        assert_eq!(
            resolve_rule(&rows, query),
            Err(RuleResolutionError::NotFound)
        );
    }
}

#[test]
fn selector_catalog_stable_ties_are_not_reordered_by_selection() {
    let bytes =
        br#"{"autopilot":{"z":{"name":"Same"},"a":{"name":"Same"},"m":{"name":"Same extra"}}}"#;
    let catalog = AutopilotCatalog::from_loxapp3_json(bytes).unwrap();
    let RuleResolutionError::Ambiguous { candidates } =
        resolve_rule(catalog.rules(), "Same").unwrap_err()
    else {
        panic!("exact names must not override containing names");
    };
    assert_eq!(candidates, catalog.rules().iter().collect::<Vec<_>>());
    assert!(std::error::Error::source(&RuleResolutionError::NotFound).is_none());
}
