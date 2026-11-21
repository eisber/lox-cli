//! Owned direct-connector observations; loading and selection remain in Core.

use lox_core::configuration::{
    editor::{EditError, ElementIdentity, SelectionError},
    wiring as core,
};
use std::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum WiringDirection {
    Input,
    Output,
    Parameter,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConnectorWiring {
    pub connector: String,
    pub direction: WiringDirection,
    pub target_uuid: String,
    pub connected: bool,
}

impl From<core::ConnectorWiring> for ConnectorWiring {
    fn from(value: core::ConnectorWiring) -> Self {
        Self {
            connector: value.connector,
            direction: match value.direction {
                core::ConnectorDirection::Input => WiringDirection::Input,
                core::ConnectorDirection::Output => WiringDirection::Output,
                core::ConnectorDirection::Parameter => WiringDirection::Parameter,
            },
            target_uuid: value.target_uuid,
            connected: value.connected,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct WiringElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl From<ElementIdentity> for WiringElementIdentity {
    fn from(value: ElementIdentity) -> Self {
        Self {
            title: value.title,
            block_type: value.block_type,
            uuid: value.uuid,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum WiringCatalogError {
    #[error("Configuration load failed: {source_message}")]
    Load { source_message: String },
    #[error("No element matches {selector:?}")]
    NotFound {
        selector: String,
        titles: Vec<String>,
    },
    #[error("Multiple elements match {selector:?}")]
    Ambiguous {
        selector: String,
        matches: Vec<WiringElementIdentity>,
    },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<EditError> for WiringCatalogError {
    fn from(error: EditError) -> Self {
        match error {
            EditError::Load(cause) => Self::Load {
                source_message: cause.to_string(),
            },
            EditError::Selection(SelectionError::NotFound { selector, titles }) => {
                Self::NotFound { selector, titles }
            }
            EditError::Selection(SelectionError::Ambiguous { selector, matches }) => {
                Self::Ambiguous {
                    selector,
                    matches: matches.into_iter().map(Into::into).collect(),
                }
            }
            other => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Read direct connector children without mutation, endpoint validation or I/O.
/// The shared legacy loader limitation lox-n37w remains unchanged.
#[uniffi::export]
pub fn list_connector_wiring(
    data: Vec<u8>,
    selector: String,
) -> Result<Vec<ConnectorWiring>, WiringCatalogError> {
    Ok(core::connector_wiring(&data, &selector)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_projection_preserves_direct_order_raw_values_and_legacy_zero() {
        let bytes = include_bytes!("../tests/fixtures/wiring_catalog.xml");
        let expected: Vec<ConnectorWiring> = core::connector_wiring(bytes, "uuid:raw-id")
            .unwrap()
            .into_iter()
            .map(Into::into)
            .collect();
        let mut actual = list_connector_wiring(bytes.to_vec(), "uuid:raw-id".into()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 7);
        assert!(!actual[0].connected);
        assert!(actual[1].connected);
        assert_eq!(actual[2].direction, WiringDirection::Parameter);
        assert!(!actual[2].connected);
        assert!(actual[3].connected);
        assert_eq!(actual[4], actual[5]);
        actual[0].connector.clear();
        assert_eq!(
            list_connector_wiring(bytes.to_vec(), "uuid:raw-id".into()).unwrap(),
            expected
        );
    }

    #[test]
    fn actual_loader_and_selector_errors_retain_core_payloads() {
        assert!(matches!(
            list_connector_wiring(b"<C>".to_vec(), "missing".into()),
            Err(WiringCatalogError::Load { .. })
        ));
        let bytes = include_bytes!("../tests/fixtures/wiring_catalog.xml");
        match list_connector_wiring(bytes.to_vec(), "missing".into()).unwrap_err() {
            WiringCatalogError::NotFound { selector, titles } => {
                assert_eq!(selector, "missing");
                match core::connector_wiring(bytes, "missing").unwrap_err() {
                    EditError::Selection(SelectionError::NotFound {
                        titles: expected, ..
                    }) => assert_eq!(titles, expected),
                    error => panic!("{error}"),
                }
            }
            error => panic!("{error}"),
        }
        match list_connector_wiring(bytes.to_vec(), "Twin".into()).unwrap_err() {
            WiringCatalogError::Ambiguous { selector, matches } => {
                assert_eq!(selector, "Twin");
                assert_eq!(matches.len(), 2);
                assert_eq!(matches[0].uuid, "one");
                assert_eq!(matches[1].uuid, "two");
            }
            error => panic!("{error}"),
        }
    }
}
