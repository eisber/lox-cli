use std::collections::BTreeMap;

use anyhow::{Result, bail};
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

impl ConfigEditor {
    pub fn remove_daytimer_entry(
        &mut self,
        selector: &str,
        identity: DayTimerEntryIdentity,
    ) -> Result<DayTimerEntryRemoval> {
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
                "Analog DayTimer entry deletion is not yet qualified; this operation supports digital schedules only. Nothing was changed."
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
                    "Overlapping or touching entries in one mode may require native normalization; this bounded delete refuses them without changing anything."
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
        let removal = DayTimerEntryRemoval {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            entry: identity,
            stored_mode_uuid: stored_modes[mode_index].to_owned(),
            removed_attributes: block.children[position]
                .as_element()
                .unwrap()
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            remaining_entries: positions.len() - 1,
        };
        let block = self.get_element_mut(&path);
        block.children.remove(position);
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
}
