//! Read-only projection of runtime LoxApp3 autopilot rules, without acquisition.

pub use crate::control_catalog::CatalogDecodeError;
use serde_json::Value;

/// Owned runtime rule identifiers, not a rule/action/condition model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopilotRule {
    pub name: String,
    pub uuid: String,
    pub state_changed: String,
    pub state_history: String,
}

/// A permissive snapshot of the top-level LoxApp3 `autopilot` map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopilotCatalog {
    rules: Vec<AutopilotRule>,
}

impl AutopilotCatalog {
    /// Accept any valid JSON root; only an object-valued `autopilot` is iterated.
    ///
    /// Every map key produces a row, even for non-object entries. Missing or
    /// non-string names become `"?"`; changed/history states independently
    /// default to `""`. Explicit empty strings and all opaque IDs are preserved.
    /// Rules sort stably by case-sensitive name, retaining map iteration order
    /// for ties under the enabled serde_json features. No normalization,
    /// validation, selection or XML configuration interpretation is performed.
    pub fn from_loxapp3_json(bytes: &[u8]) -> Result<Self, CatalogDecodeError> {
        let structure: Value =
            serde_json::from_slice(bytes).map_err(|error| CatalogDecodeError::InvalidJson {
                line: error.line(),
                column: error.column(),
            })?;
        let mut rules = Vec::new();
        if let Some(map) = structure.get("autopilot").and_then(Value::as_object) {
            for (uuid, entry) in map {
                let state = |key| {
                    entry
                        .get("states")
                        .and_then(|states| states.get(key))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_owned()
                };
                rules.push(AutopilotRule {
                    name: entry
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("?")
                        .to_owned(),
                    uuid: uuid.clone(),
                    state_changed: state("changed"),
                    state_history: state("history"),
                });
            }
        }
        rules.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { rules })
    }

    /// Rules in stable, case-sensitive name order.
    pub fn rules(&self) -> &[AutopilotRule] {
        &self.rules
    }

    /// Transfer the owned rules out of this snapshot.
    pub fn into_rules(self) -> Vec<AutopilotRule> {
        self.rules
    }
}
