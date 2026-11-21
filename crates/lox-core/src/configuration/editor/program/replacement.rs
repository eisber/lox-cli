use super::{EditError, ElementIdentity, ProgramTextError, State, is_configuration_field};
use chrono::{DateTime, FixedOffset};
use quick_xml::{Reader, events::Event};
use std::{error::Error, fmt};
use xmltree::XMLNode;

#[derive(Debug)]
pub enum ProgramTextReplacementError {
    Edit(EditError),
    UnsupportedType { element: ElementIdentity },
    UnsupportedDocument,
    LoaderRewrite,
    UnsupportedOwner { element: ElementIdentity },
    MissingField { element: ElementIdentity },
    AmbiguousField { element: ElementIdentity },
    UnsupportedField { element: ElementIdentity },
    UnsupportedText,
    RoundTrip,
}

impl fmt::Display for ProgramTextReplacementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::UnsupportedType { element } => {
                write!(f, "'{}' is not a SequenceController", element.title)
            }
            Self::UnsupportedDocument => f.write_str(
                "Program text replacement requires a plain UTF-8 XML 1.0 document without duplicate attributes or external markup",
            ),
            Self::LoaderRewrite => f.write_str(
                "Program text replacement refuses shared loader digit-name rewrites (lox-n37w)",
            ),
            Self::UnsupportedOwner { element } => {
                write!(f, "'{}' has no unique ordinary program owner", element.title)
            }
            Self::MissingField { element } => {
                write!(f, "'{}' has no direct Configuration field", element.title)
            }
            Self::AmbiguousField { element } => {
                write!(f, "'{}' has multiple direct Configuration fields", element.title)
            }
            Self::UnsupportedField { element } => write!(
                f,
                "'{}' does not store a single direct Configuration text node",
                element.title
            ),
            Self::UnsupportedText => f.write_str(
                "Replacement must be nonblank XML 1.0 text with LF line breaks and no ]]>",
            ),
            Self::RoundTrip => f.write_str(
                "Program text replacement cannot preserve the loaded document through serialization",
            ),
        }
    }
}

impl Error for ProgramTextReplacementError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EditError> for ProgramTextReplacementError {
    fn from(error: EditError) -> Self {
        Self::Edit(error)
    }
}

#[derive(Debug)]
pub struct ProgramTextReplacementOutcome {
    pub bytes: Vec<u8>,
    pub element: ElementIdentity,
    /// Compared with an unedited canonical serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Replace the stored text of one existing, unambiguous SequenceController.
///
/// The legacy selector runs first, followed by the exact type check. The selected
/// owner must have a unique, nonempty stored `U`. If a Document exists, it must be
/// the sole stampable Document and own the target. Device bindings,
/// templates/instances and alternate storage in `Configuration` or `Value`
/// attributes are not admitted. There must be exactly one direct
/// `Field Name="Configuration"` with one nonblank text node.
/// A nested or differently spelled field does not count. Nothing is created.
///
/// This is an XML storage operation: it does not validate or execute sequence
/// syntax. The CLI keeps its existing validator and more permissive legacy path.
/// Input must be plain UTF-8 XML 1.0, with no namespaces, DTD, top-level comment
/// or processing instruction, standalone declaration, or duplicate attributes.
/// The shared loader is not repaired: input or output affected by digit-name
/// rewriting is refused.
///
/// Both canonical serializations must reload without changes to the retained
/// DOM. Formatting, entity spelling and whitespace-only nodes ignored by the
/// legacy editor can normalize. Replacement text must be nonblank XML 1.0 text
/// with LF (not CR/CRLF) line breaks and no `]]>` delimiter. Serialized output
/// follows the document's line endings. Changed canonical content receives the
/// existing monotonic stamp using `now`, if it has a Document.
/// Canonical no-ops do not stamp. Errors return no revised bytes.
pub fn replace_program_text(
    bytes: &[u8],
    selector: &str,
    program: &str,
    now: &DateTime<FixedOffset>,
) -> Result<ProgramTextReplacementOutcome, ProgramTextReplacementError> {
    use ProgramTextReplacementError as Refusal;

    let mut state = State::load(bytes)?;
    let (path, element) = state
        .select_program_owner(selector)
        .map_err(|error| match error {
            ProgramTextError::Edit(error) => Refusal::Edit(error),
            ProgramTextError::Unsupported { element } => Refusal::UnsupportedType { element },
        })?;
    if !state.digit_attr_renames.is_empty() {
        return Err(Refusal::LoaderRewrite);
    }
    if !is_supported_document(bytes) {
        return Err(Refusal::UnsupportedDocument);
    }
    if !is_supported_owner(&state, &path, &element) {
        return Err(Refusal::UnsupportedOwner { element });
    }
    let owner = state.get_element(&path);
    let mut fields = owner
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|field| is_configuration_field(field));
    let Some(field) = fields.next() else {
        return Err(Refusal::MissingField { element });
    };
    if fields.next().is_some() {
        return Err(Refusal::AmbiguousField { element });
    }
    if owner.attributes.contains_key("Configuration")
        || field.attributes.contains_key("Value")
        || !matches!(field.children.as_slice(), [XMLNode::Text(text)] if !text.trim().is_empty())
    {
        return Err(Refusal::UnsupportedField { element });
    }
    if program.trim().is_empty()
        || program.contains('\r')
        || program.contains("]]>")
        || !program.chars().all(is_xml_1_0_char)
    {
        return Err(Refusal::UnsupportedText);
    }
    if !State::sanitize_digit_attrs(program.as_bytes()).1.is_empty() {
        return Err(Refusal::LoaderRewrite);
    }

    let baseline = state.to_bytes()?;
    verify_round_trip(&state, &baseline)?;
    let text = if state.had_crlf {
        program.replace('\n', "\r\n")
    } else {
        program.to_owned()
    };
    state.write_program_field(&path, text);
    let mut output = state.to_bytes()?;
    let content_changed = output != baseline;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    verify_round_trip(&state, &output)?;
    Ok(ProgramTextReplacementOutcome {
        bytes: output,
        element,
        content_changed,
        document_stamped,
    })
}

fn verify_round_trip(state: &State, bytes: &[u8]) -> Result<(), ProgramTextReplacementError> {
    let readback = State::load(bytes).map_err(|_| ProgramTextReplacementError::RoundTrip)?;
    if !readback.digit_attr_renames.is_empty() {
        return Err(ProgramTextReplacementError::LoaderRewrite);
    }
    if readback.root != state.root {
        return Err(ProgramTextReplacementError::RoundTrip);
    }
    Ok(())
}

fn is_supported_owner(state: &State, path: &[usize], element: &ElementIdentity) -> bool {
    if element.uuid.is_empty()
        || State::iter_elements(&state.root)
            .into_iter()
            .filter(|node| node.attributes.get("U") == Some(&element.uuid))
            .take(2)
            .count()
            != 1
        || state.get_element(path).attributes.contains_key("Dev")
        || (0..=path.len()).any(|i| {
            let node = state.get_element(&path[..i]);
            matches!(node.name.as_str(), "Template" | "AutomaticDesigner")
                || matches!(
                    node.attributes.get("Type").map(String::as_str),
                    Some("Template" | "DocumentTemplate" | "AutomaticDesigner")
                )
                || node.attributes.contains_key("IfT")
                || node.attributes.contains_key("uuidParent")
        })
    {
        return false;
    }
    match state.find_elements("Type:Document").as_slice() {
        [] => true,
        [document] => {
            document.len() <= 1
                && path.len() > document.len()
                && path.starts_with(document)
                && state
                    .get_element(document)
                    .attributes
                    .get("Type")
                    .map(String::as_str)
                    == Some("Document")
        }
        _ => false,
    }
}

fn is_xml_1_0_char(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
    )
}

// The DOM loses qualified attribute names, duplicate attributes and outer
// markup before it can compare the canonical copy with the loaded document.
fn is_supported_document(bytes: &[u8]) -> bool {
    if !std::str::from_utf8(bytes).is_ok_and(|xml| xml.chars().all(is_xml_1_0_char)) {
        return false;
    }
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().check_comments = true;
    let mut depth = 0usize;
    let mut roots = 0usize;
    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(ref tag) | Event::Empty(ref tag) => {
                if tag.name().as_ref().contains(&b':')
                    || tag.attributes().any(|attribute| match attribute {
                        Ok(attribute) => {
                            let name = attribute.key.as_ref();
                            name == b"xmlns" || name.contains(&b':')
                        }
                        Err(_) => true,
                    })
                {
                    return false;
                }
                roots += usize::from(depth == 0);
                depth += usize::from(matches!(event, Event::Start(_)));
            }
            Event::End(_) if depth != 0 => depth -= 1,
            Event::Decl(decl) if roots == 0 => {
                if !decl.version().is_ok_and(|version| version.as_ref() == b"1.0")
                    || decl.encoding().is_some_and(|encoding| {
                        !encoding.is_ok_and(|encoding| encoding.eq_ignore_ascii_case(b"utf-8"))
                    })
                    || decl.standalone().is_some()
                {
                    return false;
                }
            }
            Event::Text(text) if depth == 0 && text.iter().all(u8::is_ascii_whitespace) => {}
            Event::Text(text) if depth != 0 && !text.windows(3).any(|part| part == b"]]>") => {}
            Event::CData(_) | Event::Comment(_) | Event::PI(_) if depth != 0 => {}
            Event::Eof => return depth == 0 && roots == 1,
            _ => return false,
        }
    }
    false
}
