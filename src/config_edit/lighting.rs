//! Stored lighting data and explicit mood-count repair; no inferred output defaults or roles.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::schema::Diagnostic;
use super::{ConfigEditor, MoodInfo};

type Attributes = BTreeMap<String, String>;

#[derive(Debug, Serialize)]
pub struct LightingStorage {
    pub element: String,
    pub attributes: Attributes,
    pub children: Vec<LightingStorage>,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodOutput {
    pub key: String,
    pub index: usize,
    pub stored: String,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LightingMood {
    pub summary: MoodInfo,
    pub attributes: Attributes,
    pub outputs: Vec<LightingMoodOutput>,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodCountEdit {
    pub before: Option<String>,
    pub after: String,
    pub scenes: usize,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct LightingWireTarget {
    pub block_uuid: Option<String>,
    pub block_type: Option<String>,
    pub block_title: Option<String>,
    pub connector_uuid: Option<String>,
    pub connector_key: Option<String>,
    pub wire_attributes: Attributes,
}

#[derive(Debug, Serialize)]
pub struct LightingChannel {
    pub key: String,
    pub index: usize,
    pub uuid: Option<String>,
    pub attributes: Attributes,
    pub role: Option<String>,
    pub downstream: Vec<LightingWireTarget>,
}

#[derive(Debug, Serialize)]
pub struct LightingInspection {
    pub status: &'static str,
    pub read_only: bool,
    pub gui_verified: bool,
    pub recovery_version: &'static str,
    pub recovery_target_sha256: &'static str,
    pub document_versions: Vec<Attributes>,
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub attributes: Attributes,
    pub channels: Vec<LightingChannel>,
    pub moods: Vec<LightingMood>,
    pub nested_storage: Vec<LightingStorage>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ConfigEditor {
    /// Set only the direct collection's Num to its direct scene-record count.
    ///
    /// Native append (0x1405b48e0) and reconciliation (0x1405c6770) assign
    /// Num from the scene-vector size, independently of either Outputs field.
    pub fn recount_lighting_moods(&mut self, selector: &str) -> Result<LightingMoodCountEdit> {
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                "Mood recount requires LightController2. Use `lox config controls FILE --type LightController2` to find one.",
            )
            .into());
        }
        let containers: Vec<_> = block
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| element.name == "LightscenesC")
                    .map(|element| (index, element))
            })
            .collect();
        let [(container_index, collection)] = containers.as_slice() else {
            return Err(Diagnostic::new(
                "unsupported_mood_container_shape",
                selector,
                format!(
                    "Mood recount requires one direct LightscenesC container, found {}. Inspect the XML before editing.",
                    containers.len()
                ),
            )
            .into());
        };
        let scenes = direct_mood_count(collection);
        let count = i32::try_from(scenes).map_err(|_| {
            Diagnostic::new(
                "mood_count_out_of_range",
                selector,
                "The direct scene count does not fit the native signed 32-bit Num field.",
            )
        })?;
        let before = collection.attributes.get("Num").cloned();
        let changed = before.as_ref().and_then(|raw| raw.parse::<i32>().ok()) != Some(count);
        let after = if changed {
            count.to_string()
        } else {
            before.as_ref().unwrap().clone()
        };
        let container_index = *container_index;
        if changed {
            let collection = self.get_element_mut(&path).children[container_index]
                .as_mut_element()
                .unwrap();
            collection.attributes.insert("Num".into(), after.clone());
        }
        Ok(LightingMoodCountEdit {
            before,
            after,
            scenes,
            changed,
        })
    }

    pub fn inspect_lighting(&self, selector: &str) -> Result<LightingInspection> {
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                "Lighting inspection requires LightController2. Use `lox config controls FILE --type LightController2` to find one.",
            )
            .into());
        }
        let mut diagnostics = vec![
            Diagnostic::new(
                "unverified_lighting_contract",
                selector,
                "Read-only stored XML inspection. Native GUI roundtrip and runtime parity are unverified; no nested setting is editable through this surface.",
            ),
            Diagnostic::new(
                "unknown_channel_roles",
                selector,
                "AQn and Qn suffixes are storage indices only. NameAI/CapAI/PresM metadata, FC, HCL, PSD, LSConfig and mood value units/encodings are not interpreted or joined to actuator roles.",
            ),
        ];
        let summaries = self.list_moods(selector)?;
        let mood_elements: Vec<_> = Self::iter_elements(block)
            .into_iter()
            .filter(|element| element.name == "LightsceneC")
            .collect();
        let mut moods = Vec::new();
        for (summary, element) in summaries.into_iter().zip(mood_elements) {
            check_identity(element, "UUID", "LightsceneC", &mut diagnostics);
            for key in ["SID", "CID"] {
                if element
                    .attributes
                    .get(key)
                    .is_some_and(|raw| raw.parse::<i64>().is_err())
                {
                    diagnostics.push(Diagnostic::new(
                        "invalid_mood_id",
                        &format!("LightsceneC/@{key}"),
                        "Stored ID is not an integer; raw spelling is retained in attributes.",
                    ));
                }
            }
            let mut outputs: Vec<_> = element
                .attributes
                .iter()
                .filter_map(|(key, stored)| {
                    storage_index(key, "Q").map(|index| LightingMoodOutput {
                        key: key.clone(),
                        index,
                        stored: stored.clone(),
                        unit: None,
                    })
                })
                .collect();
            outputs.sort_by_key(|output| output.index);
            moods.push(LightingMood {
                summary,
                attributes: attributes(element),
                outputs,
            });
        }
        let mut channels = Vec::new();
        for co in block.children.iter().filter_map(XMLNode::as_element) {
            let Some((key, index)) = co.attributes.get("K").and_then(|key| {
                (co.name == "Co")
                    .then(|| storage_index(key, "AQ").map(|index| (key, index)))
                    .flatten()
            }) else {
                continue;
            };
            check_identity(co, "U", key, &mut diagnostics);
            let uuid = nonempty(co, "U");
            let mut downstream = Vec::new();
            if let Some(uuid) = &uuid {
                let occurrences = Self::iter_elements(&self.root)
                    .into_iter()
                    .filter(|element| element.attributes.get("U") == Some(uuid))
                    .count();
                if occurrences != 1 {
                    diagnostics.push(Diagnostic::new(
                        "ambiguous_connector_uuid",
                        key,
                        format!("Connector U={uuid} appears {occurrences} times; downstream references cannot be uniquely attributed."),
                    ));
                }
                collect_targets(&self.root, uuid, None, None, &mut downstream);
                for target in &downstream {
                    if target.block_uuid.is_none() || target.connector_uuid.is_none() {
                        diagnostics.push(Diagnostic::new(
                            "unresolved_wire_target",
                            key,
                            "A matching In reference has no complete owning block/connector UUID. It is reported, not treated as a resolved actor.",
                        ));
                    }
                }
            }
            channels.push(LightingChannel {
                key: key.clone(),
                index,
                uuid,
                attributes: attributes(co),
                role: None,
                downstream,
            });
        }
        channels.sort_by_key(|channel| channel.index);
        let containers = block
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "LightscenesC")
            .count();
        if containers != 1 {
            diagnostics.push(Diagnostic::new(
                "unsupported_mood_container_shape",
                selector,
                format!("Expected one direct LightscenesC container, found {containers}; discovered mood records remain raw read-only data."),
            ));
        } else if let Some(collection) = block.get_child("LightscenesC") {
            let scenes = direct_mood_count(collection);
            let stored = collection
                .attributes
                .get("Num")
                .and_then(|raw| raw.parse::<i32>().ok());
            let count_matches =
                stored.and_then(|count| usize::try_from(count).ok()) == Some(scenes);
            if !count_matches {
                diagnostics.push(Diagnostic::new(
                    "inconsistent_mood_count",
                    "LightscenesC/@Num",
                    format!(
                        "Stored Num does not match {scenes} direct LightsceneC records. Use `lox config lighting recount-moods FILE BLOCK` to repair only Num; Outputs and sparse Qn remain untouched."
                    ),
                ));
            }
            if stored == Some(0) && scenes == 0 {
                diagnostics.push(Diagnostic::new(
                    "native_empty_mood_collection_reinitialized",
                    "LightscenesC",
                    "Config 17.2.10.21 reconciles an empty native scene vector after nested serialization on both read and write, creating default scenes in memory. An empty Num=0 collection is not a native load/save no-op. Inspection leaves all stored data unchanged.",
                ));
            }
            if count_matches {
                for (index, scene) in collection
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .filter(|element| element.name == "LightsceneC")
                    .enumerate()
                {
                    if scene
                        .attributes
                        .get("SID")
                        .and_then(|raw| raw.parse::<i32>().ok())
                        == Some(0)
                    {
                        diagnostics.push(Diagnostic::new(
                            "native_zero_mood_sid_reconciled",
                            &format!("LightscenesC/LightsceneC[{}]/@SID", index + 1),
                            "Stored Num matches the direct scene count. Config 17.2.10.21 can replace a loaded zero SID during reconciliation after nested serialization on both read and write. The allocator searches 1..99 and can return zero when exhausted; this inspector neither allocates an ID nor simulates native saving.",
                        ));
                    }
                }
            }
        }
        check_identity(block, "U", selector, &mut diagnostics);
        let document_versions = Self::iter_elements(&self.root)
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
            status: "read_only_unverified",
            read_only: true,
            gui_verified: false,
            recovery_version: "17.2.10.21",
            recovery_target_sha256: "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3",
            document_versions,
            uuid: nonempty(block, "U"),
            title: nonempty(block, "Title"),
            attributes: attributes(block),
            channels,
            moods,
            nested_storage: block
                .children
                .iter()
                .filter_map(XMLNode::as_element)
                .filter(|element| element.name != "Co")
                .map(storage)
                .collect(),
            diagnostics,
        })
    }
}

pub(super) fn direct_mood_count(collection: &Element) -> usize {
    collection
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|element| element.name == "LightsceneC")
        .count()
}

fn attributes(element: &Element) -> Attributes {
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

fn check_identity(element: &Element, key: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if nonempty(element, key).is_none() {
        diagnostics.push(Diagnostic::new(
            "missing_identity",
            path,
            format!("Missing or empty @{key}; no replacement ID is generated."),
        ));
    }
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
    targets: &mut Vec<LightingWireTarget>,
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
        targets.push(LightingWireTarget {
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
