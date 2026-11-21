use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

/// One lowering transaction for XML bytes and the UTF-8 selector.
#[derive(uniffi::Record)]
pub struct LoadManagerInspectionRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LoadManagerInspection {
    pub uuid: String,
    pub title: String,
    pub loads: Vec<LoadManagerLoad>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LoadManagerLoad {
    /// One-based position in XML, not a runtime priority.
    pub record_order: u64,
    pub io_index: u8,
    /// Loader-decoded entries in Core's map key order; absent keys stay absent.
    pub attributes: Vec<LoadManagerAttribute>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LoadManagerAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LoadManagerSelectionIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum LoadManagerInspectionError {
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
        matches: Vec<LoadManagerSelectionIdentity>,
    },
    #[error("Selected element {selector:?} is not a Load Manager")]
    NotLoadManager { selector: String },
    #[error("Expected twelve direct Load records, found {found}")]
    RecordCount { found: u64 },
    #[error("Invalid stored ioIdx: {raw:?}")]
    InvalidIoIndex { raw: String },
    #[error("Duplicate resolved ioIdx: {io_index}")]
    DuplicateIoIndex { io_index: u8 },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<core::LoadManagerInspectionError> for LoadManagerInspectionError {
    fn from(error: core::LoadManagerInspectionError) -> Self {
        use core::LoadManagerInspectionError as Refusal;
        match error {
            Refusal::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            Refusal::Edit(EditError::Selection(SelectionError::NotFound { selector, titles })) => {
                Self::NotFound { selector, titles }
            }
            Refusal::Edit(EditError::Selection(SelectionError::Ambiguous {
                selector,
                matches,
            })) => Self::Ambiguous {
                selector,
                matches: matches
                    .into_iter()
                    .map(|identity| LoadManagerSelectionIdentity {
                        title: identity.title,
                        block_type: identity.block_type,
                        uuid: identity.uuid,
                    })
                    .collect(),
            },
            Refusal::NotLoadManager { selector } => Self::NotLoadManager { selector },
            Refusal::RecordCount { found } => Self::RecordCount {
                found: found as u64,
            },
            Refusal::InvalidIoIndex { raw } => Self::InvalidIoIndex { raw },
            Refusal::DuplicateIoIndex { io_index } => Self::DuplicateIoIndex { io_index },
            Refusal::Edit(other) => Self::other(&other),
            other => Self::other(&other),
        }
    }
}

impl LoadManagerInspectionError {
    fn other(error: &dyn Error) -> Self {
        Self::Other {
            message: error.to_string(),
            source_message: error.source().map(ToString::to_string),
        }
    }
}

/// Read a selected Load Manager's twelve stored records via Core, without I/O or mutation.
#[uniffi::export]
pub fn inspect_load_manager(
    request: LoadManagerInspectionRequest,
) -> Result<LoadManagerInspection, LoadManagerInspectionError> {
    let value = core::inspect_load_manager(&request.bytes, &request.selector)?;
    Ok(LoadManagerInspection {
        uuid: value.uuid,
        title: value.title,
        loads: value
            .loads
            .into_iter()
            .map(|load| LoadManagerLoad {
                record_order: load.priority as u64,
                io_index: load.io_index,
                attributes: load
                    .attributes
                    .into_iter()
                    .map(|(key, value)| LoadManagerAttribute { key, value })
                    .collect(),
            })
            .collect(),
    })
}
