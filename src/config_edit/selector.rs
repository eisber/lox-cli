use super::{ConfigEditor, matches_selector};
use anyhow::{Result, bail};
use xmltree::Element;

impl ConfigEditor {
    /// Find all elements matching a selector.
    pub fn find_elements(&self, selector: &str) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        self.find_recursive(&self.root, selector, &mut Vec::new(), &mut results);
        results
    }

    fn find_recursive(
        &self,
        elem: &Element,
        selector: &str,
        path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if matches_selector(elem, selector) {
            results.push(path.clone());
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                self.find_recursive(child_elem, selector, path, results);
                path.pop();
            }
        }
    }

    /// Require exactly one matching element. Returns the path.
    pub fn require_one(&self, selector: &str) -> Result<Vec<usize>> {
        let mut matches = self.find_elements(selector);

        // Bracket syntax room filtering: "Title [Room]"
        if let Some(bracket_start) = selector.rfind('[')
            && selector.ends_with(']')
        {
            let room_part = &selector[bracket_start + 1..selector.len() - 1];
            let room_lower = room_part.to_lowercase();
            // Build room UUID → name map
            let room_map = self.build_room_map();
            matches.retain(|path| {
                let elem = self.get_element(path);
                let room_uuid = elem.children.iter().find_map(|c| {
                    c.as_element()
                        .filter(|e| e.name == "IoData")
                        .and_then(|e| e.attributes.get("Pr").cloned())
                });
                if let Some(ru) = room_uuid {
                    room_map
                        .get(&ru)
                        .map(|name| name.to_lowercase().contains(&room_lower))
                        .unwrap_or(false)
                } else {
                    false
                }
            });
        }

        // De-duplicate by UUID (AutomaticDesigner mirrors Program elements)
        {
            let mut seen = std::collections::HashSet::new();
            matches.retain(|path| {
                let elem = self.get_element(path);
                let uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                if uuid.is_empty() {
                    true
                } else {
                    seen.insert(uuid)
                }
            });
        }

        match matches.len() {
            0 => {
                // Use Levenshtein fuzzy matching for suggestions
                let all_titles = self.collect_all_titles();
                Err(crate::errors::not_found_error(
                    "Element",
                    selector,
                    &all_titles,
                    "lox config controls <file>",
                ))
            }
            1 => Ok(matches.into_iter().next().unwrap()),
            n => {
                // List matches to help disambiguate
                let mut details = Vec::new();
                for path in &matches {
                    let elem = self.get_element(path);
                    let title = elem.attributes.get("Title").cloned().unwrap_or_default();
                    let etype = elem.attributes.get("Type").cloned().unwrap_or_default();
                    let uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                    details.push(format!("  {} '{}' (uuid:{})", etype, title, uuid));
                }
                let shown = if details.len() > 8 {
                    let mut d = details[..8].to_vec();
                    d.push(format!("  ... and {} more", details.len() - 8));
                    d
                } else {
                    details
                };
                bail!(
                    "Selector '{}' matches {} elements (expected 1):\n{}\nUse uuid:<uuid> for exact match.",
                    selector,
                    n,
                    shown.join("\n")
                )
            }
        }
    }

    /// Build a map of room UUID → room name from Place elements.
    fn build_room_map(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        fn walk(elem: &Element, map: &mut std::collections::HashMap<String, String>) {
            if elem.name == "C"
                && elem
                    .attributes
                    .get("Type")
                    .map(|t| t == "Place")
                    .unwrap_or(false)
                && let (Some(u), Some(t)) = (elem.attributes.get("U"), elem.attributes.get("Title"))
            {
                map.insert(u.clone(), t.clone());
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    walk(e, map);
                }
            }
        }
        walk(&self.root, &mut map);
        map
    }

    fn collect_all_titles(&self) -> Vec<String> {
        let mut titles = Vec::new();
        fn walk(elem: &Element, titles: &mut Vec<String>) {
            if let Some(t) = elem.attributes.get("Title")
                && !t.is_empty()
            {
                titles.push(t.clone());
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    walk(e, titles);
                }
            }
        }
        walk(&self.root, &mut titles);
        titles.sort();
        titles.dedup();
        titles
    }

    #[allow(dead_code)]
    fn find_exact_title(
        &self,
        elem: &Element,
        title: &str,
        path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if elem.name == "C"
            && elem
                .attributes
                .get("Title")
                .map(|t| t == title)
                .unwrap_or(false)
        {
            results.push(path.clone());
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                self.find_exact_title(child_elem, title, path, results);
                path.pop();
            }
        }
    }
}
