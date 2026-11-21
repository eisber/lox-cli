use chrono::{DateTime, FixedOffset};
use quick_xml::{Reader, events::Event};
use std::{error::Error, fmt};

use super::{
    EditError, LoadManagerInspection, LoadManagerInspectionError, State, inspect_positions,
    load_positions, resolve_io_index,
};
use crate::configuration::editor::transaction_scope::{
    ScopeError, require_ordinary_owner, require_plain_xml,
};

// Bound recursive xmltree construction, traversal, comparison, and teardown.
const MAX_XML_ELEMENT_DEPTH: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerMinOnChange {
    pub io_index: u8,
    /// Loader-decoded `minOn` spelling, or `None` for the omitted zero default.
    pub before: Option<String>,
    /// An actual change to zero omits `minOn`; a no-op retains its spelling.
    pub after: Option<String>,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LoadManagerMinOnOutcome {
    /// Original bytes on a no-op; otherwise the complete, verified canonical XML.
    pub bytes: Vec<u8>,
    pub change: LoadManagerMinOnChange,
    pub inspection: LoadManagerInspection,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadManagerMinOnError {
    Edit(EditError),
    Inspection(LoadManagerInspectionError),
    InvalidRequestedIoIndex { io_index: u8 },
    InvalidSeconds,
    InvalidStorage { io_index: u8, raw: String },
    NamespacedXml,
    UnsupportedXml,
    UnsupportedCanonicalContent,
    XmlDepthLimit { max_depth: usize },
    DocumentScope,
    TemplateOrInherited,
    CanonicalReadmission(Box<Self>),
    CanonicalPreservation,
}

impl fmt::Display for LoadManagerMinOnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Inspection(error) => error.fmt(f),
            Self::InvalidRequestedIoIndex { .. } => {
                f.write_str("Load Manager io-index must be 0..11, not a 1-based priority.")
            }
            Self::InvalidSeconds => f.write_str(
                "Load Manager minimum on time must be 0 or whole seconds in 10..2147483647, using decimal digits only. Native values 1..9 clamp to 10; this CLI rejects them instead. No rounding, clamping or unit conversion; nothing was changed.",
            ),
            Self::InvalidStorage { io_index, .. } => write!(
                f,
                "Unsupported Load Manager minOn storage for io-index {io_index}: expected a nonnegative signed-32-bit integer in seconds (an optional leading plus is supported). Empty, negative, fractional, exponent, whitespace and malformed storage are rejected; nothing was changed."
            ),
            Self::NamespacedXml => {
                f.write_str("Load Manager minimum-on editing does not support XML namespaces.")
            }
            Self::UnsupportedXml => f.write_str(
                "Load Manager minimum-on editing requires one UTF-8 XML 1.0 root and a supported declaration if present.",
            ),
            Self::UnsupportedCanonicalContent => f.write_str(
                "Load Manager minimum-on editing requires element-only XML without comments, processing instructions, CDATA, leaf text, duplicate attributes, literal attribute markup/tabs/line breaks, or digit-attribute rewriting.",
            ),
            Self::XmlDepthLimit { max_depth } => write!(
                f,
                "Load Manager minimum-on editing supports at most {max_depth} nested XML elements, counting the root and self-closing elements."
            ),
            Self::DocumentScope => f.write_str(
                "Load Manager minimum-on editing requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => f.write_str(
                "Load Manager minimum-on editing does not support template or inherited owner state.",
            ),
            Self::CanonicalReadmission(error) => {
                write!(f, "Canonical minimum-on output was refused: {error}")
            }
            Self::CanonicalPreservation => f.write_str(
                "Canonical minimum-on output did not preserve the edited document or selection.",
            ),
        }
    }
}

impl Error for LoadManagerMinOnError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Inspection(error) => Some(error),
            Self::CanonicalReadmission(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<ScopeError> for LoadManagerMinOnError {
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

/// Edit the stored `minOn` of one direct `Load` by its resolved connector `ioIdx`.
///
/// The request is ASCII decimal digits for zero or 10..=2147483647 whole seconds.
/// Existing storage may be any nonnegative `i32` in digits with an optional
/// leading plus. Omission means zero. A numeric no-op keeps the original bytes,
/// including explicit zero or leading zeros. A change to zero removes `minOn`.
///
/// Refusals are ordered: request index, requested seconds, XML admission, owner
/// selection and scope, twelve-record shape/identities, then selected storage.
/// Admission is limited to one namespace-free, DTD-free UTF-8 root, with no
/// declaration or `<?xml version="1.0"?>`, optionally including
/// ` encoding="utf-8"` or ` encoding="UTF-8"` before `?>`. Only elements and
/// attributes are admitted, with XML whitespace as inter-element formatting.
/// Comments, processing instructions, CDATA, leaf text, duplicate attributes,
/// literal `<`/tabs/line breaks in attributes, and digit-attribute rewrites are refused.
/// The byte API caps nesting at 64 elements, counting the root as depth 1 and
/// including self-closing leaves. This conservative limit bounds recursive DOM
/// work; it is checked before loading both input and changed output, even for
/// an input that would otherwise be a numeric no-op.
/// This narrower byte boundary does not change legacy CLI admission.
///
/// On a change, `now` supplies the monotonic stamp of the owning Document, if
/// present. Changed output must pass the same admission and preserve the entire
/// edited DOM and original selection on canonical reread. No revised bytes are
/// returned on error. This API has no filesystem or runtime effects.
pub fn set_load_manager_load_min_on(
    bytes: &[u8],
    selector: &str,
    io_index: u8,
    min_on_seconds: &str,
    now: &DateTime<FixedOffset>,
) -> Result<LoadManagerMinOnOutcome, LoadManagerMinOnError> {
    use LoadManagerMinOnError as Refusal;

    let seconds = validate_request(io_index, min_on_seconds)?;
    let (mut state, path) = read_strict_owner(bytes, selector)?;
    let plan = state.plan_load_manager_min_on(&path, io_index, seconds)?;
    if !plan.change.changed {
        return Ok(LoadManagerMinOnOutcome {
            bytes: bytes.to_vec(),
            change: plan.change,
            inspection: inspect_positions(state.get_element(&path), &plan.positions),
            document_stamped: false,
        });
    }

    state.apply_load_manager_min_on(&path, &plan);
    let document_stamped = state.stamp_document(now);
    let bytes = state.to_bytes().map_err(Refusal::Edit)?;
    let (reread, reread_path) = read_strict_owner(&bytes, selector)
        .map_err(|error| Refusal::CanonicalReadmission(Box::new(error)))?;
    if reread.root != state.root
        || reread.had_bom != state.had_bom
        || reread.had_crlf != state.had_crlf
        || reread_path != path
    {
        return Err(Refusal::CanonicalPreservation);
    }
    Ok(LoadManagerMinOnOutcome {
        bytes,
        change: plan.change,
        inspection: inspect_positions(reread.get_element(&path), &plan.positions),
        document_stamped,
    })
}

fn read_strict_owner(
    bytes: &[u8],
    selector: &str,
) -> Result<(State, Vec<usize>), LoadManagerMinOnError> {
    use LoadManagerMinOnError as Refusal;

    require_plain_xml(bytes)?;
    require_element_only_xml(bytes)?;
    let state = State::load(bytes).map_err(Refusal::Edit)?;
    if !state.digit_attr_renames.is_empty() {
        return Err(Refusal::UnsupportedCanonicalContent);
    }
    let path = state
        .load_manager_path(selector)
        .map_err(Refusal::Inspection)?;
    require_ordinary_owner(&state, &path)?;
    Ok((state, path))
}

fn require_element_only_xml(bytes: &[u8]) -> Result<(), LoadManagerMinOnError> {
    use LoadManagerMinOnError as Refusal;

    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().expand_empty_elements = true;
    let mut first = true;
    // Whitespace in a leaf is data. Only containers with element children and
    // the exterior may have layout whitespace; opaque nodes are never admitted.
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
            Event::PI(pi) if pi.target().eq_ignore_ascii_case(b"xml") => {
                return Err(Refusal::UnsupportedXml);
            }
            Event::Start(start) => {
                if containers.len() == MAX_XML_ELEMENT_DEPTH {
                    return Err(Refusal::XmlDepthLimit {
                        max_depth: MAX_XML_ELEMENT_DEPTH,
                    });
                }
                for attr in start.attributes() {
                    let attr = attr.map_err(|_| Refusal::UnsupportedCanonicalContent)?;
                    if attr
                        .value
                        .iter()
                        .any(|byte| matches!(byte, b'<' | b'\t' | b'\r' | b'\n'))
                    {
                        return Err(Refusal::UnsupportedCanonicalContent);
                    }
                }
                if let Some((has_child, _)) = containers.last_mut() {
                    *has_child = true;
                }
                containers.push((false, false));
            }
            Event::Text(text)
                if text
                    .iter()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n')) =>
            {
                if let Some((_, has_whitespace)) = containers.last_mut() {
                    *has_whitespace = true;
                }
            }
            Event::End(_) => {
                if containers.pop() == Some((false, true)) {
                    return Err(Refusal::UnsupportedCanonicalContent);
                }
            }
            Event::Eof => return Ok(()),
            _ => return Err(Refusal::UnsupportedCanonicalContent),
        }
        first = false;
    }
}

fn decimal_seconds(raw: &str) -> Option<i32> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    raw.parse().ok()
}

fn validate_request(io_index: u8, raw: &str) -> Result<i32, LoadManagerMinOnError> {
    if io_index > 11 {
        return Err(LoadManagerMinOnError::InvalidRequestedIoIndex { io_index });
    }
    decimal_seconds(raw)
        .filter(|seconds| *seconds == 0 || *seconds >= 10)
        .ok_or(LoadManagerMinOnError::InvalidSeconds)
}

struct MinOnPlan {
    position: usize,
    positions: Vec<usize>,
    change: LoadManagerMinOnChange,
}

impl State {
    fn plan_load_manager_min_on(
        &self,
        path: &[usize],
        io_index: u8,
        seconds: i32,
    ) -> Result<MinOnPlan, LoadManagerMinOnError> {
        use LoadManagerMinOnError as Refusal;

        let block = self.get_element(path);
        let positions = load_positions(block).map_err(Refusal::Inspection)?;
        // Twelve distinct resolved indices in 0..=11 always include the request.
        let position = positions
            .iter()
            .copied()
            .find(|&position| {
                resolve_io_index(block.children[position].as_element().unwrap()).unwrap()
                    == io_index
            })
            .unwrap();
        let before = block.children[position]
            .as_element()
            .unwrap()
            .attributes
            .get("minOn")
            .cloned();
        let previous = match &before {
            None => 0,
            Some(raw) => {
                decimal_seconds(raw.strip_prefix('+').unwrap_or(raw)).ok_or_else(|| {
                    Refusal::InvalidStorage {
                        io_index,
                        raw: raw.clone(),
                    }
                })?
            }
        };
        let changed = previous != seconds;
        let after = if changed {
            (seconds != 0).then(|| seconds.to_string())
        } else {
            before.clone()
        };
        Ok(MinOnPlan {
            position,
            positions,
            change: LoadManagerMinOnChange {
                io_index,
                before,
                after,
                changed,
            },
        })
    }

    fn apply_load_manager_min_on(&mut self, path: &[usize], plan: &MinOnPlan) {
        if !plan.change.changed {
            return;
        }
        let attributes = &mut self.get_element_mut(path).children[plan.position]
            .as_mut_element()
            .unwrap()
            .attributes;
        if let Some(seconds) = &plan.change.after {
            attributes.insert("minOn".into(), seconds.clone());
        } else {
            attributes.shift_remove("minOn");
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_load_manager_load_min_on(
        &mut self,
        selector: &str,
        io_index: u8,
        min_on_seconds: &str,
    ) -> Result<bool, LoadManagerMinOnError> {
        let seconds = validate_request(io_index, min_on_seconds)?;
        let path = self
            .load_manager_path(selector)
            .map_err(LoadManagerMinOnError::Inspection)?;
        let plan = self.plan_load_manager_min_on(&path, io_index, seconds)?;
        self.apply_load_manager_min_on(&path, &plan);
        Ok(plan.change.changed)
    }
}
