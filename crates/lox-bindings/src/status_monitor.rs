use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

#[derive(uniffi::Record)]
pub struct StatusMonitorRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StatusMonitorInspection {
    pub uuid: String,
    pub title: String,
    pub states: Vec<StatusMonitorState>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StatusMonitorState {
    pub priority: u64,
    pub id: String,
    pub output_index: u8,
    pub configured: bool,
    /// Loader-observed attributes in Core's key order, not XML declaration order.
    pub attributes: Vec<StatusMonitorAttribute>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StatusMonitorAttribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StatusMonitorElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

#[derive(Debug, PartialEq, thiserror::Error, uniffi::Error)]
pub enum StatusMonitorError {
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
        matches: Vec<StatusMonitorElementIdentity>,
    },
    #[error("{selector:?} is not a StatusMonitor")]
    NotStatusMonitor { selector: String },
    #[error("Configuration contains iObjS inheritance")]
    InheritedConfiguration,
    #[error("Expected eleven direct State records; found {found}")]
    StateCount { found: u64 },
    #[error("Namespaced State records are unsupported")]
    NamespacedState,
    #[error("State has no explicit Id")]
    MissingStateId,
    #[error("Invalid State Id {id:?}")]
    InvalidStateId { id: String },
    #[error("Duplicate State Id {id:?}")]
    DuplicateStateId { id: String },
    #[error("State must have an explicit decimal OIdx in 0..=10")]
    InvalidOutputIndex,
    #[error("Duplicate State OIdx {output_index}")]
    DuplicateOutputIndex { output_index: u8 },
    #[error("Rest state must be last")]
    RestNotLast,
    #[error("Invalid State Val {raw:?}")]
    InvalidValue { raw: String },
    #[error("Rest state must omit Val")]
    RestHasValue,
    #[error("Duplicate State Val {value}")]
    DuplicateValue { value: f64 },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<core::StatusMonitorInspectionError> for StatusMonitorError {
    fn from(error: core::StatusMonitorInspectionError) -> Self {
        use core::StatusMonitorInspectionError as Failure;

        match error {
            Failure::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            Failure::Edit(EditError::Selection(SelectionError::NotFound { selector, titles })) => {
                Self::NotFound { selector, titles }
            }
            Failure::Edit(EditError::Selection(SelectionError::Ambiguous {
                selector,
                matches,
            })) => Self::Ambiguous {
                selector,
                matches: matches
                    .into_iter()
                    .map(|identity| StatusMonitorElementIdentity {
                        title: identity.title,
                        block_type: identity.block_type,
                        uuid: identity.uuid,
                    })
                    .collect(),
            },
            Failure::NotStatusMonitor { selector } => Self::NotStatusMonitor { selector },
            Failure::InheritedConfiguration => Self::InheritedConfiguration,
            Failure::StateCount { found } => Self::StateCount {
                found: found as u64,
            },
            Failure::NamespacedState => Self::NamespacedState,
            Failure::MissingStateId => Self::MissingStateId,
            Failure::InvalidStateId { id } => Self::InvalidStateId { id },
            Failure::DuplicateStateId { id } => Self::DuplicateStateId { id },
            Failure::InvalidOutputIndex => Self::InvalidOutputIndex,
            Failure::DuplicateOutputIndex { output_index } => {
                Self::DuplicateOutputIndex { output_index }
            }
            Failure::RestNotLast => Self::RestNotLast,
            Failure::InvalidValue { raw } => Self::InvalidValue { raw },
            Failure::RestHasValue => Self::RestHasValue,
            Failure::DuplicateValue { value } => Self::DuplicateValue { value },
            // Currently reachable load, selection and shape failures all have named variants.
            other => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Read stored standalone Status Monitor states in Core's encounter order.
///
/// The request lowers bytes and selector as a single transaction. The operation
/// uses the shared loader and selector as-is; it does not edit or validate
/// native Config behavior, evaluate live state, or fill in missing attributes.
#[uniffi::export]
pub fn inspect_status_monitor(
    request: StatusMonitorRequest,
) -> Result<StatusMonitorInspection, StatusMonitorError> {
    let result = core::inspect_status_monitor(&request.bytes, &request.selector)?;
    Ok(StatusMonitorInspection {
        uuid: result.uuid,
        title: result.title,
        states: result
            .states
            .into_iter()
            .map(|state| StatusMonitorState {
                priority: state.priority as u64,
                id: state.id,
                output_index: state.output_index,
                configured: state.configured,
                attributes: state
                    .attributes
                    .into_iter()
                    .map(|(name, value)| StatusMonitorAttribute { name, value })
                    .collect(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_named_shape_errors_without_erasing_their_payloads() {
        let xml = include_str!("../tests/fixtures/status_monitor.xml");
        let id = "A0000000-0000-0000-0000000000000001";
        for (old, new, expected) in [
            (
                "<State OIdx=\"02\"",
                "<x:State xmlns:x=\"urn:test\" OIdx=\"02\"",
                StatusMonitorError::NamespacedState,
            ),
            (
                " Id=\"A0000000-0000-0000-0000000000000001\"",
                "",
                StatusMonitorError::MissingStateId,
            ),
            (
                id,
                "00000000-0000-0000-0000000000000000",
                StatusMonitorError::InvalidStateId {
                    id: "00000000-0000-0000-0000000000000000".into(),
                },
            ),
            (
                " OIdx=\"02\"",
                " OIdx=\"+2\"",
                StatusMonitorError::InvalidOutputIndex,
            ),
            (
                "OIdx=\"02\"",
                "OIdx=\"10\"",
                StatusMonitorError::RestNotLast,
            ),
            (
                "OIdx=\"10\"",
                "OIdx=\"10\" Val=\"42\"",
                StatusMonitorError::RestHasValue,
            ),
        ] {
            let changed = xml.replacen(old, new, 1);
            assert_ne!(changed, xml);
            let result = inspect_status_monitor(StatusMonitorRequest {
                bytes: changed.into_bytes(),
                selector: "uuid:mon".into(),
            });
            assert_eq!(result.unwrap_err(), expected);
        }
    }

    #[test]
    fn preserves_parser_cause_text_in_the_load_variant() {
        let bytes = b"<C A='1' A='2'/>";
        let core::StatusMonitorInspectionError::Edit(EditError::Load(cause)) =
            core::inspect_status_monitor(bytes, "missing").unwrap_err()
        else {
            panic!("expected Core load error before selection");
        };
        assert_eq!(
            inspect_status_monitor(StatusMonitorRequest {
                bytes: bytes.to_vec(),
                selector: "missing".into(),
            })
            .unwrap_err(),
            StatusMonitorError::Load {
                source_message: cause.to_string()
            }
        );
    }
}
