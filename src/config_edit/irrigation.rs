use anyhow::{Result, bail};
use serde::Serialize;
use xmltree::XMLNode;

use super::{
    ConfigEditor,
    alarm_clock::{EntryNameStorage, entry_name_bytes, entry_name_ordinal},
};

#[derive(Debug, Serialize)]
pub struct IrrigationZoneNameChange {
    pub block_uuid: String,
    pub io_index: u8,
    pub previous_name: String,
    pub name: String,
    pub changed: bool,
}

impl ConfigEditor {
    /// Rename an already named zone without opening the dialog's empty-name autofill path.
    pub fn edit_irrigation_zone_name(
        data: &[u8],
        selector: &str,
        io_index: u8,
        name: &str,
    ) -> Result<(Vec<u8>, IrrigationZoneNameChange)> {
        if selector
            .strip_prefix("uuid:")
            .and_then(super::icons::native_identity)
            .filter(|uuid| !uuid.is_nil())
            .is_none()
        {
            bail!(
                "Irrigation zone naming requires uuid:<nonnil-stored-block-U>; name/room selectors are not qualified against original XML. Nothing was changed."
            );
        }
        if io_index > 7 {
            bail!(
                "Irrigation io-index must be 0..7, not a 1-based display row; nothing was changed."
            );
        }
        if name.trim().is_empty()
            || name.chars().any(|character| {
                character.is_control()
                    || matches!(character, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')
            })
        {
            bail!(
                "Irrigation zone name: this CLI supports nonempty, nonblank single-line XML text only; nothing was changed."
            );
        }
        let mut editor = Self::load(data)?;
        let path = editor.require_one(selector)?;
        let block = editor.get_element(&path);
        if block.name != "C"
            || block.attributes.get("Type").map(String::as_str) != Some("Irrigation")
        {
            bail!("'{selector}' is not an Irrigation block; nothing was changed.");
        }
        let zones: Vec<_> = block
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|child| child.name == "zone")
            .collect();
        if zones.len() != 8 {
            bail!(
                "Irrigation zone naming requires all eight existing zone records in ioIdx order 0..7; missing/default/migrated layouts are not qualified. Nothing was changed."
            );
        }
        let owner = entry_name_ordinal(&editor.root, block, &mut 0).unwrap();
        let block_uuid = block.attributes["U"].clone();
        let mut selected = None;
        for (index, zone) in zones.iter().enumerate() {
            let identity = index.to_string();
            let stored_identity = zone.attributes.get("ioIdx");
            if stored_identity != Some(&identity) && !(index == 0 && stored_identity.is_none()) {
                bail!(
                    "Irrigation zone records must have canonical ioIdx 0..7 in storage order (only zero may be omitted), with no duplicates; nothing was changed."
                );
            }
            let ordinal = entry_name_ordinal(&editor.root, zone, &mut 0).unwrap();
            let (_, previous) = entry_name_bytes(
                data,
                ordinal,
                owner,
                EntryNameStorage::Irrigation {
                    io_index: stored_identity.map(String::as_str),
                    block_uuid: &block_uuid,
                },
                None,
                None,
            )?;
            let previous = previous.filter(|name| !name.is_empty()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Every irrigation zone must already have a nonempty nm. Native dialog opening can fill empty names from connected outputs; that path is not qualified. Nothing was changed."
                )
            })?;
            if index == usize::from(io_index) {
                selected = Some((ordinal, previous, stored_identity.cloned()));
            }
        }
        let (ordinal, previous_name, stored_identity) = selected.unwrap();
        let change = IrrigationZoneNameChange {
            block_uuid,
            io_index,
            changed: previous_name != name,
            previous_name,
            name: name.into(),
        };
        let stamp = if change.changed && editor.stamp_document(&chrono::Local::now().fixed_offset())
        {
            let document = std::iter::once(&editor.root)
                .chain(editor.root.children.iter().filter_map(XMLNode::as_element))
                .find(|element| {
                    element.name == "C"
                        && element.attributes.get("Type").map(String::as_str) == Some("Document")
                })
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
            EntryNameStorage::Irrigation {
                io_index: stored_identity.as_deref(),
                block_uuid: &change.block_uuid,
            },
            change.changed.then_some(name),
            stamp,
        )?;
        Ok((output, change))
    }
}
