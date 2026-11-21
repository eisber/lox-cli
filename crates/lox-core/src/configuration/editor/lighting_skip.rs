use chrono::{DateTime, FixedOffset};
use quick_xml::{Reader, events::Event};
use std::{error::Error, fmt};

use super::{
    EditError,
    legacy::{State, UTF8_BOM},
    transaction_scope::{ScopeError, require_ordinary_owner, require_plain_xml},
};

const MAX_XML_DEPTH: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub struct LightingMoodSkipChange {
    pub mood_uuid: Option<String>,
    pub sid: String,
    pub cid: Option<String>,
    /// Loader-decoded spelling; omission means false.
    pub before: Option<String>,
    pub before_effective: bool,
    /// Lowercase true/false on a change; otherwise the original spelling or absence.
    pub after: Option<String>,
    pub skip: bool,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LightingMoodSkipOutcome {
    /// Exact input on a no-op; verified canonical XML on a change.
    pub bytes: Vec<u8>,
    pub change: LightingMoodSkipChange,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LightingMoodSkipError {
    Edit(EditError),
    UnsupportedSelector,
    ControllerCount { found: usize },
    WrongBlockType,
    ContainerCount { found: usize },
    MoodMatchCount { found: usize },
    InvalidSelectedSid,
    NonpositiveSelectedSid,
    InvalidSiblingSid { index: usize },
    AmbiguousSid,
    UnsupportedSkipValue { raw: String },
    NamespacedXml,
    UnsupportedXml,
    UnsupportedCanonicalContent,
    XmlDepthLimit { max_depth: usize },
    DocumentScope,
    TemplateOrInherited,
    CanonicalReadmission(Box<Self>),
    CanonicalPreservation,
}

impl fmt::Display for LightingMoodSkipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::UnsupportedSelector => f.write_str("Use a nonempty exact uuid: selector."),
            Self::ControllerCount { found } => write!(
                f,
                "Expected one controller with the exact stored U, found {found}."
            ),
            Self::WrongBlockType => f.write_str("Mood metadata requires LightController2."),
            Self::ContainerCount { found } => write!(
                f,
                "Expected one direct LightscenesC container, found {found}."
            ),
            Self::MoodMatchCount { found } => write!(
                f,
                "Expected one direct mood matching exact stored SID or Name, found {found}."
            ),
            Self::InvalidSelectedSid => f.write_str("Selected mood SID is not a stored i32."),
            Self::NonpositiveSelectedSid => f.write_str("Selected mood SID must be positive."),
            Self::InvalidSiblingSid { index } => {
                write!(f, "Direct mood at child index {index} has no supported SID.")
            }
            Self::AmbiguousSid => f.write_str("The selected numeric SID is not unique."),
            Self::UnsupportedSkipValue { raw } => {
                write!(f, "Unsupported selected SkipNext spelling: {raw:?}.")
            }
            Self::NamespacedXml => f.write_str("SkipNext editing does not support namespaces."),
            Self::UnsupportedXml => {
                f.write_str("SkipNext editing requires one UTF-8 XML 1.0 root and no DTD.")
            }
            Self::UnsupportedCanonicalContent => f.write_str(
                "SkipNext editing requires element-only XML with supported attribute and layout content.",
            ),
            Self::XmlDepthLimit { max_depth } => {
                write!(f, "SkipNext editing supports at most {max_depth} nested XML elements.")
            }
            Self::DocumentScope => f.write_str(
                "SkipNext editing requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => {
                f.write_str("SkipNext editing does not support template or inherited ownership.")
            }
            Self::CanonicalReadmission(error) => {
                write!(f, "Canonical SkipNext output was refused: {error}")
            }
            Self::CanonicalPreservation => {
                f.write_str("Canonical SkipNext output did not preserve the edited tree or selection.")
            }
        }
    }
}

impl Error for LightingMoodSkipError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::CanonicalReadmission(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<ScopeError> for LightingMoodSkipError {
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

/// Edit the stored `SkipNext` on one direct mood of one offline `LightController2`.
///
/// The byte API requires `uuid:<nonempty U>` (case-sensitive, no UUID mirror
/// deduplication) and matches the mood by exact stored SID spelling or Name.
/// Refusals precede mutation: selector syntax, XML admission, unique controller
/// and direct mood, ordinary ownership, explicit positive selected SID, valid
/// sibling SIDs and numeric uniqueness, then supported selected flag storage.
///
/// The XML boundary admits one namespace- and DTD-free UTF-8 XML 1.0 root with
/// no declaration or one of `<?xml version="1.0"?>`,
/// `<?xml version="1.0" encoding="utf-8"?>` and its `UTF-8` spelling.
/// Only elements and attributes are admitted, with newline-bearing whitespace
/// in containers treated as layout. Depth is capped at 64 elements, including
/// root and self-closing leaves. Comments, PIs, CDATA, other text, duplicate
/// attributes, and input undergoing legacy digit-attribute rewriting are
/// outside this API. Attribute values allow the five predefined XML entities;
/// numeric references and literal `<`/tabs/line endings are not admitted.
/// These restrictions do not apply to the CLI's existing admission.
///
/// Omission is false; explicit flags must be ASCII case variants of true or
/// false. An effective no-op returns the input unchanged and unstamped. A
/// change writes only the selected attribute and the owning Document save
/// stamp, if present, from the explicit clock `now` and existing stamp policy.
/// Changed output is re-admitted in full and must preserve the edited DOM,
/// selected path, BOM and outer line-ending mode. No filesystem, clock,
/// native dialog or runtime is accessed, and errors return no revised bytes.
pub fn set_lighting_mood_skip(
    bytes: &[u8],
    selector: &str,
    mood_ref: &str,
    skip: bool,
    now: &DateTime<FixedOffset>,
) -> Result<LightingMoodSkipOutcome, LightingMoodSkipError> {
    use LightingMoodSkipError as Refusal;

    if selector.strip_prefix("uuid:").is_none_or(str::is_empty) {
        return Err(Refusal::UnsupportedSelector);
    }
    let (mut state, path, change) = read_skip_request(bytes, selector, mood_ref, skip)?;
    if !change.changed {
        return Ok(LightingMoodSkipOutcome {
            bytes: bytes.to_vec(),
            change,
            document_stamped: false,
        });
    }

    state.apply_lighting_mood_skip(&path, &change);
    let document_stamped = state.stamp_document(now);
    let bytes = state.to_bytes().map_err(Refusal::Edit)?;
    let (reread, reread_path, reread_change) = read_skip_request(&bytes, selector, mood_ref, skip)
        .map_err(|error| Refusal::CanonicalReadmission(Box::new(error)))?;
    if reread.root != state.root
        || reread_path != path
        || reread_change.changed
        || reread_change.before != change.after
        || reread.had_bom != state.had_bom
        || reread.had_crlf != state.had_crlf
    {
        return Err(Refusal::CanonicalPreservation);
    }
    Ok(LightingMoodSkipOutcome {
        bytes,
        change,
        document_stamped,
    })
}

fn read_skip_request(
    bytes: &[u8],
    selector: &str,
    mood_ref: &str,
    skip: bool,
) -> Result<(State, Vec<usize>, LightingMoodSkipChange), LightingMoodSkipError> {
    use LightingMoodSkipError as Refusal;

    require_plain_xml(bytes)?;
    require_canonical_content(bytes)?;
    let state = State::load(bytes).map_err(Refusal::Edit)?;
    if !state.digit_attr_renames.is_empty() {
        return Err(Refusal::UnsupportedCanonicalContent);
    }
    let mut owners = state.find_elements(selector);
    if owners.len() != 1 {
        return Err(Refusal::ControllerCount {
            found: owners.len(),
        });
    }
    let path = state.lighting_metadata_mood_path(owners.pop().unwrap(), mood_ref)?;
    require_ordinary_owner(&state, &path)?;
    let change = state.plan_lighting_mood_skip(&path, skip)?;
    Ok((state, path, change))
}

fn require_canonical_content(bytes: &[u8]) -> Result<(), LightingMoodSkipError> {
    use LightingMoodSkipError as Refusal;

    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| Refusal::UnsupportedXml)?;
    if text.starts_with('\u{feff}') || !text.chars().all(|c| {
        matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
    }) {
        return Err(Refusal::UnsupportedXml);
    }

    let mut reader = Reader::from_str(text);
    reader.config_mut().expand_empty_elements = true;
    let mut first = true;
    // Whitespace in a leaf or inline between tags can be data.
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
                if containers.len() == MAX_XML_DEPTH {
                    return Err(Refusal::XmlDepthLimit {
                        max_depth: MAX_XML_DEPTH,
                    });
                }
                for attr in start.attributes() {
                    let attr = attr.map_err(|_| Refusal::UnsupportedCanonicalContent)?;
                    if !supported_attribute_value(&attr.value) {
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
                    .all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n')) =>
            {
                if let Some((_, has_layout)) = containers.last_mut() {
                    if !text.contains(&b'\n') {
                        return Err(Refusal::UnsupportedCanonicalContent);
                    }
                    *has_layout = true;
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

fn supported_attribute_value(value: &[u8]) -> bool {
    !value
        .iter()
        .any(|b| matches!(b, b'<' | b'\t' | b'\r' | b'\n'))
        && value.split(|b| *b == b'&').skip(1).all(|tail| {
            [b"amp;".as_slice(), b"lt;", b"gt;", b"apos;", b"quot;"]
                .iter()
                .any(|entity| tail.starts_with(entity))
        })
}

impl State {
    fn lighting_metadata_mood_path(
        &self,
        mut path: Vec<usize>,
        mood_ref: &str,
    ) -> Result<Vec<usize>, LightingMoodSkipError> {
        use LightingMoodSkipError as Refusal;

        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Refusal::WrongBlockType);
        }
        let collections: Vec<_> = block
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| element.name == "LightscenesC")
                    .map(|element| (index, element))
            })
            .collect();
        let [(collection_index, collection)] = collections.as_slice() else {
            return Err(Refusal::ContainerCount {
                found: collections.len(),
            });
        };
        let matches: Vec<_> = collection
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| {
                        element.name == "LightsceneC"
                            && ["SID", "Name"].iter().any(|key| {
                                element.attributes.get(*key).map(String::as_str) == Some(mood_ref)
                            })
                    })
                    .map(|_| index)
            })
            .collect();
        let [mood_index] = matches.as_slice() else {
            return Err(Refusal::MoodMatchCount {
                found: matches.len(),
            });
        };
        path.extend([*collection_index, *mood_index]);
        Ok(path)
    }

    fn plan_lighting_mood_skip(
        &self,
        path: &[usize],
        skip: bool,
    ) -> Result<LightingMoodSkipChange, LightingMoodSkipError> {
        use LightingMoodSkipError as Refusal;

        let mood = self.get_element(path);
        let raw_sid = mood
            .attributes
            .get("SID")
            .ok_or(Refusal::InvalidSelectedSid)?;
        let sid = raw_sid
            .parse::<i32>()
            .map_err(|_| Refusal::InvalidSelectedSid)?;
        if sid <= 0 {
            return Err(Refusal::NonpositiveSelectedSid);
        }
        let collection = self.get_element(&path[..path.len() - 1]);
        let mut matching_sids = 0;
        for (index, node) in collection.children.iter().enumerate() {
            let Some(sibling) = node.as_element().filter(|e| e.name == "LightsceneC") else {
                continue;
            };
            let sibling_sid = sibling
                .attributes
                .get("SID")
                .and_then(|raw| raw.parse::<i32>().ok())
                .ok_or(Refusal::InvalidSiblingSid { index })?;
            matching_sids += usize::from(sibling_sid == sid);
        }
        if matching_sids != 1 {
            return Err(Refusal::AmbiguousSid);
        }
        let before = mood.attributes.get("SkipNext").cloned();
        let before_effective = match before.as_deref() {
            None => false,
            Some(raw) if raw.eq_ignore_ascii_case("false") => false,
            Some(raw) if raw.eq_ignore_ascii_case("true") => true,
            Some(raw) => return Err(Refusal::UnsupportedSkipValue { raw: raw.into() }),
        };
        let changed = before_effective != skip;
        Ok(LightingMoodSkipChange {
            mood_uuid: mood.attributes.get("UUID").cloned(),
            sid: raw_sid.clone(),
            cid: mood.attributes.get("CID").cloned(),
            after: if changed {
                Some(skip.to_string())
            } else {
                before.clone()
            },
            before,
            before_effective,
            skip,
            changed,
        })
    }

    fn apply_lighting_mood_skip(&mut self, path: &[usize], change: &LightingMoodSkipChange) {
        if let Some(after) = &change.after
            && change.changed
        {
            self.get_element_mut(path)
                .attributes
                .insert("SkipNext".into(), after.clone());
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn select_lighting_metadata_mood(
        &self,
        selector: &str,
        mood_ref: &str,
    ) -> Result<Vec<usize>, LightingMoodSkipError> {
        let path = self
            .require_one(selector)
            .map_err(EditError::from)
            .map_err(LightingMoodSkipError::Edit)?;
        self.lighting_metadata_mood_path(path, mood_ref)
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn preview_lighting_mood_skip_selected(
        &self,
        selector: &str,
        mood_ref: &str,
        skip: bool,
    ) -> Result<LightingMoodSkipChange, LightingMoodSkipError> {
        let path = self.select_lighting_metadata_mood(selector, mood_ref)?;
        self.plan_lighting_mood_skip(&path, skip)
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_lighting_mood_skip_selected(
        &mut self,
        selector: &str,
        mood_ref: &str,
        skip: bool,
    ) -> Result<LightingMoodSkipChange, LightingMoodSkipError> {
        let path = self.select_lighting_metadata_mood(selector, mood_ref)?;
        let change = self.plan_lighting_mood_skip(&path, skip)?;
        self.apply_lighting_mood_skip(&path, &change);
        Ok(change)
    }
}
