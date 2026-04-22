use super::{ConfigEditor, ConnectorMap, remove_by_uuid};
use anyhow::Result;
use xmltree::Element;

impl ConfigEditor {
    /// Add a child element under a parent. Returns the generated UUID.
    #[allow(clippy::too_many_arguments)]
    pub fn add_element(
        &mut self,
        parent_selector: &str,
        element_type: &str,
        title: &str,
        gid: Option<&str>,
        room_uuid: Option<&str>,
        category_uuid: Option<&str>,
        properties: &[(&str, &str, &str)], // (name, value, type_code)
    ) -> Result<String> {
        let parent_path = self.require_one(parent_selector)?;
        let serial = self.find_miniserver_serial().unwrap_or_default();
        let uuid = Self::loxone_uuid(&serial);

        let mut elem = Element::new("C");
        elem.attributes
            .insert("Type".to_string(), element_type.to_string());
        elem.attributes.insert("V".to_string(), "175".to_string());
        elem.attributes.insert("U".to_string(), uuid.clone());
        elem.attributes
            .insert("Title".to_string(), title.to_string());
        elem.attributes
            .insert("WF".to_string(), "16384".to_string());

        // Auto-assign IName for types that require it (SPS registration key)
        if let Some(prefix) = Self::iname_prefix(element_type) {
            let iname = self.next_iname(prefix);
            elem.attributes.insert("IName".to_string(), iname);
        }

        // Auto-create connectors from embedded connector map (190 types, 2384 connectors)
        let connector_map = Self::connector_map();
        let connectors: Vec<String> = connector_map
            .get(element_type)
            .map(|(conns, _, _)| conns.clone())
            .unwrap_or_default();
        let defaults: std::collections::HashMap<String, String> = connector_map
            .get(element_type)
            .map(|(_, defs, _)| defs.clone())
            .unwrap_or_default();

        if !connectors.is_empty() {
            elem.attributes
                .insert("Nio".to_string(), connectors.len().to_string());
            for conn_key in &connectors {
                let mut co = Element::new("Co");
                co.attributes.insert("K".to_string(), conn_key.clone());
                co.attributes
                    .insert("U".to_string(), Self::loxone_uuid(&serial));
                if let Some(def_val) = defaults.get(conn_key) {
                    co.attributes.insert("Def".to_string(), def_val.clone());
                }
                elem.children.push(xmltree::XMLNode::Element(co));
            }
        } else if Self::iname_prefix(element_type).is_some() {
            // Generic I/O types without specific connector layout
            elem.attributes.insert("Nio".to_string(), "1".to_string());
        }

        // Type-specific attributes
        if element_type == "StateV" {
            elem.attributes
                .insert("Analog".to_string(), "true".to_string());
            elem.attributes
                .insert("Tx".to_string(), "false".to_string());
            // IoData Visu=true makes it readable via HTTP API
            let mut iodata = Element::new("IoData");
            iodata
                .attributes
                .insert("Visu".to_string(), "true".to_string());
            elem.children.push(xmltree::XMLNode::Element(iodata));
            // Display element
            let mut display = Element::new("Display");
            display
                .attributes
                .insert("Unit".to_string(), "<v>".to_string());
            display
                .attributes
                .insert("StateOnly".to_string(), "true".to_string());
            elem.children.push(xmltree::XMLNode::Element(display));
        }

        // VirtualState/VirtualOut need IoData + Display for I/O registration
        if element_type == "VirtualState" || element_type == "VirtualOut" {
            let mut iodata = Element::new("IoData");
            iodata.attributes.insert("St".to_string(), "1".to_string());
            elem.children.push(xmltree::XMLNode::Element(iodata));
            let mut display = Element::new("Display");
            display
                .attributes
                .insert("Unit".to_string(), "<v>".to_string());
            display
                .attributes
                .insert("StateOnly".to_string(), "true".to_string());
            elem.children.push(xmltree::XMLNode::Element(display));
        }

        if let Some(g) = gid {
            elem.attributes.insert("gid".to_string(), g.to_string());
        }

        // Add IoData if room or category specified
        if room_uuid.is_some() || category_uuid.is_some() {
            let mut iodata = Element::new("IoData");
            if let Some(r) = room_uuid {
                iodata.attributes.insert("Pr".to_string(), r.to_string());
            }
            if let Some(c) = category_uuid {
                iodata.attributes.insert("Cr".to_string(), c.to_string());
            }
            elem.children.push(xmltree::XMLNode::Element(iodata));
        }

        // Add properties
        if !properties.is_empty() {
            let mut set = Element::new("SET");
            for (name, value, type_code) in properties {
                let mut prop = Element::new(name);
                prop.attributes
                    .insert("t".to_string(), type_code.to_string());
                prop.attributes.insert("v".to_string(), value.to_string());
                set.children.push(xmltree::XMLNode::Element(prop));
            }
            elem.children.push(xmltree::XMLNode::Element(set));
        }

        let parent = self.get_element_mut(&parent_path);
        parent.children.push(xmltree::XMLNode::Element(elem));

        Ok(uuid)
    }

    /// Add an element directly to the root. Returns the generated UUID.
    pub fn add_element_to_root(
        &mut self,
        element_type: &str,
        title: &str,
        room_uuid: Option<&str>,
        category_uuid: Option<&str>,
        properties: &[(&str, &str, &str)],
    ) -> Result<String> {
        let serial = self.find_miniserver_serial().unwrap_or_default();
        let uuid = Self::loxone_uuid(&serial);

        let mut elem = Element::new("C");
        elem.attributes
            .insert("Type".to_string(), element_type.to_string());
        elem.attributes.insert("V".to_string(), "175".to_string());
        elem.attributes.insert("U".to_string(), uuid.clone());
        elem.attributes
            .insert("Title".to_string(), title.to_string());
        elem.attributes
            .insert("WF".to_string(), "16384".to_string());

        if room_uuid.is_some() || category_uuid.is_some() {
            let mut iodata = Element::new("IoData");
            if let Some(r) = room_uuid {
                iodata.attributes.insert("Pr".to_string(), r.to_string());
            }
            if let Some(c) = category_uuid {
                iodata.attributes.insert("Cr".to_string(), c.to_string());
            }
            elem.children.push(xmltree::XMLNode::Element(iodata));
        }

        if !properties.is_empty() {
            let mut set = Element::new("SET");
            for (name, value, type_code) in properties {
                let mut prop = Element::new(name);
                prop.attributes
                    .insert("t".to_string(), type_code.to_string());
                prop.attributes.insert("v".to_string(), value.to_string());
                set.children.push(xmltree::XMLNode::Element(prop));
            }
            elem.children.push(xmltree::XMLNode::Element(set));
        }

        // Auto-create connectors from embedded connector map
        let connector_map = Self::connector_map();
        if let Some((connectors, defaults, _)) = connector_map.get(element_type)
            && !connectors.is_empty()
        {
            elem.attributes
                .insert("Nio".to_string(), connectors.len().to_string());
            for conn_key in connectors {
                let mut co = Element::new("Co");
                co.attributes.insert("K".to_string(), conn_key.clone());
                co.attributes
                    .insert("U".to_string(), Self::loxone_uuid(&serial));
                if let Some(def_val) = defaults.get(conn_key) {
                    co.attributes.insert("Def".to_string(), def_val.clone());
                }
                elem.children.push(xmltree::XMLNode::Element(co));
            }
        }

        // Auto-assign IName for types that require it
        if let Some(prefix) = Self::iname_prefix(element_type) {
            let iname = self.next_iname(prefix);
            elem.attributes.insert("IName".to_string(), iname);
        }

        self.root.children.push(xmltree::XMLNode::Element(elem));
        Ok(uuid)
    }

    /// Remove an element by UUID.
    pub fn remove_element(&mut self, uuid: &str) -> Result<String> {
        remove_by_uuid(&mut self.root.children, uuid)
    }

    /// Create a VirtualIn element under the given parent. Returns the block UUID.
    pub fn add_virtual_in(
        &mut self,
        title: &str,
        analog: bool,
        parent_selector: &str,
    ) -> Result<String> {
        let parent_path = self.require_one(parent_selector)?;

        // Extract Miniserver serial from existing UUIDs (last 12 hex chars after "ffff")
        let serial = self.find_miniserver_serial().unwrap_or_default();

        // Generate Loxone-format UUIDs: {random}-ffff{serial}
        let block_uuid = Self::loxone_uuid(&serial);
        let conn_uuid = Self::loxone_uuid(&serial);
        let conn_qm_uuid = Self::loxone_uuid(&serial);

        // Find next free IName index (VI0, VI1, VI2, ...)
        let iname = self.next_iname("VI");

        // Sanitize title to create a safe VIName (lowercase, underscores)
        let _vi_name: String = title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect();

        let mut elem = Element::new("C");
        elem.attributes
            .insert("Type".to_string(), "VirtualIn".to_string());
        elem.attributes.insert("IName".to_string(), iname);
        elem.attributes.insert("V".to_string(), "175".to_string());
        elem.attributes.insert("U".to_string(), block_uuid.clone());
        elem.attributes
            .insert("Title".to_string(), title.to_string());
        elem.attributes
            .insert("Cl".to_string(), "238,238,238".to_string());
        elem.attributes
            .insert("Analog".to_string(), analog.to_string());
        elem.attributes.insert("Nio".to_string(), "2".to_string());
        elem.attributes
            .insert("WF".to_string(), "16400".to_string());
        elem.attributes.insert("Idx".to_string(), "-1".to_string());
        elem.attributes.insert("ValOT".to_string(), "1".to_string());
        if analog {
            elem.attributes
                .insert("EnVal".to_string(), "true".to_string());
            elem.attributes
                .insert("MinChange".to_string(), "0.25".to_string());
            elem.attributes
                .insert("MinTime".to_string(), "1000".to_string());
            elem.attributes
                .insert("MaxVal".to_string(), "1000".to_string());
        }

        // Add Q connector
        let mut co_q = Element::new("Co");
        co_q.attributes.insert("K".to_string(), "Q".to_string());
        co_q.attributes.insert("U".to_string(), conn_uuid.clone());
        elem.children.push(xmltree::XMLNode::Element(co_q));

        // Add Qm connector (memory/state tracking)
        let mut co_qm = Element::new("Co");
        co_qm.attributes.insert("K".to_string(), "Qm".to_string());
        co_qm
            .attributes
            .insert("U".to_string(), conn_qm_uuid.clone());
        elem.children.push(xmltree::XMLNode::Element(co_qm));

        // Add IoData — St=2 for analog I/O registration, Visu=true for API visibility
        let mut iodata = Element::new("IoData");
        iodata.attributes.insert("St".to_string(), "2".to_string());
        iodata
            .attributes
            .insert("Visu".to_string(), "true".to_string());
        // Room/category from parent's IoData if available, else find first room
        let parent_elem = self.get_element(&parent_path);
        if let Some(parent_iodata) = parent_elem
            .children
            .iter()
            .find_map(|c| c.as_element().filter(|e| e.name == "IoData"))
        {
            if let Some(pr) = parent_iodata.attributes.get("Pr") {
                iodata.attributes.insert("Pr".to_string(), pr.clone());
            }
            if let Some(cr) = parent_iodata.attributes.get("Cr") {
                iodata.attributes.insert("Cr".to_string(), cr.clone());
            }
        }
        // Fallback: find first room and category from the config
        if !iodata.attributes.contains_key("Pr") {
            fn find_first(elem: &Element, type_name: &str) -> Option<String> {
                if elem.name == "C"
                    && elem
                        .attributes
                        .get("Type")
                        .map(|t| t == type_name)
                        .unwrap_or(false)
                {
                    return elem.attributes.get("U").cloned();
                }
                for child in &elem.children {
                    if let Some(e) = child.as_element()
                        && let Some(u) = find_first(e, type_name)
                    {
                        return Some(u);
                    }
                }
                None
            }
            if let Some(room) = find_first(&self.root, "Place") {
                iodata.attributes.insert("Pr".to_string(), room);
            }
            if let Some(cat) = find_first(&self.root, "Category") {
                iodata.attributes.insert("Cr".to_string(), cat);
            }
        }
        elem.children.push(xmltree::XMLNode::Element(iodata));

        // Add Display
        let mut display = Element::new("Display");
        display
            .attributes
            .insert("Unit".to_string(), "<v>".to_string());
        display
            .attributes
            .insert("StateOnly".to_string(), "true".to_string());
        elem.children.push(xmltree::XMLNode::Element(display));

        // Add InputRef converter (required for wiring to blocks).
        // Blocks wire from InputRef.AQ, not directly from VirtualIn.Q.
        let ref_uuid = Self::loxone_uuid(&serial);
        let ref_ai_uuid = Self::loxone_uuid(&serial);
        let ref_i_uuid = Self::loxone_uuid(&serial);
        let ref_aq_uuid = Self::loxone_uuid(&serial);
        let ref_q_uuid = Self::loxone_uuid(&serial);

        let mut input_ref = Element::new("C");
        input_ref
            .attributes
            .insert("Type".to_string(), "InputRef".to_string());
        input_ref
            .attributes
            .insert("V".to_string(), "175".to_string());
        input_ref.attributes.insert("U".to_string(), ref_uuid);
        input_ref
            .attributes
            .insert("Title".to_string(), title.to_string());
        input_ref
            .attributes
            .insert("Ref".to_string(), block_uuid.clone());
        input_ref
            .attributes
            .insert("LinkRefType".to_string(), "71".to_string());
        input_ref
            .attributes
            .insert("Analog".to_string(), analog.to_string());
        input_ref
            .attributes
            .insert("Nio".to_string(), "4".to_string());
        input_ref
            .attributes
            .insert("WF".to_string(), "18432".to_string());

        // AI ← VirtualIn.Q
        let mut ref_co_ai = Element::new("Co");
        ref_co_ai
            .attributes
            .insert("K".to_string(), "AI".to_string());
        ref_co_ai.attributes.insert("U".to_string(), ref_ai_uuid);
        let mut ai_in = Element::new("In");
        ai_in.attributes.insert("Input".to_string(), conn_uuid);
        ref_co_ai.children.push(xmltree::XMLNode::Element(ai_in));
        input_ref
            .children
            .push(xmltree::XMLNode::Element(ref_co_ai));

        // I ← VirtualIn.Qm
        let mut ref_co_i = Element::new("Co");
        ref_co_i.attributes.insert("K".to_string(), "I".to_string());
        ref_co_i.attributes.insert("U".to_string(), ref_i_uuid);
        let mut i_in = Element::new("In");
        i_in.attributes.insert("Input".to_string(), conn_qm_uuid);
        ref_co_i.children.push(xmltree::XMLNode::Element(i_in));
        input_ref.children.push(xmltree::XMLNode::Element(ref_co_i));

        // AQ (output — blocks wire FROM this)
        let mut ref_co_aq = Element::new("Co");
        ref_co_aq
            .attributes
            .insert("K".to_string(), "AQ".to_string());
        ref_co_aq
            .attributes
            .insert("U".to_string(), ref_aq_uuid.clone());
        input_ref
            .children
            .push(xmltree::XMLNode::Element(ref_co_aq));

        // Q
        let mut ref_co_q = Element::new("Co");
        ref_co_q.attributes.insert("K".to_string(), "Q".to_string());
        ref_co_q.attributes.insert("U".to_string(), ref_q_uuid);
        input_ref.children.push(xmltree::XMLNode::Element(ref_co_q));

        // DO NOT embed InputRef as child of VirtualIn — the Miniserver SPS
        // refuses to register VirtualIns that have InputRef children.
        // InputRef goes on the Page only.

        let parent = self.get_element_mut(&parent_path);
        parent.children.push(xmltree::XMLNode::Element(elem));

        // Place InputRef on the Page so the SPS circuit resolver can find it.
        let page_paths = self.find_elements("Type:Page");
        if let Some(page_path) = page_paths.into_iter().next() {
            let page = self.get_element_mut(&page_path);
            page.children.push(xmltree::XMLNode::Element(input_ref));
        }

        // Return InputRef AQ UUID — this is what blocks should wire from
        Ok(ref_aq_uuid)
    }

    /// Load the embedded connector map (190 types, 2384 connectors from TechDoc + real XML).
    /// Returns: { "LxType" => (vec!["conn1", "conn2"], { "conn1" => "default_val" }) }
    /// Load embedded connector map with types.
    /// Returns: { "LxType" => (connectors, defaults, io_types) }
    /// io_types maps connector name → "I" (Input), "O" (Output), "P" (Parameter)
    pub fn connector_map() -> ConnectorMap {
        let json_str = include_str!("../../docs/schemas/connector-map.json");
        let raw: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(json_str).unwrap_or_default();

        let mut map = std::collections::HashMap::new();
        for (lx_type, val) in raw {
            let conns: Vec<String> = val
                .get("c")
                .and_then(|c| serde_json::from_value(c.clone()).ok())
                .unwrap_or_default();
            let defs: std::collections::HashMap<String, String> = val
                .get("d")
                .and_then(|d| serde_json::from_value(d.clone()).ok())
                .unwrap_or_default();
            let types: std::collections::HashMap<String, String> = val
                .get("t")
                .and_then(|t| serde_json::from_value(t.clone()).ok())
                .unwrap_or_default();
            map.insert(lx_type, (conns, defs, types));
        }
        map
    }

    /// Get the I/O type of a connector: "I" (Input), "O" (Output), "P" (Parameter)
    pub fn connector_io_type(block_type: &str, connector_key: &str) -> Option<String> {
        let map = Self::connector_map();
        map.get(block_type)
            .and_then(|(_, _, types)| types.get(connector_key).cloned())
    }

    /// Extract Miniserver serial from existing UUIDs in the config.
    /// Looks for UUIDs with "ffff" pattern and extracts the 12-char serial suffix.
    pub fn find_miniserver_serial(&self) -> Option<String> {
        fn scan(elem: &Element) -> Option<String> {
            if let Some(u) = elem.attributes.get("U")
                && let Some(pos) = u.find("ffff")
            {
                let serial = &u[pos + 4..];
                if serial.len() >= 12 && serial.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(serial[..12].to_string());
                }
            }
            for child in &elem.children {
                if let xmltree::XMLNode::Element(c) = child
                    && let Some(s) = scan(c)
                {
                    return Some(s);
                }
            }
            None
        }
        scan(&self.root)
    }

    /// Generate a Loxone-format UUID: {8hex}-{4hex}-{4hex}-ffff{12hex_serial}
    pub fn loxone_uuid(serial: &str) -> String {
        let r: u64 = rand::random();
        let suffix = if serial.is_empty() {
            "000000000000".to_string()
        } else {
            serial.to_string()
        };
        format!(
            "{:08x}-{:04x}-{:04x}-ffff{}",
            (r >> 32) as u32,
            (r >> 16) as u16,
            r as u16,
            suffix,
        )
    }

    /// Find the next free IName index (e.g. "VI" → "VI0", "VI1", ...).
    pub fn next_iname(&self, prefix: &str) -> String {
        let mut max_idx: i32 = -1;
        fn scan(elem: &Element, prefix: &str, max: &mut i32) {
            if let Some(iname) = elem.attributes.get("IName")
                && iname.starts_with(prefix)
                && let Ok(idx) = iname[prefix.len()..].parse::<i32>()
                && idx > *max
            {
                *max = idx;
            }
            for child in &elem.children {
                if let xmltree::XMLNode::Element(c) = child {
                    scan(c, prefix, max);
                }
            }
        }
        scan(&self.root, prefix, &mut max_idx);
        format!("{}{}", prefix, max_idx + 1)
    }

    /// Return the IName prefix for element types that require hardware registration.
    /// Returns None for types that don't use IName.
    pub fn iname_prefix(element_type: &str) -> Option<&'static str> {
        match element_type {
            "VirtualIn" => Some("VI"),
            "VirtualOut" | "VirtualState" => Some("VO"),
            "DigitalIn" | "TreeSensor" | "LoxAIRsensor" => Some("I"),
            "Actor" | "TreeActor" | "LoxAIRactor" => Some("Q"),
            "VoltageIn" | "TreeAsensor" | "LoxAIRAsensor" => Some("AI"),
            "TreeAactor" | "LoxAIRAactor" => Some("AQ"),
            "SysVar" => Some("SYS"),
            "WeatherData" => Some("WDC"),
            "Online" => Some("S"),
            // Logic/math blocks also need Nio but NOT IName
            _ => None,
        }
    }
}
