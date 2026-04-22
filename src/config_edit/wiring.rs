use super::{ConfigEditor, MqttTopic, WireInfo};
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
    pub fn wire_connector(
        &mut self,
        block_selector: &str,
        conn_key: &str,
        source_uuid: &str,
    ) -> Result<()> {
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

        // Add <In Input="source_uuid"/> child, with FLG="2" for cross-page wiring
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
}
