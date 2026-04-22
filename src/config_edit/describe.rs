use super::{
    ConfigEditor, ConfigStats, DescribeEntry, DeviceBusSummary, RoomCompleteness, SceneInfo,
};
use std::collections::HashMap;
use xmltree::Element;

impl ConfigEditor {
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
