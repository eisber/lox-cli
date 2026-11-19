use quick_xml::{Reader, events::Event};
use xmltree::Element;

use super::{EditError, legacy::State};

pub(super) enum ScopeError {
    Edit(EditError),
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
}

pub(super) fn require_plain_xml(bytes: &[u8]) -> Result<(), ScopeError> {
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
                if element.name().as_ref().contains(&b':') {
                    return Err(ScopeError::NamespacedXml);
                }
                // Identical duplicate attributes remain the shared loader's policy.
                for attr in element.attributes().with_checks(false) {
                    let attr = attr.map_err(|error| ScopeError::Edit(EditError::load(error)))?;
                    let name = attr.key.as_ref();
                    if name == b"xmlns" || name.contains(&b':') {
                        return Err(ScopeError::NamespacedXml);
                    }
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).ok_or(ScopeError::UnsupportedXml)?;
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
                if !text
                    .as_ref()
                    .iter()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
                {
                    return Err(ScopeError::UnsupportedXml);
                }
            }
            Event::CData(_) if depth == 0 => return Err(ScopeError::UnsupportedXml),
            Event::Eof if root_seen && depth == 0 => return Ok(()),
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
