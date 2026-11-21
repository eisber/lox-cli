use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::{error::Error, num::IntErrorKind};

/// One lowering transaction for XML bytes and the UTF-8 selector.
#[derive(uniffi::Record)]
pub struct CentralLightLinksRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct CentralLightLink {
    pub position: u64,
    pub uuid: String,
    pub title: String,
    pub control_type: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct CentralLightLinks {
    pub uuid: String,
    pub title: String,
    pub links: Vec<CentralLightLink>,
}

impl From<core::CentralLightLinks> for CentralLightLinks {
    fn from(value: core::CentralLightLinks) -> Self {
        Self {
            uuid: value.uuid,
            title: value.title,
            links: value
                .links
                .into_iter()
                .map(|link| CentralLightLink {
                    position: link.position as u64,
                    uuid: link.uuid,
                    title: link.title,
                    control_type: link.control_type,
                })
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct CentralLightElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl From<core::ElementIdentity> for CentralLightElementIdentity {
    fn from(value: core::ElementIdentity) -> Self {
        Self {
            title: value.title,
            block_type: value.block_type,
            uuid: value.uuid,
        }
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Enum)]
pub enum CentralLightFlagsCause {
    Empty,
    InvalidDigit,
    PositiveOverflow,
    NegativeOverflow,
    Zero,
    Unknown,
}

impl From<&IntErrorKind> for CentralLightFlagsCause {
    fn from(kind: &IntErrorKind) -> Self {
        match kind {
            IntErrorKind::Empty => Self::Empty,
            IntErrorKind::InvalidDigit => Self::InvalidDigit,
            IntErrorKind::PosOverflow => Self::PositiveOverflow,
            IntErrorKind::NegOverflow => Self::NegativeOverflow,
            IntErrorKind::Zero => Self::Zero,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum CentralLightLinksError {
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
        matches: Vec<CentralLightElementIdentity>,
    },
    #[error("'{selector}' is not a plain CentralLight")]
    NotPlainCentralLight { selector: String },
    #[error("CentralLight links require one owning Document")]
    NoSingleOwningDocument,
    #[error("CentralLight links require a plain owning Document")]
    NotPlainDocument,
    #[error("CentralLight links do not support Document Template configurations")]
    DocumentTemplate,
    #[error("CentralLight links require one direct IoData with explicit Visu=\"true\"")]
    NotDirectVisibleOwner,
    #[error("CentralLight has no U identity")]
    MissingOwnerIdentity,
    #[error("'{uuid}' is not a nonzero native UUID (8-4-4-16 hexadecimal digits)")]
    InvalidIdentity { uuid: String },
    #[error("Duplicate linked-function UUID '{uuid}'")]
    DuplicateReference { uuid: String },
    #[error("Linked-function UUID '{uuid}' does not resolve to exactly one plain control")]
    UnresolvedControl { uuid: String },
    #[error("Unsupported decimal WF '{value}': {source_message}")]
    InvalidFlags {
        value: String,
        kind: CentralLightFlagsCause,
        source_message: String,
    },
    #[error("Inconsistent document linked-function WF marker on {identity:?}")]
    InconsistentMarker { identity: Option<String> },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<core::CentralLightLinksError> for CentralLightLinksError {
    fn from(error: core::CentralLightLinksError) -> Self {
        use core::CentralLightLinksError as Refusal;
        match error {
            Refusal::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            Refusal::Edit(EditError::Selection(SelectionError::NotFound { selector, titles })) => {
                Self::NotFound { selector, titles }
            }
            Refusal::Edit(EditError::Selection(SelectionError::Ambiguous { selector, matches })) => {
                Self::Ambiguous {
                    selector,
                    matches: matches.into_iter().map(Into::into).collect(),
                }
            }
            Refusal::Edit(other) => Self::other(&other),
            Refusal::NotPlainCentralLight { selector } => Self::NotPlainCentralLight { selector },
            Refusal::NoSingleOwningDocument => Self::NoSingleOwningDocument,
            Refusal::NotPlainDocument => Self::NotPlainDocument,
            Refusal::DocumentTemplate => Self::DocumentTemplate,
            Refusal::NotDirectVisibleOwner => Self::NotDirectVisibleOwner,
            Refusal::MissingOwnerIdentity => Self::MissingOwnerIdentity,
            Refusal::InvalidIdentity { uuid } => Self::InvalidIdentity { uuid },
            Refusal::DuplicateReference { uuid } => Self::DuplicateReference { uuid },
            Refusal::UnresolvedControl { uuid } => Self::UnresolvedControl { uuid },
            Refusal::InvalidFlags { value, cause } => Self::InvalidFlags {
                value,
                kind: cause.kind().into(),
                source_message: cause.to_string(),
            },
            Refusal::InconsistentMarker { identity } => Self::InconsistentMarker { identity },
            other => Self::other(&other),
        }
    }
}

impl CentralLightLinksError {
    fn other(error: &dyn Error) -> Self {
        Self::Other {
            message: error.to_string(),
            source_message: error.source().map(ToString::to_string),
        }
    }
}

/// Read owned, ordered stored `linkC` records through Core, without any I/O or edits.
#[uniffi::export]
pub fn inspect_central_light_links(
    request: CentralLightLinksRequest,
) -> Result<CentralLightLinks, CentralLightLinksError> {
    core::inspect_central_light_links(&request.bytes, &request.selector)
        .map(Into::into)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = include_str!("../tests/fixtures/central_light_links.xml");
    const OWNER: &str = "ABCDEF01-0002-0003-0000000000000001";
    const FIRST: &str = "F00BABE1-0002-0003-000000000000000A";
    const SECOND: &str = "f00babe2-0002-0003-000000000000000b";

    fn inspect(xml: &str, selector: &str) -> Result<CentralLightLinks, CentralLightLinksError> {
        inspect_central_light_links(CentralLightLinksRequest {
            bytes: xml.as_bytes().into(),
            selector: selector.into(),
        })
    }

    #[test]
    fn keeps_stored_order_reference_spelling_and_empty_fields() {
        let value = inspect(XML, "Zentrale & 灯 [dEn]").unwrap();
        assert_eq!(
            (value.uuid.as_str(), value.title.as_str()),
            (OWNER, "Zentrale & 灯")
        );
        assert_eq!(
            value.links,
            [
                CentralLightLink {
                    position: 1,
                    uuid: FIRST.into(),
                    title: String::new(),
                    control_type: String::new(),
                },
                CentralLightLink {
                    position: 2,
                    uuid: SECOND.into(),
                    title: "Küche & status".into(),
                    control_type: "Not".into(),
                },
            ]
        );
        assert_eq!(inspect(XML, "gid:PRIMARY").unwrap().links, value.links);
        assert!(inspect(XML, "Empty").unwrap().links.is_empty());
    }

    #[test]
    fn retains_load_cause_typed_flags_and_optional_marker_identity() {
        let malformed = "<C A='1' A='2'/>";
        let core::CentralLightLinksError::Edit(EditError::Load(cause)) =
            core::inspect_central_light_links(malformed.as_bytes(), "missing").unwrap_err()
        else {
            panic!("expected Core load error before selection");
        };
        assert_eq!(
            inspect(malformed, "missing").unwrap_err(),
            CentralLightLinksError::Load {
                source_message: cause.to_string(),
            }
        );
        let bad = XML.replace("268451840", "4294967296");
        let core::CentralLightLinksError::InvalidFlags { cause, .. } =
            core::inspect_central_light_links(bad.as_bytes(), "gid:primary").unwrap_err()
        else {
            panic!("expected Core flags error");
        };
        assert_eq!(
            inspect(&bad, "gid:primary").unwrap_err(),
            CentralLightLinksError::InvalidFlags {
                value: "4294967296".into(),
                kind: CentralLightFlagsCause::PositiveOverflow,
                source_message: cause.to_string(),
            }
        );
        for (replacement, identity) in [
            ("16384", Some(SECOND.to_owned())),
            ("268451840\"/><C WF=\"268435456", None),
        ] {
            assert_eq!(
                inspect(&XML.replace("268451840", replacement), "gid:primary").unwrap_err(),
                CentralLightLinksError::InconsistentMarker { identity }
            );
        }
    }
}
