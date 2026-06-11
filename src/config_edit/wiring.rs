use super::{ConfigEditor, MoodInfo, MqttTopic, WireInfo};
use anyhow::Result;
use xmltree::Element;

impl ConfigEditor {
    /// Create an OutputRef to wire a control's output to a physical device.
    /// This creates the OutputRef proxy and wires both sides.
    pub fn create_output_ref(
        &mut self,
        control_selector: &str,
        output_conn: &str,
        device_selector: &str,
    ) -> Result<String> {
        // Find the control and its output connector UUID
        let ctrl_path = self.require_one(control_selector)?;
        let ctrl = self.get_element(&ctrl_path);
        let ctrl_title = ctrl.attributes.get("Title").cloned().unwrap_or_default();
        let _ctrl_type = ctrl.attributes.get("Type").cloned().unwrap_or_default();

        let output_uuid = ctrl
            .children
            .iter()
            .find_map(|c| {
                c.as_element().and_then(|e| {
                    if e.name == "Co"
                        && e.attributes
                            .get("K")
                            .map(|k| k == output_conn)
                            .unwrap_or(false)
                    {
                        e.attributes.get("U").cloned()
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                let available: Vec<String> = ctrl
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "Co")
                    .filter_map(|e| e.attributes.get("K").cloned())
                    .collect();
                anyhow::anyhow!(
                    "Connector '{}' not found on '{}'. Available: {}",
                    output_conn,
                    ctrl_title,
                    available.join(", ")
                )
            })?;

        // Find the device
        let dev_path = self.require_one(device_selector)?;
        let dev = self.get_element(&dev_path);
        let dev_title = dev.attributes.get("Title").cloned().unwrap_or_default();
        let dev_type = dev.attributes.get("Type").cloned().unwrap_or_default();
        let dev_uuid = dev.attributes.get("U").cloned().unwrap_or_default();
        let dev_is_analog = dev
            .attributes
            .get("IName")
            .map(|n| n.starts_with("AQ"))
            .unwrap_or(false);

        // Determine LinkRefType based on device type
        let link_ref_type = match dev_type.as_str() {
            "Actor" => "55",
            "LoxAIRAactor" | "TreeAactor" => "175",
            "LoxAIRactor" | "TreeActor" => "55",
            _ => "175",
        };

        let serial = self.find_miniserver_serial().unwrap_or_default();

        // Create OutputRef element
        let oref_uuid = Self::loxone_uuid(&serial);
        let ai_uuid = Self::loxone_uuid(&serial);
        let aq_uuid = Self::loxone_uuid(&serial);

        let mut oref = Element::new("C");
        oref.attributes
            .insert("Type".to_string(), "OutputRef".to_string());
        oref.attributes.insert("V".to_string(), "175".to_string());
        oref.attributes.insert("U".to_string(), oref_uuid.clone());
        oref.attributes
            .insert("Title".to_string(), dev_title.clone());
        oref.attributes.insert("Ref".to_string(), dev_uuid.clone());
        oref.attributes
            .insert("LinkRefType".to_string(), link_ref_type.to_string());
        oref.attributes.insert("Nio".to_string(), "2".to_string());
        oref.attributes
            .insert("WF".to_string(), "18432".to_string());
        if dev_is_analog {
            oref.attributes
                .insert("Analog".to_string(), "true".to_string());
        }

        // AI connector: receives from control's output
        let mut co_ai = Element::new("Co");
        co_ai.attributes.insert("K".to_string(), "AI".to_string());
        co_ai.attributes.insert("U".to_string(), ai_uuid.clone());
        co_ai.attributes.insert("Nc".to_string(), "1".to_string());
        let mut in_elem = Element::new("In");
        in_elem
            .attributes
            .insert("Input".to_string(), output_uuid.clone());
        co_ai.children.push(xmltree::XMLNode::Element(in_elem));
        oref.children.push(xmltree::XMLNode::Element(co_ai));

        // AQ connector
        let mut co_aq = Element::new("Co");
        co_aq.attributes.insert("K".to_string(), "AQ".to_string());
        co_aq.attributes.insert("U".to_string(), aq_uuid.clone());
        oref.children.push(xmltree::XMLNode::Element(co_aq));

        // Place OutputRef on the first Page (same as where controls live)
        let page_paths = self.find_elements("Type:Page");
        if let Some(page_path) = page_paths.into_iter().next() {
            let page = self.get_element_mut(&page_path);
            page.children.push(xmltree::XMLNode::Element(oref));
        } else {
            // Fallback: add to root
            self.root.children.push(xmltree::XMLNode::Element(oref));
        }

        // Wire device.I ← OutputRef.AQ
        let dev = self.get_element_mut(&dev_path);
        for child in &mut dev.children {
            if let Some(co) = child.as_mut_element()
                && co.name == "Co"
                && co.attributes.get("K").map(|k| k == "I").unwrap_or(false)
            {
                let mut in_elem = Element::new("In");
                in_elem
                    .attributes
                    .insert("Input".to_string(), aq_uuid.clone());
                co.children.push(xmltree::XMLNode::Element(in_elem));
                let in_count = co
                    .children
                    .iter()
                    .filter(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false))
                    .count();
                co.attributes.insert("Nc".to_string(), in_count.to_string());
                break;
            }
        }

        Ok(format!(
            "✓ Bound {}.{} → {} '{}' via OutputRef (LRT={})",
            ctrl_title, output_conn, dev_type, dev_title, link_ref_type
        ))
    }

    /// Wire two connectors: set source_element.connector_name → target_element's connector UUID.
    ///
    /// `source`: element selector (e.g. "Kitchen Light")
    /// `source_connector`: connector name on the source (e.g. "On", "AQ1")
    /// `target`: element selector for the target
    /// `target_connector`: connector name on the target (e.g. "I", "Q")
    pub fn wire(
        &mut self,
        source: &str,
        source_connector: &str,
        target: &str,
        target_connector: &str,
    ) -> Result<String> {
        // Find target element and its connector UUID
        let target_path = self.require_one(target)?;

        // Auto-create variadic input connectors (I3-I5) for logic gate blocks
        {
            let target_elem = self.get_element(&target_path);
            let block_type = target_elem
                .attributes
                .get("Type")
                .cloned()
                .unwrap_or_default();
            let variadic_types = ["And", "Or", "Nand", "Nor", "Xor", "State"];
            let has_conn = target_elem.children.iter().any(|c| {
                c.as_element()
                    .map(|e| {
                        e.name == "Co"
                            && e.attributes
                                .get("K")
                                .map(|k| k == target_connector)
                                .unwrap_or(false)
                    })
                    .unwrap_or(false)
            });
            if !has_conn
                && variadic_types.contains(&block_type.as_str())
                && target_connector.starts_with('I')
                && target_connector.len() <= 3
                && let Ok(idx) = target_connector[1..].parse::<u32>()
                && (3..=20).contains(&idx)
            {
                let serial = self.find_miniserver_serial().unwrap_or_default();
                let elem = self.get_element_mut(&target_path);
                for i in 3..=idx {
                    let key = format!("I{i}");
                    let already = elem.children.iter().any(|c| {
                        c.as_element()
                            .map(|e| {
                                e.name == "Co"
                                    && e.attributes.get("K").map(|k| k == &key).unwrap_or(false)
                            })
                            .unwrap_or(false)
                    });
                    if !already {
                        let mut co = Element::new("Co");
                        co.attributes.insert("K".to_string(), key);
                        co.attributes
                            .insert("U".to_string(), Self::loxone_uuid(&serial));
                        elem.children.push(xmltree::XMLNode::Element(co));
                    }
                }
                let nio = elem
                    .children
                    .iter()
                    .filter(|c| c.as_element().map(|e| e.name == "Co").unwrap_or(false))
                    .count();
                elem.attributes.insert("Nio".to_string(), nio.to_string());
            }
        }

        let target_elem = self.get_element(&target_path);
        let target_title = target_elem
            .attributes
            .get("Title")
            .cloned()
            .unwrap_or_default();

        let target_co_uuid = target_elem
            .children
            .iter()
            .find_map(|c| {
                c.as_element().and_then(|e| {
                    if e.name == "Co"
                        && e.attributes
                            .get("K")
                            .map(|k| k == target_connector)
                            .unwrap_or(false)
                    {
                        e.attributes.get("U").cloned()
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                let available: Vec<String> = target_elem
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "Co")
                    .filter_map(|e| e.attributes.get("K").cloned())
                    .collect();
                crate::errors::not_found_error(
                    "Connector",
                    target_connector,
                    &available,
                    &format!("lox config control describe <file> \"{}\"", target),
                )
            })?;

        // Find source element and update its connector
        let source_path = self.require_one(source)?;
        let source_elem = self.get_element_mut(&source_path);
        let source_title = source_elem
            .attributes
            .get("Title")
            .cloned()
            .unwrap_or_default();

        let source_co = source_elem
            .children
            .iter_mut()
            .find_map(|c| {
                c.as_mut_element().and_then(|e| {
                    if e.name == "Co"
                        && e.attributes
                            .get("K")
                            .map(|k| k == source_connector)
                            .unwrap_or(false)
                    {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Connector '{}' not found on '{}'",
                    source_connector,
                    source_title
                )
            })?;

        let _old_target = source_co.attributes.get("U").cloned().unwrap_or_default();
        source_co
            .attributes
            .insert("U".to_string(), target_co_uuid.clone());

        Ok(format!(
            "Wired {}.{} → {}.{} ({})",
            source_title, source_connector, target_title, target_connector, target_co_uuid
        ))
    }

    /// Disconnect a connector (set its target UUID to empty/zero).
    pub fn unwire(&mut self, selector: &str, connector_name: &str) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();

        let co = elem
            .children
            .iter_mut()
            .find_map(|c| {
                c.as_mut_element().and_then(|e| {
                    if e.name == "Co"
                        && e.attributes
                            .get("K")
                            .map(|k| k == connector_name)
                            .unwrap_or(false)
                    {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Connector '{}' not found on '{}'", connector_name, title)
            })?;

        let old = co.attributes.get("U").cloned().unwrap_or_default();
        // Remove all <In> children (the wiring references)
        let had_in_children = co
            .children
            .iter()
            .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
        co.children
            .retain(|c| !c.as_element().map(|e| e.name == "In").unwrap_or(false));
        // Remove Nc attribute (connection count)
        co.attributes.shift_remove("Nc");
        // If no <In> children were present, the wiring was via Co U= attribute — clear it
        if !had_in_children {
            co.attributes.shift_remove("U");
        }

        Ok(format!(
            "Unwired {}.{} (was {})",
            title, connector_name, old
        ))
    }

    /// List all connectors and their wiring for an element.
    pub fn list_wires(&self, selector: &str) -> Result<Vec<WireInfo>> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(&path);

        let mut wires = Vec::new();
        for child in &elem.children {
            if let Some(co) = child.as_element()
                && co.name == "Co"
            {
                let name = co.attributes.get("K").cloned().unwrap_or_default();
                let target_uuid = co.attributes.get("U").cloned().unwrap_or_default();

                // Classify direction
                let direction =
                    if name.starts_with('I') || name.starts_with("AI") || name == "Input" {
                        "input"
                    } else if name.starts_with('Q')
                        || name.starts_with("AQ")
                        || name.starts_with("Output")
                    {
                        "output"
                    } else {
                        "parameter"
                    };

                let has_in_children = co
                    .children
                    .iter()
                    .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
                let connected = has_in_children
                    || (!target_uuid.is_empty()
                        && target_uuid != "00000000-0000-0000-0000000000000000");

                wires.push(WireInfo {
                    connector: name,
                    direction: direction.to_string(),
                    target_uuid,
                    connected,
                });
            }
        }
        Ok(wires)
    }

    /// List all MQTT topics (GenTSensor subscriptions + GenTActor publishes).
    pub fn list_mqtt_topics(&self) -> Vec<MqttTopic> {
        let mut topics = Vec::new();
        self.collect_mqtt_topics(&self.root, &mut topics);
        topics
    }

    fn collect_mqtt_topics(&self, elem: &Element, topics: &mut Vec<MqttTopic>) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && (t == "GenTSensor" || t == "GenTActor")
        {
            let title = elem.attributes.get("Title").cloned().unwrap_or_default();
            let direction = if t == "GenTSensor" {
                "subscribe"
            } else {
                "publish"
            };

            // Get topic from SET properties
            let mut topic = String::new();
            let mut qos = String::new();
            for child in &elem.children {
                if let Some(set) = child.as_element()
                    && set.name == "SET"
                {
                    for prop in &set.children {
                        if let Some(p) = prop.as_element() {
                            if p.name == "mqtt_topic" {
                                topic = p.attributes.get("v").cloned().unwrap_or_default();
                            }
                            if p.name == "mqtt_qos" {
                                qos = p.attributes.get("v").cloned().unwrap_or_default();
                            }
                        }
                    }
                }
            }

            topics.push(MqttTopic {
                title,
                direction: direction.to_string(),
                topic,
                qos,
            });
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.collect_mqtt_topics(child_elem, topics);
            }
        }
    }

    /// Resolve stale InputRef wiring by updating connector UUIDs
    /// to match the live config from the Miniserver.
    ///
    /// When Loxone Config UX saves, it regenerates InputRef connector UUIDs.
    /// Blocks wired to old UUIDs still function (SPS follows VirtualInCaption-level refs)
    /// but UX won't draw visual wires. This remaps old → current UUIDs.
    pub fn resolve_wiring(&mut self, live_xml: &[u8]) -> Result<usize> {
        let live = ConfigEditor::load(live_xml)?;

        // From live config: map VirtualIn UUID → current AQ/Q connector UUID
        let mut live_vi_to_aq: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        Self::collect_inputref_aq(&live.root, &mut live_vi_to_aq);

        // From our config: map old AQ UUID → VirtualIn UUID
        let mut our_aq_to_vi: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        Self::collect_inputref_aq_reverse(&self.root, &mut our_aq_to_vi);

        // Build old→new replacement map
        let mut replacements: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (old_aq, vi_uuid) in &our_aq_to_vi {
            if let Some(new_aq) = live_vi_to_aq.get(vi_uuid)
                && old_aq != new_aq
            {
                replacements.insert(old_aq.clone(), new_aq.clone());
            }
        }

        if replacements.is_empty() {
            return Ok(0);
        }

        // Rewrite <In Input="old"/> → <In Input="new"/>
        let count = Self::rewrite_in_inputs(&mut self.root, &replacements);

        // Also update InputRef connector UUIDs themselves (Co K="AQ" U="old" → U="new")
        Self::rewrite_connector_uuids(&mut self.root, &replacements);

        Ok(count)
    }

    fn collect_inputref_aq(
        elem: &Element,
        vi_to_aq: &mut std::collections::HashMap<String, String>,
    ) {
        if elem
            .attributes
            .get("Type")
            .map(|t| t == "InputRef")
            .unwrap_or(false)
            && let Some(ref_uuid) = elem.attributes.get("Ref")
        {
            for child in &elem.children {
                if let Some(co) = child.as_element()
                    && co.name == "Co"
                {
                    let k = co.attributes.get("K").map(|s| s.as_str()).unwrap_or("");
                    if let Some(u) = co.attributes.get("U")
                        && (k == "AQ" || k == "Q")
                    {
                        vi_to_aq.insert(format!("{}:{}", ref_uuid, k), u.clone());
                    }
                }
            }
        }
        for child in &elem.children {
            if let Some(e) = child.as_element() {
                Self::collect_inputref_aq(e, vi_to_aq);
            }
        }
    }

    fn collect_inputref_aq_reverse(
        elem: &Element,
        aq_to_vi: &mut std::collections::HashMap<String, String>,
    ) {
        if elem
            .attributes
            .get("Type")
            .map(|t| t == "InputRef")
            .unwrap_or(false)
            && let Some(ref_uuid) = elem.attributes.get("Ref")
        {
            for child in &elem.children {
                if let Some(co) = child.as_element()
                    && co.name == "Co"
                {
                    let k = co.attributes.get("K").map(|s| s.as_str()).unwrap_or("");
                    if let Some(u) = co.attributes.get("U")
                        && (k == "AQ" || k == "Q")
                    {
                        aq_to_vi.insert(u.clone(), format!("{}:{}", ref_uuid, k));
                    }
                }
            }
        }
        for child in &elem.children {
            if let Some(e) = child.as_element() {
                Self::collect_inputref_aq_reverse(e, aq_to_vi);
            }
        }
    }

    fn rewrite_in_inputs(
        elem: &mut Element,
        replacements: &std::collections::HashMap<String, String>,
    ) -> usize {
        let mut count = 0;
        for child in &mut elem.children {
            if let Some(e) = child.as_mut_element() {
                if e.name == "In"
                    && let Some(input) = e.attributes.get("Input").cloned()
                    && let Some(new_uuid) = replacements.get(&input)
                {
                    e.attributes.insert("Input".to_string(), new_uuid.clone());
                    count += 1;
                }
                count += Self::rewrite_in_inputs(e, replacements);
            }
        }
        count
    }

    fn rewrite_connector_uuids(
        elem: &mut Element,
        replacements: &std::collections::HashMap<String, String>,
    ) {
        // Update Co U="old" → U="new" for InputRef connectors
        if elem.name == "Co"
            && let Some(u) = elem.attributes.get("U").cloned()
            && let Some(new_u) = replacements.get(&u)
        {
            elem.attributes.insert("U".to_string(), new_u.clone());
        }
        for child in &mut elem.children {
            if let Some(e) = child.as_mut_element() {
                Self::rewrite_connector_uuids(e, replacements);
            }
        }
    }

    /// Find a connector UUID by block title and connector key.
    #[allow(dead_code)]
    pub fn find_connector_uuid(&self, block_selector: &str, conn_key: &str) -> Result<String> {
        let path = self.require_one(block_selector)?;
        let elem = self.get_element(&path);
        elem.children
            .iter()
            .find_map(|c| {
                c.as_element().and_then(|e| {
                    if e.name == "Co"
                        && e.attributes
                            .get("K")
                            .map(|k| k == conn_key)
                            .unwrap_or(false)
                    {
                        e.attributes.get("U").cloned()
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Connector '{}' not found on '{}'", conn_key, block_selector)
            })
    }

    /// Add schedule entries to a DayTimer block.
    ///
    /// Replaces any existing entries with a pair that defines an active window:
    ///   - Entry To=start_minutes V="0"  (off before start)
    ///   - Entry To=end_minutes   V=value (on during window)
    ///
    /// If start > 0, a leading off-entry is added; a trailing off-entry at 1440
    /// is added when end < 1440.
    pub fn add_daytimer_entries(
        &mut self,
        selector: &str,
        start_minutes: u32,
        end_minutes: u32,
        value: &str,
    ) -> Result<()> {
        use anyhow::bail;

        if start_minutes >= end_minutes && start_minutes < 1440 && end_minutes > 0 {
            // Overnight schedule (e.g. 22:00-06:00): split into two windows
            // Window 1: 00:00 → end_minutes (active)
            // Window 2: end_minutes → start_minutes (off)
            // Window 3: start_minutes → 24:00 (active)
            let path = self.require_one(selector)?;
            let elem = self.get_element_mut(&path);
            let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
            if block_type != "DayTimer" {
                bail!("'{}' is a {}, not a DayTimer", selector, block_type);
            }
            elem.children
                .retain(|c| !c.as_element().map(|e| e.name == "Entry").unwrap_or(false));

            // 00:00 → end: active
            let mut e1 = Element::new("Entry");
            e1.attributes
                .insert("To".to_string(), end_minutes.to_string());
            e1.attributes.insert("V".to_string(), value.to_string());
            elem.children.push(xmltree::XMLNode::Element(e1));

            // end → start: off
            let mut e2 = Element::new("Entry");
            e2.attributes
                .insert("To".to_string(), start_minutes.to_string());
            e2.attributes.insert("V".to_string(), "0".to_string());
            elem.children.push(xmltree::XMLNode::Element(e2));

            // start → 24:00: active
            let mut e3 = Element::new("Entry");
            e3.attributes.insert("To".to_string(), "1440".to_string());
            e3.attributes.insert("V".to_string(), value.to_string());
            elem.children.push(xmltree::XMLNode::Element(e3));

            return Ok(());
        }

        if start_minutes >= end_minutes {
            bail!("Start time must be before end time");
        }
        if end_minutes > 1440 {
            bail!("End time cannot exceed 24:00 (1440 minutes)");
        }

        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);

        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
        if block_type != "DayTimer" {
            bail!("'{}' is a {}, not a DayTimer", selector, block_type);
        }

        // Remove existing Entry elements
        elem.children
            .retain(|c| !c.as_element().map(|e| e.name == "Entry").unwrap_or(false));

        // Add off-period before start (if start > 0)
        if start_minutes > 0 {
            let mut entry = Element::new("Entry");
            entry
                .attributes
                .insert("To".to_string(), start_minutes.to_string());
            entry.attributes.insert("V".to_string(), "0".to_string());
            elem.children.push(xmltree::XMLNode::Element(entry));
        }

        // Add on-period
        let mut entry = Element::new("Entry");
        entry
            .attributes
            .insert("To".to_string(), end_minutes.to_string());
        entry.attributes.insert("V".to_string(), value.to_string());
        elem.children.push(xmltree::XMLNode::Element(entry));

        // Add off-period after end (if end < 1440)
        if end_minutes < 1440 {
            let mut entry = Element::new("Entry");
            entry
                .attributes
                .insert("To".to_string(), "1440".to_string());
            entry.attributes.insert("V".to_string(), "0".to_string());
            elem.children.push(xmltree::XMLNode::Element(entry));
        }

        Ok(())
    }

    /// Wire a connector by adding `<In Input="source_uuid"/>` to target connector.
    /// Uses exact title match to avoid ambiguity.
    ///
    /// `source_uuid` may be passed bare or with a `uuid:` prefix (the prefix is
    /// stripped). By default an existing single source is **replaced**; pass
    /// `append = true` to add an additional source instead.
    pub fn wire_connector(
        &mut self,
        block_selector: &str,
        conn_key: &str,
        source_uuid: &str,
        append: bool,
    ) -> Result<()> {
        // Accept (and strip) a `uuid:` prefix on the source — the wire stores a
        // bare connector UUID, never the selector form.
        let source_uuid = source_uuid
            .strip_prefix("uuid:")
            .unwrap_or(source_uuid)
            .trim();

        // Validate the source is a real output connector (warn, don't fail —
        // cross-file references and synthesized sources are legitimate).
        match self.connector_dir_by_uuid(source_uuid) {
            Some(true) => {}
            Some(false) => eprintln!(
                "⚠ Warning: source '{}' is an INPUT connector — wires flow FROM outputs TO inputs",
                source_uuid
            ),
            None => eprintln!(
                "⚠ Warning: source '{}' is not a known connector in this file (dangling reference?)",
                source_uuid
            ),
        }

        // Use standard selector matching (supports uuid:, gid:, Type:, and title)
        let path = self.require_one(block_selector)?;

        // Detect cross-page wiring: check if source is on a different Page than target
        let target_page = self.find_page_for_path(&path);
        let source_page = self.find_page_for_connector(source_uuid);
        let cross_page = match (&target_page, &source_page) {
            (Some(tp), Some(sp)) => tp != sp,
            (_, None) => true, // source not on any page = cross-page
            _ => false,
        };

        // Validate: target connector should be Input or Parameter, not Output
        let elem = self.get_element(&path);
        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
        if let Some(io_type) = Self::connector_io_type(&block_type, conn_key)
            && io_type == "O"
        {
            eprintln!(
                "⚠ Warning: wiring TO output connector '{}.{}' — normally you wire TO inputs, FROM outputs",
                block_selector, conn_key
            );
        }

        // Pre-fetch serial before mutable borrow (needed for auto-creating connectors)
        let serial = self.find_miniserver_serial().unwrap_or_default();

        let elem = self.get_element_mut(&path);

        // Collect available connectors for error message
        let available: Vec<String> = elem
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|e| e.name == "Co")
            .filter_map(|e| e.attributes.get("K").cloned())
            .collect();

        // Auto-create variadic input connectors (I3+) for logic gate and state blocks
        let variadic_types = ["And", "Or", "Nand", "Nor", "Xor", "State"];
        if !available.contains(&conn_key.to_string())
            && variadic_types.contains(&block_type.as_str())
            && conn_key.starts_with('I')
            && conn_key.len() <= 3
            && let Ok(idx) = conn_key[1..].parse::<u32>()
            && (3..=20).contains(&idx)
        {
            // Create any missing intermediate connectors (e.g. I3 before I4)
            for i in 3..=idx {
                let key = format!("I{i}");
                let already = elem.children.iter().any(|c| {
                    c.as_element()
                        .map(|e| {
                            e.name == "Co"
                                && e.attributes.get("K").map(|k| k == &key).unwrap_or(false)
                        })
                        .unwrap_or(false)
                });
                if !already {
                    let mut co = Element::new("Co");
                    co.attributes.insert("K".to_string(), key);
                    co.attributes
                        .insert("U".to_string(), Self::loxone_uuid(&serial));
                    elem.children.push(xmltree::XMLNode::Element(co));
                }
            }
            // Update Nio to reflect the new connector count
            let nio = elem
                .children
                .iter()
                .filter(|c| c.as_element().map(|e| e.name == "Co").unwrap_or(false))
                .count();
            elem.attributes.insert("Nio".to_string(), nio.to_string());
        }

        // Auto-create missing connectors from the embedded connector map
        if !available.contains(&conn_key.to_string()) && !block_type.is_empty() {
            let connector_map = Self::connector_map();
            if let Some((connectors, defaults, _)) = connector_map.get(block_type.as_str())
                && connectors.contains(&conn_key.to_string())
            {
                let mut co = Element::new("Co");
                co.attributes.insert("K".to_string(), conn_key.to_string());
                co.attributes
                    .insert("U".to_string(), Self::loxone_uuid(&serial));
                if let Some(def_val) = defaults.get(conn_key) {
                    co.attributes.insert("Def".to_string(), def_val.clone());
                }
                elem.children.push(xmltree::XMLNode::Element(co));
                let nio = elem
                    .children
                    .iter()
                    .filter(|c| c.as_element().map(|e| e.name == "Co").unwrap_or(false))
                    .count();
                elem.attributes.insert("Nio".to_string(), nio.to_string());
            }
        }

        let co = elem
            .children
            .iter_mut()
            .find_map(|c| {
                c.as_mut_element().and_then(|e| {
                    if e.name == "Co"
                        && e.attributes
                            .get("K")
                            .map(|k| k == conn_key)
                            .unwrap_or(false)
                    {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Connector '{}' not found on '{}'. Available connectors: {}",
                    conn_key,
                    block_selector,
                    available.join(", ")
                )
            })?;

        // Add <In Input="source_uuid"/> child, with FLG="2" for cross-page wiring.
        // By default replace any existing source(s); only keep them when appending.
        if !append {
            co.children
                .retain(|c| c.as_element().map(|e| e.name != "In").unwrap_or(true));
        }
        let mut in_elem = Element::new("In");
        in_elem
            .attributes
            .insert("Input".to_string(), source_uuid.to_string());
        if cross_page {
            in_elem
                .attributes
                .insert("FLG".to_string(), "2".to_string());
        }
        co.children.push(xmltree::XMLNode::Element(in_elem));

        // Set Nc (connection count) to the number of In children
        let in_count = co
            .children
            .iter()
            .filter(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false))
            .count();
        co.attributes.insert("Nc".to_string(), in_count.to_string());

        Ok(())
    }

    /// Find which Page UUID a path belongs to (walks up the path to find a Page ancestor).
    fn find_page_for_path(&self, path: &[usize]) -> Option<String> {
        // Walk path from root, check if any ancestor is a Page
        let mut current = &self.root;
        let mut last_page = None;
        for &idx in path {
            if current.name == "C"
                && current
                    .attributes
                    .get("Type")
                    .map(|t| t == "Page")
                    .unwrap_or(false)
            {
                last_page = current.attributes.get("U").cloned();
            }
            if let Some(child) = current.children.get(idx).and_then(|c| c.as_element()) {
                current = child;
            } else {
                break;
            }
        }
        last_page
    }

    /// Re-point an actor's `OutputRef.AI` to a new source ("detach" the actor
    /// from its owning LightController so a value source — mux / VirtualIn —
    /// can drive it directly).
    ///
    /// `selector` may resolve to the `OutputRef` block itself, or to the device
    /// actor (`LoxAIRAactor`/`TreeAactor`) whose input `I` is fed by the
    /// OutputRef's `AQ` — in which case the driving OutputRef is found
    /// automatically. The OutputRef's `AI` source is replaced with `source_uuid`.
    pub fn splice_actor(&mut self, selector: &str, source_uuid: &str) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(&path);
        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();

        let outputref_uuid = if block_type == "OutputRef" {
            elem.attributes
                .get("U")
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("OutputRef '{selector}' has no UUID"))?
        } else {
            // Find the actor's driving input source (the OutputRef's AQ uuid).
            let aq_source = elem
                .children
                .iter()
                .filter_map(|c| c.as_element())
                .filter(|e| e.name == "Co")
                .filter(|e| {
                    matches!(
                        e.attributes.get("K").map(String::as_str),
                        Some("I") | Some("AI")
                    )
                })
                .find_map(|co| {
                    co.children
                        .iter()
                        .filter_map(|c| c.as_element())
                        .find_map(|inn| {
                            if inn.name == "In" {
                                inn.attributes.get("Input").cloned()
                            } else {
                                None
                            }
                        })
                })
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "'{selector}' (type {block_type}) has no wired I/AI input — \
                         pass the OutputRef block directly"
                    )
                })?;
            self.find_block_owning_connector(&aq_source, "AQ")
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "could not find an OutputRef whose AQ ({aq_source}) drives '{selector}'"
                    )
                })?
        };

        let sel = format!("uuid:{outputref_uuid}");
        self.wire_connector(&sel, "AI", source_uuid, false)?;
        Ok(outputref_uuid)
    }

    /// Find the UUID of the block (`C`) that owns a connector with the given
    /// connector UUID and key.
    fn find_block_owning_connector(&self, connector_uuid: &str, key: &str) -> Option<String> {
        fn search(elem: &Element, target: &str, key: &str) -> Option<String> {
            if elem.name == "C" {
                for child in &elem.children {
                    if let Some(co) = child.as_element()
                        && co.name == "Co"
                        && co.attributes.get("K").map(|k| k == key).unwrap_or(false)
                        && co.attributes.get("U").map(|u| u == target).unwrap_or(false)
                    {
                        return elem.attributes.get("U").cloned();
                    }
                }
            }
            for child in &elem.children {
                if let Some(e) = child.as_element()
                    && let Some(found) = search(e, target, key)
                {
                    return Some(found);
                }
            }
            None
        }
        search(&self.root, connector_uuid, key)
    }

    /// List the light scenes ("moods") defined on a LightController2.
    ///
    /// Moods are stored as `<LightsceneC>` children of the controller's
    /// `<LightscenesC>` element. Each carries a `Name`, `UUID`, scene id (`SID`)
    /// and the per-output values `Q1..Qn`.
    pub fn list_moods(&self, selector: &str) -> Result<Vec<MoodInfo>> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(&path);
        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
        if block_type != "LightController2" {
            anyhow::bail!(
                "'{selector}' is a {block_type}, not a LightController2 — moods only exist on light controllers"
            );
        }
        let mut moods = Vec::new();
        fn collect(elem: &Element, out: &mut Vec<MoodInfo>) {
            if elem.name == "LightsceneC" {
                out.push(MoodInfo {
                    name: elem.attributes.get("Name").cloned().unwrap_or_default(),
                    uuid: elem.attributes.get("UUID").cloned().unwrap_or_default(),
                    sid: elem.attributes.get("SID").and_then(|s| s.parse().ok()),
                    cid: elem.attributes.get("CID").and_then(|s| s.parse().ok()),
                    q1: elem.attributes.get("Q1").cloned(),
                });
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    collect(e, out);
                }
            }
        }
        collect(elem, &mut moods);
        Ok(moods)
    }

    /// Rewrite the color of an existing mood (`LightsceneC`) on a
    /// `LightController2` by setting its per-output value `Q{output}`.
    ///
    /// `mood_ref` matches the mood by `SID` (numeric scene id) or `Name`.
    /// `value` is the packed mood value (percent channels + `0x60000000`),
    /// e.g. from `lox color encode --mood`. `output` is the 1-based Q index
    /// (default 1 — the first/color output).
    pub fn set_mood_color(
        &mut self,
        selector: &str,
        mood_ref: &str,
        output: usize,
        value: i64,
    ) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
        if block_type != "LightController2" {
            anyhow::bail!(
                "'{selector}' is a {block_type}, not a LightController2 — moods only exist on light controllers"
            );
        }
        let controller_title = elem.attributes.get("Title").cloned().unwrap_or_default();
        let qkey = format!("Q{output}");

        fn set_on_match(elem: &mut Element, mref: &str, qkey: &str, value: &str) -> bool {
            if elem.name == "LightsceneC" {
                let sid = elem.attributes.get("SID").map(|s| s.as_str()).unwrap_or("");
                let name = elem
                    .attributes
                    .get("Name")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if sid == mref || name == mref {
                    elem.attributes.insert(qkey.to_string(), value.to_string());
                    return true;
                }
            }
            for child in &mut elem.children {
                if let Some(e) = child.as_mut_element()
                    && set_on_match(e, mref, qkey, value)
                {
                    return true;
                }
            }
            false
        }

        if set_on_match(elem, mood_ref, &qkey, &value.to_string()) {
            Ok(format!(
                "Set {qkey}={value} on mood '{mood_ref}' of LightController2 '{controller_title}'"
            ))
        } else {
            anyhow::bail!(
                "mood '{mood_ref}' not found on LightController2 '{controller_title}' (match by SID or Name)"
            )
        }
    }

    /// Create a brand-new mood (`LightsceneC`) on a `LightController2`.
    ///
    /// Mirrors how Loxone Config appends a scene: a fresh `LightsceneC` is
    /// added to the controller's `<LightscenesC>` container with a generated
    /// UUID, the next free custom scene id (`SID`) and color id (`CID`), and
    /// per-output values `Q1..Qn` (Q1 set to `value`, the rest 0). The
    /// container's `Num` count is incremented.
    ///
    /// Reserved SIDs (1/776/777/778 = Entspannen/Wecker/Viel Licht/Aus) are
    /// skipped; custom moods start at SID=2 / CID=9. Pass `sid`/`cid` to
    /// override. Returns `(name, uuid, sid, cid)`.
    pub fn add_mood(
        &mut self,
        selector: &str,
        name: &str,
        value: i64,
        sid: Option<i64>,
        cid: Option<i64>,
    ) -> Result<(String, String, i64, i64)> {
        const RESERVED_SIDS: [i64; 4] = [1, 776, 777, 778];

        // Validate the target is a LightController2 and gather existing moods
        // (immutable borrow) before mutating.
        let existing = self.list_moods(selector)?;
        if existing.iter().any(|m| m.name == name) {
            anyhow::bail!(
                "a mood named '{name}' already exists on this LightController2 — \
                 use 'config set-mood-color' to change its color"
            );
        }

        // Next free custom SID: max custom SID + 1 (base 2).
        let new_sid = match sid {
            Some(s) => s,
            None => existing
                .iter()
                .filter_map(|m| m.sid)
                .filter(|s| !RESERVED_SIDS.contains(s))
                .max()
                .map(|m| m + 1)
                .unwrap_or(2),
        };
        // Next free custom CID: max custom CID + 1 (base 9).
        let new_cid = match cid {
            Some(c) => c,
            None => existing
                .iter()
                .filter(|m| m.sid.map(|s| !RESERVED_SIDS.contains(&s)).unwrap_or(true))
                .filter_map(|m| m.cid)
                .max()
                .map(|m| m + 1)
                .unwrap_or(9),
        };
        if existing.iter().any(|m| m.sid == Some(new_sid)) {
            anyhow::bail!("SID {new_sid} is already in use on this LightController2");
        }

        let serial = self.find_miniserver_serial().unwrap_or_default();
        let uuid = Self::loxone_uuid(&serial);

        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);

        // Locate the <LightscenesC> container, read its Outputs count, append
        // the new scene and bump Num.
        fn append_scene(
            elem: &mut Element,
            name: &str,
            uuid: &str,
            sid: i64,
            cid: i64,
            value: i64,
        ) -> bool {
            if elem.name == "LightscenesC" {
                let outputs: usize = elem
                    .attributes
                    .get("Outputs")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(18);
                let mut scene = Element::new("LightsceneC");
                scene
                    .attributes
                    .insert("Name".to_string(), name.to_string());
                scene
                    .attributes
                    .insert("UUID".to_string(), uuid.to_string());
                scene
                    .attributes
                    .insert("Outputs".to_string(), outputs.to_string());
                scene.attributes.insert("SID".to_string(), sid.to_string());
                scene.attributes.insert("CID".to_string(), cid.to_string());
                for q in 1..=outputs.max(1) {
                    let v = if q == 1 {
                        value.to_string()
                    } else {
                        "0".to_string()
                    };
                    scene.attributes.insert(format!("Q{q}"), v);
                }
                elem.children.push(xmltree::XMLNode::Element(scene));
                let num: i64 = elem
                    .attributes
                    .get("Num")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                elem.attributes
                    .insert("Num".to_string(), (num + 1).to_string());
                return true;
            }
            for child in &mut elem.children {
                if let Some(e) = child.as_mut_element()
                    && append_scene(e, name, uuid, sid, cid, value)
                {
                    return true;
                }
            }
            false
        }

        if append_scene(elem, name, &uuid, new_sid, new_cid, value) {
            Ok((name.to_string(), uuid, new_sid, new_cid))
        } else {
            anyhow::bail!(
                "could not find a <LightscenesC> scene container on LightController2 '{selector}'"
            )
        }
    }

    /// Find which Page contains a connector UUID.
    fn find_page_for_connector(&self, connector_uuid: &str) -> Option<String> {
        fn search(elem: &Element, target: &str, current_page: &Option<String>) -> Option<String> {
            let page = if elem.name == "C"
                && elem
                    .attributes
                    .get("Type")
                    .map(|t| t == "Page")
                    .unwrap_or(false)
            {
                elem.attributes.get("U").cloned()
            } else {
                current_page.clone()
            };

            // Check connectors on this element
            for child in &elem.children {
                if let Some(co) = child.as_element()
                    && co.name == "Co"
                    && co.attributes.get("U").map(|u| u == target).unwrap_or(false)
                {
                    return page;
                }
            }
            // Recurse
            for child in &elem.children {
                if let Some(e) = child.as_element()
                    && let Some(p) = search(e, target, &page)
                {
                    return Some(p);
                }
            }
            None
        }
        search(&self.root, connector_uuid, &None)
    }

    /// Determine the direction of a connector by its UUID.
    /// Returns `Some(true)` if it is an output, `Some(false)` if an input/param,
    /// or `None` if no connector with that UUID exists in the file.
    fn connector_dir_by_uuid(&self, uuid: &str) -> Option<bool> {
        fn search(elem: &Element, target: &str) -> Option<bool> {
            if elem.name == "C" {
                let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
                for child in &elem.children {
                    if let Some(co) = child.as_element()
                        && co.name == "Co"
                        && co.attributes.get("U").map(|u| u == target).unwrap_or(false)
                    {
                        let key = co.attributes.get("K").cloned().unwrap_or_default();
                        let is_output = ConfigEditor::connector_io_type(&block_type, &key)
                            .map(|t| t == "O")
                            .unwrap_or_else(|| {
                                // Fall back to naming convention when the type is unknown.
                                key.starts_with('Q')
                                    || key.starts_with("AQ")
                                    || key.starts_with("Output")
                            });
                        return Some(is_output);
                    }
                }
            }
            for child in &elem.children {
                if let Some(e) = child.as_element()
                    && let Some(found) = search(e, target)
                {
                    return Some(found);
                }
            }
            None
        }
        search(&self.root, uuid)
    }
}
