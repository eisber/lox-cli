//! Owned structural-summary comparison; no XML interpretation or comparison here.

use crate::configuration::{ConfigControlEntry, ConfigSummary};
use lox_core::configuration::diff as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigRoomReference {
    pub uuid: String,
    pub name: Option<String>,
}

impl From<core::RoomReference> for ConfigRoomReference {
    fn from(value: core::RoomReference) -> Self {
        Self {
            uuid: value.uuid,
            name: value.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ConfigControlFieldChange {
    Name {
        old: String,
        new: String,
    },
    Type {
        old: String,
        new: String,
    },
    Room {
        old: ConfigRoomReference,
        new: ConfigRoomReference,
    },
}

impl From<core::ControlFieldChange> for ConfigControlFieldChange {
    fn from(value: core::ControlFieldChange) -> Self {
        match value {
            core::ControlFieldChange::Name { old, new } => Self::Name { old, new },
            core::ControlFieldChange::Type { old, new } => Self::Type { old, new },
            core::ControlFieldChange::Room { old, new } => Self::Room {
                old: old.into(),
                new: new.into(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigControlRecord {
    pub uuid: String,
    pub entry: ConfigControlEntry,
}

impl From<core::ControlRecord> for ConfigControlRecord {
    fn from(value: core::ControlRecord) -> Self {
        Self {
            uuid: value.uuid,
            entry: value.entry.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigControlChange {
    pub uuid: String,
    pub name: String,
    pub field: ConfigControlFieldChange,
}

impl From<core::ControlChange> for ConfigControlChange {
    fn from(value: core::ControlChange) -> Self {
        Self {
            uuid: value.uuid,
            name: value.name,
            field: value.field.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigNamedEntry {
    pub uuid: String,
    pub name: String,
}

impl From<core::NamedEntry> for ConfigNamedEntry {
    fn from(value: core::NamedEntry) -> Self {
        Self {
            uuid: value.uuid,
            name: value.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigNameChange {
    pub uuid: String,
    pub old: String,
    pub new: String,
}

impl From<core::NameChange> for ConfigNameChange {
    fn from(value: core::NameChange) -> Self {
        Self {
            uuid: value.uuid,
            old: value.old,
            new: value.new,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigDiff {
    pub version_old: String,
    pub version_new: String,
    pub date_old: String,
    pub date_new: String,
    pub controls_added: Vec<ConfigControlRecord>,
    pub controls_removed: Vec<ConfigControlRecord>,
    pub controls_changed: Vec<ConfigControlChange>,
    pub rooms_added: Vec<ConfigNamedEntry>,
    pub rooms_removed: Vec<ConfigNamedEntry>,
    pub rooms_renamed: Vec<ConfigNameChange>,
    pub categories_added: Vec<ConfigNamedEntry>,
    pub categories_removed: Vec<ConfigNamedEntry>,
    pub categories_renamed: Vec<ConfigNameChange>,
    pub users_added: Vec<String>,
    pub users_removed: Vec<String>,
    /// Computed by Core before conversion; metadata alone is not structural.
    pub has_changes: bool,
}

impl From<core::ConfigDiff> for ConfigDiff {
    fn from(value: core::ConfigDiff) -> Self {
        let has_changes = value.has_changes();
        Self {
            version_old: value.version_old,
            version_new: value.version_new,
            date_old: value.date_old,
            date_new: value.date_new,
            controls_added: value.controls_added.into_iter().map(Into::into).collect(),
            controls_removed: value.controls_removed.into_iter().map(Into::into).collect(),
            controls_changed: value.controls_changed.into_iter().map(Into::into).collect(),
            rooms_added: value.rooms_added.into_iter().map(Into::into).collect(),
            rooms_removed: value.rooms_removed.into_iter().map(Into::into).collect(),
            rooms_renamed: value.rooms_renamed.into_iter().map(Into::into).collect(),
            categories_added: value.categories_added.into_iter().map(Into::into).collect(),
            categories_removed: value
                .categories_removed
                .into_iter()
                .map(Into::into)
                .collect(),
            categories_renamed: value
                .categories_renamed
                .into_iter()
                .map(Into::into)
                .collect(),
            users_added: value.users_added,
            users_removed: value.users_removed,
            has_changes,
        }
    }
}

/// Infallible comparison of supplied typed snapshots, not a full XML diff.
///
/// Raw UUIDs identify entries; map/set enumeration order is unspecified.
#[uniffi::export]
pub fn diff_configs(old: ConfigSummary, new: ConfigSummary) -> ConfigDiff {
    core::diff_configs(&old.into(), &new.into()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn snapshot() -> ConfigSummary {
        ConfigSummary {
            version: " raw Ω ".into(),
            date: "".into(),
            controls: HashMap::from([(
                " Same ".into(),
                ConfigControlEntry {
                    name: "Old".into(),
                    control_type: "Switch".into(),
                    room_uuid: "r".into(),
                },
            )]),
            rooms: HashMap::from([("r".into(), "".into())]),
            categories: HashMap::new(),
            users: vec!["".into(), "Ω".into(), "Ω".into()],
        }
    }

    #[test]
    fn reverse_snapshot_conversion_is_lossless_without_normalization() {
        let value = snapshot();
        let core: lox_core::configuration::summary::ConfigSummary = value.clone().into();
        assert_eq!(ConfigSummary::from(core), value);
    }

    #[test]
    fn complete_diff_conversion_retains_payload_order_identity_and_owned_values() {
        let mut old = snapshot();
        let mut new = old.clone();
        new.version = "new".into();
        new.date = "today".into();
        let entry = new.controls.get_mut(" Same ").unwrap();
        entry.name = "Neu Ω".into();
        entry.control_type = "Dimmer".into();
        entry.room_uuid = "missing".into();
        old.controls
            .insert("A".into(), old.controls[" Same "].clone());
        new.controls
            .insert("a".into(), new.controls[" Same "].clone());
        old.rooms.insert("gone".into(), "Twin".into());
        new.rooms.insert("added".into(), "Twin".into());
        new.rooms.insert("r".into(), "renamed".into());
        old.categories.insert("gone".into(), "Twin".into());
        old.categories.insert("c".into(), "Old".into());
        new.categories.insert("added".into(), "Twin".into());
        new.categories.insert("c".into(), "New".into());
        new.users = vec!["Ω".into(), "Added".into(), "Added".into()];
        let result = diff_configs(old.clone(), new.clone());
        assert!(result.has_changes);
        assert_eq!(result.controls_added[0].uuid, "a");
        assert_eq!(result.controls_removed[0].uuid, "A");
        assert_eq!(result.controls_changed.len(), 3);
        assert!(
            result
                .controls_changed
                .iter()
                .all(|c| c.uuid == " Same " && c.name == "Neu Ω")
        );
        assert_eq!(
            result.controls_changed[0].field,
            ConfigControlFieldChange::Name {
                old: "Old".into(),
                new: "Neu Ω".into()
            }
        );
        assert_eq!(
            result.controls_changed[1].field,
            ConfigControlFieldChange::Type {
                old: "Switch".into(),
                new: "Dimmer".into()
            }
        );
        assert_eq!(
            result.controls_changed[2].field,
            ConfigControlFieldChange::Room {
                old: ConfigRoomReference {
                    uuid: "r".into(),
                    name: Some("".into())
                },
                new: ConfigRoomReference {
                    uuid: "missing".into(),
                    name: None
                }
            }
        );
        assert_eq!(result.rooms_added[0].uuid, "added");
        assert_eq!(result.rooms_removed[0].uuid, "gone");
        assert_eq!(result.rooms_renamed[0].new, "renamed");
        assert_eq!(result.categories_added[0].uuid, "added");
        assert_eq!(result.categories_removed[0].uuid, "gone");
        assert_eq!(result.categories_renamed[0].old, "Old");
        assert_eq!(result.categories_renamed[0].new, "New");
        assert_eq!(result.users_added, ["Added"]);
        assert_eq!(result.users_removed, [""]);
        old.controls.clear();
        new.controls.clear();
        assert_eq!(result.controls_added[0].entry.name, "Neu Ω");
        assert_eq!(
            (result.version_old.as_str(), result.version_new.as_str()),
            (" raw Ω ", "new")
        );
        assert_eq!(
            (result.date_old.as_str(), result.date_new.as_str()),
            ("", "today")
        );
    }

    #[test]
    fn metadata_users_sets_and_room_rename_keep_core_has_changes_contract() {
        let old = snapshot();
        let mut new = old.clone();
        new.version = "changed".into();
        new.date = "changed".into();
        new.users = vec!["Ω".into(), "".into()];
        let result = diff_configs(old.clone(), new.clone());
        assert!(!result.has_changes);
        assert_eq!(result.version_old, " raw Ω ");
        assert_eq!(result.date_new, "changed");
        new.rooms.insert("r".into(), "renamed".into());
        let result = diff_configs(old, new);
        assert!(result.has_changes);
        assert!(result.controls_changed.is_empty());
        assert_eq!(result.rooms_renamed.len(), 1);
    }
}
