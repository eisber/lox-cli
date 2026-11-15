//! Read-only offline XML control inventory, distinct from the online catalog.

use super::xml::attr_value;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlInfo {
    pub control_type: String,
    pub title: String,
    pub uuid: String,
    pub room: String,
    pub category: String,
}

#[derive(Debug)]
pub enum ControlInventoryError {
    Xml { source: quick_xml::Error },
}

impl fmt::Display for ControlInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { source } => source.fmt(f),
        }
    }
}

impl std::error::Error for ControlInventoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source } => Some(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, ControlInventoryError>;

/// Inventory borrowed XML using the CLI's permissive legacy candidate tracking.
///
/// Any `C` type except exact `Place`/`Category` can become pending with a readable
/// ID and an ASCII-case-insensitive exact type match. A readable title, including
/// empty, permits the next `IoData` anywhere to emit it once. End tags do not
/// clear pending state; this is deliberately not Summary's direct-parent rule.
///
/// Attributes stay raw UTF-8; first matching attributes win and attribute errors
/// are ignored. Room/category definitions resolve after scanning, last ID wins,
/// with raw ID fallback. Room filtering uses Unicode lowercase substring matching.
/// Results sort stably by case-sensitive type then title, retaining duplicates.
/// Input is not mutated, repaired or validated; incomplete EOF may succeed.
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
                                // Keep legacy pending state, not a parent stack or blacklist.
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
            Err(source) => return Err(ControlInventoryError::Xml { source }),
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
