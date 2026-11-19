use super::{EditError, legacy::State};
use std::{error::Error, fmt};
use xmltree::{Element, XMLNode};

/// One stored light scene ("mood"), without interpreting its output values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoodInfo {
    /// Missing and explicitly empty names both appear as an empty string.
    pub name: String,
    /// Missing and explicitly empty UUIDs both appear as an empty string.
    pub uuid: String,
    /// Parsed scene ID, if present and valid as an `i64`.
    pub sid: Option<i64>,
    /// Parsed color ID, if present and valid as an `i64`.
    pub cid: Option<i64>,
    /// Stored `Q1` from the loaded DOM, with absence distinct from empty.
    pub q1: Option<String>,
}

/// Failure to read moods from one selected Lighting Controller.
#[derive(Debug)]
pub enum MoodInventoryError {
    /// The legacy editor failed to load the configuration or resolve the selector.
    Edit(EditError),
    /// The selected element's `Type` is not exactly `LightController2`.
    NotLightController {
        selector: String,
        /// The stored type, or an empty string if absent.
        block_type: String,
    },
}

impl fmt::Display for MoodInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::NotLightController {
                selector,
                block_type,
            } => write!(
                f,
                "'{selector}' is a {block_type}, not a LightController2 — moods only exist on light controllers"
            ),
        }
    }
}

impl Error for MoodInventoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::NotLightController { .. } => None,
        }
    }
}

/// List stored moods on one exactly `LightController2` selected by the legacy editor.
///
/// Loading and selection precede the exact type check. The selected subtree is
/// visited depth first: every parsed element named `LightsceneC` is included in
/// document order, without requiring a direct container or deduplicating records.
/// An eligible controller with no records returns an empty list. This does not
/// decode packed color values, validate a scene schema, or serialize the input.
/// The legacy loader limitation for digit-name/quoted-text handling (lox-n37w)
/// is unchanged.
pub fn list_moods(bytes: &[u8], selector: &str) -> Result<Vec<MoodInfo>, MoodInventoryError> {
    State::load(bytes)
        .map_err(MoodInventoryError::Edit)?
        .list_moods_selected(selector)
}

impl State {
    pub fn list_moods_selected(&self, selector: &str) -> Result<Vec<MoodInfo>, MoodInventoryError> {
        let element = self.select_light_controller(selector)?;
        let mut moods = Vec::new();
        visit_moods(element, &mut |mood| moods.push(mood_summary(mood)));
        Ok(moods)
    }

    pub(super) fn select_light_controller(
        &self,
        selector: &str,
    ) -> Result<&Element, MoodInventoryError> {
        let path = self
            .require_one(selector)
            .map_err(|error| MoodInventoryError::Edit(error.into()))?;
        let element = self.get_element(&path);
        let block_type = element.attributes.get("Type").cloned().unwrap_or_default();
        if block_type != "LightController2" {
            return Err(MoodInventoryError::NotLightController {
                selector: selector.to_owned(),
                block_type,
            });
        }
        Ok(element)
    }
}

pub(super) fn visit_moods(element: &Element, visit: &mut impl FnMut(&Element)) {
    if element.name == "LightsceneC" {
        visit(element);
    }
    for child in element.children.iter().filter_map(XMLNode::as_element) {
        visit_moods(child, visit);
    }
}

pub(super) fn mood_summary(element: &Element) -> MoodInfo {
    MoodInfo {
        name: element.attributes.get("Name").cloned().unwrap_or_default(),
        uuid: element.attributes.get("UUID").cloned().unwrap_or_default(),
        sid: element.attributes.get("SID").and_then(|s| s.parse().ok()),
        cid: element.attributes.get("CID").and_then(|s| s.parse().ok()),
        q1: element.attributes.get("Q1").cloned(),
    }
}
