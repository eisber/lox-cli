//! Owned three-way bytes and a linear, overlay-only typed error projection.

use lox_core::configuration::runtime_overlay as core;
use std::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum OverlayXmlContext {
    Document,
    Bare,
}

impl From<core::XmlContext> for OverlayXmlContext {
    fn from(value: core::XmlContext) -> Self {
        match value {
            core::XmlContext::Document => Self::Document,
            core::XmlContext::Bare => Self::Bare,
        }
    }
}

/// Outer-to-inner domain frames. Native causes terminate at their typed frame;
/// XML reader/attribute diagnostics remain native Display text, not Debug.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum OverlayErrorFrame {
    DocumentUtf8 {
        valid_up_to: u64,
        error_len: Option<u64>,
    },
    Utf8 {
        valid_up_to: u64,
        error_len: Option<u64>,
    },
    StringUtf8 {
        bytes: Vec<u8>,
        valid_up_to: u64,
        error_len: Option<u64>,
    },
    Xml {
        context: OverlayXmlContext,
        source_message: String,
    },
    Attribute {
        context: OverlayXmlContext,
        source_message: String,
    },
    DuplicateUuid {
        id: String,
    },
    MissingControlType,
    UnexpectedClosingTag,
    Dtd,
    OutsideRoot,
    IncompleteRoot,
    ShellConflict {
        id: String,
    },
    ControlConflict {
        id: String,
    },
    Placement,
    MergedConsistency,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum OverlayError {
    #[error("Offline three-way merge refused")]
    Refused { frames: Vec<OverlayErrorFrame> },
    /// No diagnostic payload is flattened for a new or unsupported Core variant.
    #[error("Unsupported Core overlay error variant")]
    UnsupportedCoreVariant,
}

impl From<core::OverlayError> for OverlayError {
    fn from(error: core::OverlayError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        let mut frames = Vec::new();
        let mut current = Some(&error);
        while let Some(value) = current {
            frames.push(match value {
                core::OverlayError::DocumentUtf8 { source } => OverlayErrorFrame::DocumentUtf8 {
                    valid_up_to: source.valid_up_to() as u64,
                    error_len: source.error_len().map(|n| n as u64),
                },
                core::OverlayError::Utf8 { source } => OverlayErrorFrame::Utf8 {
                    valid_up_to: source.valid_up_to() as u64,
                    error_len: source.error_len().map(|n| n as u64),
                },
                core::OverlayError::StringUtf8 { source } => OverlayErrorFrame::StringUtf8 {
                    bytes: source.as_bytes().to_vec(),
                    valid_up_to: source.utf8_error().valid_up_to() as u64,
                    error_len: source.utf8_error().error_len().map(|n| n as u64),
                },
                core::OverlayError::Xml { source, context } => OverlayErrorFrame::Xml {
                    context: (*context).into(),
                    source_message: source.to_string(),
                },
                core::OverlayError::Attribute { source, context } => OverlayErrorFrame::Attribute {
                    context: (*context).into(),
                    source_message: source.to_string(),
                },
                core::OverlayError::DuplicateUuid { id } => {
                    OverlayErrorFrame::DuplicateUuid { id: id.clone() }
                }
                core::OverlayError::MissingControlType => OverlayErrorFrame::MissingControlType,
                core::OverlayError::UnexpectedClosingTag => OverlayErrorFrame::UnexpectedClosingTag,
                core::OverlayError::Dtd => OverlayErrorFrame::Dtd,
                core::OverlayError::OutsideRoot => OverlayErrorFrame::OutsideRoot,
                core::OverlayError::IncompleteRoot => OverlayErrorFrame::IncompleteRoot,
                core::OverlayError::ShellConflict { id } => {
                    OverlayErrorFrame::ShellConflict { id: id.clone() }
                }
                core::OverlayError::ControlConflict { id } => {
                    OverlayErrorFrame::ControlConflict { id: id.clone() }
                }
                core::OverlayError::Placement => OverlayErrorFrame::Placement,
                core::OverlayError::MergedConsistency { .. } => {
                    OverlayErrorFrame::MergedConsistency
                }
                _ => return Self::UnsupportedCoreVariant,
            });
            current = value
                .source()
                .and_then(|source| source.downcast_ref::<core::OverlayError>());
        }
        Self::Refused { frames }
    }
}

/// Merge already acquired XML bytes using only Core's byte-preserving kernel.
/// This performs no acquisition, baseline, schema or deployment validation.
#[uniffi::export]
pub fn three_way(base: Vec<u8>, local: Vec<u8>, live: Vec<u8>) -> Result<Vec<u8>, OverlayError> {
    core::three_way(&base, &local, &live).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(error: OverlayError) -> Vec<OverlayErrorFrame> {
        match error {
            OverlayError::Refused { frames } => frames,
            other => panic!("unexpected refusal: {other}"),
        }
    }

    #[test]
    fn selected_raw_bytes_and_conflicts_are_owned_core_results() {
        let base = br#"<Root><C U="a" Type="And" V="0"/><C U="b" Type="Or" V="0"/></Root>"#;
        let local = b"\xef\xbb\xbf<Root>\r\n<C V=\"1\" U=\"a\" Type=\"And\"></C><C U=\"b\" Type=\"Or\" V=\"0\"/>\r\n</Root>";
        let live = br#"<Root><C U="a" Type="And" V="0"/><C U="b" Type="Or" V="2"/></Root>"#;
        let expected = core::three_way(base, local, live).unwrap();
        assert_eq!(
            three_way(base.to_vec(), local.to_vec(), live.to_vec()).unwrap(),
            expected
        );
        assert_eq!(
            three_way(base.to_vec(), base.to_vec(), local.to_vec()).unwrap(),
            local
        );
        let conflict = br#"<Root><C U="a" Type="And" V="2"/><C U="b" Type="Or" V="0"/></Root>"#;
        assert_eq!(
            frames(three_way(base.to_vec(), local.to_vec(), conflict.to_vec()).unwrap_err()),
            [OverlayErrorFrame::ShellConflict { id: "a".into() }]
        );
    }

    #[test]
    fn merged_duplicate_retains_the_actual_two_domain_sources_in_order() {
        let base = br#"<Root><C U="a" Type="And"></C><C U="b" Type="And"></C></Root>"#;
        let local = br#"<Root><C U="a" Type="And"><C U="c" Type="And"/></C><C U="b" Type="And"></C></Root>"#;
        let live = br#"<Root><C U="a" Type="And"></C><C U="b" Type="And"><C U="c" Type="And"/></C></Root>"#;
        let error = core::three_way(base, local, live).unwrap_err();
        let source = error.source().unwrap();
        assert!(source.is::<core::OverlayError>());
        assert!(source.source().is_none());
        assert_eq!(
            frames(three_way(base.to_vec(), local.to_vec(), live.to_vec()).unwrap_err()),
            [
                OverlayErrorFrame::MergedConsistency,
                OverlayErrorFrame::DuplicateUuid { id: "c".into() }
            ]
        );
    }

    #[test]
    fn source_chain_has_no_arbitrary_cutoff_or_partial_unsupported_projection() {
        let mut error = core::OverlayError::DuplicateUuid { id: "last".into() };
        for _ in 0..257 {
            error = core::OverlayError::MergedConsistency {
                source: Box::new(error),
            };
        }
        let mut projected = frames(error.into());
        assert_eq!(projected.len(), 258);
        assert_eq!(
            projected.pop(),
            Some(OverlayErrorFrame::DuplicateUuid { id: "last".into() })
        );
        assert!(
            projected
                .iter()
                .all(|frame| *frame == OverlayErrorFrame::MergedConsistency)
        );
        let unsupported = core::OverlayError::MergedConsistency {
            source: Box::new(core::OverlayError::FilenameSuffix),
        };
        assert!(matches!(
            OverlayError::from(unsupported),
            OverlayError::UnsupportedCoreVariant
        ));
    }

    #[test]
    fn native_boundaries_retain_context_display_and_utf8_byte_units() {
        for data in [b"<Root a=\"1\" a=\"2\"/>".as_slice(), b"<Root></wrong>"] {
            let error = core::three_way(data, b"<Root/>", b"<Root/>").unwrap_err();
            let message = error.source().unwrap().to_string();
            match frames(error.into()).as_slice() {
                [
                    OverlayErrorFrame::Xml {
                        context,
                        source_message,
                    },
                ]
                | [
                    OverlayErrorFrame::Attribute {
                        context,
                        source_message,
                    },
                ] => {
                    assert_eq!(*context, OverlayXmlContext::Document);
                    assert_eq!(*source_message, message);
                }
                other => panic!("unexpected frames: {other:?}"),
            }
        }
        let invalid = vec![b'<', 0xff];
        let native_error = std::str::from_utf8(&invalid).unwrap_err();
        assert_eq!(
            frames(
                core::OverlayError::Utf8 {
                    source: native_error
                }
                .into()
            ),
            [OverlayErrorFrame::Utf8 {
                valid_up_to: 1,
                error_len: Some(1)
            }]
        );
        assert_eq!(
            frames(three_way(invalid, b"<Root/>".to_vec(), b"<Root/>".to_vec()).unwrap_err()),
            [OverlayErrorFrame::DocumentUtf8 {
                valid_up_to: 1,
                error_len: Some(1)
            }]
        );
        let error = core::three_way(b"<Root></wrong>", b"<Root/>", b"<Root/>").unwrap_err();
        if let core::OverlayError::Xml { source, .. } = error {
            let message = source.to_string();
            assert_eq!(
                frames(
                    core::OverlayError::Xml {
                        source,
                        context: core::XmlContext::Bare
                    }
                    .into()
                ),
                [OverlayErrorFrame::Xml {
                    context: OverlayXmlContext::Bare,
                    source_message: message
                }]
            );
        } else {
            panic!("expected Core XML reader cause");
        }
        let invalid = vec![0xe2, 0x82];
        let error = String::from_utf8(invalid.clone()).unwrap_err();
        assert_eq!(
            frames(core::OverlayError::StringUtf8 { source: error }.into()),
            [OverlayErrorFrame::StringUtf8 {
                bytes: invalid,
                valid_up_to: 0,
                error_len: None
            }]
        );
    }

    #[test]
    fn domain_payloads_and_unsupported_refusal_are_not_stringified() {
        for (error, expected) in [
            (
                core::OverlayError::DuplicateUuid { id: " Raw ".into() },
                OverlayErrorFrame::DuplicateUuid { id: " Raw ".into() },
            ),
            (
                core::OverlayError::ControlConflict { id: "id".into() },
                OverlayErrorFrame::ControlConflict { id: "id".into() },
            ),
            (core::OverlayError::Placement, OverlayErrorFrame::Placement),
            (core::OverlayError::Dtd, OverlayErrorFrame::Dtd),
            (
                core::OverlayError::OutsideRoot,
                OverlayErrorFrame::OutsideRoot,
            ),
            (
                core::OverlayError::IncompleteRoot,
                OverlayErrorFrame::IncompleteRoot,
            ),
            (
                core::OverlayError::MissingControlType,
                OverlayErrorFrame::MissingControlType,
            ),
            (
                core::OverlayError::UnexpectedClosingTag,
                OverlayErrorFrame::UnexpectedClosingTag,
            ),
        ] {
            assert_eq!(frames(error.into()), [expected]);
        }
        assert!(matches!(
            OverlayError::from(core::OverlayError::FilenameSuffix),
            OverlayError::UnsupportedCoreVariant
        ));
    }
}
