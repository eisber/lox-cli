use std::collections::{BTreeMap, HashSet};

use anyhow::{Result, bail};
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::ConfigEditor;

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryRemoval {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub stored_entry_uuid: String,
    pub removed_attributes: BTreeMap<String, String>,
    pub remaining_entries: usize,
}

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryActivation {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub stored_entry_uuid: String,
    pub previous_active: bool,
    pub active: bool,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryModeChange {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub stored_entry_uuid: String,
    pub mode_bit: u8,
    pub previous_enabled: bool,
    pub enabled: bool,
    pub previous_modes: u32,
    pub modes: u32,
    pub active: bool,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryTimeChange {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub stored_entry_uuid: String,
    pub previous_time_seconds: u32,
    pub time_seconds: u32,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryNameChange {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub stored_entry_uuid: String,
    pub previous_name: Option<String>,
    pub name: String,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryAddition {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub id: u32,
    pub name: String,
    pub time_seconds: u32,
    pub active: bool,
    pub modes: u32,
    pub total_entries: usize,
}

fn uuid_key(raw: &str) -> Result<String> {
    let dashes: &[usize] = match raw.len() {
        35 => &[8, 13, 18],
        36 => &[8, 13, 18, 23],
        _ => bail!("Alarm entry UUID must use native 8-4-4-16 or RFC 8-4-4-4-12 grouping."),
    };
    let mut key = String::with_capacity(32);
    for (index, byte) in raw.bytes().enumerate() {
        if dashes.contains(&index) {
            if byte != b'-' {
                bail!("Invalid alarm entry UUID grouping.");
            }
        } else if byte.is_ascii_hexdigit() {
            key.push(char::from(byte.to_ascii_lowercase()));
        } else {
            bail!("Alarm entry UUID must contain exactly 32 ASCII hex digits.");
        }
    }
    if key.bytes().all(|byte| byte == b'0') {
        bail!("A nil alarm entry UUID is not supported.");
    }
    Ok(key)
}

fn integer(element: &Element, key: &str) -> Result<u32> {
    let Some(raw) = element.attributes.get(key) else {
        return Ok(0);
    };
    let digits = raw.strip_prefix('+').unwrap_or(raw);
    if !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(value) = digits.parse::<i32>()
        && value >= 0
    {
        return Ok(value as u32);
    }
    bail!(
        "Unsupported AlarmClock {key}='{raw}': expected a nonnegative signed-32-bit integer; nothing was changed."
    );
}

fn flag(element: &Element, key: &str) -> Result<bool> {
    match element.attributes.get(key) {
        None => Ok(false),
        Some(raw) if raw.eq_ignore_ascii_case("false") => Ok(false),
        Some(raw) if raw.eq_ignore_ascii_case("true") => Ok(true),
        Some(raw) => bail!(
            "Unsupported AlarmClock {key}='{raw}': use true/false or absence; nothing was changed."
        ),
    }
}

fn time_seconds(time: &str) -> Result<u32> {
    let parts: Vec<_> = time.split(':').collect();
    if !(2..=3).contains(&parts.len())
        || parts
            .iter()
            .any(|part| part.len() != 2 || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        bail!("Alarm time must be HH:MM or HH:MM:SS within one day; nothing was changed.");
    }
    let parts: Vec<u32> = parts.iter().map(|part| part.parse().unwrap()).collect();
    if parts[0] >= 24 || parts[1] >= 60 || parts.get(2).is_some_and(|second| *second >= 60) {
        bail!("Alarm time must be HH:MM or HH:MM:SS within one day; nothing was changed.");
    }
    Ok(parts[0] * 3600 + parts[1] * 60 + parts.get(2).copied().unwrap_or(0))
}

pub(super) fn entry_name_ordinal(
    element: &Element,
    target: &Element,
    ordinal: &mut usize,
) -> Option<usize> {
    if std::ptr::eq(element, target) {
        return Some(*ordinal);
    }
    *ordinal += 1;
    element
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .find_map(|child| entry_name_ordinal(child, target, ordinal))
}

pub(super) enum EntryNameStorage<'a> {
    Alarm {
        entry_uuid: &'a str,
        block_uuid: &'a str,
    },
    Irrigation {
        io_index: Option<&'a str>,
        block_uuid: &'a str,
    },
}

pub(super) fn entry_name_bytes(
    data: &[u8],
    target: usize,
    owner: usize,
    identity: EntryNameStorage<'_>,
    name: Option<&str>,
    stamp: Option<(&str, &str)>,
) -> Result<(Vec<u8>, Option<String>)> {
    use quick_xml::{Reader, events::Event};

    let clear_irrigation_name =
        matches!(&identity, EntryNameStorage::Irrigation { .. }) && name == Some("");
    let (owner_type, entry_tag, identity_key, identity_value, name_key, owner_uuid) = match identity
    {
        EntryNameStorage::Alarm {
            entry_uuid,
            block_uuid,
        } => (
            "AlarmClock",
            "AlarmClock",
            "U",
            Some(entry_uuid),
            "Name",
            block_uuid,
        ),
        EntryNameStorage::Irrigation {
            io_index,
            block_uuid,
        } => ("Irrigation", "zone", "ioIdx", io_index, "nm", block_uuid),
    };
    std::str::from_utf8(data).map_err(|_| {
        anyhow::anyhow!(
            "Unsupported target-name-storage: UTF-8 XML is required; nothing was changed."
        )
    })?;
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    let mut patches = Vec::new();
    let mut ordinal = 0;
    let mut depth = 0usize;
    let mut found = false;
    let mut document_seen = false;
    let mut previous_name = None;
    loop {
        match reader.read_event()? {
            Event::Start(element) | Event::Empty(element) => {
                let end = reader.buffer_position() as usize + offset;
                let empty = data[end - 2] == b'/';
                let selected = ordinal == target;
                let selected_owner = ordinal == owner;
                let attributes = element
                    .attributes()
                    .with_checks(false)
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                let document = !document_seen
                    && depth <= 1
                    && element.local_name().as_ref() == b"C"
                    && attributes.iter().any(|attribute| {
                        attribute.key.local_name().as_ref() == b"Type"
                            && attribute
                                .unescape_value()
                                .is_ok_and(|value| value == "Document")
                    });
                for attribute in &attributes {
                    let key = attribute.key.as_ref();
                    if ((selected && key == identity_key.as_bytes())
                        || (selected_owner && matches!(key, b"Type" | b"U"))
                        || (document && matches!(key, b"Type" | b"DateS")))
                        && attribute.value.windows(2).any(|bytes| bytes == b"&#")
                    {
                        bail!(
                            "Unsupported target-name-storage: numeric XML references in selected/owner identity or Document stamp metadata are not qualified; nothing was changed."
                        );
                    }
                }
                if selected_owner {
                    let raw_type = attributes
                        .iter()
                        .find(|attribute| attribute.key.as_ref() == b"Type")
                        .map(|attribute| attribute.unescape_value())
                        .transpose()?;
                    let raw_uuid = attributes
                        .iter()
                        .find(|attribute| attribute.key.as_ref() == b"U")
                        .map(|attribute| attribute.unescape_value())
                        .transpose()?;
                    if element.name().as_ref() != b"C"
                        || raw_type.as_deref() != Some(owner_type)
                        || raw_uuid.as_deref().unwrap_or("") != owner_uuid
                    {
                        bail!(
                            "Unsupported target-name-storage: raw owner identity differs from admitted identity; nothing was changed."
                        );
                    }
                }
                if (selected || document)
                    && (element.name().as_ref().contains(&b':')
                        || attributes
                            .iter()
                            .any(|attribute| attribute.key.as_ref().contains(&b':')))
                {
                    bail!(
                        "Unsupported target-name-storage: namespaced target/stamp attributes are not qualified; nothing was changed."
                    );
                }
                let mut replacements = Vec::new();
                if selected {
                    let raw_identity = attributes
                        .iter()
                        .find(|attribute| attribute.key.as_ref() == identity_key.as_bytes())
                        .map(|attribute| attribute.unescape_value())
                        .transpose()?;
                    if element.name().as_ref() != entry_tag.as_bytes()
                        || raw_identity.as_deref() != identity_value
                    {
                        bail!(
                            "Unsupported target-name-storage: raw selected entry identity differs from admitted identity; nothing was changed."
                        );
                    }
                    for attribute in &attributes {
                        if attribute.key.as_ref() == name_key.as_bytes()
                            && (attribute.value.windows(2).any(|bytes| bytes == b"&#")
                                || attribute
                                    .value
                                    .iter()
                                    .any(|byte| matches!(byte, b'\t' | b'\r' | b'\n')))
                        {
                            bail!(
                                "Unsupported target-name-storage: the selected Name contains numeric XML references or literal tab/newline storage. Native string decoding differs from standard XML normalization; nothing was changed."
                            );
                        }
                        if attribute.key.as_ref() == name_key.as_bytes() {
                            previous_name = Some(
                                std::str::from_utf8(attribute.value.as_ref())?
                                    .replace("&lt;", "<")
                                    .replace("&gt;", ">")
                                    .replace("&quot;", "\"")
                                    .replace("&apos;", "'")
                                    .replace("&amp;", "&"),
                            );
                        }
                    }
                    found = true;
                    if clear_irrigation_name {
                        for attribute in &attributes {
                            if attribute.key.as_ref() != name_key.as_bytes() {
                                continue;
                            }
                            let key_start = (attribute.key.as_ref().as_ptr() as usize)
                                .checked_sub(data.as_ptr() as usize)
                                .filter(|start| {
                                    data.get(*start..*start + name_key.len())
                                        == Some(name_key.as_bytes())
                                })
                                .ok_or_else(|| anyhow::anyhow!("Cannot locate raw irrigation nm attribute; nothing was changed."))?;
                            let value_start = (attribute.value.as_ptr() as usize)
                                .checked_sub(data.as_ptr() as usize)
                                .filter(|start| {
                                    data.get(*start..*start + attribute.value.len())
                                        == Some(attribute.value.as_ref())
                                })
                                .ok_or_else(|| anyhow::anyhow!("Cannot locate raw irrigation nm value; nothing was changed."))?;
                            let end = value_start + attribute.value.len() + 1;
                            if !matches!(data.get(end - 1), Some(b'\'' | b'"')) {
                                bail!(
                                    "Cannot locate raw irrigation nm closing quote; nothing was changed."
                                );
                            }
                            let start = if key_start > 0 && data[key_start - 1] == b' ' {
                                key_start - 1
                            } else {
                                key_start
                            };
                            patches.push((start..end, String::new()));
                        }
                    } else if let Some(name) = name {
                        replacements.push((name_key, name));
                    }
                }
                if document {
                    document_seen = true;
                    if let Some((date, seconds)) = stamp {
                        replacements.extend([("Date", date), ("DateS", seconds)]);
                    }
                }
                for (key, value) in replacements {
                    let value = quick_xml::escape::escape(value);
                    let mut matching = attributes
                        .iter()
                        .filter(|attribute| attribute.key.as_ref() == key.as_bytes())
                        .peekable();
                    if matching.peek().is_none() {
                        let insertion = end - if empty { 2 } else { 1 };
                        patches.push((insertion..insertion, format!(" {key}=\"{value}\"")));
                    }
                    for attribute in matching {
                        // Slice-reader attributes borrow the original, still-escaped value.
                        let start = (attribute.value.as_ptr() as usize)
                            .checked_sub(data.as_ptr() as usize)
                            .filter(|start| data.get(*start..*start + attribute.value.len()) == Some(attribute.value.as_ref()))
                            .ok_or_else(|| anyhow::anyhow!("Cannot locate raw alarm Name/save stamp attribute; nothing was changed."))?;
                        patches.push((start..start + attribute.value.len(), value.to_string()));
                    }
                }
                ordinal += 1;
                if !empty {
                    depth += 1;
                }
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    if !encoding.eq_ignore_ascii_case(b"utf-8")
                        && !encoding.eq_ignore_ascii_case(b"utf8")
                    {
                        bail!(
                            "Unsupported target-name-storage: a UTF-8 XML declaration is required; nothing was changed."
                        );
                    }
                }
            }
            Event::DocType(_) => bail!(
                "Unsupported target-name-storage: DTD-dependent Name decoding is not qualified; nothing was changed."
            ),
            Event::Eof => break,
            _ => {}
        }
    }
    if !found || (stamp.is_some() && !document_seen) {
        bail!(
            "Cannot locate target-name-storage or Document stamp in the original XML; nothing was changed."
        );
    }
    patches.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut output = data.to_vec();
    for (range, value) in patches {
        output.splice(range, value.bytes());
    }
    Ok((output, previous_name))
}

impl ConfigEditor {
    /// Source bytes are needed to guard native Name decoding before XML normalization.
    pub fn edit_alarm_clock_entry_name(
        data: &[u8],
        selector: &str,
        entry_uuid: &str,
        name: &str,
    ) -> Result<(Vec<u8>, AlarmClockEntryNameChange)> {
        if name.trim().is_empty()
            || name.chars().any(|character| {
                character.is_control()
                    || matches!(character, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')
            })
        {
            bail!(
                "Alarm entry name: this CLI supports nonempty, nonblank single-line XML text only; nothing was changed."
            );
        }
        let mut editor = Self::load(data)?;
        let (path, position, _) = editor.ordinary_alarm_clock_entry(selector, entry_uuid)?;
        let block = editor.get_element(&path);
        let entry = block.children[position].as_element().unwrap();
        let ordinal = entry_name_ordinal(&editor.root, entry, &mut 0).unwrap();
        let owner = entry_name_ordinal(&editor.root, block, &mut 0).unwrap();
        let (_, previous_name) = entry_name_bytes(
            data,
            ordinal,
            owner,
            EntryNameStorage::Alarm {
                entry_uuid: &entry.attributes["U"],
                block_uuid: block.attributes.get("U").map_or("", String::as_str),
            },
            None,
            None,
        )?;
        let change = AlarmClockEntryNameChange {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry_uuid: entry_uuid.into(),
            stored_entry_uuid: entry.attributes["U"].clone(),
            changed: previous_name.as_deref().unwrap_or("") != name,
            previous_name,
            name: name.into(),
        };
        let stamp = if change.changed && editor.stamp_document(&chrono::Local::now().fixed_offset())
        {
            let is_document = |element: &&Element| {
                element.name == "C"
                    && element.attributes.get("Type").map(String::as_str) == Some("Document")
            };
            let document = std::iter::once(&editor.root)
                .chain(editor.root.children.iter().filter_map(XMLNode::as_element))
                .find(is_document)
                .unwrap();
            Some((
                document.attributes["Date"].as_str(),
                document.attributes["DateS"].as_str(),
            ))
        } else {
            None
        };
        let (output, _) = entry_name_bytes(
            data,
            ordinal,
            owner,
            EntryNameStorage::Alarm {
                entry_uuid: &change.stored_entry_uuid,
                block_uuid: &change.block_uuid,
            },
            change.changed.then_some(name),
            stamp,
        )?;
        Ok((output, change))
    }

    fn admitted_alarm_clock_entries(
        &self,
        selector: &str,
        entry_uuid: Option<&str>,
    ) -> Result<(Vec<usize>, Option<usize>, usize)> {
        let requested = entry_uuid.map(uuid_key).transpose()?;
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.name != "C"
            || block.attributes.get("Type").map(String::as_str) != Some("AlarmClock")
        {
            bail!("'{selector}' is not an AlarmClock; nothing was changed.");
        }
        if integer(block, "V")? != 175 {
            bail!(
                "Alarm entry editing supports V=175 storage only; legacy/default migration is not qualified. Nothing was changed."
            );
        }
        let entries: Vec<_> = block
            .children
            .iter()
            .enumerate()
            .filter_map(|(position, node)| {
                node.as_element()
                    .filter(|element| element.name == "AlarmClock")
                    .map(|entry| (position, entry))
            })
            .collect();
        if integer(block, "N")? as usize != entries.len() {
            bail!("AlarmClock N does not match stored alarm entries; nothing was changed.");
        }
        let mut uuids = HashSet::new();
        let mut ids = HashSet::new();
        let mut internal_count = 0;
        let mut previous_time = None;
        let mut selected = None;
        for (position, entry) in &entries {
            let stored_uuid = entry
                .attributes
                .get("U")
                .ok_or_else(|| anyhow::anyhow!("Alarm entry is missing U; nothing was changed."))?;
            if stored_uuid.len() != 35 {
                bail!(
                    "Stored alarm entry U must use native 35-character 8-4-4-16 grouping; RFC UUID aliases are accepted only for --entry-uuid. Nothing was changed."
                );
            }
            let key = uuid_key(stored_uuid)?;
            let id = integer(entry, "Id")?;
            if !uuids.insert(key.clone()) || !ids.insert(id) {
                bail!(
                    "AlarmClock contains duplicate entry UUIDs or Id values; nothing was changed."
                );
            }
            if ["Year", "Month", "Day"]
                .iter()
                .map(|key| integer(entry, key))
                .collect::<Result<Vec<_>>>()?
                .iter()
                .any(|value| *value != 0)
            {
                bail!("Dated alarms are not exposed by this dialog route; nothing was changed.");
            }
            let time = integer(entry, "Time")?;
            if time >= 86400 || previous_time.is_some_and(|previous| previous >= time) {
                bail!(
                    "Alarm entries must already have strictly increasing valid Time values; native locale-dependent tie sorting is not qualified. Nothing was changed."
                );
            }
            previous_time = Some(time);
            let modes = integer(entry, "Modes")?;
            if !(1..=1023).contains(&modes) {
                bail!(
                    "Alarm entry Modes must be a nonzero 10-bit mask; native normalization is not applied. Nothing was changed."
                );
            }
            flag(entry, "Active")?;
            flag(entry, "Sa")?;
            let internal = flag(entry, "Int")?;
            if internal {
                internal_count += 1;
                if id != 0 {
                    bail!("The internal standard alarm must retain Id=0; nothing was changed.");
                }
            } else if id == 0 {
                bail!(
                    "Ordinary alarm Id=0 requires native default/reference migration; nothing was changed."
                );
            }
            if requested.as_ref() == Some(&key) {
                if internal {
                    bail!(
                        "This command supports ordinary alarms only, not the internal standard alarm (Int=true); nothing was changed."
                    );
                }
                selected = Some(*position);
            }
        }
        if internal_count != 1 {
            bail!(
                "This operation requires exactly one existing internal standard alarm; default creation/repair is not implemented. Nothing was changed."
            );
        }
        if let Some(entry_uuid) = entry_uuid
            && selected.is_none()
        {
            bail!("No configured alarm entry matches UUID '{entry_uuid}'; nothing was changed.");
        }
        let count = entries.len();
        Ok((path, selected, count))
    }

    fn ordinary_alarm_clock_entry(
        &self,
        selector: &str,
        entry_uuid: &str,
    ) -> Result<(Vec<usize>, usize, usize)> {
        let (path, position, count) =
            self.admitted_alarm_clock_entries(selector, Some(entry_uuid))?;
        Ok((path, position.unwrap(), count))
    }

    pub fn add_alarm_clock_entry(
        &mut self,
        selector: &str,
        name: &str,
        time: &str,
    ) -> Result<AlarmClockEntryAddition> {
        if name.trim().is_empty()
            || name
                .chars()
                .any(|c| c.is_control() || matches!(c, '\u{fffe}' | '\u{ffff}'))
            || name.len() > (i32::MAX as usize).saturating_sub(2)
        {
            bail!("Alarm entry name must be nonempty single-line XML text; nothing was changed.");
        }
        let seconds = time_seconds(time)?;
        let (path, _, count) = self.admitted_alarm_clock_entries(selector, None)?;
        let block = self.get_element(&path);
        let mut ids = HashSet::new();
        let mut insertion = None;
        let mut after_last = 0;
        for (position, node) in block.children.iter().enumerate() {
            if let Some(entry) = node.as_element().filter(|entry| entry.name == "AlarmClock") {
                ids.insert(integer(entry, "Id")?);
                let existing_time = integer(entry, "Time")?;
                if existing_time == seconds {
                    bail!(
                        "An alarm already uses {time}; equal-time native name/locale sorting is outside this command. Nothing was changed."
                    );
                }
                if insertion.is_none() && existing_time > seconds {
                    insertion = Some(position);
                }
                after_last = position + 1;
            }
        }
        let total_entries = count
            .checked_add(1)
            .filter(|count| *count <= i32::MAX as usize)
            .ok_or_else(|| {
                anyhow::anyhow!("Alarm count cannot be represented; nothing was changed.")
            })?;
        // Native allocation restarts its scan at zero after every occupied Id.
        let id = (0..=count as u32)
            .find(|id| !ids.contains(id))
            .ok_or_else(|| {
                anyhow::anyhow!("No representable free alarm Id; nothing was changed.")
            })?;
        fn used(element: &Element, key: &str) -> bool {
            element
                .attributes
                .get("U")
                .and_then(|raw| uuid_key(raw).ok())
                .is_some_and(|existing| existing == key)
                || element
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .any(|child| used(child, key))
        }
        let entry_uuid = (0..16)
            .find_map(|_| {
                let hex = uuid::Uuid::new_v4().simple().to_string();
                (!used(&self.root, &hex)).then(|| {
                    format!(
                        "{}-{}-{}-{}",
                        &hex[..8],
                        &hex[8..12],
                        &hex[12..16],
                        &hex[16..]
                    )
                })
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Cannot allocate an unused alarm UUID; nothing was changed.")
            })?;
        let addition = AlarmClockEntryAddition {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry_uuid: entry_uuid.clone(),
            id,
            name: name.to_owned(),
            time_seconds: seconds,
            active: true,
            modes: 1023,
            total_entries,
        };
        let mut entry = Element::new("AlarmClock");
        for (key, value) in [
            ("U", entry_uuid),
            ("Id", id.to_string()),
            ("Name", name.to_owned()),
            ("Active", "true".into()),
            ("Modes", "1023".into()),
        ] {
            entry.attributes.insert(key.into(), value);
        }
        if seconds != 0 {
            entry.attributes.insert("Time".into(), seconds.to_string());
        }
        let block = self.get_element_mut(&path);
        block
            .children
            .insert(insertion.unwrap_or(after_last), XMLNode::Element(entry));
        block
            .attributes
            .insert("N".into(), total_entries.to_string());
        Ok(addition)
    }

    pub fn set_alarm_clock_entry_time(
        &mut self,
        selector: &str,
        entry_uuid: &str,
        time: &str,
    ) -> Result<AlarmClockEntryTimeChange> {
        let seconds = time_seconds(time)?;
        let (path, position, _) = self.ordinary_alarm_clock_entry(selector, entry_uuid)?;
        let block = self.get_element(&path);
        let entry = block.children[position].as_element().unwrap();
        let previous_time_seconds = integer(entry, "Time")?;
        for (index, node) in block.children.iter().enumerate() {
            if index != position
                && let Some(other) = node.as_element().filter(|entry| entry.name == "AlarmClock")
                && integer(other, "Time")? == seconds
            {
                bail!(
                    "Another alarm already uses {time}; equal-time native name/locale sorting is outside this command. Nothing was changed."
                );
            }
        }
        let change = AlarmClockEntryTimeChange {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry_uuid: entry_uuid.to_owned(),
            stored_entry_uuid: entry.attributes["U"].clone(),
            previous_time_seconds,
            time_seconds: seconds,
            changed: previous_time_seconds != seconds,
        };
        if change.changed {
            let block = self.get_element_mut(&path);
            let entry = block.children[position].as_mut_element().unwrap();
            if seconds == 0 {
                entry.attributes.shift_remove("Time");
            } else {
                entry.attributes.insert("Time".into(), seconds.to_string());
            }
            // Native serialization physically sorts record pointers; keep non-entry XML slots intact.
            let positions: Vec<_> = block
                .children
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    node.as_element()
                        .filter(|entry| entry.name == "AlarmClock")
                        .map(|_| index)
                })
                .collect();
            let mut entries: Vec<_> = positions
                .iter()
                .map(|index| block.children[*index].clone())
                .collect();
            entries.sort_by_key(|node| integer(node.as_element().unwrap(), "Time").unwrap());
            for (index, node) in positions.into_iter().zip(entries) {
                block.children[index] = node;
            }
        }
        Ok(change)
    }

    pub fn set_alarm_clock_entry_active(
        &mut self,
        selector: &str,
        entry_uuid: &str,
        active: bool,
    ) -> Result<AlarmClockEntryActivation> {
        let (path, position, _) = self.ordinary_alarm_clock_entry(selector, entry_uuid)?;
        let block = self.get_element(&path);
        let entry = block.children[position].as_element().unwrap();
        let previous_active = flag(entry, "Active")?;
        let change = AlarmClockEntryActivation {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry_uuid: entry_uuid.to_owned(),
            stored_entry_uuid: entry.attributes["U"].clone(),
            previous_active,
            active,
            changed: previous_active != active,
        };
        if change.changed {
            let entry = self.get_element_mut(&path).children[position]
                .as_mut_element()
                .unwrap();
            if active {
                entry.attributes.insert("Active".into(), "true".into());
            } else {
                entry.attributes.shift_remove("Active");
            }
        }
        Ok(change)
    }

    pub fn set_alarm_clock_entry_mode(
        &mut self,
        selector: &str,
        entry_uuid: &str,
        mode_bit: u8,
        enabled: bool,
    ) -> Result<AlarmClockEntryModeChange> {
        if mode_bit >= 10 {
            bail!(
                "Alarm mode bit must be 0..9, not a column or global mode Id; nothing was changed."
            );
        }
        let (path, position, _) = self.ordinary_alarm_clock_entry(selector, entry_uuid)?;
        let block = self.get_element(&path);
        let entry = block.children[position].as_element().unwrap();
        let previous_modes = integer(entry, "Modes")?;
        let mask = 1u32 << mode_bit;
        let modes = if enabled {
            previous_modes | mask
        } else {
            previous_modes & !mask
        };
        if modes == 0 {
            bail!(
                "Clearing mode bit {mode_bit} would remove the final selected mode. Native zero-Modes save/load normalization is outside this command; nothing was changed. Use alarm-entry-set-active --no-active to disable the alarm without clearing its modes."
            );
        }
        let change = AlarmClockEntryModeChange {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry_uuid: entry_uuid.to_owned(),
            stored_entry_uuid: entry.attributes["U"].clone(),
            mode_bit,
            previous_enabled: previous_modes & mask != 0,
            enabled,
            previous_modes,
            modes,
            active: flag(entry, "Active")?,
            changed: modes != previous_modes,
        };
        // The native setter changes Active only across a zero-Modes boundary.
        if change.changed {
            self.get_element_mut(&path).children[position]
                .as_mut_element()
                .unwrap()
                .attributes
                .insert("Modes".into(), modes.to_string());
        }
        Ok(change)
    }

    pub fn remove_alarm_clock_entry(
        &mut self,
        selector: &str,
        entry_uuid: &str,
    ) -> Result<AlarmClockEntryRemoval> {
        let (path, position, count) = self.ordinary_alarm_clock_entry(selector, entry_uuid)?;
        let block = self.get_element(&path);
        let entry = block.children[position].as_element().unwrap();
        let removal = AlarmClockEntryRemoval {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry_uuid: entry_uuid.to_owned(),
            stored_entry_uuid: entry.attributes["U"].clone(),
            removed_attributes: entry
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            remaining_entries: count - 1,
        };
        let block = self.get_element_mut(&path);
        block.children.remove(position);
        block
            .attributes
            .insert("N".into(), removal.remaining_entries.to_string());
        Ok(removal)
    }
}
