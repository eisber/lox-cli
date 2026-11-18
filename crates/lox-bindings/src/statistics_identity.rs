//! Owned identity results and diagnostics; matching remains in Core.

use crate::statistics_index::StatEntry;
use lox_core::statistics as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct StatisticIdentity {
    pub name: String,
    pub group: Option<u32>,
    pub uuid: String,
}

impl From<core::StatisticIdentity<'_>> for StatisticIdentity {
    fn from(value: core::StatisticIdentity<'_>) -> Self {
        Self {
            name: value.name.to_owned(),
            group: value.group,
            uuid: value.uuid.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum StatisticResolutionError {
    #[error("no matching statistic identity")]
    NotFound { names: Vec<String> },
    #[error("query matches {distinct_uuids} statistic identities")]
    Ambiguous {
        distinct_uuids: u64,
        candidates: Vec<StatisticIdentity>,
    },
}

impl From<core::StatisticResolutionError<'_>> for StatisticResolutionError {
    fn from(value: core::StatisticResolutionError<'_>) -> Self {
        match value {
            core::StatisticResolutionError::NotFound { names } => Self::NotFound {
                names: names.into_iter().map(str::to_owned).collect(),
            },
            core::StatisticResolutionError::Ambiguous {
                distinct_uuids,
                candidates,
            } => Self::Ambiguous {
                distinct_uuids: distinct_uuids as u64,
                candidates: candidates.into_iter().map(Into::into).collect(),
            },
        }
    }
}

/// Resolve supplied index rows to their raw UUID, without acquisition or parsing.
///
/// Core tries query-lowercased raw UUID, exact lowercased name, then substring.
/// File, label and month are not searched. Callers select all original rows with
/// the returned raw UUID, preserving their order, duplicates and other fields.
#[uniffi::export]
pub fn resolve_statistics_identity(
    entries: Vec<StatEntry>,
    query: String,
) -> Result<String, StatisticResolutionError> {
    let identities: Vec<_> = entries
        .iter()
        .map(|entry| core::StatisticIdentity {
            name: &entry.name,
            group: entry.group,
            uuid: &entry.uuid,
        })
        .collect();
    core::resolve_identity(&identities, &query)
        .map(str::to_owned)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, uuid: &str, group: Option<u32>) -> StatEntry {
        StatEntry {
            file: "not-an-identity.xml".into(),
            uuid: uuid.into(),
            group,
            month: "opaque-month".into(),
            name: name.into(),
            label: "not-an-identity-label".into(),
        }
    }

    #[test]
    fn owned_adapter_preserves_core_priority_and_opaque_values() {
        let rows = vec![
            row("Heat", "winner", Some(u32::MAX)),
            row("winner", "other", None),
            row("Heat Pump", "third", None),
            row("Heat", "winner", Some(0)),
        ];
        assert_eq!(
            resolve_statistics_identity(rows.clone(), "WINNER".into()),
            Ok("winner".into())
        );
        assert_eq!(
            resolve_statistics_identity(rows.clone(), "hEaT".into()),
            Ok("winner".into())
        );
        let identities: Vec<_> = rows
            .iter()
            .map(|entry| core::StatisticIdentity {
                name: &entry.name,
                group: entry.group,
                uuid: &entry.uuid,
            })
            .collect();
        for query in ["", "pump", "missing", "Heat ", "not-an-identity.xml"] {
            assert_eq!(
                resolve_statistics_identity(rows.clone(), query.into()),
                core::resolve_identity(&identities, query)
                    .map(str::to_owned)
                    .map_err(Into::into)
            );
        }
        let raw = "RAW\0é e\u{301}\r\nuuid";
        assert_eq!(
            resolve_statistics_identity(vec![row("İSTANBUL", raw, None)], "İstanbul".into()),
            Ok(raw.into())
        );
        for query in [
            "ABC",
            "Istanbul",
            "cafe\u{301}",
            "Café ",
            "not-an-identity-label",
        ] {
            assert!(matches!(
                resolve_statistics_identity(
                    vec![row("İSTANBUL", "ABC", None), row("Café", "x", None)],
                    query.into()
                ),
                Err(StatisticResolutionError::NotFound { .. })
            ));
        }
        assert_eq!(
            resolve_statistics_identity(
                vec![row("unrelated", "", None), row("", "other", None)],
                "".into()
            ),
            Ok("".into())
        );
    }

    #[test]
    fn diagnostics_are_owned_sorted_deduplicated_and_count_raw_uuids() {
        let rows = vec![
            row("Same", "z", Some(u32::MAX)),
            row("Same", "z", None),
            row("Same", "a", Some(0)),
            row("Same", "z", Some(u32::MAX)),
            row("same", "a", None),
        ];
        let ambiguous = resolve_statistics_identity(rows.clone(), "SAME".into()).unwrap_err();
        let missing = resolve_statistics_identity(rows, "missing".into()).unwrap_err();
        assert_eq!(
            ambiguous,
            StatisticResolutionError::Ambiguous {
                distinct_uuids: 2,
                candidates: vec![
                    StatisticIdentity {
                        name: "Same".into(),
                        group: None,
                        uuid: "z".into()
                    },
                    StatisticIdentity {
                        name: "Same".into(),
                        group: Some(0),
                        uuid: "a".into()
                    },
                    StatisticIdentity {
                        name: "Same".into(),
                        group: Some(u32::MAX),
                        uuid: "z".into()
                    },
                    StatisticIdentity {
                        name: "same".into(),
                        group: None,
                        uuid: "a".into()
                    },
                ],
            }
        );
        assert_eq!(
            missing,
            StatisticResolutionError::NotFound {
                names: vec!["Same".into(), "same".into()]
            }
        );
        assert_eq!(
            resolve_statistics_identity(vec![], "raw".into()),
            Err(StatisticResolutionError::NotFound { names: vec![] })
        );
        assert_eq!(
            resolve_statistics_identity(
                vec![row("a", "same", None), row("b", "same", Some(1))],
                "".into()
            ),
            Ok("same".into())
        );
    }
}
