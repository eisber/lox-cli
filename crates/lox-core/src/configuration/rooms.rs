//! Read-only room inventory using the existing CLI's permissive XML contract.
//!
//! Attribute strings are raw UTF-8, not entity-decoded or normalized. This is
//! neither document validation nor a serialization/preservation API.

use super::xml::attr_value;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomInfo {
    pub uuid: String,
    pub name: String,
    /// Number of `IoData` start/empty elements with an exactly matching raw `Pr`.
    /// Includes devices, sensors and other items, not just function blocks.
    pub item_count: usize,
}

#[derive(Debug)]
pub enum RoomInventoryError {
    Xml { source: quick_xml::Error },
}

impl fmt::Display for RoomInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { source } => source.fmt(f),
        }
    }
}

impl std::error::Error for RoomInventoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source } => Some(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, RoomInventoryError>;

/// Inventory all `C Type="Place"` elements with readable `U` and `Title` attributes.
///
/// Empty IDs/titles are accepted. Later definitions of an ID replace its name.
/// Counts include matching `IoData` anywhere, even before a room's definition.
/// Results use case-sensitive name ordering; equal-name ordering is unspecified.
/// Attribute iterator errors are ignored, and incomplete XML at EOF may succeed.
/// No loader repair, I/O, context selection or input mutation takes place.
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
            Err(source) => return Err(RoomInventoryError::Xml { source }),
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
