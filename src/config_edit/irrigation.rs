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

fn native_owner_uuid(raw: &str) -> Option<uuid::Uuid> {
    if raw.len() != 35 {
        return None;
    }
    super::icons::native_identity(raw).filter(|uuid| !uuid.is_nil())
}

impl ConfigEditor {
    /// Rename an already named zone without opening the dialog's empty-name autofill path.
    pub fn edit_irrigation_zone_name(
        data: &[u8],
        selector: &str,
        io_index: u8,
        name: &str,
    ) -> Result<(Vec<u8>, IrrigationZoneNameChange)> {
        let requested_owner = selector.strip_prefix("uuid:").unwrap_or("");
        let Some(requested_identity) = native_owner_uuid(requested_owner) else {
            bail!(
                "Irrigation zone naming requires uuid:<exact-stored-native35-U>, using nonnil 8-4-4-16 grouping; RFC/name/room selectors are not supported. Nothing was changed."
            );
        };
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
        let documents = editor.find_elements("Type:Document");
        if documents.len() != 1 || documents[0].len() > 1 {
            bail!(
                "Irrigation zone naming requires exactly one stampable Document at the root or directly under it; nothing was changed."
            );
        }
        let document_path = &documents[0];
        let document = editor.get_element(document_path);
        if document.name != "C"
            || document.attributes.get("Type").map(String::as_str) != Some("Document")
            || document.namespace.is_some()
            || document.prefix.is_some()
        {
            bail!(
                "Irrigation zone naming requires a plain stampable Document; nothing was changed."
            );
        }
        if document
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .any(|child| child.name == "Template")
        {
            bail!(
                "Irrigation zone naming does not support any direct Document Template child: native template locking and accepted-edit propagation are not implemented. Nothing was changed."
            );
        }
        let matches = editor.find_elements(selector);
        if matches.len() != 1 || !matches[0].starts_with(document_path) {
            bail!(
                "Irrigation zone naming requires exactly one selected C object inside the owning Document; duplicate or missing U identities are not supported. Nothing was changed."
            );
        }
        let equivalent_owners = Self::iter_elements(&editor.root)
            .into_iter()
            .filter(|element| element.name == "C")
            .filter(|element| {
                element
                    .attributes
                    .get("U")
                    .and_then(|raw| super::icons::native_identity(raw))
                    == Some(requested_identity)
            })
            .count();
        if equivalent_owners != 1 {
            bail!(
                "Irrigation zone naming requires a unique effective owner UUID across all C objects, including case/grouping aliases; nothing was changed."
            );
        }
        let path = editor.require_one(selector)?;
        let block = editor.get_element(&path);
        if block.name != "C"
            || block.attributes.get("Type").map(String::as_str) != Some("Irrigation")
        {
            bail!("'{selector}' is not an Irrigation block; nothing was changed.");
        }
        let block_uuid = block
            .attributes
            .get("U")
            .filter(|stored| {
                stored.as_str() == requested_owner
                    && native_owner_uuid(stored) == Some(requested_identity)
            })
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Irrigation stored owner U must be nonnil native35 (8-4-4-16) and match the exact selector; nothing was changed."
                )
            })?;
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
        let stamp = if change.changed {
            if !editor.stamp_document(&chrono::Local::now().fixed_offset()) {
                bail!("Cannot stamp the admitted Irrigation Document; nothing was changed.");
            }
            let document = editor.get_element(document_path);
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
