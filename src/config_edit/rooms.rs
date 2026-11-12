use super::ConfigEditor;
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use xmltree::Element;

#[derive(Debug, serde::Serialize)]
pub struct RoomRenameSelection {
    pub path: Vec<usize>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct RoomRenamePreview {
    pub selector: String,
    pub selected: RoomRenameSelection,
    pub uuid_source: RoomRenameSelection,
    pub target: RoomRenameSelection,
    pub before_title: Option<String>,
    pub after_title: String,
    pub changed: bool,
    pub report: String,
    pub scope: &'static str,
}

struct RoomRenamePlan {
    preview: RoomRenamePreview,
}

impl ConfigEditor {
    pub fn preview_room_rename(&self, selector: &str, name: &str) -> Result<RoomRenamePreview> {
        Ok(self.plan_room_rename(selector, name)?.preview)
    }

    pub fn rename_room(&mut self, selector: &str, name: &str) -> Result<String> {
        let plan = self.plan_room_rename(selector, name)?;
        self.get_element_mut(&plan.preview.target.path)
            .attributes
            .insert("Title".to_string(), plan.preview.after_title);
        Ok(plan.preview.report)
    }

    fn plan_room_rename(&self, selector: &str, name: &str) -> Result<RoomRenamePlan> {
        let selected_path = self.require_one(selector)?;
        let uuid = self
            .get_element(&selected_path)
            .attributes
            .get("U")
            .cloned()
            .unwrap_or_default();
        // Preserve the final UUID lookup's canonical mirror selection.
        let target_path = self.require_one(&format!("uuid:{uuid}"))?;
        let target = self.get_element(&target_path);
        let before_title = target.attributes.get("Title").cloned();
        let title = before_title.as_deref().unwrap_or_default();
        let report = format!(
            "Set {}.Title: {} → '{}'",
            title,
            before_title.as_deref().unwrap_or("(none)"),
            name
        );
        let selection = |path: Vec<usize>| RoomRenameSelection {
            attributes: self
                .get_element(&path)
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            path,
        };
        Ok(RoomRenamePlan {
            preview: RoomRenamePreview {
                selector: selector.to_owned(),
                selected: selection(selected_path.clone()),
                uuid_source: selection(selected_path),
                target: selection(target_path),
                changed: before_title.as_deref() != Some(name),
                before_title,
                after_title: name.to_owned(),
                report,
                scope: "Existing CLI Title attribute edit only; no native UI or physical-effect claim",
            },
        })
    }

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
        Self::collect_typed_with_iodata(
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
        Self::walk_rooms(&self.root, &lower, &mut found);
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

    fn walk_rooms(elem: &Element, name_lower: &str, found: &mut Vec<(String, String)>) {
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
                Self::walk_rooms(child_elem, name_lower, found);
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
        Self::walk_categories(&self.root, &lower, &mut found);
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

    fn walk_categories(elem: &Element, name_lower: &str, found: &mut Vec<(String, String)>) {
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
                Self::walk_categories(child_elem, name_lower, found);
            }
        }
    }

    fn collect_typed_with_iodata(
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
                Self::collect_typed_with_iodata(
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
        Self::walk_rooms(&self.root, &name.to_lowercase(), &mut existing);
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
        Self::walk_users(&self.root, &mut |title| {
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

    fn walk_users(elem: &Element, cb: &mut dyn FnMut(&str)) {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == "User"
            && let Some(title) = elem.attributes.get("Title")
        {
            cb(title);
        }
        for child in &elem.children {
            if let Some(child_elem) = child.as_element() {
                Self::walk_users(child_elem, cb);
            }
        }
    }

    fn find_user_caption(&self) -> Option<Vec<usize>> {
        Self::find_user_caption_recursive(&self.root, &mut Vec::new())
    }

    fn find_user_caption_recursive(elem: &Element, path: &mut Vec<usize>) -> Option<Vec<usize>> {
        if elem.name == "C"
            && let Some(t) = elem.attributes.get("Type")
            && t == "UserCaption"
        {
            return Some(path.clone());
        }
        for (i, child) in elem.children.iter().enumerate() {
            if let Some(child_elem) = child.as_element() {
                path.push(i);
                if let Some(result) = Self::find_user_caption_recursive(child_elem, path) {
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

#[cfg(test)]
mod room_rename_tests {
    use super::*;

    fn legacy_rename(editor: &mut ConfigEditor, selector: &str, name: &str) -> Result<String> {
        let _path = editor.require_one(selector)?;
        editor.set_attribute(
            &format!("uuid:{}", {
                let elements = editor.find_elements(selector);
                if elements.is_empty() {
                    bail!("Room '{}' not found", selector);
                }
                editor
                    .get_element(&elements[0])
                    .attributes
                    .get("U")
                    .cloned()
                    .unwrap_or_default()
            }),
            "Title",
            name,
        )
    }

    #[test]
    fn room_rename_plan_preserves_legacy_admission_raw_titles_and_entire_dom() {
        let xml = r#"<ControlList Future="retain">
          <C Type="Document" U="doc" Date="2020-01-01 00:00:00" DateS="347068800"/>
          <C Type="Place" U="room" Title="  Küche,&quot;室&quot;  " gid="RoomGid" Future="opaque">
            <IoData Pr="other" Cr="cat" Future="01"/><SET><Unknown v="+003"/></SET>
          </C>
          <C Type="Place" U="other" Title="Duplicate"/>
          <C Type="Page" U="page"><C Type="FutureBlock" U="block" Title="NonRoom">
            <IoData Pr="room" Cr="cat" Future="002"/><IoData Pr="other"/>
            <Co K="AQ" U="out" Def="+000"><In Input="ref" FLG="01"/></Co>
            <Unknown Ref="room" Title="retain"/>
          </C></C>
          <C Type="AutomaticDesigner"><C Type="Place" U="room" Title="mirror"><IoData Pr="room"/></C></C>
          <Unknown Pr="room" U="future" Title="retain"/>
        </ControlList>"#;
        let data = format!("\u{feff}{}\r\n", xml.replace('\n', "\r\n"));
        for selector in [
            "KÜCHE",
            "uuid:room",
            "gid:roomgid",
            "NonRoom",
            "Type:FutureBlock",
        ] {
            for name in [
                "",
                "  ",
                "Duplicate",
                "Grüße 室 🏠",
                "  Küche,\"室\"  ",
                "x,\n\"y\"",
            ] {
                let mut editor = ConfigEditor::load(data.as_bytes()).unwrap();
                let before = format!("{:?}", editor.root);
                let preview = editor.preview_room_rename(selector, name).unwrap();
                assert_eq!(format!("{:?}", editor.root), before);
                assert_eq!(preview.after_title, name);
                assert_eq!(
                    preview.before_title.as_deref(),
                    preview.target.attributes.get("Title").map(String::as_str)
                );
                assert_eq!(
                    preview.changed,
                    preview.before_title.as_deref() != Some(name)
                );
                let mut legacy = ConfigEditor::load(data.as_bytes()).unwrap();
                let expected_report = legacy_rename(&mut legacy, selector, name).unwrap();
                assert_eq!(preview.report, expected_report);
                assert_eq!(editor.rename_room(selector, name).unwrap(), expected_report);
                assert_eq!(format!("{:?}", editor.root), format!("{:?}", legacy.root));
                assert_eq!(editor.to_bytes().unwrap(), legacy.to_bytes().unwrap());
                let output = editor.to_bytes().unwrap();
                assert!(output.starts_with(&[0xef, 0xbb, 0xbf]));
                assert!(output.windows(2).any(|bytes| bytes == b"\r\n"));
            }
        }
    }

    #[test]
    fn room_rename_plan_keeps_uuid_relookup_and_missing_title() {
        for (xml, selector, selected, target, before) in [
            (
                r#"<ControlList><C Type="Future" U="same" Title="First"/><C Type="Place" U="same" Title="Second"/></ControlList>"#,
                "Second",
                "same",
                "same",
                Some("First"),
            ),
            (
                r#"<ControlList><C Type="Future" U="x"/></ControlList>"#,
                "uuid:x",
                "x",
                "x",
                None,
            ),
            (
                r#"<ControlList><C Type="place" U="x" Title="" title="opaque" Future="001"/></ControlList>"#,
                "uuid:x",
                "x",
                "x",
                Some(""),
            ),
            (
                r#"<ControlList><C U="x" Title="Only"/></ControlList>"#,
                "",
                "x",
                "x",
                Some("Only"),
            ),
            (
                r#"<ControlList><C Type="Future" U="" Title="Empty"/></ControlList>"#,
                "Empty",
                "",
                "",
                Some("Empty"),
            ),
            (
                r#"<ControlList><C Type="Future" Title="NoUuid"/><C U="" Title="Fallback"/></ControlList>"#,
                "NoUuid",
                "",
                "",
                Some("Fallback"),
            ),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let preview = editor.preview_room_rename(selector, "").unwrap();
            assert_eq!(
                preview
                    .selected
                    .attributes
                    .get("U")
                    .map(String::as_str)
                    .unwrap_or_default(),
                selected
            );
            assert_eq!(preview.target.attributes["U"], target);
            assert_eq!(preview.before_title.as_deref(), before);
            assert_eq!(preview.changed, before != Some(""));
            let mut legacy = ConfigEditor::load(xml.as_bytes()).unwrap();
            assert_eq!(
                editor.rename_room(selector, "").unwrap(),
                legacy_rename(&mut legacy, selector, "").unwrap()
            );
            assert_eq!(editor.to_bytes().unwrap(), legacy.to_bytes().unwrap());
        }
    }

    #[test]
    fn room_rename_qualified_selection_is_uuid_source_in_either_dom_order() {
        let alpha = r#"<C Type="Future" U="alpha" Title="Lamp" Future="+003"><IoData Pr="a" Cr="cat" Future="01"/><Co U="alpha-out"/><Unknown Ref="beta" Value="keep"/></C>"#;
        let beta = r#"<C Type="Future" U="beta" Title="Lamp" Future="+004"><IoData Pr="b" Cr="cat" Future="02"/><Co U="beta-out"/><Unknown Ref="alpha" Value="retain"/></C>"#;
        for objects in [format!("{alpha}{beta}"), format!("{beta}{alpha}")] {
            let xml = format!(
                r#"<ControlList Future="opaque"><C Type="Place" U="a" Title="Alpha"/><C Type="Place" U="b" Title="Beta"/>{objects}</ControlList>"#
            );
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            let preview = editor
                .preview_room_rename("Lamp [Beta]", "Grüße 室")
                .unwrap();
            assert_eq!(format!("{:?}", editor.root), before);
            assert_eq!(preview.selected.attributes["U"], "beta");
            assert_eq!(preview.uuid_source.path, preview.selected.path);
            assert_eq!(preview.uuid_source.attributes, preview.selected.attributes);
            assert_eq!(preview.target.path, preview.selected.path);
            assert_eq!(preview.before_title.as_deref(), Some("Lamp"));
            let mut expected = ConfigEditor::load(xml.as_bytes()).unwrap();
            let path = expected.require_one("uuid:beta").unwrap();
            expected
                .get_element_mut(&path)
                .attributes
                .insert("Title".to_owned(), "Grüße 室".to_owned());
            assert_eq!(
                editor.rename_room("Lamp [Beta]", "Grüße 室").unwrap(),
                preview.report
            );
            assert_eq!(format!("{:?}", editor.root), format!("{:?}", expected.root));
        }
    }

    #[test]
    fn room_rename_qualified_selection_still_relooks_up_canonical_uuid_mirror() {
        let xml = r#"<ControlList><C Type="Place" U="a" Title="Alpha"/><C Type="Place" U="b" Title="Beta"/><C Type="Future" U="beta" Title="Canonical"><IoData Pr="a" Future="keep"/></C><C Type="Future" U="alpha" Title="Lamp"><IoData Pr="a"/></C><C Type="Future" U="beta" Title="Lamp"><IoData Pr="b"/><Unknown U="metadata" Ref="beta"/></C></ControlList>"#;
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = format!("{:?}", editor.root);
        let preview = editor.preview_room_rename("Lamp [Beta]", "New").unwrap();
        assert_eq!(format!("{:?}", editor.root), before);
        assert_eq!(preview.selected.path, vec![4]);
        assert_eq!(preview.uuid_source.path, preview.selected.path);
        assert_eq!(preview.uuid_source.attributes, preview.selected.attributes);
        assert_eq!(preview.target.path, vec![2]);
        assert_eq!(preview.target.attributes["U"], "beta");
        assert_eq!(preview.before_title.as_deref(), Some("Canonical"));
        let mut expected = ConfigEditor::load(xml.as_bytes()).unwrap();
        let report = expected.set_attribute("uuid:beta", "Title", "New").unwrap();
        assert_eq!(preview.report, report);
        assert_eq!(editor.rename_room("Lamp [Beta]", "New").unwrap(), report);
        assert_eq!(format!("{:?}", editor.root), format!("{:?}", expected.root));
    }

    #[test]
    fn room_rename_qualified_missing_uuid_uses_existing_empty_uuid_resolution() {
        for (uuid_attr, fallback, error) in [
            ("", "", Some("'uuid:' not found")),
            (r#" U="""#, "", None),
            ("", r#"<C U="" Title="Fallback"/>"#, None),
            (
                "",
                r#"<C U="" Title="Fallback"/><C U="" Title="Another"/>"#,
                Some("Selector 'uuid:' matches 2 elements"),
            ),
        ] {
            let xml = format!(
                r#"<ControlList><C Type="Place" U="a" Title="Alpha"/><C Type="Place" U="b" Title="Beta"/><C Type="Future" U="alpha" Title="Lamp"><IoData Pr="a"/></C><C Type="Future"{uuid_attr} Title="Lamp"><IoData Pr="b"/><Unknown Value="retain"/></C>{fallback}</ControlList>"#
            );
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            if let Some(error) = error {
                let preview_error = editor
                    .preview_room_rename("Lamp [Beta]", "New")
                    .unwrap_err()
                    .to_string();
                assert!(preview_error.contains(error), "{preview_error}");
                assert_eq!(
                    editor
                        .rename_room("Lamp [Beta]", "New")
                        .unwrap_err()
                        .to_string(),
                    preview_error
                );
                assert_eq!(format!("{:?}", editor.root), before);
            } else {
                let preview = editor.preview_room_rename("Lamp [Beta]", "New").unwrap();
                assert_eq!(format!("{:?}", editor.root), before);
                assert_eq!(preview.uuid_source.path, preview.selected.path);
                assert_eq!(preview.uuid_source.attributes, preview.selected.attributes);
                assert_eq!(
                    preview.target.path,
                    if uuid_attr.is_empty() {
                        vec![4]
                    } else {
                        vec![3]
                    }
                );
                let mut expected = ConfigEditor::load(xml.as_bytes()).unwrap();
                let report = expected.set_attribute("uuid:", "Title", "New").unwrap();
                assert_eq!(preview.report, report);
                assert_eq!(editor.rename_room("Lamp [Beta]", "New").unwrap(), report);
                assert_eq!(format!("{:?}", editor.root), format!("{:?}", expected.root));
            }
        }
    }

    #[test]
    fn room_rename_plan_keeps_ordered_errors_without_mutation() {
        for (xml, selector, error) in [
            (
                r#"<ControlList><C U="a" Title="Kitchen"/><C U="b" Title="Kitchen"/></ControlList>"#,
                "Kitchen",
                "Selector 'Kitchen' matches 2 elements",
            ),
            (
                r#"<ControlList><C Title="NoUuid"/></ControlList>"#,
                "NoUuid",
                "'uuid:' not found",
            ),
            (
                r#"<ControlList><C Title="NoUuid"/><C U=""/><C U=""/></ControlList>"#,
                "NoUuid",
                "Selector 'uuid:' matches 2 elements",
            ),
            (
                r#"<ControlList><C U="a" Title="Kitchen"/><C U="b" Title="Kitchen extension"/></ControlList>"#,
                "Kitchen",
                "Selector 'Kitchen' matches 2 elements",
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Kitchen"/></ControlList>"#,
                "missing",
                "'missing' not found",
            ),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            let preview_error = editor
                .preview_room_rename(selector, "")
                .unwrap_err()
                .to_string();
            assert!(preview_error.contains(error), "{preview_error}");
            let mut legacy = ConfigEditor::load(xml.as_bytes()).unwrap();
            let legacy_error = legacy_rename(&mut legacy, selector, "")
                .unwrap_err()
                .to_string();
            assert_eq!(preview_error, legacy_error);
            assert_eq!(
                editor.rename_room(selector, "").unwrap_err().to_string(),
                legacy_error
            );
            assert_eq!(format!("{:?}", editor.root), before);
        }
    }
}
