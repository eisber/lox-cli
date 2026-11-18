//! Owned record conversion; parsing, ordering and selection remain in Core.

use crate::control_catalog::CatalogDecodeError;
use lox_core::autopilot_catalog as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct AutopilotRule {
    pub name: String,
    pub uuid: String,
    pub state_changed: String,
    pub state_history: String,
}

impl From<core::AutopilotRule> for AutopilotRule {
    fn from(value: core::AutopilotRule) -> Self {
        Self {
            name: value.name,
            uuid: value.uuid,
            state_changed: value.state_changed,
            state_history: value.state_history,
        }
    }
}

impl From<AutopilotRule> for core::AutopilotRule {
    fn from(value: AutopilotRule) -> Self {
        Self {
            name: value.name,
            uuid: value.uuid,
            state_changed: value.state_changed,
            state_history: value.state_history,
        }
    }
}

/// Core failure variants with the supplied query and detached candidate rows.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum RuleResolutionError {
    #[error("autopilot rule UUID not found")]
    UuidNotFound { query: String },
    #[error("autopilot rule name matched no rules")]
    NotFound { query: String },
    #[error("autopilot rule name matched {} rules", candidates.len())]
    Ambiguous {
        query: String,
        candidates: Vec<AutopilotRule>,
    },
}

/// Resolve supplied typed rows without acquisition, parsing or reordering.
#[uniffi::export]
pub fn resolve_autopilot_rule(
    rules: Vec<AutopilotRule>,
    query: String,
) -> Result<AutopilotRule, RuleResolutionError> {
    let rules: Vec<core::AutopilotRule> = rules.into_iter().map(Into::into).collect();
    match core::resolve_rule(&rules, &query) {
        Ok(rule) => Ok(rule.clone().into()),
        Err(core::RuleResolutionError::UuidNotFound) => {
            Err(RuleResolutionError::UuidNotFound { query })
        }
        Err(core::RuleResolutionError::NotFound) => Err(RuleResolutionError::NotFound { query }),
        Err(core::RuleResolutionError::Ambiguous { candidates }) => {
            Err(RuleResolutionError::Ambiguous {
                query,
                candidates: candidates
                    .into_iter()
                    .map(|rule| rule.clone().into())
                    .collect(),
            })
        }
    }
}

/// Project one acquired snapshot into owned rules, without selection or state reads.
#[uniffi::export]
pub fn list_loxapp3_autopilot_rules(
    structure: Vec<u8>,
) -> Result<Vec<AutopilotRule>, CatalogDecodeError> {
    Ok(core::AutopilotCatalog::from_loxapp3_json(&structure)?
        .into_rules()
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(uuid: &str, name: &str) -> AutopilotRule {
        AutopilotRule {
            name: name.into(),
            uuid: uuid.into(),
            state_changed: "raw/\0".into(),
            state_history: "<history>\r\n".into(),
        }
    }

    #[test]
    fn selector_retains_raw_first_uuid_and_distinct_failure_variants() {
        let uuid = "abcdef01-2345-6789-abcdef0123456789";
        let rows = vec![rule(uuid, "first"), rule(uuid, "second")];
        let mut selected = resolve_autopilot_rule(rows.clone(), uuid.into()).unwrap();
        assert_eq!(selected, rows[0]);
        selected.state_changed.clear();
        assert_eq!(rows[0].state_changed, "raw/\0");
        let query = uuid.to_uppercase();
        assert_eq!(
            resolve_autopilot_rule(vec![rule(uuid, &query)], query.clone()),
            Err(RuleResolutionError::UuidNotFound { query })
        );
        let query = "---------------------".to_owned();
        assert_eq!(
            resolve_autopilot_rule(vec![rule("opaque", &query)], query.clone()),
            Err(RuleResolutionError::UuidNotFound { query })
        );
        assert_eq!(
            resolve_autopilot_rule(rows, "missing".into()),
            Err(RuleResolutionError::NotFound {
                query: "missing".into()
            })
        );
    }

    #[test]
    fn selector_ambiguity_owns_all_rows_in_caller_order_without_ranking() {
        let rows = vec![
            rule("same", "Kitchen Extra"),
            rule("same", "Kitchen"),
            rule("same", "Kitchen"),
        ];
        let error = resolve_autopilot_rule(rows.clone(), "Kitchen".into()).unwrap_err();
        assert_eq!(
            error,
            RuleResolutionError::Ambiguous {
                query: "Kitchen".into(),
                candidates: rows.clone()
            }
        );
        let RuleResolutionError::Ambiguous { mut candidates, .. } = error else {
            unreachable!()
        };
        candidates[0].state_history.clear();
        assert_eq!(rows[0].state_history, "<history>\r\n");
        for query in [" Kitchen", "Kitchen [Room]", "same"] {
            assert_eq!(
                resolve_autopilot_rule(rows.clone(), query.into()),
                Err(RuleResolutionError::NotFound {
                    query: query.into()
                })
            );
        }
    }

    #[test]
    fn selector_preserves_empty_unicode_and_opaque_fields() {
        assert_eq!(
            resolve_autopilot_rule(vec![], "".into()),
            Err(RuleResolutionError::NotFound { query: "".into() })
        );
        let row = rule("raw/\0ID", "Ü İ Σ Straße é");
        for query in ["", "ü", "i\u{307}", "σ", "straße", "é"] {
            assert_eq!(
                resolve_autopilot_rule(vec![row.clone()], query.into()),
                Ok(row.clone())
            );
        }
        for query in ["STRASSE", "e\u{301}", " ü", "ς"] {
            assert_eq!(
                resolve_autopilot_rule(vec![row.clone()], query.into()),
                Err(RuleResolutionError::NotFound {
                    query: query.into()
                })
            );
        }
        for query in [
            "--------------------",
            "abcdef0123456789abcdef0123456789",
            "gggggggg-gggg-gggg-gggggggggggggggg",
        ] {
            let row = rule("opaque", query);
            assert_eq!(
                resolve_autopilot_rule(vec![row.clone()], query.into()),
                Ok(row)
            );
        }
        let rows = vec![rule("raw", ""), rule("?", "?")];
        assert_eq!(
            resolve_autopilot_rule(rows.clone(), "".into()),
            Err(RuleResolutionError::Ambiguous {
                query: "".into(),
                candidates: rows
            })
        );
    }

    #[test]
    fn owned_projection_preserves_core_permissive_values_and_order() {
        let input = include_bytes!("../tests/fixtures/autopilot_catalog.json").to_vec();
        let expected: Vec<AutopilotRule> = core::AutopilotCatalog::from_loxapp3_json(&input)
            .unwrap()
            .into_rules()
            .into_iter()
            .map(Into::into)
            .collect();
        let mut actual = list_loxapp3_autopilot_rules(input).unwrap();
        assert_eq!(actual, expected);
        actual[0].name.push_str("changed");
        assert_ne!(actual, expected);
    }

    #[test]
    fn empty_shapes_and_existing_error_alias_are_unchanged() {
        for input in [
            "null",
            "true",
            "3",
            "\"x\"",
            "[]",
            "{}",
            r#"{"autopilot":[]}"#,
            r#"{"autopilot":false}"#,
            r#"{"autopilot":null}"#,
        ] {
            assert!(
                list_loxapp3_autopilot_rules(input.as_bytes().to_vec())
                    .unwrap()
                    .is_empty()
            );
        }
        for input in [b"{\n\"autopilot\":".as_slice(), b"{} secret", b"\"\xff\""] {
            let error: lox_core::control_catalog::CatalogDecodeError =
                core::AutopilotCatalog::from_loxapp3_json(input).unwrap_err();
            assert_eq!(
                list_loxapp3_autopilot_rules(input.to_vec()),
                Err(error.into())
            );
        }
    }
}
