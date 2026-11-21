use std::{error::Error, fmt};

use super::{DayTimerRemovalError, SelectedDayTimerEntry, State};
#[cfg(feature = "configuration-xml")]
use super::{daytimer_mode_uuid_key, validate_entry_identity};
use crate::configuration::editor::EditError;
#[cfg(feature = "configuration-xml")]
use crate::configuration::editor::transaction_scope::{ScopeError, require_ordinary_owner};
#[cfg(feature = "configuration-xml")]
use chrono::{DateTime, FixedOffset};
#[cfg(feature = "configuration-xml")]
use quick_xml::{Reader, events::Event};

/// Selected `Entry/@NeedActivate` change. UUID spellings are not normalized in the report.
#[derive(Debug, PartialEq, Eq)]
pub struct DayTimerEntryActivation {
    pub block_uuid: String,
    pub requested_mode_uuid: String,
    pub stored_mode_uuid: String,
    pub from_minutes: u32,
    pub to_minutes: u32,
    /// Original decoded attribute, without applying the absent-false default.
    pub previous_attribute: Option<String>,
    pub previous_required: bool,
    pub activation_required: bool,
    pub changed: bool,
}

#[cfg(feature = "configuration-xml")]
#[derive(Debug)]
pub struct DayTimerActivationOutcome {
    /// Exact input on a no-op, otherwise verified canonical XML.
    pub bytes: Vec<u8>,
    pub change: DayTimerEntryActivation,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DayTimerActivationError {
    Edit(EditError),
    Schedule(DayTimerRemovalError),
    InvalidStoredFlag { raw: String },
    NamespacedXml,
    UnsupportedXml,
    UnsupportedCanonicalContent,
    UnsupportedLineEndings,
    XmlDepthLimit { max_depth: usize },
    OutsidePage,
    InvalidOwnerUuid,
    DuplicateOwnerUuid { block_uuid: String },
    TemplateOrInherited,
    CanonicalReadmission(Box<Self>),
    CanonicalPreservation,
}

impl fmt::Display for DayTimerActivationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Schedule(error) => error.fmt(f),
            Self::InvalidStoredFlag { raw } => write!(
                f,
                "Unsupported DayTimer NeedActivate='{raw}': use true/false (case-insensitive) or absence; nothing was changed."
            ),
            Self::NamespacedXml => f.write_str("DayTimer activation does not support XML namespaces."),
            Self::UnsupportedXml => f.write_str(
                "DayTimer activation requires one UTF-8 XML 1.0 root and a supported declaration if present.",
            ),
            Self::UnsupportedCanonicalContent => f.write_str(
                "DayTimer activation requires element-only XML with plain names, valid single-line attributes and no digit-attribute rewriting.",
            ),
            Self::UnsupportedLineEndings => f.write_str(
                "Changing DayTimer activation requires uniform LF or CRLF line endings, or no line endings; CR-only and mixed endings are unsupported.",
            ),
            Self::XmlDepthLimit { max_depth } => write!(
                f,
                "DayTimer activation supports at most {max_depth} nested XML elements."
            ),
            Self::OutsidePage => f.write_str(
                "DayTimer activation requires a direct Page child in one stampable Document; nothing was changed.",
            ),
            Self::InvalidOwnerUuid => f.write_str(
                "DayTimer activation requires a non-nil native or RFC block UUID; nothing was changed.",
            ),
            Self::DuplicateOwnerUuid { block_uuid } => write!(
                f,
                "DayTimer activation owner '{block_uuid}' is not unique; nothing was changed."
            ),
            Self::TemplateOrInherited => f.write_str(
                "DayTimer activation does not support template or inherited owner state.",
            ),
            Self::CanonicalReadmission(error) => {
                write!(f, "Canonical DayTimer activation output was refused: {error}")
            }
            Self::CanonicalPreservation => f.write_str(
                "Canonical DayTimer activation output did not preserve the edited document or selection.",
            ),
        }
    }
}

impl Error for DayTimerActivationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Schedule(error) => Some(error),
            Self::CanonicalReadmission(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

#[cfg(feature = "configuration-xml")]
impl From<ScopeError> for DayTimerActivationError {
    fn from(error: ScopeError) -> Self {
        match error {
            ScopeError::Edit(error) => Self::Edit(error),
            ScopeError::NamespacedXml => Self::NamespacedXml,
            ScopeError::UnsupportedXml => Self::UnsupportedXml,
            ScopeError::DocumentScope => Self::OutsidePage,
            ScopeError::TemplateOrInherited => Self::TemplateOrInherited,
        }
    }
}

/// Set the configured activation flag of one existing ordinary digital DayTimer entry.
///
/// Refusals are ordered: requested mode UUID, requested interval, XML admission,
/// legacy selection and schedule validation, ordinary Page ownership, then selected
/// `NeedActivate` storage. The Page is directly under the sole stampable Document
/// or one direct Program within it. The block has a unique non-nil native or RFC
/// UUID (including aliases); `uuid:` selection still compares its spelling exactly.
///
/// Absence means false; explicit true and false are accepted case-insensitively.
/// A change to true writes `true`; a change to false omits the attribute.
/// A semantic no-op returns the original bytes without stamping or serializing.
///
/// The byte API admits namespace-free, DTD-free UTF-8 XML 1.0 without a declaration
/// or with `<?xml version="1.0"?>`, optionally including ` encoding="utf-8"` or
/// ` encoding="UTF-8"` before `?>`. It admits ASCII element/attribute names,
/// single-line attribute values (including valid XML 1.0 references that do not
/// decode to tabs or line breaks), and layout whitespace between elements or
/// outside the root. Comments, processing instructions, CDATA, leaf text,
/// duplicate attributes and inputs affected by the legacy digit-name sanitizer
/// are refused. Depth is capped at 64 elements, including self-closing leaves.
/// This narrower byte boundary does not change legacy CLI admission.
///
/// A change uses `now` for the shared save stamp. It returns canonical XML only
/// after re-admitting the whole output and verifying the entire edited tree, BOM,
/// line-ending convention and selected record. Uniform LF and CRLF are preserved;
/// input without line endings produces canonical LF. CR-only and mixed conventions
/// are refused on a change; otherwise admitted semantic no-ops still return exact
/// input bytes. No file, clock or runtime is accessed.
#[cfg(feature = "configuration-xml")]
pub fn set_daytimer_entry_activation(
    bytes: &[u8],
    selector: &str,
    mode_uuid: &str,
    from_minutes: u32,
    to_minutes: u32,
    activation_required: bool,
    now: &DateTime<FixedOffset>,
) -> Result<DayTimerActivationOutcome, DayTimerActivationError> {
    use DayTimerActivationError as Refusal;

    let mode_key =
        validate_entry_identity(mode_uuid, from_minutes, to_minutes).map_err(Refusal::Schedule)?;
    let read_plan = |xml: &[u8]| -> Result<(State, ActivationPlan), Refusal> {
        require_activation_xml(xml)?;
        let state = State::load(xml).map_err(Refusal::Edit)?;
        if !state.digit_attr_renames.is_empty() {
            return Err(Refusal::UnsupportedCanonicalContent);
        }
        let selected = state
            .select_daytimer_entry_by_mode_key(selector, &mode_key, from_minutes, to_minutes)
            .map_err(Refusal::Schedule)?;
        require_activation_owner(&state, &selected)?;
        let plan = state.plan_daytimer_activation(
            selected,
            mode_uuid,
            from_minutes,
            to_minutes,
            activation_required,
        )?;
        Ok((state, plan))
    };

    let (mut state, plan) = read_plan(bytes)?;
    if !plan.change.changed {
        return Ok(DayTimerActivationOutcome {
            bytes: bytes.to_vec(),
            change: plan.change,
            document_stamped: false,
        });
    }
    let expected_endings = line_endings(bytes)?.unwrap_or(LineEndings::Lf);
    state.apply_daytimer_activation(&plan);
    if !state.stamp_document(now) {
        return Err(Refusal::OutsidePage);
    }
    let bytes = state.to_bytes().map_err(Refusal::Edit)?;
    let output_endings =
        line_endings(&bytes).map_err(|error| Refusal::CanonicalReadmission(Box::new(error)))?;
    let (reread, reselected) =
        read_plan(&bytes).map_err(|error| Refusal::CanonicalReadmission(Box::new(error)))?;
    if reread.root != state.root
        || reread.had_bom != state.had_bom
        || output_endings != Some(expected_endings)
        || reselected.path != plan.path
        || reselected.change.block_uuid != plan.change.block_uuid
        || reselected.change.stored_mode_uuid != plan.change.stored_mode_uuid
        || reselected.change.changed
        || reselected.change.previous_attribute.as_deref() != activation_required.then_some("true")
    {
        return Err(Refusal::CanonicalPreservation);
    }
    Ok(DayTimerActivationOutcome {
        bytes,
        change: plan.change,
        document_stamped: true,
    })
}

#[cfg(feature = "configuration-xml")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum LineEndings {
    Lf,
    Crlf,
}

#[cfg(feature = "configuration-xml")]
fn line_endings(bytes: &[u8]) -> Result<Option<LineEndings>, DayTimerActivationError> {
    let mut style = None;
    let mut bytes = bytes.iter().copied();
    while let Some(byte) = bytes.next() {
        let current = match byte {
            b'\n' => LineEndings::Lf,
            b'\r' if bytes.next() == Some(b'\n') => LineEndings::Crlf,
            b'\r' => return Err(DayTimerActivationError::UnsupportedLineEndings),
            _ => continue,
        };
        if style.is_some_and(|previous| previous != current) {
            return Err(DayTimerActivationError::UnsupportedLineEndings);
        }
        style = Some(current);
    }
    Ok(style)
}

#[cfg(feature = "configuration-xml")]
fn require_activation_owner(
    state: &State,
    selected: &SelectedDayTimerEntry,
) -> Result<(), DayTimerActivationError> {
    use DayTimerActivationError as Refusal;

    if !state.daytimer_is_page_child(&selected.path) {
        return Err(Refusal::OutsidePage);
    }
    let mut entry_path = selected.path.clone();
    entry_path.push(selected.position);
    require_ordinary_owner(state, &entry_path)?;
    let block_uuid = state
        .get_element(&selected.path)
        .attributes
        .get("U")
        .ok_or(Refusal::InvalidOwnerUuid)?;
    let key = daytimer_mode_uuid_key(block_uuid).ok_or(Refusal::InvalidOwnerUuid)?;
    // The legacy selector deliberately de-duplicates mirror C elements by U.
    // Do not admit either a mirror or an alias of the selected owner to the byte API.
    let count = State::iter_elements(&state.root)
        .into_iter()
        .filter(|element| {
            element.name == "C"
                && element
                    .attributes
                    .get("U")
                    .and_then(|raw| daytimer_mode_uuid_key(raw))
                    .as_ref()
                    == Some(&key)
        })
        .take(2)
        .count();
    if count != 1 {
        return Err(Refusal::DuplicateOwnerUuid {
            block_uuid: block_uuid.clone(),
        });
    }
    Ok(())
}

#[cfg(feature = "configuration-xml")]
fn require_activation_xml(bytes: &[u8]) -> Result<(), DayTimerActivationError> {
    use DayTimerActivationError as Refusal;
    const MAX_DEPTH: usize = 64;

    let text = std::str::from_utf8(bytes).map_err(|_| Refusal::UnsupportedXml)?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.starts_with('\u{feff}') || !text.chars().all(xml_1_0_char) {
        return Err(Refusal::UnsupportedXml);
    }
    let mut reader = Reader::from_str(text);
    reader.config_mut().expand_empty_elements = true;
    let mut first = true;
    let mut root_seen = false;
    // Track whether formatting whitespace is in a container or is leaf data.
    let mut containers = Vec::<(bool, bool)>::new();
    loop {
        match reader
            .read_event()
            .map_err(|error| Refusal::Edit(EditError::load(error)))?
        {
            Event::Decl(decl) => {
                if !first
                    || !matches!(
                        &*decl,
                        b"xml version=\"1.0\""
                            | b"xml version=\"1.0\" encoding=\"utf-8\""
                            | b"xml version=\"1.0\" encoding=\"UTF-8\""
                    )
                {
                    return Err(Refusal::UnsupportedXml);
                }
            }
            Event::Start(start) => {
                if containers.is_empty() && std::mem::replace(&mut root_seen, true) {
                    return Err(Refusal::UnsupportedXml);
                }
                if containers.len() == MAX_DEPTH {
                    return Err(Refusal::XmlDepthLimit {
                        max_depth: MAX_DEPTH,
                    });
                }
                let name = start.name();
                if name.as_ref().contains(&b':') {
                    return Err(Refusal::NamespacedXml);
                }
                if !plain_name(name.as_ref()) {
                    return Err(Refusal::UnsupportedCanonicalContent);
                }
                for attr in start.attributes() {
                    let attr = attr.map_err(|_| Refusal::UnsupportedCanonicalContent)?;
                    let key = attr.key.as_ref();
                    if key == b"xmlns" || key.contains(&b':') {
                        return Err(Refusal::NamespacedXml);
                    }
                    // Attribute iteration alone does not require separating XML whitespace.
                    let separated = (key.as_ptr() as usize)
                        .checked_sub(start.as_ptr() as usize)
                        .and_then(|offset| offset.checked_sub(1))
                        .and_then(|offset| start.get(offset))
                        .is_some_and(|&byte| xml_whitespace(byte));
                    if !separated
                        || !plain_name(key)
                        || !std::str::from_utf8(&attr.value).is_ok_and(plain_value)
                    {
                        return Err(Refusal::UnsupportedCanonicalContent);
                    }
                }
                if let Some((has_child, _)) = containers.last_mut() {
                    *has_child = true;
                }
                containers.push((false, false));
            }
            Event::Text(text) if text.iter().copied().all(xml_whitespace) => {
                if let Some((_, has_whitespace)) = containers.last_mut() {
                    *has_whitespace = true;
                }
            }
            Event::End(_) => match containers.pop() {
                Some((false, true)) => return Err(Refusal::UnsupportedCanonicalContent),
                Some(_) => {}
                None => return Err(Refusal::UnsupportedXml),
            },
            Event::Eof if root_seen && containers.is_empty() => return Ok(()),
            Event::Eof | Event::DocType(_) => return Err(Refusal::UnsupportedXml),
            Event::Text(_) if containers.is_empty() => return Err(Refusal::UnsupportedXml),
            _ => return Err(Refusal::UnsupportedCanonicalContent),
        }
        first = false;
    }
}

#[cfg(feature = "configuration-xml")]
fn xml_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

#[cfg(feature = "configuration-xml")]
fn xml_1_0_char(ch: char) -> bool {
    matches!(
        ch,
        '\t' | '\r' | '\n' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
    )
}

#[cfg(feature = "configuration-xml")]
fn plain_name(name: &[u8]) -> bool {
    name.split_first().is_some_and(|(&first, tail)| {
        (first.is_ascii_alphabetic() || first == b'_')
            && tail
                .iter()
                .all(|&b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
    })
}

#[cfg(feature = "configuration-xml")]
fn plain_value(raw: &str) -> bool {
    !raw.bytes()
        .any(|b| matches!(b, b'<' | b'\t' | b'\r' | b'\n'))
        && raw.split('&').skip(1).all(|tail| {
            let Some((entity, _)) = tail.split_once(';') else {
                return false;
            };
            if matches!(entity, "amp" | "apos" | "gt" | "lt" | "quot") {
                return true;
            }
            let (digits, radix) = match entity.strip_prefix("#x") {
                Some(hex) if hex.bytes().all(|b| b.is_ascii_hexdigit()) => (hex, 16),
                None => match entity.strip_prefix('#') {
                    Some(decimal) if decimal.bytes().all(|b| b.is_ascii_digit()) => (decimal, 10),
                    _ => return false,
                },
                _ => return false,
            };
            u32::from_str_radix(digits, radix)
                .ok()
                .and_then(char::from_u32)
                .is_some_and(|ch| xml_1_0_char(ch) && !matches!(ch, '\t' | '\r' | '\n'))
        })
}

struct ActivationPlan {
    path: Vec<usize>,
    change: DayTimerEntryActivation,
}

impl State {
    fn plan_daytimer_activation(
        &self,
        mut selected: SelectedDayTimerEntry,
        mode_uuid: &str,
        from_minutes: u32,
        to_minutes: u32,
        activation_required: bool,
    ) -> Result<ActivationPlan, DayTimerActivationError> {
        let block = self.get_element(&selected.path);
        let block_uuid = block.attributes.get("U").cloned().unwrap_or_default();
        selected.path.push(selected.position);
        let previous_attribute = self
            .get_element(&selected.path)
            .attributes
            .get("NeedActivate")
            .cloned();
        let previous_required = match previous_attribute.as_deref() {
            None => false,
            Some(raw) if raw.eq_ignore_ascii_case("true") => true,
            Some(raw) if raw.eq_ignore_ascii_case("false") => false,
            Some(raw) => {
                return Err(DayTimerActivationError::InvalidStoredFlag { raw: raw.into() });
            }
        };
        Ok(ActivationPlan {
            path: selected.path,
            change: DayTimerEntryActivation {
                block_uuid,
                requested_mode_uuid: mode_uuid.into(),
                stored_mode_uuid: selected.stored_mode_uuid,
                from_minutes,
                to_minutes,
                previous_attribute,
                previous_required,
                activation_required,
                changed: previous_required != activation_required,
            },
        })
    }

    fn apply_daytimer_activation(&mut self, plan: &ActivationPlan) {
        if !plan.change.changed {
            return;
        }
        let attributes = &mut self.get_element_mut(&plan.path).attributes;
        if plan.change.activation_required {
            attributes.insert("NeedActivate".into(), "true".into());
        } else {
            attributes.shift_remove("NeedActivate");
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_daytimer_entry_activation_legacy(
        &mut self,
        selector: &str,
        mode_uuid: &str,
        from_minutes: u32,
        to_minutes: u32,
        activation_required: bool,
    ) -> Result<DayTimerEntryActivation, DayTimerActivationError> {
        let selected = self
            .select_daytimer_entry(selector, mode_uuid, from_minutes, to_minutes)
            .map_err(DayTimerActivationError::Schedule)?;
        let plan = self.plan_daytimer_activation(
            selected,
            mode_uuid,
            from_minutes,
            to_minutes,
            activation_required,
        )?;
        self.apply_daytimer_activation(&plan);
        Ok(plan.change)
    }
}
