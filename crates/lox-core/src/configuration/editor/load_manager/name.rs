use chrono::{DateTime, FixedOffset};
use quick_xml::{
    Reader,
    events::{Event, attributes::Attributes},
};
use std::{error::Error, fmt};

use super::{
    EditError, LoadManagerInspection, LoadManagerInspectionError, State, inspect_positions,
    load_positions, resolve_io_index,
};
use crate::configuration::editor::transaction_scope::{
    ScopeError, require_ordinary_owner, require_plain_xml,
};

#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerNameChange {
    pub io_index: u8,
    /// Loader-decoded `nm` before the edit; absent and explicitly empty are distinct.
    pub before: Option<String>,
    /// Stored `nm` after the edit, with an actual clear represented by `None`.
    pub after: Option<String>,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LoadManagerNameOutcome {
    /// Original bytes on a no-op, otherwise the complete canonical XML.
    pub bytes: Vec<u8>,
    pub change: LoadManagerNameChange,
    pub inspection: LoadManagerInspection,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadManagerNameError {
    Edit(EditError),
    Inspection(LoadManagerInspectionError),
    InvalidRequestedIoIndex { io_index: u8 },
    InvalidName,
    NameTextCollision,
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
}

impl fmt::Display for LoadManagerNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Inspection(error) => error.fmt(f),
            Self::InvalidRequestedIoIndex { .. } => {
                f.write_str("Load Manager io-index must be 0..11, not a 1-based priority.")
            }
            Self::InvalidName => f.write_str(
                "Unsupported Load Manager name: use single-line Unicode without control characters, at most 32767 UTF-16 code units. This is a conservative CLI subset, not a recovered native maximum; nothing was changed.",
            ),
            Self::NameTextCollision => f.write_str(
                "The Load Manager name conflicts with the shared XML editor's digit-attribute rewriting; nothing was changed.",
            ),
            Self::NamespacedXml => {
                f.write_str("Load Manager name editing does not support XML namespaces.")
            }
            Self::UnsupportedXml => f.write_str(
                "Load Manager name editing requires one UTF-8 XML 1.0 root and no document type declaration.",
            ),
            Self::DocumentScope => f.write_str(
                "Load Manager name editing requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => {
                f.write_str("Load Manager name editing does not support template or inherited owner state.")
            }
        }
    }
}

impl Error for LoadManagerNameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Inspection(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ScopeError> for LoadManagerNameError {
    fn from(error: ScopeError) -> Self {
        match error {
            ScopeError::Edit(error) => Self::Edit(error),
            ScopeError::NamespacedXml => Self::NamespacedXml,
            ScopeError::UnsupportedXml => Self::UnsupportedXml,
            ScopeError::DocumentScope => Self::DocumentScope,
            ScopeError::TemplateOrInherited => Self::TemplateOrInherited,
        }
    }
}

/// Edit the stored `nm` of one direct `Load` by its resolved connector `ioIdx`.
///
/// The request index and single-line Unicode value are checked before loading
/// and selecting the ordinary `LoadShed` owner. The owner's twelve direct loads
/// must have unique resolved indices in 0..=11. An omitted index is zero;
/// neither order nor any other record data is edited. Names are exact, without
/// trimming or uniqueness requirements. Control characters, Unicode line and
/// paragraph separators, U+FFFE and U+FFFF are refused. The request is limited
/// to 32767 UTF-16 code units.
/// An empty request clears a nonempty `nm` by removing it. An identical value,
/// including an empty request against absent or explicit-empty `nm`, returns
/// the original bytes with no stamp; the inspection retains the distinction.
///
/// Namespaces, declarations other than UTF-8 XML 1.0, DTDs, multiple roots,
/// unsupported Document scope, and template or inherited owners are refused
/// even on a no-op. On a change, `now` supplies the monotonic stamp of an owning
/// Document, if present.
/// Output uses the shared canonical writer, not a raw XML patch. A changed
/// name affected by the loader/writer's digit-attribute substitutions is
/// refused; the shared loader limitation (lox-n37w) is otherwise unchanged.
/// This API has no filesystem or runtime effects and returns no bytes on error.
pub fn set_load_manager_load_name(
    bytes: &[u8],
    selector: &str,
    io_index: u8,
    name: &str,
    now: &DateTime<FixedOffset>,
) -> Result<LoadManagerNameOutcome, LoadManagerNameError> {
    use LoadManagerNameError as Refusal;

    validate_name_request(io_index, name)?;
    require_xml_1_0_declaration(bytes)?;
    let mut state = State::load(bytes).map_err(Refusal::Edit)?;
    require_plain_xml(bytes)?;
    let path = state
        .load_manager_path(selector)
        .map_err(Refusal::Inspection)?;
    require_ordinary_owner(&state, &path)?;
    let plan = state.plan_load_manager_name(&path, io_index, name)?;
    let (bytes, document_stamped) = if plan.change.changed {
        if !State::sanitize_digit_attrs(name.as_bytes()).1.is_empty()
            || state
                .digit_attr_renames
                .iter()
                .any(|(sanitized, _)| name.contains(&format!(" {sanitized}=")))
        {
            return Err(Refusal::NameTextCollision);
        }
        state.apply_load_manager_name(&path, &plan);
        let document_stamped = state.stamp_document(now);
        (state.to_bytes().map_err(Refusal::Edit)?, document_stamped)
    } else {
        (bytes.to_vec(), false)
    };
    // Selection depends on C elements and room metadata, not on Load/@nm.
    let inspection = inspect_positions(state.get_element(&path), &plan.positions);
    Ok(LoadManagerNameOutcome {
        bytes,
        change: plan.change,
        inspection,
        document_stamped,
    })
}

fn require_xml_1_0_declaration(bytes: &[u8]) -> Result<(), LoadManagerNameError> {
    // The writer always emits 1.0. Check declarations before the DOM can admit
    // 1.1-only data or report another version as a generic load failure.
    let mut reader = Reader::from_reader(bytes);
    let mut first = true;
    while let Ok(event) = reader.read_event() {
        match event {
            Event::Decl(decl) => {
                if !first
                    || !matches!(decl.version(), Ok(version) if version.as_ref() == b"1.0")
                    || !std::str::from_utf8(&decl)
                        .is_ok_and(|text| Attributes::new(text, 3).all(|attr| attr.is_ok()))
                {
                    return Err(LoadManagerNameError::UnsupportedXml);
                }
            }
            Event::PI(pi) if pi.target().eq_ignore_ascii_case(b"xml") => {
                return Err(LoadManagerNameError::UnsupportedXml);
            }
            Event::Eof => break,
            _ => {}
        }
        first = false;
    }
    // Structural parse failures remain the shared loader's diagnostics.
    Ok(())
}

fn validate_name_request(io_index: u8, name: &str) -> Result<(), LoadManagerNameError> {
    if io_index > 11 {
        return Err(LoadManagerNameError::InvalidRequestedIoIndex { io_index });
    }
    // Tab/newline aren't single-cell text; the length bound isn't a native maximum.
    if name.encode_utf16().count() > 32767
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')
        })
    {
        return Err(LoadManagerNameError::InvalidName);
    }
    Ok(())
}

struct NamePlan {
    position: usize,
    positions: Vec<usize>,
    change: LoadManagerNameChange,
}

impl State {
    fn plan_load_manager_name(
        &self,
        path: &[usize],
        io_index: u8,
        name: &str,
    ) -> Result<NamePlan, LoadManagerNameError> {
        let block = self.get_element(path);
        let positions = load_positions(block).map_err(LoadManagerNameError::Inspection)?;
        // Twelve distinct resolved indices in 0..=11 always include the request.
        let position = positions
            .iter()
            .copied()
            .find(|&position| {
                resolve_io_index(block.children[position].as_element().unwrap()).unwrap()
                    == io_index
            })
            .unwrap();
        let attributes = &block.children[position].as_element().unwrap().attributes;
        let before = attributes.get("nm").cloned();
        let changed = before.as_deref().unwrap_or("") != name;
        let after = if changed {
            (!name.is_empty()).then(|| name.to_owned())
        } else {
            before.clone()
        };
        Ok(NamePlan {
            position,
            positions,
            change: LoadManagerNameChange {
                io_index,
                before,
                after,
                changed,
            },
        })
    }

    fn apply_load_manager_name(&mut self, path: &[usize], plan: &NamePlan) {
        if !plan.change.changed {
            return;
        }
        let attributes = &mut self.get_element_mut(path).children[plan.position]
            .as_mut_element()
            .unwrap()
            .attributes;
        if let Some(name) = &plan.change.after {
            attributes.insert("nm".into(), name.clone());
        } else {
            attributes.shift_remove("nm");
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_load_manager_load_name(
        &mut self,
        selector: &str,
        io_index: u8,
        name: &str,
    ) -> Result<bool, LoadManagerNameError> {
        validate_name_request(io_index, name)?;
        let path = self
            .load_manager_path(selector)
            .map_err(LoadManagerNameError::Inspection)?;
        let plan = self.plan_load_manager_name(&path, io_index, name)?;
        self.apply_load_manager_name(&path, &plan);
        Ok(plan.change.changed)
    }
}
