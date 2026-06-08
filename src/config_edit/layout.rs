use super::ConfigEditor;
use anyhow::{Context, Result, bail};

/// Canvas size (width, height) in Loxone layout units for a block type.
/// Used both by the auto-layout passes and by `add_element` to emit sensible
/// default `Px2`/`Py2` so Loxone Config doesn't "repair" coordinate-less blocks.
pub(crate) fn block_size(block_type: &str) -> (i32, i32) {
    match block_type {
        "InputRef" | "OutputRef" | "Memory" => (2112, 192),
        "LightController2" => (2688, 1848),
        "AlarmClock" => (2688, 1272),
        "PresenceDetector" | "Presence" => (2688, 1272),
        "Thermostat" | "JalousieUpDown2" | "Ventilation2" => (2688, 1272),
        _ => (1344, 696),
    }
}

impl ConfigEditor {
    /// Auto-layout blocks on a Page using ELK (Eclipse Layout Kernel).
    #[allow(dead_code)]
    pub fn elk_layout(&mut self, page_selector: &str) -> Result<usize> {
        use serde_json::{Value, json};
        use std::process::Command;

        // Find the page
        let page_paths = self.find_elements(page_selector);
        let page_path = page_paths
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No page found matching '{}'", page_selector))?;

        let page = self.get_element(&page_path);

        // Block sizes
        let wide_types = ["InputRef", "OutputRef", "StateV", "VirtualState"];
        let wide_w = 2112.0;
        let wide_h = 500.0;
        let block_w = 1344.0;
        let block_h = 696.0;

        // Build ELK graph
        let mut elk_nodes = Vec::new();
        let mut elk_edges = Vec::new();
        let mut uuid_to_idx: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut connector_to_node: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for (i, child) in page.children.iter().enumerate() {
            let elem = match child.as_element() {
                Some(e) if e.name == "C" => e,
                _ => continue,
            };
            let uuid = match elem.attributes.get("U") {
                Some(u) => u.clone(),
                None => continue,
            };
            let elem_type = elem.attributes.get("Type").cloned().unwrap_or_default();
            if elem_type.is_empty() {
                continue;
            }

            let (w, h) = if wide_types.contains(&elem_type.as_str()) {
                (wide_w, wide_h)
            } else {
                (block_w, block_h)
            };

            // Map connector UUIDs to this node
            for co in &elem.children {
                if let Some(co_elem) = co.as_element()
                    && co_elem.name == "Co"
                    && let Some(co_uuid) = co_elem.attributes.get("U")
                {
                    connector_to_node.insert(co_uuid.clone(), uuid.clone());
                }
            }

            elk_nodes.push(json!({
                "id": uuid,
                "width": w,
                "height": h,
            }));
            uuid_to_idx.insert(uuid, i);
        }

        // Build edges from <In Input="..."/> elements
        let mut edge_id = 0;
        let page = self.get_element(&page_path);
        for child in &page.children {
            let elem = match child.as_element() {
                Some(e) if e.name == "C" => e,
                _ => continue,
            };
            let target_uuid = match elem.attributes.get("U") {
                Some(u) => u.clone(),
                None => continue,
            };

            for co in &elem.children {
                if let Some(co_elem) = co.as_element()
                    && co_elem.name == "Co"
                {
                    for inp in &co_elem.children {
                        if let Some(in_elem) = inp.as_element()
                            && in_elem.name == "In"
                            && let Some(src_co_uuid) = in_elem.attributes.get("Input")
                            && let Some(src_node_uuid) = connector_to_node.get(src_co_uuid)
                            && uuid_to_idx.contains_key(src_node_uuid)
                        {
                            elk_edges.push(json!({
                                "id": format!("e{}", edge_id),
                                "sources": [src_node_uuid],
                                "targets": [target_uuid],
                            }));
                            edge_id += 1;
                        }
                    }
                }
            }
        }

        let elk_graph = json!({
            "id": "root",
            "layoutOptions": {
                "elk.algorithm": "layered",
                "elk.direction": "RIGHT",
                "elk.spacing.nodeNode": "50",
                "elk.layered.spacing.nodeNodeBetweenLayers": "100",
                "elk.spacing.edgeNode": "30",
                "elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX"
            },
            "children": elk_nodes,
            "edges": elk_edges,
        });

        // Find elk-layout.js script
        let script = std::env::current_exe()?
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("../scripts/elk-layout.js");
        let script = if script.exists() {
            script
        } else {
            // Try relative to cwd
            std::path::PathBuf::from("scripts/elk-layout.js")
        };

        // Call ELK
        let output = Command::new("node")
            .arg(&script)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(serde_json::to_string(&elk_graph)?.as_bytes())?;
                }
                child.wait_with_output()
            })
            .context("Failed to run ELK layout (node scripts/elk-layout.js)")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("ELK layout failed: {}", stderr);
        }

        let positions: std::collections::HashMap<String, Value> =
            serde_json::from_slice(&output.stdout).context("Failed to parse ELK output")?;

        // Apply positions — scale ELK coordinates to Loxone units
        // ELK uses the sizes we provided, so coordinates are already in Loxone units
        let page = self.get_element_mut(&page_path);
        let mut count = 0;
        for child in &mut page.children {
            if let Some(elem) = child.as_mut_element()
                && let Some(uuid) = elem.attributes.get("U").cloned()
                && let Some(pos) = positions.get(&uuid)
            {
                let x = pos["x"].as_f64().unwrap_or(0.0) as i64;
                let y = pos["y"].as_f64().unwrap_or(0.0) as i64;
                let w = pos["width"].as_f64().unwrap_or(block_w) as i64;
                let h = pos["height"].as_f64().unwrap_or(block_h) as i64;

                elem.attributes.insert("Px".to_string(), x.to_string());
                elem.attributes.insert("Py".to_string(), y.to_string());
                elem.attributes
                    .insert("Px2".to_string(), (x + w).to_string());
                elem.attributes
                    .insert("Py2".to_string(), (y + h).to_string());

                if !elem.attributes.contains_key("Cl") {
                    let elem_type = elem.attributes.get("Type").cloned().unwrap_or_default();
                    let cl = if elem_type == "StateV" {
                        "141,255,112"
                    } else {
                        "0,0,0"
                    };
                    elem.attributes.insert("Cl".to_string(), cl.to_string());
                }
                count += 1;
            }
        }

        Ok(count)
    }

    /// Layout blocks on a Page using the Loxone UX Ausrichten grid pattern.
    /// 3 columns: inputs (X=4320), controllers (X=7392), outputs (X=11040).
    /// Block sizes match UX exactly. Blocks stack vertically in each column.
    pub fn grid_layout(&mut self, page_selector: &str) -> Result<usize> {
        let page_paths = self.find_elements(page_selector);
        let page_path = page_paths
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No page found matching '{}'", page_selector))?;

        let input_types = ["InputRef"];
        let output_types = ["OutputRef", "Memory"];

        let page = self.get_element(&page_path);
        let mut left: Vec<(usize, i32, i32)> = Vec::new();
        let mut center: Vec<(usize, i32, i32, String)> = Vec::new();
        let mut right: Vec<(usize, i32, i32)> = Vec::new();

        for (i, child) in page.children.iter().enumerate() {
            if let Some(elem) = child.as_element()
                && elem.name == "C"
                && let Some(block_type) = elem.attributes.get("Type")
            {
                let (w, h) = block_size(block_type);
                if input_types.contains(&block_type.as_str()) {
                    left.push((i, w, h));
                } else if output_types.contains(&block_type.as_str()) {
                    right.push((i, w, h));
                } else {
                    center.push((i, w, h, block_type.clone()));
                }
            }
        }

        // Sort center by type priority: LightController2 > AlarmClock > others
        fn type_priority(t: &str) -> u8 {
            match t {
                "LightController2" => 0,
                "AlarmClock" => 1,
                "Thermostat" | "JalousieUpDown2" => 2,
                "PresenceDetector" | "Presence" => 3,
                _ => 4,
            }
        }
        center.sort_by_key(|(_, _, _, t)| type_priority(t));

        let y_start = 576;
        let gap_y = 168;
        let mut count = 0;

        // Align InputRefs vertically with first center block
        let center_offset = if !center.is_empty() {
            (center[0].2 - 192) / 2
        } else {
            0
        };

        // Left column (X=4320)
        let mut y = y_start + center_offset;
        for &(idx, w, h) in &left {
            let page = self.get_element_mut(&page_path);
            if let Some(elem) = page.children[idx].as_mut_element() {
                elem.attributes.insert("Px".to_string(), "4320".to_string());
                elem.attributes.insert("Py".to_string(), y.to_string());
                elem.attributes
                    .insert("Px2".to_string(), (4320 + w).to_string());
                elem.attributes
                    .insert("Py2".to_string(), (y + h).to_string());
                count += 1;
            }
            y += h + gap_y;
        }

        // Center column (X=7392)
        y = y_start;
        for &(idx, w, h, _) in &center {
            let page = self.get_element_mut(&page_path);
            if let Some(elem) = page.children[idx].as_mut_element() {
                elem.attributes.insert("Px".to_string(), "7392".to_string());
                elem.attributes.insert("Py".to_string(), y.to_string());
                elem.attributes
                    .insert("Px2".to_string(), (7392 + w).to_string());
                elem.attributes
                    .insert("Py2".to_string(), (y + h).to_string());
                count += 1;
            }
            y += h + gap_y;
        }

        // Right column (X=11040)
        y = y_start + center_offset;
        for &(idx, w, h) in &right {
            let page = self.get_element_mut(&page_path);
            if let Some(elem) = page.children[idx].as_mut_element() {
                elem.attributes
                    .insert("Px".to_string(), "11040".to_string());
                elem.attributes.insert("Py".to_string(), y.to_string());
                elem.attributes
                    .insert("Px2".to_string(), (11040 + w).to_string());
                elem.attributes
                    .insert("Py2".to_string(), (y + h).to_string());
                count += 1;
            }
            y += h + gap_y;
        }

        Ok(count)
    }
}
