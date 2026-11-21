use super::{EditError, legacy::State};
use chrono::{DateTime, FixedOffset};
use std::{collections::BTreeMap, error::Error, fmt};
use xmltree::Element;

#[cfg(any(feature = "configuration-xml", feature = "unstable-legacy-cli-editor"))]
pub(super) mod activation;

/// Failure to remove one existing ordinary digital DayTimer entry.
#[derive(Debug)]
#[non_exhaustive]
pub enum DayTimerRemovalError {
    Edit(EditError),
    InvalidModeUuid,
    InvalidInterval,
    NotDayTimer {
        selector: String,
    },
    UnsupportedScheduleMode,
    AnalogSchedule,
    UnsupportedAnalog,
    MissingModes,
    InvalidModes,
    DuplicateModes,
    UnresolvedMode {
        stored_mode_uuid: String,
    },
    ModeNotInSchedule,
    InvalidInteger {
        attribute: &'static str,
        value: String,
    },
    EntryCountMismatch,
    InvalidStoredInterval,
    TouchingOrOverlappingEntries,
    EntryNotFound,
    OutsidePage,
    NamespacedEntry,
}

impl fmt::Display for DayTimerRemovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::InvalidModeUuid => {
                f.write_str("Invalid operating mode UUID; nothing was changed.")
            }
            Self::InvalidInterval => f.write_str(
                "Entry times must satisfy 00:00 <= from < to <= 24:00; nothing was changed.",
            ),
            Self::NotDayTimer { selector } => {
                write!(f, "'{selector}' is not a DayTimer; nothing was changed.")
            }
            Self::UnsupportedScheduleMode => f.write_str(
                "Only ordinary DayTimer M=0 schedules are supported; nothing was changed.",
            ),
            Self::AnalogSchedule => f.write_str(
                "Analog DayTimer entry editing is not yet qualified; this operation supports digital schedules only. Nothing was changed.",
            ),
            Self::UnsupportedAnalog => {
                f.write_str("Unsupported DayTimer Analog spelling; nothing was changed.")
            }
            Self::MissingModes => {
                f.write_str("DayTimer has no explicit Modes references; nothing was changed.")
            }
            Self::InvalidModes => f.write_str(
                "DayTimer Modes requires comma-separated non-nil native 8-4-4-16 or RFC 8-4-4-4-12 UUIDs without whitespace; nothing was changed.",
            ),
            Self::DuplicateModes => {
                f.write_str("DayTimer Modes contains duplicate references; nothing was changed.")
            }
            Self::UnresolvedMode { stored_mode_uuid } => write!(
                f,
                "DayTimer operating mode '{stored_mode_uuid}' is missing, ambiguous or not a Mode; nothing was changed."
            ),
            Self::ModeNotInSchedule => f.write_str(
                "Requested operating mode is not in this DayTimer's Modes; nothing was changed.",
            ),
            Self::InvalidInteger { attribute, value } => write!(
                f,
                "Unsupported DayTimer {attribute}='{value}': expected a nonnegative signed-32-bit integer; nothing was changed."
            ),
            Self::EntryCountMismatch => f.write_str(
                "DayTimer N does not match stored entries; refusing to repair counts or interpret legacy timer-schedule storage.",
            ),
            Self::InvalidStoredInterval => f.write_str(
                "DayTimer contains an invalid interval or unresolved Ix; nothing was changed.",
            ),
            Self::TouchingOrOverlappingEntries => f.write_str(
                "Overlapping or touching entries in one mode may require native normalization; this bounded edit refuses them without changing anything.",
            ),
            Self::EntryNotFound => f.write_str(
                "No existing entry matches that operating mode and exact interval; nothing was changed.",
            ),
            Self::OutsidePage => f.write_str(
                "DayTimer removal requires a direct Page child in one stampable Document; nothing was changed.",
            ),
            Self::NamespacedEntry => f.write_str(
                "DayTimer removal does not admit prefixed or namespaced direct Entry elements; nothing was changed.",
            ),
        }
    }
}

impl Error for DayTimerRemovalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            _ => None,
        }
    }
}

/// Identity and stored data of the one removed record. UUID spellings are not normalized.
#[derive(Debug, PartialEq, Eq)]
pub struct DayTimerEntryRemoval {
    pub block_uuid: String,
    pub requested_mode_uuid: String,
    pub stored_mode_uuid: String,
    pub from_minutes: u32,
    pub to_minutes: u32,
    pub removed_attributes: BTreeMap<String, String>,
    pub remaining_entries: usize,
}

#[derive(Debug)]
pub struct DayTimerRemovalOutcome {
    pub bytes: Vec<u8>,
    pub removal: DayTimerEntryRemoval,
}

/// Remove one existing direct `Entry` from an ordinary digital (`M=0`) DayTimer.
///
/// The legacy selector identifies the block unchanged (including exact spelling
/// for `uuid:`). The mode ID is matched case-insensitively across the existing
/// non-nil native 8-4-4-16 and RFC 8-4-4-4-12 spellings. The requested mode and
/// exact interval, in minutes, identify the record; XML order is not identity.
///
/// Only a direct child of a plain `C Type="Page"` is admitted, under the sole
/// stampable `C Type="Document"` either directly or via `C Type="Program"`.
/// A prefixed or namespaced direct `Entry` is refused even when it is not the
/// selected record. The operation validates the stored count, mode references
/// and intervals before mutating, then updates `N` or omits it when no records
/// remain. Other records are not reordered, reindexed, or normalized.
///
/// Success returns the complete canonically serialized edit with the Document
/// stamp advanced by the shared writer using `now`. Failure returns no bytes.
/// This performs no filesystem I/O and is not a general XML byte-preservation
/// or native Config validation API; the loader limitation lox-n37w still applies.
pub fn remove_daytimer_entry(
    bytes: &[u8],
    selector: &str,
    mode_uuid: &str,
    from_minutes: u32,
    to_minutes: u32,
    now: &DateTime<FixedOffset>,
) -> Result<DayTimerRemovalOutcome, DayTimerRemovalError> {
    let mut state = State::load(bytes).map_err(DayTimerRemovalError::Edit)?;
    let selected = state.select_daytimer_entry(selector, mode_uuid, from_minutes, to_minutes)?;
    if !state.daytimer_is_page_child(&selected.path) {
        return Err(DayTimerRemovalError::OutsidePage);
    }
    if state
        .get_element(&selected.path)
        .children
        .iter()
        .filter_map(|child| child.as_element())
        .any(|child| {
            child.name == "Entry"
                && (child.prefix.is_some()
                    || child
                        .namespace
                        .as_deref()
                        .is_some_and(|uri| !uri.is_empty()))
        })
    {
        return Err(DayTimerRemovalError::NamespacedEntry);
    }
    let removal =
        state.remove_selected_daytimer_entry(selected, mode_uuid, from_minutes, to_minutes);
    if !state.stamp_document(now) {
        return Err(DayTimerRemovalError::OutsidePage);
    }
    let bytes = state.to_bytes().map_err(DayTimerRemovalError::Edit)?;
    Ok(DayTimerRemovalOutcome { bytes, removal })
}

#[doc(hidden)]
pub struct SelectedDayTimerEntry {
    pub path: Vec<usize>,
    pub position: usize,
    pub stored_mode_uuid: String,
    entry_count: usize,
}

#[doc(hidden)]
pub fn daytimer_integer(element: &Element, key: &'static str) -> Result<u32, DayTimerRemovalError> {
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
    Err(DayTimerRemovalError::InvalidInteger {
        attribute: key,
        value: raw.clone(),
    })
}

#[doc(hidden)]
pub fn daytimer_mode_uuid_key(raw: &str) -> Option<String> {
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

fn validate_entry_identity(
    mode_uuid: &str,
    from_minutes: u32,
    to_minutes: u32,
) -> Result<String, DayTimerRemovalError> {
    let mode_key =
        daytimer_mode_uuid_key(mode_uuid).ok_or(DayTimerRemovalError::InvalidModeUuid)?;
    if from_minutes >= to_minutes || to_minutes > 1440 {
        return Err(DayTimerRemovalError::InvalidInterval);
    }
    Ok(mode_key)
}

impl State {
    fn daytimer_is_page_child(&self, path: &[usize]) -> bool {
        let is_plain = |element: &Element, kind: &str| {
            element.name == "C"
                && element.attributes.get("Type").map(String::as_str) == Some(kind)
                && element.namespace.is_none()
                && element.prefix.is_none()
        };
        let documents = self.find_elements("Type:Document");
        let [document] = documents.as_slice() else {
            return false;
        };
        if document.len() > 1 || !is_plain(self.get_element(document), "Document") {
            return false;
        }
        let depth = document.len();
        (match path.strip_prefix(document.as_slice()) {
            Some([_, _]) => is_plain(self.get_element(&path[..depth + 1]), "Page"),
            Some([_, _, _]) => {
                is_plain(self.get_element(&path[..depth + 1]), "Program")
                    && is_plain(self.get_element(&path[..depth + 2]), "Page")
            }
            _ => false,
        }) && is_plain(self.get_element(path), "DayTimer")
    }

    #[doc(hidden)]
    pub fn select_daytimer_entry(
        &self,
        selector: &str,
        mode_uuid: &str,
        from_minutes: u32,
        to_minutes: u32,
    ) -> Result<SelectedDayTimerEntry, DayTimerRemovalError> {
        let mode_key = validate_entry_identity(mode_uuid, from_minutes, to_minutes)?;
        self.select_daytimer_entry_by_mode_key(selector, &mode_key, from_minutes, to_minutes)
    }

    fn select_daytimer_entry_by_mode_key(
        &self,
        selector: &str,
        mode_key: &str,
        from_minutes: u32,
        to_minutes: u32,
    ) -> Result<SelectedDayTimerEntry, DayTimerRemovalError> {
        use DayTimerRemovalError as Failure;
        let path = self
            .require_one(selector)
            .map_err(|error| Failure::Edit(error.into()))?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("DayTimer") {
            return Err(Failure::NotDayTimer {
                selector: selector.to_owned(),
            });
        }
        if daytimer_integer(block, "M")? != 0 {
            return Err(Failure::UnsupportedScheduleMode);
        }
        match block.attributes.get("Analog").map(String::as_str) {
            None | Some("false") => {}
            Some("true") => return Err(Failure::AnalogSchedule),
            _ => return Err(Failure::UnsupportedAnalog),
        }
        let stored_modes: Vec<_> = block
            .attributes
            .get("Modes")
            .ok_or(Failure::MissingModes)?
            .split(',')
            .collect();
        let modes = stored_modes
            .iter()
            .map(|raw| daytimer_mode_uuid_key(raw))
            .collect::<Option<Vec<_>>>()
            .ok_or(Failure::InvalidModes)?;
        for (index, mode) in modes.iter().enumerate() {
            if modes[..index].contains(mode) {
                return Err(Failure::DuplicateModes);
            }
            let matches: Vec<_> = Self::iter_elements(&self.root)
                .into_iter()
                .filter(|element| {
                    element.name == "C"
                        && element
                            .attributes
                            .get("U")
                            .and_then(|raw| daytimer_mode_uuid_key(raw))
                            .as_ref()
                            == Some(mode)
                })
                .collect();
            if matches.len() != 1
                || matches[0].attributes.get("Type").map(String::as_str) != Some("Mode")
            {
                return Err(Failure::UnresolvedMode {
                    stored_mode_uuid: stored_modes[index].to_owned(),
                });
            }
        }
        let mode_index = modes
            .iter()
            .position(|mode| mode.as_str() == mode_key)
            .ok_or(Failure::ModeNotInSchedule)?;
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
        if daytimer_integer(block, "N")? as usize != positions.len() {
            return Err(Failure::EntryCountMismatch);
        }
        let mut records = Vec::new();
        for position in &positions {
            let entry = block.children[*position].as_element().unwrap();
            let from = daytimer_integer(entry, "From")?;
            let to = daytimer_integer(entry, "To")?;
            let index = daytimer_integer(entry, "Ix")? as usize;
            if from >= to || to > 1440 || index >= modes.len() {
                return Err(Failure::InvalidStoredInterval);
            }
            if records.iter().any(|&(other_index, other_from, other_to)| {
                index == other_index && from <= other_to && other_from <= to
            }) {
                return Err(Failure::TouchingOrOverlappingEntries);
            }
            records.push((index, from, to));
        }
        let selected = records
            .iter()
            .position(|record| *record == (mode_index, from_minutes, to_minutes))
            .ok_or(Failure::EntryNotFound)?;
        Ok(SelectedDayTimerEntry {
            stored_mode_uuid: stored_modes[mode_index].to_owned(),
            entry_count: positions.len(),
            path,
            position: positions[selected],
        })
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn remove_daytimer_entry_legacy(
        &mut self,
        selector: &str,
        mode_uuid: &str,
        from_minutes: u32,
        to_minutes: u32,
    ) -> Result<DayTimerEntryRemoval, DayTimerRemovalError> {
        let selected = self.select_daytimer_entry(selector, mode_uuid, from_minutes, to_minutes)?;
        Ok(self.remove_selected_daytimer_entry(selected, mode_uuid, from_minutes, to_minutes))
    }

    fn remove_selected_daytimer_entry(
        &mut self,
        selected: SelectedDayTimerEntry,
        mode_uuid: &str,
        from_minutes: u32,
        to_minutes: u32,
    ) -> DayTimerEntryRemoval {
        let block = self.get_element_mut(&selected.path);
        let removed = block.children.remove(selected.position);
        let removal = DayTimerEntryRemoval {
            block_uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            requested_mode_uuid: mode_uuid.to_owned(),
            stored_mode_uuid: selected.stored_mode_uuid,
            from_minutes,
            to_minutes,
            removed_attributes: removed
                .as_element()
                .unwrap()
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            remaining_entries: selected.entry_count - 1,
        };
        // Native N counts serialized non-default records and omits zero.
        if removal.remaining_entries == 0 {
            block.attributes.shift_remove("N");
        } else {
            block
                .attributes
                .insert("N".into(), removal.remaining_entries.to_string());
        }
        removal
    }
}
