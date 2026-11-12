use super::ConfigEditor;
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use xmltree::Element;

#[derive(Debug, serde::Serialize)]
pub struct RoomAddElement {
    pub path: Vec<usize>,
    pub element_name: String,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct RoomAddPreview {
    pub name: String,
    pub parent: RoomAddElement,
    pub insert_after: Option<RoomAddElement>,
    pub room: RoomAddElement,
    pub uuid_ephemeral: bool,
    pub report: String,
    pub scope: &'static str,
}

struct RoomAddPlan {
    attributes: Vec<(String, String)>,
    preview: RoomAddPreview,
}

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

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ControlMoveSelector {
    Type(String),
    Title(String),
}

#[derive(Debug, serde::Serialize)]
pub struct ControlMoveRoomReference {
    pub path: Vec<usize>,
    pub attributes: BTreeMap<String, String>,
    pub before_pr: Option<String>,
    pub after_pr: String,
    pub created: bool,
    pub changed: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct ControlMoveSelection {
    pub path: Vec<usize>,
    pub attributes: BTreeMap<String, String>,
    pub io_data: Vec<ControlMoveRoomReference>,
}

#[derive(Debug, serde::Serialize)]
pub struct ControlMovePreview {
    pub selector: ControlMoveSelector,
    pub exclude: Vec<String>,
    pub target_room: String,
    pub room_uuid: String,
    pub selected: Vec<ControlMoveSelection>,
    pub count: usize,
    pub changed: bool,
    pub report: String,
    pub scope: &'static str,
}

struct ControlMovePlan {
    preview: ControlMovePreview,
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
        let plan =
            self.plan_control_move(Some(type_filter), None, target_room_name, exclude_types)?;
        self.apply_control_move(&plan);
        Ok((plan.preview.count, plan.preview.room_uuid))
    }

    pub fn preview_control_move(
        &self,
        type_filter: Option<&str>,
        title: Option<&str>,
        target_room: &str,
        exclude: &[&str],
    ) -> Result<ControlMovePreview> {
        Ok(self
            .plan_control_move(type_filter, title, target_room, exclude)?
            .preview)
    }

    pub fn move_control(
        &mut self,
        type_filter: Option<&str>,
        title: Option<&str>,
        target_room: &str,
        exclude: &[&str],
    ) -> Result<String> {
        let plan = self.plan_control_move(type_filter, title, target_room, exclude)?;
        self.apply_control_move(&plan);
        Ok(plan.preview.report)
    }

    fn plan_control_move(
        &self,
        type_filter: Option<&str>,
        title: Option<&str>,
        target_room: &str,
        exclude: &[&str],
    ) -> Result<ControlMovePlan> {
        let (selector, paths, room_uuid) = if let Some(tf) = type_filter {
            let room_uuid = self.find_room_uuid(target_room)?;
            let mut paths = Vec::new();
            Self::collect_typed_with_iodata(&self.root, tf, exclude, &mut Vec::new(), &mut paths);
            (ControlMoveSelector::Type(tf.to_owned()), paths, room_uuid)
        } else if let Some(title) = title {
            let path = self.require_one(title)?;
            let room_uuid = self.find_room_uuid(target_room)?;
            (
                ControlMoveSelector::Title(title.to_owned()),
                vec![path],
                room_uuid,
            )
        } else {
            bail!("Specify --type or --title to select controls to move");
        };
        let attributes = |elem: &Element| {
            elem.attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        };
        let selected: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let elem = self.get_element(&path);
                let io_data = elem
                    .children
                    .iter()
                    .enumerate()
                    .filter_map(|(index, child)| {
                        let child = child.as_element()?;
                        if child.name != "IoData" {
                            return None;
                        }
                        let mut io_path = path.clone();
                        io_path.push(index);
                        let before_pr = child.attributes.get("Pr").cloned();
                        Some(ControlMoveRoomReference {
                            path: io_path,
                            attributes: attributes(child),
                            created: before_pr.is_none(),
                            changed: before_pr.as_deref() != Some(&room_uuid),
                            before_pr,
                            after_pr: room_uuid.clone(),
                        })
                    })
                    .collect();
                ControlMoveSelection {
                    attributes: attributes(elem),
                    path,
                    io_data,
                }
            })
            .collect();
        let count = selected.len();
        let report = match &selector {
            ControlMoveSelector::Type(tf) => {
                format!("Moved {count} {tf} items to '{target_room}' ({room_uuid})")
            }
            ControlMoveSelector::Title(title) => {
                format!("Moved '{title}' to '{target_room}' ({room_uuid})")
            }
        };
        Ok(ControlMovePlan {
            preview: ControlMovePreview {
                changed: selected
                    .iter()
                    .any(|item| item.io_data.iter().any(|io| io.changed)),
                selector,
                exclude: exclude.iter().map(|value| (*value).to_owned()).collect(),
                target_room: target_room.to_owned(),
                room_uuid,
                selected,
                count,
                report,
                scope: "Existing CLI IoData.Pr attribute edit only; no native UI or physical-effect claim",
            },
        })
    }

    fn apply_control_move(&mut self, plan: &ControlMovePlan) {
        // Attribute-only edits leave pre-order paths stable, even for nested selections.
        for selected in &plan.preview.selected {
            for io in &selected.io_data {
                self.get_element_mut(&io.path)
                    .attributes
                    .insert("Pr".to_owned(), io.after_pr.clone());
            }
        }
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
        let plan = self.plan_room_add(name)?;
        self.apply_room_add(&plan);
        Ok(plan.preview.room.attributes["U"].clone())
    }

    pub fn preview_room_add(&self, name: &str) -> Result<RoomAddPreview> {
        Ok(self.plan_room_add(name)?.preview)
    }

    fn plan_room_add(&self, name: &str) -> Result<RoomAddPlan> {
        let mut existing = Vec::new();
        Self::walk_rooms(&self.root, &name.to_lowercase(), &mut existing);
        if !existing.is_empty() {
            bail!("Room '{}' already exists", name);
        }

        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            rand::random::<u32>(),
            rand::random::<u16>(),
            rand::random::<u16>()
        );

        // The legacy writer uses the last matching direct child, not a nested caption.
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

        let attributes = vec![
            ("Type".to_owned(), "Place".to_owned()),
            ("V".to_owned(), "175".to_owned()),
            ("U".to_owned(), uuid.clone()),
            ("Title".to_owned(), name.to_owned()),
            ("WF".to_owned(), "16384".to_owned()),
        ];
        let stored = |element: &Element, path| RoomAddElement {
            path,
            element_name: element.name.clone(),
            attributes: element
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        };
        Ok(RoomAddPlan {
            preview: RoomAddPreview {
                name: name.to_owned(),
                parent: stored(&self.root, vec![]),
                insert_after: insert_path.map(|index| {
                    stored(self.root.children[index].as_element().unwrap(), vec![index])
                }),
                room: RoomAddElement {
                    path: vec![insert_path.map_or(self.root.children.len(), |index| index + 1)],
                    element_name: "C".to_owned(),
                    attributes: attributes.iter().cloned().collect(),
                },
                uuid_ephemeral: true,
                report: format!("Added room '{}' (UUID: {})", name, uuid),
                scope: "Existing CLI Place insertion only; generated UUID is ephemeral across invocations, not reserved for execution; no native UI or physical-effect claim",
            },
            attributes,
        })
    }

    fn apply_room_add(&mut self, plan: &RoomAddPlan) {
        let mut place = Element::new(&plan.preview.room.element_name);
        place.attributes.extend(plan.attributes.iter().cloned());
        self.root
            .children
            .insert(plan.preview.room.path[0], xmltree::XMLNode::Element(place));
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
mod room_add_tests {
    use super::*;

    // Baseline add_room, with only its random UUID supplied by the test.
    fn legacy_add(editor: &mut ConfigEditor, name: &str, uuid: &str) -> Result<String> {
        let mut existing = Vec::new();
        ConfigEditor::walk_rooms(&editor.root, &name.to_lowercase(), &mut existing);
        if !existing.is_empty() {
            bail!("Room '{}' already exists", name);
        }
        let mut insert_path = None;
        for (i, child) in editor.root.children.iter().enumerate() {
            if let Some(elem) = child.as_element()
                && elem.name == "C"
                && let Some(t) = elem.attributes.get("Type")
                && (t == "Place" || t == "PlaceCaption")
            {
                insert_path = Some(i);
            }
        }
        let mut place = Element::new("C");
        place.attributes.insert("Type".into(), "Place".into());
        place.attributes.insert("V".into(), "175".into());
        place.attributes.insert("U".into(), uuid.into());
        place.attributes.insert("Title".into(), name.into());
        place.attributes.insert("WF".into(), "16384".into());
        if let Some(index) = insert_path {
            editor
                .root
                .children
                .insert(index + 1, xmltree::XMLNode::Element(place));
        } else {
            editor.root.children.push(xmltree::XMLNode::Element(place));
        }
        Ok(uuid.into())
    }

    fn assert_legacy_uuid(uuid: &str) {
        let parts: Vec<_> = uuid.split('-').collect();
        assert_eq!(
            parts.iter().map(|part| part.len()).collect::<Vec<_>>(),
            [8, 4, 4, 16]
        );
        assert_eq!(parts[3], "ffff000000000000");
        assert!(parts[..3].iter().all(|part| {
            part.bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }));
    }

    #[test]
    fn room_add_private_plan_applies_exact_identity_attributes_and_legacy_insertion() {
        for (xml, insertion, anchor) in [
            (
                r#"<ControlList Future="+003" U=""><C Type="PlaceCaption" U="first"/><C Type="Place" U="a" Title="Alpha"/><Unknown Ref="a"/><C Type="PlaceCaption" Title="" WF="+000"/><Tail/></ControlList>"#,
                4,
                Some(3),
            ),
            (
                r#"<ControlList><C Type="Place" Title="missing UUID"/><C Type="Place" U="" Title=""/><Tail/></ControlList>"#,
                2,
                Some(1),
            ),
            (
                r#"<ControlList><C Type="Page"><C Type="PlaceCaption"/><C Type="Place" U="a" Title="Alpha"/><IoData Pr="a" Cr="c"/><Co U="out" Def="+003"><In Input="ref" FLG="01"/></Co></C><Unknown/></ControlList>"#,
                2,
                None,
            ),
            (
                r#"<FutureRoot Title="" Future="opaque"><C Type="place"/><Other Type="Place"/><C/><C Type="Future"><Unknown/></C></FutureRoot>"#,
                4,
                None,
            ),
            (
                r#"<C Type="Place" U="root" Title="Root"><Unknown/></C>"#,
                1,
                None,
            ),
            (r#"<ControlList/>"#, 0, None),
        ] {
            let bytes = format!("\u{feff}{xml}\r\n");
            let name = "  Grüße,\"室\"  ";
            let mut editor = ConfigEditor::load(bytes.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            let before_bytes = editor.to_bytes().unwrap();
            let plan = editor.plan_room_add(name).unwrap();
            let preview = &plan.preview;
            assert_eq!(format!("{:?}", editor.root), before);
            assert_eq!(editor.to_bytes().unwrap(), before_bytes);
            assert_eq!(preview.parent.path, Vec::<usize>::new());
            assert_eq!(preview.parent.element_name, editor.root.name);
            assert_eq!(
                preview.parent.attributes,
                editor
                    .root
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            );
            assert_eq!(preview.room.path, [insertion]);
            assert_eq!(
                preview.insert_after.as_ref().map(|elem| elem.path[0]),
                anchor
            );
            if let Some(anchor) = &preview.insert_after {
                let actual = editor.get_element(&anchor.path);
                assert_eq!(anchor.element_name, actual.name);
                assert_eq!(
                    anchor.attributes,
                    actual
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                );
            }
            assert!(preview.uuid_ephemeral);
            let uuid = &preview.room.attributes["U"];
            assert_legacy_uuid(uuid);
            assert_eq!(
                preview.report,
                format!("Added room '{name}' (UUID: {uuid})")
            );
            let mut legacy = ConfigEditor::load(bytes.as_bytes()).unwrap();
            assert_eq!(legacy_add(&mut legacy, name, uuid).unwrap(), *uuid);
            editor.apply_room_add(&plan);
            let room = editor.get_element(&preview.room.path);
            assert_eq!(room.name, preview.room.element_name);
            assert!(room.children.is_empty());
            assert_eq!(
                room.attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<BTreeMap<_, _>>(),
                preview.room.attributes
            );
            assert_eq!(
                room.attributes
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["Type", "V", "U", "Title", "WF"]
            );
            assert_eq!(format!("{:?}", editor.root), format!("{:?}", legacy.root));
            assert_eq!(editor.to_bytes().unwrap(), legacy.to_bytes().unwrap());
            editor.root.children.remove(insertion);
            assert_eq!(
                format!("{:?}", editor.root),
                before,
                "all other DOM content and order must stay exact"
            );
            assert_eq!(editor.to_bytes().unwrap(), before_bytes);

            let public_preview = editor.preview_room_add(name).unwrap();
            assert_eq!(public_preview.room.path, preview.room.path);
            assert_eq!(format!("{:?}", editor.root), before);
            let uuid = editor.add_room(name).unwrap();
            assert_legacy_uuid(&uuid);
            let mut legacy = ConfigEditor::load(bytes.as_bytes()).unwrap();
            legacy_add(&mut legacy, name, &uuid).unwrap();
            assert_eq!(format!("{:?}", editor.root), format!("{:?}", legacy.root));
            assert_eq!(editor.to_bytes().unwrap(), legacy.to_bytes().unwrap());
        }
    }

    #[test]
    fn room_add_plan_preserves_substring_unicode_empty_and_legacy_duplicate_admission() {
        for (xml, name, duplicate) in [
            (
                r#"<ControlList><C Type="Place" U="a" Title="Kitchen Long"/></ControlList>"#,
                "kItChEn",
                true,
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Kitchen"/></ControlList>"#,
                "Kitchen Long",
                false,
            ),
            (
                r#"<ControlList><C Type="Page"><C Type="Place" U="a" Title="Nested"/></C></ControlList>"#,
                "est",
                true,
            ),
            (r#"<C Type="Place" U="a" Title="Root"/>"#, "root", true),
            (
                r#"<ControlList><C Type="Place" U="" Title="Alpha"/></ControlList>"#,
                "alpha",
                true,
            ),
            (
                r#"<ControlList><C Type="Place" Title="Alpha"/></ControlList>"#,
                "alpha",
                false,
            ),
            (
                r#"<ControlList><C Type="Place" U="a"/></ControlList>"#,
                "",
                false,
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title=""/></ControlList>"#,
                "",
                true,
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Anything"/></ControlList>"#,
                "",
                true,
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Küche"/></ControlList>"#,
                "KÜCHE",
                true,
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Küche"/></ControlList>"#,
                "Ku\u{308}che",
                false,
            ),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Straße"/></ControlList>"#,
                "STRASSE",
                false,
            ),
            (
                r#"<ControlList><C Type="place" U="a" Title="Alpha"/><Other Type="Place" U="b" Title="Alpha"/></ControlList>"#,
                "Alpha",
                false,
            ),
            (
                r#"<ControlList><C Type="PlaceCaption" Title="Alpha"/><C Type="Future" U="a" Title="Alpha"/></ControlList>"#,
                "Alpha",
                false,
            ),
            (r#"<ControlList/>"#, "", false),
            (r#"<ControlList/>"#, "  ", false),
            (r#"<ControlList/>"#, "uuid:raw [room]", false),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            let mut legacy = ConfigEditor::load(xml.as_bytes()).unwrap();
            let expected = legacy_add(&mut legacy, name, "test-only")
                .err()
                .map(|e| e.to_string());
            let preview = editor.preview_room_add(name);
            assert_eq!(preview.is_err(), duplicate, "{xml}: {name:?}");
            assert_eq!(preview.as_ref().err().map(ToString::to_string), expected);
            assert_eq!(format!("{:?}", editor.root), before);
            if duplicate {
                assert_eq!(
                    editor.add_room(name).unwrap_err().to_string(),
                    expected.unwrap()
                );
                assert_eq!(format!("{:?}", editor.root), before);
            } else {
                assert_eq!(preview.unwrap().room.attributes["Title"], name);
                let uuid = editor.add_room(name).unwrap();
                let mut legacy = ConfigEditor::load(xml.as_bytes()).unwrap();
                legacy_add(&mut legacy, name, &uuid).unwrap();
                assert_eq!(format!("{:?}", editor.root), format!("{:?}", legacy.root));
            }
        }
    }

    #[test]
    fn room_add_preview_keeps_raw_metadata_and_all_editor_state_without_defaults_or_stamp() {
        let bytes = b"\xef\xbb\xbf<FutureRoot U=\"\" Future=\"+003\" 1wire=\"raw\">\r\n<C Type=\"Document\" Date=\"old\" DateS=\"+000\"/>\r\n<C Type=\"PlaceCaption\" Title=\"\" WF=\"\"/>\r\n<C Type=\"Place\" U=\"a\" Title=\"Alpha\"/>\r\n</FutureRoot>\r\n";
        let editor = ConfigEditor::load(bytes).unwrap();
        let before = format!("{:?}", editor.root);
        let serialized = editor.to_bytes().unwrap();
        let renames = editor.digit_attr_renames.clone();
        let preview = editor.preview_room_add("β").unwrap();
        assert_eq!(
            preview.parent.attributes.get("U").map(String::as_str),
            Some("")
        );
        assert!(!preview.parent.attributes.contains_key("Type"));
        assert_eq!(preview.parent.attributes["Future"], "+003");
        let anchor = preview.insert_after.unwrap();
        assert_eq!(anchor.attributes["U"], "a");
        assert!(!anchor.attributes.contains_key("WF"));
        let preview = editor.preview_room_add("ALPHA").unwrap_err();
        assert_eq!(preview.to_string(), "Room 'ALPHA' already exists");
        assert_eq!(format!("{:?}", editor.root), before);
        assert_eq!(editor.to_bytes().unwrap(), serialized);
        assert!(editor.had_bom && editor.had_crlf);
        assert_eq!(editor.digit_attr_renames, renames);
    }
}

#[cfg(test)]
mod control_move_tests {
    use super::*;

    const XML: &str = r#"<ControlList Future="retain">
      <C Type="Place" U="a" Title="Alpha"/><C Type="Place" U="b" Title="Beta"/>
      <C Type="Place" U="long" Title="Beta Long"/>
      <C Type="Page" U="page" Title="Page"><IoData Pr="a" Cr="category"/>
        <C Type="Future" U="first" Title="Lamp" gid="LampGid" Future="+003">
          <IoData Pr="a" Cr="category" Future="01"/><IoData Pr="b"/>
          <Unknown><IoData Pr="retain"/></Unknown>
          <Co K="AQ" U="out" Def="+000"><In Input="ref" FLG="01"/></Co>
          <C Type="Future" U="nested" Title="Nested"><IoData Pr=""/><IoData Future="002"/></C>
        </C>
        <C Type="Future" U="second" Title="Lamp"><IoData Pr="b"/></C>
        <C Type="Future" U="dry" Title="Dry"><Unknown><IoData Pr="retain"/></Unknown></C>
        <C Type="Other" U="other" Title="Other"><IoData Pr="a"/></C>
      </C>
      <C Type="AutomaticDesigner"><C Type="Future" U="first" Title="Mirror"><IoData Pr="a"/></C></C>
    </ControlList>"#;

    fn legacy_move(
        editor: &mut ConfigEditor,
        type_filter: Option<&str>,
        title: Option<&str>,
        target: &str,
        exclude: &[&str],
    ) -> Result<String> {
        let (paths, room_uuid, report) = if let Some(tf) = type_filter {
            let room_uuid = editor.find_room_uuid(target)?;
            let mut paths = Vec::new();
            ConfigEditor::collect_typed_with_iodata(
                &editor.root,
                tf,
                exclude,
                &mut Vec::new(),
                &mut paths,
            );
            let report = format!(
                "Moved {} {} items to '{}' ({})",
                paths.len(),
                tf,
                target,
                room_uuid
            );
            (paths, room_uuid, report)
        } else if let Some(title) = title {
            let path = editor.require_one(title)?;
            let room_uuid = editor.find_room_uuid(target)?;
            let report = format!("Moved '{}' to '{}' ({})", title, target, room_uuid);
            (vec![path], room_uuid, report)
        } else {
            bail!("Specify --type or --title to select controls to move");
        };
        for path in paths {
            for child in &mut editor.get_element_mut(&path).children {
                if let Some(io) = child.as_mut_element()
                    && io.name == "IoData"
                {
                    io.attributes.insert("Pr".to_owned(), room_uuid.clone());
                }
            }
        }
        Ok(report)
    }

    #[test]
    fn control_move_plan_matches_legacy_branches_and_entire_dom() {
        for (tf, title, exclude) in [
            (Some("fUtUrE"), Some("nonexistent"), vec![]),
            (Some("Future"), None, vec!["FUTURE"]),
            (Some("Missing"), None, vec![]),
            (Some(""), None, vec![]),
            (Some("Type:Future"), None, vec![]),
            (None, Some("Lamp [Beta]"), vec![]),
            (None, Some("gid:lampgid"), vec!["Future"]),
            (None, Some("uuid:first"), vec![]),
            (None, Some("Mirror"), vec![]),
            (None, Some("Type:Page"), vec![]),
            (None, Some("Dry"), vec![]),
            (None, Some("Nested"), vec![]),
        ] {
            let mut editor = ConfigEditor::load(XML.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            let preview = editor
                .preview_control_move(tf, title, "bEtA", &exclude)
                .unwrap();
            assert_eq!(format!("{:?}", editor.root), before);
            assert_eq!(preview.room_uuid, "b");
            assert_eq!(preview.target_room, "bEtA");
            let mut planned = ConfigEditor::load(XML.as_bytes()).unwrap();
            for selected in &preview.selected {
                assert_eq!(
                    selected.attributes,
                    editor
                        .get_element(&selected.path)
                        .attributes
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect()
                );
                for io in &selected.io_data {
                    assert_eq!(
                        io.before_pr,
                        editor.get_element(&io.path).attributes.get("Pr").cloned()
                    );
                    assert_eq!(io.changed, io.before_pr.as_deref() != Some("b"));
                    assert_eq!(io.created, io.before_pr.is_none());
                    planned
                        .get_element_mut(&io.path)
                        .attributes
                        .insert("Pr".to_owned(), io.after_pr.clone());
                }
            }
            let mut legacy = ConfigEditor::load(XML.as_bytes()).unwrap();
            let report = legacy_move(&mut legacy, tf, title, "bEtA", &exclude).unwrap();
            assert_eq!(preview.report, report);
            assert_eq!(
                editor.move_control(tf, title, "bEtA", &exclude).unwrap(),
                report
            );
            assert_eq!(format!("{:?}", editor.root), format!("{:?}", legacy.root));
            assert_eq!(format!("{:?}", planned.root), format!("{:?}", legacy.root));
            assert_eq!(editor.to_bytes().unwrap(), legacy.to_bytes().unwrap());
            if let Some(tf) = tf {
                let mut helper = ConfigEditor::load(XML.as_bytes()).unwrap();
                assert_eq!(
                    helper.move_to_room(tf, "bEtA", &exclude).unwrap(),
                    (preview.count, "b".to_owned())
                );
                assert_eq!(format!("{:?}", helper.root), format!("{:?}", legacy.root));
            }
        }
    }

    #[test]
    fn control_move_plan_exposes_preorder_duplicates_missing_empty_and_noop() {
        let editor = ConfigEditor::load(XML.as_bytes()).unwrap();
        let preview = editor
            .preview_control_move(Some("Future"), None, "Beta", &[])
            .unwrap();
        let identities: Vec<_> = preview
            .selected
            .iter()
            .map(|item| item.attributes["U"].as_str())
            .collect();
        assert_eq!(identities, ["first", "nested", "second", "first"]);
        assert!(
            preview.selected[1]
                .path
                .starts_with(&preview.selected[0].path)
        );
        assert_eq!(preview.selected[0].io_data.len(), 2);
        assert_eq!(preview.selected[0].io_data[0].attributes["Future"], "01");
        assert_eq!(
            preview.selected[1].io_data[0].before_pr.as_deref(),
            Some("")
        );
        assert!(!preview.selected[1].io_data[0].created);
        assert_eq!(preview.selected[1].io_data[1].before_pr, None);
        assert!(preview.selected[1].io_data[1].created);
        assert!(!preview.selected[2].io_data[0].changed);
        for (tf, title, count) in [
            (Some("Missing"), None, 0),
            (None, Some("Dry"), 1),
            (None, Some("Lamp [Beta]"), 1),
        ] {
            let preview = editor.preview_control_move(tf, title, "Beta", &[]).unwrap();
            assert_eq!(preview.count, count);
            assert!(!preview.changed);
        }
        let empty = r#"<ControlList><C Type="Place" U="" Title=""/>
          <C Type="Future" Title="Raw"><IoData/><IoData Pr=""/></C></ControlList>"#;
        let mut editor = ConfigEditor::load(empty.as_bytes()).unwrap();
        let preview = editor
            .preview_control_move(None, Some("Raw"), "", &[])
            .unwrap();
        assert_eq!(preview.room_uuid, "");
        assert!(preview.selected[0].io_data[0].changed);
        assert!(!preview.selected[0].io_data[1].changed);
        assert_eq!(
            editor.move_control(None, Some("Raw"), "", &[]).unwrap(),
            "Moved 'Raw' to '' ()"
        );
    }

    #[test]
    fn control_move_plan_preserves_error_order_and_failure_nonmutation() {
        for (xml, tf, title, target, expected) in [
            (XML, None, None, "missing", "Specify --type or --title"),
            (
                XML,
                None,
                Some("Missing"),
                "missing",
                "Element 'Missing' not found",
            ),
            (
                XML,
                None,
                Some("Lamp"),
                "missing",
                "Selector 'Lamp' matches",
            ),
            (
                XML,
                Some("Missing"),
                Some("Lamp"),
                "missing",
                "Room 'missing' not found",
            ),
            (
                XML,
                None,
                Some("Dry"),
                "missing",
                "Room 'missing' not found",
            ),
            (XML, Some("Future"), None, "", "Multiple rooms match"),
            (
                r#"<ControlList><C Type="Place" U="a" Title="Same"/><C Type="Place" U="b" Title="Same"/></ControlList>"#,
                Some("Future"),
                None,
                "Same",
                "Multiple rooms match",
            ),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = format!("{:?}", editor.root);
            let preview_error = editor
                .preview_control_move(tf, title, target, &[])
                .unwrap_err()
                .to_string();
            let mut legacy = ConfigEditor::load(xml.as_bytes()).unwrap();
            let legacy_error = legacy_move(&mut legacy, tf, title, target, &[])
                .unwrap_err()
                .to_string();
            assert!(preview_error.contains(expected), "{preview_error}");
            assert_eq!(preview_error, legacy_error);
            assert_eq!(
                editor
                    .move_control(tf, title, target, &[])
                    .unwrap_err()
                    .to_string(),
                legacy_error
            );
            assert_eq!(format!("{:?}", editor.root), before);
        }
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
