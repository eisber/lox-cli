use super::{
    EditError, MoodInfo, MoodInventoryError,
    legacy::State,
    moods::{mood_summary, visit_moods},
};
use std::{collections::BTreeMap, error::Error, fmt};
use xmltree::{Element, XMLNode};

/// A selected controller's owned, stored configuration observations.
#[derive(Debug, PartialEq, Eq)]
pub struct LightingInspection {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub attributes: BTreeMap<String, String>,
    /// Direct indexed `Co` elements in document order.
    pub channels: Vec<StoredLightingChannel>,
    /// All `LightsceneC` elements in depth-first order, including nested lookalikes.
    pub moods: Vec<StoredLightingMood>,
    /// The non-`Co` direct element children, in document order.
    pub nested_storage: Vec<LightingStorage>,
    pub direct_scene_collection_count: usize,
    /// Index into `nested_storage` when the legacy named-child lookup finds
    /// the sole direct `LightscenesC`.
    pub sole_scene_collection: Option<usize>,
    /// Stored `Version`, `ConfigVersion` and `V` attributes on each `ControlList`
    /// and `Type="Document"` element, in depth-first document order.
    pub document_versions: Vec<BTreeMap<String, String>>,
}

/// An element-name and attribute projection, without text, comments or defaults.
#[derive(Debug, PartialEq, Eq)]
pub struct LightingStorage {
    pub element: String,
    pub attributes: BTreeMap<String, String>,
    pub children: Vec<LightingStorage>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredLightingMood {
    pub summary: MoodInfo,
    pub attributes: BTreeMap<String, String>,
    /// True only for a direct mood in exactly one direct `LightscenesC`.
    pub in_sole_direct_collection: bool,
    /// Present indexed `Qn` attributes in loader order, without decoding.
    pub outputs: Vec<StoredLightingOutput>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredLightingOutput {
    pub key: String,
    pub index: usize,
    pub stored: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredLightingChannel {
    pub key: String,
    pub index: usize,
    pub uuid: Option<String>,
    pub attributes: BTreeMap<String, String>,
    /// Count of elements carrying the same stored `U`, if nonempty.
    pub uuid_occurrences: Option<usize>,
    /// Exact `In/@Input` matches in document order, not resolved actor bindings.
    pub downstream: Vec<StoredLightingWire>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredLightingWire {
    pub block_uuid: Option<String>,
    pub block_type: Option<String>,
    pub block_title: Option<String>,
    pub connector_uuid: Option<String>,
    pub connector_key: Option<String>,
    pub wire_attributes: BTreeMap<String, String>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LightingInspectionError {
    /// The shared editor could not load the document or select one element.
    Edit(EditError),
    NotLightController {
        selector: String,
        /// The stored type, or an empty string if absent.
        block_type: String,
    },
}

impl fmt::Display for LightingInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::NotLightController {
                selector,
                block_type,
            } => write!(
                f,
                "Lighting inspection requires LightController2; '{selector}' has type '{block_type}'"
            ),
        }
    }
}

impl Error for LightingInspectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::NotLightController { .. } => None,
        }
    }
}

/// Inspect the stored XML projection for one legacy-selected `LightController2`.
///
/// The legacy loader limitation for digit-name/quoted-text handling (lox-n37w)
/// is unchanged. Attribute values are from that loader, not raw XML bytes.
/// The projection does not classify editors, decode colors, infer roles or units,
/// validate native behavior, modify the configuration, or access the filesystem.
pub fn inspect_lighting(
    bytes: &[u8],
    selector: &str,
) -> Result<LightingInspection, LightingInspectionError> {
    State::load(bytes)
        .map_err(LightingInspectionError::Edit)?
        .inspect_lighting_selected(selector)
}

impl State {
    pub fn inspect_lighting_selected(
        &self,
        selector: &str,
    ) -> Result<LightingInspection, LightingInspectionError> {
        let block = self
            .select_light_controller(selector)
            .map_err(|error| match error {
                MoodInventoryError::Edit(error) => LightingInspectionError::Edit(error),
                MoodInventoryError::NotLightController {
                    selector,
                    block_type,
                } => LightingInspectionError::NotLightController {
                    selector,
                    block_type,
                },
            })?;

        let nested_elements: Vec<_> = block
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name != "Co")
            .collect();
        let collections: Vec<_> = nested_elements
            .iter()
            .copied()
            .filter(|element| element.name == "LightscenesC")
            .collect();
        let direct_scene_collection_count = collections.len();
        let collection = match collections.as_slice() {
            [collection] => Some(*collection),
            _ => None,
        };
        // The CLI's collection diagnostics use the named-child lookup separately.
        let sole_scene_collection = collection
            .and_then(|_| block.get_child("LightscenesC"))
            .and_then(|collection| {
                nested_elements
                    .iter()
                    .position(|element| std::ptr::eq(*element, collection))
            });

        let mut moods = Vec::new();
        visit_moods(block, &mut |element| {
            let in_sole_direct_collection = collection.is_some_and(|collection| {
                collection
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .any(|child| std::ptr::eq(child, element))
            });
            let outputs = element
                .attributes
                .iter()
                .filter_map(|(key, stored)| {
                    storage_index(key, "Q").map(|index| StoredLightingOutput {
                        key: key.clone(),
                        index,
                        stored: stored.clone(),
                    })
                })
                .collect();
            moods.push(StoredLightingMood {
                summary: mood_summary(element),
                attributes: attributes(element),
                in_sole_direct_collection,
                outputs,
            });
        });

        let elements = Self::iter_elements(&self.root);
        let mut channels = Vec::new();
        for co in block
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "Co")
        {
            let Some((key, index)) = co
                .attributes
                .get("K")
                .and_then(|key| storage_index(key, "AQ").map(|index| (key, index)))
            else {
                continue;
            };
            let uuid = nonempty(co, "U");
            let mut downstream = Vec::new();
            let uuid_occurrences = uuid.as_ref().map(|uuid| {
                collect_targets(&self.root, uuid, None, None, &mut downstream);
                elements
                    .iter()
                    .filter(|element| element.attributes.get("U") == Some(uuid))
                    .count()
            });
            channels.push(StoredLightingChannel {
                key: key.clone(),
                index,
                uuid,
                attributes: attributes(co),
                uuid_occurrences,
                downstream,
            });
        }
        let document_versions = elements
            .into_iter()
            .filter(|element| {
                element.name == "ControlList"
                    || element.attributes.get("Type").map(String::as_str) == Some("Document")
            })
            .map(|element| {
                element
                    .attributes
                    .iter()
                    .filter(|(key, _)| matches!(key.as_str(), "Version" | "ConfigVersion" | "V"))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            })
            .collect();

        Ok(LightingInspection {
            uuid: nonempty(block, "U"),
            title: nonempty(block, "Title"),
            attributes: attributes(block),
            channels,
            moods,
            nested_storage: nested_elements.into_iter().map(storage).collect(),
            direct_scene_collection_count,
            sole_scene_collection,
            document_versions,
        })
    }
}

fn attributes(element: &Element) -> BTreeMap<String, String> {
    element
        .attributes
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn storage(element: &Element) -> LightingStorage {
    LightingStorage {
        element: element.name.clone(),
        attributes: attributes(element),
        children: element
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .map(storage)
            .collect(),
    }
}

fn nonempty(element: &Element, key: &str) -> Option<String> {
    element
        .attributes
        .get(key)
        .filter(|value| !value.is_empty())
        .cloned()
}

fn storage_index(key: &str, prefix: &str) -> Option<usize> {
    let suffix = key.strip_prefix(prefix)?;
    if suffix.is_empty()
        || !suffix.bytes().all(|byte| byte.is_ascii_digit())
        || suffix.starts_with('0')
    {
        return None;
    }
    suffix.parse().ok()
}

fn collect_targets(
    element: &Element,
    source_uuid: &str,
    block: Option<&Element>,
    connector: Option<&Element>,
    targets: &mut Vec<StoredLightingWire>,
) {
    let (block, connector) = if element.name == "C" {
        (Some(element), None)
    } else if element.name == "Co" {
        (block, Some(element))
    } else {
        (block, connector)
    };
    if element.name == "In"
        && element.attributes.get("Input").map(String::as_str) == Some(source_uuid)
    {
        targets.push(StoredLightingWire {
            block_uuid: block.and_then(|block| nonempty(block, "U")),
            block_type: block.and_then(|block| nonempty(block, "Type")),
            block_title: block.and_then(|block| nonempty(block, "Title")),
            connector_uuid: connector.and_then(|co| nonempty(co, "U")),
            connector_key: connector.and_then(|co| nonempty(co, "K")),
            wire_attributes: attributes(element),
        });
    }
    for child in element.children.iter().filter_map(XMLNode::as_element) {
        collect_targets(child, source_uuid, block, connector, targets);
    }
}
