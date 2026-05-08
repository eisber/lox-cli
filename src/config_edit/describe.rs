use super::{
    ConfigEditor, ConfigStats, ConfigWire, ConfigWireEndpoint, DescribeBlockEntry,
    DescribeConnectorEntry, DescribeEntry, DescribeRoomEntry, DeviceBusSummary, RoomCompleteness,
    SceneInfo,
};
use std::collections::HashMap;
use xmltree::Element;

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
