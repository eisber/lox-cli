//! Owned record conversion; parsing and ordering remain in Core.

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
