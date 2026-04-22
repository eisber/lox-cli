use super::ConfigEditor;
use anyhow::{Result, bail};
use xmltree::Element;

impl ConfigEditor {
    /// Move all elements matching a type filter to a different room.
    /// Returns the number of elements moved.
    pub fn move_to_room(
        &mut self,
        type_filter: &str,
        target_room_name: &str,
        exclude_types: &[&str],
    ) -> Result<(usize, String)> {
        // First, find the target room UUID
        let room_uuid = self.find_room_uuid(target_room_name)?;

        // Collect paths to matching elements
        let mut paths = Vec::new();
        self.collect_typed_with_iodata(
            &self.root,
            type_filter,
            exclude_types,
            &mut Vec::new(),
            &mut paths,
        );

        let count = paths.len();
        for path in &paths {
            let elem = self.get_element_mut(path);
            // Find IoData child and update Pr attribute
            for child in &mut elem.children {
                if let Some(iodata) = child.as_mut_element()
                    && iodata.name == "IoData"
                {
                    iodata
                        .attributes
                        .insert("Pr".to_string(), room_uuid.clone());
                }
            }
        }

        Ok((count, room_uuid))
    }

    pub fn find_room_uuid(&self, room_name: &str) -> Result<String> {
        let lower = room_name.to_lowercase();
        let mut found = Vec::new();
        self.walk_rooms(&self.root, &lower, &mut found);
        match found.len() {
            0 => bail!("Room '{}' not found in config", room_name),
            1 => Ok(found.into_iter().next().unwrap().0),
            _ => {
                // Try exact match
                let exact: Vec<_> = found
                    .iter()
                    .filter(|(_uuid, name)| name.to_lowercase() == lower)
                    .collect();
                if exact.len() == 1 {
                    Ok(exact[0].0.clone())
                } else {
                    bail!(
                        "Multiple rooms match '{}': {:?}",
                        room_name,
                        found.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>()
                    )
                }
            }
        }
    }

    fn walk_rooms(&self, elem: &Element, name_lower: &str, found: &mut Vec<(String, String)>) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == "Place"
            && let (Some(uuid), Some(title)) =
                (elem.attributes.get("U"), elem.attributes.get("Title"))
            && title.to_lowercase().contains(name_lower)
        {
            found.push((uuid.clone(), title.clone()));
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.walk_rooms(child_elem, name_lower, found);
            }
        }
    }

    /// Find a Page element by title (fuzzy substring match).
    /// Returns the path to the Page element.
    /// De-duplicates by UUID (AutomaticDesigner mirrors Program pages).
    pub fn find_page_path(&self, page_name: &str) -> Result<Vec<usize>> {
        let lower = page_name.to_lowercase();
        let all_pages = self.find_elements("Type:Page");
        let mut found: Vec<(Vec<usize>, String)> = Vec::new();
        let mut seen_uuids = std::collections::HashSet::new();
        for path in all_pages {
            let elem = self.get_element(&path);
            if let Some(title) = elem.attributes.get("Title")
                && title.to_lowercase().contains(&lower)
            {
                let uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                if seen_uuids.insert(uuid) {
                    found.push((path, title.clone()));
                }
            }
        }
        match found.len() {
            0 => bail!("Page '{}' not found in config", page_name),
            1 => Ok(found.into_iter().next().unwrap().0),
            _ => {
                let exact: Vec<_> = found
                    .iter()
                    .filter(|(_, title)| title.to_lowercase() == lower)
                    .collect();
                if exact.len() == 1 {
                    Ok(exact[0].0.clone())
                } else {
                    bail!(
                        "Multiple pages match '{}': {:?}",
                        page_name,
                        found.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>()
                    )
                }
            }
        }
    }

    /// Try to find a Page whose title matches a room name.
    /// Returns None if no match found (non-fatal).
    pub fn find_page_path_for_room(&self, room_name: &str) -> Option<Vec<usize>> {
        self.find_page_path(room_name).ok()
    }

    /// Find the maximum Py value among children of the given Page element.
    /// Used for stacking new blocks below existing ones.
    pub fn max_py_on_page(&self, page_path: &[usize]) -> i64 {
        let page = self.get_element(page_path);
        let mut max_py: i64 = 0;
        for child in &page.children {
            if let Some(child_elem) = child.as_element()
                && let Some(py_str) = child_elem.attributes.get("Py")
                && let Ok(py) = py_str.parse::<i64>()
                && py > max_py
            {
                max_py = py;
            }
        }
        max_py
    }

    pub fn find_category_uuid(&self, cat_name: &str) -> Result<String> {
        let lower = cat_name.to_lowercase();
        let mut found = Vec::new();
        self.walk_categories(&self.root, &lower, &mut found);
        match found.len() {
            0 => bail!("Category '{}' not found in config", cat_name),
            1 => Ok(found.into_iter().next().unwrap().0),
            _ => {
                let exact: Vec<_> = found
                    .iter()
                    .filter(|(_uuid, name)| name.to_lowercase() == lower)
                    .collect();
                if exact.len() == 1 {
                    Ok(exact[0].0.clone())
                } else {
                    bail!(
                        "Multiple categories match '{}': {:?}",
                        cat_name,
                        found.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>()
                    )
                }
            }
        }
    }

    fn walk_categories(&self, elem: &Element, name_lower: &str, found: &mut Vec<(String, String)>) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == "Category"
            && let (Some(uuid), Some(title)) =
                (elem.attributes.get("U"), elem.attributes.get("Title"))
            && title.to_lowercase().contains(name_lower)
        {
            found.push((uuid.clone(), title.clone()));
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.walk_categories(child_elem, name_lower, found);
            }
        }
    }

    fn collect_typed_with_iodata(
        &self,
        elem: &Element,
        type_filter: &str,
        exclude_types: &[&str],
        path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t.eq_ignore_ascii_case(type_filter)
            && !exclude_types.iter().any(|ex| t.eq_ignore_ascii_case(ex))
        {
            // Check if has IoData child
            if elem
                .children
                .iter()
                .any(|c| c.as_element().map(|e| e.name == "IoData").unwrap_or(false))
            {
                results.push(path.clone());
            }
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                self.collect_typed_with_iodata(
                    child_elem,
                    type_filter,
                    exclude_types,
                    path,
                    results,
                );
                path.pop();
            }
        }
    }

    /// Add a new room (Place element).
    pub fn add_room(&mut self, name: &str) -> Result<String> {
        // Check if room already exists
        let mut existing = Vec::new();
        self.walk_rooms(&self.root, &name.to_lowercase(), &mut existing);
        if !existing.is_empty() {
            bail!("Room '{}' already exists", name);
        }

        // Generate a UUID
        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>()
        );

        // Find PlaceCaption or first Place and insert after
        let mut insert_path = None;
        for (i, child) in self.root.children.iter().enumerate() {
            if let Some(elem) = child.as_element()
                && elem.name == "C"
                && let Some(t) = elem.attributes.get("Type")
                && (t == "Place" || t == "PlaceCaption")
            {
                insert_path = Some(i);
            }
        }

        let mut place = Element::new("C");
        place
            .attributes
            .insert("Type".to_string(), "Place".to_string());
        place.attributes.insert("V".to_string(), "175".to_string());
        place.attributes.insert("U".to_string(), uuid.clone());
        place
            .attributes
            .insert("Title".to_string(), name.to_string());
        place
            .attributes
            .insert("WF".to_string(), "16384".to_string());

        if let Some(idx) = insert_path {
            self.root
                .children
                .insert(idx + 1, xmltree::XMLNode::Element(place));
        } else {
            self.root.children.push(xmltree::XMLNode::Element(place));
        }

        Ok(uuid)
    }

    /// Add a new category to the config.
    pub fn add_category(&mut self, name: &str) -> Result<String> {
        // Check if category already exists
        if let Ok(uuid) = self.find_category_uuid(name) {
            return Ok(uuid);
        }

        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>()
        );

        // Find last Category element and insert after
        let mut insert_path = None;
        for (i, child) in self.root.children.iter().enumerate() {
            if let Some(elem) = child.as_element()
                && elem.name == "C"
                && let Some(t) = elem.attributes.get("Type")
                && t == "Category"
            {
                insert_path = Some(i);
            }
        }

        let mut cat = Element::new("C");
        cat.attributes
            .insert("Type".to_string(), "Category".to_string());
        cat.attributes.insert("V".to_string(), "175".to_string());
        cat.attributes.insert("U".to_string(), uuid.clone());
        cat.attributes.insert("Title".to_string(), name.to_string());
        cat.attributes.insert("WF".to_string(), "16384".to_string());

        if let Some(idx) = insert_path {
            self.root
                .children
                .insert(idx + 1, xmltree::XMLNode::Element(cat));
        } else {
            self.root.children.push(xmltree::XMLNode::Element(cat));
        }

        Ok(uuid)
    }

    /// Add a new user account.
    pub fn add_user(&mut self, name: &str) -> Result<String> {
        // Check if user already exists
        let mut exists = false;
        self.walk_users(&self.root, &mut |title| {
            if title.eq_ignore_ascii_case(name) {
                exists = true;
            }
        });
        if exists {
            bail!("User '{}' already exists", name);
        }

        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>()
        );

        let mut user = Element::new("C");
        user.attributes
            .insert("Type".to_string(), "User".to_string());
        user.attributes.insert("V".to_string(), "175".to_string());
        user.attributes.insert("U".to_string(), uuid.clone());
        user.attributes
            .insert("Title".to_string(), name.to_string());
        user.attributes.insert("NFCArr".to_string(), String::new());
        user.attributes.insert("Desc".to_string(), String::new());

        // Find UserCaption to insert under, or insert at root
        if let Some(caption_path) = self.find_user_caption() {
            let caption = self.get_element_mut(&caption_path);
            caption.children.push(xmltree::XMLNode::Element(user));
        } else {
            self.root.children.push(xmltree::XMLNode::Element(user));
        }

        Ok(uuid)
    }

    /// Remove a user account by name. Returns the UUID.
    pub fn remove_user(&mut self, name: &str) -> Result<String> {
        let lower = name.to_lowercase();
        // Find and remove the user
        if let Some(uuid) = self.find_and_remove_user(&lower) {
            Ok(uuid)
        } else {
            bail!("User '{}' not found", name)
        }
    }

    fn walk_users(&self, elem: &Element, cb: &mut dyn FnMut(&str)) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == "User"
            && let Some(title) = elem.attributes.get("Title")
        {
            cb(title);
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                self.walk_users(child_elem, cb);
            }
        }
    }

    fn find_user_caption(&self) -> Option<Vec<usize>> {
        self.find_user_caption_recursive(&self.root, &mut Vec::new())
    }

    fn find_user_caption_recursive(
        &self,
        elem: &Element,
        path: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == "UserCaption"
        {
            return Some(path.clone());
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                if let Some(result) = self.find_user_caption_recursive(child_elem, path) {
                    return Some(result);
                }
                path.pop();
            }
        }
        None
    }

    fn find_and_remove_user(&mut self, name_lower: &str) -> Option<String> {
        Self::remove_user_recursive(&mut self.root.children, name_lower)
    }

    fn remove_user_recursive(
        children: &mut Vec<xmltree::XMLNode>,
        name_lower: &str,
    ) -> Option<String> {
        for i in 0..children.len() {
            if let Some(elem) = children[i].as_element()
                && elem.name == "C"
                && let Some(t) = elem.attributes.get("Type")
                && t == "User"
                && let Some(title) = elem.attributes.get("Title")
                && title.to_lowercase() == name_lower
            {
                let uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                children.remove(i);
                return Some(uuid);
            }
        }
        for child in children.iter_mut() {
            if let Some(elem) = child.as_mut_element()
                && let Some(uuid) = Self::remove_user_recursive(&mut elem.children, name_lower)
            {
                return Some(uuid);
            }
        }
        None
    }
}
