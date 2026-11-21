use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ProgramElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl From<core::ElementIdentity> for ProgramElementIdentity {
    fn from(value: core::ElementIdentity) -> Self {
        Self {
            title: value.title,
            block_type: value.block_type,
            uuid: value.uuid,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum ProgramTextError {
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
        matches: Vec<ProgramElementIdentity>,
    },
    #[error("Selected element is not a SequenceController")]
    Unsupported { element: ProgramElementIdentity },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<core::ProgramTextError> for ProgramTextError {
    fn from(error: core::ProgramTextError) -> Self {
        match error {
            core::ProgramTextError::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            core::ProgramTextError::Edit(EditError::Selection(SelectionError::NotFound {
                selector,
                titles,
            })) => Self::NotFound { selector, titles },
            core::ProgramTextError::Edit(EditError::Selection(SelectionError::Ambiguous {
                selector,
                matches,
            })) => Self::Ambiguous {
                selector,
                matches: matches.into_iter().map(Into::into).collect(),
            },
            core::ProgramTextError::Unsupported { element } => Self::Unsupported {
                element: element.into(),
            },
            core::ProgramTextError::Edit(other) => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Read a selected SequenceController's first direct Configuration field as owned text.
///
/// Only direct text nodes contribute, in order; no field or text is empty success.
/// Core supplies selection, exact block eligibility and the shared loader, including
/// its lox-n37w quoted-text/digit-name limitation. There is no program validation,
/// mutation, or file I/O.
#[uniffi::export]
pub fn get_program_text(bytes: Vec<u8>, selector: String) -> Result<String, ProgramTextError> {
    core::get_program_text(&bytes, &selector).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &[u8] = include_bytes!("../tests/fixtures/program_text.xml");

    fn read(selector: &str) -> Result<String, ProgramTextError> {
        get_program_text(XML.to_vec(), selector.into())
    }

    fn identity(title: &str, block_type: &str, uuid: &str) -> ProgramElementIdentity {
        ProgramElementIdentity {
            title: title.into(),
            block_type: block_type.into(),
            uuid: uuid.into(),
        }
    }

    #[test]
    fn owns_exact_core_text_including_empty_and_legacy_loader_observation() {
        let expected = "first & Küche\ne\u{301}secondthird 灯  ";
        for selector in ["uuid:sequence", "gid:cook", "Cook & 灯 [Kitchen]"] {
            assert_eq!(read(selector).unwrap(), expected);
            assert_eq!(core::get_program_text(XML, selector).unwrap(), expected);
        }
        assert_eq!(read("Empty").unwrap(), "");
        assert_eq!(read("Blank").unwrap(), "");
        assert_eq!(read("uuid:legacy").unwrap(), "text _12hTF=value");
    }

    #[test]
    fn preserves_distinct_load_selection_and_type_failures() {
        let invalid = b"<C A='1' A='2'/>";
        let core::ProgramTextError::Edit(EditError::Load(cause)) =
            core::get_program_text(invalid, "missing").unwrap_err()
        else {
            panic!("expected Core load error before selection");
        };
        assert_eq!(
            get_program_text(invalid.to_vec(), "missing".into()).unwrap_err(),
            ProgramTextError::Load {
                source_message: cause.to_string(),
            }
        );
        assert_eq!(
            read("missing").unwrap_err(),
            ProgramTextError::NotFound {
                selector: "missing".into(),
                titles: ["Blank", "Cook & 灯", "Empty", "Kitchen", "Legacy _12hTF=x"]
                    .map(str::to_owned)
                    .to_vec(),
            }
        );
        assert_eq!(
            read("Cook & 灯").unwrap_err(),
            ProgramTextError::Ambiguous {
                selector: "Cook & 灯".into(),
                matches: vec![
                    identity("Cook & 灯", "SequenceController", "sequence"),
                    identity("Cook & 灯", "sequencecontroller", "wrong"),
                ],
            }
        );
        assert_eq!(
            read("uuid:wrong").unwrap_err(),
            ProgramTextError::Unsupported {
                element: identity("Cook & 灯", "sequencecontroller", "wrong"),
            }
        );
        assert_eq!(
            read("uuid:sparse").unwrap_err(),
            ProgramTextError::Unsupported {
                element: identity("", "", "sparse"),
            }
        );
    }
}
