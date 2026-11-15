//! Read-only legacy XML device inventory, not DOM device identity or port discovery.

use super::xml::attr_value;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceBus {
    Tree,
    Air,
    Network,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub bus: DeviceBus,
    pub serial: Option<String>,
    pub sub_type: Option<u32>,
    pub type_label: String,
    pub mac: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug)]
pub enum DeviceInventoryError {
    Xml { source: quick_xml::Error },
}

impl fmt::Display for DeviceInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { source } => source.fmt(f),
        }
    }
}

impl std::error::Error for DeviceInventoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source } => Some(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, DeviceInventoryError>;

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

/// Inventory exact `C` types `TreeDevice`, `LoxAIRDevice` and `NetworkDevice`.
///
/// Bus classification depends only on that raw type, never ancestors, pages,
/// serials or child connectors. Results retain document order and duplicates;
/// there is no filtering, sorting, deduplication or DOM identity grouping.
///
/// Attributes retain raw UTF-8/entity spellings. First matching attributes win;
/// unreadable values are absent and attribute errors are ignored. Missing titles
/// become empty; subtypes use untrimmed `u32` parsing. Only Tree retains `Serial`,
/// only Network retains `MAC`/`Addr`; Network's label ignores subtype.
///
/// Input is borrowed and never repaired or validated. Empty/text-only input and
/// incomplete EOF can succeed. Reader errors return no partial inventory.
pub fn parse_devices(xml: &[u8]) -> Result<Vec<DeviceInfo>> {
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
                        devices.push(DeviceInfo {
                            name: attr_value(e, b"Title").unwrap_or_default(),
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
                        devices.push(DeviceInfo {
                            name: attr_value(e, b"Title").unwrap_or_default(),
                            bus: DeviceBus::Air,
                            serial: None,
                            sub_type: sub,
                            type_label: device_type_label(sub),
                            mac: None,
                            address: None,
                        });
                    }
                    Some("NetworkDevice") => {
                        devices.push(DeviceInfo {
                            name: attr_value(e, b"Title").unwrap_or_default(),
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
            Err(source) => return Err(DeviceInventoryError::Xml { source }),
            _ => {}
        }
        buf.clear();
    }
    Ok(devices)
}
