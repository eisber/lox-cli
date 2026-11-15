//! Owned snapshot conversion; all parsing and projection policy belongs to Core.

use crate::control_catalog::CatalogDecodeError;
use lox_core::structure_metadata as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Category {
    pub uuid: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct GlobalState {
    pub name: String,
    pub uuid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct OperatingMode {
    pub raw_id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct StructureMetadata {
    pub categories: Vec<Category>,
    pub global_states: Vec<GlobalState>,
    pub operating_modes: Vec<OperatingMode>,
}

/// Parse one acquired snapshot into plain owned metadata, not a document handle.
#[uniffi::export]
pub fn parse_loxapp3_structure_metadata(
    structure: Vec<u8>,
) -> Result<StructureMetadata, CatalogDecodeError> {
    let snapshot = core::StructureMetadata::from_loxapp3_json(&structure)?;
    Ok(StructureMetadata {
        categories: snapshot
            .categories()
            .iter()
            .map(|value| Category {
                uuid: value.uuid.clone(),
                name: value.name.clone(),
            })
            .collect(),
        global_states: snapshot
            .global_states()
            .iter()
            .map(|value| GlobalState {
                name: value.name.clone(),
                uuid: value.uuid.clone(),
            })
            .collect(),
        operating_modes: snapshot
            .operating_modes()
            .iter()
            .map(|value| OperatingMode {
                raw_id: value.raw_id.clone(),
                name: value.name.clone(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_values_and_core_order_survive_owned_conversion() {
        let input = include_bytes!("../tests/fixtures/structure_metadata.json").to_vec();
        let expected = core::StructureMetadata::from_loxapp3_json(&input).unwrap();
        let actual = parse_loxapp3_structure_metadata(input).unwrap();
        assert_eq!(
            actual
                .categories
                .iter()
                .map(|v| (&v.uuid, &v.name))
                .collect::<Vec<_>>(),
            expected
                .categories()
                .iter()
                .map(|v| (&v.uuid, &v.name))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            actual
                .global_states
                .iter()
                .map(|v| (&v.name, &v.uuid))
                .collect::<Vec<_>>(),
            expected
                .global_states()
                .iter()
                .map(|v| (&v.name, &v.uuid))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            actual
                .operating_modes
                .iter()
                .map(|v| (&v.raw_id, &v.name))
                .collect::<Vec<_>>(),
            expected
                .operating_modes()
                .iter()
                .map(|v| (&v.raw_id, &v.name))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn permissive_shapes_and_existing_catalog_errors_are_preserved() {
        for bytes in [
            "null",
            "true",
            "3",
            "\"x\"",
            "[]",
            "{}",
            r#"{"cats":[],"globalStates":false,"operatingModes":null}"#,
        ] {
            let actual = parse_loxapp3_structure_metadata(bytes.as_bytes().to_vec()).unwrap();
            assert!(
                actual.categories.is_empty()
                    && actual.global_states.is_empty()
                    && actual.operating_modes.is_empty()
            );
        }
        for bytes in [b"{\n\"cats\":".as_slice(), b"{} secret", b"\"\xff\""] {
            assert_eq!(
                parse_loxapp3_structure_metadata(bytes.to_vec()),
                Err(CatalogDecodeError::from(
                    core::StructureMetadata::from_loxapp3_json(bytes).unwrap_err()
                ))
            );
        }
    }
}
