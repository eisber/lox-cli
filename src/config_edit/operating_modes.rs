use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::{ConfigEditor, icons::native_identity};

#[derive(Debug, Serialize)]
pub struct OperatingModeAdded {
    pub uuid: String,
    pub mode: i32,
    pub title: String,
    pub input_uuid: String,
    pub output_uuid: String,
}

fn stored_identity(raw: &str) -> Option<uuid::Uuid> {
    (raw.len() == 35)
        .then(|| native_identity(raw))
        .flatten()
        .filter(|identity| !identity.is_nil())
}

impl ConfigEditor {
    /// Append a custom global mode to an ordinary document's existing mode collection.
    pub fn add_operating_mode(data: &[u8], title: &str) -> Result<(Vec<u8>, OperatingModeAdded)> {
        use quick_xml::{Reader, events::Event};

        let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
        let mut reader = Reader::from_reader(&data[offset..]);
        let mut depth = 0usize;
        let mut caption_depth = None;
        let mut caption_end = None;
        let mut document_end = None;
        let mut document_attributes = Vec::new();
        loop {
            let start = reader.buffer_position() as usize + offset;
            match reader.read_event()? {
                Event::Start(element) | Event::Empty(element) => {
                    let end = reader.buffer_position() as usize + offset;
                    let empty = data[end - 2] == b'/';
                    if element.name().as_ref() == b"C" {
                        let attributes = element
                            .attributes()
                            .with_checks(false)
                            .collect::<std::result::Result<Vec<_>, _>>()?;
                        let kind = attributes
                            .iter()
                            .find(|attribute| attribute.key.as_ref() == b"Type")
                            .map(|attribute| attribute.unescape_value())
                            .transpose()?;
                        if kind.as_deref() == Some("ModeCaption") {
                            caption_depth = Some(depth + 1);
                        }
                        if kind.as_deref() == Some("Document") {
                            document_end = Some(end - if empty { 2 } else { 1 });
                        }
                        for attribute in &attributes {
                            let key = attribute.key.as_ref();
                            let selected = matches!(
                                kind.as_deref(),
                                Some("Document" | "ModeCaption" | "Mode")
                            );
                            if (key == b"Type"
                                || selected
                                    && matches!(
                                        key,
                                        b"U" | b"Title" | b"Mode" | b"NumO" | b"WF" | b"DateS"
                                    ))
                                && (attribute.value.windows(2).any(|bytes| bytes == b"&#")
                                    || attribute
                                        .value
                                        .iter()
                                        .any(|byte| matches!(byte, b'\t' | b'\r' | b'\n')))
                            {
                                bail!(
                                    "Operating mode identity/name metadata requires literal native storage, without numeric XML references or literal tab/newline normalization; nothing was changed."
                                );
                            }
                            if kind.as_deref() == Some("Document")
                                && matches!(key, b"Date" | b"DateS" | b"NumO")
                            {
                                let position = (attribute.value.as_ptr() as usize).checked_sub(data.as_ptr() as usize)
                                    .filter(|position| data.get(*position..*position + attribute.value.len()) == Some(attribute.value.as_ref()))
                                    .ok_or_else(|| anyhow::anyhow!("Cannot locate native Document count/stamp storage; nothing was changed."))?;
                                document_attributes.push((
                                    String::from_utf8_lossy(key).into_owned(),
                                    position..position + attribute.value.len(),
                                ));
                            }
                        }
                    }
                    if !empty {
                        depth += 1;
                    }
                }
                Event::End(_) => {
                    if caption_depth == Some(depth) {
                        caption_end = Some(start);
                        caption_depth = None;
                    }
                    depth = depth.saturating_sub(1);
                }
                Event::Eof => break,
                _ => {}
            }
        }
        let mut editor = Self::load(data)?;
        let added = editor.append_operating_mode(title)?;
        if !editor.stamp_document(&chrono::Local::now().fixed_offset()) {
            bail!("Cannot stamp the admitted operating-mode Document; nothing was changed.");
        }
        let document = editor.get_element(&editor.find_elements("Type:Document")[0]);
        let mut patches = Vec::new();
        for key in ["Date", "DateS", "NumO"] {
            let value = document.attributes[key].clone();
            let matching: Vec<_> = document_attributes
                .iter()
                .filter(|(name, _)| name == key)
                .collect();
            if matching.is_empty() {
                let position = document_end.ok_or_else(|| {
                    anyhow::anyhow!("Cannot locate Document storage; nothing was changed.")
                })?;
                patches.push((position..position, format!(" {key}=\"{value}\"")));
            } else {
                for (_, range) in matching {
                    patches.push((range.clone(), value.clone()));
                }
            }
        }
        let caption = editor.get_element(&editor.find_elements("Type:ModeCaption")[0]);
        let mode = caption
            .children
            .last()
            .and_then(XMLNode::as_element)
            .unwrap();
        let mut fragment = Vec::new();
        mode.write_with_config(
            &mut fragment,
            xmltree::EmitterConfig::new()
                .write_document_declaration(false)
                .pad_self_closing(false),
        )?;
        let position = caption_end.ok_or_else(|| {
            anyhow::anyhow!("Cannot locate ModeCaption storage; nothing was changed.")
        })?;
        patches.push((position..position, String::from_utf8(fragment)?));
        patches.sort_by_key(|patch| std::cmp::Reverse(patch.0.start));
        let mut output = data.to_vec();
        for (range, value) in patches {
            output.splice(range, value.bytes());
        }
        Ok((output, added))
    }

    fn append_operating_mode(&mut self, title: &str) -> Result<OperatingModeAdded> {
        if title.trim().is_empty()
            || title.chars().any(|c| {
                c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')
            })
            || !Self::sanitize_digit_attrs(title.as_bytes()).1.is_empty()
        {
            bail!(
                "Operating mode title must be nonblank single-line XML text unaffected by the shared digit-attribute loader; nothing was changed."
            );
        }

        let all = Self::iter_elements(&self.root);
        for (sanitized, _) in &self.digit_attr_renames {
            let pattern = format!(" {sanitized}=");
            if title.contains(&pattern)
                || all.iter().any(|element| {
                    element
                        .attributes
                        .values()
                        .any(|value| value.contains(&pattern))
                        || element.children.iter().any(|node| match node {
                            XMLNode::Text(value) | XMLNode::CData(value) => {
                                value.contains(&pattern)
                            }
                            _ => false,
                        })
                })
            {
                bail!(
                    "Operating mode admission is affected by shared loader issue lox-n37w (digit-like text); nothing was changed."
                );
            }
        }
        let documents = self.find_elements("Type:Document");
        let captions = self.find_elements("Type:ModeCaption");
        if documents.len() != 1
            || documents[0].len() > 1
            || captions.len() != 1
            || captions[0].len() != documents[0].len() + 1
            || !captions[0].starts_with(&documents[0])
        {
            bail!(
                "Adding an operating mode requires one ordinary Document and one direct ModeCaption child; nothing was changed."
            );
        }
        let document = self.get_element(&documents[0]);
        let caption = self.get_element(&captions[0]);
        let count = document
            .attributes
            .get("NumO")
            .and_then(|value| value.parse::<i32>().ok())
            .filter(|count| *count > 0)
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "A positive, incrementable Document NumO is required; nothing was changed."
                )
            })?;
        if all.iter().any(|element| {
            element.namespace.is_some()
                || element.prefix.is_some()
                || element.name == "Template"
                || element
                    .attributes
                    .get("Type")
                    .is_some_and(|kind| kind == "DocumentTemplate")
                || element.attributes.contains_key("IfT")
                || element.attributes.contains_key("uuidParent")
        }) {
            bail!(
                "Operating mode creation does not support namespaces, templates, IfT or uuidParent instance state; nothing was changed."
            );
        }
        for owner in [document, caption] {
            if owner.name != "C"
                || owner
                    .attributes
                    .get("U")
                    .and_then(|raw| stored_identity(raw))
                    .is_none()
                || owner.attributes.get("WF").is_some_and(|value| {
                    value
                        .parse::<u32>()
                        .map_or(true, |flags| flags & 0x20002000 != 0)
                })
            {
                bail!(
                    "Operating mode owners require unique native35 UUIDs and ordinary, non-template/non-excluded flags; nothing was changed."
                );
            }
        }
        let modes: Vec<_> = caption
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "C")
            .collect();
        if modes.len() != self.find_elements("Type:Mode").len()
            || modes
                .iter()
                .any(|element| element.attributes.get("Type").map(String::as_str) != Some("Mode"))
        {
            bail!(
                "Every global Mode must be a direct child of the single ModeCaption, without other C children; nothing was changed."
            );
        }
        let mut ids = HashSet::new();
        for mode in &modes {
            let raw = mode
                .attributes
                .get("Mode")
                .map(String::as_str)
                .unwrap_or("0");
            let id = raw.parse::<i32>().ok().filter(|id| id.to_string() == raw)
                .ok_or_else(|| anyhow::anyhow!("Mode identifiers must be canonical signed 32-bit integers (only zero may be omitted); nothing was changed."))?;
            if !ids.insert(id) {
                bail!("Duplicate global Mode identifier {id}; nothing was changed.");
            }
            if mode
                .attributes
                .get("Title")
                .is_some_and(|name| name.to_lowercase() == title.to_lowercase())
            {
                bail!("An operating mode named '{title}' already exists; nothing was changed.");
            }
        }
        if (0..=9).any(|id| !ids.contains(&id)) {
            bail!(
                "Built-in modes 0 through 9 must already exist: native loading restores missing defaults. Partial mode collections are not supported; nothing was changed."
            );
        }
        if (-6..=-1).any(|id| !ids.contains(&id)) {
            bail!(
                "Built-in modes -1 through -6 must already exist: native allocation would otherwise fill a protected built-in ID, not add a custom mode. Nothing was changed."
            );
        }
        for selected in std::iter::once(document)
            .chain(std::iter::once(caption))
            .chain(modes.iter().copied())
        {
            let identity = selected.attributes.get("U").and_then(|raw| stored_identity(raw))
                .ok_or_else(|| anyhow::anyhow!("Mode collection objects require nonnil native35 UUIDs, not RFC36 aliases; nothing was changed."))?;
            if all
                .iter()
                .filter(|element| matches!(element.name.as_str(), "C" | "Co"))
                .filter(|element| {
                    element
                        .attributes
                        .get("U")
                        .and_then(|raw| native_identity(raw))
                        == Some(identity)
                })
                .count()
                != 1
            {
                bail!(
                    "Mode collection identities must be unique, including case/grouping aliases; nothing was changed."
                );
            }
        }
        let mut id = -1i32;
        while ids.contains(&id) {
            id = id.checked_sub(1).ok_or_else(|| {
                anyhow::anyhow!("No free negative Mode identifier; nothing was changed.")
            })?;
        }
        let serial = caption.attributes["U"].rsplit('-').next().unwrap();
        let serial = &serial[4..];
        let mut used: HashSet<_> = all
            .iter()
            .filter_map(|element| {
                element
                    .attributes
                    .get("U")
                    .and_then(|raw| native_identity(raw))
            })
            .collect();
        let mut fresh = |connector: Option<u8>| -> Result<String> {
            for _ in 0..32 {
                let mut value = Self::loxone_uuid(serial);
                if let Some(index) = connector {
                    value.replace_range(19..21, &format!("{index:02x}"));
                }
                if let Some(identity) = stored_identity(&value)
                    && used.insert(identity)
                {
                    return Ok(value);
                }
            }
            bail!("Could not allocate a unique native mode UUID; nothing was changed.")
        };
        let change = OperatingModeAdded {
            uuid: fresh(None)?,
            mode: id,
            title: title.into(),
            input_uuid: fresh(Some(0))?,
            output_uuid: fresh(Some(1))?,
        };
        let mut mode = Element::new("C");
        for (key, value) in [
            ("Type", "Mode".to_owned()),
            ("V", "178".to_owned()),
            ("U", change.uuid.clone()),
            ("Title", title.into()),
            ("Nio", "2".to_owned()),
            ("WF", "16384".to_owned()),
            ("Mode", id.to_string()),
        ] {
            mode.attributes.insert(key.into(), value);
        }
        for (key, identity) in [("I", &change.input_uuid), ("Q", &change.output_uuid)] {
            let mut connector = Element::new("Co");
            connector.attributes.insert("K".into(), key.into());
            connector.attributes.insert("U".into(), identity.clone());
            mode.children.push(XMLNode::Element(connector));
        }
        mode.children.push(XMLNode::Element(Element::new("IoData")));
        let mut display = Element::new("Display");
        display.attributes.insert("StateOnly".into(), "true".into());
        mode.children.push(XMLNode::Element(display));
        self.get_element_mut(&captions[0])
            .children
            .push(XMLNode::Element(mode));
        self.get_element_mut(&documents[0])
            .attributes
            .insert("NumO".into(), count.to_string());
        Ok(change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(extra: &str) -> String {
        let mut modes: String = (1..=6).map(|id| format!(
            "<C Type=\"Mode\" U=\"00000000-0000-{id:04x}-ffff112233445566\" Title=\"Builtin{id}\" Mode=\"-{id}\"/>"
        )).collect();
        for id in 0..=9 {
            let identity = id + 1;
            let mode = if id == 0 {
                String::new()
            } else {
                format!(" Mode=\"{id}\"")
            };
            modes.push_str(&format!("<C Type=\"Mode\" U=\"00000000-0000-{identity:04x}-1500000000000000\" Title=\"Day{id}\"{mode}/>"));
        }
        let count = if extra.contains("<C ") { 19 } else { 18 };
        format!(
            "<ControlList><C Type=\"Document\" U=\"11111111-2222-3333-ffff112233445566\" NumO=\"{count}\" Keep=\"original\"><C Type=\"ModeCaption\" U=\"22222222-2222-3333-ffff112233445566\">{modes}{extra}</C></C></ControlList>"
        )
    }

    #[test]
    fn operating_mode_add_allocates_gap_and_preserves_existing_state() {
        let input = fixture(
            "<C Type=\"Mode\" U=\"33333333-2222-3333-ffff112233445566\" Title=\"Later\" Mode=\"-8\"><Unknown value=\"kept\"/></C>",
        );
        let mut editor = ConfigEditor::load(input.as_bytes()).unwrap();
        let old_caption = editor
            .get_element(&editor.find_elements("Type:ModeCaption")[0])
            .clone();
        let added = editor.append_operating_mode("Evening & Guests").unwrap();
        assert_eq!(added.mode, -7);
        assert!(!stored_identity(&added.uuid).unwrap().is_nil());
        let after = ConfigEditor::load(&editor.to_bytes().unwrap()).unwrap();
        let caption = after.get_element(&after.find_elements("Type:ModeCaption")[0]);
        assert_eq!(
            &caption.children[..old_caption.children.len()],
            old_caption.children.as_slice()
        );
        let mode = caption.children.last().unwrap().as_element().unwrap();
        assert_eq!(mode.attributes["Mode"], "-7");
        assert_eq!(mode.attributes["Nio"], "2");
        assert!(!mode.attributes.contains_key("Px"));
        let children: Vec<_> = mode
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .collect();
        assert_eq!(
            children
                .iter()
                .map(|child| child.name.as_str())
                .collect::<Vec<_>>(),
            ["Co", "Co", "IoData", "Display"]
        );
        assert_eq!(children[0].attributes["K"], "I");
        assert_eq!(children[1].attributes["K"], "Q");
        assert_eq!(children[3].attributes["StateOnly"], "true");
        assert!(!children[3].attributes.contains_key("Unit"));
        let document = after.get_element(&after.find_elements("Type:Document")[0]);
        assert_eq!(document.attributes["Keep"], "original");
        assert_eq!(document.attributes["NumO"], "20");
        assert_eq!(editor.append_operating_mode("Second").unwrap().mode, -9);
    }

    #[test]
    fn operating_mode_add_preserves_unrelated_native_storage() {
        let existing = "<C Type='Mode' U='33333333-2222-3333-ffff112233445566' Title='Later' Mode='-8'><Unknown value='&#65;' Keep='a &amp; b'/><!-- unchanged --></C>";
        let input = format!(
            "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n{}",
            fixture(existing)
        );
        let (output, added) =
            ConfigEditor::add_operating_mode(input.as_bytes(), "Evening & Guests").unwrap();
        assert!(output.starts_with(b"\xef\xbb\xbf"));
        let output_text = std::str::from_utf8(&output).unwrap();
        assert!(output_text.contains(existing));
        assert!(output_text.contains("?>\r\n"));
        let editor = ConfigEditor::load(&output).unwrap();
        let mode = editor.get_element(&editor.find_elements(&format!("uuid:{}", added.uuid))[0]);
        assert_eq!(mode.attributes["Title"], "Evening & Guests");
        let document = editor.get_element(&editor.find_elements("Type:Document")[0]);
        assert_eq!(document.attributes["NumO"], "20");
        assert!(document.attributes.contains_key("DateS"));
    }

    #[test]
    fn operating_mode_add_refuses_ambiguous_protected_and_instance_state_atomically() {
        let base = fixture("");
        let bad_inputs = [
            base.replace("Mode=\"-6\"", "Mode=\"-5\""),
            base.replace("Mode=\"-6\"", "Mode=\"-7\""),
            base.replace("Mode=\"9\"", "Mode=\"42\""),
            base.replace("Mode=\"-6\"", "Mode=\"-2147483649\""),
            base.replace("00000000-0000-0006-", "00000000-0000-0005-"),
            base.replace(
                "00000000-0000-0006-ffff112233445566",
                "00000000-0000-0006-ffff-112233445566",
            ),
            base.replace(
                "00000000-0000-0006-ffff112233445566",
                "00000000-0000-0000-0000000000000000",
            ),
            base.replace("NumO=\"18\"", "NumO=\"2147483647\""),
            base.replace("NumO=\"18\"", "NumO=\"18\" IfT=\"false\""),
            base.replace(
                "<C Type=\"ModeCaption\"",
                "<Template/><C Type=\"ModeCaption\"",
            ),
            base.replace("Title=\"Builtin6\"", "Title=\"evening\""),
            base.replace(
                "</ControlList>",
                "<C Type=\"Other\" U=\"00000000-0000-0006-ffff-112233445566\"/></ControlList>",
            ),
        ];
        for input in bad_inputs {
            let mut editor = ConfigEditor::load(input.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            assert!(editor.append_operating_mode("Evening").is_err(), "{input}");
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }

    #[test]
    fn operating_mode_add_refuses_shared_loader_text_collision() {
        let input = fixture("").replace("Keep=\"original\"", "Keep=\"Wake 7am=ok\" 12hTF=\"true\"");
        let mut editor = ConfigEditor::load(input.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        assert!(editor.append_operating_mode("Evening").is_err());
        assert_eq!(editor.to_bytes().unwrap(), before);
        let mut editor = ConfigEditor::load(fixture("").as_bytes()).unwrap();
        assert!(editor.append_operating_mode("Wake 7am=ok").is_err());
        let input = fixture("").replace("Title=\"Builtin6\"", "Title=\"&#69;vening\"");
        assert!(ConfigEditor::add_operating_mode(input.as_bytes(), "Evening").is_err());
        let input = fixture("").replace("00000000-0000-0006-", "&#48;0000000-0000-0006-");
        assert!(ConfigEditor::add_operating_mode(input.as_bytes(), "Evening").is_err());
    }
}
