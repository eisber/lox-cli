use crate::mood_inventory::MoodInfo;
use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::{collections::BTreeMap, error::Error};

/// One lowering transaction for XML bytes and the UTF-8 selector.
#[derive(uniffi::Record)]
pub struct LightingInspectionRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LightingInspection {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub attributes: Vec<LightingAttribute>,
    pub channels: Vec<StoredLightingChannel>,
    pub moods: Vec<StoredLightingMood>,
    pub nested_storage: Vec<LightingStorage>,
    pub direct_scene_collection_count: u64,
    /// Zero-based index into `nested_storage`, if found by Core.
    pub sole_scene_collection: Option<u64>,
    pub document_versions: Vec<LightingDocumentVersion>,
}

/// Attribute entries retain the Core map's key order.
#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LightingAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LightingDocumentVersion {
    pub attributes: Vec<LightingAttribute>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LightingStorage {
    pub element: String,
    pub attributes: Vec<LightingAttribute>,
    pub children: Vec<LightingStorage>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StoredLightingMood {
    pub summary: MoodInfo,
    pub attributes: Vec<LightingAttribute>,
    pub in_sole_direct_collection: bool,
    pub outputs: Vec<StoredLightingOutput>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StoredLightingOutput {
    pub key: String,
    pub index: u64,
    pub stored: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StoredLightingChannel {
    pub key: String,
    pub index: u64,
    pub uuid: Option<String>,
    pub attributes: Vec<LightingAttribute>,
    pub uuid_occurrences: Option<u64>,
    pub downstream: Vec<StoredLightingWire>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct StoredLightingWire {
    pub block_uuid: Option<String>,
    pub block_type: Option<String>,
    pub block_title: Option<String>,
    pub connector_uuid: Option<String>,
    pub connector_key: Option<String>,
    pub wire_attributes: Vec<LightingAttribute>,
}

fn attributes(map: BTreeMap<String, String>) -> Vec<LightingAttribute> {
    map.into_iter()
        .map(|(key, value)| LightingAttribute { key, value })
        .collect()
}

impl From<core::LightingInspection> for LightingInspection {
    fn from(value: core::LightingInspection) -> Self {
        Self {
            uuid: value.uuid,
            title: value.title,
            attributes: attributes(value.attributes),
            channels: value.channels.into_iter().map(Into::into).collect(),
            moods: value.moods.into_iter().map(Into::into).collect(),
            nested_storage: value.nested_storage.into_iter().map(Into::into).collect(),
            direct_scene_collection_count: value.direct_scene_collection_count as u64,
            sole_scene_collection: value.sole_scene_collection.map(|index| index as u64),
            document_versions: value
                .document_versions
                .into_iter()
                .map(|map| LightingDocumentVersion {
                    attributes: attributes(map),
                })
                .collect(),
        }
    }
}

impl From<core::LightingStorage> for LightingStorage {
    fn from(value: core::LightingStorage) -> Self {
        Self {
            element: value.element,
            attributes: attributes(value.attributes),
            children: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<core::StoredLightingMood> for StoredLightingMood {
    fn from(value: core::StoredLightingMood) -> Self {
        Self {
            summary: value.summary.into(),
            attributes: attributes(value.attributes),
            in_sole_direct_collection: value.in_sole_direct_collection,
            outputs: value
                .outputs
                .into_iter()
                .map(|output| StoredLightingOutput {
                    key: output.key,
                    index: output.index as u64,
                    stored: output.stored,
                })
                .collect(),
        }
    }
}

impl From<core::StoredLightingChannel> for StoredLightingChannel {
    fn from(value: core::StoredLightingChannel) -> Self {
        Self {
            key: value.key,
            index: value.index as u64,
            uuid: value.uuid,
            attributes: attributes(value.attributes),
            uuid_occurrences: value.uuid_occurrences.map(|count| count as u64),
            downstream: value.downstream.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<core::StoredLightingWire> for StoredLightingWire {
    fn from(value: core::StoredLightingWire) -> Self {
        Self {
            block_uuid: value.block_uuid,
            block_type: value.block_type,
            block_title: value.block_title,
            connector_uuid: value.connector_uuid,
            connector_key: value.connector_key,
            wire_attributes: attributes(value.wire_attributes),
        }
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct LightingSelectionIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum LightingInspectionError {
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
        matches: Vec<LightingSelectionIdentity>,
    },
    #[error("Selected element {selector:?} has type {block_type:?}, not LightController2")]
    NotLightController {
        selector: String,
        block_type: String,
    },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<core::LightingInspectionError> for LightingInspectionError {
    fn from(error: core::LightingInspectionError) -> Self {
        use core::LightingInspectionError as Refusal;
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
                    .map(|identity| LightingSelectionIdentity {
                        title: identity.title,
                        block_type: identity.block_type,
                        uuid: identity.uuid,
                    })
                    .collect(),
            },
            Refusal::NotLightController {
                selector,
                block_type,
            } => Self::NotLightController {
                selector,
                block_type,
            },
            Refusal::Edit(other) => Self::other(&other),
            other => Self::other(&other),
        }
    }
}

impl LightingInspectionError {
    fn other(error: &dyn Error) -> Self {
        Self::Other {
            message: error.to_string(),
            source_message: error.source().map(ToString::to_string),
        }
    }
}

/// Read the published Core stored Lighting projection without I/O or interpretation.
#[uniffi::export]
pub fn inspect_lighting(
    request: LightingInspectionRequest,
) -> Result<LightingInspection, LightingInspectionError> {
    core::inspect_lighting(&request.bytes, &request.selector)
        .map(Into::into)
        .map_err(Into::into)
}
