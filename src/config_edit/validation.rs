use super::{ConfigEditor, ConnectorMap};
use xmltree::Element;

impl ConfigEditor {
    /// Check automation blocks for completeness issues an agent should fix.
    /// Returns a list of findings (✓ ok, ⚠ warning, ✗ error).
    #[allow(clippy::too_many_lines)]
    pub fn check_blocks(&self, selector: Option<&str>) -> Vec<String> {
        use std::collections::{HashMap, HashSet};

        let mut results = Vec::new();

        // Block types that have important parameters: (type, param_name, description)
        let param_checks: Vec<(&str, &str, &str)> = vec![
            ("GreaterEqual", "Input2", "threshold value"),
            ("Less", "Input2", "threshold value"),
            ("Mult", "Input2", "multiplier value"),
            ("Add", "Input2", "addend value"),
            ("Sub", "Input2", "subtrahend value"),
            ("Div", "Input2", "divisor value"),
            ("StairwayLS", "TimeHigh", "duration in seconds"),
            ("OnPulseDelay", "Time", "delay in seconds"),
            ("OffDelay", "Time", "delay in seconds"),
            ("Monoflop", "Time", "pulse duration in seconds"),
            ("AnalogThresholdTrigger", "On", "threshold on value"),
        ];

        let logic_types: HashSet<&str> = [
            "And",
            "Or",
            "Not",
            "Xor",
            "GreaterEqual",
            "Less",
            "Mult",
            "Add",
            "Sub",
            "Div",
            "StairwayLS",
            "OnPulseDelay",
            "OffDelay",
            "Monoflop",
            "PulseGen",
            "AnalogThresholdTrigger",
        ]
        .iter()
        .copied()
        .collect();

        let input_keys: HashSet<&str> = [
            "I",
            "I1",
            "I2",
            "Input1",
            "Input",
            "InputEnable",
            "InputTrigger",
            "Temp",
            "toggle",
            "Presence",
        ]
        .iter()
        .copied()
        .collect();

        let output_keys: HashSet<&str> = ["Q", "AQ", "Q1"].iter().copied().collect();

        let infra_types: HashSet<&str> = [
            "",
            "Document",
            "Page",
            "Place",
            "Category",
            "Program",
            "VirtualInCaption",
            "LightscenesC",
            "LightsceneC",
            "Lightscene",
            "Co",
            "IoData",
            "SET",
            "WeatherServer",
            "SysVar",
        ]
        .iter()
        .copied()
        .collect();

        // Build wiring maps
        let mut wired_inputs: HashSet<String> = HashSet::new();
        let mut wired_outputs: HashSet<String> = HashSet::new();
        let mut uuid_block_count: HashMap<String, usize> = HashMap::new();

        fn scan_wiring(
            elem: &Element,
            wired_in: &mut HashSet<String>,
            wired_out: &mut HashSet<String>,
            uuid_count: &mut HashMap<String, usize>,
        ) {
            for child in &elem.children {
                let Some(c) = child.as_element() else {
                    continue;
                };
                let is_co =
                    c.name == "Co" || c.attributes.get("Type").map(|s| s.as_str()) == Some("Co");
                if is_co && c.attributes.contains_key("U") {
                    let u = c.attributes.get("U").unwrap();
                    *uuid_count.entry(u.clone()).or_insert(0) += 1;
                    for inp_child in &c.children {
                        let Some(inp) = inp_child.as_element() else {
                            continue;
                        };
                        if inp.name == "In" {
                            wired_in.insert(u.clone());
                            if let Some(src) = inp.attributes.get("Input") {
                                wired_out.insert(src.clone());
                            }
                        }
                    }
                }
                scan_wiring(c, wired_in, wired_out, uuid_count);
            }
        }
        scan_wiring(
            &self.root,
            &mut wired_inputs,
            &mut wired_outputs,
            &mut uuid_block_count,
        );

        // Resolve title-based wiring ("BlockTitle.ConnKey") to UUIDs so output
        // connectivity checks work for both UUID and title-based references.
        let mut title_conn_to_uuid: HashMap<String, String> = HashMap::new();
        for elem in self.iter_elements(&self.root) {
            if let Some(title) = elem.attributes.get("Title") {
                for child in &elem.children {
                    if let Some(co) = child.as_element()
                        && co.name == "Co"
                        && let (Some(k), Some(u)) = (co.attributes.get("K"), co.attributes.get("U"))
                    {
                        title_conn_to_uuid.insert(format!("{}.{}", title, k), u.clone());
                    }
                }
            }
        }
        for title_ref in wired_outputs.clone() {
            if title_ref.contains('.')
                && !title_ref.contains('-')
                && let Some(uuid) = title_conn_to_uuid.get(&title_ref)
            {
                wired_outputs.insert(uuid.clone());
            }
        }

        // Walk all blocks
        struct CheckCtx<'a> {
            param_checks: &'a [(&'a str, &'a str, &'a str)],
            logic_types: &'a HashSet<&'a str>,
            input_keys: &'a HashSet<&'a str>,
            output_keys: &'a HashSet<&'a str>,
            infra_types: &'a HashSet<&'a str>,
            wired_inputs: &'a HashSet<String>,
            wired_outputs: &'a HashSet<String>,
            uuid_block_count: &'a HashMap<String, usize>,
            connector_map: &'a ConnectorMap,
        }

        fn walk_blocks(
            elem: &Element,
            selector: Option<&str>,
            ctx: &CheckCtx<'_>,
            results: &mut Vec<String>,
            on_page: bool,
        ) {
            let etype = elem
                .attributes
                .get("Type")
                .map(|s| s.as_str())
                .unwrap_or("");
            let title = elem
                .attributes
                .get("Title")
                .map(|s| s.as_str())
                .unwrap_or("");
            let uuid = elem.attributes.get("U").map(|s| s.as_str()).unwrap_or("");

            if !ctx.infra_types.contains(etype) && !uuid.is_empty() {
                let matches_sel = match selector {
                    Some(sel) => title.contains(sel) || etype == sel || uuid == sel,
                    None => true,
                };

                if matches_sel {
                    // Check required parameters
                    for &(ptype, param_name, description) in ctx.param_checks {
                        if etype != ptype {
                            continue;
                        }
                        let has_value = elem.children.iter().any(|child| {
                            let Some(c) = child.as_element() else {
                                return false;
                            };
                            let is_co = c.name == "Co"
                                || c.attributes.get("Type").map(|s| s.as_str()) == Some("Co");
                            is_co
                                && c.attributes.get("K").map(|s| s.as_str()) == Some(param_name)
                                && c.attributes.get("Def").is_some_and(|s| !s.is_empty())
                        });
                        if !has_value {
                            results.push(format!(
                                "✗ {etype} '{title}': {param_name} not set — needs {description}"
                            ));
                        }
                    }

                    // Check wiring for all automation blocks (not just logic types)
                    {
                        let mut any_input = false;
                        let mut any_output = false;
                        let mut unwired_inputs = Vec::new();

                        // Get connector types from the connector map for this block type
                        let cmap_types = ctx
                            .connector_map
                            .get(etype)
                            .map(|(_, _, t)| t.clone())
                            .unwrap_or_default();

                        for child in &elem.children {
                            let Some(c) = child.as_element() else {
                                continue;
                            };
                            let is_co = c.name == "Co"
                                || c.attributes.get("Type").map(|s| s.as_str()) == Some("Co");
                            if !is_co {
                                continue;
                            }
                            let k = c.attributes.get("K").map(|s| s.as_str()).unwrap_or("");
                            let u = c.attributes.get("U").map(|s| s.as_str()).unwrap_or("");

                            let is_wired = ctx.wired_inputs.contains(u)
                                || ctx.wired_outputs.contains(u)
                                || *ctx.uuid_block_count.get(u).unwrap_or(&0) >= 2;

                            // Determine direction: prefer connector map, fall back to static sets
                            let direction = cmap_types.get(k).map(|s| s.as_str());
                            let is_input = direction == Some("I")
                                || (direction.is_none() && ctx.input_keys.contains(k));
                            let is_output = direction == Some("O")
                                || (direction.is_none() && ctx.output_keys.contains(k));

                            if is_input {
                                let has_default =
                                    c.attributes.get("Def").is_some_and(|s| !s.is_empty());
                                if is_wired || has_default {
                                    any_input = true;
                                } else {
                                    unwired_inputs.push(k.to_string());
                                }
                            }
                            if is_output && is_wired {
                                any_output = true;
                            }
                        }

                        // Only report if block actually has input connectors
                        let has_input_connectors = any_input || !unwired_inputs.is_empty();
                        if has_input_connectors {
                            if !any_input {
                                if any_output {
                                    // Block has outputs wired but no inputs — circuit is incomplete
                                    results.push(format!(
                                        "✗ {etype} '{title}': no inputs wired — block has no signal source"
                                    ));
                                } else {
                                    // Standalone block, no wiring at all — just a warning
                                    results.push(format!(
                                        "⚠ {etype} '{title}': no inputs wired — may need signal source"
                                    ));
                                }
                            } else if !unwired_inputs.is_empty() {
                                // Partially wired — some inputs missing
                                results.push(format!(
                                    "✗ {etype} '{title}': unwired inputs: {} — wire signal sources to complete the circuit",
                                    unwired_inputs.join(", ")
                                ));
                            }
                        }
                        if ctx.logic_types.contains(etype) && !any_output {
                            results.push(format!(
                                "⚠ {etype} '{title}': output not connected — block result goes nowhere"
                            ));
                        }
                        if has_input_connectors && any_input && unwired_inputs.is_empty() {
                            results.push(format!("✓ {etype} '{title}': inputs and outputs wired"));
                        }
                    }

                    // ── Sub-element completeness checks ──

                    // Canvas layout / connector-count check. Blocks placed on a
                    // page without coordinates (or with a stale Nio that doesn't
                    // match the connector count) get "repaired" by Loxone Config
                    // on first save — it assigns coords, recomputes Nio, and
                    // regenerates connectors with fresh UUIDs, silently dropping
                    // every attached wire. Warn before that footgun fires.
                    if on_page {
                        let co_count = elem
                            .children
                            .iter()
                            .filter(|child| {
                                child.as_element().is_some_and(|c| {
                                    c.name == "Co"
                                        || c.attributes.get("Type").map(|s| s.as_str())
                                            == Some("Co")
                                })
                            })
                            .count();
                        if co_count > 0 {
                            let missing_coords = ["Px", "Py", "Px2", "Py2"]
                                .iter()
                                .any(|a| !elem.attributes.contains_key(*a));
                            if missing_coords {
                                results.push(format!(
                                    "⚠ {etype} '{title}': missing canvas layout (Px/Py/Px2/Py2) — Loxone Config will repair this block on first save and may drop wires; run `lox config layout`"
                                ));
                            }
                            if let Some(nio) = elem
                                .attributes
                                .get("Nio")
                                .and_then(|s| s.parse::<usize>().ok())
                                && nio != co_count
                            {
                                results.push(format!(
                                    "⚠ {etype} '{title}': Nio={nio} but {co_count} connectors — mismatch may cause Loxone Config to regenerate connectors and drop wires"
                                ));
                            }
                        }
                    }

                    // LightController2 needs LightscenesC with scenes
                    if etype == "LightController2" {
                        let has_scenes = elem.children.iter().any(|child| {
                            child.as_element().is_some_and(|c| {
                                (c.name == "C"
                                    && c.attributes.get("Type").map(|s| s.as_str())
                                        == Some("LightscenesC"))
                                    || c.name == "LightscenesC"
                            })
                        });
                        if !has_scenes {
                            results.push(format!(
                                "✗ {etype} '{title}': missing LightscenesC — no light scenes defined (need at least Viel Licht + Aus)"
                            ));
                        }
                    }

                    // IoData checks for controls that should be visible
                    let needs_iodata = matches!(
                        etype,
                        "LightController2"
                            | "JalousieUpDown2"
                            | "EIBJalousie"
                            | "HeatIRoomController2"
                            | "AcControl"
                            | "VirtualIn"
                            | "VirtualOut"
                            | "VirtualState"
                            | "PresenceDetector"
                    );
                    if needs_iodata {
                        let has_iodata = elem
                            .children
                            .iter()
                            .any(|child| child.as_element().is_some_and(|c| c.name == "IoData"));
                        if !has_iodata {
                            results.push(format!(
                                "✗ {etype} '{title}': missing IoData — block won't be assigned to a room"
                            ));
                        } else {
                            let has_room = elem.children.iter().any(|child| {
                                child.as_element().is_some_and(|c| {
                                    c.name == "IoData"
                                        && c.attributes.get("Pr").is_some_and(|s| !s.is_empty())
                                })
                            });
                            if !has_room
                                && matches!(
                                    etype,
                                    "LightController2"
                                        | "JalousieUpDown2"
                                        | "EIBJalousie"
                                        | "HeatIRoomController2"
                                        | "AcControl"
                                )
                            {
                                results.push(format!(
                                    "⚠ {etype} '{title}': IoData missing Pr (room) reference"
                                ));
                            }
                        }
                    }

                    // DayTimer needs Entry elements for schedule
                    if etype == "DayTimer" {
                        let entry_count = elem
                            .children
                            .iter()
                            .filter(|child| child.as_element().is_some_and(|c| c.name == "Entry"))
                            .count();
                        if entry_count == 0 {
                            results.push(format!(
                                "⚠ {etype} '{title}': no Entry elements — schedule is empty (configure in Loxone app)"
                            ));
                        } else {
                            results.push(format!(
                                "✓ {etype} '{title}': {entry_count} schedule entries defined"
                            ));
                        }
                    }

                    // SequenceController program validation
                    if etype == "SequenceController" {
                        let program_text = elem.children.iter().find_map(|child| {
                            let f = child.as_element()?;
                            if f.name == "Field"
                                && f.attributes
                                    .get("Name")
                                    .map(|n| n == "Configuration")
                                    .unwrap_or(false)
                            {
                                Some(
                                    f.children
                                        .iter()
                                        .filter_map(|c| {
                                            if let xmltree::XMLNode::Text(t) = c {
                                                Some(t.as_str())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                        .join(""),
                                )
                            } else {
                                None
                            }
                        });

                        match program_text {
                            None => {
                                results.push(format!(
                                    "⚠ {etype} '{title}': has no program — it will do nothing"
                                ));
                            }
                            Some(ref text) if text.trim().is_empty() => {
                                results.push(format!(
                                    "⚠ {etype} '{title}': has no program — it will do nothing"
                                ));
                            }
                            Some(ref text) => {
                                let errors = lox_sim::blocks::sequence::validate_program(text);
                                if errors.is_empty() {
                                    let lines =
                                        lox_sim::blocks::sequence::count_program_lines(text);
                                    let seqs = lox_sim::blocks::sequence::count_sequences(text);
                                    results.push(format!(
                                        "✓ {etype} '{title}': program OK ({} line{}, {} sequence{})",
                                        lines,
                                        if lines == 1 { "" } else { "s" },
                                        seqs,
                                        if seqs == 1 { "" } else { "s" },
                                    ));
                                } else {
                                    for err in &errors {
                                        results.push(format!("✗ {etype} '{title}': {}", err));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for child in &elem.children {
                if let Some(c) = child.as_element() {
                    let child_on_page = on_page || etype == "Page";
                    walk_blocks(c, selector, ctx, results, child_on_page);
                }
            }
        }

        let cmap = Self::connector_map();
        let ctx = CheckCtx {
            param_checks: &param_checks,
            logic_types: &logic_types,
            input_keys: &input_keys,
            output_keys: &output_keys,
            infra_types: &infra_types,
            wired_inputs: &wired_inputs,
            wired_outputs: &wired_outputs,
            uuid_block_count: &uuid_block_count,
            connector_map: &cmap,
        };

        walk_blocks(&self.root, selector, &ctx, &mut results, false);

        if results.is_empty() {
            results.push("✓ No automation blocks found (or none match selector)".to_string());
        }

        results
    }

    /// Validate config for common issues.
    pub fn validate_config(&self) -> Vec<String> {
        let mut results = Vec::new();

        // Collect all Place UUIDs
        let mut place_uuids = std::collections::HashSet::new();
        self.collect_typed_uuids(&self.root, "Place", &mut place_uuids);

        // Collect all Category UUIDs
        let mut category_uuids = std::collections::HashSet::new();
        self.collect_typed_uuids(&self.root, "Category", &mut category_uuids);

        // Check IoData references
        let mut bad_rooms = Vec::new();
        let mut bad_cats = Vec::new();
        let mut unconnected = Vec::new();
        self.validate_recursive(
            &self.root,
            &place_uuids,
            &category_uuids,
            &mut bad_rooms,
            &mut bad_cats,
            &mut unconnected,
        );

        // Room references
        if bad_rooms.is_empty() {
            results.push("✓ All IoData room references (Pr=) point to existing rooms".to_string());
        } else {
            for r in &bad_rooms {
                results.push(format!("✗ IoData Pr='{}' references non-existent room", r));
            }
        }

        // Category references
        if bad_cats.is_empty() {
            results.push(
                "✓ All IoData category references (Cr=) point to existing categories".to_string(),
            );
        } else {
            for c in &bad_cats {
                results.push(format!(
                    "✗ IoData Cr='{}' references non-existent category",
                    c
                ));
            }
        }

        // Unconnected connectors
        if unconnected.is_empty() {
            results.push("✓ All connectors are wired".to_string());
        } else {
            results.push(format!(
                "⚠ {} connectors are unconnected",
                unconnected.len()
            ));
        }

        // MQTT broker check
        let mqtt_elements = self.find_elements("gid:Mqtt");
        if !mqtt_elements.is_empty() {
            let mqtt_elem = self.get_element(mqtt_elements[0].as_slice());
            let mut has_broker = false;
            for child in &mqtt_elem.children {
                if let Some(set) = child.as_element()
                    && set.name == "SET"
                {
                    for prop in &set.children {
                        if let Some(p) = prop.as_element()
                            && p.name == "mqtt_broker_address"
                        {
                            let v = p.attributes.get("v").cloned().unwrap_or_default();
                            has_broker = !v.is_empty();
                        }
                    }
                }
            }
            if has_broker {
                results.push("✓ MQTT broker address is configured".to_string());
            } else {
                results.push("✗ MQTT plugin found but broker address is not set".to_string());
            }
        }

        // ── Loxone-specific validation rules ──────────────────────────────

        // Extract Miniserver serial from existing UUIDs
        let serial = self.find_miniserver_serial().unwrap_or_default();

        // I/O types that require IName
        let iname_types: std::collections::HashSet<&str> = [
            "VirtualIn",
            "VirtualOut",
            "VirtualState",
            "DigitalIn",
            "Actor",
            "VoltageIn",
            "SysVar",
            "WeatherData",
            "Online",
            "TreeSensor",
            "TreeActor",
            "TreeAsensor",
            "TreeAactor",
            "LoxAIRsensor",
            "LoxAIRactor",
            "LoxAIRAsensor",
            "LoxAIRAactor",
        ]
        .iter()
        .copied()
        .collect();

        let mut uuid_warnings = Vec::new();
        let mut iname_errors = Vec::new();
        let mut placement_errors = Vec::new();

        fn check_elements(
            elem: &Element,
            serial: &str,
            iname_types: &std::collections::HashSet<&str>,
            uuid_warnings: &mut Vec<String>,
            iname_errors: &mut Vec<String>,
            placement_errors: &mut Vec<String>,
            parent_type: &str,
        ) {
            if elem.name == "C" {
                let elem_type = elem.attributes.get("Type").cloned().unwrap_or_default();
                let title = elem.attributes.get("Title").cloned().unwrap_or_default();
                let uuid = elem.attributes.get("U").cloned().unwrap_or_default();

                // Check UUID format (must contain serial suffix). Skip known
                // system/mode/registered-device short-form UUIDs — these
                // legitimately lack the serial in every Gen-2 export.
                if !uuid.is_empty()
                    && !serial.is_empty()
                    && !uuid.contains(serial)
                    && !is_system_short_uuid(&uuid)
                {
                    uuid_warnings.push(format!(
                        "{} '{}': UUID '{}' missing serial suffix '{}'",
                        elem_type,
                        title,
                        &uuid[..std::cmp::min(20, uuid.len())],
                        serial
                    ));
                }

                // Check IName for types that need it
                if iname_types.contains(elem_type.as_str())
                    && !elem.attributes.contains_key("IName")
                {
                    iname_errors.push(format!(
                        "{} '{}': missing required IName attribute",
                        elem_type, title
                    ));
                }

                // Check connector UUIDs
                for child in &elem.children {
                    if let Some(co) = child.as_element()
                        && co.name == "Co"
                    {
                        let co_uuid = co.attributes.get("U").cloned().unwrap_or_default();
                        if !co_uuid.is_empty()
                            && !serial.is_empty()
                            && !co_uuid.contains(serial)
                            && !is_system_short_uuid(&co_uuid)
                        {
                            uuid_warnings.push(format!(
                                "{} '{}' connector {}: UUID missing serial suffix",
                                elem_type,
                                title,
                                co.attributes.get("K").cloned().unwrap_or_default()
                            ));
                        }
                    }
                }

                // Check placement: logic/math blocks should be under Page
                let logic_types: std::collections::HashSet<&str> = [
                    "And",
                    "Or",
                    "Not",
                    "Xor",
                    "Add",
                    "Add4",
                    "Sub",
                    "Mult",
                    "Div",
                    "Modulo",
                    "AnalogComparator",
                    "Memory",
                    "FlipFlop",
                    "Counter",
                    "Monoflop",
                ]
                .iter()
                .copied()
                .collect();
                if logic_types.contains(elem_type.as_str()) && parent_type != "Page" {
                    placement_errors.push(format!(
                        "{} '{}': should be under a Page element (found under {})",
                        elem_type, title, parent_type
                    ));
                }
            }

            let my_type = elem
                .attributes
                .get("Type")
                .cloned()
                .unwrap_or_else(|| elem.name.clone());
            for child in &elem.children {
                if let Some(child_elem) = child.as_element() {
                    check_elements(
                        child_elem,
                        serial,
                        iname_types,
                        uuid_warnings,
                        iname_errors,
                        placement_errors,
                        &my_type,
                    );
                }
            }
        }

        check_elements(
            &self.root,
            &serial,
            &iname_types,
            &mut uuid_warnings,
            &mut iname_errors,
            &mut placement_errors,
            "root",
        );

        if uuid_warnings.is_empty() {
            results.push(format!(
                "✓ All UUIDs use Miniserver serial suffix ({}...)",
                &serial[..std::cmp::min(6, serial.len())]
            ));
        } else {
            for e in &uuid_warnings[..std::cmp::min(5, uuid_warnings.len())] {
                results.push(format!("⚠ {}", e));
            }
            if uuid_warnings.len() > 5 {
                results.push(format!(
                    "  ...and {} more UUID errors",
                    uuid_warnings.len() - 5
                ));
            }
        }

        if iname_errors.is_empty() {
            results.push("✓ All I/O elements have IName".to_string());
        } else {
            for e in &iname_errors {
                results.push(format!("⚠ {}", e));
            }
        }

        if placement_errors.is_empty() {
            results.push("✓ Logic/math blocks correctly placed under Page".to_string());
        } else {
            for e in &placement_errors {
                results.push(format!("⚠ {}", e));
            }
        }

        // Check wiring direction: only wire TO Input/Parameter connectors
        let cmap = Self::connector_map();
        let mut wiring_warnings = 0;
        fn check_wiring(
            elem: &Element,
            cmap: &ConnectorMap,
            results: &mut Vec<String>,
            warnings: &mut usize,
        ) {
            let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
            let title = elem.attributes.get("Title").cloned().unwrap_or_default();
            if let Some((_, _, types)) = cmap.get(&block_type) {
                for child in &elem.children {
                    if let Some(co) = child.as_element()
                        && co.name == "Co"
                    {
                        let k = co.attributes.get("K").cloned().unwrap_or_default();
                        let has_in = co
                            .children
                            .iter()
                            .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
                        if has_in
                            && let Some(io) = types.get(&k)
                            && io == "O"
                        {
                            results.push(format!(
                                            "⚠ {}.{}: wired TO an Output connector (should wire TO Input/Parameter)",
                                            title, k
                                        ));
                            *warnings += 1;
                        }
                    }
                }
            }
            for child in &elem.children {
                if let Some(child_elem) = child.as_element() {
                    check_wiring(child_elem, cmap, results, warnings);
                }
            }
        }
        check_wiring(&self.root, &cmap, &mut results, &mut wiring_warnings);
        if wiring_warnings == 0 {
            results.push("✓ All wiring directions valid (Output→Input/Parameter)".to_string());
        }

        // ── Tree/Air sensor hardware block checks ─────────────────────────
        let mut sensor_issues: Vec<String> = Vec::new();
        Self::check_sensor_blocks(&self.root, &mut sensor_issues);
        if sensor_issues.is_empty() {
            results.push(
                "✓ All Tree/Air sensor blocks have Display, connectors, and IoData.Cr".to_string(),
            );
        } else {
            for issue in &sensor_issues {
                results.push(format!("⚠ {issue}"));
            }
        }

        results
    }

    /// Validate Tree/Air hardware sensor blocks for completeness.
    fn check_sensor_blocks(elem: &Element, issues: &mut Vec<String>) {
        if elem.name == "C" {
            let etype = elem
                .attributes
                .get("Type")
                .map(String::as_str)
                .unwrap_or("");
            let title = elem.attributes.get("Title").cloned().unwrap_or_default();

            let is_digital = matches!(etype, "TreeSensor" | "LoxAIRsensor");
            let is_analog = matches!(etype, "TreeAsensor" | "LoxAIRAsensor");

            if is_digital || is_analog {
                // Every sensor must have a <Display> child
                let has_display = elem
                    .children
                    .iter()
                    .any(|c| c.as_element().is_some_and(|e| e.name == "Display"));
                if !has_display {
                    issues.push(format!("{etype} '{title}': missing <Display> child"));
                }

                // Every sensor IoData must have a Cr (category) reference
                let has_iodata_cr = elem.children.iter().any(|c| {
                    c.as_element().is_some_and(|e| {
                        e.name == "IoData" && e.attributes.get("Cr").is_some_and(|v| !v.is_empty())
                    })
                });
                if !has_iodata_cr {
                    issues.push(format!(
                        "{etype} '{title}': IoData missing Cr (category) reference"
                    ));
                }
            }

            // Analog sensors must have both AQ and (Q or Qm) connectors
            if is_analog {
                let connector_keys: Vec<String> = elem
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "Co")
                    .filter_map(|e| e.attributes.get("K").cloned())
                    .collect();

                let has_aq = connector_keys.iter().any(|k| k == "AQ");
                let has_q_or_qm = connector_keys.iter().any(|k| k == "Q" || k == "Qm");

                if !has_aq || !has_q_or_qm {
                    issues.push(format!(
                        "{etype} '{title}': must have both AQ and Q (or Qm) connectors"
                    ));
                }
            }
        }

        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                Self::check_sensor_blocks(child_elem, issues);
            }
        }
    }

    fn collect_typed_uuids(
        &self,
        elem: &Element,
        type_name: &str,
        uuids: &mut std::collections::HashSet<String>,
    ) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == type_name
            && let Some(u) = elem.attributes.get("U")
        {
            uuids.insert(u.clone());
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.collect_typed_uuids(child_elem, type_name, uuids);
            }
        }
    }

    fn validate_recursive(
        &self,
        elem: &Element,
        places: &std::collections::HashSet<String>,
        categories: &std::collections::HashSet<String>,
        bad_rooms: &mut Vec<String>,
        bad_cats: &mut Vec<String>,
        unconnected: &mut Vec<String>,
    ) {
        if elem.name == "IoData" {
            if let Some(pr) = elem.attributes.get("Pr")
                && !pr.is_empty()
                && !places.contains(pr)
            {
                bad_rooms.push(pr.clone());
            }
            if let Some(cr) = elem.attributes.get("Cr")
                && !cr.is_empty()
                && !categories.contains(cr)
            {
                bad_cats.push(cr.clone());
            }
        }
        if elem.name == "Co"
            && let Some(u) = elem.attributes.get("U")
            && u.is_empty()
        {
            let kind = elem.attributes.get("K").cloned().unwrap_or_default();
            unconnected.push(kind);
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.validate_recursive(
                    child_elem,
                    places,
                    categories,
                    bad_rooms,
                    bad_cats,
                    unconnected,
                );
            }
        }
    }
}

/// Returns true for known system / operating-mode / registered-device short-form
/// UUIDs that legitimately lack the Miniserver serial suffix in every Gen-2
/// export (e.g. `00000000-0000-0002-1` operating modes, `205d5330-…-f`
/// registered devices). These should be treated as informational, not errors.
fn is_system_short_uuid(uuid: &str) -> bool {
    // All-zero prefixed system/state UUIDs.
    if uuid.starts_with("00000000-0000-") {
        return true;
    }
    // Short-form UUIDs: the final dash-segment is shorter than a serial-bearing
    // segment (normal connector UUIDs end with `ffff` + a 12-hex serial = 16).
    match uuid.rsplit_once('-') {
        Some((_, last)) => last.len() < 12,
        None => false,
    }
}
