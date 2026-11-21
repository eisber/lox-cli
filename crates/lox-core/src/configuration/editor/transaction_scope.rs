use quick_xml::{
    Reader,
    events::{Event, attributes::Attributes},
};
use xmltree::Element;

use super::{EditError, legacy::State};

pub(super) enum ScopeError {
    Edit(EditError),
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
}

pub(super) fn require_xml_1_0_declaration(bytes: &[u8]) -> Result<(), ScopeError> {
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
                    return Err(ScopeError::UnsupportedXml);
                }
            }
            Event::PI(pi) if pi.target().eq_ignore_ascii_case(b"xml") => {
                return Err(ScopeError::UnsupportedXml);
            }
            Event::Eof => break,
            _ => {}
        }
        first = false;
    }
    // Structural parse failures remain the shared loader's diagnostics.
    Ok(())
}

pub(super) fn require_plain_xml(bytes: &[u8]) -> Result<(), ScopeError> {
    require_plain_xml_scope(bytes, false).map(|_| ())
}

/// Returns whether input contains whitespace that cannot safely be treated as indentation.
pub(super) fn require_round_trip_xml(bytes: &[u8]) -> Result<bool, ScopeError> {
    require_plain_xml_scope(bytes, true)
}

#[derive(Default)]
struct TextScope {
    has_element: bool,
    has_payload: bool,
    has_indent: bool,
}

fn xml_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn require_plain_xml_scope(bytes: &[u8], round_trip: bool) -> Result<bool, ScopeError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ScopeError::UnsupportedXml)?;
    if bytes.contains(&0) {
        return Err(ScopeError::UnsupportedXml);
    }
    // xmltree discards attribute prefixes. Check qualified names with the XML
    // reader before trusting DOM attributes; comments, CDATA and PI are opaque.
    let mut reader = Reader::from_str(text.strip_prefix('\u{feff}').unwrap_or(text));
    reader.config_mut().expand_empty_elements = true;
    let mut depth = 0usize;
    let mut root_seen = false;
    let mut text_scopes = Vec::<TextScope>::new();
    let mut ambiguous_whitespace = false;
    loop {
        match reader
            .read_event()
            .map_err(|error| ScopeError::Edit(EditError::load(error)))?
        {
            Event::Start(element) => {
                if depth == 0 {
                    if root_seen {
                        return Err(ScopeError::UnsupportedXml);
                    }
                    root_seen = true;
                }
                depth += 1;
                if round_trip {
                    if let Some(parent) = text_scopes.last_mut() {
                        parent.has_element = true;
                    }
                    text_scopes.push(TextScope::default());
                }
                if element.name().as_ref().contains(&b':') {
                    return Err(ScopeError::NamespacedXml);
                }
                // Legacy callers allow identical duplicates; strict writers cannot retain them.
                for attr in element.attributes().with_checks(round_trip) {
                    let attr = attr.map_err(|error| {
                        if round_trip {
                            ScopeError::UnsupportedXml
                        } else {
                            ScopeError::Edit(EditError::load(error))
                        }
                    })?;
                    let name = attr.key.as_ref();
                    if name == b"xmlns" || name.contains(&b':') {
                        return Err(ScopeError::NamespacedXml);
                    }
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).ok_or(ScopeError::UnsupportedXml)?;
                if let Some(scope) = text_scopes.pop() {
                    ambiguous_whitespace |=
                        scope.has_indent && (!scope.has_element || scope.has_payload);
                }
            }
            Event::DocType(_) => return Err(ScopeError::UnsupportedXml),
            Event::Decl(decl) => {
                if let Some(encoding) = decl.encoding()
                    && !encoding
                        .map_err(|error| ScopeError::Edit(EditError::load(error)))?
                        .eq_ignore_ascii_case(b"utf-8")
                {
                    return Err(ScopeError::UnsupportedXml);
                }
            }
            Event::Text(text) if depth == 0 => {
                if !text.as_ref().iter().copied().all(xml_space) {
                    return Err(ScopeError::UnsupportedXml);
                }
            }
            Event::Text(text) if round_trip => {
                let decoded = text
                    .unescape()
                    .map_err(|error| ScopeError::Edit(EditError::load(error)))?;
                let scope = text_scopes.last_mut().ok_or(ScopeError::UnsupportedXml)?;
                if !decoded.is_empty() && decoded.bytes().all(xml_space) {
                    // Only raw multiline indentation in element-only layout is admitted.
                    // An explicit character reference or an inline gap could be payload.
                    if text.as_ref().contains(&b'\n')
                        && text.as_ref().iter().copied().all(xml_space)
                    {
                        scope.has_indent = true;
                    } else {
                        ambiguous_whitespace = true;
                    }
                } else if !decoded.is_empty() {
                    scope.has_payload = true;
                }
            }
            Event::CData(_) if depth == 0 => return Err(ScopeError::UnsupportedXml),
            Event::CData(_) if round_trip => {
                let scope = text_scopes.last_mut().ok_or(ScopeError::UnsupportedXml)?;
                scope.has_payload = true;
            }
            // The canonical writer owns only the loaded root, not its surrounding nodes.
            Event::Comment(_) | Event::PI(_) if round_trip && depth == 0 => {
                return Err(ScopeError::UnsupportedXml);
            }
            Event::Eof if root_seen && depth == 0 => return Ok(ambiguous_whitespace),
            Event::Eof => return Err(ScopeError::UnsupportedXml),
            _ => {}
        }
    }
}

fn template(element: &Element) -> bool {
    matches!(element.name.as_str(), "Template" | "DocumentTemplate")
        || (element.name == "C"
            && matches!(
                element.attributes.get("Type").map(String::as_str),
                Some("Template" | "DocumentTemplate")
            ))
}

pub(super) fn require_ordinary_owner(state: &State, path: &[usize]) -> Result<(), ScopeError> {
    for depth in 0..=path.len() {
        let element = state.get_element(&path[..depth]);
        if template(element)
            || element.attributes.contains_key("IfT")
            || element.attributes.contains_key("uuidParent")
            || element.attributes.get("WF").is_some_and(|raw| {
                raw.parse::<u32>()
                    .map_or(true, |flags| flags & 0x2000_0000 != 0)
            })
        {
            return Err(ScopeError::TemplateOrInherited);
        }
    }
    let documents = state.find_elements("Type:Document");
    let document = match documents.as_slice() {
        [] => &state.root,
        [owner] if owner.len() <= 1 && path.starts_with(owner) => {
            let document = state.get_element(owner);
            if document.attributes.get("Type").map(String::as_str) != Some("Document") {
                return Err(ScopeError::DocumentScope);
            }
            document
        }
        _ => return Err(ScopeError::DocumentScope),
    };
    let has_direct_template = |element: &Element| {
        element
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .any(template)
    };
    if has_direct_template(document) || has_direct_template(&state.root) {
        return Err(ScopeError::TemplateOrInherited);
    }
    Ok(())
}
