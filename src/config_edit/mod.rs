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
mod layout;
mod properties;
mod rooms;
mod selector;
mod template;
mod validation;
mod wiring;
mod write;

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
}

impl ConfigEditor {
    /// Load a .Loxone XML file for editing.
    pub fn load(data: &[u8]) -> Result<Self> {
        let had_bom = data.starts_with(UTF8_BOM);
        let had_crlf = data.windows(2).any(|w| w == b"\r\n");
        let xml_data = if had_bom { &data[3..] } else { data };
        let reader = BufReader::new(Cursor::new(xml_data));
        let root = Element::parse(reader).context("Failed to parse Loxone XML")?;
        Ok(ConfigEditor {
            root,
            had_bom,
            had_crlf,
        })
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

    fn iter_elements<'a>(&'a self, elem: &'a Element) -> Vec<&'a Element> {
        let mut result = vec![elem];
        for child in &elem.children {
            if let Some(e) = child.as_element() {
                result.extend(self.iter_elements(e));
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
    fn test_crlf_preservation() {
        let xml_str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList Version=\"267\">\r\n  <C Type=\"Place\" U=\"r-1\" Title=\"Room1\"/>\r\n</ControlList>";
        let editor = ConfigEditor::load(xml_str.as_bytes()).unwrap();
        assert!(editor.had_crlf);
        let output = editor.to_bytes().unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\r\n"));
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
        let aq_uuid = editor
            .add_virtual_in("TestVI", false, "Type:VirtualInCaption")
            .unwrap();
        assert!(!aq_uuid.is_empty());

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
}
