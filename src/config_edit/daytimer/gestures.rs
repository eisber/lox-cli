use super::*;

#[derive(Debug, Serialize)]
pub struct DayTimerEntryCopy {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub shift_minutes: i32,
    pub new_from_minutes: u32,
    pub new_to_minutes: u32,
    pub activation_required: bool,
    pub source_previous_index: usize,
    pub copy_index: usize,
    pub entry_count: usize,
}

#[derive(Debug, Serialize)]
pub struct DayTimerEntryModeMove {
    pub block_uuid: String,
    pub entry: DayTimerEntryIdentity,
    pub stored_mode_uuid: String,
    pub target_mode_uuid: String,
    pub stored_target_mode_uuid: String,
    pub previous_mode_index: u32,
    pub target_mode_index: u32,
    pub changed: bool,
}

struct GestureStorage {
    boundary: BoundaryStorage,
    count: Range<usize>,
    index: Option<Range<usize>>,
    index_attribute: Option<Range<usize>>,
    entries: Vec<Range<usize>>,
}

struct Record {
    position: usize,
    index: u32,
    from: u32,
    to: u32,
}

fn records(editor: &ConfigEditor, selected: &SelectedEntry) -> Result<Vec<Record>> {
    let mut records = Vec::new();
    for (position, child) in editor.get_element(&selected.path).children.iter().enumerate() {
        let Some(entry) = child.as_element().filter(|entry| entry.name == "Entry") else {
            continue;
        };
        let record = Record {
            position,
            index: integer(entry, "Ix")?,
            from: integer(entry, "From")?,
            to: integer(entry, "To")?,
        };
        ensure!(
            records.last().is_none_or(|previous: &Record| {
                (previous.index, previous.from) < (record.index, record.from)
            }),
            "DayTimer gestures require original sorted Ix/From order; incidental native sorting is refused. Nothing was changed."
        );
        records.push(record);
    }
    Ok(records)
}

fn gesture_context(
    editor: &ConfigEditor,
    selected: &SelectedEntry,
) -> Result<(Vec<usize>, String)> {
    let (document_path, block_uuid) = editor.daytimer_boundary_context(selected)?;
    ensure!(
        document_path.is_empty()
            || (document_path.len() == 1
                && editor.root.name == "ControlList"
                && !editor.root.attributes.contains_key("Type")
                && editor.root.namespace.is_none()
                && editor.root.prefix.is_none()),
        "DayTimer gestures require a root Document or a direct Document under an exact untyped ControlList; nothing was changed."
    );
    let owner_depth = selected.path.len() - document_path.len();
    ensure!(
        matches!(owner_depth, 2 | 3),
        "DayTimer gestures require a direct native Page, optionally through one direct Program; nothing was changed."
    );
    for (distance, expected) in [(1, "Page"), (2, "Program")] {
        if distance == 2 && owner_depth != 3 {
            continue;
        }
        let parent = editor.get_element(&selected.path[..selected.path.len() - distance]);
        ensure!(
            parent.name == "C"
                && parent.attributes.get("Type").map(String::as_str) == Some(expected)
                && parent.namespace.is_none()
                && parent.prefix.is_none(),
            "DayTimer gestures require exact native C/Page and optional C/Program ancestry; nothing was changed."
        );
    }
    for depth in document_path.len()..=selected.path.len() {
        ensure!(
            !editor
                .get_element(&selected.path[..depth])
                .attributes
                .contains_key("IfT"),
            "DayTimer gestures do not emulate IfT instance state; nothing was changed."
        );
    }
    let block = editor.get_element(&selected.path);
    ensure!(
        block.name == "C",
        "DayTimer gestures require an exact native C/DayTimer; nothing was changed."
    );
    let elements = ConfigEditor::iter_elements(&editor.root);
    for identity in std::iter::once(block_uuid.as_str()).chain(block.attributes["Modes"].split(',')) {
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
    Ok((document_path, block_uuid))
}

fn gesture_storage(
    data: &[u8],
    editor: &ConfigEditor,
    selected: &SelectedEntry,
    document_path: &[usize],
) -> Result<GestureStorage> {
    let boundary = boundary_storage(data, editor, selected, document_path)?;
    let elements = ConfigEditor::iter_elements(&editor.root);
    let block = editor.get_element(&selected.path);
    let target = block.children[selected.position].as_element().unwrap();
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    let mut ordinal = 0;
    let mut count = None;
    let mut index = None;
    let mut index_attribute = None;
    let mut entries = Vec::new();
    loop {
        let start = reader.buffer_position() as usize + offset;
        let event = reader.read_event()?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(raw) | Event::Empty(raw) => {
                let element = elements[ordinal];
                let owner = std::ptr::eq(element, block);
                let selected_entry = std::ptr::eq(element, target);
                let entry = block.children.iter().filter_map(|node| node.as_element()).any(
                    |child| child.name == "Entry" && std::ptr::eq(child, element),
                );
                if entry {
                    ensure!(
                        empty,
                        "DayTimer gestures require empty Entry records; nested-entry storage is unsupported. Nothing was changed."
                    );
                    entries.push(start..reader.buffer_position() as usize + offset);
                }
                if owner || selected_entry {
                    for attribute in raw.attributes() {
                        let attribute = attribute?;
                        let key = attribute.key.as_ref();
                        if (owner && key == b"N") || (selected_entry && key == b"Ix") {
                            let start = (attribute.value.as_ptr() as usize)
                                .checked_sub(data.as_ptr() as usize)
                                .filter(|start| {
                                    data.get(*start..*start + attribute.value.len())
                                        == Some(attribute.value.as_ref())
                                })
                                .context("Cannot locate original DayTimer gesture attribute")?;
                            let value = start..start + attribute.value.len();
                            if owner {
                                count = Some(value);
                            } else {
                                let start = (attribute.key.as_ref().as_ptr() as usize)
                                    .checked_sub(data.as_ptr() as usize)
                                    .context("Cannot locate original DayTimer Ix")?;
                                index_attribute = Some(start..value.end + 1);
                                index = Some(value);
                            }
                        }
                    }
                }
                ordinal += 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(GestureStorage {
        boundary,
        count: count.context("Missing original DayTimer N; nothing was changed")?,
        index,
        index_attribute,
        entries,
    })
}

fn preceding_whitespace(data: &[u8], end: usize) -> &str {
    let start = data[..end]
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(0, |position| position + 1);
    std::str::from_utf8(&data[start..end]).unwrap()
}

impl ConfigEditor {
    pub fn copy_daytimer_entry(
        data: &[u8],
        selector: &str,
        identity: DayTimerEntryIdentity,
        shift_minutes: i32,
    ) -> Result<(Vec<u8>, DayTimerEntryCopy)> {
        let mut editor = Self::load(data)?;
        let selected = editor.select_daytimer_entry(selector, &identity)?;
        let (document_path, block_uuid) = gesture_context(&editor, &selected)?;
        let records = records(&editor, &selected)?;
        let storage = gesture_storage(data, &editor, &selected, &document_path)?;
        let source_previous_index = records
            .iter()
            .position(|record| record.position == selected.position)
            .unwrap();
        let source = &records[source_previous_index];
        let original_from = source.from as i32;
        let original_to = source.to as i32;
        ensure!(
            (-original_from - 1..=1440 - original_from).contains(&shift_minutes),
            "Copy shift must be a reachable timeline-minute delta from the From grab point ({}..={} minutes); nothing was changed.",
            -original_from - 1,
            1440 - original_from
        );
        let mut from = original_from + shift_minutes;
        let mut to = original_to + shift_minutes;
        if from < 0 {
            from = 0;
            to = original_to - original_from;
        } else if to > 1440 {
            to = 1440;
            from = 1440 - (original_to - original_from);
        }
        let (from, to) = (from as u32, to as u32);
        ensure!(
            records.iter().all(|other| {
                other.index != source.index || to < other.from || other.to < from
            }),
            "The copied interval would touch or overlap an existing entry; native merging is unsupported. Nothing was changed."
        );
        ensure!(
            selected.entry_count < i32::MAX as usize / 40,
            "DayTimer record buffer cannot grow within the supported native range; nothing was changed."
        );
        let block = editor.get_element(&selected.path);
        let entry = block.children[selected.position].as_element().unwrap();
        let activation_required = match entry.attributes.get("NeedActivate").map(String::as_str) {
            None => false,
            Some(raw) if raw.eq_ignore_ascii_case("true") => true,
            Some(raw) if raw.eq_ignore_ascii_case("false") => false,
            Some(_) => bail!(
                "Unsupported DayTimer NeedActivate spelling for copying; nothing was changed."
            ),
        };
        // 080650 appends the owned record, not a clone of its original XML node.
        let mut new_entry = String::from("<Entry");
        if from != 0 {
            new_entry.push_str(&format!(" From=\"{from}\""));
        }
        new_entry.push_str(&format!(" To=\"{to}\""));
        if source.index != 0 {
            new_entry.push_str(&format!(" Ix=\"{}\"", source.index));
        }
        if activation_required {
            new_entry.push_str(" NeedActivate=\"true\"");
        }
        new_entry.push_str(" V=\"1\"/>");
        let copy_index = records
            .iter()
            .position(|other| (source.index, from) < (other.index, other.from))
            .unwrap_or(records.len());
        let (insert, fragment) = if let Some(next) = storage.entries.get(copy_index) {
            (
                next.start,
                format!("{new_entry}{}", preceding_whitespace(data, next.start)),
            )
        } else {
            let last = storage.entries.last().unwrap();
            (
                last.end,
                format!("{}{new_entry}", preceding_whitespace(data, last.start)),
            )
        };
        let entry_count = selected.entry_count + 1;
        let change = DayTimerEntryCopy {
            block_uuid,
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            shift_minutes,
            new_from_minutes: from,
            new_to_minutes: to,
            activation_required,
            source_previous_index,
            copy_index,
            entry_count,
        };
        let output = editor.patch_daytimer_boundary(
            data,
            &document_path,
            &storage.boundary,
            vec![
                (storage.count, entry_count.to_string()),
                (insert..insert, fragment),
            ],
        )?;
        Ok((output, change))
    }

    pub fn move_daytimer_entry_mode(
        data: &[u8],
        selector: &str,
        identity: DayTimerEntryIdentity,
        target_mode_uuid: &str,
    ) -> Result<(Vec<u8>, DayTimerEntryModeMove)> {
        let target_key = mode_uuid_key(target_mode_uuid)
            .context("--target-mode-uuid must be a non-nil native or RFC mode UUID")?;
        let mut editor = Self::load(data)?;
        let selected = editor.select_daytimer_entry(selector, &identity)?;
        let (document_path, block_uuid) = gesture_context(&editor, &selected)?;
        let records = records(&editor, &selected)?;
        let storage = gesture_storage(data, &editor, &selected, &document_path)?;
        let block = editor.get_element(&selected.path);
        let (target_index, stored_target_mode_uuid) = block.attributes["Modes"]
            .split(',')
            .enumerate()
            .find(|(_, mode)| mode_uuid_key(mode).as_ref() == Some(&target_key))
            .context("Target operating mode is not already in this DayTimer's Modes; nothing was changed")?;
        let target_index = target_index as u32;
        let source = records
            .iter()
            .find(|record| record.position == selected.position)
            .unwrap();
        let changed = source.index != target_index;
        ensure!(
            !changed || source.from % 15 == 0,
            "Native cross-mode movement would snap this off-grid From and may change To; endpoint-preserving movement requires From on the 15-minute grid. Nothing was changed."
        );
        let mut previous_key = None;
        for record in &records {
            let key = (
                if record.position == selected.position {
                    target_index
                } else {
                    record.index
                },
                record.from,
            );
            ensure!(
                previous_key.is_none_or(|previous| previous < key),
                "Cross-mode movement would change physical rank; incidental native sorting is refused. Nothing was changed."
            );
            previous_key = Some(key);
            ensure!(
                record.position == selected.position
                    || record.index != target_index
                    || source.to < record.from
                    || record.to < source.from,
                "Cross-mode movement would touch or overlap a destination entry; native merging is unsupported. Nothing was changed."
            );
        }
        let change = DayTimerEntryModeMove {
            block_uuid,
            entry: identity,
            stored_mode_uuid: selected.stored_mode_uuid,
            target_mode_uuid: target_mode_uuid.to_owned(),
            stored_target_mode_uuid: stored_target_mode_uuid.to_owned(),
            previous_mode_index: source.index,
            target_mode_index: target_index,
            changed,
        };
        if !changed {
            return Ok((data.to_vec(), change));
        }
        let patch = if target_index == 0 {
            (
                storage.index_attribute.context("Missing original DayTimer Ix")?,
                String::new(),
            )
        } else if let Some(index) = storage.index {
            (index, target_index.to_string())
        } else {
            (
                storage.boundary.entry_insert..storage.boundary.entry_insert,
                format!(" Ix=\"{target_index}\""),
            )
        };
        let output =
            editor.patch_daytimer_boundary(data, &document_path, &storage.boundary, vec![patch])?;
        Ok((output, change))
    }
}
