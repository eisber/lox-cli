use super::{
    ConfigEditor, ConfigStats, ConfigWire, ConfigWireEndpoint, DescribeBlockEntry,
    DescribeConnectorEntry, DescribeEntry, DescribeRoomEntry, DetectedDevice,
    DetectedDeviceConnector, DetectedDeviceIdentity, DeviceBusSummary, RoomCompleteness, SceneInfo,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use xmltree::Element;

const EXCLUDED_BLOCK_TYPES: &[&str] = &[
    "AlarmClock",
    "AlarmCenter",
    "Alarm",
    "AlarmChain",
    "AMemory",
    "AMinmax",
    "AcControl",
    "AnalogComparator",
    "AnalogDiffTrigger",
    "AnalogMultiplexer",
    "AnalogMultiplexer2",
    "AnalogScaler",
    "AnalogStepper",
    "AnalogThreshold",
    "AnalogThresholdTrigger",
    "AnalogWatchdog",
    "And",
    "Application",
    "AutoJalousie",
    "AutomaticScene",
    "Average",
    "Avg",
    "BinDecoder",
    "BinEncoder",
    "BrightnessControl",
    "Calculator",
    "CallGen",
    "CentralAlarm",
    "CentralFancoil",
    "CentralGate",
    "CentralLight",
    "CentralMusic",
    "CentralPresence",
    "CentralRoofwindow",
    "CentralShade",
    "ClimateControllerUS",
    "CmdRecognition",
    "Code1",
    "Code4",
    "Code8",
    "Code16",
    "Comparator",
    "Counter",
    "DayTimer",
    "DaylightController",
    "DbConE",
    "DbConS",
    "DbConT",
    "DewPoint",
    "Div",
    "Door",
    "Doorcontroller",
    "EIBJalousie",
    "Edge",
    "EdgeDetection",
    "EdgeWipingRelay",
    "Energy",
    "EnergyManager",
    "EnergyManager2",
    "Equal",
    "Fan",
    "Fancoil",
    "FancoilFreshAir",
    "FlipFlop",
    "Formula",
    "GlobalStates",
    "Greater",
    "GreaterEqual",
    "HVACController",
    "HeatCentral",
    "HeatIRoomController2",
    "Heatcurve",
    "Heatmixer",
    "Heatmixer2",
    "HourCounter",
    "HvacAC",
    "IRoomcontrol",
    "InputRef",
    "Int",
    "Irrigation",
    "IRoomController",
    "Jalousie",
    "JalousieUpDown2",
    "JoinWindowSensor",
    "Leaf",
    "Less",
    "LessEqual",
    "LightController",
    "LightController2",
    "LightControllerH",
    "LightControllerV2",
    "Lightscene",
    "LightsceneC",
    "LightsceneLearn",
    "LightsceneRGB",
    "LoadShed",
    "LongClick",
    "MailBox",
    "MailGen",
    "Marker",
    "Media",
    "MediaClient",
    "Memory",
    "MeterAbsBi",
    "MeterAbsSt",
    "MeterAbsUni",
    "MeterDig",
    "MeterPBi",
    "MeterPSt",
    "MeterPUni",
    "MinMax",
    "Minmax",
    "Mod",
    "Monoflop",
    "Mood",
    "MsShortcut",
    "Mult",
    "MultiClick",
    "MultiFuncSW",
    "Multiplexer",
    "MusicPlayer",
    "Nevo",
    "Nor",
    "Not",
    "NotEqual",
    "OffDelay",
    "OnDelay",
    "OnOffDelay",
    "OnPulseDelay",
    "Or",
    "OutputRef",
    "PButtonT",
    "PI",
    "PID",
    "PVProductionForecast",
    "PWM",
    "Ping",
    "Plugin",
    "PoolController",
    "Power",
    "PowerUnit",
    "Presence",
    "PresenceController",
    "PresenceDetector",
    "PulseAt",
    "PulseBy",
    "PulseGen",
    "Pushbutton",
    "PushButton",
    "PushButton2",
    "PushButton2Sel",
    "PushButtonSel",
    "PushDimmer",
    "Radio",
    "Radio2",
    "Ramp",
    "Rand",
    "RandomGen",
    "RetOnDelay",
    "RoofWindow",
    "Roomcontrol",
    "Random",
    "Remote",
    "RsFlipFlop",
    "RSFlipFlop",
    "Sauna",
    "SaunaVapor",
    "Scene",
    "SequenceController",
    "Sequencer",
    "Sequence",
    "ShadeRoof",
    "Shift",
    "SmokeAlarm",
    "Solarpumpcontrol",
    "SpotOpt",
    "StairwayLS",
    "State",
    "Statistic",
    "Statistics",
    "StatusMonitor",
    "SteakThermo",
    "StepSel",
    "Sub",
    "Switch",
    "SysVar",
    "SystemScheme",
    "Tablet",
    "Text",
    "TextGenerator",
    "TextState",
    "TimeMinmax",
    "Timer",
    "ToiletFan",
    "TpfController",
    "Tracker",
    "UpDownCounter",
    "Validator",
    "Ventilation",
    "VentInternorm",
    "VirtualIn",
    "VirtualOut",
    "VirtualState",
    "WBEM",
    "WeatherServer",
    "Weed",
    "Wind",
    "WindowsMonitor",
    "Xor",
];

#[derive(Clone, Default)]
struct DeviceBusContext {
    bus_type: Option<String>,
    bus_serial: Option<String>,
    parent_uuid: Option<String>,
}

#[derive(Clone)]
struct ConnectorLookupEntry {
    block_uuid: String,
    block_title: String,
    block_type: String,
    connector_uuid: String,
    connector_key: String,
    direction: String,
}

impl ConnectorLookupEntry {
    fn wire_endpoint(&self) -> ConfigWireEndpoint {
        ConfigWireEndpoint {
            block_uuid: self.block_uuid.clone(),
            block_title: self.block_title.clone(),
            block_type: self.block_type.clone(),
            connector_uuid: self.connector_uuid.clone(),
            connector_key: self.connector_key.clone(),
        }
    }
}

fn room_name_for_block(block: &Element, room_names: &HashMap<String, String>) -> String {
    let mut room_id = String::new();
    for child in &block.children {
        if let Some(io) = child.as_element()
            && io.name == "IoData"
        {
            room_id = io.attributes.get("Pr").cloned().unwrap_or_default();
        }
    }
    room_names
        .get(&room_id)
        .cloned()
        .unwrap_or_else(|| "(unassigned)".to_string())
}

fn is_sentinel_uuid(uuid: &str) -> bool {
    let normalized: String = uuid.chars().filter(|c| *c != '-').collect();
    !normalized.is_empty() && normalized.chars().all(|c| c == '0')
}

fn is_input_connector_key(key: &str) -> bool {
    key.starts_with('I') || key.starts_with("AI") || key.starts_with("Input")
}

fn infer_connector_direction(key: &str) -> String {
    if is_input_connector_key(key) {
        "I".to_string()
    } else if key.starts_with('Q') || key.starts_with("AQ") {
        "O".to_string()
    } else {
        "?".to_string()
    }
}

fn is_describe_skipped_type(etype: &str) -> bool {
    matches!(
        etype,
        "InputRef"
            | "OutputRef"
            | "StateV"
            | "VirtualIn"
            | "VirtualOut"
            | "VirtualState"
            | "Page"
            | "Program"
            | "Document"
            | "Category"
            | "CategoryCaption"
            | "Place"
            | "PlaceCaption"
            | "ConstantCaption"
            | "CalendarCaption"
            | "VirtualInCaption"
            | "VirtualOutCaption"
            | "LoxCaption"
            | "TaskCaption"
            | "WeatherCaption"
            | "LoggerOutCaption"
            | "DateTime"
            | "Day"
            | "Day2009"
            | "DayOfWeek"
            | "Daylight"
            | "Daylight2"
            | "Online"
            | "Co"
            | "In"
            | "IoData"
            | "Display"
            | "SET"
            | "Key"
            | "ApiActor"
            | "LoxTree"
            | "LoxAIR"
            | "LoxLIVE"
            | "LoxMORE"
            | "MBusExtension"
            | "Devicemonitor"
            | "MessageCenter"
            | "GlobalStates"
            | "Comm1wire"
            | "Comm232"
            | "Comm485"
            | "CommDMX"
    )
}

fn is_device_container_type(etype: &str) -> bool {
    matches!(
        etype,
        "AirExtension"
            | "DALIextension"
            | "DaliExtension"
            | "KNXExtension"
            | "KNXextension"
            | "EIBExtension"
            | "EIBextension"
            | "LoxAIRextension"
            | "TreeExtension"
            | "1WireExtension"
            | "OneWireExtension"
            | "Comm1wire"
    )
}

fn is_excluded_device_type(etype: &str) -> bool {
    EXCLUDED_BLOCK_TYPES.contains(&etype)
        || etype.contains("Calculator")
        || etype.contains("Comparator")
        || etype.contains("Counter")
        || etype.contains("Memory")
        || etype.contains("Statistic")
        || etype.contains("Timer")
        || etype.contains("WeatherServer")
}

fn first_attr(elem: &Element, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| elem.attributes.get(*name))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn first_attr_in_subtree(elem: &Element, names: &[&str]) -> Option<String> {
    first_attr(elem, names).or_else(|| {
        elem.children
            .iter()
            .filter_map(|child| child.as_element())
            .find_map(|child| first_attr_in_subtree(child, names))
    })
}

fn find_knx_address(elem: &Element) -> Option<String> {
    first_attr_in_subtree(
        elem,
        &[
            "GroupAddress",
            "GroupAddr",
            "KNXAddress",
            "KnxAddress",
            "EIBAddress",
            "EibAddress",
            "BusAddress",
            "Address",
        ],
    )
    .map(|address| address.replace('/', "."))
}

fn room_label_for_device(elem: &Element, room_names: &HashMap<String, String>) -> Option<String> {
    if let Some(room) = elem
        .children
        .iter()
        .filter_map(|child| child.as_element())
        .find(|child| child.name == "IoData")
        .and_then(|io| io.attributes.get("Pr"))
        .and_then(|room_id| room_names.get(room_id))
        .cloned()
        .filter(|room| !room.is_empty())
    {
        return Some(room);
    }

    elem.children
        .iter()
        .filter_map(|child| child.as_element())
        .find_map(|child| room_label_for_device(child, room_names))
}

fn channel_index_from_role(role: &str) -> Option<u32> {
    let digits: String = role
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn connector_type_from_role(role: &str, direction: Option<&str>) -> String {
    if role.starts_with("CH") {
        "channel".to_string()
    } else {
        match direction {
            Some("I") => "input".to_string(),
            Some("O") => "output".to_string(),
            Some("P") => "parameter".to_string(),
            _ if role.starts_with("AI") || role.starts_with('I') => "input".to_string(),
            _ if role.starts_with("AQ") || role.starts_with('Q') => "output".to_string(),
            _ => "parameter".to_string(),
        }
    }
}

fn collect_device_connectors(
    elem: &Element,
    connector_map: &super::ConnectorMap,
) -> Vec<DetectedDeviceConnector> {
    let mut connectors = Vec::new();
    collect_device_connectors_inner(elem, connector_map, &mut connectors);
    connectors.sort_by(|a, b| (&a.role, &a.uuid).cmp(&(&b.role, &b.uuid)));
    connectors.dedup_by(|a, b| a.uuid == b.uuid);
    connectors
}

fn collect_device_connectors_inner(
    elem: &Element,
    connector_map: &super::ConnectorMap,
    connectors: &mut Vec<DetectedDeviceConnector>,
) {
    let block_type = elem
        .attributes
        .get("Type")
        .map(String::as_str)
        .unwrap_or("");
    let connector_types = connector_map.get(block_type).map(|(_, _, types)| types);
    for child in elem.children.iter().filter_map(|child| child.as_element()) {
        if child.name == "Co" {
            let uuid = child.attributes.get("U").cloned().unwrap_or_default();
            if uuid.is_empty() || is_sentinel_uuid(&uuid) {
                continue;
            }
            let role = child.attributes.get("K").cloned().unwrap_or_default();
            let direction = connector_types.and_then(|types| types.get(&role).map(String::as_str));
            connectors.push(DetectedDeviceConnector {
                uuid,
                channel_index: channel_index_from_role(&role),
                connector_type: connector_type_from_role(&role, direction),
                role,
            });
        } else {
            collect_device_connectors_inner(child, connector_map, connectors);
        }
    }
}

fn collect_secondary_block_uuids(elem: &Element, primary_uuid: &str) -> Vec<String> {
    let mut uuids = Vec::new();
    collect_secondary_block_uuids_inner(elem, primary_uuid, &mut uuids);
    uuids.sort();
    uuids.dedup();
    uuids
}

fn collect_secondary_block_uuids_inner(
    elem: &Element,
    primary_uuid: &str,
    uuids: &mut Vec<String>,
) {
    for child in elem.children.iter().filter_map(|child| child.as_element()) {
        if child.name == "C" {
            let block_type = child
                .attributes
                .get("Type")
                .map(String::as_str)
                .unwrap_or("");
            let title = child
                .attributes
                .get("Title")
                .map(String::as_str)
                .unwrap_or("");
            if !title.is_empty()
                && !is_describe_skipped_type(block_type)
                && let Some(uuid) = child.attributes.get("U")
                && uuid != primary_uuid
            {
                uuids.push(uuid.clone());
            }
        }
        collect_secondary_block_uuids_inner(child, primary_uuid, uuids);
    }
}

fn stable_device_key(identity: &DetectedDeviceIdentity) -> String {
    let raw = format!(
        "{}|{}|{}|{}",
        identity.bus_type,
        identity.bus_serial.as_deref().unwrap_or(""),
        identity.bus_address.as_deref().unwrap_or(""),
        identity.channel_role.as_deref().unwrap_or("")
    );
    hex::encode(Sha256::digest(raw.as_bytes()))
}

fn device_id_from_key(stable_key: &str) -> String {
    format!("dev-{}", &stable_key[..16])
}

fn push_or_merge_device(devices: &mut Vec<DetectedDevice>, mut device: DetectedDevice) {
    if let Some(existing) = devices
        .iter_mut()
        .find(|existing| existing.stable_device_key == device.stable_device_key)
    {
        if device.primary_block_uuid < existing.primary_block_uuid {
            existing
                .secondary_block_uuids
                .push(existing.primary_block_uuid.clone());
            existing.primary_block_uuid = device.primary_block_uuid;
        } else if device.primary_block_uuid != existing.primary_block_uuid {
            existing
                .secondary_block_uuids
                .push(device.primary_block_uuid);
        }
        existing
            .secondary_block_uuids
            .append(&mut device.secondary_block_uuids);
        existing.secondary_block_uuids.sort();
        existing.secondary_block_uuids.dedup();
        existing.connectors.append(&mut device.connectors);
        existing
            .connectors
            .sort_by(|a, b| (&a.role, &a.uuid).cmp(&(&b.role, &b.uuid)));
        existing.connectors.dedup_by(|a, b| a.uuid == b.uuid);
        if existing.snapshot_room_label.is_none() {
            existing.snapshot_room_label = device.snapshot_room_label;
        }
        existing.low_confidence_identity &= device.low_confidence_identity;
    } else {
        devices.push(device);
    }
}

fn build_detected_device(
    elem: &Element,
    room_names: &HashMap<String, String>,
    connector_map: &super::ConnectorMap,
    identity: DetectedDeviceIdentity,
    low_confidence_identity: bool,
) -> Option<DetectedDevice> {
    let primary_block_uuid = elem.attributes.get("U")?.clone();
    let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
    let title = elem.attributes.get("Title").cloned().unwrap_or_default();
    let device_type = first_attr(elem, &["DeviceType", "HardwareType", "ProductType"])
        .unwrap_or_else(|| block_type.clone());
    let stable_device_key = stable_device_key(&identity);
    Some(DetectedDevice {
        device_id: device_id_from_key(&stable_device_key),
        stable_device_key,
        bus_type: identity.bus_type.clone(),
        bus_serial: identity.bus_serial.clone(),
        bus_address: identity.bus_address.clone(),
        device_type,
        secondary_block_uuids: collect_secondary_block_uuids(elem, &primary_block_uuid),
        connectors: collect_device_connectors(elem, connector_map),
        snapshot_room_label: room_label_for_device(elem, room_names),
        derived_label: if title.is_empty() { block_type } else { title },
        low_confidence_identity,
        identity_components: identity,
        primary_block_uuid,
    })
}

fn extension_bus_context(elem: &Element, context: &DeviceBusContext) -> DeviceBusContext {
    let mut next = context.clone();
    let etype = elem
        .attributes
        .get("Type")
        .map(String::as_str)
        .unwrap_or("");
    let bus_serial = first_attr(elem, &["Serial", "BusSerial", "Address", "U"]);
    match etype {
        "TreeExtension" | "LoxAIRextension" => {
            next.bus_type = Some("tree".to_string());
            next.bus_serial = bus_serial;
            next.parent_uuid = elem.attributes.get("U").cloned();
        }
        "DALIextension" | "DaliExtension" => {
            next.bus_type = Some("dali".to_string());
            next.bus_serial = bus_serial;
            next.parent_uuid = elem.attributes.get("U").cloned();
        }
        "KNXExtension" | "KNXextension" | "EIBExtension" | "EIBextension" => {
            next.bus_type = Some("knx".to_string());
            next.bus_serial = elem.attributes.get("U").cloned();
            next.parent_uuid = elem.attributes.get("U").cloned();
        }
        "AirExtension" => {
            next.bus_type = Some("loxone-air".to_string());
            next.bus_serial = bus_serial;
            next.parent_uuid = elem.attributes.get("U").cloned();
        }
        "1WireExtension" | "OneWireExtension" | "Comm1wire" => {
            next.bus_type = Some("1-wire".to_string());
            next.bus_serial = bus_serial;
            next.parent_uuid = elem.attributes.get("U").cloned();
        }
        _ => {}
    }
    next
}

fn collect_config_devices(
    elem: &Element,
    context: &DeviceBusContext,
    room_names: &HashMap<String, String>,
    connector_map: &super::ConnectorMap,
    devices: &mut Vec<DetectedDevice>,
) {
    let etype = elem
        .attributes
        .get("Type")
        .map(String::as_str)
        .unwrap_or("");
    let next_context = extension_bus_context(elem, context);

    if elem.name != "C" {
        for child in elem.children.iter().filter_map(|child| child.as_element()) {
            collect_config_devices(child, &next_context, room_names, connector_map, devices);
        }
        return;
    }

    if is_device_container_type(etype) {
        for child in elem.children.iter().filter_map(|child| child.as_element()) {
            collect_config_devices(child, &next_context, room_names, connector_map, devices);
        }
        return;
    }

    if is_describe_skipped_type(etype) || etype.is_empty() {
        for child in elem.children.iter().filter_map(|child| child.as_element()) {
            collect_config_devices(child, &next_context, room_names, connector_map, devices);
        }
        return;
    }

    let title = elem
        .attributes
        .get("Title")
        .map(String::as_str)
        .unwrap_or("");
    if title.is_empty() {
        for child in elem.children.iter().filter_map(|child| child.as_element()) {
            collect_config_devices(child, &next_context, room_names, connector_map, devices);
        }
        return;
    }

    let primary_uuid = elem.attributes.get("U").cloned().unwrap_or_default();
    let context_bus_type = next_context.bus_type.as_deref();
    let etype_lower = etype.to_ascii_lowercase();
    let tree_serial = first_attr(elem, &["Serial", "BusSerial"])
        .or_else(|| first_attr_in_subtree(elem, &["Serial", "BusSerial"]));
    let bus_address = first_attr(
        elem,
        &["BusAddress", "Address", "Addr", "Adr", "DaliAddress"],
    );

    let identity = if (context_bus_type == Some("tree") || etype.starts_with("Tree"))
        && tree_serial.is_some()
    {
        Some((
            DetectedDeviceIdentity {
                bus_type: "tree".to_string(),
                bus_serial: next_context.bus_serial.clone(),
                bus_address: tree_serial,
                channel_role: None,
            },
            false,
        ))
    } else if context_bus_type == Some("dali") || etype_lower.contains("dali") {
        bus_address.clone().map(|address| {
            (
                DetectedDeviceIdentity {
                    bus_type: "dali".to_string(),
                    bus_serial: next_context.bus_serial.clone(),
                    bus_address: Some(address),
                    channel_role: None,
                },
                false,
            )
        })
    } else if context_bus_type == Some("knx")
        || etype.starts_with("KNX")
        || etype.starts_with("EIB")
    {
        find_knx_address(elem).map(|address| {
            (
                DetectedDeviceIdentity {
                    bus_type: "knx".to_string(),
                    bus_serial: next_context
                        .parent_uuid
                        .clone()
                        .or_else(|| next_context.bus_serial.clone()),
                    bus_address: Some(address),
                    channel_role: None,
                },
                false,
            )
        })
    } else if context_bus_type == Some("loxone-air") || etype_lower.contains("air") {
        first_attr(elem, &["BusSerial", "Serial", "IP"])
            .or_else(|| first_attr_in_subtree(elem, &["BusSerial", "Serial", "IP"]))
            .map(|serial| {
                (
                    DetectedDeviceIdentity {
                        bus_type: "loxone-air".to_string(),
                        bus_serial: next_context.bus_serial.clone(),
                        bus_address: Some(serial),
                        channel_role: None,
                    },
                    false,
                )
            })
    } else if context_bus_type == Some("1-wire")
        || etype_lower.contains("1wire")
        || etype_lower.contains("onewire")
    {
        first_attr(elem, &["BusAddress", "Address", "Serial", "Addr", "Adr"])
            .or_else(|| first_attr_in_subtree(elem, &["BusAddress", "Address", "Serial"]))
            .map(|address| {
                (
                    DetectedDeviceIdentity {
                        bus_type: "1-wire".to_string(),
                        bus_serial: next_context.bus_serial.clone(),
                        bus_address: Some(address),
                        channel_role: None,
                    },
                    false,
                )
            })
    } else {
        None
    };

    if identity.is_none() && is_excluded_device_type(etype) {
        for child in elem.children.iter().filter_map(|child| child.as_element()) {
            collect_config_devices(child, &next_context, room_names, connector_map, devices);
        }
        return;
    }

    let (identity, low_confidence_identity) = identity.unwrap_or_else(|| {
        (
            DetectedDeviceIdentity {
                bus_type: "standalone".to_string(),
                bus_serial: None,
                bus_address: None,
                channel_role: Some(primary_uuid),
            },
            true,
        )
    });

    if let Some(device) = build_detected_device(
        elem,
        room_names,
        connector_map,
        identity,
        low_confidence_identity,
    ) {
        push_or_merge_device(devices, device);
    }
}

fn resolve_source_endpoint<'a>(
    candidates: Option<&'a Vec<ConnectorLookupEntry>>,
    target_block_uuid: &str,
    target_connector_uuid: &str,
) -> Option<&'a ConnectorLookupEntry> {
    let candidates = candidates?;
    let is_self_reference = |c: &ConnectorLookupEntry| {
        c.block_uuid == target_block_uuid && c.connector_uuid == target_connector_uuid
    };
    candidates
        .iter()
        .find(|c| !is_self_reference(c) && c.direction == "O")
        .or_else(|| {
            candidates
                .iter()
                .find(|c| !is_self_reference(c) && !is_input_connector_key(&c.connector_key))
        })
        .or_else(|| candidates.iter().find(|c| !is_self_reference(c)))
}

impl ConfigEditor {
    fn room_names(&self) -> HashMap<String, String> {
        let mut room_names = HashMap::new();
        for e in self.iter_elements(&self.root) {
            if e.attributes.get("Type").map(|s| s.as_str()) == Some("Place")
                && let (Some(u), Some(t)) = (e.attributes.get("U"), e.attributes.get("Title"))
            {
                room_names.insert(u.clone(), t.clone());
            }
        }
        room_names
    }

    /// Describe the configuration in human-readable form.
    pub fn describe_config(&self, room_filter: Option<&str>) -> String {
        let mut out = String::new();

        // Build room UUID → name map
        let mut room_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut cat_names: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for e in self.iter_elements(&self.root) {
            match e.attributes.get("Type").map(|s| s.as_str()) {
                Some("Place") => {
                    if let (Some(u), Some(t)) = (e.attributes.get("U"), e.attributes.get("Title")) {
                        room_names.insert(u.clone(), t.clone());
                    }
                }
                Some("Category") => {
                    if let (Some(u), Some(t)) = (e.attributes.get("U"), e.attributes.get("Title")) {
                        cat_names.insert(u.clone(), t.clone());
                    }
                }
                _ => {}
            }
        }

        // Collect controls by room
        let mut by_room: HashMap<String, Vec<DescribeEntry>> = HashMap::new();

        let _cmap = Self::connector_map();
        let skip_types = [
            "InputRef",
            "OutputRef",
            "StateV",
            "VirtualIn",
            "VirtualOut",
            "VirtualState",
            "Page",
            "Program",
            "Document",
            "Category",
            "CategoryCaption",
            "Place",
            "PlaceCaption",
            "ConstantCaption",
            "CalendarCaption",
            "VirtualInCaption",
            "VirtualOutCaption",
            "LoxCaption",
            "TaskCaption",
            "WeatherCaption",
            "LoggerOutCaption",
            "DateTime",
            "Day",
            "Day2009",
            "DayOfWeek",
            "Daylight",
            "Daylight2",
            "Online",
            "Co",
            "In",
            "IoData",
            "Display",
            "SET",
            "Key",
            "ApiActor",
            "LoxTree",
            "LoxAIR",
            "LoxLIVE",
            "LoxMORE",
            "MBusExtension",
            "Devicemonitor",
            "MessageCenter",
            "GlobalStates",
            "Comm1wire",
            "Comm232",
            "Comm485",
            "CommDMX",
        ];

        for e in self.iter_elements(&self.root) {
            let etype = e.attributes.get("Type").cloned().unwrap_or_default();
            if skip_types.contains(&etype.as_str()) || etype.is_empty() {
                continue;
            }
            let title = e.attributes.get("Title").cloned().unwrap_or_default();
            if title.is_empty() {
                continue;
            }

            // Find room from IoData
            let mut room_id = String::new();
            for child in &e.children {
                if let Some(io) = child.as_element()
                    && io.name == "IoData"
                {
                    room_id = io.attributes.get("Pr").cloned().unwrap_or_default();
                }
            }
            let room_name = room_names
                .get(&room_id)
                .cloned()
                .unwrap_or_else(|| "(unassigned)".to_string());

            if let Some(filter) = room_filter
                && !room_name.to_lowercase().contains(&filter.to_lowercase())
            {
                continue;
            }

            // Collect wired connectors
            let mut wired: Vec<String> = Vec::new();
            for child in &e.children {
                if let Some(co) = child.as_element()
                    && co.name == "Co"
                {
                    let k = co.attributes.get("K").cloned().unwrap_or_default();
                    let has_in = co
                        .children
                        .iter()
                        .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
                    if has_in {
                        wired.push(k);
                    }
                }
            }

            // Collect scenes for LightController2
            let mut scenes: Vec<String> = Vec::new();
            if etype == "LightController2" {
                for child in &e.children {
                    if let Some(lsc) = child.as_element()
                        && lsc.name == "LightscenesC"
                    {
                        for scene in &lsc.children {
                            if let Some(sc) = scene.as_element()
                                && sc.name == "LightsceneC"
                                && let Some(name) = sc.attributes.get("Name")
                            {
                                scenes.push(name.clone());
                            }
                        }
                    }
                }
            }

            by_room
                .entry(room_name)
                .or_default()
                .push((etype, title, scenes, wired));
        }

        // Format output
        let mut rooms: Vec<_> = by_room.into_iter().collect();
        rooms.sort_by(|a, b| a.0.cmp(&b.0));

        for (room, controls) in &rooms {
            out.push_str(&format!("\n{}  ({} controls)\n", room, controls.len()));
            out.push_str(&format!("{}\n", "─".repeat(room.len() + 15)));
            for (etype, title, scenes, _wired) in controls {
                if scenes.is_empty() {
                    out.push_str(&format!("  {} ({})\n", title, etype));
                } else {
                    out.push_str(&format!(
                        "  {} ({}) — moods: {}\n",
                        title,
                        etype,
                        scenes.join(", ")
                    ));
                }
            }
        }

        if rooms.is_empty() {
            out.push_str("No controls found.\n");
        } else {
            let total: usize = rooms.iter().map(|(_, c)| c.len()).sum();
            out.push_str(&format!(
                "\n{} controls across {} rooms\n",
                total,
                rooms.len()
            ));
        }

        out
    }

    /// Describe the configuration as structured JSON-serializable data.
    pub fn describe_config_structured(&self, room_filter: Option<&str>) -> Vec<DescribeRoomEntry> {
        // Build room UUID → name map
        let mut room_names: HashMap<String, String> = HashMap::new();
        for e in self.iter_elements(&self.root) {
            if e.attributes.get("Type").map(|s| s.as_str()) == Some("Place")
                && let (Some(u), Some(t)) = (e.attributes.get("U"), e.attributes.get("Title"))
            {
                room_names.insert(u.clone(), t.clone());
            }
        }

        let skip_types = [
            "InputRef",
            "OutputRef",
            "StateV",
            "VirtualIn",
            "VirtualOut",
            "VirtualState",
            "Page",
            "Program",
            "Document",
            "Category",
            "CategoryCaption",
            "Place",
            "PlaceCaption",
            "ConstantCaption",
            "CalendarCaption",
            "VirtualInCaption",
            "VirtualOutCaption",
            "LoxCaption",
            "TaskCaption",
            "WeatherCaption",
            "LoggerOutCaption",
            "DateTime",
            "Day",
            "Day2009",
            "DayOfWeek",
            "Daylight",
            "Daylight2",
            "Online",
            "Co",
            "In",
            "IoData",
            "Display",
            "SET",
            "Key",
            "ApiActor",
            "LoxTree",
            "LoxAIR",
            "LoxLIVE",
            "LoxMORE",
            "MBusExtension",
            "Devicemonitor",
            "MessageCenter",
            "GlobalStates",
            "Comm1wire",
            "Comm232",
            "Comm485",
            "CommDMX",
        ];

        let mut by_room: HashMap<String, Vec<DescribeBlockEntry>> = HashMap::new();

        for e in self.iter_elements(&self.root) {
            let etype = e.attributes.get("Type").cloned().unwrap_or_default();
            if skip_types.contains(&etype.as_str()) || etype.is_empty() {
                continue;
            }
            let title = e.attributes.get("Title").cloned().unwrap_or_default();
            if title.is_empty() {
                continue;
            }
            let uuid = e.attributes.get("U").cloned().unwrap_or_default();

            let mut room_id = String::new();
            for child in &e.children {
                if let Some(io) = child.as_element()
                    && io.name == "IoData"
                {
                    room_id = io.attributes.get("Pr").cloned().unwrap_or_default();
                }
            }
            let room_name = room_names
                .get(&room_id)
                .cloned()
                .unwrap_or_else(|| "(unassigned)".to_string());

            if let Some(filter) = room_filter
                && !room_name.to_lowercase().contains(&filter.to_lowercase())
            {
                continue;
            }

            // Collect connectors with UUIDs and wiring status
            let cmap = Self::connector_map();
            let types = cmap
                .get(&etype)
                .map(|(_, _, t)| t.clone())
                .unwrap_or_default();

            let mut connectors = Vec::new();
            for child in &e.children {
                if let Some(co) = child.as_element()
                    && co.name == "Co"
                {
                    let k = co.attributes.get("K").cloned().unwrap_or_default();
                    let co_uuid = co.attributes.get("U").cloned().unwrap_or_default();
                    let io = types.get(&k).cloned().unwrap_or_else(|| "?".to_string());
                    let wired = co
                        .children
                        .iter()
                        .any(|c| c.as_element().is_some_and(|e| e.name == "In"));
                    let def = co.attributes.get("Def").cloned();
                    connectors.push(DescribeConnectorEntry {
                        key: k,
                        uuid: co_uuid,
                        direction: io,
                        wired,
                        default: def,
                    });
                }
            }

            by_room
                .entry(room_name)
                .or_default()
                .push(DescribeBlockEntry {
                    block_type: etype,
                    title,
                    uuid,
                    connectors,
                });
        }

        let mut rooms: Vec<_> = by_room.into_iter().collect();
        rooms.sort_by(|a, b| a.0.cmp(&b.0));

        rooms
            .into_iter()
            .map(|(room, blocks)| DescribeRoomEntry { room, blocks })
            .collect()
    }

    /// List all resolved config-wide wires as JSON-serializable edges.
    pub fn config_wires(&self, room_filter: Option<&str>) -> Vec<ConfigWire> {
        let room_names = self.room_names();
        let connector_map = Self::connector_map();
        let mut endpoints_by_uuid: HashMap<String, Vec<ConnectorLookupEntry>> = HashMap::new();

        for block in self.iter_elements(&self.root) {
            if block.name != "C" {
                continue;
            }

            let block_uuid = block.attributes.get("U").cloned().unwrap_or_default();
            let block_title = block.attributes.get("Title").cloned().unwrap_or_default();
            let block_type = block.attributes.get("Type").cloned().unwrap_or_default();
            if block_title.is_empty()
                || block_type.is_empty()
                || is_describe_skipped_type(&block_type)
            {
                continue;
            }
            let connector_types = connector_map
                .get(&block_type)
                .map(|(_, _, types)| types)
                .cloned()
                .unwrap_or_default();

            for co in block.children.iter().filter_map(|c| c.as_element()) {
                if co.name != "Co" {
                    continue;
                }
                let Some(connector_uuid) = co.attributes.get("U").cloned() else {
                    continue;
                };
                if is_sentinel_uuid(&connector_uuid) {
                    continue;
                }
                let connector_key = co.attributes.get("K").cloned().unwrap_or_default();
                let direction = connector_types
                    .get(&connector_key)
                    .cloned()
                    .unwrap_or_else(|| infer_connector_direction(&connector_key));

                endpoints_by_uuid
                    .entry(connector_uuid.clone())
                    .or_default()
                    .push(ConnectorLookupEntry {
                        block_uuid: block_uuid.clone(),
                        block_title: block_title.clone(),
                        block_type: block_type.clone(),
                        connector_uuid,
                        connector_key,
                        direction,
                    });
            }
        }

        let mut wires = Vec::new();

        for block in self.iter_elements(&self.root) {
            if block.name != "C" {
                continue;
            }

            let block_uuid = block.attributes.get("U").cloned().unwrap_or_default();
            let block_title = block.attributes.get("Title").cloned().unwrap_or_default();
            let block_type = block.attributes.get("Type").cloned().unwrap_or_default();
            if block_title.is_empty()
                || block_type.is_empty()
                || is_describe_skipped_type(&block_type)
            {
                continue;
            }
            let room_name = room_name_for_block(block, &room_names);
            if let Some(filter) = room_filter
                && !room_name.to_lowercase().contains(&filter.to_lowercase())
            {
                continue;
            }

            for co in block.children.iter().filter_map(|c| c.as_element()) {
                if co.name != "Co" {
                    continue;
                }

                let connector_key = co.attributes.get("K").cloned().unwrap_or_default();
                let connector_uuid = co.attributes.get("U").cloned().unwrap_or_default();
                let target = ConfigWireEndpoint {
                    block_uuid: block_uuid.clone(),
                    block_title: block_title.clone(),
                    block_type: block_type.clone(),
                    connector_uuid: connector_uuid.clone(),
                    connector_key: connector_key.clone(),
                };

                let input_refs: Vec<String> = co
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "In")
                    .filter_map(|e| e.attributes.get("Input").cloned())
                    .filter(|u| !is_sentinel_uuid(u))
                    .collect();

                if !input_refs.is_empty() {
                    if is_sentinel_uuid(&connector_uuid) {
                        continue;
                    }
                    for source_uuid in input_refs {
                        if let Some(source) = resolve_source_endpoint(
                            endpoints_by_uuid.get(&source_uuid),
                            &block_uuid,
                            &connector_uuid,
                        ) {
                            wires.push(ConfigWire {
                                source: source.wire_endpoint(),
                                target: target.clone(),
                            });
                        }
                    }
                    continue;
                }

                if is_sentinel_uuid(&connector_uuid) || !is_input_connector_key(&connector_key) {
                    continue;
                }

                if let Some(source) = resolve_source_endpoint(
                    endpoints_by_uuid.get(&connector_uuid),
                    &block_uuid,
                    &connector_uuid,
                ) {
                    wires.push(ConfigWire {
                        source: source.wire_endpoint(),
                        target,
                    });
                }
            }
        }

        wires.sort_by(|a, b| {
            (
                &a.target.block_uuid,
                &a.target.connector_key,
                &a.source.block_uuid,
            )
                .cmp(&(
                    &b.target.block_uuid,
                    &b.target.connector_key,
                    &b.source.block_uuid,
                ))
        });

        wires
    }

    /// Detect physical devices from the configuration tree.
    pub fn config_devices(&self, room_filter: Option<&str>) -> Vec<DetectedDevice> {
        let room_names = self.room_names();
        let connector_map = Self::connector_map();
        let mut devices = Vec::new();
        collect_config_devices(
            &self.root,
            &DeviceBusContext::default(),
            &room_names,
            &connector_map,
            &mut devices,
        );

        if let Some(filter) = room_filter {
            let filter = filter.to_lowercase();
            devices.retain(|device| {
                device
                    .snapshot_room_label
                    .as_ref()
                    .is_some_and(|room| room.to_lowercase().contains(&filter))
            });
        }

        devices.sort_by(|a, b| {
            (
                &a.bus_type,
                &a.bus_serial,
                &a.bus_address,
                &a.identity_components.channel_role,
                &a.primary_block_uuid,
            )
                .cmp(&(
                    &b.bus_type,
                    &b.bus_serial,
                    &b.bus_address,
                    &b.identity_components.channel_role,
                    &b.primary_block_uuid,
                ))
        });

        devices
    }

    /// Compute comprehensive config statistics in a single tree walk.
    pub fn config_stats(&self) -> ConfigStats {
        let lighting_types = ["LightController2", "LightController", "LightControllerV2"];
        let blind_types = ["JalousieUpDown2", "EIBJalousie", "AutoJalousie", "Jalousie"];
        let climate_types = [
            "HeatIRoomController2",
            "IRoomcontrol",
            "Thermostat",
            "AcControl",
            "IRoomController",
        ];
        let presence_types = ["Presence", "PresenceDetector"];

        let mut room_count = 0usize;
        let mut page_count = 0usize;
        let mut category_count = 0usize;
        let mut total_items = 0usize;
        let mut block_type_counts: HashMap<String, usize> = HashMap::new();
        let mut wiring_total = 0usize;
        let mut wiring_cross_page = 0usize;
        let mut wiring_multi_input = 0usize;

        // Room UUID → name
        let mut room_names: HashMap<String, String> = HashMap::new();
        // Room UUID → completeness flags
        let mut room_flags: HashMap<String, (bool, bool, bool, bool)> = HashMap::new();

        // LightController2 UUID → (title, Vec<mood_name>)
        let mut lc2_scenes: HashMap<String, (String, Vec<String>)> = HashMap::new();

        // Device bus: "Tree" | "Air" | "Network" → Vec<device_name>
        let mut device_bus: HashMap<String, Vec<String>> = HashMap::new();

        for elem in self.iter_elements(&self.root) {
            // Count wiring: Co elements don't have Type, handle before type check
            if elem.name == "Co" {
                let in_children: Vec<&Element> = elem
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "In")
                    .collect();
                let count = in_children.len();
                wiring_total += count;
                if count > 1 {
                    wiring_multi_input += 1;
                }
                for in_elem in &in_children {
                    if in_elem.attributes.get("FLG").is_some_and(|f| f == "2") {
                        wiring_cross_page += 1;
                    }
                }
            }

            let etype = match elem.attributes.get("Type") {
                Some(t) => t.as_str(),
                None => continue,
            };

            // Count structural elements
            match etype {
                "Place" => {
                    room_count += 1;
                    if let (Some(u), Some(t)) =
                        (elem.attributes.get("U"), elem.attributes.get("Title"))
                    {
                        room_names.insert(u.clone(), t.clone());
                        room_flags.entry(u.clone()).or_default();
                    }
                }
                "Page" => page_count += 1,
                "Category" => category_count += 1,
                _ => {}
            }

            // Count all C elements as items (blocks that have a Type and Title)
            if elem.name == "C" && elem.attributes.contains_key("Title") {
                total_items += 1;
            }

            // Count block types (only named C elements)
            if elem.name == "C" && elem.attributes.contains_key("Title") && !etype.is_empty() {
                *block_type_counts.entry(etype.to_string()).or_default() += 1;
            }

            // Track room completeness from IoData
            if elem.name == "C" {
                let mut room_uuid = String::new();
                for child in &elem.children {
                    if let Some(io) = child.as_element()
                        && io.name == "IoData"
                    {
                        room_uuid = io.attributes.get("Pr").cloned().unwrap_or_default();
                    }
                }
                if !room_uuid.is_empty() {
                    let flags = room_flags.entry(room_uuid).or_default();
                    if lighting_types.contains(&etype) {
                        flags.0 = true;
                    }
                    if blind_types.contains(&etype) {
                        flags.1 = true;
                    }
                    if climate_types.contains(&etype) {
                        flags.2 = true;
                    }
                    if presence_types.contains(&etype) {
                        flags.3 = true;
                    }
                }
            }

            // Collect LightController2 moods
            if lighting_types.contains(&etype)
                && let Some(uuid) = elem.attributes.get("U")
            {
                let title = elem.attributes.get("Title").cloned().unwrap_or_default();
                let mut moods = Vec::new();
                for child in &elem.children {
                    if let Some(lsc) = child.as_element()
                        && lsc.name == "C"
                        && lsc
                            .attributes
                            .get("Type")
                            .is_some_and(|t| t == "LightscenesC")
                    {
                        for sc_child in &lsc.children {
                            if let Some(sc) = sc_child.as_element()
                                && sc.name == "C"
                                && sc
                                    .attributes
                                    .get("Type")
                                    .is_some_and(|t| t == "LightsceneC" || t == "Lightscene")
                                && let Some(name) = sc.attributes.get("Title")
                            {
                                moods.push(name.clone());
                            }
                        }
                    }
                }
                if !moods.is_empty() {
                    lc2_scenes.insert(uuid.clone(), (title, moods));
                }
            }

            // Count devices by bus type
            match etype {
                "TreeDevice" => {
                    let name = elem
                        .attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    device_bus.entry("Tree".to_string()).or_default().push(name);
                }
                "LoxAIRDevice" => {
                    let name = elem
                        .attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    device_bus.entry("Air".to_string()).or_default().push(name);
                }
                "NetworkDevice" => {
                    let name = elem
                        .attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    device_bus
                        .entry("Network".to_string())
                        .or_default()
                        .push(name);
                }
                _ => {}
            }
        }

        // Sort block types by count descending
        let mut block_types: Vec<(String, usize)> = block_type_counts.into_iter().collect();
        block_types.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Build room completeness list, sorted by name
        let mut room_completeness: Vec<RoomCompleteness> = room_flags
            .iter()
            .filter_map(|(uuid, (l, b, c, p))| {
                let name = room_names.get(uuid)?.clone();
                // Only include rooms that have at least one flag set
                if *l || *b || *c || *p {
                    Some(RoomCompleteness {
                        name,
                        has_lighting: *l,
                        has_blinds: *b,
                        has_climate: *c,
                        has_presence: *p,
                    })
                } else {
                    None
                }
            })
            .collect();
        room_completeness.sort_by(|a, b| a.name.cmp(&b.name));

        // Build scenes list
        let mut scenes: Vec<SceneInfo> = lc2_scenes
            .into_values()
            .map(|(title, moods)| SceneInfo {
                control_name: title,
                mood_count: moods.len(),
                mood_names: moods,
            })
            .collect();
        scenes.sort_by(|a, b| a.control_name.cmp(&b.control_name));

        // Build device bus summaries
        let mut devices: Vec<DeviceBusSummary> = device_bus
            .into_iter()
            .map(|(bus, names)| {
                let mut unique = names.clone();
                unique.sort();
                unique.dedup();
                DeviceBusSummary {
                    bus_type: bus,
                    count: names.len(),
                    device_names: unique,
                }
            })
            .collect();
        devices.sort_by(|a, b| a.bus_type.cmp(&b.bus_type));

        ConfigStats {
            room_count,
            page_count,
            category_count,
            total_items,
            block_types,
            wiring_total,
            wiring_cross_page,
            wiring_multi_input,
            room_completeness,
            scenes,
            devices,
        }
    }

    /// List all hardware I/O ports with used/free status.
    /// Walks device trees (TreeDevice, LoxAIRDevice, NetworkDevice) and finds
    /// all actor/sensor sub-elements. Cross-references with OutputRef.Ref and
    /// InputRef wiring to determine which ports are in use.
    pub fn list_device_ports(&self) -> String {
        let actor_types = [
            "Actor",
            "TreeAactor",
            "TreeActor",
            "LoxAIRAactor",
            "LoxAIRactor",
            "ApiActor",
        ];
        let sensor_types = [
            "DigitalIn",
            "VoltageIn",
            "TreeAsensor",
            "TreeSensor",
            "LoxAIRsensor",
            "LoxAIRAsensor",
        ];
        let skip_types = ["Online", "OutputCaption", "InputCaption"];

        // Collect all OutputRef Ref targets (device UUIDs that are wired)
        let mut wired_refs: HashMap<String, String> = HashMap::new(); // device_uuid → control_title
        fn collect_output_refs(elem: &Element, wired: &mut HashMap<String, String>) {
            if elem
                .attributes
                .get("Type")
                .map(|t| t == "OutputRef")
                .unwrap_or(false)
                && let (Some(ref_uuid), Some(title)) =
                    (elem.attributes.get("Ref"), elem.attributes.get("Title"))
            {
                wired.insert(ref_uuid.clone(), title.clone());
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    collect_output_refs(e, wired);
                }
            }
        }
        collect_output_refs(&self.root, &mut wired_refs);

        // Also collect InputRef wiring (sensor → control connections)
        // An InputRef with Ref pointing to a sensor's connector means it's wired
        let mut wired_input_refs: HashMap<String, String> = HashMap::new();
        fn collect_input_refs(elem: &Element, wired: &mut HashMap<String, String>) {
            if elem
                .attributes
                .get("Type")
                .map(|t| t == "InputRef")
                .unwrap_or(false)
                && let (Some(ref_uuid), Some(title)) =
                    (elem.attributes.get("Ref"), elem.attributes.get("Title"))
            {
                wired.insert(ref_uuid.clone(), title.clone());
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    collect_input_refs(e, wired);
                }
            }
        }
        collect_input_refs(&self.root, &mut wired_input_refs);

        // Walk devices and collect ports
        struct DeviceInfo {
            name: String,
            device_type: String,
            ports: Vec<PortInfo>,
        }
        struct PortInfo {
            name: String,
            iname: String,
            port_type: String, // "Actor" or "Sensor"
            uuid: String,
            status: String, // "free" or "→ Control Name"
        }

        let mut devices: Vec<DeviceInfo> = Vec::new();

        #[allow(clippy::too_many_arguments)]
        fn walk_devices(
            elem: &Element,
            actor_types: &[&str],
            sensor_types: &[&str],
            skip_types: &[&str],
            wired_refs: &HashMap<String, String>,
            wired_input_refs: &HashMap<String, String>,
            devices: &mut Vec<DeviceInfo>,
            parent_device: Option<&str>,
        ) {
            let etype = elem.attributes.get("Type").cloned().unwrap_or_default();

            // Is this a device container?
            let is_device = matches!(
                etype.as_str(),
                "TreeDevice" | "LoxAIRDevice" | "NetworkDevice"
            );
            let device_name = if is_device {
                Some(
                    elem.attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string()),
                )
            } else {
                parent_device.map(|s| s.to_string())
            };

            if is_device {
                let mut ports = Vec::new();

                // Walk children for actor/sensor ports
                #[allow(clippy::too_many_arguments)]
                fn collect_ports(
                    elem: &Element,
                    actor_types: &[&str],
                    sensor_types: &[&str],
                    skip_types: &[&str],
                    wired_refs: &HashMap<String, String>,
                    wired_input_refs: &HashMap<String, String>,
                    ports: &mut Vec<PortInfo>,
                ) {
                    let et = elem.attributes.get("Type").cloned().unwrap_or_default();
                    if skip_types.contains(&et.as_str()) {
                        return;
                    }

                    let is_actor = actor_types.contains(&et.as_str());
                    let is_sensor = sensor_types.contains(&et.as_str());

                    if is_actor || is_sensor {
                        let uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
                        let iname = elem.attributes.get("IName").cloned().unwrap_or_default();

                        // Check if wired via OutputRef (actors) or InputRef (sensors)
                        let status = if let Some(ctrl) = wired_refs.get(&uuid) {
                            format!("→ {}", ctrl)
                        } else if let Some(ctrl) = wired_input_refs.get(&uuid) {
                            format!("→ {}", ctrl)
                        } else {
                            "free".to_string()
                        };

                        ports.push(PortInfo {
                            name: title,
                            iname,
                            port_type: if is_actor {
                                "Actor".to_string()
                            } else {
                                "Sensor".to_string()
                            },
                            uuid,
                            status,
                        });
                    }

                    for child in &elem.children {
                        if let Some(e) = child.as_element() {
                            collect_ports(
                                e,
                                actor_types,
                                sensor_types,
                                skip_types,
                                wired_refs,
                                wired_input_refs,
                                ports,
                            );
                        }
                    }
                }

                collect_ports(
                    elem,
                    actor_types,
                    sensor_types,
                    skip_types,
                    wired_refs,
                    wired_input_refs,
                    &mut ports,
                );

                if !ports.is_empty() {
                    devices.push(DeviceInfo {
                        name: device_name.clone().unwrap_or_default(),
                        device_type: etype.clone(),
                        ports,
                    });
                }
            }

            // Also look at Miniserver built-in I/O (not under a device)
            if etype == "ControlList" || etype.is_empty() {
                // Walk for Actor/DigitalIn directly under root-level captions
                for child in &elem.children {
                    if let Some(e) = child.as_element() {
                        walk_devices(
                            e,
                            actor_types,
                            sensor_types,
                            skip_types,
                            wired_refs,
                            wired_input_refs,
                            devices,
                            device_name.as_deref(),
                        );
                    }
                }
                return;
            }

            // Check for caption containers that hold built-in MS I/O
            let is_caption = matches!(etype.as_str(), "OutputCaption" | "InputCaption");
            if is_caption {
                let caption_title = elem
                    .attributes
                    .get("Title")
                    .cloned()
                    .unwrap_or_else(|| "Miniserver".to_string());
                let mut ports = Vec::new();

                fn collect_ports_flat(
                    elem: &Element,
                    actor_types: &[&str],
                    sensor_types: &[&str],
                    wired_refs: &HashMap<String, String>,
                    wired_input_refs: &HashMap<String, String>,
                    ports: &mut Vec<PortInfo>,
                ) {
                    let et = elem.attributes.get("Type").cloned().unwrap_or_default();
                    let is_actor = actor_types.contains(&et.as_str());
                    let is_sensor = sensor_types.contains(&et.as_str());

                    if is_actor || is_sensor {
                        let uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
                        let iname = elem.attributes.get("IName").cloned().unwrap_or_default();
                        let status = if let Some(ctrl) = wired_refs.get(&uuid) {
                            format!("→ {}", ctrl)
                        } else if let Some(ctrl) = wired_input_refs.get(&uuid) {
                            format!("→ {}", ctrl)
                        } else {
                            "free".to_string()
                        };

                        ports.push(PortInfo {
                            name: title,
                            iname,
                            port_type: if is_actor {
                                "Actor".to_string()
                            } else {
                                "Sensor".to_string()
                            },
                            uuid,
                            status,
                        });
                    }

                    for child in &elem.children {
                        if let Some(e) = child.as_element() {
                            collect_ports_flat(
                                e,
                                actor_types,
                                sensor_types,
                                wired_refs,
                                wired_input_refs,
                                ports,
                            );
                        }
                    }
                }

                for child in &elem.children {
                    if let Some(e) = child.as_element() {
                        collect_ports_flat(
                            e,
                            actor_types,
                            sensor_types,
                            wired_refs,
                            wired_input_refs,
                            &mut ports,
                        );
                    }
                }

                if !ports.is_empty() {
                    devices.push(DeviceInfo {
                        name: caption_title,
                        device_type: "Miniserver".to_string(),
                        ports,
                    });
                }
                return;
            }

            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    walk_devices(
                        e,
                        actor_types,
                        sensor_types,
                        skip_types,
                        wired_refs,
                        wired_input_refs,
                        devices,
                        device_name.as_deref(),
                    );
                }
            }
        }

        walk_devices(
            &self.root,
            &actor_types,
            &sensor_types,
            &skip_types,
            &wired_refs,
            &wired_input_refs,
            &mut devices,
            None,
        );

        // Format output
        let mut out = String::new();
        let mut total_ports = 0;
        let mut used_ports = 0;

        for dev in &devices {
            let dev_used = dev.ports.iter().filter(|p| p.status != "free").count();
            out.push_str(&format!(
                "\n{} ({}) — {}/{} ports used\n",
                dev.name,
                dev.device_type,
                dev_used,
                dev.ports.len()
            ));
            out.push_str(&format!(
                "  {:<6} {:<30} {:<8} {}\n",
                "Port", "Name", "Kind", "Status / UUID"
            ));
            for p in &dev.ports {
                let status_col = if p.status == "free" {
                    format!("free  (uuid:{})", p.uuid)
                } else {
                    p.status.clone()
                };
                out.push_str(&format!(
                    "  {:<6} {:<30} {:<8} {}\n",
                    p.iname, p.name, p.port_type, status_col
                ));
            }
            total_ports += dev.ports.len();
            used_ports += dev_used;
        }

        if devices.is_empty() {
            out.push_str("No hardware devices found in config.\n");
        } else {
            out.push_str(&format!(
                "\nSummary: {} devices, {} ports ({} used, {} free)\n",
                devices.len(),
                total_ports,
                used_ports,
                total_ports - used_ports
            ));
        }

        out
    }
}
