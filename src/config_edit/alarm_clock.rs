use std::collections::{BTreeMap, HashSet};

use anyhow::{Result, bail};
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;

#[derive(Debug, Serialize)]
pub struct AlarmClockEntryRemoval {
    pub block_uuid: String,
    pub entry_uuid: String,
    pub stored_entry_uuid: String,
    pub removed_attributes: BTreeMap<String, String>,
    pub remaining_entries: usize,
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

impl ConfigEditor {
    pub fn remove_alarm_clock_entry(
        &mut self,
        selector: &str,
        entry_uuid: &str,
    ) -> Result<AlarmClockEntryRemoval> {
        let requested = uuid_key(entry_uuid)?;
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.name != "C"
            || block.attributes.get("Type").map(String::as_str) != Some("AlarmClock")
        {
            bail!("'{selector}' is not an AlarmClock; nothing was changed.");
        }
        if integer(block, "V")? != 175 {
            bail!(
                "Alarm entry deletion supports V=175 storage only; legacy/default migration is not qualified. Nothing was changed."
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
            if key == requested {
                if internal {
                    bail!(
                        "The internal standard alarm (Int=true) cannot be deleted in the native dialog; nothing was changed."
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
        let position = selected.ok_or_else(|| {
            anyhow::anyhow!(
                "No configured alarm entry matches UUID '{entry_uuid}'; nothing was changed."
            )
        })?;
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
            remaining_entries: entries.len() - 1,
        };
        let block = self.get_element_mut(&path);
        block.children.remove(position);
        block
            .attributes
            .insert("N".into(), removal.remaining_entries.to_string());
        Ok(removal)
    }
}
