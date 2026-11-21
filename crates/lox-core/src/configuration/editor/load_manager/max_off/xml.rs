use quick_xml::{
    Reader,
    escape::unescape,
    events::{BytesStart, Event},
};
use std::str;

use super::{MaxOffPlan, State};

pub(super) fn has_trusted_declaration(bytes: &[u8]) -> bool {
    let Ok(text) = str::from_utf8(bytes) else {
        return false;
    };
    if !text.chars().all(|c| {
        matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
    }) {
        return false;
    }

    let mut reader = Reader::from_reader(bytes);
    let mut first = true;
    // Let the shared loader and envelope guard diagnose structural XML errors.
    while let Ok(event) = reader.read_event() {
        match event {
            Event::Decl(decl) if !first || !valid_declaration(&decl) => return false,
            Event::PI(pi) if pi.target().eq_ignore_ascii_case(b"xml") => return false,
            Event::Eof => break,
            _ => {}
        }
        first = false;
    }
    true
}

fn xml_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn skip_space(input: &[u8]) -> &[u8] {
    &input[input.iter().take_while(|&&byte| xml_space(byte)).count()..]
}

fn declaration_value<'a>(input: &mut &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let rest = skip_space(input);
    if rest.len() == input.len() {
        return None;
    }
    let rest = skip_space(skip_space(rest.strip_prefix(key)?).strip_prefix(b"=")?);
    let (&quote, value) = rest.split_first()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let end = value.iter().position(|&byte| byte == quote)?;
    *input = &value[end + 1..];
    Some(&value[..end])
}

fn valid_declaration(declaration: &[u8]) -> bool {
    // The writer emits 1.0 without standalone; admit only declarations it can
    // replace without dropping information. Check separators and order explicitly.
    let Some(mut tail) = declaration.strip_prefix(b"xml") else {
        return false;
    };
    if !matches!(declaration_value(&mut tail, b"version"), Some(b"1.0")) {
        return false;
    }
    skip_space(tail).is_empty()
        || (declaration_value(&mut tail, b"encoding")
            .is_some_and(|value| value.eq_ignore_ascii_case(b"utf-8"))
            && skip_space(tail).is_empty())
}

#[derive(Default)]
struct Content {
    element: bool,
    text: bool,
    padding: bool,
}

struct Payload<'a> {
    reader: Reader<&'a [u8]>,
    scopes: Vec<Content>,
}

impl<'a> Payload<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        let mut reader = Reader::from_reader(bytes);
        reader.config_mut().expand_empty_elements = true;
        Self {
            reader,
            scopes: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<Event<'a>> {
        loop {
            let event = self.reader.read_event().ok()?;
            match &event {
                Event::Decl(_) => continue,
                Event::Start(_) => {
                    if let Some(parent) = self.scopes.last_mut() {
                        parent.element = true;
                    }
                    self.scopes.push(Content::default());
                }
                Event::End(_) => {
                    let scope = self.scopes.pop()?;
                    // Whitespace-only runs are layout in element-only containers,
                    // but the DOM may drop data in leaf or mixed text content.
                    if scope.padding && (!scope.element || scope.text) {
                        return None;
                    }
                }
                Event::Text(text) if text.as_ref().iter().copied().all(xml_space) => {
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.padding = true;
                    }
                    continue;
                }
                Event::Text(_) | Event::CData(_) => {
                    self.scopes.last_mut()?.text = true;
                }
                _ => {}
            }
            return Some(event);
        }
    }
}

#[derive(PartialEq, Eq)]
struct Value {
    decoded: String,
    normalized_xml: Option<String>,
}

fn value(raw: &[u8], in_attribute: bool) -> Option<Value> {
    let raw = str::from_utf8(raw).ok()?;
    let decoded = unescape(raw).ok()?;
    // XML 1.0 folds literal EOL (and attribute whitespace) before expanding
    // character references. The legacy DOM does not always make that distinction.
    let normalized_xml = if raw.contains('\r') || (in_attribute && raw.contains(['\n', '\t'])) {
        let eol = raw.replace("\r\n", "\n").replace('\r', "\n");
        let folded = if in_attribute {
            eol.replace(['\n', '\t'], " ")
        } else {
            eol
        };
        let normalized = unescape(&folded).ok()?;
        (normalized != decoded).then(|| normalized.into_owned())
    } else {
        None
    };
    Some(Value {
        decoded: decoded.into_owned(),
        normalized_xml,
    })
}

type Attributes = Vec<(String, Value)>;

fn attributes(element: &BytesStart<'_>) -> Option<Attributes> {
    let mut ordered = Attributes::new();
    for attribute in element.attributes().with_checks(false) {
        let attribute = attribute.ok()?;
        let key = str::from_utf8(attribute.key.as_ref()).ok()?;
        let value = value(attribute.value.as_ref(), true)?;
        if let Some((_, previous)) = ordered.iter().find(|(name, _)| name == key) {
            if previous != &value {
                return None;
            }
        } else {
            ordered.push((key.to_owned(), value));
        }
    }
    Some(ordered)
}

fn overwrite(attributes: &mut Attributes, key: &str, value: &str) {
    let value = Value {
        decoded: value.to_owned(),
        normalized_xml: None,
    };
    if let Some((_, old)) = attributes.iter_mut().find(|(name, _)| name == key) {
        *old = value;
    } else {
        attributes.push((key.to_owned(), value));
    }
}

pub(super) fn preserves_payload(
    source: &[u8],
    revised: &[u8],
    state: &State,
    path: &[usize],
    plan: &MaxOffPlan,
    document_stamped: bool,
) -> bool {
    let selected = state.get_element(path).children[plan.position]
        .as_element()
        .unwrap();
    let elements = State::iter_elements(&state.root);
    let target_index = elements
        .iter()
        .position(|&element| std::ptr::eq(element, selected))
        .unwrap();
    let stamp = if document_stamped {
        let Some((index, owner)) = elements.iter().enumerate().find(|(_, element)| {
            element.name == "C"
                && element.attributes.get("Type").map(String::as_str) == Some("Document")
        }) else {
            return false;
        };
        Some((index, &owner.attributes["Date"], &owner.attributes["DateS"]))
    } else {
        None
    };
    let mut source = Payload::new(source);
    let mut revised = Payload::new(revised);
    let mut index = 0;
    // Compare the byte-side payload too: DOM equality alone misses discarded
    // prolog/epilog nodes and comment/PI padding the writer can introduce.
    loop {
        match (source.next(), revised.next()) {
            (Some(Event::Start(a)), Some(Event::Start(b))) if a.name() == b.name() => {
                let (Some(mut expected), Some(actual)) = (attributes(&a), attributes(&b)) else {
                    return false;
                };
                if index == target_index {
                    let before = expected
                        .iter()
                        .find(|(name, _)| name == "maxOff")
                        .map(|(_, value)| value.decoded.as_str());
                    if before != plan.change.before.as_deref() {
                        return false;
                    }
                    overwrite(&mut expected, "maxOff", &plan.change.after);
                }
                if let Some((owner_index, date, dates)) = stamp
                    && index == owner_index
                {
                    overwrite(&mut expected, "Date", date);
                    overwrite(&mut expected, "DateS", dates);
                }
                if actual != expected {
                    return false;
                }
                index += 1;
            }
            (Some(Event::End(a)), Some(Event::End(b))) if a.name() == b.name() => {}
            (Some(Event::Text(a)), Some(Event::Text(b))) => {
                if !matches!((value(&a, false), value(&b, false)), (Some(a), Some(b)) if a == b) {
                    return false;
                }
            }
            (Some(Event::Comment(a)), Some(Event::Comment(b))) if *a == *b => {}
            (Some(Event::CData(a)), Some(Event::CData(b))) if *a == *b => {}
            (Some(Event::PI(a)), Some(Event::PI(b))) if *a == *b => {}
            (Some(Event::Eof), Some(Event::Eof)) => return index == elements.len(),
            _ => return false,
        }
    }
}
