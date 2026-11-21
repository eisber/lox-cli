use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

/// One lowering transaction for XML bytes and the selector.
#[derive(uniffi::Record)]
pub struct MoodInventoryRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
}

/// One stored mood, with no interpretation of the raw output value.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct MoodInfo {
    pub name: String,
    pub uuid: String,
    pub sid: Option<i64>,
    pub cid: Option<i64>,
    pub q1: Option<String>,
}

impl From<core::MoodInfo> for MoodInfo {
    fn from(value: core::MoodInfo) -> Self {
        Self {
            name: value.name,
            uuid: value.uuid,
            sid: value.sid,
            cid: value.cid,
            q1: value.q1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct MoodElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl From<core::ElementIdentity> for MoodElementIdentity {
    fn from(value: core::ElementIdentity) -> Self {
        Self {
            title: value.title,
            block_type: value.block_type,
            uuid: value.uuid,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum MoodInventoryError {
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
        matches: Vec<MoodElementIdentity>,
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

impl From<core::MoodInventoryError> for MoodInventoryError {
    fn from(error: core::MoodInventoryError) -> Self {
        match error {
            core::MoodInventoryError::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            core::MoodInventoryError::Edit(EditError::Selection(SelectionError::NotFound {
                selector,
                titles,
            })) => Self::NotFound { selector, titles },
            core::MoodInventoryError::Edit(EditError::Selection(SelectionError::Ambiguous {
                selector,
                matches,
            })) => Self::Ambiguous {
                selector,
                matches: matches.into_iter().map(Into::into).collect(),
            },
            core::MoodInventoryError::NotLightController {
                selector,
                block_type,
            } => Self::NotLightController {
                selector,
                block_type,
            },
            core::MoodInventoryError::Edit(other) => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Read stored moods in Core document order from one selected `LightController2`.
///
/// The legacy loader, selector, field defaults and exact block type check stay in Core.
/// This API has no color decoding, mutation or file access.
#[uniffi::export]
pub fn list_moods(request: MoodInventoryRequest) -> Result<Vec<MoodInfo>, MoodInventoryError> {
    core::list_moods(&request.bytes, &request.selector)
        .map(|moods| moods.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &[u8] = include_bytes!("../tests/fixtures/mood_inventory.xml");

    fn list(bytes: &[u8], selector: &str) -> Result<Vec<MoodInfo>, MoodInventoryError> {
        list_moods(MoodInventoryRequest {
            bytes: bytes.to_vec(),
            selector: selector.into(),
        })
    }

    #[test]
    fn projects_ordered_duplicate_records_and_optional_fields() {
        let moods = list(XML, "Lamps [kit]").unwrap();
        let fields: Vec<_> = moods
            .iter()
            .map(|mood| {
                (
                    mood.name.as_str(),
                    mood.uuid.as_str(),
                    mood.sid,
                    mood.cid,
                    mood.q1.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            fields,
            [
                ("Outer & 灯", "same", Some(2), Some(-3), Some("00037")),
                ("Inner", "same", Some(2), None, Some("")),
                ("", "", None, None, None),
                ("", "", None, Some(i64::MAX), Some("opaque&unparsed")),
            ]
        );
        assert_eq!(list(XML, "gid:LIGHT").unwrap(), moods);
        assert!(list(XML, "uuid:empty").unwrap().is_empty());
    }

    #[test]
    fn projects_only_relevant_editor_failures_as_distinct_types() {
        let malformed = b"<C A='1' A='2'/>";
        let core::MoodInventoryError::Edit(EditError::Load(cause)) =
            core::list_moods(malformed, "missing").unwrap_err()
        else {
            panic!("expected Core load error before selection");
        };
        assert_eq!(
            list(malformed, "missing").unwrap_err(),
            MoodInventoryError::Load {
                source_message: cause.to_string(),
            }
        );
        assert!(matches!(
            list(XML, "uuid:LC"),
            Err(MoodInventoryError::NotFound { selector, titles })
                if selector == "uuid:LC"
                    && titles == ["Case only", "Kitchen", "Kitchen lamps", "Mixed lights"]
        ));
        assert!(matches!(
            list(XML, "Mixed lights"),
            Err(MoodInventoryError::Ambiguous { selector, matches })
                if selector == "Mixed lights"
                    && matches.iter().map(|m| (m.title.as_str(), m.block_type.as_str(), m.uuid.as_str()))
                        .eq([("Mixed lights", "And", "a"), ("Mixed lights", "LightController2", "b")])
        ));
        for (selector, block_type) in [("uuid:case", "lightcontroller2"), ("uuid:sparse", "")] {
            assert_eq!(
                list(XML, selector).unwrap_err(),
                MoodInventoryError::NotLightController {
                    selector: selector.into(),
                    block_type: block_type.into(),
                }
            );
        }
    }
}
