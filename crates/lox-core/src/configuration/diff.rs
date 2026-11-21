//! Structural-summary comparison, not a full XML or configuration-semantic diff.

use super::summary::{ConfigSummary, ControlEntry};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDiff {
    pub version_old: String,
    pub version_new: String,
    pub date_old: String,
    pub date_new: String,
    pub controls_added: Vec<ControlRecord>,
    pub controls_removed: Vec<ControlRecord>,
    pub controls_changed: Vec<ControlChange>,
    pub rooms_added: Vec<NamedEntry>,
    pub rooms_removed: Vec<NamedEntry>,
    pub rooms_renamed: Vec<NameChange>,
    pub categories_added: Vec<NamedEntry>,
    pub categories_removed: Vec<NamedEntry>,
    pub categories_renamed: Vec<NameChange>,
    pub users_added: Vec<String>,
    pub users_removed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlRecord {
    pub uuid: String,
    pub entry: ControlEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlChange {
    pub uuid: String,
    /// The control's name in the new summary, for every changed field.
    pub name: String,
    pub field: ControlFieldChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFieldChange {
    Name {
        old: String,
        new: String,
    },
    Type {
        old: String,
        new: String,
    },
    Room {
        old: RoomReference,
        new: RoomReference,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomReference {
    /// Raw relationship value; an empty string represents no assigned room.
    pub uuid: String,
    /// Title from that side's room map, if present (including an empty title).
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedEntry {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameChange {
    pub uuid: String,
    pub old: String,
    pub new: String,
}

impl ConfigDiff {
    /// Metadata is carried through but does not count as a structural change.
    pub fn has_changes(&self) -> bool {
        !self.controls_added.is_empty()
            || !self.controls_removed.is_empty()
            || !self.controls_changed.is_empty()
            || !self.rooms_added.is_empty()
            || !self.rooms_removed.is_empty()
            || !self.rooms_renamed.is_empty()
            || !self.categories_added.is_empty()
            || !self.categories_removed.is_empty()
            || !self.categories_renamed.is_empty()
            || !self.users_added.is_empty()
            || !self.users_removed.is_empty()
    }
}

/// Compare exactly the fields represented by two summaries, without I/O.
///
/// Controls, rooms and categories match by raw UUID, not title. A control's
/// name, type and room UUID are compared in that order. Renaming a room without
/// changing its UUID does not produce a control-room change. User titles compare
/// as sets: order and duplicate counts do not matter. Metadata changes alone do
/// not make [`ConfigDiff::has_changes`] true. Map/set iteration order remains
/// unspecified; no sorting or additional XML interpretation is performed.
pub fn diff_configs(old: &ConfigSummary, new: &ConfigSummary) -> ConfigDiff {
    let mut diff = ConfigDiff {
        version_old: old.version.clone(),
        version_new: new.version.clone(),
        date_old: old.date.clone(),
        date_new: new.date.clone(),
        controls_added: Vec::new(),
        controls_removed: Vec::new(),
        controls_changed: Vec::new(),
        rooms_added: Vec::new(),
        rooms_removed: Vec::new(),
        rooms_renamed: Vec::new(),
        categories_added: Vec::new(),
        categories_removed: Vec::new(),
        categories_renamed: Vec::new(),
        users_added: Vec::new(),
        users_removed: Vec::new(),
    };

    for (uuid, entry) in &new.controls {
        match old.controls.get(uuid) {
            None => diff.controls_added.push(ControlRecord {
                uuid: uuid.clone(),
                entry: entry.clone(),
            }),
            Some(old_entry) => {
                let mut changed = |field| {
                    diff.controls_changed.push(ControlChange {
                        uuid: uuid.clone(),
                        name: entry.name.clone(),
                        field,
                    });
                };
                if old_entry.name != entry.name {
                    changed(ControlFieldChange::Name {
                        old: old_entry.name.clone(),
                        new: entry.name.clone(),
                    });
                }
                if old_entry.control_type != entry.control_type {
                    changed(ControlFieldChange::Type {
                        old: old_entry.control_type.clone(),
                        new: entry.control_type.clone(),
                    });
                }
                if old_entry.room_uuid != entry.room_uuid {
                    changed(ControlFieldChange::Room {
                        old: RoomReference {
                            uuid: old_entry.room_uuid.clone(),
                            name: old.rooms.get(&old_entry.room_uuid).cloned(),
                        },
                        new: RoomReference {
                            uuid: entry.room_uuid.clone(),
                            name: new.rooms.get(&entry.room_uuid).cloned(),
                        },
                    });
                }
            }
        }
    }
    for (uuid, entry) in &old.controls {
        if !new.controls.contains_key(uuid) {
            diff.controls_removed.push(ControlRecord {
                uuid: uuid.clone(),
                entry: entry.clone(),
            });
        }
    }

    diff_named_map(
        &old.rooms,
        &new.rooms,
        &mut diff.rooms_added,
        &mut diff.rooms_removed,
        &mut diff.rooms_renamed,
    );
    diff_named_map(
        &old.categories,
        &new.categories,
        &mut diff.categories_added,
        &mut diff.categories_removed,
        &mut diff.categories_renamed,
    );

    let old_users: HashSet<&String> = old.users.iter().collect();
    let new_users: HashSet<&String> = new.users.iter().collect();
    for user in &new_users {
        if !old_users.contains(user) {
            diff.users_added.push((*user).clone());
        }
    }
    for user in &old_users {
        if !new_users.contains(user) {
            diff.users_removed.push((*user).clone());
        }
    }
    diff
}

fn diff_named_map(
    old: &HashMap<String, String>,
    new: &HashMap<String, String>,
    added: &mut Vec<NamedEntry>,
    removed: &mut Vec<NamedEntry>,
    renamed: &mut Vec<NameChange>,
) {
    for (uuid, name) in new {
        match old.get(uuid) {
            None => added.push(NamedEntry {
                uuid: uuid.clone(),
                name: name.clone(),
            }),
            Some(old_name) if old_name != name => renamed.push(NameChange {
                uuid: uuid.clone(),
                old: old_name.clone(),
                new: name.clone(),
            }),
            _ => {}
        }
    }
    for (uuid, name) in old {
        if !new.contains_key(uuid) {
            removed.push(NamedEntry {
                uuid: uuid.clone(),
                name: name.clone(),
            });
        }
    }
}
