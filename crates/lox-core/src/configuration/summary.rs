//! Read-only structural summaries using the CLI's permissive XML contract.

use super::xml::attr_value;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSummary {
    pub version: String,
    pub date: String,
    /// Function-block candidates keyed by raw ID, using the legacy blacklist.
    pub controls: HashMap<String, ControlEntry>,
    pub rooms: HashMap<String, String>,
    pub categories: HashMap<String, String>,
    /// Titles in document order, including empty and duplicate titles.
    pub users: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEntry {
    pub name: String,
    pub control_type: String,
    /// Last readable `Pr` on a direct `IoData` child, or an empty string.
    pub room_uuid: String,
}

#[derive(Debug)]
pub enum ConfigSummaryError {
    Xml { source: quick_xml::Error },
}

impl fmt::Display for ConfigSummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { source } => source.fmt(f),
        }
    }
}

impl std::error::Error for ConfigSummaryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source } => Some(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, ConfigSummaryError>;

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

/// Summarize borrowed XML bytes without I/O, normalization or validation.
///
/// Attributes remain raw UTF-8, including entity spellings. Later definitions
/// replace earlier map entries; control candidates need a readable ID and a
/// nonempty title. Unknown types are candidates unless blacklisted. `IoData`
/// belongs only to its direct parent control, not the last control encountered.
/// Version/date use the last matching element, defaulting missing attributes to
/// empty strings. Attribute errors are ignored; incomplete XML at EOF may succeed.
/// Map iteration order is unspecified. No XML is mutated or serialized.
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
    // IoData belongs to its parent element, not the last control in document order.
    let mut element_controls: Vec<Option<String>> = Vec::new();

    loop {
        let event = reader.read_event_into(&mut buf);
        let opens_element = matches!(&event, Ok(Event::Start(_)));
        match event {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let mut control_uuid = None;
                let tag = e.name();
                if tag.as_ref() == b"ControlList" {
                    version = attr_value(e, b"Version").unwrap_or_default();
                } else if tag.as_ref() == b"IoData" {
                    if let Some(Some(uuid)) = element_controls.last()
                        && let Some(pr) = attr_value(e, b"Pr")
                        && let Some(entry) = controls.get_mut(uuid)
                    {
                        entry.room_uuid = pr;
                    }
                } else if tag.as_ref() == b"C" {
                    let type_val = attr_value(e, b"Type");
                    match type_val.as_deref() {
                        Some("Document") => {
                            date = attr_value(e, b"Date").unwrap_or_default();
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
                                control_uuid = Some(u.clone());
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
                if opens_element {
                    element_controls.push(control_uuid);
                }
            }
            Ok(Event::End(_)) => {
                element_controls.pop();
            }
            Ok(Event::Eof) => break,
            Err(source) => return Err(ConfigSummaryError::Xml { source }),
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
