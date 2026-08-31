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

    /// Snap a coordinate to the 96-unit Loxone editor grid.
    fn snap96(v: i32) -> i32 {
        ((v as f64 / 96.0).round() as i32) * 96
    }

    /// Place ONLY blocks that have no canvas position yet (`Px` unset), leaving every
    /// already-positioned block untouched. Uses a Sugiyama-style layered layout so the
    /// new blocks flow left→right along their wiring, snapped to the 96-grid, anchored
    /// in the free area to the right of the page's existing content.
    ///
    /// This is the incremental counterpart to `grid_layout` (which re-arranges the whole
    /// page). Returns the number of blocks positioned.
    pub fn incremental_layout(&mut self, page_selector: &str) -> Result<usize> {
        use std::collections::{HashMap, HashSet};

        let page_paths = self.find_elements(page_selector);
        let page_path = page_paths
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No page found matching '{}'", page_selector))?;

        // --- Pass 1: read-only harvest of every block on the page ---
        struct Node {
            idx: usize,
            uuid: String,
            btype: String,
            has_px: bool,
            ins: Vec<String>, // source connector UUIDs this block consumes
        }
        let mut nodes: Vec<Node> = Vec::new();
        let mut conn_owner: HashMap<String, String> = HashMap::new(); // connector UUID -> block UUID
        // existing bounding box (only positioned blocks)
        let mut max_px2 = i32::MIN;
        let mut min_py = i32::MAX;

        let page = self.get_element(&page_path);
        for (i, child) in page.children.iter().enumerate() {
            let elem = match child.as_element() {
                Some(e) if e.name == "C" => e,
                _ => continue,
            };
            let uuid = match elem.attributes.get("U") {
                Some(u) => u.clone(),
                None => continue,
            };
            let btype = elem.attributes.get("Type").cloned().unwrap_or_default();
            if btype.is_empty() {
                continue;
            }
            let has_px = elem.attributes.contains_key("Px");
            if has_px {
                if let Some(px2) = elem.attributes.get("Px2").and_then(|v| v.parse::<i32>().ok()) {
                    max_px2 = max_px2.max(px2);
                } else if let Some(px) = elem.attributes.get("Px").and_then(|v| v.parse::<i32>().ok()) {
                    max_px2 = max_px2.max(px);
                }
                if let Some(py) = elem.attributes.get("Py").and_then(|v| v.parse::<i32>().ok()) {
                    min_py = min_py.min(py);
                }
            }
            let mut ins = Vec::new();
            for co in &elem.children {
                if let Some(co_elem) = co.as_element()
                    && co_elem.name == "Co"
                {
                    if let Some(cu) = co_elem.attributes.get("U") {
                        conn_owner.insert(cu.clone(), uuid.clone());
                    }
                    for inp in &co_elem.children {
                        if let Some(in_elem) = inp.as_element()
                            && in_elem.name == "In"
                            && let Some(src) = in_elem.attributes.get("Input")
                        {
                            ins.push(src.clone());
                        }
                    }
                }
            }
            nodes.push(Node { idx: i, uuid, btype, has_px, ins });
        }

        // indices (into `nodes`) of the blocks we must place
        let new_ids: Vec<usize> = (0..nodes.len()).filter(|&n| !nodes[n].has_px).collect();
        if new_ids.is_empty() {
            return Ok(0);
        }
        let uuid_to_node: HashMap<String, usize> =
            nodes.iter().enumerate().map(|(n, nd)| (nd.uuid.clone(), n)).collect();
        let new_set: HashSet<usize> = new_ids.iter().copied().collect();

        // predecessors among NEW blocks only: which new nodes feed node n
        let preds = |n: usize| -> Vec<usize> {
            let mut out = Vec::new();
            for src in &nodes[n].ins {
                if let Some(owner) = conn_owner.get(src)
                    && let Some(&pn) = uuid_to_node.get(owner)
                    && pn != n
                    && new_set.contains(&pn)
                {
                    out.push(pn);
                }
            }
            out
        };

        // --- layer assignment: longest path over the new-block sub-DAG ---
        let mut layer: HashMap<usize, i32> = HashMap::new();
        fn calc(
            n: usize,
            preds: &dyn Fn(usize) -> Vec<usize>,
            layer: &mut HashMap<usize, i32>,
            seen: &mut Vec<usize>,
        ) -> i32 {
            if let Some(&l) = layer.get(&n) {
                return l;
            }
            if seen.contains(&n) {
                return 0; // cycle guard
            }
            seen.push(n);
            let ps = preds(n);
            let l = if ps.is_empty() {
                0
            } else {
                1 + ps.iter().map(|&p| calc(p, preds, layer, seen)).max().unwrap_or(0)
            };
            seen.pop();
            layer.insert(n, l);
            l
        }
        for &n in &new_ids {
            let mut seen = Vec::new();
            calc(n, &preds, &mut layer, &mut seen);
        }

        // group by layer, deterministic base order = document order
        let mut by_layer: HashMap<i32, Vec<usize>> = HashMap::new();
        for &n in &new_ids {
            by_layer.entry(layer[&n]).or_default().push(n);
        }
        let mut layers: Vec<i32> = by_layer.keys().copied().collect();
        layers.sort_unstable();

        // --- ordering within a layer: barycenter of predecessor rows ---
        let mut row: HashMap<usize, usize> = HashMap::new();
        for &l in &layers {
            let mut lst = by_layer[&l].clone();
            if l > 0 {
                lst.sort_by(|&a, &b| {
                    let bary = |n: usize| -> f64 {
                        let rs: Vec<f64> = preds(n)
                            .iter()
                            .filter_map(|p| row.get(p).map(|&r| r as f64))
                            .collect();
                        if rs.is_empty() { 0.0 } else { rs.iter().sum::<f64>() / rs.len() as f64 }
                    };
                    bary(a).partial_cmp(&bary(b)).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            for (r, &n) in lst.iter().enumerate() {
                row.insert(n, r);
            }
            by_layer.insert(l, lst);
        }

        // --- coordinates ---
        // base: free area to the right of existing content (fallback to editor origin)
        let base_x = if max_px2 == i32::MIN { 576 } else { Self::snap96(max_px2 + 576) };
        let base_y = if min_py == i32::MAX { 576 } else { Self::snap96(min_py) };
        const COL_STEP: i32 = 2688; // widest block + gap (28 * 96)
        const ROW_STEP: i32 = 960; // 10 * 96

        // resolve target coordinates per new node
        let mut targets: Vec<(usize, i32, i32, i32, i32)> = Vec::new(); // (child idx, Px, Py, Px2, Py2)
        for &l in &layers {
            for &n in &by_layer[&l] {
                let (w, h) = block_size(&nodes[n].btype);
                let px = Self::snap96(base_x + l * COL_STEP);
                let py = Self::snap96(base_y + row[&n] as i32 * ROW_STEP);
                targets.push((nodes[n].idx, px, py, px + w, py + h));
            }
        }

        // --- Pass 2: apply (mutable, by child index) ---
        let page = self.get_element_mut(&page_path);
        let mut count = 0;
        for (idx, px, py, px2, py2) in targets {
            if let Some(elem) = page.children[idx].as_mut_element() {
                elem.attributes.insert("Px".to_string(), px.to_string());
                elem.attributes.insert("Py".to_string(), py.to_string());
                elem.attributes.insert("Px2".to_string(), px2.to_string());
                elem.attributes.insert("Py2".to_string(), py2.to_string());
                count += 1;
            }
        }
        Ok(count)
    }

}

#[cfg(test)]
mod incremental_tests {
    use super::super::ConfigEditor;

    // Src(positioned) --Q--> Not A(new) --Q--> And B(new)
    // incremental_layout must place A and B in left→right layers and leave Src untouched.
    const XML: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<C Type=\"Page\" U=\"page-1\" Title=\"P\">\n\
\t<C Type=\"Switch\" U=\"src\" Title=\"Src\" Px=\"1000\" Py=\"1000\" Px2=\"2344\" Py2=\"1696\">\n\
\t\t<Co K=\"Q\" U=\"q-src\"/>\n\
\t</C>\n\
\t<C Type=\"Not\" U=\"a\" Title=\"A\">\n\
\t\t<Co K=\"I\" U=\"i-a\"><In Input=\"q-src\"/></Co>\n\
\t\t<Co K=\"Q\" U=\"q-a\"/>\n\
\t</C>\n\
\t<C Type=\"And\" U=\"b\" Title=\"B\">\n\
\t\t<Co K=\"I1\" U=\"i-b\"><In Input=\"q-a\"/></Co>\n\
\t\t<Co K=\"Q\" U=\"q-b\"/>\n\
\t</C>\n\
</C>\n";

    fn px_of(xml: &str, uuid: &str) -> Option<i32> {
        // crude: find `U="uuid"` then the following `Px="..."` within the same tag
        let key = format!("U=\"{uuid}\"");
        let start = xml.find(&key)?;
        let tag_end = xml[start..].find('>')? + start;
        let seg = &xml[start..tag_end];
        let p = seg.find("Px=\"")? + 4;
        let end = seg[p..].find('"')? + p;
        seg[p..end].parse().ok()
    }

    #[test]
    fn incremental_places_only_new_in_layers() {
        let mut editor = ConfigEditor::load(XML.as_bytes()).unwrap();
        let count = editor.incremental_layout("Type:Page").unwrap();
        assert_eq!(count, 2, "only the two unpositioned blocks are placed");

        let out = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        // existing positioned block is untouched
        assert_eq!(px_of(&out, "src"), Some(1000), "Src Px must not change");
        // A and B are now positioned
        let ax = px_of(&out, "a").expect("A positioned");
        let bx = px_of(&out, "b").expect("B positioned");
        // B (fed by A) sits in a later layer → strictly further right
        assert!(bx > ax, "B (layer 1) must be right of A (layer 0): ax={ax} bx={bx}");
        // grid-snapped
        assert_eq!(ax % 96, 0, "A.Px snapped to 96-grid");
        assert_eq!(bx % 96, 0, "B.Px snapped to 96-grid");
    }

    #[test]
    fn incremental_noop_when_all_positioned() {
        // strip the two unpositioned gates → nothing to place
        let only_src = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<C Type=\"Page\" U=\"page-1\" Title=\"P\">\n\
\t<C Type=\"Switch\" U=\"src\" Title=\"Src\" Px=\"1000\" Py=\"1000\" Px2=\"2344\" Py2=\"1696\">\n\
\t\t<Co K=\"Q\" U=\"q-src\"/>\n\
\t</C>\n\
</C>\n";
        let mut editor = ConfigEditor::load(only_src.as_bytes()).unwrap();
        assert_eq!(editor.incremental_layout("Type:Page").unwrap(), 0);
    }
}
