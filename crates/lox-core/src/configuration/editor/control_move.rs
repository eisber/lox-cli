use super::{EditError, legacy::State};
use chrono::{DateTime, FixedOffset};
#[cfg(feature = "unstable-legacy-cli-editor")]
use std::collections::BTreeMap;
use xmltree::Element;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlMoveSelector {
    Type(String),
    Title(String),
}

#[derive(Debug)]
pub struct ControlMoveChange {
    pub selector: ControlMoveSelector,
    pub target_room: String,
    pub room_uuid: String,
    /// Selected elements, including selections whose association is already correct.
    pub selected_count: usize,
    pub association_changed: bool,
}

#[derive(Debug)]
pub struct ControlMoveOutcome {
    pub bytes: Vec<u8>,
    pub change: ControlMoveChange,
    /// Compared with an unedited canonical serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Move existing direct `IoData.Pr` associations using the legacy selection rules.
///
/// A supplied type takes precedence over a title and resolves the room first.
/// Type matching is ASCII-insensitive, retains UUID mirrors and requires direct
/// `IoData`; title selection uses the shared selector first and ignores exclusions.
/// Neither branch creates `IoData`. Missing and empty `Pr` remain distinct.
///
/// The explicit clock and canonical comparison use the same stamp/serializer as
/// [`super::set_param`]. A no-op can normalize raw bytes without stamping. Errors
/// return no revised bytes; no files, contexts or devices are accessed.
pub fn move_control(
    bytes: &[u8],
    type_filter: Option<&str>,
    title: Option<&str>,
    target_room: &str,
    exclude: &[&str],
    now: &DateTime<FixedOffset>,
) -> Result<ControlMoveOutcome, EditError> {
    let mut state = State::load(bytes)?;
    let plan = state.move_control(type_filter, title, target_room, exclude)?;
    let mut output = state.to_bytes()?;
    let baseline = State::load(bytes)?.to_bytes()?;
    let content_changed = baseline != output;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    Ok(ControlMoveOutcome {
        bytes: output,
        change: ControlMoveChange {
            selector: plan.selector,
            target_room: plan.target_room,
            room_uuid: plan.room_uuid,
            selected_count: plan.count,
            association_changed: plan.changed,
        },
        content_changed,
        document_stamped,
    })
}

/// Value-only snapshot for the unstable legacy CLI preview adapter, not a DOM.
#[doc(hidden)]
#[derive(Debug)]
pub struct ControlMovePlan {
    pub selector: ControlMoveSelector,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub exclude: Vec<String>,
    pub target_room: String,
    pub room_uuid: String,
    pub selected: Vec<ControlMoveSelection>,
    pub count: usize,
    pub changed: bool,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ControlMoveSelection {
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub path: Vec<usize>,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub attributes: BTreeMap<String, String>,
    pub io_data: Vec<ControlMoveRoomReference>,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ControlMoveRoomReference {
    pub path: Vec<usize>,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub attributes: BTreeMap<String, String>,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub before_pr: Option<String>,
    pub after_pr: String,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub created: bool,
    pub changed: bool,
}

impl State {
    pub fn plan_control_move(
        &self,
        type_filter: Option<&str>,
        title: Option<&str>,
        target_room: &str,
        exclude: &[&str],
    ) -> Result<ControlMovePlan, EditError> {
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
            return Err(EditError::ControlMoveSelectorMissing);
        };
        #[cfg(feature = "unstable-legacy-cli-editor")]
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
                            #[cfg(feature = "unstable-legacy-cli-editor")]
                            attributes: attributes(child),
                            #[cfg(feature = "unstable-legacy-cli-editor")]
                            created: before_pr.is_none(),
                            changed: before_pr.as_deref() != Some(&room_uuid),
                            #[cfg(feature = "unstable-legacy-cli-editor")]
                            before_pr,
                            after_pr: room_uuid.clone(),
                        })
                    })
                    .collect();
                ControlMoveSelection {
                    #[cfg(feature = "unstable-legacy-cli-editor")]
                    attributes: attributes(elem),
                    #[cfg(feature = "unstable-legacy-cli-editor")]
                    path,
                    io_data,
                }
            })
            .collect();
        Ok(ControlMovePlan {
            changed: selected
                .iter()
                .any(|item| item.io_data.iter().any(|io| io.changed)),
            selector,
            #[cfg(feature = "unstable-legacy-cli-editor")]
            exclude: exclude.iter().map(|value| (*value).to_owned()).collect(),
            target_room: target_room.to_owned(),
            room_uuid,
            count: selected.len(),
            selected,
        })
    }

    pub fn move_control(
        &mut self,
        type_filter: Option<&str>,
        title: Option<&str>,
        target_room: &str,
        exclude: &[&str],
    ) -> Result<ControlMovePlan, EditError> {
        let plan = self.plan_control_move(type_filter, title, target_room, exclude)?;
        self.apply_control_move(&plan);
        Ok(plan)
    }

    fn apply_control_move(&mut self, plan: &ControlMovePlan) {
        // Attribute-only edits leave pre-order paths stable for nested selections.
        for selected in &plan.selected {
            for io in &selected.io_data {
                self.get_element_mut(&io.path)
                    .attributes
                    .insert("Pr".to_owned(), io.after_pr.clone());
            }
        }
    }

    pub fn find_room_uuid(&self, room_name: &str) -> Result<String, EditError> {
        let lower = room_name.to_lowercase();
        let found = self.matching_rooms(&lower);
        match found.len() {
            0 => Err(EditError::RoomNotFound {
                room: room_name.to_owned(),
            }),
            1 => Ok(found.into_iter().next().unwrap().0),
            _ => {
                let exact: Vec<_> = found
                    .iter()
                    .filter(|(_uuid, name)| name.to_lowercase() == lower)
                    .collect();
                if exact.len() == 1 {
                    Ok(exact[0].0.clone())
                } else {
                    Err(EditError::RoomAmbiguous {
                        room: room_name.to_owned(),
                        matches: found.into_iter().map(|(_, name)| name).collect(),
                    })
                }
            }
        }
    }

    pub fn matching_rooms(&self, name_lower: &str) -> Vec<(String, String)> {
        let mut found = Vec::new();
        Self::walk_rooms(&self.root, name_lower, &mut found);
        found
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
            && elem
                .children
                .iter()
                .any(|c| c.as_element().map(|e| e.name == "IoData").unwrap_or(false))
        {
            results.push(path.clone());
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
}
