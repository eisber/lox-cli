use super::{ConfigEditor, ConnectorMap, remove_by_uuid};
use anyhow::Result;
use xmltree::Element;

/// Options for [`ConfigEditor::add_virtual_in`].
#[derive(Default, Clone)]
pub struct VirtualInOpts {
    /// Analog (true) vs digital (false) Virtual Input.
    pub analog: bool,
    /// Optional minimum value (analog only). When unset, no `MinVal` is emitted.
    pub min: Option<f64>,
    /// Optional maximum value (analog only). When unset, **no** `MaxVal` is
    /// emitted so the input does not silently clamp (e.g. a color composite).
    pub max: Option<f64>,
    /// Optional display unit format string (e.g. `"<v>"`, `"<v.1>°C"`).
    pub unit: Option<String>,
}

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

        // Auto-create connectors from embedded connector map (190 types, 2384 connectors).
        // Loxone Config regenerates connectors (with fresh UUIDs, dropping wires) for
        // blocks whose connector set/order is non-canonical, so emit them inputs/params
        // first, then outputs — the order the desktop editor expects.
        let connector_map = Self::connector_map();
        let raw_connectors: Vec<String> = connector_map
            .get(element_type)
            .map(|(conns, _, _)| conns.clone())
            .unwrap_or_default();
        let defaults: std::collections::HashMap<String, String> = connector_map
            .get(element_type)
            .map(|(_, defs, _)| defs.clone())
            .unwrap_or_default();
        let types: std::collections::HashMap<String, String> = connector_map
            .get(element_type)
            .map(|(_, _, t)| t.clone())
            .unwrap_or_default();
        let connectors = Self::order_connectors(&raw_connectors, &types);

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

        // Emit default canvas coordinates so Loxone Config does not "repair" a
        // coordinate-less block on first save (which regenerates connectors and
        // drops wires). Blocks are stacked vertically on the parent page; run
        // `lox config layout` afterwards for a tidy arrangement.
        let (px, py) = self.next_block_origin(&parent_path);
        let (w, h) = super::layout::block_size(element_type);
        elem.attributes.insert("Px".to_string(), px.to_string());
        elem.attributes.insert("Py".to_string(), py.to_string());
        elem.attributes
            .insert("Px2".to_string(), (px + w).to_string());
        elem.attributes
            .insert("Py2".to_string(), (py + h).to_string());
        elem.attributes
            .entry("Cl".to_string())
            .or_insert_with(|| "0,0,0".to_string());

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

        // Auto-create connectors from embedded connector map (inputs/params first)
        let connector_map = Self::connector_map();
        if let Some((connectors, defaults, types)) = connector_map.get(element_type)
            && !connectors.is_empty()
        {
            let ordered = Self::order_connectors(connectors, types);
            elem.attributes
                .insert("Nio".to_string(), ordered.len().to_string());
            for conn_key in &ordered {
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

    /// Create a VirtualIn element under the given parent.
    ///
    /// Returns `(virtual_in_uuid, input_ref_aq_uuid)`: the VirtualIn element's own UUID
    /// (what user permissions, `PRM REF`, refer to) and the UUID of its InputRef's AQ
    /// output (what blocks wire from).
    pub fn add_virtual_in(
        &mut self,
        title: &str,
        opts: &VirtualInOpts,
        parent_selector: &str,
    ) -> Result<(String, String)> {
        let analog = opts.analog;
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
            if let Some(min) = opts.min {
                elem.attributes.insert("MinVal".to_string(), fmt_num(min));
            }
            // Only emit MaxVal when explicitly requested — a default MaxVal
            // silently clamps values such as a color composite.
            if let Some(max) = opts.max {
                elem.attributes.insert("MaxVal".to_string(), fmt_num(max));
            }
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
            for reference in ["Pr", "Cr"] {
                if let Some(value) = parent_iodata
                    .attributes
                    .get(reference)
                    .filter(|value| !value.is_empty())
                {
                    iodata
                        .attributes
                        .insert(reference.to_string(), value.clone());
                }
            }
        }
        // Fallback: find first room and category from the config
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
        for (reference, type_name) in [("Pr", "Place"), ("Cr", "Category")] {
            if !iodata.attributes.contains_key(reference)
                && let Some(uuid) = find_first(&self.root, type_name)
            {
                iodata.attributes.insert(reference.to_string(), uuid);
            }
        }
        elem.children.push(xmltree::XMLNode::Element(iodata));

        // Add Display
        let mut display = Element::new("Display");
        let unit = opts.unit.as_deref().unwrap_or("<v>");
        display
            .attributes
            .insert("Unit".to_string(), unit.to_string());
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

        // InputRef AQ UUID is what blocks should wire from
        Ok((block_uuid, ref_aq_uuid))
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

    /// Order connector keys the way Loxone Config expects: inputs and parameters
    /// first (preserving their given order), then outputs. Connectors with an
    /// unknown type are treated as input-side. A non-canonical order makes the
    /// desktop editor regenerate connectors and silently drop attached wires.
    pub(crate) fn order_connectors(
        keys: &[String],
        types: &std::collections::HashMap<String, String>,
    ) -> Vec<String> {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for k in keys {
            if types.get(k).map(|s| s.as_str()) == Some("O") {
                outputs.push(k.clone());
            } else {
                inputs.push(k.clone());
            }
        }
        inputs.extend(outputs);
        inputs
    }

    /// Compute a default origin `(Px, Py)` for a new block on `parent_path`,
    /// stacking it below the lowest existing block so it doesn't overlap.
    fn next_block_origin(&self, parent_path: &[usize]) -> (i32, i32) {
        let parent = self.get_element(parent_path);
        let mut max_py2: Option<i32> = None;
        for child in &parent.children {
            if let Some(e) = child.as_element()
                && e.name == "C"
                && let Some(py2) = e.attributes.get("Py2").and_then(|s| s.parse::<i32>().ok())
            {
                max_py2 = Some(max_py2.map_or(py2, |m| m.max(py2)));
            }
        }
        // Center column (X=7392) matches grid_layout; stack with a 168-unit gap.
        let py = match max_py2 {
            Some(m) => m + 168,
            None => 576,
        };
        (7392, py)
    }

    /// Extract the existing 12-hex identity suffix after the final `-ffff`.
    /// The legacy name does not imply the suffix is a hardware serial.
    pub fn find_miniserver_serial(&self) -> Option<String> {
        fn scan(elem: &Element) -> Option<String> {
            if let Some(u) = elem.attributes.get("U")
                && let Some((_, serial)) = u.rsplit_once("-ffff")
                && serial.len() >= 12
                && serial.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Some(serial[..12].to_string());
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

/// Format an f64 without a trailing `.0` for whole numbers (Loxone prefers
/// `100` over `100.0` for integer-valued attributes).
fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn virtual_input_fixture(parent_iodata: &str) -> ConfigEditor {
        ConfigEditor::load(
            format!(
                r#"<ControlList NextObj="29" NextMem="1">
  <C Type="Document" U="ffffabcd-1234-5678-ffffaabbccddeeff">
    <C Type="Place" U="room-first"/>
    <C Type="Place" U="room-parent"/>
    <C Type="Category" U="cat-first"/>
    <C Type="Category" U="cat-parent"/>
    <C Type="VirtualInCaption" U="caption">{parent_iodata}
      <C Type="VirtualIn" U="old-vi" IName="VI7"/>
    </C>
    <C Type="Program" U="program">
      <C Type="Page" U="page" Title="Program"/>
    </C>
  </C>
</ControlList>"#
            )
            .as_bytes(),
        )
        .unwrap()
    }

    fn connector<'a>(element: &'a Element, key: &str) -> &'a Element {
        element
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .find(|child| child.name == "Co" && child.attributes.get("K").is_some_and(|k| k == key))
            .unwrap()
    }

    #[test]
    fn virtual_input_registration_inherits_each_reference_independently() {
        for (iodata, room, category) in [
            (
                r#"<IoData Pr="room-parent" Cr="cat-parent"/>"#,
                "room-parent",
                "cat-parent",
            ),
            (r#"<IoData Pr="room-parent"/>"#, "room-parent", "cat-first"),
            (r#"<IoData Cr="cat-parent"/>"#, "room-first", "cat-parent"),
            ("", "room-first", "cat-first"),
            (r#"<IoData Pr="" Cr=""/>"#, "room-first", "cat-first"),
            (
                r#"<IoData Pr="room-parent" Cr=""/>"#,
                "room-parent",
                "cat-first",
            ),
            (
                r#"<IoData Pr="" Cr="cat-parent"/>"#,
                "room-first",
                "cat-parent",
            ),
        ] {
            for analog in [false, true] {
                let mut editor = virtual_input_fixture(iodata);
                let (vi_uuid, aq_uuid) = editor
                    .add_virtual_in(
                        "New input",
                        &VirtualInOpts {
                            analog,
                            ..Default::default()
                        },
                        "uuid:caption",
                    )
                    .unwrap();
                let reloaded = ConfigEditor::load(&editor.to_bytes().unwrap()).unwrap();
                let vi_path = reloaded.require_one(&format!("uuid:{vi_uuid}")).unwrap();
                let vi = reloaded.get_element(&vi_path);
                let io = vi.get_child("IoData").unwrap();
                assert_eq!(io.attributes.get("Pr").unwrap(), room, "{iodata}");
                assert_eq!(io.attributes.get("Cr").unwrap(), category, "{iodata}");
                assert_eq!(vi.attributes["WF"], "16400");
                assert_eq!(vi.attributes["Idx"], "-1");
                assert_eq!(vi.attributes["IName"], "VI8");
                assert_eq!(vi.attributes["Analog"], analog.to_string());
                assert!(!vi.attributes.contains_key("MaxVal"));
                assert!(!vi.children.iter().any(|child| {
                    child
                        .as_element()
                        .is_some_and(|e| e.attributes.get("Type").is_some_and(|t| t == "InputRef"))
                }));

                let page = reloaded.get_element(&reloaded.require_one("Type:Page").unwrap());
                let input_ref = page
                    .children
                    .iter()
                    .filter_map(|child| child.as_element())
                    .find(|child| {
                        child
                            .attributes
                            .get("Type")
                            .is_some_and(|t| t == "InputRef")
                    })
                    .unwrap();
                assert_eq!(input_ref.attributes["Ref"], vi_uuid);
                assert_eq!(connector(input_ref, "AQ").attributes["U"], aq_uuid);
                for (input, output) in [("AI", "Q"), ("I", "Qm")] {
                    assert_eq!(
                        connector(input_ref, input)
                            .get_child("In")
                            .unwrap()
                            .attributes["Input"],
                        connector(vi, output).attributes["U"]
                    );
                }
                assert!(!reloaded.validate_config().iter().any(|line| {
                    line.starts_with('✗') && (line.contains("Pr=") || line.contains("Cr="))
                }));
            }
        }
    }

    #[test]
    fn identity_suffix_is_not_confused_with_ffff_in_uuid_prefix() {
        for uuid in [
            "ffffabcd-1234-5678-ffffaabbccddeeff",
            "01234567-ffff-5678-ffffaabbccddeeff",
            "01234567-1234-ffff-ffffaabbccddeeff",
            "01234567-1234-5678-ffffaabbffffeeff",
        ] {
            let editor = ConfigEditor::load(
                format!(r#"<ControlList><C U="{uuid}"/></ControlList>"#).as_bytes(),
            )
            .unwrap();
            let suffix = &uuid[uuid.len() - 12..];
            assert_eq!(editor.find_miniserver_serial().as_deref(), Some(suffix));
        }
    }

    #[test]
    fn generated_block_and_connector_identities_preserve_native_counters_and_suffix() {
        let mut editor = virtual_input_fixture("");
        let existing: HashSet<_> = ConfigEditor::iter_elements(&editor.root)
            .into_iter()
            .filter_map(|element| element.attributes.get("U").cloned())
            .collect();
        let existing_warnings: Vec<_> = editor
            .validate_config()
            .into_iter()
            .filter(|line| line.contains("malformed UUID"))
            .collect();
        editor
            .add_element("uuid:page", "And", "Logic", None, None, None, &[])
            .unwrap();
        editor
            .add_element_to_root("Or", "Root logic", None, None, &[])
            .unwrap();
        editor
            .add_virtual_in("New input", &VirtualInOpts::default(), "uuid:caption")
            .unwrap();
        let reloaded = ConfigEditor::load(&editor.to_bytes().unwrap()).unwrap();
        let mut identities = HashSet::new();
        let mut new_connectors = 0;
        for element in ConfigEditor::iter_elements(&reloaded.root) {
            let Some(uuid) = element.attributes.get("U") else {
                continue;
            };
            assert!(identities.insert(uuid), "duplicate identity: {uuid}");
            if !existing.contains(uuid) {
                assert!(uuid.ends_with("-ffffaabbccddeeff"), "{uuid}");
                let groups: Vec<_> = uuid.split('-').collect();
                assert_eq!(
                    groups.iter().map(|g| g.len()).collect::<Vec<_>>(),
                    [8, 4, 4, 16]
                );
                assert!(
                    groups
                        .iter()
                        .all(|g| g.chars().all(|c| c.is_ascii_hexdigit()))
                );
                if element.name == "Co" {
                    new_connectors += 1;
                }
            }
        }
        assert!(
            new_connectors > 6,
            "logic and VirtualIn connectors were checked"
        );
        assert_eq!(reloaded.root.attributes["NextObj"], "29");
        assert_eq!(reloaded.root.attributes["NextMem"], "1");
        assert_eq!(
            reloaded
                .validate_config()
                .into_iter()
                .filter(|line| line.contains("malformed UUID"))
                .collect::<Vec<_>>(),
            existing_warnings
        );
    }
}
