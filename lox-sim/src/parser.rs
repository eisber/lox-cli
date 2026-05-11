//! parser module — .Loxone XML -> SimGraph loader

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use xmltree::Element;

use crate::blocks::{create_block, Block, DayTimer, DayTimerEntry};
use crate::graph::SimGraph;
use crate::types::ConnectorDir;

#[derive(Debug, Clone)]
struct ParsedConnector {
    uuid: String,
    key: String,
    dir: ConnectorDir,
    default_value: f64,
    explicit_source_uuid: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedBlock {
    name: String,
    block_type: String,
    room: Option<String>,
    connectors: Vec<ParsedConnector>,
    daytimer_entries: Vec<DayTimerEntry>,
}

/// Parse a `.Loxone` XML file from disk into a [`SimGraph`].
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<SimGraph, String> {
    let data = fs::read(path.as_ref())
        .map_err(|error| format!("failed to read {}: {error}", path.as_ref().display()))?;
    parse_bytes(&data)
}

/// Parse a `.Loxone` XML payload from bytes.
pub fn parse_bytes(data: &[u8]) -> Result<SimGraph, String> {
    let data = data.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(data);
    let root = Element::parse(data).map_err(|error| format!("XML parse error: {error}"))?;
    parse_element(&root)
}

/// Parse from an already-parsed XML element tree.
pub fn parse_element(root: &Element) -> Result<SimGraph, String> {
    let mut parsed_blocks = Vec::new();
    walk_blocks(root, &mut parsed_blocks);

    let mut graph = SimGraph::new();
    let mut uuid_to_connector = HashMap::new();
    let mut shared_uuid_map: HashMap<String, Vec<(ConnectorDir, usize)>> = HashMap::new();
    let mut explicit_wires = Vec::new();

    for parsed in &parsed_blocks {
        // Get the canonical connector order from block_signature.
        // The block's eval() function expects inputs/outputs/params in this order,
        // but XML may list connectors in any order.
        let (sig_inputs, sig_outputs, sig_params) = block_signature(&parsed.block_type);

        let input_keys =
            order_keys_by_signature(&parsed.connectors, ConnectorDir::Input, sig_inputs);
        let output_keys =
            order_keys_by_signature(&parsed.connectors, ConnectorDir::Output, sig_outputs);
        let param_keys =
            order_keys_by_signature(&parsed.connectors, ConnectorDir::Parameter, sig_params);

        let block: Box<dyn Block> = if parsed.block_type == "DayTimer" {
            Box::new(DayTimer::new(parsed.daytimer_entries.clone()))
        } else {
            create_block(&parsed.block_type)
        };

        let input_refs: Vec<&str> = input_keys.iter().map(|s| s.as_str()).collect();
        let output_refs: Vec<&str> = output_keys.iter().map(|s| s.as_str()).collect();
        let param_refs: Vec<&str> = param_keys.iter().map(|s| s.as_str()).collect();

        let block_id = graph.add_block(
            parsed.name.clone(),
            block,
            &input_refs,
            &output_refs,
            &param_refs,
        );

        // Set room info for name disambiguation
        if let Some(ref room) = parsed.room {
            graph.blocks[block_id].room = Some(room.clone());
        }

        for connector in &parsed.connectors {
            let cid = graph
                .find_connector(block_id, &connector.key)
                .ok_or_else(|| {
                    format!(
                        "missing connector '{}' on block '{}'",
                        connector.key, parsed.name
                    )
                })?;
            graph.connectors[cid].default_value = connector.default_value;
            // Track first occurrence for explicit wire resolution
            uuid_to_connector
                .entry(connector.uuid.clone())
                .or_insert(cid);
            // Also track per-block for shared-UUID wiring
            // If this connector has an explicit source, record the wire with THIS cid
            if let Some(source_ref) = &connector.explicit_source_uuid {
                explicit_wires.push((source_ref.clone(), cid));
            }
            shared_uuid_map
                .entry(connector.uuid.clone())
                .or_default()
                .push((connector.dir, cid));
        }
    }

    let mut wired_inputs = HashSet::new();
    // First pass: resolve UUID-based wires (source is a UUID)
    for (source_ref, dest_cid) in &explicit_wires {
        if let Some(&from) = uuid_to_connector.get(source_ref) {
            if wired_inputs.insert(*dest_cid) {
                graph
                    .add_wire(from, *dest_cid)
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    // Second pass: resolve name-based wiring ("Title.Connector" format, FLG="2")
    let mut name_to_output: HashMap<String, usize> = HashMap::new();
    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        for &cid in &info.outputs {
            let key = format!("{}.{}", info.name, graph.connector(cid).key);
            name_to_output.insert(key, cid);
            // Room-qualified: "Name [Room].Connector"
            if let Some(ref room) = info.room {
                let rkey = format!("{} [{}].{}", info.name, room, graph.connector(cid).key);
                name_to_output.insert(rkey, cid);
            }
        }
        if let Some(&cid) = info.outputs.first() {
            name_to_output.entry(info.name.clone()).or_insert(cid);
            if let Some(ref room) = info.room {
                name_to_output
                    .entry(format!("{} [{}]", info.name, room))
                    .or_insert(cid);
            }
        }
    }

    for (source_ref, dest_cid) in &explicit_wires {
        // Already resolved by UUID above?
        if uuid_to_connector.contains_key(source_ref) {
            continue;
        }
        // Try name-based resolution
        if let Some(&from) = name_to_output.get(source_ref) {
            if wired_inputs.insert(*dest_cid) {
                let _ = graph.add_wire(from, *dest_cid);
            }
        }
    }

    for entries in shared_uuid_map.values() {
        let source = entries
            .iter()
            .find(|(dir, _)| *dir == ConnectorDir::Output)
            .map(|(_, cid)| *cid);
        if let Some(source_cid) = source {
            for (_, dest_cid) in entries
                .iter()
                .filter(|(dir, _)| *dir == ConnectorDir::Input)
            {
                if wired_inputs.insert(*dest_cid) {
                    graph
                        .add_wire(source_cid, *dest_cid)
                        .map_err(|error| error.to_string())?;
                }
            }
        }
    }

    Ok(graph)
}

fn walk_blocks(elem: &Element, out: &mut Vec<ParsedBlock>) {
    walk_blocks_with_room(elem, out, None);
}

fn walk_blocks_with_room(elem: &Element, out: &mut Vec<ParsedBlock>, current_room: Option<&str>) {
    let mut room = current_room;

    if elem.name == "C" {
        if let Some(block_type) = elem.attributes.get("Type") {
            // Track room/page context for name disambiguation
            if block_type == "Place" || block_type == "Page" {
                room = elem.attributes.get("Title").map(|s| s.as_str());
            }
            if !is_structural_type(block_type) {
                let name = elem
                    .attributes
                    .get("Title")
                    .filter(|title| !title.is_empty())
                    .cloned()
                    .or_else(|| elem.attributes.get("U").cloned())
                    .unwrap_or_else(|| block_type.clone());
                out.push(ParsedBlock {
                    name,
                    block_type: block_type.clone(),
                    room: room.map(|s| s.to_string()),
                    connectors: parse_connectors(block_type, elem),
                    daytimer_entries: parse_daytimer_entries(elem),
                });
            }
        }
    }

    for child in &elem.children {
        if let Some(child) = child.as_element() {
            walk_blocks_with_room(child, out, room);
        }
    }
}

/// Reorder parsed connector keys to match the block_signature order.
/// Keys present in the signature come first (in signature order),
/// followed by any extra keys not in the signature (in XML order).
fn order_keys_by_signature(
    connectors: &[ParsedConnector],
    dir: ConnectorDir,
    signature: &[&str],
) -> Vec<String> {
    let available: Vec<&str> = connectors
        .iter()
        .filter(|c| c.dir == dir)
        .map(|c| c.key.as_str())
        .collect();

    let mut ordered = Vec::new();
    // First: keys from signature, in signature order
    for &sig_key in signature {
        if available.contains(&sig_key) {
            ordered.push(sig_key.to_string());
        }
    }
    // Then: any extra keys not in signature (preserves XML order)
    for &key in &available {
        if !ordered.iter().any(|k| k == key) {
            ordered.push(key.to_string());
        }
    }
    ordered
}

fn parse_connectors(block_type: &str, elem: &Element) -> Vec<ParsedConnector> {
    let explicit: Vec<ParsedConnector> = elem
        .children
        .iter()
        .filter_map(|child| child.as_element())
        .filter(|child| child.name == "Co")
        .filter_map(|connector| {
            let key = connector.attributes.get("K")?.clone();
            let uuid = connector.attributes.get("U")?.clone();
            let explicit_source_uuid = connector
                .children
                .iter()
                .filter_map(|child| child.as_element())
                .find(|child| child.name == "In")
                .and_then(|input| input.attributes.get("Input").cloned());
            let default_value = connector
                .attributes
                .get("Def")
                .and_then(|value| value.replace(',', ".").parse::<f64>().ok())
                .unwrap_or(0.0);
            let dir = connector_direction(block_type, &key, explicit_source_uuid.is_some());
            Some(ParsedConnector {
                uuid,
                key,
                dir,
                default_value,
                explicit_source_uuid,
            })
        })
        .collect();

    normalize_connectors(block_type, explicit)
}

fn parse_daytimer_entries(elem: &Element) -> Vec<DayTimerEntry> {
    elem.children
        .iter()
        .filter_map(|child| child.as_element())
        .filter(|child| child.name == "Entry")
        .filter_map(|entry| {
            let to_minute = entry
                .attributes
                .get("To")
                .and_then(|value| value.replace(',', ".").parse::<f64>().ok())?;
            let value = entry
                .attributes
                .get("V")
                .and_then(|raw| raw.replace(',', ".").parse::<f64>().ok())
                .unwrap_or(0.0);
            let day_of_week = entry
                .attributes
                .get("Ix")
                .and_then(|value| value.parse::<u32>().ok());
            Some(DayTimerEntry {
                day_of_week,
                to_minute,
                value,
            })
        })
        .collect()
}

fn connector_direction(block_type: &str, key: &str, has_explicit_input: bool) -> ConnectorDir {
    let (inputs, outputs, params) = block_signature(block_type);
    if outputs.contains(&key) {
        ConnectorDir::Output
    } else if params.contains(&key) {
        ConnectorDir::Parameter
    } else if inputs.contains(&key) || has_explicit_input {
        ConnectorDir::Input
    } else if matches!(
        key,
        "Q" | "AQ"
            | "AQm"
            | "AQmt"
            | "Qon"
            | "Qoff"
            | "RisingEdge"
            | "FallingEdge"
            | "Edge"
            | "Q1"
            | "Q2"
    ) {
        ConnectorDir::Output
    } else {
        ConnectorDir::Parameter
    }
}

fn normalize_connectors(block_type: &str, explicit: Vec<ParsedConnector>) -> Vec<ParsedConnector> {
    let (inputs, outputs, params) = block_signature(block_type);
    let mut by_key: HashMap<String, ParsedConnector> = explicit
        .into_iter()
        .map(|connector| (connector.key.clone(), connector))
        .collect();
    let mut normalized = Vec::new();

    for (keys, dir) in [
        (inputs, ConnectorDir::Input),
        (outputs, ConnectorDir::Output),
        (params, ConnectorDir::Parameter),
    ] {
        for key in keys {
            if let Some(mut connector) = by_key.remove(*key) {
                connector.dir = dir;
                normalized.push(connector);
            } else if dir != ConnectorDir::Parameter {
                // Synthesize missing inputs/outputs so blocks always have
                // a consistent connector layout. Parameters are NOT
                // synthesized — blocks use internal defaults (via unwrap_or)
                // when params are absent from the XML.
                normalized.push(ParsedConnector {
                    uuid: format!("__synthetic__{block_type}__{key}"),
                    key: (*key).to_string(),
                    dir,
                    default_value: 0.0,
                    explicit_source_uuid: None,
                });
            }
        }
    }

    normalized.extend(by_key.into_values());
    normalized
}

fn block_signature(
    block_type: &str,
) -> (
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
) {
    match block_type {
        "AcControl" => (
            &[
                "toggle",
                "on",
                "off",
                "inTempTarget",
                "inTempCurr",
                "inMode",
                "inFan",
                "inAirDir",
                "RtD",
                "inWin",
                "inPause",
                "inLoadS",
                "SilentMode",
            ],
            &["status", "mode", "fan", "airDir", "tempTarget", "tempCurr"],
            &["parPause", "parHyst", "parOff", "minTemp", "maxTemp"],
        ),
        "AMemory" => (&["Input", "Trigger", "Reset"], &["AQ"], &[]),
        "Add" | "Add4" => (&["Input1", "Input2", "Input3", "Input4"], &["AQ", "Q"], &[]),
        "AnalogThresholdTrigger" => (
            &["Input"],
            &["Q", "RisingEdge", "FallingEdge"],
            &["On", "Off", "PulseTime"],
        ),
        "AnalogMultiplexer" => (
            &["Input1", "Input2", "Input3", "Input4", "InputDisable"],
            &["AQ"],
            &["Select"],
        ),
        "AnalogMultiplexer2" => (&["Input1", "Input2", "InputDisable"], &["AQ"], &["Select"]),
        "AnalogScaler" => (&["Input"], &["AQ"], &["Src1", "Src2", "Dst1", "Dst2"]),
        "And" => (&["I1", "I2"], &["Q"], &[]),
        "Constant" => (&[], &["Q"], &["Value"]),
        "Counter" => (&["Trigger", "I1"], &["Q", "AQ"], &["EndValue", "Mode"]),
        "AlarmClock" => (
            &[
                "minutes_since_midnight",
                "day_of_week",
                "Deactivate",
                "Acknowledge",
                "Snooze",
                "TgMe",
                "TMe",
            ],
            &["Qat", "QTp", "Qa", "QTa", "QTe", "AQs", "QMe", "QTna"],
            &["AlarmTime", "PrepTime", "SnoozeTime"],
        ),
        "DayTimer" => (
            &["minutes_since_midnight", "day_of_week", "InputTrigger"],
            &["AQ", "AQm", "Qon", "Qoff", "AQmt"],
            &["Manual", "Mode", "PulseTime"],
        ),
        "DewPoint" => (&["Temp", "Humid", "Baro"], &["DewP"], &["Off"]),
        "Div" => (&["Input1", "Input2"], &["AQ", "Q"], &[]),
        "Mod" => (&["Input1", "Input2"], &["AQ", "AQ2"], &[]),
        "EdgeDetection" => (
            &["Input", "I1"],
            &["Edge", "RisingEdge", "FallingEdge"],
            &["PulseTime"],
        ),
        "FlipFlop" | "RSFlipFlop" | "SRFlipFlop" => {
            (&["InputS", "InputR", "InputTrigger"], &["Q"], &[])
        }
        "Fan" | "Fancoil" | "FancoilFreshAir" | "ToiletFan" => {
            (&["Fan"], &["OFan", "OFanS"], &["Fmax"])
        }
        "Ventilation" | "Ventilation2" | "VentInternorm" => (
            &[
                "Fan", "IN_H_I", "IN_A_I", "IN_T_O", "IN_W_C", "IN_P", "IN_S", "IN_SL", "IN_T",
                "IN_E", "Trigger", "Reset",
            ],
            &["OFan", "OFanS", "OUT_V", "AQ"],
            &["Fmax"],
        ),
        "Gain" => (&["I1"], &["Q"], &["Factor"]),
        "Greater" | "GreaterEqual" | "Less" | "LessEqual" => (&["Input1", "Input2"], &["Q"], &[]),
        "LongClick" => (&["InputTrigger", "Reset"], &["Q2"], &["Max"]),
        "Monoflop" => (&["InputTrigger"], &["Q"], &["Time"]),
        "Minmax" => (
            &["Input1", "Input2", "Input3", "Input4"],
            &["AQmin", "AQmax"],
            &[],
        ),
        "Mult" => (&["Input1", "Input2"], &["AQ", "Q"], &[]),
        "Not" => (&["I", "I1"], &["Q"], &[]),
        "OffDelay" => (&["InputTrigger"], &["Q"], &["Time"]),
        "OnDelay" => (&["InputTrigger"], &["Q"], &["Time"]),
        "OnOffDelay" => (&["InputTrigger"], &["Q"], &["TimeOn", "TimeOff"]),
        "OnPulseDelay" => (&["InputTrigger"], &["Q"], &["Delay", "Time"]),
        "Or" => (&["I1", "I2"], &["Q"], &[]),
        "PassThrough" => (&["I1", "Input"], &["Q", "AQ"], &[]),
        "PresenceDetector" => (&["InputTrigger"], &["OutputPresence"], &[]),
        "PulseGen" => (
            &["InputEnable", "InputInvert"],
            &["Q"],
            &["TimeHigh", "TimeLow"],
        ),
        "PushButton" | "PushButton2" | "PushButtonSel" | "PushButton2Sel" => (
            &["InputTrigger", "On"],
            &["Q", "Qoff", "Qon", "AQ"],
            &["Min", "Max"],
        ),
        "Ramp" => (
            &["InputEnable", "InputSelect", "InputStop"],
            &["AQ"],
            &["Rate", "StartValue", "Level1", "Level2"],
        ),
        "RisingEdge" => (&["I1", "Input"], &["Q", "RisingEdge"], &[]),
        "LightController2" => (
            &[
                "I1",
                "Presence",
                "Brightness",
                "Move",
                "Sel1",
                "Sel2",
                "Sel3",
                "Sel4",
                "Sel5",
                "Sel6",
                "Sel7",
                "Sel8",
                "Reset",
                "InputDisable",
            ],
            &["AQ1", "AQ2", "Scene", "PresenceActive"],
            &["FadingTime", "SceneMixTime"],
        ),
        "LightController" | "LightControllerH" => (
            &["I1", "Presence", "Brightness", "Reset", "InputDisable"],
            &["AQ1", "Scene", "PresenceActive"],
            &["FadingTime"],
        ),
        "JalousieUpDown2" => (
            &[
                "InputTriggerUp",
                "InputTriggerDown",
                "InputPos",
                "InputDisable",
                "Shade",
            ],
            &["Pos", "Dir", "Moving"],
            &["TimeEnd"],
        ),
        "AutoJalousie" => (
            &["SunAzimuth", "SunAltitude", "Brightness", "InputDisable"],
            &[
                "OutputAutoShade",
                "OutputUp",
                "OutputDown",
                "OutputPos",
                "OutputLPos",
                "OutputSafety",
                "OutputLock",
                "OutputCombined",
                "TargetPos",
            ],
            &[
                "Dir",
                "DirTol",
                "DirTol2",
                "Width",
                "Space",
                "AutMode",
                "AutoShadeTime",
                "SRoff",
                "SSoff",
                "Type",
                "SO",
                "AutoShadeEnd",
                "MinPulse",
                "TimeEnd",
                "TimeEndDown",
                "TimeBlock",
                "DblClk",
                "Back",
                "TurnOffset",
                "Deadtime",
                "MinMove",
                "Rdd",
            ],
        ),
        "Jalousiemotor" | "EIBJalousie" | "Pergola" | "RoofWindow" | "ShadeRoof" | "Skylight" => (
            &[
                "InputTriggerUp",
                "InputTriggerDown",
                "InputPos",
                "InputDisable",
            ],
            &["Pos", "Dir", "Moving"],
            &["TimeEnd"],
        ),
        "Heatcurve" => (
            &["Outer", "Input"],
            &["AQ"],
            &["Translate", "Transconductance"],
        ),
        "Heatmixer" => (
            &["Input", "Temp", "Stop", "InputDisable"],
            &["AQ", "Q1", "Q2", "Qe"],
            &[
                "Time", "Sample", "Kp", "Ki", "StopMode", "Mode", "Min", "Max", "Inv", "TimeMove",
            ],
        ),
        "Heatmixer2" => (
            &["Outer", "Boost", "Stop", "InBuffer"],
            &[
                "AQt",
                "TxQr",
                "AQf",
                "AQb",
                "Qp",
                "AQr",
                "AQl",
                "AQi",
                "Qe",
                "OutputAPI",
            ],
            &[
                "Min",
                "Max",
                "BufferInc",
                "Transconductance",
                "Translate",
                "Treshold",
                "Gain",
                "PrevInc",
                "TimeActive",
            ],
        ),
        "HeatIRoomController2" | "ClimateControllerUS" | "HVACController" => (
            &[
                "Temp",
                "Input",
                "Reset",
                "InputDisable",
                "AMode",
                "CoolingSetpoint",
                "Window",
                "Comfort",
                "Save",
                "Save2",
                "Move",
                "DisMv",
                "TempO",
                "InCo2",
                "InHumid",
                "inFan",
                "inAirDir",
            ],
            &["AQh", "AQc", "AQhc", "AQh1", "AQh2"],
            &[
                "TComfort",
                "TComfortC",
                "TDiff",
                "THCelvin",
                "TCCelvin",
                "TSaveL",
                "TSaveU",
                "TDeepSleep",
            ],
        ),
        "StairwayLS" => (
            &["InputTrigger", "On"],
            &["Q"],
            &["TimeHigh", "TimeWarn", "WarnTime"],
        ),
        "RetOnDelay" => (&["InputTrigger", "InputReset"], &["Q"], &["Time"]),
        "Sub" => (&["Input1", "Input2"], &["AQ", "Q"], &[]),
        "SmokeAlarm" => (
            &[
                "Confirm",
                "Mute",
                "InputAlarm",
                "InputAlarmW",
                "InputAlarmS",
                "InputTemp",
            ],
            &[
                "OutAlarm1",
                "OutAlarm2",
                "OutSilent",
                "OutHorn",
                "OutNumAlarms",
                "OutAlarmTest",
            ],
            &[
                "AlarmDelay",
                "MaxTemp",
                "MaxDuration",
                "Autoconfirm",
                "Servicemode",
            ],
        ),
        "State" => (&["I1", "I2", "I3", "I4", "I5"], &["AQ"], &["NumStates"]),
        "StateV" => (&["I"], &["AQ"], &[]),
        "SysVar" => (&["I1"], &["AQ"], &[]),
        "VirtualIn" | "VirtualOut" => (&["I1", "Input"], &["Q", "AQ", "Qm"], &[]),
        // Tree / Air hardware sensors — source blocks with synthetic input for sim injection
        "TreeSensor" | "LoxAIRsensor" => (&["I1"], &["Q", "Qe"], &[]),
        "TreeAsensor" | "LoxAIRAsensor" => (&["I1"], &["AQ", "Q"], &[]),
        "Xor" => (&["I1", "I2"], &["Q"], &[]),
        // NFC Code Touch — keypad with code validation
        "NfcCodeTouch" => (
            &["Disable", "Lr", "Lg", "Lb", "Lw"],
            &[
                "Q1", "Q2", "TQ", "TQU", "TQo", "TQt", "Qd", "Qa", "Qn", "Be",
            ],
            &[],
        ),
        // Wallbox Energy Manager
        "WBEM" => (
            &["Pmax", "Peco", "Off"],
            &["Cp", "Ap", "OutputAPI", "allow"],
            &["PrEco", "PrPrio", "PrHour"],
        ),
        // Wallbox charger
        "Wallbox" | "WallboxAir" => (
            &[
                "allow",
                "prio",
                "connected",
                "power",
                "energy",
                "active",
                "ocppAuth",
                "sessStop",
                "rebootDev",
                "setmode1",
                "setmode2",
                "setmode3",
                "setmode4",
                "setmode5",
            ],
            &[
                "valmode1", "valmode2", "valmode3", "valmode4", "valmode5", "state", "charging",
                "plugged",
            ],
            &[],
        ),
        // Central override controllers
        "CentralLight" => (
            &["AllOn", "AllOff", "AllScene", "InputDisable"],
            &["AQ", "AQScene", "Active"],
            &[],
        ),
        "CentralShade" => (
            &["AllUp", "AllDown", "AllShade", "AllPos", "InputDisable"],
            &["AQ", "QAnyMoving", "Active"],
            &[],
        ),
        // Window/door monitoring
        "WindowsMonitor" => (
            &["W", "Wt", "Wl", "HI1", "HI2", "HI3", "Remanence"],
            &["AQo", "AQt", "AQc", "AQof", "AQl", "AQu", "TQ", "TQo"],
            &[],
        ),
        "JoinWindowSensor" => (&["W", "Wt", "Wl"], &["AQ", "Q"], &[]),
        "Doorcontroller" => (&["Trigger", "Lock", "Unlock"], &["Q", "QLock"], &[]),
        // Outdoor / garden
        "Irrigation" => (&["InputTrigger", "Rain", "Moisture"], &["Q"], &["Duration"]),
        // I/O hardware blocks
        "EIBactor" => (&["I1", "Input"], &["Q", "AQ"], &[]),
        "EIBsensor" => (&["I1", "Input"], &["Q", "AQ"], &[]),
        "MeterAbsUni" => (&["I1"], &["AQ"], &[]),
        "VoltageIn" => (&["I1"], &["AQ"], &[]),
        "DigitalIn" => (&["I1"], &["Q"], &[]),
        // Structural/cosmetic blocks
        "VirtualInCaption" | "WeatherServer" | "LightscenesC" | "LightsceneC" | "TreeDevice"
        | "LoxAIRDevice" | "NetworkDevice" | "LoxCaption" => (&[], &[], &[]),
        _ => (&[], &[], &[]),
    }
}

fn is_structural_type(block_type: &str) -> bool {
    matches!(
        block_type,
        "Category"
            | "Document"
            | "Page"
            | "Place"
            | "Program"
            // Device containers — hold sensors/actors but are not blocks
            | "LoxTree"
            | "LoxCaption"
            | "TreeDevice"
            | "LoxAIR"
            | "LoxAIRDevice"
            | "Online"
            | "SysTemp"
            | "InputCaption"
            | "KeycodeCaption"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_xml(xml: &str) -> SimGraph {
        parse_bytes(xml.as_bytes()).expect("parse failed")
    }

    #[test]
    fn parse_single_block() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="And" U="0001" Title="Gate">
    <Co K="I1" U="0001-i1"/>
    <Co K="I2" U="0001-i2"/>
    <Co K="Q" U="0001-q"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        assert_eq!(graph.block_count(), 1);
        let block = graph.block_info(0);
        assert_eq!(block.name, "Gate");
        assert_eq!(graph.block_impls[0].block_type(), "And");
    }

    #[test]
    fn parse_explicit_wiring() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="VirtualIn" U="src" Title="Switch">
    <Co K="Q" U="src-q"/>
  </C>
  <C Type="And" U="gate" Title="Gate">
    <Co K="I1" U="gate-i1"><In Input="src-q"/></Co>
    <Co K="I2" U="gate-i2" Def="1"/>
    <Co K="Q" U="gate-q"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        assert_eq!(graph.wires().len(), 1);
        let gate = graph.find_block_by_name("Gate").unwrap();
        let input = graph.find_connector(gate, "I1").unwrap();
        assert!(graph.input_source_of(input).is_some());
    }

    #[test]
    fn parse_uuid_sharing_wires_output_to_input() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="VirtualIn" U="src" Title="Source">
    <Co K="Q" U="shared"/>
  </C>
  <C Type="And" U="dest" Title="Dest">
    <Co K="I1" U="shared"/>
    <Co K="I2" U="dest-i2" Def="1"/>
    <Co K="Q" U="dest-q"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        assert_eq!(graph.wires().len(), 1);
    }

    #[test]
    fn parse_daytimer_entries_into_schedule_block() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="DayTimer" U="dt" Title="Schedule">
    <Co K="AQ" U="dt-aq"/>
    <Co K="AQm" U="dt-aqm"/>
    <Co K="Qon" U="dt-qon"/>
    <Co K="Qoff" U="dt-qoff"/>
    <Co K="AQmt" U="dt-aqmt"/>
    <Entry To="480" V="0"/>
    <Entry To="1020" V="1"/>
    <Entry To="1440" V="0"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        let block = graph.find_block_by_name("Schedule").unwrap();
        assert_eq!(graph.block_info(block).outputs.len(), 5);
        assert_eq!(graph.block_impls[block].block_type(), "DayTimer");
    }

    #[test]
    fn sysvar_gets_synthetic_input() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="SysVar" U="temp" Title="Temperature">
    <Co K="AQ" U="temp-aq"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        let block = graph.find_block_by_name("Temperature").unwrap();
        assert!(graph.find_connector(block, "I1").is_some());
        assert!(graph.find_connector(block, "AQ").is_some());
    }

    #[test]
    fn presence_detector_wires_output_presence() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="PresenceDetector" U="sensor" Title="Sensor">
    <Co K="InputTrigger" U="sensor-in"/>
    <Co K="OutputPresence" U="presence-out"/>
  </C>
  <C Type="LightController2" U="light" Title="Light">
    <Co K="Presence" U="light-presence"><In Input="presence-out"/></Co>
    <Co K="AQ1" U="light-out"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        assert_eq!(graph.wires().len(), 1);
    }

    #[test]
    fn light_controller2_exposes_scene_outputs() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="LightController2" U="light" Title="Light">
    <Co K="AQ1" U="light-aq1"/>
    <Co K="Scene" U="light-scene"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        let block = graph.find_block_by_name("Light").unwrap();
        assert!(graph.find_connector(block, "AQ1").is_some());
        assert!(graph.find_connector(block, "Scene").is_some());
        assert!(graph.find_connector(block, "Presence").is_some());
    }

    #[test]
    fn heat_room_controller_gets_core_connectors() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="HeatIRoomController2" U="heat" Title="Heat">
    <Co K="AQh" U="heat-aqh"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        let block = graph.find_block_by_name("Heat").unwrap();
        assert!(graph.find_connector(block, "Temp").is_some());
        assert!(graph.find_connector(block, "AQh").is_some());
    }

    #[test]
    fn unknown_types_fall_back_to_passthrough() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList>
  <C Type="FancyNewBlock" U="mystery" Title="Unknown">
    <Co K="I1" U="mystery-i1"/>
    <Co K="Q" U="mystery-q"/>
  </C>
</ControlList>"#;
        let graph = parse_xml(xml);
        assert_eq!(graph.block_impls[0].block_type(), "PassThrough");
    }
}
