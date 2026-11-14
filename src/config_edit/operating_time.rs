use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, Result, ensure};
use chrono::NaiveDate;
use quick_xml::{Reader, events::Event};
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::{ConfigEditor, alarm_clock::entry_name_ordinal, icons::native_identity};

#[derive(Debug, Serialize)]
pub struct OperatingTimeRangeChange {
    pub uuid: String,
    pub mode_uuid: String,
    pub cal_type: i32,
    pub previous_from: String,
    pub previous_to: String,
    pub from: String,
    pub to: String,
    pub from_index: i32,
    pub to_index: i32,
    pub changed: bool,
}

type RawGuards = BTreeMap<usize, BTreeMap<String, String>>;
type Replacements = BTreeMap<usize, Vec<(&'static str, String)>>;

fn kind(element: &Element) -> &str {
    element.attributes.get("Type").map_or("", String::as_str)
}

fn identity(value: &str) -> Option<uuid::Uuid> {
    (value.len() == 35)
        .then(|| native_identity(value))
        .flatten()
        .filter(|value| !value.is_nil())
}

fn mode_id(element: &Element, key: &str) -> Result<i32> {
    let raw = element.attributes.get(key).map_or("0", String::as_str);
    raw.parse::<i32>()
        .ok()
        .filter(|value| value.to_string() == raw)
        .context("Operating Time requires canonical signed32 mode identifiers; nothing was changed")
}

fn date_index(value: &str) -> Result<i32> {
    ensure!(
        value.len() == 10
            && value.bytes().enumerate().all(|(index, byte)| {
                if matches!(index, 4 | 7) {
                    byte == b'-'
                } else {
                    byte.is_ascii_digit()
                }
            }),
        "Operating Time dates must be YYYY-MM-DD, without a time or timezone; nothing was changed."
    );
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .context("Invalid Operating Time civil date; nothing was changed")?;
    let epoch = NaiveDate::from_ymd_opt(2009, 1, 1).unwrap();
    ensure!(
        date > epoch && date <= NaiveDate::from_ymd_opt(2100, 12, 31).unwrap(),
        "Operating Time dates support 2009-01-02 through 2100-12-31. Native day index zero materializes the current date and is unsupported; nothing was changed."
    );
    Ok((date - epoch).num_days() as i32)
}

fn stored_date(element: &Element, key: &str) -> Result<(i32, String)> {
    let raw = element
        .attributes
        .get(key)
        .with_context(|| format!("Operating Time requires explicit {key}; nothing was changed"))?;
    let index = raw
        .parse::<i32>()
        .ok()
        .filter(|index| *index > 0 && index.to_string() == *raw)
        .context(
            "Operating Time requires positive canonical From/To day indices, not clock-derived zero/default storage; nothing was changed",
        )?;
    let date = NaiveDate::from_ymd_opt(2009, 1, 1)
        .unwrap()
        .checked_add_days(chrono::Days::new(index as u64))
        .context("Operating Time stored date is out of range; nothing was changed")?
        .to_string();
    ensure!(
        date_index(&date)? == index,
        "Operating Time date conversion disagrees with stored index; nothing was changed."
    );
    Ok((index, date))
}

fn valid_stamp(document: &Element) -> bool {
    document.attributes.get("DateS").is_none_or(|value| {
        !value.is_empty()
            && value.bytes().all(|byte| byte.is_ascii_digit())
            && value
                .parse::<i32>()
                .is_ok_and(|value| (0..i32::MAX).contains(&value))
    })
}

fn guard(editor: &ConfigEditor, element: &Element, guards: &mut RawGuards) -> usize {
    let ordinal = entry_name_ordinal(&editor.root, element, &mut 0).unwrap();
    guards.insert(
        ordinal,
        element
            .attributes
            .iter()
            .filter(|(key, _)| {
                matches!(
                    key.as_str(),
                    "Type"
                        | "U"
                        | "K"
                        | "V"
                        | "WF"
                        | "Nio"
                        | "CalType"
                        | "Mode"
                        | "From"
                        | "To"
                        | "DateS"
                )
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );
    ordinal
}

impl ConfigEditor {
    /// Edit only an existing ordinary CalendarEntry's absolute (Mode=3) date span.
    pub fn edit_operating_time_range(
        data: &[u8],
        selector: &str,
        from: &str,
        to: &str,
    ) -> Result<(Vec<u8>, OperatingTimeRangeChange)> {
        let from_index = date_index(from)?;
        let to_index = date_index(to)?;
        ensure!(
            from_index <= to_index,
            "Operating Time supports chronological inclusive spans only; reversed spans are unsupported and nothing was changed."
        );
        let requested = selector.strip_prefix("uuid:").unwrap_or("");
        ensure!(
            identity(requested).is_some(),
            "Operating Time requires uuid:<exact nonnil native35 U> (8-4-4-16); nothing was changed."
        );
        range_bytes(data, &RawGuards::new(), &Replacements::new())?;
        let mut editor =
            Self::load(data).context("Cannot admit Operating Time XML; nothing was changed")?;
        let all = Self::iter_elements(&editor.root);
        for (sanitized, _) in &editor.digit_attr_renames {
            let pattern = format!(" {sanitized}=");
            ensure!(
                !all.iter().any(|element| {
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
                }),
                "Operating Time admission is affected by shared-loader digit-like text (lox-n37w); nothing was changed."
            );
        }
        ensure!(
            all.iter().all(|element| {
                element.namespace.is_none()
                    && element.prefix.is_none()
                    && element.name != "Template"
                    && kind(element) != "DocumentTemplate"
                    && !element.attributes.contains_key("IfT")
                    && !element.attributes.contains_key("uuidParent")
            }),
            "Operating Time does not support namespaces or Template/DocumentTemplate/IfT/uuidParent state; nothing was changed."
        );
        let documents = editor.find_elements("Type:Document");
        let captions = editor.find_elements("Type:CalendarCaption");
        let mode_captions = editor.find_elements("Type:ModeCaption");
        let owners = editor.find_elements(selector);
        ensure!(
            documents.len() == 1
                && documents[0].len() <= 1
                && captions.len() == 1
                && captions[0].len() == documents[0].len() + 1
                && captions[0].starts_with(&documents[0])
                && mode_captions.len() == 1
                && mode_captions[0].len() == documents[0].len() + 1
                && mode_captions[0].starts_with(&documents[0])
                && owners.len() == 1
                && owners[0].len() == captions[0].len() + 1
                && owners[0].starts_with(&captions[0]),
            "Operating Time requires one root/direct Document with direct CalendarCaption and ModeCaption, and an exact direct CalendarEntry; nothing was changed."
        );
        let document = editor.get_element(&documents[0]);
        let caption = editor.get_element(&captions[0]);
        let mode_caption = editor.get_element(&mode_captions[0]);
        let owner = editor.get_element(&owners[0]);
        ensure!(
            kind(owner) == "CalendarEntry"
                && owner.attributes.get("U").map(String::as_str) == Some(requested)
                && owner.attributes.get("Mode").map(String::as_str) == Some("3")
                && owner.attributes.get("Nio").map(String::as_str) == Some("2")
                && !owner
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .any(|element| element.name == "C"),
            "Operating Time supports only an existing CalendarEntry with Mode=3, Nio=2 and no child controls; annual/other recurrence, Calendar indicators and creation are unsupported. Nothing was changed."
        );
        ensure!(
            valid_stamp(document),
            "Operating Time requires absent or nonnegative ASCII signed32 Document DateS below MAX; nothing was changed."
        );
        let (old_from, previous_from) = stored_date(owner, "From")?;
        let (old_to, previous_to) = stored_date(owner, "To")?;
        ensure!(
            old_from <= old_to,
            "Operating Time cannot admit a reversed existing span; nothing was changed."
        );
        let cal_type = mode_id(owner, "CalType")?;
        let modes: Vec<_> = mode_caption
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "C")
            .collect();
        ensure!(
            modes.len() == editor.find_elements("Type:Mode").len()
                && modes.iter().all(|element| kind(element) == "Mode"),
            "Operating Time requires all global Modes directly under the one ModeCaption; nothing was changed."
        );
        let mut mode_ids = HashSet::new();
        let mut selected_mode = None;
        let mut guards = RawGuards::new();
        for mode in &modes {
            let id = mode_id(mode, "Mode")?;
            ensure!(
                mode_ids.insert(id),
                "Operating Time requires unambiguous global Mode numbers; nothing was changed."
            );
            if id == cal_type {
                selected_mode = Some(*mode);
            }
            guard(&editor, mode, &mut guards);
        }
        let selected_mode = selected_mode.context(
            "Operating Time CalType must resolve to one existing global Mode number, not a UUID or weekday mask; nothing was changed",
        )?;
        let connectors: Vec<_> = owner
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "Co")
            .collect();
        ensure!(
            connectors.len() == 2
                && connectors[0].attributes.get("K").map(String::as_str) == Some("I")
                && connectors[1].attributes.get("K").map(String::as_str) == Some("Q"),
            "Operating Time requires the existing native I/Q connector order; nothing was changed."
        );
        for element in [document, caption, mode_caption, owner, selected_mode] {
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
                        .is_none_or(|value| { matches!(value.as_str(), "0" | "16384") }),
                "Operating Time requires modern V>=90 and ordinary absent/0/16384 WF on the selected owner, ancestors and associated Mode; template/forwarding/excluded flags and device bindings are unsupported. Nothing was changed."
            );
        }
        for element in [document, caption, mode_caption, owner, selected_mode]
            .into_iter()
            .chain(connectors.iter().copied())
        {
            let id = element
                .attributes
                .get("U")
                .and_then(|value| identity(value))
                .context("Operating Time requires nonnil native35 selected identities; nothing was changed")?;
            ensure!(
                all.iter()
                    .filter(|candidate| matches!(candidate.name.as_str(), "C" | "Co"))
                    .filter(|candidate| candidate
                        .attributes
                        .get("U")
                        .and_then(|value| native_identity(value))
                        == Some(id))
                    .count()
                    == 1,
                "Operating Time requires unique effective selected identities, including case/RFC-grouping aliases; nothing was changed."
            );
            guard(&editor, element, &mut guards);
        }
        let target = entry_name_ordinal(&editor.root, owner, &mut 0).unwrap();
        let doc = entry_name_ordinal(&editor.root, document, &mut 0).unwrap();
        let change = OperatingTimeRangeChange {
            uuid: requested.to_owned(),
            mode_uuid: selected_mode.attributes["U"].clone(),
            cal_type,
            previous_from,
            previous_to,
            from: from.to_owned(),
            to: to.to_owned(),
            from_index,
            to_index,
            changed: (old_from, old_to) != (from_index, to_index),
        };
        let admitted = range_bytes(data, &guards, &Replacements::new())?;
        if !change.changed {
            return Ok((admitted, change));
        }
        ensure!(
            editor.stamp_document(&chrono::Local::now().fixed_offset()),
            "Cannot stamp the admitted Operating Time Document; nothing was changed."
        );
        let document = editor.get_element(&documents[0]);
        ensure!(
            valid_stamp(document),
            "Operating Time Document DateS cannot advance below signed32 MAX; nothing was changed."
        );
        let replacements = BTreeMap::from([
            (
                target,
                vec![
                    ("From", from_index.to_string()),
                    ("To", to_index.to_string()),
                    ("WF", "16384".to_owned()),
                ],
            ),
            (
                doc,
                vec![
                    ("Date", document.attributes["Date"].clone()),
                    ("DateS", document.attributes["DateS"].clone()),
                ],
            ),
        ]);
        Ok((range_bytes(data, &guards, &replacements)?, change))
    }
}

fn raw_span(data: &[u8], value: &[u8]) -> Result<std::ops::Range<usize>> {
    let start = (value.as_ptr() as usize)
        .checked_sub(data.as_ptr() as usize)
        .context("Cannot locate original Operating Time XML storage; nothing was changed")?;
    let end = start
        .checked_add(value.len())
        .context("Operating Time XML storage overflow")?;
    ensure!(
        data.get(start..end) == Some(value),
        "Operating Time storage must borrow original XML; nothing was changed."
    );
    Ok(start..end)
}

fn range_bytes(data: &[u8], guards: &RawGuards, replacements: &Replacements) -> Result<Vec<u8>> {
    const XML_ERROR: &str = "Operating Time requires balanced single-root XML without trailing content; nothing was changed.";
    std::str::from_utf8(data)?;
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    reader.config_mut().check_comments = true;
    let mut open = Vec::new();
    let mut root_seen = false;
    let mut ordinal = 0;
    let mut found = 0;
    let mut patches = Vec::new();
    loop {
        let event_start = reader.buffer_position();
        let event = reader.read_event().context(XML_ERROR)?;
        let is_start = matches!(&event, Event::Start(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                if open.is_empty() {
                    ensure!(!root_seen, XML_ERROR);
                    root_seen = true;
                }
                if is_start {
                    open.push(element.name().as_ref().to_vec());
                }
                let name = element.name();
                let expected = guards.get(&ordinal);
                let mut seen = BTreeMap::new();
                let mut attributes = Vec::new();
                for attribute in element.attributes().with_checks(false) {
                    let attribute = attribute?;
                    let key = std::str::from_utf8(attribute.key.as_ref())?;
                    let global_identity = (matches!(name.as_ref(), b"C" | b"Co" | b"Ref")
                        && key == "U")
                        || (name.as_ref() == b"C" && key == "Type")
                        || (name.as_ref() == b"Co" && key == "K");
                    if let Some(previous) = seen.insert(key.to_owned(), attribute.value.to_vec()) {
                        ensure!(
                            expected.is_none()
                                && !global_identity
                                && previous == attribute.value.as_ref(),
                            "Operating Time refuses guarded/identity duplicates or conflicting raw attributes; nothing was changed."
                        );
                        continue;
                    }
                    raw_span(data, attribute.key.as_ref())?;
                    raw_span(data, attribute.value.as_ref())?;
                    attributes.push(attribute);
                }
                ensure!(
                    !name.as_ref().contains(&b':')
                        && attributes.iter().all(|attribute| {
                            attribute.key.as_ref() != b"xmlns"
                                && !attribute.key.as_ref().contains(&b':')
                        }),
                    "Operating Time does not support XML namespaces; nothing was changed."
                );
                if let Some(expected) = expected {
                    for (key, value) in expected {
                        let attribute = attributes
                            .iter()
                            .find(|attribute| attribute.key.as_ref() == key.as_bytes())
                            .context(
                                "Operating Time raw/DOM attribute is missing; nothing was changed",
                            )?;
                        ensure!(
                            !attribute.value.windows(2).any(|bytes| bytes == b"&#")
                                && attribute.unescape_value()?.as_ref() == value,
                            "Operating Time refuses numeric-reference admission metadata or raw/DOM disagreement; nothing was changed."
                        );
                    }
                    found += 1;
                }
                if let Some(replacements) = replacements.get(&ordinal) {
                    let end = reader.buffer_position() as usize + offset;
                    for (key, value) in replacements {
                        let escaped = quick_xml::escape::escape(value).into_owned();
                        if let Some(attribute) = attributes
                            .iter()
                            .find(|attribute| attribute.key.as_ref() == key.as_bytes())
                        {
                            patches.push((raw_span(data, attribute.value.as_ref())?, escaped));
                        } else {
                            let at = end - if data[end - 2] == b'/' { 2 } else { 1 };
                            patches.push((at..at, format!(" {key}=\"{escaped}\"")));
                        }
                    }
                }
                ordinal += 1;
            }
            Event::End(element) => {
                ensure!(
                    open.pop().as_deref() == Some(element.name().as_ref()),
                    XML_ERROR
                );
            }
            Event::Text(text) if open.is_empty() => {
                ensure!(
                    text.as_ref()
                        .iter()
                        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n')),
                    XML_ERROR
                );
            }
            Event::CData(_) if open.is_empty() => anyhow::bail!(XML_ERROR),
            Event::Decl(declaration) => {
                ensure!(event_start == 0 && !root_seen, XML_ERROR);
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"utf8"),
                        "Operating Time supports UTF-8 XML only; nothing was changed."
                    );
                }
            }
            Event::DocType(_) => anyhow::bail!(
                "Operating Time does not support DTD-dependent storage; nothing was changed."
            ),
            Event::Eof => {
                ensure!(root_seen && open.is_empty(), XML_ERROR);
                break;
            }
            _ => {}
        }
    }
    ensure!(
        found == guards.len(),
        "Cannot locate all admitted Operating Time metadata in original XML; nothing was changed."
    );
    patches.sort_by_key(|patch| std::cmp::Reverse(patch.0.start));
    let mut output = data.to_vec();
    for (range, value) in patches {
        output.splice(range, value.bytes());
    }
    Ok(output)
}
