//! Owned offline fragment application; the transaction and its checks stay in Core.

use crate::runtime_overlay::OverlayErrorFrame;
use lox_core::configuration::{archive::ArchiveError, runtime_overlay as core};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    error::Error,
};

/// One filename and the caller's exact compressed LoxCC bytes.
#[derive(Debug, Clone, uniffi::Record)]
pub struct LoxccFragment {
    pub name: String,
    pub data: Vec<u8>,
}

/// Outer-to-inner typed causes of a Core fragment-application refusal.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FragmentApplicationErrorFrame {
    Overlay { frame: OverlayErrorFrame },
    FilenameSuffix,
    InvalidFilename { name: String },
    DuplicateOverlay { id: String },
    ParentChildOverlap,
    FragmentRead { name: String },
    Archive,
    ArchiveHeaderTooShort { actual: u64 },
    ArchiveInvalidMagic { actual: u32 },
    ArchiveZeroBackReference { position: u64 },
    ArchiveBackReferenceOutOfBounds { offset: u64, output_len: u64 },
    FragmentIntegrity,
    NotControl,
    FragmentOverlap { name: String },
    UnsupportedRoot { name: String },
    InvalidControlRoot { name: String },
    TypeChange { name: String },
    RulePlacement { name: String },
    UnknownControl { name: String, kind: String },
    UuidConsistency { name: String },
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FragmentApplicationError {
    #[error("Offline LoxCC fragment application refused")]
    Refused {
        frames: Vec<FragmentApplicationErrorFrame>,
    },
    #[error("Unsupported Core fragment application error variant")]
    UnsupportedCoreVariant,
    #[error("Repeated runtime overlay filename {name}")]
    DuplicateFilename { name: String },
}

impl FragmentApplicationErrorFrame {
    fn overlay(value: &core::OverlayError) -> Option<Self> {
        use core::OverlayError as E;
        Some(match value {
            E::FilenameSuffix => Self::FilenameSuffix,
            E::InvalidFilename { name } => Self::InvalidFilename { name: name.clone() },
            E::DuplicateOverlay { id } => Self::DuplicateOverlay { id: id.clone() },
            E::ParentChildOverlap => Self::ParentChildOverlap,
            E::FragmentRead { name, .. } => Self::FragmentRead { name: name.clone() },
            E::Archive { .. } => Self::Archive,
            E::FragmentIntegrity => Self::FragmentIntegrity,
            E::NotControl => Self::NotControl,
            E::FragmentOverlap { name } => Self::FragmentOverlap { name: name.clone() },
            E::UnsupportedRoot { name } => Self::UnsupportedRoot { name: name.clone() },
            E::InvalidControlRoot { name } => Self::InvalidControlRoot { name: name.clone() },
            E::TypeChange { name } => Self::TypeChange { name: name.clone() },
            E::RulePlacement { name } => Self::RulePlacement { name: name.clone() },
            E::UnknownControl { name, kind } => Self::UnknownControl {
                name: name.clone(),
                kind: kind.clone(),
            },
            E::UuidConsistency { name, .. } => Self::UuidConsistency { name: name.clone() },
            E::StringUtf8 { .. }
            | E::ShellConflict { .. }
            | E::ControlConflict { .. }
            | E::Placement
            | E::MergedConsistency { .. } => return None,
            other => Self::Overlay {
                frame: OverlayErrorFrame::project(other)?,
            },
        })
    }

    fn archive(value: &ArchiveError) -> Option<Self> {
        const { assert!(usize::BITS <= u64::BITS) };
        Some(match *value {
            ArchiveError::HeaderTooShort { actual } => Self::ArchiveHeaderTooShort {
                actual: actual as u64,
            },
            ArchiveError::InvalidMagic { actual } => Self::ArchiveInvalidMagic { actual },
            ArchiveError::ZeroBackReference { position } => Self::ArchiveZeroBackReference {
                position: position as u64,
            },
            ArchiveError::BackReferenceOutOfBounds { offset, output_len } => {
                Self::ArchiveBackReferenceOutOfBounds {
                    offset: offset as u64,
                    output_len: output_len as u64,
                }
            }
            _ => return None,
        })
    }
}

impl From<core::OverlayError> for FragmentApplicationError {
    fn from(error: core::OverlayError) -> Self {
        let mut frames = Vec::new();
        let mut current = &error as &(dyn Error + 'static);
        loop {
            if let Some(value) = current.downcast_ref::<core::OverlayError>() {
                let Some(frame) = FragmentApplicationErrorFrame::overlay(value) else {
                    return Self::UnsupportedCoreVariant;
                };
                let domain_source = matches!(
                    frame,
                    FragmentApplicationErrorFrame::Archive
                        | FragmentApplicationErrorFrame::FragmentRead { .. }
                        | FragmentApplicationErrorFrame::UuidConsistency { .. }
                );
                frames.push(frame);
                if !domain_source {
                    break;
                }
                let Some(source) = value.source() else {
                    return Self::UnsupportedCoreVariant;
                };
                current = source;
            } else if let Some(value) = current.downcast_ref::<ArchiveError>() {
                let Some(frame) = FragmentApplicationErrorFrame::archive(value) else {
                    return Self::UnsupportedCoreVariant;
                };
                frames.push(frame);
                break;
            } else {
                return Self::UnsupportedCoreVariant;
            }
        }
        Self::Refused { frames }
    }
}

/// Apply supplied LoxCC fragments by filename order, without I/O.
///
/// The entry sequence preserves repeated exact names until they can be rejected
/// before Core sees the unique-key map. The lowest repeated filename is
/// reported, as with the CLI's sorted listing. Core handles aliases and all
/// config checks.
#[uniffi::export]
pub fn merge_overlays(
    base: Vec<u8>,
    fragments: Vec<LoxccFragment>,
) -> Result<Vec<u8>, FragmentApplicationError> {
    let mut overlays = BTreeMap::new();
    let mut repeated: Option<String> = None;
    for LoxccFragment { name, data } in fragments {
        match overlays.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(data);
            }
            Entry::Occupied(entry) if repeated.as_ref().is_none_or(|name| entry.key() < name) => {
                repeated = Some(entry.key().clone());
            }
            Entry::Occupied(_) => {}
        }
    }
    if let Some(name) = repeated {
        return Err(FragmentApplicationError::DuplicateFilename { name });
    }
    core::merge_overlays(&base, &overlays).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::{FragmentApplicationErrorFrame as F, *};
    use crate::compress_loxcc;

    const A: &str = "abcdef00-0000-0000-ffff000000000001";
    const B: &str = "fedcba00-0000-0000-ffff000000000002";

    fn fragment(id: &str, xml: String) -> LoxccFragment {
        LoxccFragment {
            name: format!("{id}.LoxCC"),
            data: compress_loxcc(xml.into_bytes()),
        }
    }

    fn frames(base: &[u8], fragments: Vec<LoxccFragment>) -> Vec<F> {
        match merge_overlays(base.to_vec(), fragments).unwrap_err() {
            FragmentApplicationError::Refused { frames } => frames,
            other => panic!("unexpected refusal: {other}"),
        }
    }

    #[test]
    fn bypasses_malformed_base_and_applies_in_filename_order() {
        let invalid = vec![255, b'<', 0];
        assert_eq!(merge_overlays(invalid.clone(), vec![]).unwrap(), invalid);
        let base =
            "\u{feff}<Root>\r\n<!--keep--><C Type=\"Page\" U=\"page\" SpNo=\"1\"/>\r\n</Root>";
        let a = format!(
            r#"<C Type="AutopilotRule" U="{A}" V="line
break"/>"#
        );
        let b = format!(r#"<C U="{B}" Type="AutopilotRule"/>"#);
        let expected = base.replace("/>", &format!(">{a}{b}</C>"));
        let entries = vec![fragment(B, b), fragment(A, a)];
        let map = entries
            .iter()
            .map(|part| (part.name.clone(), part.data.clone()))
            .collect();
        assert_eq!(
            core::merge_overlays(base.as_bytes(), &map).unwrap(),
            expected.as_bytes()
        );
        assert_eq!(
            merge_overlays(base.as_bytes().to_vec(), entries).unwrap(),
            expected.as_bytes()
        );
    }

    #[test]
    fn exact_repeated_entries_are_not_core_aliases() {
        let a = fragment(A, String::new());
        let b = fragment(B, String::new());
        let mut repeated = a.clone();
        repeated.data.clear();
        assert!(matches!(
            merge_overlays(b"<Root/>".to_vec(), vec![b.clone(), b, a.clone(), repeated]),
            Err(FragmentApplicationError::DuplicateFilename { name }) if name == a.name
        ));
        let alias = fragment("ABCDEF00-0000-0000-FFFF-000000000001", String::new());
        assert_eq!(
            frames(b"<Root/>", vec![a, alias]),
            [F::DuplicateOverlay {
                id: A.replace('-', "")
            }]
        );
    }

    #[test]
    fn refusal_keeps_each_archive_cause_and_never_returns_a_partial_merge() {
        let base = format!(r#"<Root><C U="{A}" Type="And" V="before"/></Root>"#);
        let first = fragment(A, format!(r#"<C U="{A}" Type="And" V="after"/>"#));
        let bare = compress_loxcc(Vec::new());
        let bad_offset = |offset| [&bare[..16], &[0x10, b'x', offset, 0]].concat();
        for (data, cause) in [
            (vec![0; 15], F::ArchiveHeaderTooShort { actual: 15 }),
            (vec![0; 16], F::ArchiveInvalidMagic { actual: 0 }),
            (bad_offset(0), F::ArchiveZeroBackReference { position: 2 }),
            (
                bad_offset(2),
                F::ArchiveBackReferenceOutOfBounds {
                    offset: 2,
                    output_len: 1,
                },
            ),
        ] {
            let name = format!("{B}.LoxCC");
            let second = LoxccFragment {
                name: name.clone(),
                data,
            };
            let entries = vec![second, first.clone()];
            assert_eq!(
                frames(base.as_bytes(), entries),
                [F::FragmentRead { name }, F::Archive, cause]
            );
        }
        let mut corrupt = first;
        corrupt.data[12] ^= 1;
        assert_eq!(
            frames(base.as_bytes(), vec![corrupt]),
            [
                F::FragmentRead {
                    name: format!("{A}.LoxCC")
                },
                F::FragmentIntegrity
            ]
        );
    }

    #[test]
    fn consistency_keeps_the_fragment_name_and_inner_duplicate_spelling() {
        let base = format!(r#"<Root><C U="{A}" Type="And"/><C U="Keep-Me" Type="Or"/></Root>"#);
        let a = fragment(
            A,
            format!(r#"<C U="{A}" Type="And"><C U="keep-me" Type="Or"/></C>"#),
        );
        let name = a.name.clone();
        assert_eq!(
            frames(base.as_bytes(), vec![a]),
            [
                F::UuidConsistency { name },
                F::Overlay {
                    frame: OverlayErrorFrame::DuplicateUuid {
                        id: "Keep-Me".into()
                    }
                }
            ]
        );
    }

    #[test]
    fn placement_and_fragment_boundary_diagnostics_keep_fields() {
        let name = format!("{A}.LoxCC");
        let a = |kind| fragment(A, format!(r#"<C U="{A}" Type="{kind}"/>"#));
        assert_eq!(
            frames(b"<Root/>", vec![a("AutopilotRule")]),
            [F::RulePlacement { name: name.clone() }]
        );
        assert_eq!(
            frames(b"<Root/>", vec![a("And")]),
            [F::UnknownControl {
                name: name.clone(),
                kind: "And".into()
            }]
        );
        assert_eq!(
            frames(
                b"<Root/>",
                vec![fragment(
                    A,
                    format!(r#"<Wrapper><C U="{A}" Type="And"/></Wrapper>"#)
                )]
            ),
            [F::InvalidControlRoot { name: name.clone() }]
        );
        assert_eq!(
            frames(
                b"<Root/>",
                vec![fragment(
                    A,
                    format!(r#"<?test data?><C U="{A}" Type="And"/>"#)
                )]
            ),
            [F::UnsupportedRoot { name: name.clone() }]
        );
        assert_eq!(
            frames(b"<Root/>", vec![fragment(A, "<Root/>".into())]),
            [F::NotControl]
        );
        assert_eq!(
            frames(
                format!(r#"<Root><C U="{A}" Type="And"/></Root>"#).as_bytes(),
                vec![a("Or")]
            ),
            [F::TypeChange { name }]
        );
    }

    #[test]
    fn document_and_overlap_errors_keep_core_precedence() {
        let a = fragment(A, format!(r#"<C U="{A}" Type="And"/>"#));
        let b = fragment(B, format!(r#"<C U="{B}" Type="Or"/>"#));
        let bad_name = LoxccFragment {
            name: "bad.LoxCC".into(),
            data: Vec::new(),
        };
        assert_eq!(
            frames(b"\xff<", vec![bad_name.clone()]),
            [F::Overlay {
                frame: OverlayErrorFrame::DocumentUtf8 {
                    valid_up_to: 0,
                    error_len: Some(1)
                }
            }]
        );
        assert_eq!(
            frames(b"<Root/>", vec![bad_name]),
            [F::InvalidFilename {
                name: "bad.LoxCC".into()
            }]
        );
        let nested = format!(r#"<C U="{A}" Type="And"><C U="{B}" Type="Or"/></C>"#);
        assert_eq!(
            frames(nested.as_bytes(), vec![b.clone(), a.clone()]),
            [F::ParentChildOverlap]
        );
        assert_eq!(
            frames(b"<Root/>", vec![b, fragment(A, nested)]),
            [F::FragmentOverlap { name: a.name }]
        );
    }
}
