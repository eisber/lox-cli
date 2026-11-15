use lox_core::configuration::diff::{
    ControlFieldChange, ControlRecord, NameChange, NamedEntry, RoomReference, diff_configs,
};
use lox_core::configuration::summary::{ConfigSummary, ControlEntry};
use std::collections::HashMap;

fn summary() -> ConfigSummary {
    ConfigSummary {
        version: "1".into(),
        date: "old".into(),
        controls: HashMap::new(),
        rooms: HashMap::new(),
        categories: HashMap::new(),
        users: Vec::new(),
    }
}

fn control(name: &str, control_type: &str, room_uuid: &str) -> ControlEntry {
    ControlEntry {
        name: name.into(),
        control_type: control_type.into(),
        room_uuid: room_uuid.into(),
    }
}

#[test]
fn raw_identity_and_all_changed_fields_retain_typed_context() {
    let mut old = summary();
    old.controls
        .insert("same".into(), control("Old", "Switch", "r"));
    old.controls
        .insert("A".into(), control("Twin", "Switch", ""));
    old.rooms.insert("r".into(), "Before".into());
    old.categories.insert("c".into(), "Lights".into());
    let mut new = summary();
    new.controls
        .insert("same".into(), control("New", "Dimmer", "missing"));
    new.controls
        .insert("a".into(), control("Twin", "Switch", ""));
    new.rooms.insert("r".into(), "After".into());
    new.categories.insert("c".into(), "Lighting".into());
    let old_snapshot = old.clone();
    let new_snapshot = new.clone();

    let diff = diff_configs(&old, &new);
    assert!(diff.has_changes());
    assert_eq!(
        diff.controls_added,
        [ControlRecord {
            uuid: "a".into(),
            entry: new.controls["a"].clone(),
        }]
    );
    assert_eq!(
        diff.controls_removed,
        [ControlRecord {
            uuid: "A".into(),
            entry: old.controls["A"].clone(),
        }]
    );
    assert_eq!(diff.controls_changed.len(), 3);
    for change in &diff.controls_changed {
        assert_eq!(change.uuid, "same");
        assert_eq!(change.name, "New");
    }
    assert_eq!(
        diff.controls_changed[0].field,
        ControlFieldChange::Name {
            old: "Old".into(),
            new: "New".into(),
        }
    );
    assert_eq!(
        diff.controls_changed[1].field,
        ControlFieldChange::Type {
            old: "Switch".into(),
            new: "Dimmer".into(),
        }
    );
    assert_eq!(
        diff.controls_changed[2].field,
        ControlFieldChange::Room {
            old: RoomReference {
                uuid: "r".into(),
                name: Some("Before".into())
            },
            new: RoomReference {
                uuid: "missing".into(),
                name: None
            },
        }
    );
    assert_eq!(
        diff.rooms_renamed,
        [NameChange {
            uuid: "r".into(),
            old: "Before".into(),
            new: "After".into(),
        }]
    );
    assert_eq!(
        diff.categories_renamed,
        [NameChange {
            uuid: "c".into(),
            old: "Lights".into(),
            new: "Lighting".into(),
        }]
    );
    assert_eq!(old, old_snapshot);
    assert_eq!(new, new_snapshot);
}

#[test]
fn equal_titles_do_not_hide_room_category_or_relationship_identity_changes() {
    let mut old = summary();
    old.rooms.insert("old".into(), "Twin".into());
    old.categories.insert("old".into(), "Twin".into());
    old.controls
        .insert("control".into(), control("Stable", "Switch", "old"));
    let mut new = summary();
    new.rooms.insert("new".into(), "Twin".into());
    new.categories.insert("new".into(), "Twin".into());
    new.controls
        .insert("control".into(), control("Stable", "Switch", "new"));

    let diff = diff_configs(&old, &new);
    let added = NamedEntry {
        uuid: "new".into(),
        name: "Twin".into(),
    };
    let removed = NamedEntry {
        uuid: "old".into(),
        name: "Twin".into(),
    };
    assert_eq!(diff.rooms_added, [added.clone()]);
    assert_eq!(diff.rooms_removed, [removed.clone()]);
    assert_eq!(diff.categories_added, [added]);
    assert_eq!(diff.categories_removed, [removed]);
    assert!(diff.rooms_renamed.is_empty());
    assert!(diff.categories_renamed.is_empty());
    assert_eq!(diff.controls_changed.len(), 1);
    assert_eq!(
        diff.controls_changed[0].field,
        ControlFieldChange::Room {
            old: RoomReference {
                uuid: "old".into(),
                name: Some("Twin".into())
            },
            new: RoomReference {
                uuid: "new".into(),
                name: Some("Twin".into())
            },
        }
    );
}

#[test]
fn metadata_and_user_multiplicity_do_not_count_but_name_membership_does() {
    let mut old = summary();
    old.users = vec!["".into(), "Keep".into(), "Keep".into()];
    let mut new = old.clone();
    new.version = "2".into();
    new.date = "new".into();
    new.users = vec!["Keep".into(), "".into()];
    let diff = diff_configs(&old, &new);
    assert!(!diff.has_changes());
    assert_eq!(
        (diff.version_old.as_str(), diff.version_new.as_str()),
        ("1", "2")
    );
    assert_eq!(
        (diff.date_old.as_str(), diff.date_new.as_str()),
        ("old", "new")
    );
    new.users = vec!["Added".into(), "Added".into(), "Keep".into()];
    let diff = diff_configs(&old, &new);
    assert!(diff.has_changes());
    assert_eq!(diff.users_added, ["Added"]);
    assert_eq!(diff.users_removed, [""]);
}

#[test]
fn room_rename_is_not_a_control_move_and_missing_or_empty_titles_stay_distinct() {
    let mut old = summary();
    old.rooms.insert("r".into(), "Before".into());
    old.controls
        .insert("a".into(), control("Stable", "Switch", "r"));
    let mut new = old.clone();
    new.rooms.insert("r".into(), "After".into());
    let diff = diff_configs(&old, &new);
    assert!(diff.controls_changed.is_empty());
    assert_eq!(diff.rooms_renamed.len(), 1);

    for (old_uuid, old_title, new_uuid, new_title) in [
        ("", None, "unknown", None),
        ("unknown", None, "", None),
        ("", Some("Empty ID"), "r", Some("")),
        ("unknown", Some("Named unknown"), "", Some("")),
    ] {
        let mut old = summary();
        let mut new = summary();
        old.controls
            .insert("a".into(), control("Stable", "Switch", old_uuid));
        new.controls
            .insert("a".into(), control("Stable", "Switch", new_uuid));
        if let Some(title) = old_title {
            old.rooms.insert(old_uuid.into(), title.into());
        }
        if let Some(title) = new_title {
            new.rooms.insert(new_uuid.into(), title.into());
        }
        let diff = diff_configs(&old, &new);
        assert_eq!(diff.controls_changed.len(), 1);
        assert_eq!(
            diff.controls_changed[0].field,
            ControlFieldChange::Room {
                old: RoomReference {
                    uuid: old_uuid.into(),
                    name: old_title.map(Into::into)
                },
                new: RoomReference {
                    uuid: new_uuid.into(),
                    name: new_title.map(Into::into)
                },
            }
        );
    }
}
