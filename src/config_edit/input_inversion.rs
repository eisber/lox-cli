use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use quick_xml::{Reader, events::Event};
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::{ConfigEditor, alarm_clock::entry_name_ordinal, icons::native_identity};

#[derive(Debug, Serialize)]
pub struct InputInversionChange {
    pub block_uuid: String,
    pub connector_uuid: String,
    pub connector: String,
    pub previous_inverted: bool,
    pub inverted: bool,
    pub changed: bool,
}

fn stored_identity(raw: &str) -> Option<uuid::Uuid> {
    (raw.len() == 35)
        .then(|| native_identity(raw))
        .flatten()
        .filter(|identity| !identity.is_nil())
}

fn kind(element: &Element) -> &str {
    element.attributes.get("Type").map_or("", String::as_str)
}

impl ConfigEditor {
    /// Edit a recovered ordinary, unconnected logic input without a DOM resave.
    pub fn edit_input_inversion(
        data: &[u8],
        selector: &str,
        key: &str,
        inverted: bool,
    ) -> Result<(Vec<u8>, InputInversionChange)> {
        let requested = selector.strip_prefix("uuid:").unwrap_or("");
        ensure!(
            stored_identity(requested).is_some(),
            "Input inversion requires uuid:<exact nonnil native35 owner U> (8-4-4-16); nothing was changed."
        );
        ensure!(
            matches!(key, "I" | "I1" | "I2"),
            "Input inversion supports only existing unconnected And/Or/Xor.I1/I2 or Not.I, not aliases, outputs, analog inputs or parameters; nothing was changed."
        );
        ensure!(
            Self::sanitize_digit_attrs(data).1.is_empty(),
            "Input inversion refuses shared-loader digit-name rewrites (lox-n37w), including digit-like text; nothing was changed."
        );
        let mut editor = Self::load(data).context(
            "Input inversion cannot admit malformed XML or conflicting duplicate attributes; nothing was changed",
        )?;
        let all = Self::iter_elements(&editor.root);
        ensure!(
            all.iter().all(|element| {
                element.namespace.is_none()
                    && element.prefix.is_none()
                    && !element.attributes.keys().any(|key| key.contains(':'))
                    && element.name != "Template"
                    && kind(element) != "DocumentTemplate"
                    && !element.attributes.contains_key("IfT")
                    && !element.attributes.contains_key("uuidParent")
            }),
            "Input inversion does not support namespaces, Template/IfT/DocumentTemplate or uuidParent instance state; nothing was changed."
        );
        let documents = editor.find_elements("Type:Document");
        let owners = editor.find_elements(selector);
        ensure!(
            documents.len() == 1
                && documents[0].len() <= 1
                && owners.len() == 1
                && owners[0].starts_with(&documents[0])
                && (owners[0].len() == documents[0].len() + 2
                    || owners[0].len() == documents[0].len() + 3),
            "Input inversion requires one exact owner directly on a Page in one root/direct ordinary Document, optionally through one Program; duplicate/missing owners are refused. Nothing was changed."
        );
        let document = editor.get_element(&documents[0]);
        let owner_path = &owners[0];
        let page = editor.get_element(&owner_path[..owner_path.len() - 1]);
        let owner = editor.get_element(owner_path);
        let program = if owner_path.len() == documents[0].len() + 3 {
            let program = editor.get_element(&owner_path[..owner_path.len() - 2]);
            ensure!(
                kind(program) == "Program",
                "Input inversion supports only an ordinary Program between Document and Page, not other or nested wrappers; nothing was changed."
            );
            Some(program)
        } else {
            None
        };
        ensure!(
            kind(document) == "Document"
                && kind(page) == "Page"
                && matches!(kind(owner), "And" | "Or" | "Xor" | "Not")
                && owner.attributes.get("U").map(String::as_str) == Some(requested)
                && !owner.attributes.contains_key("Dev"),
            "Input inversion requires an ordinary, non-device-bound And, Or, Xor or Not directly on a Page; nothing was changed."
        );
        let (input_keys, connector_keys, nio): (&[&str], &[&str], &str) = match kind(owner) {
            "Not" => (&["I"], &["I", "Q"], "2"),
            _ => (&["I1", "I2"], &["I1", "I2", "Q"], "3"),
        };
        ensure!(
            input_keys.contains(&key)
                && owner.attributes.get("Nio").map(String::as_str) == Some(nio),
            "Input inversion requires the owner's exact native input and explicit layout: And/Or/Xor I1/I2 with Nio=3, or Not I with Nio=2; nothing was changed."
        );
        let mut identities = Vec::new();
        let mut guarded_owners = vec![document];
        guarded_owners.extend(program);
        guarded_owners.extend([page, owner]);
        for element in guarded_owners {
            ensure!(
                element.name == "C"
                    && !element.attributes.contains_key("Dev")
                    && element
                        .attributes
                        .get("V")
                        .and_then(|value| value.parse::<i32>().ok())
                        .is_some_and(|version| version >= 90)
                    && element
                        .attributes
                        .get("WF")
                        .is_none_or(|value| matches!(value.as_str(), "0" | "16384")),
                "Input inversion requires explicit modern V>=90 and absent/literal 0/16384 WF on Document, optional Program, Page and owner; other flags/default/legacy contexts are unsupported. Nothing was changed."
            );
            let raw = element.attributes.get("U").context(
                "Input inversion requires native35 owner/ancestor identities; nothing was changed",
            )?;
            let identity = stored_identity(raw)
                .context("Input inversion requires nonnil native35 owner/ancestor identities; nothing was changed")?;
            ensure!(
                all.iter()
                    .filter(|candidate| matches!(candidate.name.as_str(), "C" | "Co"))
                    .filter(|candidate| candidate
                        .attributes
                        .get("U")
                        .and_then(|value| native_identity(value))
                        == Some(identity))
                    .count()
                    == 1,
                "Input inversion requires unique effective identities, including case/RFC-grouping aliases; nothing was changed."
            );
            identities.push((
                entry_name_ordinal(&editor.root, element, &mut 0).unwrap(),
                "C".to_string(),
                kind(element).to_string(),
                raw.clone(),
            ));
        }
        ensure!(
            document.attributes.get("DateS").is_none_or(|value| {
                !value.is_empty()
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value
                        .parse::<i32>()
                        .is_ok_and(|seconds| (0..i32::MAX).contains(&seconds))
            }),
            "Input inversion requires an absent or incrementable nonnegative signed32 Document DateS; nothing was changed."
        );
        let connectors: Vec<_> = owner
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|child| child.name == "Co")
            .collect();
        ensure!(
            connectors.len() == connector_keys.len()
                && connector_keys.iter().all(|key| {
                    connectors
                        .iter()
                        .filter(|co| co.attributes.get("K").map(String::as_str) == Some(*key))
                        .count()
                        == 1
                }),
            "Input inversion requires exactly the owner's existing native connectors (And/Or/Xor I1/I2/Q, Not I/Q); duplicate/alias/default connector layouts are unsupported. Nothing was changed."
        );
        for connector in &connectors {
            let identity = connector
                .attributes
                .get("U")
                .and_then(|raw| stored_identity(raw))
                .context("Input inversion requires existing nonnil native35 identities for every owner connector; nothing was changed")?;
            ensure!(
                all.iter()
                    .filter(|candidate| matches!(candidate.name.as_str(), "C" | "Co"))
                    .filter(|candidate| candidate
                        .attributes
                        .get("U")
                        .and_then(|value| native_identity(value))
                        == Some(identity))
                    .count()
                    == 1,
                "Input inversion requires unique effective identities for every owner connector, including case/RFC aliases; nothing was changed."
            );
            identities.push((
                entry_name_ordinal(&editor.root, connector, &mut 0).unwrap(),
                "Co".to_string(),
                connector.attributes["K"].clone(),
                connector.attributes["U"].clone(),
            ));
        }
        let connector = connectors
            .iter()
            .find(|co| co.attributes.get("K").map(String::as_str) == Some(key))
            .unwrap();
        let connector_uuid = connector
            .attributes
            .get("U")
            .context("Input inversion requires an existing connector UUID; nothing was changed")?;
        let identity = stored_identity(connector_uuid).context(
            "Input inversion requires a nonnil native35 connector U; nothing was changed",
        )?;
        ensure!(
            connector
                .children
                .iter()
                .all(|child| child.as_element().is_none())
                && connector
                    .attributes
                    .get("Nc")
                    .is_none_or(|value| value == "0")
                && connector
                    .attributes
                    .get("Def")
                    .is_none_or(|value| value == "0")
                && !all.iter().any(|element| {
                    element.name == "Ref"
                        && element
                            .attributes
                            .get("U")
                            .and_then(|value| native_identity(value))
                            == Some(identity)
                }),
            "Input inversion supports only an unconnected input with absent/zero Nc and Def, no element children or references using it as a source; wired/default-value branches are unsupported. Nothing was changed."
        );
        let previous_inverted = match connector.attributes.get("Inv").map(String::as_str) {
            None | Some("false") => false,
            Some("true") => true,
            _ => anyhow::bail!(
                "Input inversion requires absent/true/false Inv storage; nothing was changed."
            ),
        };
        let target = entry_name_ordinal(&editor.root, connector, &mut 0).unwrap();
        let change = InputInversionChange {
            block_uuid: requested.to_string(),
            connector_uuid: connector_uuid.clone(),
            connector: key.to_string(),
            previous_inverted,
            inverted,
            changed: previous_inverted != inverted,
        };
        let document_ordinal = identities[0].0;
        let admitted = inversion_bytes(data, &identities, target, document_ordinal, None, None)?;
        let stamp = if change.changed {
            ensure!(
                editor.stamp_document(&chrono::Local::now().fixed_offset()),
                "Cannot stamp the admitted input-inversion Document; nothing was changed."
            );
            let document = editor.get_element(&documents[0]);
            Some((
                document.attributes["Date"].clone(),
                document.attributes["DateS"].clone(),
            ))
        } else {
            None
        };
        let output = if change.changed {
            inversion_bytes(
                data,
                &identities,
                target,
                document_ordinal,
                Some(&change),
                stamp.as_ref(),
            )?
        } else {
            admitted
        };
        Ok((output, change))
    }
}

fn raw_span(data: &[u8], value: &[u8]) -> Result<std::ops::Range<usize>> {
    let start = (value.as_ptr() as usize)
        .checked_sub(data.as_ptr() as usize)
        .context("Cannot locate raw input-inversion storage; nothing was changed")?;
    let end = start
        .checked_add(value.len())
        .context("Input-inversion storage range overflow; nothing was changed")?;
    ensure!(
        data.get(start..end) == Some(value),
        "Input-inversion storage is not borrowed from the original XML; nothing was changed."
    );
    Ok(start..end)
}

fn valid_processing_instruction(raw: &[u8]) -> bool {
    // XML 1.0 NameStartChar/NameChar, not a Unicode-letter approximation.
    fn name_start(character: char) -> bool {
        matches!(
            character,
            ':' | 'A'..='Z' | '_' | 'a'..='z'
                | '\u{c0}'..='\u{d6}' | '\u{d8}'..='\u{f6}' | '\u{f8}'..='\u{2ff}'
                | '\u{370}'..='\u{37d}' | '\u{37f}'..='\u{1fff}'
                | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}'
                | '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}'
                | '\u{f900}'..='\u{fdcf}' | '\u{fdf0}'..='\u{fffd}'
                | '\u{10000}'..='\u{effff}'
        )
    }

    let Ok(text) = std::str::from_utf8(raw) else {
        return false;
    };
    let target = text.split([' ', '\t', '\r', '\n']).next().unwrap_or("");
    let mut characters = target.chars();
    !target.eq_ignore_ascii_case("xml")
        && characters.next().is_some_and(name_start)
        && characters.all(|character| {
            name_start(character)
                || matches!(
                    character,
                    '-' | '.' | '0'..='9' | '\u{b7}' | '\u{300}'..='\u{36f}' | '\u{203f}'..='\u{2040}'
                )
        })
        && text.chars().all(|character| {
            matches!(
                character,
                '\t' | '\r' | '\n' | ' '..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
            )
        })
}

fn inversion_bytes(
    data: &[u8],
    identities: &[(usize, String, String, String)],
    target: usize,
    document: usize,
    change: Option<&InputInversionChange>,
    stamp: Option<&(String, String)>,
) -> Result<Vec<u8>> {
    const DOCUMENT_ERROR: &str = "Input inversion requires exactly one balanced XML root with no trailing elements or text; nothing was changed.";
    std::str::from_utf8(data)?;
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    let mut ordinal = 0;
    let mut found = 0;
    let mut patches = Vec::new();
    let mut root_seen = false;
    let mut open_elements = Vec::new();
    loop {
        let event_start = reader.buffer_position();
        let event = reader.read_event().context(DOCUMENT_ERROR)?;
        let is_start = matches!(&event, Event::Start(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                if open_elements.is_empty() {
                    ensure!(!root_seen, DOCUMENT_ERROR);
                    root_seen = true;
                }
                if is_start {
                    open_elements.push(element.name().as_ref().to_vec());
                }
                let end = reader.buffer_position() as usize + offset;
                let guarded = identities.iter().any(|identity| identity.0 == ordinal);
                let control = element.name().as_ref() == b"C";
                let connector = element.name().as_ref() == b"Co";
                let reference = element.name().as_ref() == b"Ref";
                let mut seen = BTreeMap::new();
                let mut attributes = Vec::new();
                for attribute in element.attributes().with_checks(false) {
                    let attribute = attribute.context(
                        "Input inversion cannot inspect raw attributes; nothing was changed",
                    )?;
                    let key = std::str::from_utf8(attribute.key.as_ref()).context(
                        "Input inversion requires UTF-8 attribute names; nothing was changed",
                    )?;
                    let global_identity = ((control || connector || reference) && key == "U")
                        || (control && key == "Type")
                        || (connector && key == "K");
                    if let Some(previous) = seen.insert(key.to_owned(), attribute.value.to_vec()) {
                        ensure!(
                            !guarded && !global_identity && previous == attribute.value.as_ref(),
                            "Input inversion refuses duplicate attribute '{key}' on guarded state/identities or conflicting raw values; nothing was changed."
                        );
                        // Deduplicate only this view; the original duplicate bytes stay untouched.
                        continue;
                    }
                    attributes.push(attribute);
                }
                for attribute in &attributes {
                    raw_span(data, attribute.key.as_ref())?;
                    let value = raw_span(data, attribute.value.as_ref())?;
                    ensure!(
                        matches!(data.get(value.end), Some(b'\'' | b'"')),
                        "Cannot locate a raw input-inversion attribute delimiter; nothing was changed."
                    );
                }
                ensure!(
                    !element.name().as_ref().contains(&b':')
                        && attributes.iter().all(|attribute| {
                            attribute.key.as_ref() != b"xmlns"
                                && !attribute.key.as_ref().contains(&b':')
                        }),
                    "Input inversion does not support XML namespaces; nothing was changed."
                );
                if let Some((_, tag, key, uuid)) =
                    identities.iter().find(|identity| identity.0 == ordinal)
                {
                    ensure!(
                        element.name().as_ref() == tag.as_bytes()
                            && attributes.iter().all(|attribute| !attribute
                                .value
                                .windows(2)
                                .any(|bytes| bytes == b"&#")),
                        "Input inversion refuses numeric-reference admission metadata or a raw/DOM identity mismatch; nothing was changed."
                    );
                    for (field, expected) in
                        [(if tag == "Co" { "K" } else { "Type" }, key), ("U", uuid)]
                    {
                        ensure!(
                            attributes
                                .iter()
                                .any(|attribute| attribute.key.as_ref() == field.as_bytes()
                                    && attribute.value.as_ref() == expected.as_bytes()),
                            "Input inversion raw identity differs from admitted identity; nothing was changed."
                        );
                    }
                    found += 1;
                }
                let mut replacements = Vec::new();
                if ordinal == target
                    && let Some(change) = change
                {
                    if change.inverted {
                        replacements.push(("Inv", "true"));
                    } else if let Some(attribute) = attributes
                        .iter()
                        .find(|attribute| attribute.key.as_ref() == b"Inv")
                    {
                        let start = raw_span(data, attribute.key.as_ref())?.start;
                        let end = raw_span(data, attribute.value.as_ref())?.end + 1;
                        patches.push((start..end, String::new()));
                    }
                }
                if ordinal == document
                    && let Some((date, seconds)) = stamp
                {
                    replacements.extend([("Date", date.as_str()), ("DateS", seconds.as_str())]);
                }
                for (key, value) in replacements {
                    if let Some(attribute) = attributes
                        .iter()
                        .find(|attribute| attribute.key.as_ref() == key.as_bytes())
                    {
                        patches
                            .push((raw_span(data, attribute.value.as_ref())?, value.to_string()));
                    } else {
                        let insertion = end - if data[end - 2] == b'/' { 2 } else { 1 };
                        patches.push((insertion..insertion, format!(" {key}=\"{value}\"")));
                    }
                }
                ordinal += 1;
            }
            Event::End(element) => {
                ensure!(
                    open_elements.pop().as_deref() == Some(element.name().as_ref()),
                    DOCUMENT_ERROR
                );
            }
            Event::Text(text) if open_elements.is_empty() => {
                ensure!(
                    text.as_ref()
                        .iter()
                        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n')),
                    DOCUMENT_ERROR
                );
            }
            Event::CData(_) if open_elements.is_empty() => anyhow::bail!(DOCUMENT_ERROR),
            Event::PI(instruction) => ensure!(
                valid_processing_instruction(instruction.as_ref()),
                "Input inversion requires valid XML processing-instruction targets and characters; nothing was changed."
            ),
            Event::Decl(declaration) => {
                ensure!(event_start == 0 && !root_seen, DOCUMENT_ERROR);
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"utf8"),
                        "Input inversion supports UTF-8 XML only; nothing was changed."
                    );
                }
            }
            Event::DocType(_) => anyhow::bail!(
                "Input inversion does not support DTD-dependent storage; nothing was changed."
            ),
            Event::Eof => {
                ensure!(root_seen && open_elements.is_empty(), DOCUMENT_ERROR);
                break;
            }
            _ => {}
        }
    }
    ensure!(
        found == identities.len(),
        "Cannot locate every admitted input-inversion identity in the original XML; nothing was changed."
    );
    patches.sort_by_key(|patch| std::cmp::Reverse(patch.0.start));
    let mut output = data.to_vec();
    for (range, value) in patches {
        output.splice(range, value.bytes());
    }
    Ok(output)
}
