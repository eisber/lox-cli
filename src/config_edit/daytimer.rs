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

#[derive(Debug, Serialize)]
pub struct DayTimerEntryStartChange {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub new_from_minutes: u32,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct DayTimerEntryMoveChange {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub shift_minutes: i32,
    pub new_from_minutes: u32,
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

struct BoundaryStorage {
    to: Range<usize>,
    from: Option<Range<usize>>,
    from_attribute: Option<Range<usize>>,
    entry_insert: usize,
    dates: BTreeMap<String, Range<usize>>,
    document_insert: usize,
}

fn boundary_storage(
    data: &[u8],
    editor: &ConfigEditor,
    selected: &SelectedEntry,
    document_path: &[usize],
) -> Result<BoundaryStorage> {
    std::str::from_utf8(data).context("DayTimer boundary editing requires UTF-8 XML")?;
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
    let mut from = None;
    let mut from_attribute = None;
    let mut entry_insert = None;
    let mut dates = BTreeMap::new();
    let mut document_insert = None;
    let mut depth = 0usize;
    let mut roots = 0usize;
    loop {
        let event_start = reader.buffer_position();
        let event = reader.read_event()?;
        let opening = matches!(&event, Event::Start(_));
        match event {
            Event::Start(raw) | Event::Empty(raw) => {
                if depth == 0 {
                    roots += 1;
                    ensure!(
                        roots == 1,
                        "DayTimer XML requires one balanced root; nothing was changed."
                    );
                }
                if opening {
                    depth += 1;
                }
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
                    let identity = (element.name == "C" && matches!(key, "Type" | "U" | "Title"))
                        || (matches!(element.name.as_str(), "Co" | "Ref") && key == "U")
                        || (element.name == "Co" && key == "K");
                    let guarded = owner
                        || entry
                        || identity
                        || (stamp && matches!(key, "Date" | "DateS"))
                        || (ancestors.contains(&ordinal) && matches!(key, "WF" | "uuidParent"))
                        || (element.name == "IoData" && key == "Pr");
                    if let Some(previous) = seen.insert(key.to_owned(), attribute.value.to_vec()) {
                        let guarded_duplicate = owner
                            || entry
                            || (element.name == "C" && matches!(key, "Type" | "U"))
                            || (matches!(element.name.as_str(), "Co" | "Ref") && key == "U")
                            || (element.name == "Co" && key == "K")
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
                    if (selected_entry && matches!(key, "From" | "To"))
                        || (stamp && matches!(key, "Date" | "DateS"))
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
                        if selected_entry && key == "From" {
                            let attribute_start = (attribute.key.as_ref().as_ptr() as usize)
                                .checked_sub(data.as_ptr() as usize)
                                .filter(|start| {
                                    data.get(*start..*start + key.len()) == Some(key.as_bytes())
                                })
                                .context("Cannot locate original DayTimer From attribute")?;
                            from_attribute = Some(attribute_start..range.end + 1);
                            from = Some(range);
                        } else if selected_entry {
                            to = Some(range);
                        } else {
                            dates.insert(key.to_owned(), range);
                        }
                    }
                }
                if stamp || selected_entry {
                    let end = reader.buffer_position() as usize + offset;
                    let insert = end - if data[end - 2] == b'/' { 2 } else { 1 };
                    if stamp {
                        document_insert = Some(insert);
                    }
                    if selected_entry {
                        entry_insert = Some(insert);
                    }
                }
                ordinal += 1;
            }
            Event::Decl(declaration) => {
                ensure!(
                    event_start == 0 && roots == 0,
                    "DayTimer XML declaration must precede all other content; nothing was changed."
                );
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"utf8"),
                        "DayTimer boundary editing requires UTF-8 XML; nothing was changed."
                    );
                }
            }
            Event::PI(instruction) => ensure!(
                super::input_inversion::valid_processing_instruction(instruction.as_ref()),
                "DayTimer editing requires valid XML processing instructions; nothing was changed."
            ),
            Event::DocType(_) => {
                bail!("DTD-dependent DayTimer XML is unsupported; nothing was changed.")
            }
            Event::End(_) => {
                ensure!(
                    depth > 0,
                    "DayTimer XML requires one balanced root; nothing was changed."
                );
                depth -= 1;
            }
            Event::Text(text) if depth == 0 => {
                ensure!(
                    text.as_ref().iter().all(u8::is_ascii_whitespace),
                    "Trailing DayTimer XML text is unsupported; nothing was changed."
                );
            }
            Event::CData(_) if depth == 0 => {
                bail!("DayTimer XML data outside the root is unsupported; nothing was changed.");
            }
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(
        ordinal == elements.len() && depth == 0 && roots == 1,
        "DayTimer raw/DOM shape differs; nothing was changed."
    );
    Ok(BoundaryStorage {
        to: to.context("Missing original DayTimer To; nothing was changed")?,
        from,
        from_attribute,
        entry_insert: entry_insert
            .context("Missing original DayTimer Entry; nothing was changed")?,
        dates,
        document_insert: document_insert
            .context("Missing original Document; nothing was changed")?,
    })
}

impl ConfigEditor {
    pub fn move_daytimer_entry(
        data: &[u8],
        selector: &str,
        identity: DayTimerEntryIdentity,
        shift_minutes: i32,
    ) -> Result<(Vec<u8>, DayTimerEntryMoveChange)> {
        let mut editor = Self::load(data)?;
        let selected = editor.select_daytimer_entry(selector, &identity)?;
        let (document_path, block_uuid) = editor.daytimer_boundary_context(&selected)?;
        for depth in document_path.len()..=selected.path.len() {
            ensure!(
                !editor
                    .get_element(&selected.path[..depth])
                    .attributes
                    .contains_key("IfT"),
                "DayTimer movement does not emulate IfT instance state; nothing was changed."
            );
        }
        let block = editor.get_element(&selected.path);
        let identities =
            std::iter::once(block_uuid.as_str()).chain(block.attributes["Modes"].split(','));
        let elements = Self::iter_elements(&editor.root);
        for identity in identities {
            let key = mode_uuid_key(identity);
            ensure!(
                elements
                    .iter()
                    .filter(|element| {
                        matches!(element.name.as_str(), "C" | "Co" | "Ref")
                            && element.attributes.get("U").is_some_and(|candidate| {
                                key.as_ref().map_or_else(
                                    || candidate == identity,
                                    |key| mode_uuid_key(candidate).as_ref() == Some(key),
                                )
                            })
                    })
                    .count()
                    == 1,
                "Ambiguous global DayTimer owner or mode identity; nothing was changed."
            );
        }
        let original_from = identity.from_minutes as i32;
        let original_to = identity.to_minutes as i32;
        ensure!(
            (-original_from - 1..=1440 - original_from).contains(&shift_minutes),
            "The shift must be a reachable timeline-minute delta from the interval's From grab point ({}..={} minutes); nothing was changed.",
            -original_from - 1,
            1440 - original_from
        );
        let duration = original_to - original_from;
        let mut from = original_from + shift_minutes;
        let mut to = original_to + shift_minutes;
        if from < 0 {
            from = 0;
            to = duration;
        } else if to > 1440 {
            to = 1440;
            from = 1440 - duration;
        }
        if (from, to) != (original_from, original_to) {
            // First native Ctrl-up body update: clamp, then snap From, not the delta.
            let remainder = from % 15;
            if (1..8).contains(&remainder) {
                from -= remainder;
                if to != 1440 {
                    to -= remainder;
                }
            } else if remainder >= 8 {
                from += 15 - remainder;
                to += 15 - remainder;
            }
        }
        ensure!(
            0 <= from && from < to && to <= 1440,
            "The native grid result leaves the supported day; a second clamp would invent semantics. Nothing was changed."
        );
        let target = block.children[selected.position].as_element().unwrap();
        let mode_index = integer(target, "Ix")?;
        let mut previous_key = None;
        let mut previous_new_key = None;
        for (position, child) in block.children.iter().enumerate() {
            let Some(entry) = child.as_element().filter(|entry| entry.name == "Entry") else {
                continue;
            };
            let index = integer(entry, "Ix")?;
            let entry_from = integer(entry, "From")?;
            let entry_to = integer(entry, "To")?;
            let key = (index, entry_from);
            let new_key = (
                index,
                if position == selected.position {
                    from as u32
                } else {
                    entry_from
                },
            );
            ensure!(
                previous_key.is_none_or(|previous| previous < key)
                    && previous_new_key.is_none_or(|previous| previous < new_key),
                "DayTimer movement must retain sorted Ix/From order; incidental native sorting is refused. Nothing was changed."
            );
            previous_key = Some(key);
            previous_new_key = Some(new_key);
            ensure!(
                position == selected.position
                    || index != mode_index
                    || (to as u32) < entry_from
                    || entry_to < from as u32,
                "The moved interval would touch or overlap another entry; incidental native normalization is refused. Nothing was changed."
            );
        }
        let storage = boundary_storage(data, &editor, &selected, &document_path)?;
        let change = DayTimerEntryMoveChange {
            block_uuid,
            changed: (from, to) != (original_from, original_to),
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            shift_minutes,
            new_from_minutes: from as u32,
            new_to_minutes: to as u32,
        };
        if !change.changed {
            return Ok((data.to_vec(), change));
        }
        let mut patches = Vec::new();
        if from != original_from {
            patches.push(if from == 0 {
                (
                    storage
                        .from_attribute
                        .clone()
                        .context("Missing selected From attribute")?,
                    String::new(),
                )
            } else if let Some(range) = &storage.from {
                (range.clone(), from.to_string())
            } else {
                (
                    storage.entry_insert..storage.entry_insert,
                    format!(" From=\"{from}\""),
                )
            });
        }
        if to != original_to {
            patches.push((storage.to.clone(), to.to_string()));
        }
        let output = editor.patch_daytimer_boundary(data, &document_path, &storage, patches)?;
        Ok((output, change))
    }

    fn daytimer_boundary_context(&self, selected: &SelectedEntry) -> Result<(Vec<usize>, String)> {
        let documents = self.find_elements("Type:Document");
        ensure!(
            documents.len() == 1 && documents[0].len() <= 1,
            "DayTimer boundary editing requires exactly one stampable root/direct-child Document; nothing was changed."
        );
        let document_path = &documents[0];
        let document = self.get_element(document_path);
        ensure!(
            document.name == "C"
                && document.attributes.get("Type").map(String::as_str) == Some("Document")
                && document.namespace.is_none()
                && document.prefix.is_none()
                && selected.path.starts_with(document_path),
            "DayTimer boundary editing requires an exact plain owning Document; nothing was changed."
        );
        ensure!(
            !document
                .children
                .iter()
                .filter_map(|node| node.as_element())
                .any(|child| child.name == "Template"),
            "DayTimer boundary editing does not emulate direct Document Template locks or accepted propagation; nothing was changed."
        );
        for depth in document_path.len()..=selected.path.len() {
            let ancestor = self.get_element(&selected.path[..depth]);
            ensure!(
                ancestor.attributes.get("Type").map(String::as_str) != Some("DocumentTemplate")
                    && !ancestor.attributes.contains_key("uuidParent")
                    && integer(ancestor, "WF")? & 0x2000_0000 == 0,
                "Template-owned DayTimer storage is unsupported; nothing was changed."
            );
        }
        let block = self.get_element(&selected.path);
        let block_uuid = block
            .attributes
            .get("U")
            .filter(|uuid| !uuid.is_empty())
            .context("DayTimer owner requires a unique stored U; nothing was changed")?
            .clone();
        let owner_key = mode_uuid_key(&block_uuid);
        let owners = Self::iter_elements(&self.root)
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
        ensure!(
            document.attributes.get("DateS").is_none_or(|value| {
                !value.is_empty()
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value
                        .parse::<i32>()
                        .is_ok_and(|seconds| (0..i32::MAX).contains(&seconds))
            }),
            "DayTimer boundary editing requires an absent or incrementable nonnegative signed32 Document DateS; nothing was changed."
        );
        Ok((document_path.clone(), block_uuid))
    }

    fn patch_daytimer_boundary(
        &mut self,
        data: &[u8],
        document_path: &[usize],
        storage: &BoundaryStorage,
        mut patches: Vec<(Range<usize>, String)>,
    ) -> Result<Vec<u8>> {
        ensure!(
            self.stamp_document(&chrono::Local::now().fixed_offset()),
            "Cannot stamp the admitted DayTimer Document; nothing was changed."
        );
        let document = self.get_element(document_path);
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
        Ok(output)
    }

    pub fn edit_daytimer_entry_start(
        data: &[u8],
        selector: &str,
        identity: DayTimerEntryIdentity,
        new_from: &str,
    ) -> Result<(Vec<u8>, DayTimerEntryStartChange)> {
        let new_from_minutes = timer_time(new_from)?;
        ensure!(
            new_from_minutes < identity.to_minutes,
            "The new From must be before the unchanged To; native coupled-To/clamp branches are not supported. Nothing was changed."
        );
        let mut editor = Self::load(data)?;
        let selected = editor.select_daytimer_entry(selector, &identity)?;
        let (document_path, block_uuid) = editor.daytimer_boundary_context(&selected)?;
        let block = editor.get_element(&selected.path);
        let target = block.children[selected.position].as_element().unwrap();
        let mode_index = integer(target, "Ix")?;
        let mut previous_key = None;
        let mut previous_new_key = None;
        for (position, child) in block.children.iter().enumerate() {
            let Some(entry) = child.as_element().filter(|entry| entry.name == "Entry") else {
                continue;
            };
            let index = integer(entry, "Ix")?;
            let from = integer(entry, "From")?;
            let to = integer(entry, "To")?;
            let key = (index, from);
            let new_key = (
                index,
                if position == selected.position {
                    new_from_minutes
                } else {
                    from
                },
            );
            ensure!(
                previous_key.is_none_or(|previous| previous < key)
                    && previous_new_key.is_none_or(|previous| previous < new_key),
                "DayTimer entries must retain their sorted Ix/From order; incidental native sorting is refused. Nothing was changed."
            );
            previous_key = Some(key);
            previous_new_key = Some(new_key);
            ensure!(
                position == selected.position
                    || index != mode_index
                    || identity.to_minutes < from
                    || to < new_from_minutes,
                "The new From would touch or overlap another entry; incidental native normalization is refused. Nothing was changed."
            );
        }
        let storage = boundary_storage(data, &editor, &selected, &document_path)?;
        let change = DayTimerEntryStartChange {
            block_uuid,
            changed: identity.from_minutes != new_from_minutes,
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            new_from_minutes,
        };
        if !change.changed {
            return Ok((data.to_vec(), change));
        }
        let patch = if new_from_minutes == 0 {
            (
                storage
                    .from_attribute
                    .clone()
                    .context("Missing selected From attribute")?,
                String::new(),
            )
        } else if let Some(range) = &storage.from {
            (range.clone(), new_from_minutes.to_string())
        } else {
            (
                storage.entry_insert..storage.entry_insert,
                format!(" From=\"{new_from_minutes}\""),
            )
        };
        let output = editor.patch_daytimer_boundary(data, &document_path, &storage, vec![patch])?;
        Ok((output, change))
    }

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
        let (document_path, block_uuid) = editor.daytimer_boundary_context(&selected)?;
        let block = editor.get_element(&selected.path);
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
        let storage = boundary_storage(data, &editor, &selected, &document_path)?;
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
        let output = editor.patch_daytimer_boundary(
            data,
            &document_path,
            &storage,
            vec![(storage.to.clone(), new_to_minutes.to_string())],
        )?;
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
