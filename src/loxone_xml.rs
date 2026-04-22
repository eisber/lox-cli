use anyhow::{Context, Result};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde::Serialize;
use std::collections::HashMap;

// ── Users ──

#[derive(Debug, Serialize)]
pub struct LoxUser {
    pub name: String,
    pub nfc: bool,
    pub description: String,
}

// ── Devices ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceBus {
    Tree,
    Air,
    Network,
}

#[derive(Debug, Serialize)]
pub struct LoxDevice {
    pub name: String,
    pub bus: DeviceBus,
    pub serial: Option<String>,
    pub sub_type: Option<u32>,
    pub type_label: String,
    pub mac: Option<String>,
    pub address: Option<String>,
}

// ── Diff ──

#[derive(Debug, Serialize)]
pub struct ConfigSummary {
    pub version: String,
    pub date: String,
    pub controls: HashMap<String, ControlEntry>,
    pub rooms: HashMap<String, String>,
    pub categories: HashMap<String, String>,
    pub users: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ControlEntry {
    pub name: String,
    pub control_type: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub room_uuid: String,
}

// ── Helpers ──

fn attr_value(e: &BytesStart, name: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == name {
            return String::from_utf8(attr.value.to_vec()).ok();
        }
    }
    None
}

fn attr_value_or(e: &BytesStart, name: &[u8], default: &str) -> String {
    attr_value(e, name).unwrap_or_else(|| default.to_string())
}

pub fn subtype_label(sub_type: u32) -> &'static str {
    match sub_type {
        7 => "Nano IO Air",
        19 => "Smoke detector",
        32 => "Water sensor",
        37 => "Smartaktor",
        48 => "Room climate sensor",
        32780 => "Dimmer",
        32784 => "Room climate sensor",
        32794 => "Presence sensor",
        32796 => "Code Touch keypad",
        32797 => "LoxAIR Bridge",
        32799 => "Flex connector",
        _ => "",
    }
}

fn device_type_label(sub_type: Option<u32>) -> String {
    match sub_type {
        Some(st) => {
            let label = subtype_label(st);
            if label.is_empty() {
                format!("Unknown ({})", st)
            } else {
                label.to_string()
            }
        }
        None => "Unknown".to_string(),
    }
}

/// Check if a `<C>` element is a known control type (function block) for diff purposes.
/// We use a blacklist of infrastructure/structural types.
fn is_control_type(type_attr: &str) -> bool {
    !matches!(
        type_attr,
        "Document"
            | "Page"
            | "Place"
            | "Category"
            | "User"
            | "UserCaption"
            | "LoxCaption"
            | "TreeDevice"
            | "LoxAIRDevice"
            | "NetworkDevice"
            | "TreeAsensor"
            | "TreeDsensor"
            | "TreeAactuator"
            | "TreeDactuator"
            | "LoxAIRAsensor"
            | "LoxAIRDsensor"
            | "LoxAIRAactuator"
            | "LoxAIRDactuator"
            | "InputRef"
            | "OutputRef"
            | "Co"
            | "IoData"
            | "Display"
            | "HP"
            | "IoList"
            | "Const"
            | "Note"
            | "ControlList"
            | "SubGroup"
            | "VirtUserDevice"
            | "CentralDevice"
    )
}

// ── Parsing ──

pub fn parse_users(xml: &[u8]) -> Result<Vec<LoxUser>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut users = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e))
                if e.name().as_ref() == b"C"
                    && attr_value(e, b"Type").as_deref() == Some("User") =>
            {
                let name = attr_value_or(e, b"Title", "");
                let nfc_arr = attr_value_or(e, b"NFCArr", "");
                let desc = attr_value_or(e, b"Desc", "").replace('_', " ");
                users.push(LoxUser {
                    name,
                    nfc: !nfc_arr.is_empty(),
                    description: desc,
                });
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e).context("Failed to parse XML"),
            _ => {}
        }
        buf.clear();
    }
    Ok(users)
}

pub fn parse_devices(xml: &[u8]) -> Result<Vec<LoxDevice>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut devices = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) if e.name().as_ref() == b"C" => {
                let type_val = attr_value(e, b"Type");
                match type_val.as_deref() {
                    Some("TreeDevice") => {
                        let sub = attr_value(e, b"SubType").and_then(|s| s.parse::<u32>().ok());
                        devices.push(LoxDevice {
                            name: attr_value_or(e, b"Title", ""),
                            bus: DeviceBus::Tree,
                            serial: attr_value(e, b"Serial"),
                            sub_type: sub,
                            type_label: device_type_label(sub),
                            mac: None,
                            address: None,
                        });
                    }
                    Some("LoxAIRDevice") => {
                        let sub = attr_value(e, b"SubType").and_then(|s| s.parse::<u32>().ok());
                        devices.push(LoxDevice {
                            name: attr_value_or(e, b"Title", ""),
                            bus: DeviceBus::Air,
                            serial: None,
                            sub_type: sub,
                            type_label: device_type_label(sub),
                            mac: None,
                            address: None,
                        });
                    }
                    Some("NetworkDevice") => {
                        devices.push(LoxDevice {
                            name: attr_value_or(e, b"Title", ""),
                            bus: DeviceBus::Network,
                            serial: None,
                            sub_type: attr_value(e, b"SubType").and_then(|s| s.parse::<u32>().ok()),
                            type_label: "Network device".to_string(),
                            mac: attr_value(e, b"MAC"),
                            address: attr_value(e, b"Addr"),
                        });
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e).context("Failed to parse XML"),
            _ => {}
        }
        buf.clear();
    }
    Ok(devices)
}

pub fn parse_config_summary(xml: &[u8]) -> Result<ConfigSummary> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut version = String::new();
    let mut date = String::new();
    let mut controls: HashMap<String, ControlEntry> = HashMap::new();
    let mut rooms: HashMap<String, String> = HashMap::new();
    let mut categories: HashMap<String, String> = HashMap::new();
    let mut users: Vec<String> = Vec::new();
    // Track the last control UUID we saw so we can attach IoData Pr= to it
    let mut last_control_uuid: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let tag = e.name();
                if tag.as_ref() == b"ControlList" {
                    version = attr_value_or(e, b"Version", "");
                } else if tag.as_ref() == b"IoData" {
                    if let Some(ref uuid) = last_control_uuid
                        && let Some(pr) = attr_value(e, b"Pr")
                        && let Some(entry) = controls.get_mut(uuid)
                    {
                        entry.room_uuid = pr;
                    }
                } else if tag.as_ref() == b"C" {
                    let type_val = attr_value(e, b"Type");
                    match type_val.as_deref() {
                        Some("Document") => {
                            date = attr_value_or(e, b"Date", "");
                        }
                        Some("Place") => {
                            if let (Some(u), Some(t)) =
                                (attr_value(e, b"U"), attr_value(e, b"Title"))
                            {
                                rooms.insert(u, t);
                            }
                        }
                        Some("Category") => {
                            if let (Some(u), Some(t)) =
                                (attr_value(e, b"U"), attr_value(e, b"Title"))
                            {
                                categories.insert(u, t);
                            }
                        }
                        Some("User") => {
                            if let Some(t) = attr_value(e, b"Title") {
                                users.push(t);
                            }
                        }
                        Some(typ) if is_control_type(typ) => {
                            if let (Some(u), Some(t)) =
                                (attr_value(e, b"U"), attr_value(e, b"Title"))
                                && !t.is_empty()
                            {
                                last_control_uuid = Some(u.clone());
                                controls.insert(
                                    u,
                                    ControlEntry {
                                        name: t,
                                        control_type: typ.to_string(),
                                        room_uuid: String::new(),
                                    },
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e).context("Failed to parse XML"),
            _ => {}
        }
        buf.clear();
    }

    Ok(ConfigSummary {
        version,
        date,
        controls,
        rooms,
        categories,
        users,
    })
}

// ── Diff output ──

#[derive(Debug, Serialize)]
pub struct ConfigDiff {
    pub version_old: String,
    pub version_new: String,
    pub date_old: String,
    pub date_new: String,
    pub controls_added: Vec<ControlEntry>,
    pub controls_removed: Vec<ControlEntry>,
    pub controls_changed: Vec<ControlChange>,
    pub rooms_added: Vec<String>,
    pub rooms_removed: Vec<String>,
    pub rooms_renamed: Vec<NameChange>,
    pub categories_added: Vec<String>,
    pub categories_removed: Vec<String>,
    pub categories_renamed: Vec<NameChange>,
    pub users_added: Vec<String>,
    pub users_removed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ControlChange {
    pub name: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Serialize)]
pub struct NameChange {
    pub old: String,
    pub new: String,
}

impl ConfigDiff {
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

// ── Rooms ──

#[derive(Debug, Serialize)]
pub struct RoomInfo {
    pub uuid: String,
    pub name: String,
    pub item_count: usize,
}

/// Parse rooms from a .Loxone XML, counting items assigned to each room via IoData Pr=.
pub fn parse_rooms(xml: &[u8]) -> Result<Vec<RoomInfo>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut rooms: HashMap<String, String> = HashMap::new();
    let mut room_counts: HashMap<String, usize> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let tag = e.name();
                if tag.as_ref() == b"C" {
                    if let Some(t) = attr_value(e, b"Type")
                        && t == "Place"
                        && let (Some(u), Some(name)) =
                            (attr_value(e, b"U"), attr_value(e, b"Title"))
                    {
                        rooms.insert(u.clone(), name);
                        room_counts.entry(u).or_insert(0);
                    }
                } else if tag.as_ref() == b"IoData"
                    && let Some(pr) = attr_value(e, b"Pr")
                {
                    *room_counts.entry(pr).or_insert(0) += 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        buf.clear();
    }

    let mut result: Vec<RoomInfo> = rooms
        .into_iter()
        .map(|(uuid, name)| RoomInfo {
            item_count: room_counts.get(&uuid).copied().unwrap_or(0),
            uuid,
            name,
        })
        .collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

// ── Controls ──

#[derive(Debug, Serialize)]
pub struct ControlInfo {
    pub control_type: String,
    pub title: String,
    pub uuid: String,
    pub room: String,
    pub category: String,
}

/// Parse controls from a .Loxone XML with their room and category names.
pub fn parse_controls(
    xml: &[u8],
    type_filter: Option<&str>,
    room_filter: Option<&str>,
) -> Result<Vec<ControlInfo>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut rooms: HashMap<String, String> = HashMap::new();
    let mut categories: HashMap<String, String> = HashMap::new();

    struct RawControl {
        control_type: String,
        title: String,
        uuid: String,
        room_uuid: String,
        cat_uuid: String,
    }

    let mut controls: Vec<RawControl> = Vec::new();
    let mut current_uuid: Option<String> = None;
    let mut current_type: Option<String> = None;
    let mut current_title: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let tag = e.name();
                if tag.as_ref() == b"C" {
                    if let Some(t) = attr_value(e, b"Type") {
                        match t.as_str() {
                            "Place" => {
                                if let (Some(u), Some(name)) =
                                    (attr_value(e, b"U"), attr_value(e, b"Title"))
                                {
                                    rooms.insert(u, name);
                                }
                            }
                            "Category" => {
                                if let (Some(u), Some(name)) =
                                    (attr_value(e, b"U"), attr_value(e, b"Title"))
                                {
                                    categories.insert(u, name);
                                }
                            }
                            _ => {
                                // Track any element with a Type, Title, and UUID
                                if type_filter
                                    .map(|f| t.eq_ignore_ascii_case(f))
                                    .unwrap_or(true)
                                    && let Some(uuid) = attr_value(e, b"U")
                                {
                                    current_uuid = Some(uuid);
                                    current_type = Some(t);
                                    current_title = attr_value(e, b"Title");
                                }
                            }
                        }
                    }
                } else if tag.as_ref() == b"IoData"
                    && let (Some(uuid), Some(ctype), Some(title)) =
                        (&current_uuid, &current_type, &current_title)
                {
                    let room_uuid = attr_value(e, b"Pr").unwrap_or_default();
                    let cat_uuid = attr_value(e, b"Cr").unwrap_or_default();
                    controls.push(RawControl {
                        control_type: ctype.clone(),
                        title: title.clone(),
                        uuid: uuid.clone(),
                        room_uuid,
                        cat_uuid,
                    });
                    current_uuid = None;
                    current_type = None;
                    current_title = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
            _ => {}
        }
        buf.clear();
    }

    let mut result: Vec<ControlInfo> = controls
        .into_iter()
        .map(|c| ControlInfo {
            control_type: c.control_type,
            title: c.title,
            uuid: c.uuid,
            room: rooms.get(&c.room_uuid).cloned().unwrap_or(c.room_uuid),
            category: categories.get(&c.cat_uuid).cloned().unwrap_or(c.cat_uuid),
        })
        .collect();

    if let Some(rf) = room_filter {
        let rf_lower = rf.to_lowercase();
        result.retain(|c| c.room.to_lowercase().contains(&rf_lower));
    }

    result.sort_by(|a, b| {
        a.control_type
            .cmp(&b.control_type)
            .then(a.title.cmp(&b.title))
    });
    Ok(result)
}

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

    // Controls
    for (uuid, entry) in &new.controls {
        match old.controls.get(uuid) {
            None => diff.controls_added.push(entry.clone()),
            Some(old_entry) => {
                if old_entry.name != entry.name {
                    diff.controls_changed.push(ControlChange {
                        name: entry.name.clone(),
                        field: "name".to_string(),
                        old_value: old_entry.name.clone(),
                        new_value: entry.name.clone(),
                    });
                }
                if old_entry.control_type != entry.control_type {
                    diff.controls_changed.push(ControlChange {
                        name: entry.name.clone(),
                        field: "type".to_string(),
                        old_value: old_entry.control_type.clone(),
                        new_value: entry.control_type.clone(),
                    });
                }
                if old_entry.room_uuid != entry.room_uuid
                    && !(old_entry.room_uuid.is_empty() && entry.room_uuid.is_empty())
                {
                    let old_room =
                        old.rooms
                            .get(&old_entry.room_uuid)
                            .cloned()
                            .unwrap_or_else(|| {
                                if old_entry.room_uuid.is_empty() {
                                    "(none)".to_string()
                                } else {
                                    old_entry.room_uuid.clone()
                                }
                            });
                    let new_room = new.rooms.get(&entry.room_uuid).cloned().unwrap_or_else(|| {
                        if entry.room_uuid.is_empty() {
                            "(none)".to_string()
                        } else {
                            entry.room_uuid.clone()
                        }
                    });
                    diff.controls_changed.push(ControlChange {
                        name: entry.name.clone(),
                        field: "room".to_string(),
                        old_value: old_room,
                        new_value: new_room,
                    });
                }
            }
        }
    }
    for (uuid, entry) in &old.controls {
        if !new.controls.contains_key(uuid) {
            diff.controls_removed.push(entry.clone());
        }
    }

    // Rooms
    diff_named_map(
        &old.rooms,
        &new.rooms,
        &mut diff.rooms_added,
        &mut diff.rooms_removed,
        &mut diff.rooms_renamed,
    );

    // Categories
    diff_named_map(
        &old.categories,
        &new.categories,
        &mut diff.categories_added,
        &mut diff.categories_removed,
        &mut diff.categories_renamed,
    );

    // Users (compare by name since they have UUIDs but name is more meaningful)
    let old_users: std::collections::HashSet<&String> = old.users.iter().collect();
    let new_users: std::collections::HashSet<&String> = new.users.iter().collect();
    for u in &new_users {
        if !old_users.contains(u) {
            diff.users_added.push((*u).clone());
        }
    }
    for u in &old_users {
        if !new_users.contains(u) {
            diff.users_removed.push((*u).clone());
        }
    }

    diff
}

fn diff_named_map(
    old: &HashMap<String, String>,
    new: &HashMap<String, String>,
    added: &mut Vec<String>,
    removed: &mut Vec<String>,
    renamed: &mut Vec<NameChange>,
) {
    for (uuid, name) in new {
        match old.get(uuid) {
            None => added.push(name.clone()),
            Some(old_name) if old_name != name => {
                renamed.push(NameChange {
                    old: old_name.clone(),
                    new: name.clone(),
                });
            }
            _ => {}
        }
    }
    for (uuid, name) in old {
        if !new.contains_key(uuid) {
            removed.push(name.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_users() {
        let xml = br#"<?xml version="1.0"?>
        <ControlList Version="1">
          <C Type="Document" Date="2026-01-01">
            <C Type="UserCaption">
              <C Type="User" Title="admin" NFCArr="" Desc=""/>
              <C Type="User" Title="chris" NFCArr="AABB" Desc="Main_user"/>
            </C>
          </C>
        </ControlList>"#;
        let users = parse_users(xml).unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "admin");
        assert!(!users[0].nfc);
        assert_eq!(users[0].description, "");
        assert_eq!(users[1].name, "chris");
        assert!(users[1].nfc);
        assert_eq!(users[1].description, "Main user");
    }

    #[test]
    fn test_parse_devices() {
        let xml = br#"<?xml version="1.0"?>
        <ControlList Version="1">
          <C Type="TreeDevice" Title="Raumklima" Serial="B03C7ED6" SubType="32784"/>
          <C Type="LoxAIRDevice" Title="Rauchmelder" SubType="19"/>
          <C Type="NetworkDevice" Title="Intercom" Addr="ice:7091" MAC="504F94E0F182" SubType="2"/>
        </ControlList>"#;
        let devices = parse_devices(xml).unwrap();
        assert_eq!(devices.len(), 3);

        assert_eq!(devices[0].name, "Raumklima");
        assert_eq!(devices[0].bus, DeviceBus::Tree);
        assert_eq!(devices[0].serial.as_deref(), Some("B03C7ED6"));
        assert_eq!(devices[0].type_label, "Room climate sensor");

        assert_eq!(devices[1].name, "Rauchmelder");
        assert_eq!(devices[1].bus, DeviceBus::Air);
        assert_eq!(devices[1].type_label, "Smoke detector");

        assert_eq!(devices[2].name, "Intercom");
        assert_eq!(devices[2].bus, DeviceBus::Network);
        assert_eq!(devices[2].mac.as_deref(), Some("504F94E0F182"));
    }

    #[test]
    fn test_subtype_label_known() {
        assert_eq!(subtype_label(32784), "Room climate sensor");
        assert_eq!(subtype_label(19), "Smoke detector");
    }

    #[test]
    fn test_subtype_label_unknown() {
        assert_eq!(subtype_label(99999), "");
    }

    #[test]
    fn test_device_type_label_unknown() {
        assert_eq!(device_type_label(Some(99999)), "Unknown (99999)");
        assert_eq!(device_type_label(None), "Unknown");
    }

    #[test]
    fn test_parse_config_summary() {
        let xml = br#"<?xml version="1.0"?>
        <ControlList Version="42">
          <C Type="Document" Date="2026-03-01 02:12:26" U="doc-1" Title="Test"/>
          <C Type="Category" U="cat-1" Title="Lighting"/>
          <C Type="Place" U="room-1" Title="Kitchen"/>
          <C Type="User" Title="admin"/>
          <C Type="Switch" U="sw-1" Title="Light Kitchen"/>
        </ControlList>"#;
        let s = parse_config_summary(xml).unwrap();
        assert_eq!(s.version, "42");
        assert_eq!(s.date, "2026-03-01 02:12:26");
        assert_eq!(s.categories.get("cat-1").unwrap(), "Lighting");
        assert_eq!(s.rooms.get("room-1").unwrap(), "Kitchen");
        assert_eq!(s.users, vec!["admin"]);
        assert!(s.controls.contains_key("sw-1"));
        assert_eq!(s.controls["sw-1"].name, "Light Kitchen");
        assert_eq!(s.controls["sw-1"].control_type, "Switch");
    }

    #[test]
    fn test_diff_configs() {
        let old = ConfigSummary {
            version: "1".into(),
            date: "2026-01-01".into(),
            controls: HashMap::from([
                (
                    "a".into(),
                    ControlEntry {
                        name: "Light".into(),
                        control_type: "Switch".into(),
                        room_uuid: "r1".into(),
                    },
                ),
                (
                    "b".into(),
                    ControlEntry {
                        name: "Blind".into(),
                        control_type: "Jalousie".into(),
                        room_uuid: String::new(),
                    },
                ),
            ]),
            rooms: HashMap::from([("r1".into(), "Kitchen".into())]),
            categories: HashMap::new(),
            users: vec!["admin".into(), "chris".into()],
        };
        let new = ConfigSummary {
            version: "2".into(),
            date: "2026-02-01".into(),
            controls: HashMap::from([
                (
                    "a".into(),
                    ControlEntry {
                        name: "Light Renamed".into(),
                        control_type: "Switch".into(),
                        room_uuid: "r1".into(),
                    },
                ),
                (
                    "c".into(),
                    ControlEntry {
                        name: "Outlet".into(),
                        control_type: "Switch".into(),
                        room_uuid: String::new(),
                    },
                ),
            ]),
            rooms: HashMap::from([("r1".into(), "Wohnküche".into())]),
            categories: HashMap::new(),
            users: vec!["admin".into(), "newuser".into()],
        };

        let diff = diff_configs(&old, &new);
        assert_eq!(diff.controls_added.len(), 1);
        assert_eq!(diff.controls_added[0].name, "Outlet");
        assert_eq!(diff.controls_removed.len(), 1);
        assert_eq!(diff.controls_removed[0].name, "Blind");
        assert_eq!(diff.controls_changed.len(), 1);
        assert_eq!(diff.controls_changed[0].old_value, "Light");
        assert_eq!(diff.rooms_renamed.len(), 1);
        assert_eq!(diff.rooms_renamed[0].old, "Kitchen");
        assert_eq!(diff.users_added, vec!["newuser"]);
        assert_eq!(diff.users_removed, vec!["chris"]);
    }

    #[test]
    fn test_parse_rooms() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Place" U="room-2" Title="Bedroom"/>
  <C Type="Place" U="room-3" Title="Zentral"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-3"/>
  </C>
  <C Type="SysVar" U="sv-1" Title="Aussentemperatur">
    <IoData Cr="cat-1" Pr="room-3"/>
  </C>
  <C Type="Switch" U="sw-1" Title="Light">
    <IoData Cr="cat-2" Pr="room-1"/>
  </C>
  <C Type="IntercomDevice" U="ic-1" Title="Intercom">
    <IoData Cr="cat-3" Pr="room-2"/>
  </C>
</ControlList>"#;
        let rooms = parse_rooms(xml).unwrap();
        assert_eq!(rooms.len(), 3);

        let kitchen = rooms.iter().find(|r| r.name == "Kitchen").unwrap();
        assert_eq!(kitchen.item_count, 1);

        let bedroom = rooms.iter().find(|r| r.name == "Bedroom").unwrap();
        assert_eq!(bedroom.item_count, 1);

        let zentral = rooms.iter().find(|r| r.name == "Zentral").unwrap();
        assert_eq!(zentral.item_count, 2);
    }

    #[test]
    fn test_parse_rooms_empty() {
        let xml = br#"<?xml version="1.0"?><ControlList/>"#;
        let rooms = parse_rooms(xml).unwrap();
        assert!(rooms.is_empty());
    }

    #[test]
    fn test_parse_controls_all() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Place" U="room-2" Title="Zentral"/>
  <C Type="Category" U="cat-1" Title="Wetter"/>
  <C Type="Category" U="cat-2" Title="Beleuchtung"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-2"/>
  </C>
  <C Type="SysVar" U="sv-1" Title="Wind">
    <IoData Cr="cat-1" Pr="room-2"/>
  </C>
  <C Type="Switch" U="sw-1" Title="Light">
    <IoData Cr="cat-2" Pr="room-1"/>
  </C>
</ControlList>"#;
        let controls = parse_controls(xml, None, None).unwrap();
        assert_eq!(controls.len(), 3);
        // Sorted by type then title
        assert_eq!(controls[0].control_type, "Switch");
        assert_eq!(controls[0].room, "Kitchen");
        assert_eq!(controls[1].control_type, "SysVar");
        assert_eq!(controls[2].control_type, "WeatherData");
        assert_eq!(controls[2].room, "Zentral");
        assert_eq!(controls[2].category, "Wetter");
    }

    #[test]
    fn test_parse_controls_type_filter() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Category" U="cat-1" Title="Wetter"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="SysVar" U="sv-1" Title="Wind">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;
        let controls = parse_controls(xml, Some("WeatherData"), None).unwrap();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].title, "Temperatur");
    }

    #[test]
    fn test_parse_controls_room_filter() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Place" U="room-2" Title="Zentral"/>
  <C Type="Category" U="cat-1" Title="Wetter"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-2"/>
  </C>
  <C Type="Switch" U="sw-1" Title="Light">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;
        let controls = parse_controls(xml, None, Some("Zentral")).unwrap();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].title, "Temperatur");
    }

    // ── Additional loxone_xml tests ──────────────────────────────────────

    #[test]
    fn test_parse_controls_type_and_room_combined() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Place" U="room-2" Title="Zentral"/>
  <C Type="Category" U="cat-1" Title="Wetter"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="WeatherData" U="wd-2" Title="Wind">
    <IoData Cr="cat-1" Pr="room-2"/>
  </C>
  <C Type="Switch" U="sw-1" Title="Light">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;
        // Filter both by type AND room
        let controls = parse_controls(xml, Some("WeatherData"), Some("Kitchen")).unwrap();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].title, "Temperatur");
        assert_eq!(controls[0].room, "Kitchen");
    }

    #[test]
    fn test_parse_controls_type_and_room_no_match() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Category" U="cat-1" Title="Wetter"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;
        let controls = parse_controls(xml, Some("Switch"), Some("Kitchen")).unwrap();
        assert_eq!(controls.len(), 0);
    }

    #[test]
    fn test_parse_rooms_no_items() {
        // Room with no IoData references should have 0 items
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Place" U="room-2" Title="Bedroom"/>
  <C Type="Switch" U="sw-1" Title="Light">
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;
        let rooms = parse_rooms(xml).unwrap();
        assert_eq!(rooms.len(), 2);
        let bedroom = rooms.iter().find(|r| r.name == "Bedroom").unwrap();
        assert_eq!(bedroom.item_count, 0);
        let kitchen = rooms.iter().find(|r| r.name == "Kitchen").unwrap();
        assert_eq!(kitchen.item_count, 1);
    }

    #[test]
    fn test_parse_rooms_sorted_by_name() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="Place" U="room-3" Title="Zentral"/>
  <C Type="Place" U="room-1" Title="Bathroom"/>
  <C Type="Place" U="room-2" Title="Kitchen"/>
</ControlList>"#;
        let rooms = parse_rooms(xml).unwrap();
        assert_eq!(rooms[0].name, "Bathroom");
        assert_eq!(rooms[1].name, "Kitchen");
        assert_eq!(rooms[2].name, "Zentral");
    }

    #[test]
    fn test_diff_configs_no_changes() {
        let config = ConfigSummary {
            version: "1".into(),
            date: "2026-01-01".into(),
            controls: HashMap::new(),
            rooms: HashMap::new(),
            categories: HashMap::new(),
            users: vec!["admin".into()],
        };
        let diff = diff_configs(&config, &config);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_diff_configs_category_changes() {
        let old = ConfigSummary {
            version: "1".into(),
            date: "2026-01-01".into(),
            controls: HashMap::new(),
            rooms: HashMap::new(),
            categories: HashMap::from([
                ("c1".into(), "Lighting".into()),
                ("c2".into(), "Heating".into()),
            ]),
            users: vec![],
        };
        let new = ConfigSummary {
            version: "2".into(),
            date: "2026-02-01".into(),
            controls: HashMap::new(),
            rooms: HashMap::new(),
            categories: HashMap::from([
                ("c1".into(), "Beleuchtung".into()), // renamed
                ("c3".into(), "Cooling".into()),     // added
            ]),
            users: vec![],
        };
        let diff = diff_configs(&old, &new);
        assert!(diff.has_changes());
        assert_eq!(diff.categories_added.len(), 1);
        assert_eq!(diff.categories_added[0], "Cooling");
        assert_eq!(diff.categories_removed.len(), 1);
        assert_eq!(diff.categories_removed[0], "Heating");
        assert_eq!(diff.categories_renamed.len(), 1);
        assert_eq!(diff.categories_renamed[0].old, "Lighting");
        assert_eq!(diff.categories_renamed[0].new, "Beleuchtung");
    }

    #[test]
    fn test_diff_configs_room_changes_with_control_room_move() {
        let old = ConfigSummary {
            version: "1".into(),
            date: "2026-01-01".into(),
            controls: HashMap::from([(
                "sw-1".into(),
                ControlEntry {
                    name: "Light".into(),
                    control_type: "Switch".into(),
                    room_uuid: "r1".into(),
                },
            )]),
            rooms: HashMap::from([
                ("r1".into(), "Kitchen".into()),
                ("r2".into(), "Bedroom".into()),
            ]),
            categories: HashMap::new(),
            users: vec![],
        };
        let new = ConfigSummary {
            version: "2".into(),
            date: "2026-02-01".into(),
            controls: HashMap::from([(
                "sw-1".into(),
                ControlEntry {
                    name: "Light".into(),
                    control_type: "Switch".into(),
                    room_uuid: "r2".into(), // moved to Bedroom
                },
            )]),
            rooms: HashMap::from([
                ("r1".into(), "Kitchen".into()),
                ("r2".into(), "Bedroom".into()),
            ]),
            categories: HashMap::new(),
            users: vec![],
        };
        let diff = diff_configs(&old, &new);
        assert!(diff.has_changes());
        assert_eq!(diff.controls_changed.len(), 1);
        assert_eq!(diff.controls_changed[0].field, "room");
        assert_eq!(diff.controls_changed[0].old_value, "Kitchen");
        assert_eq!(diff.controls_changed[0].new_value, "Bedroom");
    }

    #[test]
    fn test_parse_config_summary_empty() {
        let xml = br#"<?xml version="1.0"?><ControlList Version="99"/>"#;
        let s = parse_config_summary(xml).unwrap();
        assert_eq!(s.version, "99");
        assert!(s.controls.is_empty());
        assert!(s.rooms.is_empty());
        assert!(s.categories.is_empty());
        assert!(s.users.is_empty());
    }

    #[test]
    fn test_parse_config_summary_with_iodata() {
        let xml = br#"<?xml version="1.0"?>
<ControlList Version="42">
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Switch" U="sw-1" Title="Light">
    <IoData Pr="room-1"/>
  </C>
</ControlList>"#;
        let s = parse_config_summary(xml).unwrap();
        assert_eq!(s.controls["sw-1"].room_uuid, "room-1");
    }

    #[test]
    fn test_parse_users_empty() {
        let xml = br#"<?xml version="1.0"?><ControlList Version="1"/>"#;
        let users = parse_users(xml).unwrap();
        assert!(users.is_empty());
    }

    #[test]
    fn test_parse_devices_empty() {
        let xml = br#"<?xml version="1.0"?><ControlList Version="1"/>"#;
        let devices = parse_devices(xml).unwrap();
        assert!(devices.is_empty());
    }

    #[test]
    fn test_is_control_type_filters() {
        // Structural types should be filtered out
        assert!(!is_control_type("Document"));
        assert!(!is_control_type("Place"));
        assert!(!is_control_type("Category"));
        assert!(!is_control_type("User"));
        assert!(!is_control_type("TreeDevice"));
        assert!(!is_control_type("LoxAIRDevice"));
        assert!(!is_control_type("Co"));
        assert!(!is_control_type("IoData"));
        // Control types should pass
        assert!(is_control_type("Switch"));
        assert!(is_control_type("Dimmer"));
        assert!(is_control_type("WeatherData"));
        assert!(is_control_type("GenTSensor"));
    }

    #[test]
    fn test_subtype_label_all_known() {
        assert_eq!(subtype_label(7), "Nano IO Air");
        assert_eq!(subtype_label(32), "Water sensor");
        assert_eq!(subtype_label(37), "Smartaktor");
        assert_eq!(subtype_label(48), "Room climate sensor");
        assert_eq!(subtype_label(32780), "Dimmer");
        assert_eq!(subtype_label(32794), "Presence sensor");
        assert_eq!(subtype_label(32796), "Code Touch keypad");
        assert_eq!(subtype_label(32797), "LoxAIR Bridge");
        assert_eq!(subtype_label(32799), "Flex connector");
    }

    #[test]
    fn test_parse_controls_empty_xml() {
        let xml = br#"<?xml version="1.0"?><ControlList/>"#;
        let controls = parse_controls(xml, None, None).unwrap();
        assert!(controls.is_empty());
    }
}
