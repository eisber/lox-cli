//! Config XML editing engine — DOM-based modification of .Loxone config files.
//!
//! Provides structured XML editing with proper DOM manipulation and BOM-aware
//! write-back. Element selectors support matching by Title, Type, UUID, or gid.
//!
//! ## Element Selector Syntax
//!
//! ```text
//!   "My Control"       — match by Title (case-insensitive contains)
//!   "Type:WeatherData" — match all elements of a type
//!   "uuid:abc-123"     — match by UUID
//!   "gid:Mqtt"         — match by gid attribute
//! ```

mod blocks;
mod describe;
pub mod energy;
#[cfg(test)]
mod energy_tests;
pub mod icons;
mod layout;
pub mod lighting;
pub mod lighting_cid;
pub mod lighting_metadata;
#[cfg(test)]
mod lighting_metadata_tests;
pub mod lighting_sid;
#[cfg(test)]
mod lighting_tests;
mod permissions;
mod properties;
mod rooms;
pub mod schema;
mod selector;
mod template;
mod validation;
mod wiring;
mod write;

pub use blocks::VirtualInOpts;
pub use permissions::{PermissionChange, PermissionEdit, PrincipalKind};
pub use rooms::RoomRenamePreview;
pub use wiring::SpliceActorPreview;

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use xmltree::Element;

/// Connector map: type → (connector_names, defaults, io_types)
type ConnectorMap = HashMap<
    String,
    (
        Vec<String>,
        HashMap<String, String>,
        HashMap<String, String>,
    ),
>;

/// Describe entry: (type_name, title, scenes, wired_connectors)
type DescribeEntry = (String, String, Vec<String>, Vec<String>);

/// Structured room entry for JSON output.
#[derive(Debug, serde::Serialize)]
pub struct DescribeRoomEntry {
    pub room: String,
    pub blocks: Vec<DescribeBlockEntry>,
}

/// Structured block entry for JSON output.
#[derive(Debug, serde::Serialize)]
pub struct DescribeBlockEntry {
    pub block_type: String,
    pub title: String,
    pub uuid: String,
    pub connectors: Vec<DescribeConnectorEntry>,
}

/// Structured connector entry for JSON output.
#[derive(Debug, serde::Serialize)]
pub struct DescribeConnectorEntry {
    pub key: String,
    pub uuid: String,
    pub direction: String,
    pub wired: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Value-free storage shapes observed in a configuration, grouped by element type.
#[derive(Debug, serde::Serialize)]
pub struct ConfigSurfaceEntry {
    pub element_type: String,
    pub occurrences: usize,
    pub attributes: Vec<String>,
    pub storage_paths: Vec<String>,
}

/// A light scene / "mood" defined on a LightController2.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MoodInfo {
    pub name: String,
    pub uuid: String,
    /// Scene ID used to select the mood (e.g. via jdev `changeTo/<sid>`).
    pub sid: Option<i64>,
    pub cid: Option<i64>,
    /// Raw value of output 1 (`Q1`) — for a single RGBW actor this carries the
    /// packed scene colour value (a Loxone-internal encoding, not the simple
    /// `lox color` composite).
    pub q1: Option<String>,
}

/// Resolved endpoint for a config-wide wire.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConfigWireEndpoint {
    pub block_uuid: String,
    pub block_title: String,
    pub block_type: String,
    pub connector_uuid: String,
    pub connector_key: String,
}

/// Config-wide wire edge with source and target endpoints resolved.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConfigWire {
    pub source: ConfigWireEndpoint,
    pub target: ConfigWireEndpoint,
}

/// A connector exposed by a detected physical device.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DetectedDeviceConnector {
    pub uuid: String,
    pub role: String,
    pub channel_index: Option<u32>,
    #[serde(rename = "type")]
    pub connector_type: String,
}

/// Stable identity fields used to derive a physical-device key.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DetectedDeviceIdentity {
    pub bus_type: String,
    pub bus_serial: Option<String>,
    pub bus_address: Option<String>,
    pub channel_role: Option<String>,
}

/// Physical-device record detected from a Loxone config snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DetectedDevice {
    pub device_id: String,
    pub stable_device_key: String,
    pub bus_type: String,
    pub bus_serial: Option<String>,
    pub bus_address: Option<String>,
    pub device_type: String,
    pub primary_block_uuid: String,
    pub secondary_block_uuids: Vec<String>,
    pub connectors: Vec<DetectedDeviceConnector>,
    pub snapshot_room_label: Option<String>,
    pub derived_label: String,
    pub low_confidence_identity: bool,
    pub identity_components: DetectedDeviceIdentity,
}

/// Template result: (block_type, title, params)
type TemplateResult = Vec<(String, String, Vec<(String, String)>)>;

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Room completeness flags.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct RoomCompleteness {
    pub name: String,
    pub has_lighting: bool,
    pub has_blinds: bool,
    pub has_climate: bool,
    pub has_presence: bool,
}

/// Scene/mood info for a LightController2.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SceneInfo {
    pub control_name: String,
    pub mood_count: usize,
    pub mood_names: Vec<String>,
}

/// Device bus summary.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceBusSummary {
    pub bus_type: String,
    pub count: usize,
    pub device_names: Vec<String>,
}

/// Comprehensive config statistics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigStats {
    pub room_count: usize,
    pub page_count: usize,
    pub category_count: usize,
    pub total_items: usize,
    pub block_types: Vec<(String, usize)>,
    pub wiring_total: usize,
    pub wiring_cross_page: usize,
    pub wiring_multi_input: usize,
    pub room_completeness: Vec<RoomCompleteness>,
    pub scenes: Vec<SceneInfo>,
    pub devices: Vec<DeviceBusSummary>,
}

impl ConfigStats {
    /// Format stats as a human-readable text report.
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Config Statistics ===\n\n");

        // Overview
        out.push_str("Overview:\n");
        out.push_str(&format!("  Rooms:       {:>5}\n", self.room_count));
        out.push_str(&format!("  Pages:       {:>5}\n", self.page_count));
        out.push_str(&format!("  Categories:  {:>5}\n", self.category_count));
        out.push_str(&format!("  Total items: {:>5}\n", self.total_items));

        // Block types (top 20)
        if !self.block_types.is_empty() {
            out.push_str("\nBlock Types (top 20):\n");
            for (name, count) in self.block_types.iter().take(20) {
                out.push_str(&format!("  {:<24}{:>5}\n", name, count));
            }
            if self.block_types.len() > 20 {
                out.push_str(&format!(
                    "  ... and {} more types\n",
                    self.block_types.len() - 20
                ));
            }
        }

        // Wiring
        out.push_str("\nWiring:\n");
        out.push_str(&format!("  Total connections:  {:>5}\n", self.wiring_total));
        out.push_str(&format!(
            "  Cross-page (FLG=2):{:>5}\n",
            self.wiring_cross_page
        ));
        out.push_str(&format!(
            "  Multi-input:       {:>5}\n",
            self.wiring_multi_input
        ));

        // Room completeness
        if !self.room_completeness.is_empty() {
            out.push_str("\nRoom Completeness:\n");
            out.push_str(&format!("  {:<30} L B C P\n", "Room"));
            out.push_str(&format!("  {}\n", "─".repeat(38)));
            for rc in &self.room_completeness {
                let l = if rc.has_lighting { "✓" } else { "." };
                let b = if rc.has_blinds { "✓" } else { "." };
                let c = if rc.has_climate { "✓" } else { "." };
                let p = if rc.has_presence { "✓" } else { "." };
                let name = if rc.name.len() > 30 {
                    format!("{}…", &rc.name[..29])
                } else {
                    rc.name.clone()
                };
                out.push_str(&format!("  {:<30} {} {} {} {}\n", name, l, b, c, p));
            }
            out.push_str("  (L=Lighting B=Blinds C=Climate P=Presence)\n");
        }

        // Scenes
        if !self.scenes.is_empty() {
            out.push_str("\nScenes:\n");
            for si in &self.scenes {
                let preview: Vec<&str> = si.mood_names.iter().take(3).map(|s| s.as_str()).collect();
                let suffix = if si.mood_names.len() > 3 {
                    ", ...".to_string()
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "  {}: {} moods ({}{})\n",
                    si.control_name,
                    si.mood_count,
                    preview.join(", "),
                    suffix,
                ));
            }
        }

        // Devices
        if !self.devices.is_empty() {
            out.push_str("\nDevices:\n");
            for ds in &self.devices {
                let preview: Vec<&str> =
                    ds.device_names.iter().take(3).map(|s| s.as_str()).collect();
                let suffix = if ds.device_names.len() > 3 {
                    ", ...".to_string()
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "  {:<8}{:>3} ({}{})\n",
                    format!("{}:", ds.bus_type),
                    ds.count,
                    preview.join(", "),
                    suffix,
                ));
            }
        }

        out
    }
}

/// A Loxone config file loaded for editing.
pub struct ConfigEditor {
    pub root: Element,
    had_bom: bool,
    had_crlf: bool,
    /// Attributes renamed from digit-prefixed originals (e.g. `12hTF` → `_12hTF`).
    /// Stored as (sanitized, original) pairs for round-trip restoration in `to_bytes()`.
    digit_attr_renames: Vec<(String, String)>,
}

impl ConfigEditor {
    /// Load a .Loxone XML file for editing.
    ///
    /// Loxone Config can produce attributes with digit-prefixed names (e.g. `12hTF="true"`)
    /// which are technically invalid XML. We sanitize these before parsing and restore
    /// them on write-back.
    pub fn load(data: &[u8]) -> Result<Self> {
        let had_bom = data.starts_with(UTF8_BOM);
        // Prefer the newline after the XML declaration, or the first outside markup
        // if there is no declaration. Attribute values carry independent line breaks.
        let had_crlf = Self::has_crlf_line_endings(data);
        let xml_data = if had_bom { &data[3..] } else { data };

        let (xml_data_cow, digit_attr_renames) = Self::sanitize_digit_attrs(xml_data);
        // Loxone emits the same attribute twice on some elements (value-identical).
        // A strict parser rejects that; drop the redundant copy losslessly before parsing.
        let deduped = Self::dedupe_duplicate_attrs(xml_data_cow.as_ref());
        let reader = BufReader::new(Cursor::new(deduped.as_ref()));
        let root = Element::parse(reader).context("Failed to parse Loxone XML")?;
        Ok(ConfigEditor {
            root,
            had_bom,
            had_crlf,
            digit_attr_renames,
        })
    }

    fn has_crlf_line_endings(data: &[u8]) -> bool {
        let mut i = 0;
        while i < data.len() {
            if data[i] == b'\n' {
                return i > 0 && data[i - 1] == b'\r';
            }
            if data[i] == b'<' {
                let tail = &data[i..];
                let end = if tail.starts_with(b"<!--") {
                    tail.windows(3)
                        .position(|w| w == b"-->")
                        .map(|end| i + end + 3)
                } else if tail.starts_with(b"<![CDATA[") {
                    tail.windows(3)
                        .position(|w| w == b"]]>")
                        .map(|end| i + end + 3)
                } else if tail.starts_with(b"<?") {
                    tail.windows(2)
                        .position(|w| w == b"?>")
                        .map(|end| i + end + 2)
                } else {
                    Self::xml_tag_end(data, i)
                };
                if let Some(end) = end {
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
        false
    }

    /// Find attribute names starting with a digit and rename them with an underscore prefix.
    /// Returns the (possibly modified) data and a list of (sanitized, original) pairs.
    fn sanitize_digit_attrs(data: &[u8]) -> (std::borrow::Cow<'_, [u8]>, Vec<(String, String)>) {
        let text = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return (std::borrow::Cow::Borrowed(data), Vec::new()),
        };

        // Scan for ` <digit><word-chars>=` patterns (attribute with digit-prefixed name).
        let mut renames: Vec<(String, String)> = Vec::new();
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            // Look for a space/tab/newline followed by a digit
            if (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r')
                && i + 1 < len
                && bytes[i + 1].is_ascii_digit()
            {
                // Read the attribute name: digits and word chars until '='
                let start = i + 1;
                let mut end = start;
                while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                    end += 1;
                }
                if end < len && bytes[end] == b'=' && end > start {
                    let original = &text[start..end];
                    let sanitized = format!("_{original}");
                    if !renames.iter().any(|(_, o)| o == original) {
                        renames.push((sanitized, original.to_string()));
                    }
                }
            }
            i += 1;
        }

        if renames.is_empty() {
            return (std::borrow::Cow::Borrowed(data), renames);
        }

        // Apply replacements
        let mut result = text.to_string();
        for (sanitized, original) in &renames {
            // Replace ` <original>=` with ` _<original>=` — the leading space ensures
            // we only match attribute positions, not content.
            let from = format!(" {original}=");
            let to = format!(" {sanitized}=");
            result = result.replace(&from, &to);
        }
        (std::borrow::Cow::Owned(result.into_bytes()), renames)
    }

    /// Remove **value-identical** duplicate attributes from element start tags.
    ///
    /// Loxone Config emits the same attribute twice on some elements (observed on API
    /// Connector elements: `Title` appears twice with the same value). A strict XML parser
    /// rejects that ("Attribute 'Title' is redefined"). Since the duplicates are
    /// value-identical, dropping the redundant copy is lossless. Duplicates with *different*
    /// values are left untouched so a genuine conflict still surfaces at parse time.
    ///
    /// Both XML quote styles are supported; non-element markup is left untouched.
    fn dedupe_duplicate_attrs(data: &[u8]) -> std::borrow::Cow<'_, [u8]> {
        let text = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return std::borrow::Cow::Borrowed(data),
        };
        let bytes = text.as_bytes();
        let len = bytes.len();
        let ws = |b: u8| matches!(b, b' ' | b'\t' | b'\r' | b'\n');
        let mut deletes: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < len {
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }
            let tail = &text[i..];
            let skipped_end = if tail.starts_with("<!--") {
                tail.find("-->").map(|end| i + end + 3)
            } else if tail.starts_with("<![CDATA[") {
                tail.find("]]>").map(|end| i + end + 3)
            } else if tail.starts_with("<?") {
                tail.find("?>").map(|end| i + end + 2)
            } else if tail.starts_with("<!") {
                // A declaration can contain an internal subset with tag-like text.
                let mut quote = None;
                let mut depth: usize = 0;
                let mut end = None;
                for (offset, &byte) in bytes.iter().enumerate().skip(i + 2) {
                    if let Some(delimiter) = quote {
                        if byte == delimiter {
                            quote = None;
                        }
                    } else {
                        match byte {
                            b'"' | b'\'' => quote = Some(byte),
                            b'[' => depth += 1,
                            b']' => depth = depth.saturating_sub(1),
                            b'>' if depth == 0 => {
                                end = Some(offset + 1);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                end
            } else if tail.starts_with("</") {
                Self::xml_tag_end(bytes, i)
            } else {
                None
            };
            if let Some(end) = skipped_end {
                i = end;
                continue;
            }
            if i + 1 < len && !matches!(bytes[i + 1], b'/' | b'?' | b'!') {
                // Skip the element name.
                let mut j = i + 1;
                while j < len && !ws(bytes[j]) && !matches!(bytes[j], b'>' | b'/') {
                    j += 1;
                }
                // Scan attributes until the end of the start tag.
                let mut seen: Vec<(&str, &str)> = Vec::new();
                loop {
                    while j < len && ws(bytes[j]) {
                        j += 1;
                    }
                    if j >= len || matches!(bytes[j], b'>' | b'/') {
                        break;
                    }
                    let name_start = j;
                    while j < len && !ws(bytes[j]) && !matches!(bytes[j], b'=' | b'>' | b'/') {
                        j += 1;
                    }
                    let name_end = j;
                    while j < len && ws(bytes[j]) {
                        j += 1;
                    }
                    if j >= len || bytes[j] != b'=' {
                        break; // not `name=…`, stop scanning this tag
                    }
                    j += 1;
                    while j < len && ws(bytes[j]) {
                        j += 1;
                    }
                    if j >= len || !matches!(bytes[j], b'"' | b'\'') {
                        break;
                    }
                    let delimiter = bytes[j];
                    let val_start = j + 1;
                    j = val_start;
                    while j < len && bytes[j] != delimiter {
                        j += 1;
                    }
                    if j >= len {
                        break;
                    }
                    let val_end = j;
                    j += 1; // past closing quote
                    let attr_end = j;
                    let name = &text[name_start..name_end];
                    let value = &text[val_start..val_end];
                    if let Some(&(_, prev)) = seen.iter().find(|(n, _)| *n == name) {
                        if prev == value {
                            // Redundant copy → delete it plus one leading whitespace char.
                            let del_start = if name_start > 0 && ws(bytes[name_start - 1]) {
                                name_start - 1
                            } else {
                                name_start
                            };
                            deletes.push((del_start, attr_end));
                        }
                        // Different value: leave it, let the parser flag the conflict.
                    } else {
                        seen.push((name, value));
                    }
                }
                i = j;
                continue;
            }
            i += 1;
        }
        if deletes.is_empty() {
            return std::borrow::Cow::Borrowed(data);
        }
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for (s, e) in deletes {
            if s >= last {
                out.push_str(&text[last..s]);
                last = e;
            }
        }
        out.push_str(&text[last..]);
        std::borrow::Cow::Owned(out.into_bytes())
    }

    /// Get a mutable reference to an element by path.
    pub fn get_element_mut(&mut self, path: &[usize]) -> &mut Element {
        let mut current = &mut self.root;
        for &idx in path {
            current = current.children[idx].as_mut_element().unwrap();
        }
        current
    }

    /// Get a reference to an element by path.
    pub fn get_element(&self, path: &[usize]) -> &Element {
        let mut current = &self.root;
        for &idx in path {
            current = current.children[idx].as_element().unwrap();
        }
        current
    }

    fn iter_elements(elem: &Element) -> Vec<&Element> {
        let mut result = vec![elem];
        for child in &elem.children {
            if let Some(e) = child.as_element() {
                result.extend(Self::iter_elements(e));
            }
        }
        result
    }
}

fn remove_by_uuid(children: &mut Vec<xmltree::XMLNode>, uuid: &str) -> Result<String> {
    for i in 0..children.len() {
        if let Some(elem) = children[i].as_element()
            && elem.attributes.get("U").map(|u| u == uuid).unwrap_or(false)
        {
            let title = elem.attributes.get("Title").cloned().unwrap_or_default();
            children.remove(i);
            return Ok(title);
        }
    }
    for child in children.iter_mut() {
        if let Some(elem) = child.as_mut_element()
            && let Ok(title) = remove_by_uuid(&mut elem.children, uuid)
        {
            return Ok(title);
        }
    }
    bail!("Element with UUID '{}' not found", uuid)
}

/// Check if an element matches a selector string.
fn matches_selector(elem: &Element, selector: &str) -> bool {
    if elem.name != "C" {
        return false;
    }

    if let Some(uuid) = selector.strip_prefix("uuid:") {
        return elem.attributes.get("U").map(|u| u == uuid).unwrap_or(false);
    }
    if let Some(gid) = selector.strip_prefix("gid:") {
        return elem
            .attributes
            .get("gid")
            .map(|g| g.eq_ignore_ascii_case(gid))
            .unwrap_or(false);
    }
    if let Some(type_name) = selector.strip_prefix("Type:") {
        return elem
            .attributes
            .get("Type")
            .map(|t| t.eq_ignore_ascii_case(type_name))
            .unwrap_or(false);
    }

    // Bracket syntax: "Title [Room]" — match by title AND room name
    if let Some(bracket_start) = selector.rfind('[')
        && selector.ends_with(']')
    {
        let title_part = selector[..bracket_start].trim();
        let _room_part = &selector[bracket_start + 1..selector.len() - 1];
        let title_match = elem
            .attributes
            .get("Title")
            .map(|t| t.to_lowercase().contains(&title_part.to_lowercase()))
            .unwrap_or(false);
        if !title_match {
            return false;
        }
        // Check room via IoData Pr attribute — walk children for IoData
        let room_uuid = elem.children.iter().find_map(|c| {
            c.as_element()
                .filter(|e| e.name == "IoData")
                .and_then(|e| e.attributes.get("Pr").cloned())
        });
        if let Some(_ru) = room_uuid {
            // We need to check if this room UUID matches the room name.
            // Store the room_part for post-filtering in require_one.
            // For now, embed room UUID match in selector via a static approach:
            // We can't access the full tree here, so use a simpler heuristic:
            // the room_part will be checked by the caller after find_elements.
            return true; // Title matches, room will be filtered by caller
        }
        return false;
    }

    // Default: match by Title (case-insensitive contains)
    elem.attributes
        .get("Title")
        .map(|t| t.to_lowercase().contains(&selector.to_lowercase()))
        .unwrap_or(false)
}

// ── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct PropertyValue {
    pub value: String,
    pub type_code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ConnectorInfo {
    pub kind: String,
    pub target: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ElementDescription {
    pub element_type: String,
    pub title: String,
    pub uuid: String,
    pub gid: String,
    pub room_uuid: String,
    pub category_uuid: String,
    pub properties: HashMap<String, PropertyValue>,
    pub connectors: Vec<ConnectorInfo>,
    pub children: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct WireInfo {
    pub connector: String,
    pub direction: String,
    pub target_uuid: String,
    pub connected: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct MqttTopic {
    pub title: String,
    pub direction: String,
    pub topic: String,
    pub qos: String,
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-1" Title="Kitchen" WF="16384"/>
  <C Type="Place" V="175" U="room-2" Title="Zentral" WF="16384"/>
  <C Type="Category" V="175" U="cat-1" Title="Wetter"/>
  <C Type="Plugin" gid="Mqtt" U="mqtt-1" Title="MQTT">
    <SET>
      <mqtt_broker_address t="11" v="192.168.1.1"/>
      <mqtt_broker_port t="7" v="1883"/>
    </SET>
    <C Type="GenTSensor" U="sensor-1" Title="Temp Sub">
      <Co K="Text" U="co-sensor1-text"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
  </C>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <Co K="AQ" U="co-wd1-aq"/>
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="WeatherData" U="wd-2" Title="Wind">
    <Co K="AQ" U="co-wd2-aq"/>
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
  <C Type="SysVar" U="sv-1" Title="Aussentemp">
    <Co K="AQ" U="co-sv1-aq"/>
    <Co K="AI" U="co-sv1-ai"/>
    <Co K="Q" U="co-sv1-q"/>
    <IoData Cr="cat-1" Pr="room-1"/>
  </C>
</ControlList>"#;

    #[test]
    fn test_load_and_write_preserves_content() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let output = editor.to_bytes().unwrap();
        // Verify it's valid XML
        let _ = ConfigEditor::load(&output).unwrap();
    }

    #[test]
    fn test_dedupe_value_identical_duplicate_attr() {
        // Loxone emits the same attribute twice, value-identical → must parse, redundant copy dropped.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" U="room-1" Title="Kitchen" WF="16384" Title="Kitchen"/>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let out = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert_eq!(out.matches(r#"Title="Kitchen""#).count(), 1);
    }

    #[test]
    fn test_dedupe_leaves_conflicting_duplicate() {
        // Different values = a real conflict; must still surface at parse time.
        let xml = br#"<ControlList><C U="x" Title="A" Title="B"/></ControlList>"#;
        assert!(ConfigEditor::load(xml).is_err());
    }

    #[test]
    fn test_dedupe_is_quote_aware() {
        // A value may contain escaped angle brackets (e.g. Unit="&lt;v&gt;%"); tag scanning
        // must not be confused, and an unrelated value-identical dup still collapses.
        let xml = br#"<ControlList>
  <C U="x" Unit="&lt;v&gt;%" Nio="1" Nio="1"/>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let out = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert_eq!(out.matches(r#"Nio="1""#).count(), 1);
        assert!(out.contains("&lt;v&gt;%"));
    }

    #[test]
    fn test_dedupe_supports_both_quotes_and_xml_names() {
        let xml = "<ControlList><C.entry Title='Küche' Title=\"Küche\" \
Unit='a > b' Unit='a > b'/><部屋 Title='同じ' Title='同じ'/></ControlList>";
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let child = editor.root.get_child("C.entry").unwrap();
        assert_eq!(child.attributes["Title"], "Küche");
        assert_eq!(child.attributes["Unit"], "a > b");
        assert_eq!(
            editor.root.get_child("部屋").unwrap().attributes["Title"],
            "同じ"
        );
        assert!(ConfigEditor::load(b"<C Title='A' Title=\"B\"/>").is_err());
    }

    #[test]
    fn test_dedupe_preserves_non_element_markup() {
        let xml = br#"<?xml version="1.0"?>
<!DOCTYPE C [<!ENTITY example '<C A="x" A="x"/>'>]>
<C A="x" A="x">
<!-- <C A="x" A="x"/> -->
<![CDATA[<C A="x" A="x"/>]]>
<?test <C A="x" A="x"/>?>
</C>"#;
        let out = ConfigEditor::dedupe_duplicate_attrs(xml);
        let expected = String::from_utf8(xml.to_vec())
            .unwrap()
            .replace("\n<C A=\"x\" A=\"x\">", "\n<C A=\"x\">");
        assert_eq!(out.as_ref(), expected.as_bytes());
    }

    #[test]
    fn test_bom_preservation() {
        let mut with_bom = Vec::new();
        with_bom.extend_from_slice(UTF8_BOM);
        with_bom.extend_from_slice(SAMPLE_XML);
        let editor = ConfigEditor::load(&with_bom).unwrap();
        assert!(editor.had_bom);
        let output = editor.to_bytes().unwrap();
        assert!(output.starts_with(UTF8_BOM));
    }

    #[test]
    fn test_find_by_title() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("Temperatur");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_by_gid() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("gid:Mqtt");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_by_type() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("Type:WeatherData");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_by_uuid() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("uuid:wd-1");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_set_property_update() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let msg = editor
            .set_property("gid:Mqtt", "mqtt_broker_address", "10.0.0.1", "11")
            .unwrap();
        assert!(msg.contains("10.0.0.1"));

        let desc = editor.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.properties["mqtt_broker_address"].value, "10.0.0.1");
    }

    #[test]
    fn test_set_property_create() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        editor
            .set_property("gid:Mqtt", "mqtt_auth_pwd", "secret", "11")
            .unwrap();

        let desc = editor.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.properties["mqtt_auth_pwd"].value, "secret");
    }

    #[test]
    fn test_set_attribute() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        editor
            .set_attribute("uuid:wd-1", "Title", "Neue Temperatur")
            .unwrap();

        let matches = editor.find_elements("Neue Temperatur");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_move_to_room() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let (count, uuid) = editor.move_to_room("WeatherData", "Zentral", &[]).unwrap();
        assert_eq!(count, 2);
        assert_eq!(uuid, "room-2");

        // Verify IoData was updated
        let output = editor.to_bytes().unwrap();
        let check = ConfigEditor::load(&output).unwrap();
        let desc = check.describe("uuid:wd-1").unwrap();
        assert_eq!(desc.room_uuid, "room-2");
    }

    #[test]
    fn test_describe() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let desc = editor.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.element_type, "Plugin");
        assert_eq!(desc.title, "MQTT");
        assert!(!desc.properties.is_empty());
        assert_eq!(desc.properties["mqtt_broker_address"].value, "192.168.1.1");
        assert_eq!(desc.children.len(), 1);
    }

    #[test]
    fn test_add_room() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor.add_room("Garten").unwrap();
        assert!(!uuid.is_empty());

        let matches = editor.find_elements("Garten");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_add_room_duplicate_fails() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.add_room("Kitchen");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_write_read() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        editor
            .set_property("gid:Mqtt", "mqtt_broker_address", "10.0.0.1", "11")
            .unwrap();
        editor
            .set_attribute("uuid:wd-1", "Title", "NewTemp")
            .unwrap();
        let (count, _) = editor.move_to_room("WeatherData", "Zentral", &[]).unwrap();
        assert_eq!(count, 2);

        let output = editor.to_bytes().unwrap();
        let check = ConfigEditor::load(&output).unwrap();

        let desc = check.describe("gid:Mqtt").unwrap();
        assert_eq!(desc.properties["mqtt_broker_address"].value, "10.0.0.1");

        let matches = check.find_elements("NewTemp");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_check_blocks_warns_on_missing_layout() {
        // A page block without canvas coordinates should be flagged, because
        // Loxone Config will "repair" it on first save and may drop wires.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="And" V="175" U="and-nocoords" Title="No Coords" Nio="3">
      <Co K="I1" U="and-nc-i1" Def="1"/>
      <Co K="I2" U="and-nc-i2" Def="1"/>
      <Co K="Q" U="and-nc-q"/>
    </C>
    <C Type="And" V="175" U="and-coords" Title="Has Coords" Nio="3" Px="7392" Py="576" Px2="8736" Py2="1272" Cl="0,0,0">
      <Co K="I1" U="and-c-i1" Def="1"/>
      <Co K="I2" U="and-c-i2" Def="1"/>
      <Co K="Q" U="and-c-q"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();

        let no_coords = editor.check_blocks(Some("No Coords"));
        assert!(
            no_coords
                .iter()
                .any(|r| r.contains("missing canvas layout")),
            "expected layout warning, got: {no_coords:?}"
        );

        let has_coords = editor.check_blocks(Some("Has Coords"));
        assert!(
            !has_coords
                .iter()
                .any(|r| r.contains("missing canvas layout")),
            "block with coords should not warn, got: {has_coords:?}"
        );
    }

    #[test]
    fn test_check_blocks_warns_on_nio_mismatch() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="And" V="175" U="and-bad-nio" Title="Bad Nio" Nio="7" Px="7392" Py="576" Px2="8736" Py2="1272" Cl="0,0,0">
      <Co K="I1" U="and-bn-i1" Def="1"/>
      <Co K="I2" U="and-bn-i2" Def="1"/>
      <Co K="Q" U="and-bn-q"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.check_blocks(Some("Bad Nio"));
        assert!(
            results.iter().any(|r| r.contains("Nio=7 but 3 connectors")),
            "expected Nio mismatch warning, got: {results:?}"
        );
    }

    #[test]
    fn test_check_blocks_warns_on_low_pressure_maxval() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="VirtualIn" V="175" U="vi-druck" Title="Luftdruck" Analog="true" MaxVal="1000" Unit="hPa">
    <Co K="Q" U="vi-druck-q"/>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.check_blocks(None);
        assert!(
            results
                .iter()
                .any(|r| r.contains("below normal sea-level pressure")),
            "expected low-MaxVal pressure warning, got: {results:?}"
        );
    }

    #[test]
    fn test_check_blocks_warns_on_too_many_programs() {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<C Type=\"Document\" V=\"175\" U=\"doc-1\" Title=\"Root\">\n",
        );
        for i in 0..9 {
            xml.push_str(&format!(
                "  <C Type=\"Code1\" V=\"175\" U=\"code-{i}\" Title=\"Prog{i}\"><Co K=\"AI\" U=\"c-{i}-ai\"/><Co K=\"AQ\" U=\"c-{i}-aq\"/></C>\n"
            ));
        }
        xml.push_str("</C>");
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let results = editor.check_blocks(None);
        assert!(
            results
                .iter()
                .any(|r| r.contains("executes only the first 8") && r.contains("Prog8")),
            "expected program-limit warning naming the 9th block, got: {results:?}"
        );
    }

    #[test]
    fn test_check_blocks_layout_skips_hidden_refs_and_nested_elements() {
        // Hidden refs (WF without the 0x20000 "placed on page" bit), Keycodes
        // nested in an NfcCodeTouch and elements outside any Page have no
        // canvas coordinates by design. A placed ref or a normal block that
        // lacks coords is still a real problem.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="VirtualIn" V="175" U="vi-1" Title="Off Page VI">
    <Co K="Q" U="vi-1-q"/>
  </C>
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="InputRef" V="175" U="ir-hidden" Title="Hidden In" WF="22528" Ref="vi-1" Nio="4">
      <Co K="AI" U="ir-h-ai"/><Co K="I" U="ir-h-i"/><Co K="AQ" U="ir-h-aq"/><Co K="Q" U="ir-h-q"/>
    </C>
    <C Type="OutputRef" V="175" U="or-hidden" Title="Hidden Out" WF="20480" Ref="vi-1" Nio="2">
      <Co K="AI" U="or-h-ai"/><Co K="AQ" U="or-h-aq"/>
    </C>
    <C Type="InputRef" V="175" U="ir-placed" Title="Placed In" WF="155648" Ref="vi-1" Nio="4">
      <Co K="AI" U="ir-p-ai"/><Co K="I" U="ir-p-i"/><Co K="AQ" U="ir-p-aq"/><Co K="Q" U="ir-p-q"/>
    </C>
    <C Type="NfcCodeTouch" V="175" U="nfc-1" Title="Door Keypad" Nio="1" Px="100" Py="100" Px2="200" Py2="200">
      <Co K="Disable" U="nfc-1-dis"/>
      <C Type="Keycode" V="175" U="kc-1" Title="Guest Code" WF="20480">
        <Co K="Q" U="kc-1-q"/>
      </C>
    </C>
    <C Type="And" V="175" U="and-nocoords" Title="Unplaced And" Nio="3">
      <Co K="I1" U="and-i1" Def="1"/><Co K="I2" U="and-i2" Def="1"/><Co K="Q" U="and-q"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.check_blocks(None);
        let layout: Vec<&String> = results
            .iter()
            .filter(|r| r.contains("missing canvas layout"))
            .collect();
        for quiet in ["Hidden In", "Hidden Out", "Guest Code", "Off Page VI"] {
            assert!(
                !layout.iter().any(|r| r.contains(quiet)),
                "{quiet} should not get a layout warning, got: {layout:?}"
            );
        }
        for loud in ["Placed In", "Unplaced And"] {
            assert!(
                layout.iter().any(|r| r.contains(loud)),
                "{loud} should get a layout warning, got: {layout:?}"
            );
        }
    }

    #[test]
    fn test_check_blocks_optional_unwired_inputs_are_not_errors() {
        // Most block inputs are optional: a partially wired State block or a
        // PushButton2 driven only from the app is normal. A pure logic block
        // whose output is used but has no input is still an error.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="VirtualIn" V="175" U="src" Title="Source" Px="1" Py="1" Px2="2" Py2="2" Nio="1">
      <Co K="Q" U="src-q"/>
    </C>
    <C Type="State" V="175" U="st-1" Title="Partly Wired State" Nio="4" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="I1" U="st-i1"><In Input="src-q"/></Co>
      <Co K="I2" U="st-i2"/>
      <Co K="TQ" U="st-tq"/>
      <Co K="AQ" U="st-aq"/>
    </C>
    <C Type="PushButton2" V="175" U="pb-1" Title="App Button" Nio="3" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="InputTriggerOn" U="pb-on"/>
      <Co K="InputTriggerOff" U="pb-off"/>
      <Co K="Q" U="pb-q"/>
    </C>
    <C Type="Not" V="175" U="not-1" Title="Consumer" Nio="2" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="I" U="not-i"><In Input="pb-q"/></Co>
      <Co K="Q" U="not-q"/>
    </C>
    <C Type="And" V="175" U="and-1" Title="Starved And" Nio="3" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="I1" U="and-i1"/>
      <Co K="I2" U="and-i2"/>
      <Co K="Q" U="and-q"/>
    </C>
    <C Type="Or" V="175" U="or-1" Title="And Consumer" Nio="2" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="I1" U="or-i1"><In Input="and-q"/></Co>
      <Co K="Q" U="or-q"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.check_blocks(None);
        assert!(
            !results.iter().any(|r| r.contains("unwired inputs")),
            "optional unwired inputs must not be reported, got: {results:?}"
        );
        for quiet in ["Partly Wired State", "App Button"] {
            assert!(
                !results
                    .iter()
                    .any(|r| r.starts_with('✗') && r.contains(quiet)),
                "{quiet} should not be an error, got: {results:?}"
            );
        }
        assert!(
            results.iter().any(|r| r.starts_with('✗')
                && r.contains("Starved And")
                && r.contains("no inputs wired")),
            "logic block with used output and no input must still be an error, got: {results:?}"
        );
    }

    #[test]
    fn test_check_blocks_refs_not_reported_as_unconnected() {
        // An InputRef is fed by its Ref attribute; its AI link may live on
        // another page, so missing page wires do not make it unconnected.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="InputRef" V="175" U="ir-1" Title="Remote Sensor" WF="155648" Ref="elsewhere" Nio="4" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="AI" U="ir-ai"/><Co K="I" U="ir-i"/><Co K="AQ" U="ir-aq"/><Co K="Q" U="ir-q"/>
    </C>
    <C Type="OutputRef" V="175" U="or-1" Title="Remote Actor" WF="139264" Ref="elsewhere" Nio="2" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="AI" U="or-ai"/><Co K="AQ" U="or-aq"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.check_blocks(None);
        assert!(
            !results
                .iter()
                .any(|r| r.contains("Remote Sensor") || r.contains("Remote Actor")),
            "refs must not be reported, got: {results:?}"
        );
    }

    #[test]
    fn test_check_blocks_comparator_input_set_by_wire_or_def() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="VirtualIn" V="175" U="src" Title="Source" Px="1" Py="1" Px2="2" Py2="2" Nio="1">
      <Co K="Q" U="src-q"/>
    </C>
    <C Type="GreaterEqual" V="175" U="ge-wired" Title="Wired Threshold" Nio="3" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="Input1" U="gw-1"><In Input="src-q"/></Co>
      <Co K="Input2" U="gw-2"><In Input="src-q" FLG="2"/></Co>
      <Co K="Q" U="gw-q"/>
    </C>
    <C Type="GreaterEqual" V="175" U="ge-def" Title="Constant Threshold" Nio="3" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="Input1" U="gd-1"><In Input="src-q"/></Co>
      <Co K="Input2" U="gd-2" Def="21"/>
      <Co K="Q" U="gd-q"/>
    </C>
    <C Type="GreaterEqual" V="175" U="ge-none" Title="Missing Threshold" Nio="3" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="Input1" U="gn-1"><In Input="src-q"/></Co>
      <Co K="Input2" U="gn-2"/>
      <Co K="Q" U="gn-q"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.check_blocks(None);
        let not_set: Vec<&String> = results
            .iter()
            .filter(|r| r.contains("Input2 not set"))
            .collect();
        assert!(
            !not_set
                .iter()
                .any(|r| r.contains("Wired Threshold") || r.contains("Constant Threshold")),
            "wired or constant Input2 counts as set, got: {not_set:?}"
        );
        assert!(
            not_set
                .iter()
                .any(|r| r.starts_with('⚠') && r.contains("Missing Threshold")),
            "Input2 with neither Def nor wire must be reported as a warning, got: {results:?}"
        );
    }

    #[test]
    fn test_validate_api_connector_with_input_is_not_flagged() {
        // A virtual text input feeding a block's API connector sends it commands.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="VirtualTextIn" U="vti-1" Title="Webhook">
    <Co K="AQ" U="vti-aq"/>
  </C>
  <C Type="Doorcontroller" U="dc-1" Title="Front Door">
    <Co K="Q1" U="dc-q1"/>
    <Co K="OutputAPI" U="dc-api"><In Input="vti-aq"/></Co>
  </C>
</ControlList>"#;
        let results = ConfigEditor::load(xml).unwrap().validate_config();
        assert!(
            !results
                .iter()
                .any(|r| r.contains("wired TO an Output connector")),
            "{results:?}"
        );
    }

    #[test]
    fn test_validate_direction_still_checks_user_wires_alongside_managed_links() {
        for (extra, warns) in [("", false), (r#"<In Input="user-source"/>"#, true)] {
            let xml = format!(
                r#"<ControlList>
                  <C Type="And" U="gate" Title="Gate">
                    <Co K="Q" U="output">
                      <In Input="managed-source" FLG="1"/>{extra}
                    </Co>
                  </C>
                </ControlList>"#
            );
            let results = ConfigEditor::load(xml.as_bytes())
                .unwrap()
                .validate_config();
            assert_eq!(
                results
                    .iter()
                    .any(|r| r.contains("wired TO an Output connector")),
                warns,
                "{results:?}"
            );
        }
    }

    #[test]
    fn test_check_and_validate_ignore_autopilot_managed_inputs() {
        // AutoPilot conditions live in the rule's APC JSON; its inputs are
        // Config-managed (FLG="1") links. Nothing here is a wiring problem.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="Document" V="175" U="doc-1" Title="Root">
  <C Type="AutoPilot" V="175" U="ap-1" Title="Automatic Rules" WF="16384" Nio="2">
    <Co K="Input" U="ap-in"/>
    <Co K="output" U="ap-out"/>
  </C>
  <C Type="Page" V="175" U="page-1" Title="Logic" WF="16384">
    <C Type="AutopilotRule" V="175" U="apr-1" Title="Night Rule" APC="{&quot;conditions&quot;:[]}" Nio="6" Px="1" Py="1" Px2="2" Py2="2">
      <Co K="InputTrigger" U="apr-trig"/>
      <Co K="Reset" U="apr-reset"><In Input="ap-out" FLG="1"/></Co>
      <Co K="On" U="apr-on"/>
      <Co K="Run" U="apr-run"/>
      <Co K="Q" U="apr-q"><In Input="ap-out" FLG="1"/></Co>
      <Co K="A" U="apr-a"/>
    </C>
  </C>
</C>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let check = editor.check_blocks(None);
        assert!(
            !check
                .iter()
                .any(|r| (r.starts_with('✗') || r.starts_with('⚠'))
                    && (r.contains("Night Rule") || r.contains("Automatic Rules"))),
            "AutoPilot must not be reported by check, got: {check:?}"
        );
        let validate = editor.validate_config();
        assert!(
            !validate.iter().any(|r| r.contains("wired TO an Output")),
            "FLG=1 links are Config-managed, got: {validate:?}"
        );
    }

    #[test]
    fn test_validate_config_uuid_suffixes_are_not_serials() {
        // The last UUID group identifies the Config installation that created
        // the element; projects legitimately mix many suffixes.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Document" U="1fbf3415-030f-bc7d-ffff5ff4953f8cf6" Title="Doc">
    <C Type="Page" U="2a3b4c5d-0001-0002-ffffa2b891a5bde7" Title="Page">
      <C Type="And" U="3a3b4c5d-0001-0002-ffff123456789abc" Title="Gate">
        <Co K="I1" U="3a3b4c5d-0001-0003-00ff1748ea0f73fe"/>
        <Co K="Q" U="3a3b4c5d-0001-0004-01ff1748ea0f73fe"/>
      </C>
    </C>
    <C Type="Mode" U="00000000-0000-0002-1" Title="Mode"/>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.validate_config();
        assert!(
            !results
                .iter()
                .any(|r| r.contains("serial") || r.contains("malformed UUID")),
            "mixed suffixes must not be flagged, got: {results:?}"
        );
        assert!(results.iter().any(|r| r == "✓ All UUIDs are well-formed"));

        let bad = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="And" U="3a3b4c5d-0001-0002-zzzz123456789abc" Title="Broken">
    <Co K="Q" U="not-a-uuid"/>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(bad).unwrap();
        let results = editor.validate_config();
        assert!(
            results.iter().any(|r| r.starts_with('⚠')
                && r.contains("Broken")
                && r.contains("malformed UUID")),
            "malformed element UUID must be reported, got: {results:?}"
        );
        assert!(
            results
                .iter()
                .any(|r| r.contains("connector Q") && r.contains("malformed UUID")),
            "malformed connector UUID must be reported, got: {results:?}"
        );
    }

    #[test]
    fn test_add_element() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor
            .add_element(
                "gid:Mqtt",
                "GenTSensor",
                "New Sensor",
                Some("Mqtt.subt"),
                Some("room-2"),
                Some("cat-1"),
                &[("mqtt_topic", "test/topic", "11")],
            )
            .unwrap();
        assert!(!uuid.is_empty());

        let desc = editor.describe(&format!("uuid:{}", uuid)).unwrap();
        assert_eq!(desc.element_type, "GenTSensor");
        assert_eq!(desc.title, "New Sensor");
        assert_eq!(desc.properties["mqtt_topic"].value, "test/topic");
    }

    #[test]
    fn test_add_element_emits_layout_and_ordered_connectors() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor
            .add_element("gid:Mqtt", "Formula", "Tank Level", None, None, None, &[])
            .unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();

        // Isolate the new Formula block.
        let start = xml.find(&format!("U=\"{}\"", uuid)).unwrap();
        let block_open = xml[..start].rfind("<C ").unwrap();
        let block_end = xml[start..].find("</C>").unwrap() + start;
        let block = &xml[block_open..block_end];

        // Layout coordinates must be present so Loxone Config doesn't "repair"
        // the block (and drop wires) on first save.
        for attr in ["Px=", "Py=", "Px2=", "Py2=", "Cl="] {
            assert!(block.contains(attr), "missing {attr} in: {block}");
        }
        assert!(block.contains("Nio=\"4\""), "Nio should be 4: {block}");

        // Connectors must be inputs/params first, then outputs.
        let pos = |k: &str| block.find(&format!("K=\"{k}\"")).unwrap();
        assert!(pos("Input1") < pos("AQ"), "Input1 before AQ");
        assert!(pos("Input2") < pos("AQ"), "Input2 before AQ");
        assert!(pos("AQ") < pos("TQ"), "AQ before TQ");
    }

    #[test]
    fn test_order_connectors_inputs_first() {
        let mut types = std::collections::HashMap::new();
        types.insert("AQ".to_string(), "O".to_string());
        types.insert("TQ".to_string(), "O".to_string());
        types.insert("Input1".to_string(), "P".to_string());
        types.insert("Input2".to_string(), "P".to_string());
        let raw = vec![
            "AQ".to_string(),
            "TQ".to_string(),
            "Input1".to_string(),
            "Input2".to_string(),
        ];
        let ordered = ConfigEditor::order_connectors(&raw, &types);
        assert_eq!(ordered, vec!["Input1", "Input2", "AQ", "TQ"]);
    }

    #[test]
    fn test_remove_element() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        assert_eq!(editor.find_elements("uuid:wd-2").len(), 1);
        let title = editor.remove_element("wd-2").unwrap();
        assert_eq!(title, "Wind");
        assert_eq!(editor.find_elements("uuid:wd-2").len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_fails() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.remove_element("nonexistent-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_mqtt_topics() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // The sample has one GenTSensor but no mqtt_topic property set
        let topics = editor.list_mqtt_topics();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].direction, "subscribe");
        assert_eq!(topics[0].title, "Temp Sub");

        // Add one with a topic
        editor
            .add_element(
                "gid:Mqtt",
                "GenTActor",
                "Publisher",
                Some("Mqtt.pubt"),
                None,
                None,
                &[("mqtt_topic", "home/status", "11")],
            )
            .unwrap();
        let topics = editor.list_mqtt_topics();
        assert_eq!(topics.len(), 2);
        let pub_topic = topics.iter().find(|t| t.direction == "publish").unwrap();
        assert_eq!(pub_topic.topic, "home/status");
    }

    #[test]
    fn test_wire_connector_replaces_and_strips_uuid_prefix() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // Wire SysVar.AI ← WeatherData(Temperatur).AQ
        editor
            .wire_connector("uuid:sv-1", "AI", "co-wd1-aq", false)
            .unwrap();
        // Re-wire with a uuid: prefixed source, default (replace) mode.
        editor
            .wire_connector("uuid:sv-1", "AI", "uuid:co-wd2-aq", false)
            .unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        // The prefix is stripped and the old source replaced (single In).
        assert!(xml.contains(r#"Input="co-wd2-aq""#));
        assert!(!xml.contains("uuid:co-wd2-aq"));
        assert!(!xml.contains(r#"Input="co-wd1-aq""#));
        assert_eq!(xml.matches(r#"Input="co-wd2-aq""#).count(), 1);

        // --add appends an additional source rather than replacing.
        editor
            .wire_connector("uuid:sv-1", "AI", "co-wd1-aq", true)
            .unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"Input="co-wd1-aq""#));
        assert!(xml.contains(r#"Input="co-wd2-aq""#));
    }

    #[test]
    fn test_splice_actor_repoints_outputref_ai() {
        const SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="OutputRef" U="oref-1" Title="RGBW">
    <Co K="AI" U="oref-1-ai" Nc="1"><In Input="lc-aq1"/></Co>
    <Co K="AQ" U="oref-1-aq"/>
  </C>
  <C Type="LoxAIRAactor" U="actor-1" Title="RGBW Actor">
    <Co K="I" U="actor-1-i" Nc="1"><In Input="oref-1-aq"/></Co>
  </C>
</ControlList>"#;

        // Splice via the actor selector — the driving OutputRef is found
        // automatically (actor.I ← OutputRef.AQ).
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        let oref = editor.splice_actor("uuid:actor-1", "newsrc").unwrap();
        assert_eq!(oref, "oref-1");
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"Input="newsrc""#));
        assert!(!xml.contains(r#"Input="lc-aq1""#));

        // Splice via the OutputRef block directly also works.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        editor.splice_actor("uuid:oref-1", "uuid:other").unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"Input="other""#));
        assert!(!xml.contains(r#"Input="lc-aq1""#));
    }

    #[test]
    fn test_list_moods() {
        const SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="LightController2" U="lc-1" Title="Lichtsteuerung">
    <LightscenesC FC="0040FFFF" Outputs="1" Num="2">
      <LightsceneC Name="Entspannen" UUID="m-1" SID="1" CID="8" Q1="1613645813"/>
      <LightsceneC Name="Aus" UUID="m-2" SID="778" CID="7" Q1="0"/>
    </LightscenesC>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(SAMPLE).unwrap();
        let moods = editor.list_moods("uuid:lc-1").unwrap();
        assert_eq!(moods.len(), 2);
        assert_eq!(moods[0].name, "Entspannen");
        assert_eq!(moods[0].sid, Some(1));
        assert_eq!(moods[1].name, "Aus");
        assert_eq!(moods[1].sid, Some(778));
        // Non-light-controller selector is an error.
        let editor2 = ConfigEditor::load(SAMPLE_XML).unwrap();
        assert!(editor2.list_moods("uuid:wd-1").is_err());
    }

    #[test]
    fn test_set_mood_color() {
        const SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="LightController2" U="lc-1" Title="Lichtsteuerung">
    <LightscenesC FC="0040FFFF" Outputs="1" Num="2">
      <LightsceneC Name="TeamsGruen" UUID="m-1" SID="1" CID="8" Q1="1610617736"/>
      <LightsceneC Name="Aus" UUID="m-2" SID="778" CID="7" Q1="0"/>
    </LightscenesC>
  </C>
</ControlList>"#;
        // Match by Name, set Q1.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        editor
            .set_mood_color("uuid:lc-1", "TeamsGruen", 1, 1_610_622_736)
            .unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"Q1="1610622736""#));
        assert!(!xml.contains(r#"Q1="1610617736""#));

        // Match by SID.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        editor.set_mood_color("uuid:lc-1", "778", 1, 42).unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"SID="778" CID="7" Q1="42""#) || xml.contains(r#"Q1="42""#));

        // Unknown mood is an error.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        assert!(editor.set_mood_color("uuid:lc-1", "Nope", 1, 1).is_err());
    }

    #[test]
    fn test_add_mood() {
        const SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="LightController2" U="lc-1" Title="Lichtsteuerung">
    <LightscenesC FC="0040FFFF" Outputs="3" Num="3">
      <LightsceneC Name="Entspannen" UUID="m-1" SID="1" CID="8" Q1="1613645813" Q2="0" Q3="0"/>
      <LightsceneC Name="Aus" UUID="m-2" SID="778" CID="7" Q1="0" Q2="0" Q3="0"/>
      <LightsceneC Name="Green" UUID="m-3" SID="2" CID="9" Q1="1610617736" Q2="0" Q3="0"/>
    </LightscenesC>
  </C>
</ControlList>"#;

        // Native automatic SID/CID = lowest free SID and its available CID candidate.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        let (name, uuid, sid, cid) = editor
            .add_mood("uuid:lc-1", "Purple", 1_640_612_766, None, None)
            .unwrap();
        assert_eq!(name, "Purple");
        assert!(!uuid.is_empty());
        assert_eq!(sid, 3);
        assert_eq!(cid, 14);
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"Name="Purple""#));
        assert!(xml.contains(r#"SID="3""#));
        assert!(xml.contains(r#"CID="14""#));
        assert!(xml.contains(r#"Q1="1640612766""#));
        // Num bumped 3 -> 4.
        assert!(xml.contains(r#"Num="4""#));
        // New native-style width is independent of the stored collection width.
        assert!(xml.contains(r#"Outputs="18""#));
        assert!(xml.contains(r#"Q18="0""#));
        assert!(xml.contains(r#"Q3="0""#));

        // Explicit SID/CID overrides.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        let (_, _, sid, cid) = editor
            .add_mood("uuid:lc-1", "Custom", 42, Some(20), Some(99))
            .unwrap();
        assert_eq!((sid, cid), (20, 99));

        // Duplicate name is an error.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        assert!(
            editor
                .add_mood("uuid:lc-1", "Green", 1, None, None)
                .is_err()
        );

        // Duplicate explicit SID is an error.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        assert!(editor.add_mood("uuid:lc-1", "X", 1, Some(2), None).is_err());
    }

    #[test]
    fn test_set_param_attribute_fallback() {
        const SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="VirtualIn" U="vi-1" Title="Druck" Analog="true" MaxVal="1000">
    <Co K="Q" U="vi-1-q"/>
  </C>
</ControlList>"#;
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        editor.set_param("uuid:vi-1", "MaxVal", "1100").unwrap();
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains(r#"MaxVal="1100""#));
        // A non-connector, non-attribute param still errors.
        let mut editor = ConfigEditor::load(SAMPLE).unwrap();
        assert!(editor.set_param("uuid:vi-1", "Bogus", "1").is_err());
    }

    #[test]
    fn test_list_wires() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // WeatherData has 1 connector (AQ)
        let wires = editor.list_wires("uuid:wd-1").unwrap();
        assert_eq!(wires.len(), 1);
        assert_eq!(wires[0].connector, "AQ");
    }

    #[test]
    fn test_wire_and_unwire() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();

        // Wire WeatherData.AQ → SysVar (but SysVar has no Co in sample, so wire to sensor)
        // Wire sensor.Text → WeatherData.AQ
        let msg = editor
            .wire("uuid:sensor-1", "Text", "uuid:wd-1", "AQ")
            .unwrap();
        assert!(msg.contains("Wired"));

        // Verify it's connected
        let wires = editor.list_wires("uuid:sensor-1").unwrap();
        let text_co = wires.iter().find(|w| w.connector == "Text").unwrap();
        assert!(text_co.connected);

        // Unwire
        let msg = editor.unwire("uuid:sensor-1", "Text").unwrap();
        assert!(msg.contains("Unwired"));

        // Verify disconnected
        let wires = editor.list_wires("uuid:sensor-1").unwrap();
        let text_co = wires.iter().find(|w| w.connector == "Text").unwrap();
        assert!(!text_co.connected);
    }

    // ── validate_config tests ─────────────────────────────────────────────

    #[test]
    fn test_validate_config_all_ok() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let results = editor.validate_config();
        // Room and category refs are valid in SAMPLE_XML
        assert!(
            results
                .iter()
                .any(|r| r.contains("✓") && r.contains("room"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.contains("✓") && r.contains("category"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.contains("✓") && r.contains("MQTT broker"))
        );
    }

    #[test]
    fn test_validate_config_bad_room_ref() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="Category" U="cat-1" Title="Wetter"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="cat-1" Pr="nonexistent-room"/>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.validate_config();
        assert!(
            results
                .iter()
                .any(|r| r.contains("✗") && r.contains("nonexistent-room"))
        );
    }

    #[test]
    fn test_validate_config_bad_category_ref() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" U="room-1" Title="Kitchen"/>
  <C Type="WeatherData" U="wd-1" Title="Temperatur">
    <IoData Cr="nonexistent-cat" Pr="room-1"/>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.validate_config();
        assert!(
            results
                .iter()
                .any(|r| r.contains("✗") && r.contains("nonexistent-cat"))
        );
    }

    #[test]
    fn test_validate_config_mqtt_no_broker() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Plugin" gid="Mqtt" U="mqtt-1" Title="MQTT">
    <SET>
      <mqtt_broker_address t="11" v=""/>
    </SET>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.validate_config();
        assert!(
            results
                .iter()
                .any(|r| r.contains("✗") && r.contains("broker address is not set"))
        );
    }

    #[test]
    fn test_validate_config_empty() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let results = editor.validate_config();
        // Should report all OK (no refs to check)
        assert!(results.iter().all(|r| !r.contains("✗")));
    }

    fn validate_xml(body: &str) -> Vec<String> {
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList Version=\"267\">\n{body}\n</ControlList>"
        );
        ConfigEditor::load(xml.as_bytes())
            .unwrap()
            .validate_config()
    }

    fn warnings_containing<'a>(results: &'a [String], needle: &str) -> Vec<&'a String> {
        results
            .iter()
            .filter(|r| r.starts_with('⚠') && r.contains(needle))
            .collect()
    }

    const LC2_MODES: &str = r#"
  <C Type="Mode" U="m-7" Title="Evening" Mode="-7"/>
  <C Type="Mode" U="m-8" Title="Nighttime" Mode="-8"/>
  <C Type="DigitalIn" U="pir-1" Title="PIR">
    <Co K="Q" U="pir-q"/>
  </C>"#;

    fn lc2(id: &str, title: &str, wire: &str, op_m: &str) -> String {
        format!(
            r#"
  <C Type="LightController2" U="{id}" Title="{title}">
    <Co K="Move" U="{id}-move">{wire}</Co>
    <LSConfig OpM="{op_m}" ScMv="3" ScAc="0"/>
  </C>"#
        )
    }

    #[test]
    fn test_validate_lc2_motion_missing_mode_its_peers_cover() {
        // Two rooms have a Nighttime row, the third lacks it and has no 778 row.
        let pir = r#"<In Input="pir-q"/>"#;
        let results = validate_xml(&format!(
            "{LC2_MODES}{}{}{}",
            lc2("lc-1", "Hall Light", pir, "-7,-8"),
            lc2("lc-2", "Landing Light", pir, "-7,-8"),
            lc2("lc-3", "Bedroom Spots", pir, "-7"),
        ));
        let w = warnings_containing(&results, "All other times");
        assert_eq!(w.len(), 1, "{results:?}");
        assert!(w[0].contains("'Bedroom Spots'"));
        assert!(w[0].contains("no motion row for Nighttime (-8)"));
    }

    #[test]
    fn test_validate_lc2_motion_empty_table() {
        let results = validate_xml(&format!(
            "{LC2_MODES}{}",
            lc2("lc-1", "Hall Light", r#"<In Input="pir-q"/>"#, "")
        ));
        assert_eq!(
            warnings_containing(&results, "motion table is empty").len(),
            1,
            "{results:?}"
        );
    }

    #[test]
    fn test_validate_lc2_motion_missing_table() {
        let results = validate_xml(
            r#"<C Type="LightController2" U="lc-1" Title="Hall Light">
                <Co K="Presence" U="lc-presence"><In Input="pir-q"/></Co>
            </C>"#,
        );
        assert_eq!(
            warnings_containing(&results, "motion table is empty").len(),
            1,
            "{results:?}"
        );
    }

    #[test]
    fn test_validate_lc2_motion_rule_quiet_when_ok() {
        let pir = r#"<In Input="pir-q"/>"#;
        // Every controller lists the same modes: they may cover the whole day.
        let same_modes = validate_xml(&format!(
            "{LC2_MODES}{}{}",
            lc2("lc-1", "Hall Light", pir, "-7,-8"),
            lc2("lc-2", "Landing Light", pir, "-7,-8"),
        ));
        assert!(warnings_containing(&same_modes, "All other times").is_empty());
        // Fewer modes but a 778 row → no warning.
        let with_row = validate_xml(&format!(
            "{LC2_MODES}{}{}",
            lc2("lc-1", "Hall Light", pir, "-7,-8"),
            lc2("lc-2", "Landing Light", pir, "-7,778"),
        ));
        assert!(warnings_containing(&with_row, "All other times").is_empty());
        // Missing mode but no motion/presence wiring → not checked.
        let unwired = validate_xml(&format!(
            "{LC2_MODES}{}{}{}",
            lc2("lc-1", "Hall Light", pir, "-7,-8"),
            lc2("lc-2", "Landing Light", pir, "-7,-8"),
            lc2("lc-3", "Bedroom Spots", "", "-7"),
        ));
        assert!(warnings_containing(&unwired, "All other times").is_empty());
        assert!(
            unwired
                .iter()
                .any(|r| r.starts_with('✓') && r.contains("motion rows"))
        );
    }

    const PULSE_LC2: &str = r#"
  <C Type="LightController2" U="lc-1" Title="Lounge Light">
    <Co K="Sel1" U="lc-sel1"><In Input="ed-rise"/></Co>
    <Co K="Sel2" U="lc-sel2"><In Input="mf-q"/></Co>
    <Co K="Reset" U="lc-reset"><In Input="ed2-rise"/></Co>
    <LSConfig OpM="778" ScMv="1" ScAc="0"/>
  </C>"#;

    #[test]
    fn test_validate_long_pulse_into_mood_input() {
        let results = validate_xml(&format!(
            r#"{PULSE_LC2}
  <C Type="EdgeDetection" U="ed-1" Title="Scene Edge">
    <Co K="RisingEdge" U="ed-rise"/>
    <Co K="PulseTime" U="ed-pt" Def="1"/>
  </C>
  <C Type="Monoflop" U="mf-1" Title="Scene Pulse">
    <Co K="Q" U="mf-q"/>
    <Co K="Time" U="mf-t" Def="1.5"/>
  </C>
  <C Type="EdgeDetection" U="ed-2" Title="Reset Edge">
    <Co K="RisingEdge" U="ed2-rise"/>
    <Co K="PulseTime" U="ed2-pt" Def="5"/>
  </C>"#
        ));
        let w = warnings_containing(&results, "long-click");
        assert_eq!(w.len(), 2, "{results:?}");
        assert!(w[0].contains("'Lounge Light'.Sel1") && w[0].contains("'Scene Edge'"));
        assert!(w[1].contains("'Lounge Light'.Sel2") && w[1].contains("'Scene Pulse'"));
    }

    #[test]
    fn test_validate_short_pulse_into_mood_input_is_quiet() {
        let results = validate_xml(&format!(
            r#"{PULSE_LC2}
  <C Type="EdgeDetection" U="ed-1" Title="Scene Edge">
    <Co K="RisingEdge" U="ed-rise"/>
    <Co K="PulseTime" U="ed-pt" Def="0.2"/>
  </C>
  <C Type="Monoflop" U="mf-1" Title="Scene Pulse">
    <Co K="Q" U="mf-q"/>
    <Co K="Time" U="mf-t" Def="0.1"/>
  </C>"#
        ));
        assert!(warnings_containing(&results, "long-click").is_empty());
    }

    #[test]
    fn test_validate_mood_pulses_respect_configured_mixing_duration() {
        for (setting, pulse, warns) in [
            ("", "0.5", false),
            (r#"Def="0""#, "5", false),
            (r#"Def="2""#, "1", false),
            (r#"Def="0.1""#, "0.2", true),
        ] {
            let results = validate_xml(&format!(
                r#"<C Type="LightController2" U="lc" Title="Light">
                    <Co K="Sel1" U="sel"><In Input="pulse"/></Co>
                    <Co K="SceneMixTime" U="mix" {setting}/>
                </C>
                <C Type="Monoflop" U="mf" Title="Scene Pulse">
                    <Co K="Q" U="pulse"/>
                    <Co K="Time" U="time" Def="{pulse}"/>
                </C>"#
            ));
            assert_eq!(
                !warnings_containing(&results, "long-click").is_empty(),
                warns,
                "{results:?}"
            );
        }
    }

    #[test]
    fn test_validate_mood_pulses_with_dynamic_durations_are_quiet() {
        for (mix_wire, pulse_wire) in [
            (r#"<In Input="dynamic"/>"#, ""),
            ("", r#"<In Input="dynamic"/>"#),
        ] {
            let results = validate_xml(&format!(
                r#"<C Type="LightController2" U="lc" Title="Light">
                    <Co K="Sel1" U="sel"><In Input="pulse"/></Co>
                    <Co K="SceneMixTime" U="mix" Def="1">{mix_wire}</Co>
                </C>
                <C Type="Monoflop" U="mf" Title="Scene Pulse">
                    <Co K="Q" U="pulse"/>
                    <Co K="Time" U="time" Def="2">{pulse_wire}</Co>
                </C>"#
            ));
            assert!(
                warnings_containing(&results, "long-click").is_empty(),
                "{results:?}"
            );
        }
    }

    #[test]
    fn test_validate_default_pulse_into_mood_input() {
        // No Def: Loxone defaults are 1 s (EdgeDetection) and 2 s (Monoflop).
        let results = validate_xml(&format!(
            r#"{PULSE_LC2}
  <C Type="EdgeDetection" U="ed-1" Title="Scene Edge">
    <Co K="RisingEdge" U="ed-rise"/>
    <Co K="PulseTime" U="ed-pt"/>
  </C>
  <C Type="Monoflop" U="mf-1" Title="Scene Pulse">
    <Co K="Q" U="mf-q"/>
    <Co K="Time" U="mf-t"/>
  </C>"#
        ));
        let w = warnings_containing(&results, "long-click");
        assert_eq!(w.len(), 2, "{results:?}");
        assert!(w.iter().any(|m| m.contains("fed a 1 s pulse")));
        assert!(w.iter().any(|m| m.contains("fed a 2 s pulse")));
    }

    const REF_TARGETS: &str = r#"
  <C Type="VirtualIn" U="vi-1" Title="Unlock Webhook" IName="VI1"/>
  <C Type="Memory" U="mem-1" Title="Night Flag"/>
  <C Type="InputRef" U="ref-vi" Title="Unlock Webhook" Ref="vi-1">
    <Co K="AQ" U="ref-vi-aq"/>
    <Co K="Q" U="ref-vi-q"/>
  </C>
  <C Type="InputRef" U="ref-mem" Title="Night Flag" Ref="mem-1">
    <Co K="AQ" U="ref-mem-aq"/>
    <Co K="Q" U="ref-mem-q"/>
  </C>"#;

    #[test]
    fn test_validate_reference_wired_from_q() {
        let results = validate_xml(&format!(
            r#"{REF_TARGETS}
  <C Type="Door" U="door-1" Title="Front Door">
    <Co K="Open" U="door-open"><In Input="ref-vi-q"/></Co>
    <Co K="Close" U="door-close"><In Input="ref-mem-q"/></Co>
    <Co K="Lock" U="door-lock"><In Input="ref-vi-aq"/></Co>
  </C>"#
        ));
        let w = warnings_containing(&results, "wired from Q");
        assert_eq!(w.len(), 1, "{results:?}");
        assert!(w[0].contains("Door 'Front Door'.Open"));
        assert!(w[0].contains("VirtualIn 'Unlock Webhook'"));
    }

    #[test]
    fn test_validate_reference_q_into_notification_is_quiet() {
        let results = validate_xml(&format!(
            r#"{REF_TARGETS}
  <C Type="OutputRefLM" U="lm-1" Title="Notify" Ref="caller-1">
    <Co K="AI" U="lm-ai"><In Input="ref-vi-q"/></Co>
  </C>"#
        ));
        assert!(warnings_containing(&results, "wired from Q").is_empty());
        assert!(
            results
                .iter()
                .any(|r| r.starts_with('✓') && r.contains("instead of AQ"))
        );
    }

    #[test]
    fn test_validate_reference_q_managed_links_are_quiet() {
        let results = validate_xml(&format!(
            r#"{REF_TARGETS}
              <C Type="PresenceDetector" U="presence" Title="Presence">
                <Co K="DeviceActivate" U="activate"><In Input="ref-vi-q" FLG="1"/></Co>
              </C>"#
        ));
        assert!(
            warnings_containing(&results, "wired from Q").is_empty(),
            "{results:?}"
        );
    }

    // ── User CRUD tests ──────────────────────────────────────────────────

    #[test]
    fn test_add_user() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor.add_user("TestUser").unwrap();
        assert!(!uuid.is_empty());
        let matches = editor.find_elements("TestUser");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_add_user_duplicate_fails() {
        // Add a user first, then try to add the same one
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="User" V="175" U="user-1" Title="admin"/>
</ControlList>"#;
        let mut editor = ConfigEditor::load(xml).unwrap();
        let result = editor.add_user("admin");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_add_user_case_insensitive_duplicate() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="User" V="175" U="user-1" Title="Admin"/>
</ControlList>"#;
        let mut editor = ConfigEditor::load(xml).unwrap();
        let result = editor.add_user("admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_user() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="User" V="175" U="user-1" Title="admin"/>
  <C Type="User" V="175" U="user-2" Title="chris"/>
</ControlList>"#;
        let mut editor = ConfigEditor::load(xml).unwrap();
        let uuid = editor.remove_user("chris").unwrap();
        assert_eq!(uuid, "user-2");
        // Verify chris is gone
        let matches = editor.find_elements("chris");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_remove_user_nonexistent() {
        let editor_result = ConfigEditor::load(SAMPLE_XML);
        let mut editor = editor_result.unwrap();
        let result = editor.remove_user("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_remove_user_case_insensitive() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="User" V="175" U="user-1" Title="Admin"/>
</ControlList>"#;
        let mut editor = ConfigEditor::load(xml).unwrap();
        let uuid = editor.remove_user("admin").unwrap();
        assert_eq!(uuid, "user-1");
    }

    // ── find_category_uuid tests ──────────────────────────────────────────

    #[test]
    fn test_find_category_uuid() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor.find_category_uuid("Wetter").unwrap();
        assert_eq!(uuid, "cat-1");
    }

    #[test]
    fn test_find_category_uuid_not_found() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.find_category_uuid("Nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_find_category_uuid_partial_match() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // "Wett" is a substring of "Wetter"
        let uuid = editor.find_category_uuid("Wett").unwrap();
        assert_eq!(uuid, "cat-1");
    }

    // ── find_room_uuid tests ─────────────────────────────────────────────

    #[test]
    fn test_find_room_uuid() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor.find_room_uuid("Kitchen").unwrap();
        assert_eq!(uuid, "room-1");
    }

    #[test]
    fn test_find_room_uuid_not_found() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.find_room_uuid("Nonexistent");
        assert!(result.is_err());
    }

    // ── describe with IoData fields ──────────────────────────────────────

    #[test]
    fn test_describe_with_iodata() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let desc = editor.describe("uuid:wd-1").unwrap();
        assert_eq!(desc.element_type, "WeatherData");
        assert_eq!(desc.title, "Temperatur");
        assert_eq!(desc.uuid, "wd-1");
        assert_eq!(desc.room_uuid, "room-1");
        assert_eq!(desc.category_uuid, "cat-1");
        assert_eq!(desc.connectors.len(), 1);
        assert_eq!(desc.connectors[0].kind, "AQ");
    }

    #[test]
    fn test_describe_sysvar_connectors() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let desc = editor.describe("uuid:sv-1").unwrap();
        assert_eq!(desc.element_type, "SysVar");
        assert_eq!(desc.connectors.len(), 3);
        let kinds: Vec<&str> = desc.connectors.iter().map(|c| c.kind.as_str()).collect();
        assert!(kinds.contains(&"AQ"));
        assert!(kinds.contains(&"AI"));
        assert!(kinds.contains(&"Q"));
    }

    #[test]
    fn test_describe_nonexistent_fails() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.describe("uuid:nonexistent");
        assert!(result.is_err());
    }

    // ── add_element_to_root tests ────────────────────────────────────────

    #[test]
    fn test_add_element_to_root() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor
            .add_element_to_root("Switch", "TestLight", Some("room-1"), Some("cat-1"), &[])
            .unwrap();
        assert!(!uuid.is_empty());

        let desc = editor.describe(&format!("uuid:{}", uuid)).unwrap();
        assert_eq!(desc.element_type, "Switch");
        assert_eq!(desc.title, "TestLight");
        assert_eq!(desc.room_uuid, "room-1");
        assert_eq!(desc.category_uuid, "cat-1");
    }

    #[test]
    fn test_add_element_to_root_with_properties() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor
            .add_element_to_root(
                "Switch",
                "MySwitch",
                None,
                None,
                &[("my_prop", "my_value", "11")],
            )
            .unwrap();

        let desc = editor.describe(&format!("uuid:{}", uuid)).unwrap();
        assert_eq!(desc.properties["my_prop"].value, "my_value");
        assert_eq!(desc.properties["my_prop"].type_code, "11");
    }

    #[test]
    fn test_add_element_to_root_no_iodata() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid = editor
            .add_element_to_root("Switch", "NoRoom", None, None, &[])
            .unwrap();

        let desc = editor.describe(&format!("uuid:{}", uuid)).unwrap();
        assert_eq!(desc.room_uuid, "");
        assert_eq!(desc.category_uuid, "");
    }

    // ── Selector edge cases ──────────────────────────────────────────────

    #[test]
    fn test_find_elements_no_match() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("CompletelyNonexistent");
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_find_elements_case_insensitive_title() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let matches = editor.find_elements("kitchen");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_elements_partial_title() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // "Temp" matches "Temp Sub", "Temperatur", and "Aussentemp"
        let matches = editor.find_elements("Temp");
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_require_one_ambiguous() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        // "Type:WeatherData" matches 2 elements
        let result = editor.require_one("Type:WeatherData");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("matches 2"));
    }

    #[test]
    fn test_require_one_no_match() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.require_one("uuid:nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // ── CRLF line endings ────────────────────────────────────────────────

    #[test]
    fn test_picoc_multiline_code_survives_mutation_and_repeated_roundtrips() {
        let code = "// public synthetic header\r\nint x = 1;\r\nif (x < 2) { x = x + 1; }\r\n";
        for block_type in ["Code1", "Code4", "Code8", "Code16"] {
            for encoded in [false, true] {
                for file_newline in ["\n", "\r\n"] {
                    let attribute = code.replace('<', "&lt;");
                    let attribute = if encoded {
                        attribute.replace('\r', "&#xD;").replace('\n', "&#xA;")
                    } else {
                        attribute
                    };
                    let xml = format!(
                        "<?xml version=\"1.0\" encoding=\"utf-8\"?>{file_newline}<ControlList>{file_newline}<C Type=\"{block_type}\" U=\"code-1\" Title=\"Synthetic\" Code=\"{attribute}\"/>{file_newline}</ControlList>{file_newline}"
                    );
                    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
                    editor
                        .set_attribute("uuid:code-1", "Title", "Edited")
                        .unwrap();
                    let mut previous_bytes = None;
                    for _ in 0..3 {
                        let path = editor.require_one("uuid:code-1").unwrap();
                        assert_eq!(editor.get_element(&path).attributes["Code"], code);
                        assert!(
                            warnings_containing(&editor.validate_config(), "PicoC Code").is_empty()
                        );
                        let bytes = editor.to_bytes().unwrap();
                        assert!(
                            bytes.starts_with(
                                format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>{file_newline}")
                                    .as_bytes()
                            )
                        );
                        assert!(
                            String::from_utf8_lossy(&bytes).contains(
                                "// public synthetic header\r\nint x = 1;\r\nif (x &lt; 2)"
                            )
                        );
                        if let Some(previous) = previous_bytes {
                            assert_eq!(bytes, previous, "{block_type}, encoded={encoded}");
                        }
                        editor = ConfigEditor::load(&bytes).unwrap();
                        previous_bytes = Some(bytes);
                    }
                }
            }
        }
    }

    #[test]
    fn test_picoc_collapsed_comment_warns_in_validate_and_check() {
        for block_type in ["Code1", "Code4", "Code8", "Code16"] {
            let xml = format!(
                r#"<ControlList><C Type="{block_type}" U="00000000-0000-0000-ffff000000000001" Title="Synthetic" Code="  // header int x = 1; while (1) {{ x = x + 1; }}"/></ControlList>"#
            );
            let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            for results in [editor.validate_config(), editor.check_blocks(None)] {
                let warnings = warnings_containing(&results, "PicoC Code");
                assert_eq!(warnings.len(), 1, "{block_type}: {results:?}");
                assert!(warnings[0].contains("restore its line breaks"));
                assert!(!results.iter().any(|r| r.starts_with('✗')));
            }
            assert!(
                warnings_containing(&editor.check_blocks(Some("Other")), "PicoC Code").is_empty()
            );
        }
    }

    #[test]
    fn test_picoc_ordinary_single_line_and_multiline_code_do_not_warn() {
        for block_type in ["Code1", "Code4", "Code8", "Code16", "Other"] {
            for code in [
                "",
                "int x = 1;",
                "char *url = &quot;http://example.invalid&quot;;",
                "// header&#xD;&#xA;int x = 1;&#xD;&#xA;",
                "// header&#xA;int x = 1;",
                "// header&#xD;int x = 1;",
            ] {
                let xml = format!(
                    r#"<ControlList><C Type="{block_type}" U="code-1" Title="Synthetic" Code="{code}"/></ControlList>"#
                );
                let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
                for results in [editor.validate_config(), editor.check_blocks(None)] {
                    assert!(
                        warnings_containing(&results, "PicoC Code").is_empty(),
                        "{block_type}: {code}: {results:?}"
                    );
                }
            }
        }
        let results = validate_xml(r#"<C Type="Other" Code="// not PicoC"/>"#);
        assert!(warnings_containing(&results, "PicoC Code").is_empty());
    }

    #[test]
    fn test_crlf_preservation() {
        let xml_str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList Version=\"267\">\r\n  <C Type=\"Place\" U=\"r-1\" Title=\"Room1\"/>\r\n</ControlList>";
        let editor = ConfigEditor::load(xml_str.as_bytes()).unwrap();
        assert!(editor.had_crlf);
        let output = editor.to_bytes().unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\r\n"));
    }

    #[test]
    fn test_crlf_inside_attribute_does_not_make_lf_file_crlf() {
        // An LF file whose only CRLF sits inside an attribute value (AutoPilot rule JSON).
        let xml_str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList Version=\"1\">\n\t<C U=\"a\" APC=\"{&quot;x&quot;:1,\r\n&quot;y&quot;:2}\"/>\n</ControlList>\n";
        let editor = ConfigEditor::load(xml_str.as_bytes()).unwrap();
        assert!(!editor.had_crlf);
        let output_str = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(output_str.starts_with(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList Version=\"1\">\n\t<C "
        ));
        assert!(output_str.ends_with("/>\n</ControlList>\n"));
        assert!(output_str.contains("APC=\"{&quot;x&quot;:1,\r\n&quot;y&quot;:2}\""));
        assert_eq!(
            ConfigEditor::load(output_str.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap(),
            output_str.as_bytes(),
            "a second save preserves both newline styles"
        );
    }

    #[test]
    fn test_crlf_detection_without_a_declaration_skips_attribute_newlines() {
        for (xml, expected_crlf) in [
            (
                "<ControlList APC='one\r\ntwo'>\n<C/>\n</ControlList>\n",
                false,
            ),
            (
                "<ControlList APC='one\ntwo'>\r\n<C/>\r\n</ControlList>\r\n",
                true,
            ),
            ("<ControlList APC='one\r\ntwo'/>", false),
            ("<!-- one\r\ntwo -->\n<ControlList/>", false),
        ] {
            assert_eq!(
                ConfigEditor::load(xml.as_bytes()).unwrap().had_crlf,
                expected_crlf,
                "{xml:?}"
            );
        }
    }

    // ── Wire edge cases ──────────────────────────────────────────────────

    #[test]
    fn test_wire_nonexistent_connector() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.wire("uuid:sensor-1", "NonexistentCo", "uuid:wd-1", "AQ");
        assert!(result.is_err());
    }

    #[test]
    fn test_unwire_nonexistent_connector() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.unwire("uuid:wd-1", "NonexistentCo");
        assert!(result.is_err());
    }

    // ── VirtualIn + InputRef placement ──────────────────────────────────

    const SAMPLE_WITH_PAGE: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Document" U="doc-1" V="175">
    <C Type="LoxLIVE" V="175" U="live-1" Title="MS">
      <C Type="VirtualInCaption" V="175" U="vic-1" Title="Virtual Inputs" WF="16384"/>
    </C>
    <C Type="Program" V="175" U="prog-1" Title="MS" WF="16384">
      <C Type="Page" V="175" U="page-1" Title="Seite" WF="16384">
        <IoData/>
      </C>
    </C>
  </C>
</ControlList>"#;

    #[test]
    fn test_add_virtual_in_places_inputref_on_page() {
        let mut editor = ConfigEditor::load(SAMPLE_WITH_PAGE).unwrap();
        let (vi_uuid, aq_uuid) = editor
            .add_virtual_in("TestVI", &VirtualInOpts::default(), "Type:VirtualInCaption")
            .unwrap();
        assert!(!aq_uuid.is_empty());
        assert_ne!(vi_uuid, aq_uuid);

        // InputRef should NOT be inside VirtualIn (breaks SPS I/O registration)
        let vi_refs = editor.find_elements("Type:VirtualIn");
        assert_eq!(vi_refs.len(), 1, "should have exactly 1 VirtualIn");
        let vi = editor.get_element(&vi_refs[0]);
        let vi_has_inputref = vi.children.iter().any(|c| {
            c.as_element()
                .map(|e| {
                    e.attributes
                        .get("Type")
                        .map(|t| t == "InputRef")
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        });
        assert!(
            !vi_has_inputref,
            "VirtualIn must NOT contain an InputRef child (breaks I/O registration)"
        );

        // InputRef should exist on the Page (for SPS circuit resolution)
        let page_paths = editor.find_elements("Type:Page");
        assert_eq!(page_paths.len(), 1);
        let page = editor.get_element(&page_paths[0]);
        let page_inputrefs: Vec<_> = page
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|e| {
                e.attributes
                    .get("Type")
                    .map(|t| t == "InputRef")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(
            page_inputrefs.len(),
            1,
            "Page should have exactly 1 InputRef"
        );

        // The AQ connector UUID on the page InputRef should match what was returned
        let page_inputref = page_inputrefs[0];
        let page_aq = page_inputref
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .find(|e| e.name == "Co" && e.attributes.get("K").map(|k| k == "AQ").unwrap_or(false))
            .unwrap();
        assert_eq!(
            page_aq.attributes.get("U").unwrap(),
            &aq_uuid,
            "Page InputRef AQ UUID should match returned uuid"
        );
    }

    #[test]
    fn test_list_device_ports_empty() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let output = editor.list_device_ports();
        // SAMPLE_XML has no devices, so should show "No hardware devices"
        assert!(
            output.contains("No hardware") || output.contains("Summary"),
            "Should handle configs with no/some devices"
        );
    }

    #[test]
    fn test_apply_template_unknown() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.apply_template("nonexistent", "Room1");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_apply_template_room_not_found() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let result = editor.apply_template("standard", "NonexistentRoom");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_category() {
        let mut editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let uuid1 = editor.add_category("Beleuchtung").unwrap();
        assert!(!uuid1.is_empty());
        // Adding same category again should return same UUID
        let uuid2 = editor.add_category("Beleuchtung").unwrap();
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_set_param() {
        let xml_str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-1" Title="Page1" WF="16384">
    <C Type="And" V="175" U="a-1" Title="TestAnd" WF="16384">
      <Co K="I1" U="c-1"/>
      <Co K="I2" U="c-2"/>
      <Co K="Q" U="c-3"/>
    </C>
  </C>
</ControlList>"#;
        let mut editor = ConfigEditor::load(xml_str.as_bytes()).unwrap();
        editor.set_param("TestAnd", "I1", "42").unwrap();
        let output = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(output.contains(r#"Def="42""#));
    }

    #[test]
    fn test_config_stats_basic() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let stats = editor.config_stats();
        assert_eq!(stats.room_count, 2);
        assert_eq!(stats.category_count, 1);
        assert!(stats.total_items > 0);
        assert!(!stats.block_types.is_empty());
        // Verify format_text doesn't panic and contains key sections
        let text = stats.format_text();
        assert!(text.contains("=== Config Statistics ==="));
        assert!(text.contains("Overview:"));
        assert!(text.contains("Wiring:"));
    }

    #[test]
    fn test_config_stats_with_wiring() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-1" Title="Kitchen" WF="16384"/>
  <C Type="LightController2" U="lc2-1" Title="Kitchen Light">
    <Co K="AI" U="co-ai">
      <In Input="src-1"/>
      <In Input="src-2" FLG="2"/>
    </Co>
    <IoData Pr="room-1"/>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let stats = editor.config_stats();
        assert_eq!(stats.wiring_total, 2);
        assert_eq!(stats.wiring_cross_page, 1);
        assert_eq!(stats.wiring_multi_input, 1);
        assert_eq!(stats.room_completeness.len(), 1);
        assert!(stats.room_completeness[0].has_lighting);
        assert!(!stats.room_completeness[0].has_blinds);
    }

    #[test]
    fn test_config_stats_json_serializable() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        let stats = editor.config_stats();
        let json = serde_json::to_string_pretty(&stats).unwrap();
        assert!(json.contains("\"room_count\""));
        assert!(json.contains("\"block_types\""));
        assert!(json.contains("\"wiring_total\""));
    }

    /// Extract attribute names in order from an XML element string.
    fn attr_names_in_order(line: &str) -> Vec<String> {
        let mut names = Vec::new();
        let mut rest = line;
        // Walk forward looking for =" patterns that indicate attribute assignments
        while let Some(eq_pos) = rest.find("=\"") {
            // Walk backwards from '=' to find the attribute name start
            let before = &rest[..eq_pos];
            let name_start = before
                .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .map(|p| p + 1)
                .unwrap_or(0);
            let name = &before[name_start..];
            if !name.is_empty() {
                names.push(name.to_string());
            }
            // Skip past the closing quote
            let after_eq = &rest[eq_pos + 2..];
            match after_eq.find('"') {
                Some(end) => rest = &after_eq[end + 1..],
                None => break,
            }
        }
        names
    }

    #[test]
    fn test_attribute_order_preserved_on_roundtrip() {
        // The Loxone Miniserver's SPS engine rejects configs with shuffled XML
        // attributes and falls back to a minimal "Emergency" program — all
        // function blocks (LightController2, AlarmClock, etc.) disappear from
        // the API and only raw device actors remain.
        //
        // This happened because `xmltree` defaults to `HashMap` for attributes,
        // which randomizes order on parse→serialize. The fix is the
        // `attribute-order` feature flag (uses `IndexMap` instead).
        //
        // This test verifies that EVERY element's attribute order is preserved
        // through a load→to_bytes round-trip. It uses a representative Loxone
        // config snippet with the non-alphabetical attribute patterns that
        // Loxone Config desktop produces.
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267" LxAV="84" NextObj="190" NextConst="1">
	<C Type="Document" V="17000331" U="20123f74-0221" Title="Laufenstrasse" ConfigVersion="17000331" SPSfreq="100" OpenIsHigh="false">
		<C Type="LoxCaption" V="175" U="20123f74-0222" Title="Symbole" Cl="0,0,0" WF="16384" CaptionType="5" SubType="5"/>
		<C Type="Category" V="175" U="cat-1" Title="Beleuchtung" WF="16384" Icon="00000153" Rating="1" UseFav="true" CatGroup="2" PType="2" BkColor="255,230,71"/>
		<C Type="Place" V="175" U="room-1" Title="DG Schlafzimmer" WF="16384" Icon="00000048" PType="2"/>
		<C Type="Page" V="175" U="page-1" Title="DG Schlafzimmer" WF="16384">
			<IoData Cr="cat-1" Pr="room-1"/>
			<C V="175" Cl="255,230,71" WF="16384" NameAI0="Touch Nightlight" CapAI0="31" Px2="10080" Title="Lichtsteuerung" Py2="2424" Px="7392" COName="1" Nio="71" Type="LightController2" U="lc-1" Py="576">
				<Co K="I1" U="co-i1"/>
				<Co K="BrightnessLimit" Def="30" U="co-bl"/>
				<Co Nc="1" K="AlarmClock" U="co-ac">
					<In FLG="2" Input="alarm-qtp"/>
				</Co>
				<Co K="AQ1" U="co-aq1"/>
				<LightscenesC FC="0040FFFF,008080FF" Num="2" Outputs="18">
					<LightsceneC Q1="1613645813" SID="1" UUID="ls-1" Name="Entspannen" CID="8" Q2="0" Outputs="18"/>
				</LightscenesC>
				<LSConfig OpM="778" ScAc="2" ScMv="777"/>
				<IoData Visu="true" Rating="1" Cr="cat-1" Pr="room-1"/>
				<PSD Dmin="30" V="20" Dmax="180" Amax="4" Ae="1380" As="-2" Amin="1"/>
			</C>
			<C V="175" N="2" Py2="3864" Cl="141,255,112" Title="Touch Nightlight Air Cris" Px="7392" Dev="dev-1" Type="AlarmClock" WF="16384" Nio="26" Py="2592" U="ac-1" Px2="10080">
				<Co K="VIN1" U="co-vin1" Nc="1">
					<In Input="vi-aussentemp"/>
				</Co>
				<Co K="BrightInact" Nc="1" U="co-bi">
					<In FLG="2" Input="mult-aq"/>
				</Co>
				<Co K="BrightAct" Def="50" U="co-ba"/>
				<IoData Pr="room-1" Visu="true" Cr="cat-1"/>
				<AlarmClock Time="21600" Int="true" Modes="1023" Name="Standardalarm" U="alarm-1"/>
				<AlarmClock Modes="248" Id="1" Name="Wecker" U="alarm-2" Time="25200"/>
			</C>
			<C Nio="3" Type="GreaterEqual" Py2="1272" U="ge-1" V="175" Title="Nach 05:00" Px2="8736" WF="147456" Px="7392" Cl="0,0,0" Py="576">
				<Co U="co-ge-i1" Nc="1" K="Input1">
					<In Input="time-output"/>
				</Co>
				<Co Def="300" U="co-ge-i2" K="Input2"/>
				<Co U="co-ge-q" K="Q"/>
			</C>
		</C>
	</C>
</ControlList>"#;

        let editor = ConfigEditor::load(xml).unwrap();
        let output = editor.to_bytes().unwrap();

        let input_str = String::from_utf8_lossy(xml);
        let output_str = String::from_utf8(output).unwrap();

        // Collect attribute orders for every element in input and output.
        // Elements are matched by line position (the structure is identical).
        let input_attrs: Vec<(String, Vec<String>)> = input_str
            .lines()
            .filter(|l| l.contains('<') && !l.trim_start().starts_with("<?"))
            .map(|l| {
                let tag = l
                    .trim()
                    .split('<')
                    .nth(1)
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("?");
                (tag.trim_matches('/').to_string(), attr_names_in_order(l))
            })
            .filter(|(_, attrs)| attrs.len() >= 2) // only check elements with 2+ attrs
            .collect();

        let output_attrs: Vec<(String, Vec<String>)> = output_str
            .lines()
            .filter(|l| l.contains('<') && !l.trim_start().starts_with("<?"))
            .map(|l| {
                let tag = l
                    .trim()
                    .split('<')
                    .nth(1)
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("?");
                (tag.trim_matches('/').to_string(), attr_names_in_order(l))
            })
            .filter(|(_, attrs)| attrs.len() >= 2)
            .collect();

        assert_eq!(
            input_attrs.len(),
            output_attrs.len(),
            "Different number of elements with attributes: input={}, output={}",
            input_attrs.len(),
            output_attrs.len(),
        );

        // Every element must have exactly the same attribute order
        for (i, ((in_tag, in_attrs), (out_tag, out_attrs))) in
            input_attrs.iter().zip(output_attrs.iter()).enumerate()
        {
            assert_eq!(
                in_tag, out_tag,
                "Element {i} tag mismatch: input=<{in_tag}>, output=<{out_tag}>"
            );
            assert_eq!(
                in_attrs, out_attrs,
                "Attribute order not preserved on <{in_tag}> (element {i}):\n  \
                 input:  {in_attrs:?}\n  output: {out_attrs:?}\n\n\
                 The Loxone Miniserver rejects configs with shuffled attributes.\n\
                 Ensure xmltree has the `attribute-order` feature enabled in Cargo.toml."
            );
        }

        // Sanity: we actually checked a meaningful number of elements
        assert!(
            input_attrs.len() >= 15,
            "Test fixture too small — only {} elements checked, expected ≥15",
            input_attrs.len()
        );
    }

    #[test]
    fn test_digit_prefixed_attr_roundtrip() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<C Type="IRoomControllerV2" 12hTF="true" Title="HVAC" U="u1"/>"#;
        let editor = ConfigEditor::load(xml).unwrap();

        // The sanitized attr should be accessible in the DOM as _12hTF
        assert_eq!(editor.root.attributes.get("_12hTF").unwrap(), "true");
        assert_eq!(
            editor.digit_attr_renames,
            vec![("_12hTF".to_string(), "12hTF".to_string())]
        );

        // Round-trip: to_bytes must restore the original digit-prefixed name
        let output = editor.to_bytes().unwrap();
        let output_str = std::str::from_utf8(&output).unwrap();
        assert!(
            output_str.contains(" 12hTF=\"true\""),
            "Original attribute name not restored: {output_str}"
        );
        assert!(
            !output_str.contains("_12hTF"),
            "Sanitized name leaked into output: {output_str}"
        );
    }

    #[test]
    fn test_no_digit_attrs_is_noop() {
        let editor = ConfigEditor::load(SAMPLE_XML).unwrap();
        assert!(editor.digit_attr_renames.is_empty());
    }

    // ── resolve_wiring: InputRef twins ──────────────────────────────────

    /// A page with InputRefs (`(object uuid, AQ uuid, Q uuid)`, all
    /// mirroring `vi-1`) and one consumer block wired to each ref's AQ.
    fn page_with_refs(refs: &[(&str, &str, &str)]) -> Vec<u8> {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList Version=\"267\">\n\
             <C Type=\"VirtualIn\" U=\"vi-1\" Title=\"Taster\"><Co K=\"AQ\" U=\"vi-1-aq\"/></C>\n\
             <C Type=\"Page\" U=\"page-1\" Title=\"Seite\">\n",
        );
        for (i, (obj, aq, q)) in refs.iter().enumerate() {
            xml.push_str(&format!(
                "<C Type=\"InputRef\" U=\"{obj}\" Ref=\"vi-1\">\
                 <Co K=\"AI\" U=\"{obj}-ai\"><In Input=\"vi-1-aq\"/></Co>\
                 <Co K=\"AQ\" U=\"{aq}\"/><Co K=\"Q\" U=\"{q}\"/></C>\n\
                 <C Type=\"Memory\" U=\"mem-{i}\" Title=\"Halter {i}\">\
                 <Co K=\"Input\" U=\"mem-{i}-in\"><In Input=\"{aq}\"/></Co></C>\n"
            ));
        }
        xml.push_str("</C>\n</ControlList>");
        xml.into_bytes()
    }

    /// Connector UUIDs that occur more than once.
    fn duplicate_connector_uuids(xml: &[u8]) -> Vec<String> {
        let text = std::str::from_utf8(xml).unwrap();
        let mut seen = std::collections::HashMap::<&str, usize>::new();
        for part in text.split("<Co ").skip(1) {
            if let Some(at) = part.find(" U=\"") {
                let rest = &part[at + 4..];
                *seen.entry(&rest[..rest.find('"').unwrap()]).or_default() += 1;
            }
        }
        seen.into_iter()
            .filter(|&(_, n)| n > 1)
            .map(|(u, _)| u.to_string())
            .collect()
    }

    #[test]
    fn test_resolve_wiring_keeps_twin_refs_apart() {
        // Two mirrors of one target, unchanged since the download: nothing
        // is stale. Keyed by target alone, both twins collapsed onto the
        // last one's ports — duplicate connector UUIDs, which Loxone
        // Config re-mints on its next open.
        let refs = [("ref-a", "a-aq", "a-q"), ("ref-b", "b-aq", "b-q")];
        let xml = page_with_refs(&refs);
        let mut editor = ConfigEditor::load(&xml).unwrap();
        assert_eq!(editor.resolve_wiring(&xml).unwrap(), 0);
        let out = editor.to_bytes().unwrap();
        assert!(duplicate_connector_uuids(&out).is_empty());
        let text = std::str::from_utf8(&out).unwrap();
        assert!(text.contains("<In Input=\"a-aq\""));
        assert!(text.contains("<In Input=\"b-aq\""));
    }

    #[test]
    fn test_resolve_wiring_remaps_regenerated_ports_per_ref() {
        // Loxone Config regenerated both twins' ports: each maps to its
        // own InputRef's new UUIDs.
        let ours = page_with_refs(&[("ref-a", "a-aq", "a-q"), ("ref-b", "b-aq", "b-q")]);
        let live = page_with_refs(&[("ref-a", "a-aq2", "a-q2"), ("ref-b", "b-aq2", "b-q2")]);
        let mut editor = ConfigEditor::load(&ours).unwrap();
        assert_eq!(editor.resolve_wiring(&live).unwrap(), 2);
        let out = editor.to_bytes().unwrap();
        assert!(duplicate_connector_uuids(&out).is_empty());
        let text = std::str::from_utf8(&out).unwrap();
        assert!(text.contains("<In Input=\"a-aq2\""));
        assert!(text.contains("<In Input=\"b-aq2\""));
        assert!(text.contains("U=\"b-q2\""));
        assert_eq!(editor.resolve_wiring(&live).unwrap(), 0);
        assert_eq!(editor.to_bytes().unwrap(), out);
    }

    #[test]
    fn test_resolve_wiring_target_fallback_only_when_unique() {
        // The InputRef's own UUID changed too: a single mirror of the
        // target still resolves through `Ref=` ...
        let ours = page_with_refs(&[("ref-old", "a-aq", "a-q")]);
        let live = page_with_refs(&[("ref-new", "n-aq", "n-q")]);
        let mut editor = ConfigEditor::load(&ours).unwrap();
        assert_eq!(editor.resolve_wiring(&live).unwrap(), 1);
        let out = editor.to_bytes().unwrap();
        assert!(
            std::str::from_utf8(&out)
                .unwrap()
                .contains("<In Input=\"n-aq\"")
        );

        // ... but with several mirrors there is no telling which is which:
        // left alone rather than guessed.
        let ours = page_with_refs(&[("ref-x", "x-aq", "x-q"), ("ref-y", "y-aq", "y-q")]);
        let mut editor = ConfigEditor::load(&ours).unwrap();
        assert_eq!(editor.resolve_wiring(&live).unwrap(), 0);
    }

    #[test]
    fn test_resolve_wiring_keeps_ports_when_live_target_is_ambiguous() {
        let ours = page_with_refs(&[("ref-old", "old-aq", "old-q")]);
        let live = page_with_refs(&[("ref-a", "a-aq", "a-q"), ("ref-b", "b-aq", "b-q")]);
        let mut editor = ConfigEditor::load(&ours).unwrap();
        let before = editor.to_bytes().unwrap();
        assert_eq!(editor.resolve_wiring(&live).unwrap(), 0);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }

    #[test]
    fn test_resolve_wiring_does_not_reuse_another_connectors_uuid() {
        let ours = page_with_refs(&[("ref-a", "a-aq", "a-q")]);
        let live = page_with_refs(&[("ref-a", "mem-0-in", "a-q")]);
        let mut editor = ConfigEditor::load(&ours).unwrap();
        let before = editor.to_bytes().unwrap();
        assert_eq!(editor.resolve_wiring(&live).unwrap(), 0);
        assert_eq!(editor.to_bytes().unwrap(), before);
        assert!(duplicate_connector_uuids(&before).is_empty());
    }

    #[test]
    fn test_resolve_wiring_rejects_malformed_live_xml_without_mutation() {
        let ours = page_with_refs(&[("ref-a", "a-aq", "a-q")]);
        let mut editor = ConfigEditor::load(&ours).unwrap();
        let before = editor.to_bytes().unwrap();
        assert!(editor.resolve_wiring(b"<ControlList><C").is_err());
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}
