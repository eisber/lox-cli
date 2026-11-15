//! Read-only projection of runtime LoxApp3 autopilot rules, without acquisition.

use crate::control::{is_uuid, matches_text};
pub use crate::control_catalog::CatalogDecodeError;
use serde_json::Value;
use std::fmt;

/// Owned runtime rule identifiers, not a rule/action/condition model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopilotRule {
    pub name: String,
    pub uuid: String,
    pub state_changed: String,
    pub state_history: String,
}

/// Row-based resolution failures; candidate references retain caller order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleResolutionError<'a> {
    UuidNotFound,
    NotFound,
    Ambiguous { candidates: Vec<&'a AutopilotRule> },
}

impl fmt::Display for RuleResolutionError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UuidNotFound => f.write_str("autopilot rule UUID not found"),
            Self::NotFound => f.write_str("autopilot rule name matched no rules"),
            Self::Ambiguous { candidates } => {
                write!(f, "autopilot rule name matched {} rules", candidates.len())
            }
        }
    }
}

impl std::error::Error for RuleResolutionError<'_> {}

/// Resolve an existing snapshot without sorting, normalization or deduplication.
///
/// The legacy loose UUID detector chooses a raw, case-sensitive identity search;
/// the first identical UUID wins. Otherwise only names are matched using Rust
/// lowercase substring comparison. An exact name has no preference over another
/// containing name, and duplicate rows still contribute to ambiguity.
pub fn resolve_rule<'a>(
    rules: &'a [AutopilotRule],
    query: &str,
) -> Result<&'a AutopilotRule, RuleResolutionError<'a>> {
    if is_uuid(query) {
        return rules
            .iter()
            .find(|rule| rule.uuid == query)
            .ok_or(RuleResolutionError::UuidNotFound);
    }
    let candidates: Vec<_> = rules
        .iter()
        .filter(|rule| matches_text(&rule.name, query))
        .collect();
    match candidates.len() {
        0 => Err(RuleResolutionError::NotFound),
        1 => Ok(candidates[0]),
        _ => Err(RuleResolutionError::Ambiguous { candidates }),
    }
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
