use super::{EditError, legacy::State};
use chrono::{DateTime, FixedOffset};
#[cfg(feature = "unstable-legacy-cli-editor")]
use std::collections::BTreeMap;
use xmltree::Element;

/// The three independent legacy random draws, in their original order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomAddEntropy {
    pub first: u32,
    pub second: u16,
    pub third: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomAddChange {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug)]
pub struct RoomAddOutcome {
    pub bytes: Vec<u8>,
    pub change: RoomAddChange,
    /// Compared with an unedited canonical serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Add a legacy `Place` with explicit entropy and save time.
///
/// The entropy supplier is called exactly once after load and duplicate admission,
/// never on either refusal. The caller owns randomness; Core only formats the
/// three supplied values as the legacy native35 identifier. Collisions are not
/// checked or repaired. Names are not trimmed or validated.
///
/// Duplicate admission uses the shared recursive lowercase substring walker,
/// requiring both `U` and `Title` on an exact `C Type="Place"`. Insertion is after
/// the last matching *direct* Place/PlaceCaption, or at the end of the root.
/// Canonicalization, explicit-clock stamping and lox-n37w match the existing
/// editor; this is not a raw-lossless XML transaction. Errors return no bytes.
pub fn add_room(
    bytes: &[u8],
    name: &str,
    entropy: impl FnOnce() -> RoomAddEntropy,
    now: &DateTime<FixedOffset>,
) -> Result<RoomAddOutcome, EditError> {
    let mut state = State::load(bytes)?;
    let plan = state.plan_room_add(name, entropy)?;
    state.apply_room_add(&plan);
    let mut output = state.to_bytes()?;
    let baseline = State::load(bytes)?.to_bytes()?;
    let content_changed = baseline != output;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    Ok(RoomAddOutcome {
        bytes: output,
        change: RoomAddChange {
            name: plan.name,
            uuid: plan.uuid,
        },
        content_changed,
        document_stamped,
    })
}

#[cfg(feature = "unstable-legacy-cli-editor")]
#[doc(hidden)]
#[derive(Debug)]
pub struct RoomAddElement {
    pub path: Vec<usize>,
    pub element_name: String,
    pub attributes: BTreeMap<String, String>,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct RoomAddPlan {
    pub name: String,
    pub uuid: String,
    insertion: usize,
    attributes: Vec<(String, String)>,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub parent: RoomAddElement,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub insert_after: Option<RoomAddElement>,
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub room: RoomAddElement,
}

impl State {
    pub fn plan_room_add(
        &self,
        name: &str,
        entropy: impl FnOnce() -> RoomAddEntropy,
    ) -> Result<RoomAddPlan, EditError> {
        if !self.matching_rooms(&name.to_lowercase()).is_empty() {
            return Err(EditError::RoomAlreadyExists {
                room: name.to_owned(),
            });
        }
        let entropy = entropy();
        let uuid = format!(
            "{:08x}-{:04x}-{:04x}-ffff000000000000",
            entropy.first, entropy.second, entropy.third
        );
        let mut insert_after = None;
        for (index, child) in self.root.children.iter().enumerate() {
            if let Some(element) = child.as_element()
                && element.name == "C"
                && let Some(kind) = element.attributes.get("Type")
                && (kind == "Place" || kind == "PlaceCaption")
            {
                insert_after = Some(index);
            }
        }
        let insertion = insert_after.map_or(self.root.children.len(), |index| index + 1);
        let attributes = vec![
            ("Type".to_owned(), "Place".to_owned()),
            ("V".to_owned(), "175".to_owned()),
            ("U".to_owned(), uuid.clone()),
            ("Title".to_owned(), name.to_owned()),
            ("WF".to_owned(), "16384".to_owned()),
        ];
        #[cfg(feature = "unstable-legacy-cli-editor")]
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
            #[cfg(feature = "unstable-legacy-cli-editor")]
            parent: stored(&self.root, vec![]),
            #[cfg(feature = "unstable-legacy-cli-editor")]
            insert_after: insert_after
                .map(|index| stored(self.root.children[index].as_element().unwrap(), vec![index])),
            #[cfg(feature = "unstable-legacy-cli-editor")]
            room: RoomAddElement {
                path: vec![insertion],
                element_name: "C".to_owned(),
                attributes: attributes.iter().cloned().collect(),
            },
            name: name.to_owned(),
            uuid,
            insertion,
            attributes,
        })
    }

    pub fn apply_room_add(&mut self, plan: &RoomAddPlan) {
        let mut room = Element::new("C");
        room.attributes.extend(plan.attributes.iter().cloned());
        self.root
            .children
            .insert(plan.insertion, xmltree::XMLNode::Element(room));
    }
}
