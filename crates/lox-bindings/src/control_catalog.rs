//! Stateless byte-input catalog selection; parsing and matching belong to Core.

use crate::control::Control;
use lox_core::control_catalog as core;

#[derive(Clone, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct ControlFilter {
    pub typ: Option<String>,
    pub room: Option<String>,
    pub category: Option<String>,
    pub favorites_only: bool,
}

impl From<ControlFilter> for core::ControlFilter {
    fn from(value: ControlFilter) -> Self {
        Self {
            typ: value.typ,
            room: value.room,
            category: value.category,
            favorites_only: value.favorites_only,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum CatalogDecodeError {
    #[error("invalid LoxApp3 JSON at line {line} column {column}")]
    InvalidJson { line: u64, column: u64 },
}

impl From<core::CatalogDecodeError> for CatalogDecodeError {
    fn from(value: core::CatalogDecodeError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match value {
            core::CatalogDecodeError::InvalidJson { line, column } => Self::InvalidJson {
                line: line as u64,
                column: column as u64,
            },
        }
    }
}

/// Parse and select on every call; returns owned records, not a catalog handle.
#[uniffi::export]
pub fn select_loxapp3_controls(
    structure: Vec<u8>,
    filter: ControlFilter,
) -> Result<Vec<Control>, CatalogDecodeError> {
    let catalog = core::ControlCatalog::from_loxapp3_json(&structure)?;
    Ok(catalog
        .select(&filter.into())
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_projection_and_filters_match_core_with_owned_results() {
        let bytes = include_bytes!("../tests/fixtures/control_catalog.json").to_vec();
        let catalog = core::ControlCatalog::from_loxapp3_json(&bytes).unwrap();
        for filter in [
            ControlFilter::default(),
            ControlFilter {
                typ: Some("STRAß".into()),
                room: Some("kü".into()),
                category: Some("lüft".into()),
                favorites_only: true,
            },
            ControlFilter {
                typ: Some("STRASSE".into()),
                ..Default::default()
            },
            ControlFilter {
                room: Some("".into()),
                category: Some("".into()),
                ..Default::default()
            },
        ] {
            let actual = select_loxapp3_controls(bytes.clone(), filter.clone()).unwrap();
            let expected: Vec<Control> = catalog
                .select(&filter.into())
                .into_iter()
                .map(Into::into)
                .collect();
            assert_eq!(actual, expected);
        }
        let mut actual = select_loxapp3_controls(bytes, ControlFilter::default()).unwrap();
        actual[0].name.clear();
        assert_eq!(catalog.controls()[0].name, "?");
    }

    #[test]
    fn permissive_shapes_and_payload_free_errors_are_not_revalidated() {
        for bytes in [
            "null",
            "true",
            "3",
            "\"text\"",
            "[]",
            "{}",
            r#"{"controls":null,"rooms":7,"cats":[]}"#,
            r#"{"controls":[]}"#,
            r#"{"controls":false}"#,
        ] {
            assert!(
                select_loxapp3_controls(bytes.as_bytes().to_vec(), ControlFilter::default())
                    .unwrap()
                    .is_empty()
            );
        }
        for bytes in [b"{\n\"controls\":".as_slice(), b"{} secret", b"\"\xff\""] {
            let expected = CatalogDecodeError::from(
                core::ControlCatalog::from_loxapp3_json(bytes).unwrap_err(),
            );
            assert_eq!(
                select_loxapp3_controls(bytes.to_vec(), ControlFilter::default()),
                Err(expected)
            );
        }
        assert_eq!(
            CatalogDecodeError::from(core::CatalogDecodeError::InvalidJson {
                line: usize::MAX,
                column: usize::MAX,
            }),
            CatalogDecodeError::InvalidJson {
                line: usize::MAX as u64,
                column: usize::MAX as u64,
            }
        );
    }
}
