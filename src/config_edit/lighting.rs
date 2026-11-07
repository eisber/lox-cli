//! Stored lighting data only: no inferred channel role, default or runtime value.

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
