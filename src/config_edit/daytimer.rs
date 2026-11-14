use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use anyhow::{Context, Result, bail, ensure};
use quick_xml::{Reader, events::Event};
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;

#[derive(Debug, Serialize)]
pub struct DayTimerEntryIdentity {
    pub mode_uuid: String,
    pub from_minutes: u32,
    pub to_minutes: u32,
}

impl DayTimerEntryIdentity {
    pub fn parse(mode_uuid: &str, from: &str, to: &str) -> Result<Self> {
        mode_uuid_key(mode_uuid).ok_or_else(|| {
            anyhow::anyhow!("--mode-uuid must be a non-nil native 8-4-4-16 or RFC 8-4-4-4-12 UUID, using exactly 32 ASCII hex digits.")
        })?;
        let from_minutes = timer_time(from)?;
        let to_minutes = timer_time(to)?;
        if from_minutes >= to_minutes {
            bail!(
                "Entry times must satisfy 00:00 <= from < to <= 24:00; do not combine overnight entries."
            );
        }
        Ok(Self {
            mode_uuid: mode_uuid.to_owned(),
            from_minutes,
            to_minutes,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DayTimerEntryRemoval {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub removed_attributes: BTreeMap<String, String>,
    pub remaining_entries: usize,
}

#[derive(Debug, Serialize)]
pub struct DayTimerEntryActivationChange {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub previous_attribute: Option<String>,
    pub previous_required: bool,
    pub activation_required: bool,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct DayTimerEntryEndChange {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub new_to_minutes: u32,
    pub changed: bool,
}

struct SelectedEntry {
    path: Vec<usize>,
    position: usize,
    stored_mode_uuid: String,
    entry_count: usize,
}

fn timer_time(raw: &str) -> Result<u32> {
    let bytes = raw.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || ![bytes[0], bytes[1], bytes[3], bytes[4]]
            .iter()
            .all(u8::is_ascii_digit)
    {
        bail!("Invalid time '{raw}': use exact HH:MM in 00:00..24:00.");
    }
    let hour = u32::from(bytes[0] - b'0') * 10 + u32::from(bytes[1] - b'0');
    let minute = u32::from(bytes[3] - b'0') * 10 + u32::from(bytes[4] - b'0');
    if hour > 24 || minute > 59 || (hour == 24 && minute != 0) {
        bail!("Invalid time '{raw}': use exact HH:MM in 00:00..24:00.");
    }
    Ok(hour * 60 + minute)
}

fn integer(element: &Element, key: &str) -> Result<u32> {
    // Native type-5 reads with preserve-missing disabled parse absence as zero.
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
        "Unsupported DayTimer {key}='{raw}': expected a nonnegative signed-32-bit integer; nothing was changed."
    );
}

fn mode_uuid_key(raw: &str) -> Option<String> {
    let dashes: &[usize] = match raw.len() {
        35 => &[8, 13, 18],
        36 => &[8, 13, 18, 23],
        _ => return None,
    };
    let mut key = String::with_capacity(32);
    for (index, byte) in raw.bytes().enumerate() {
        if dashes.contains(&index) {
            if byte != b'-' {
                return None;
            }
        } else if byte.is_ascii_hexdigit() {
            key.push(char::from(byte.to_ascii_lowercase()));
        } else {
            return None;
        }
    }
    (!key.bytes().all(|byte| byte == b'0')).then_some(key)
}

struct EndStorage {
    to: Range<usize>,
    dates: BTreeMap<String, Range<usize>>,
    document_insert: usize,
}

fn end_storage(
    data: &[u8],
    editor: &ConfigEditor,
    selected: &SelectedEntry,
    document_path: &[usize],
) -> Result<EndStorage> {
    std::str::from_utf8(data).context("DayTimer end editing requires UTF-8 XML")?;
    let elements = ConfigEditor::iter_elements(&editor.root);
    let block = editor.get_element(&selected.path);
    let target = block.children[selected.position].as_element().unwrap();
    let document = editor.get_element(document_path);
    let mut ancestors = BTreeSet::new();
    for depth in document_path.len()..=selected.path.len() {
        let ancestor = editor.get_element(&selected.path[..depth]);
        ancestors.insert(
            elements
                .iter()
                .position(|element| std::ptr::eq(*element, ancestor))
                .unwrap(),
        );
    }
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    let mut ordinal = 0;
    let mut to = None;
    let mut dates = BTreeMap::new();
    let mut document_insert = None;
    loop {
        match reader.read_event()? {
            Event::Start(raw) | Event::Empty(raw) => {
                let element = elements
                    .get(ordinal)
                    .context("DayTimer raw/DOM shape differs")?;
                let selected_entry = std::ptr::eq(*element, target);
                let owner = std::ptr::eq(*element, block);
                let stamp = std::ptr::eq(*element, document);
                let entry = block
                    .children
                    .iter()
                    .filter_map(|node| node.as_element())
                    .any(|child| child.name == "Entry" && std::ptr::eq(child, *element));
                ensure!(
                    raw.name().as_ref() == element.name.as_bytes()
                        && element.namespace.is_none()
                        && element.prefix.is_none(),
                    "Namespaced or differing DayTimer raw/DOM elements are unsupported; nothing was changed."
                );
                let mut seen = BTreeMap::new();
                for attribute in raw.attributes().with_checks(false) {
                    let attribute =
                        attribute.context("Cannot inspect original DayTimer attributes")?;
                    let key = std::str::from_utf8(attribute.key.as_ref())?;
                    ensure!(
                        !key.contains(':') && key != "xmlns",
                        "Namespaced DayTimer XML is unsupported; nothing was changed."
                    );
                    let identity = element.name == "C" && matches!(key, "Type" | "U" | "Title");
                    let guarded = owner
                        || entry
                        || identity
                        || (stamp && matches!(key, "Date" | "DateS"))
                        || (ancestors.contains(&ordinal) && matches!(key, "WF" | "uuidParent"))
                        || (element.name == "IoData" && key == "Pr");
                    if let Some(previous) = seen.insert(key.to_owned(), attribute.value.to_vec()) {
                        let guarded_duplicate = owner
                            || entry
                            || (stamp && matches!(key, "Type" | "Date" | "DateS"))
                            || (ancestors.contains(&ordinal) && matches!(key, "WF" | "uuidParent"));
                        ensure!(
                            !guarded_duplicate && previous == attribute.value.as_ref(),
                            "Duplicate guarded or conflicting DayTimer attribute '{key}'; nothing was changed."
                        );
                        continue;
                    }
                    if guarded {
                        let sanitized = format!("_{key}");
                        let dom_key =
                            if editor
                                .digit_attr_renames
                                .iter()
                                .any(|(replacement, original)| {
                                    original == key && replacement == &sanitized
                                })
                            {
                                sanitized.as_str()
                            } else {
                                key
                            };
                        ensure!(
                            !attribute.value.windows(2).any(|bytes| bytes == b"&#")
                                && element.attributes.get(dom_key).map(String::as_str)
                                    == Some(attribute.unescape_value()?.as_ref()),
                            "Original DayTimer attribute '{key}' differs from admitted storage (including shared digit-name sanitizer effects); nothing was changed."
                        );
                    }
                    if (selected_entry && key == "To") || (stamp && matches!(key, "Date" | "DateS"))
                    {
                        let start = (attribute.value.as_ptr() as usize)
                            .checked_sub(data.as_ptr() as usize)
                            .filter(|start| {
                                data.get(*start..*start + attribute.value.len())
                                    == Some(attribute.value.as_ref())
                            })
                            .context(
                                "Cannot locate original DayTimer attribute; nothing was changed",
                            )?;
                        let range = start..start + attribute.value.len();
                        if selected_entry {
                            to = Some(range);
                        } else {
                            dates.insert(key.to_owned(), range);
                        }
                    }
                }
                if stamp {
                    let end = reader.buffer_position() as usize + offset;
                    document_insert = Some(end - if data[end - 2] == b'/' { 2 } else { 1 });
                }
                ordinal += 1;
            }
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"utf8"),
                        "DayTimer end editing requires UTF-8 XML; nothing was changed."
                    );
                }
            }
            Event::DocType(_) => {
                bail!("DTD-dependent DayTimer XML is unsupported; nothing was changed.")
            }
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(
        ordinal == elements.len(),
        "DayTimer raw/DOM shape differs; nothing was changed."
    );
    Ok(EndStorage {
        to: to.context("Missing original DayTimer To; nothing was changed")?,
        dates,
        document_insert: document_insert
            .context("Missing original Document; nothing was changed")?,
    })
}

impl ConfigEditor {
    pub fn edit_daytimer_entry_end(
        data: &[u8],
        selector: &str,
        identity: DayTimerEntryIdentity,
        new_to: &str,
    ) -> Result<(Vec<u8>, DayTimerEntryEndChange)> {
        let new_to_minutes = timer_time(new_to)?;
        ensure!(
            identity.from_minutes < new_to_minutes,
            "The new To must be after the unchanged From and at most 24:00; native clamp/wrap branches are not supported. Nothing was changed."
        );
        let mut editor = Self::load(data)?;
        let selected = editor.select_daytimer_entry(selector, &identity)?;
        let documents = editor.find_elements("Type:Document");
        ensure!(
            documents.len() == 1 && documents[0].len() <= 1,
            "DayTimer end editing requires exactly one stampable root/direct-child Document; nothing was changed."
        );
        let document_path = &documents[0];
        let document = editor.get_element(document_path);
        ensure!(
            document.name == "C"
                && document.attributes.get("Type").map(String::as_str) == Some("Document")
                && document.namespace.is_none()
                && document.prefix.is_none()
                && selected.path.starts_with(document_path),
            "DayTimer end editing requires an exact plain owning Document; nothing was changed."
        );
        ensure!(
            !document
                .children
                .iter()
                .filter_map(|node| node.as_element())
                .any(|child| child.name == "Template"),
            "DayTimer end editing does not emulate direct Document Template locks or accepted propagation; nothing was changed."
        );
        for depth in document_path.len()..=selected.path.len() {
            let ancestor = editor.get_element(&selected.path[..depth]);
            ensure!(
                ancestor.attributes.get("Type").map(String::as_str) != Some("DocumentTemplate")
                    && !ancestor.attributes.contains_key("uuidParent")
                    && integer(ancestor, "WF")? & 0x2000_0000 == 0,
                "Template-owned DayTimer storage is unsupported; nothing was changed."
            );
        }
        let block = editor.get_element(&selected.path);
        let block_uuid = block
            .attributes
            .get("U")
            .filter(|uuid| !uuid.is_empty())
            .context("DayTimer owner requires a unique stored U; nothing was changed")?
            .clone();
        let owner_key = mode_uuid_key(&block_uuid);
        let owners = Self::iter_elements(&editor.root)
            .into_iter()
            .filter(|element| {
                element.name == "C"
                    && element.attributes.get("U").is_some_and(|uuid| {
                        if let Some(key) = &owner_key {
                            mode_uuid_key(uuid).as_ref() == Some(key)
                        } else {
                            uuid == &block_uuid
                        }
                    })
            })
            .count();
        ensure!(
            owners == 1,
            "Ambiguous DayTimer owner identity; nothing was changed."
        );
        let target = block.children[selected.position].as_element().unwrap();
        let mode_index = integer(target, "Ix")?;
        let mut previous_key = None;
        for (position, child) in block.children.iter().enumerate() {
            let Some(entry) = child.as_element().filter(|entry| entry.name == "Entry") else {
                continue;
            };
            let index = integer(entry, "Ix")?;
            let from = integer(entry, "From")?;
            let to = integer(entry, "To")?;
            let key = (index, from);
            ensure!(
                previous_key.is_none_or(|previous| previous < key),
                "DayTimer entries are not already sorted by Ix/From; incidental native sorting is refused. Nothing was changed."
            );
            previous_key = Some(key);
            ensure!(
                position == selected.position
                    || index != mode_index
                    || new_to_minutes < from
                    || to < identity.from_minutes,
                "The new To would touch or overlap another entry; incidental native normalization is refused. Nothing was changed."
            );
        }
        let storage = end_storage(data, &editor, &selected, document_path)?;
        let change = DayTimerEntryEndChange {
            block_uuid,
            changed: identity.to_minutes != new_to_minutes,
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            new_to_minutes,
        };
        if !change.changed {
            return Ok((data.to_vec(), change));
        }
        ensure!(
            editor.stamp_document(&chrono::Local::now().fixed_offset()),
            "Cannot stamp the admitted DayTimer Document; nothing was changed."
        );
        let document = editor.get_element(document_path);
        let mut patches = vec![(storage.to, new_to_minutes.to_string())];
        let mut inserted = String::new();
        for key in ["Date", "DateS"] {
            let value = &document.attributes[key];
            if let Some(range) = storage.dates.get(key) {
                patches.push((range.clone(), value.clone()));
            } else {
                inserted.push_str(&format!(" {key}=\"{value}\""));
            }
        }
        if !inserted.is_empty() {
            patches.push((storage.document_insert..storage.document_insert, inserted));
        }
        patches.sort_by_key(|patch| std::cmp::Reverse(patch.0.start));
        let mut output = data.to_vec();
        for (range, value) in patches {
            output.splice(range, value.bytes());
        }
        Ok((output, change))
    }

    fn select_daytimer_entry(
        &self,
        selector: &str,
        identity: &DayTimerEntryIdentity,
    ) -> Result<SelectedEntry> {
        let mode_uuid = mode_uuid_key(&identity.mode_uuid)
            .ok_or_else(|| anyhow::anyhow!("Invalid operating mode UUID; nothing was changed."))?;
        if identity.from_minutes >= identity.to_minutes || identity.to_minutes > 1440 {
            bail!("Entry times must satisfy 00:00 <= from < to <= 24:00; nothing was changed.");
        }
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("DayTimer") {
            bail!("'{selector}' is not a DayTimer; nothing was changed.");
        }
        if integer(block, "M")? != 0 {
            bail!("Only ordinary DayTimer M=0 schedules are supported; nothing was changed.");
        }
        match block.attributes.get("Analog").map(String::as_str) {
            None | Some("false") => {}
            Some("true") => bail!(
                "Analog DayTimer entry editing is not yet qualified; this operation supports digital schedules only. Nothing was changed."
            ),
            _ => bail!("Unsupported DayTimer Analog spelling; nothing was changed."),
        }
        let stored_modes: Vec<_> = block
            .attributes
            .get("Modes")
            .ok_or_else(|| {
                anyhow::anyhow!("DayTimer has no explicit Modes references; nothing was changed.")
            })?
            .split(',')
            .collect();
        let modes = stored_modes
            .iter()
            .map(|raw| mode_uuid_key(raw))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                anyhow::anyhow!("DayTimer Modes requires comma-separated non-nil native 8-4-4-16 or RFC 8-4-4-4-12 UUIDs without whitespace; nothing was changed.")
            })?;
        for (index, mode) in modes.iter().enumerate() {
            if modes[..index].contains(mode) {
                bail!("DayTimer Modes contains duplicate references; nothing was changed.");
            }
            let matches: Vec<_> = Self::iter_elements(&self.root)
                .into_iter()
                .filter(|element| {
                    element.name == "C"
                        && element
                            .attributes
                            .get("U")
                            .and_then(|raw| mode_uuid_key(raw))
                            .as_ref()
                            == Some(mode)
                })
                .collect();
            if matches.len() != 1
                || matches[0].attributes.get("Type").map(String::as_str) != Some("Mode")
            {
                bail!(
                    "DayTimer operating mode '{}' is missing, ambiguous or not a Mode; nothing was changed.",
                    stored_modes[index]
                );
            }
        }
        let mode_index = modes
            .iter()
            .position(|mode| *mode == mode_uuid)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Requested operating mode is not in this DayTimer's Modes; nothing was changed."
                )
            })?;
        let positions: Vec<_> = block
            .children
            .iter()
            .enumerate()
            .filter_map(|(position, child)| {
                child
                    .as_element()
                    .filter(|element| element.name == "Entry")
                    .map(|_| position)
            })
            .collect();
        if integer(block, "N")? as usize != positions.len() {
            bail!(
                "DayTimer N does not match stored entries; refusing to repair counts or interpret legacy timer-schedule storage."
            );
        }
        let mut records = Vec::new();
        for position in &positions {
            let entry = block.children[*position].as_element().unwrap();
            let from = integer(entry, "From")?;
            let to = integer(entry, "To")?;
            let index = integer(entry, "Ix")? as usize;
            if from >= to || to > 1440 || index >= modes.len() {
                bail!(
                    "DayTimer contains an invalid interval or unresolved Ix; nothing was changed."
                );
            }
            if records.iter().any(|&(other_index, other_from, other_to)| {
                index == other_index && from <= other_to && other_from <= to
            }) {
                bail!(
                    "Overlapping or touching entries in one mode may require native normalization; this bounded edit refuses them without changing anything."
                );
            }
            records.push((index, from, to));
        }
        let selected = records
            .iter()
            .position(|record| {
                *record == (mode_index, identity.from_minutes, identity.to_minutes)
            })
            .ok_or_else(|| anyhow::anyhow!("No existing entry matches that operating mode and exact interval; nothing was changed."))?;
        let position = positions[selected];
        Ok(SelectedEntry {
            stored_mode_uuid: stored_modes[mode_index].to_owned(),
            entry_count: positions.len(),
            path,
            position,
        })
    }

    pub fn remove_daytimer_entry(
        &mut self,
        selector: &str,
        identity: DayTimerEntryIdentity,
    ) -> Result<DayTimerEntryRemoval> {
        let selected = self.select_daytimer_entry(selector, &identity)?;
        let block = self.get_element(&selected.path);
        let removal = DayTimerEntryRemoval {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            removed_attributes: block.children[selected.position]
                .as_element()
                .unwrap()
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            remaining_entries: selected.entry_count - 1,
        };
        let block = self.get_element_mut(&selected.path);
        block.children.remove(selected.position);
        // Native N counts serialized non-default records and omits zero.
        if removal.remaining_entries == 0 {
            block.attributes.shift_remove("N");
        } else {
            block
                .attributes
                .insert("N".into(), removal.remaining_entries.to_string());
        }
        Ok(removal)
    }

    pub fn set_daytimer_entry_activation(
        &mut self,
        selector: &str,
        identity: DayTimerEntryIdentity,
        activation_required: bool,
    ) -> Result<DayTimerEntryActivationChange> {
        let selected = self.select_daytimer_entry(selector, &identity)?;
        let block = self.get_element(&selected.path);
        let entry = block.children[selected.position].as_element().unwrap();
        let previous_attribute = entry.attributes.get("NeedActivate").cloned();
        let previous_required = match previous_attribute.as_deref() {
            None => false,
            Some(raw) if raw.eq_ignore_ascii_case("true") => true,
            Some(raw) if raw.eq_ignore_ascii_case("false") => false,
            Some(raw) => bail!(
                "Unsupported DayTimer NeedActivate='{raw}': use true/false (case-insensitive) or absence; nothing was changed."
            ),
        };
        let change = DayTimerEntryActivationChange {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            previous_attribute,
            previous_required,
            activation_required,
            changed: previous_required != activation_required,
        };
        if change.changed {
            let entry = self.get_element_mut(&selected.path).children[selected.position]
                .as_mut_element()
                .unwrap();
            if activation_required {
                entry
                    .attributes
                    .insert("NeedActivate".into(), "true".into());
            } else {
                entry.attributes.shift_remove("NeedActivate");
            }
        }
        Ok(change)
    }
}
