//! Stored lighting data and bounded editor kinds; no inferred physical output roles.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::schema::{Diagnostic, StoredValue};
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
    pub editor: LightingMoodEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LightingEditorKind {
    Numeric,
    Color,
    Other,
    Unresolved,
}

impl LightingEditorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Numeric => "numeric",
            Self::Color => "color",
            Self::Other => "other",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LightingMoodEditor {
    pub kind: LightingEditorKind,
    pub type_tag: Option<i32>,
    pub capability: Option<StoredValue<u16>>,
    pub issue: Option<Diagnostic>,
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
pub struct LightingMoodLevelEdit {
    pub mood: String,
    pub output: String,
    pub before: String,
    pub after: String,
    pub level: u8,
    pub type_tag: i32,
    pub capability: Option<StoredValue<u16>>,
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
    /// Encode a numeric mood level without copying the native dialog's other edits.
    pub fn set_lighting_mood_level(
        &mut self,
        selector: &str,
        mood_ref: &str,
        output: usize,
        level: u32,
    ) -> Result<LightingMoodLevelEdit> {
        let level = u8::try_from(level)
            .ok()
            .filter(|level| *level <= 100)
            .ok_or_else(|| {
                Diagnostic::new(
                    "mood_level_out_of_range",
                    "level",
                    "Numeric mood levels must be integers in 0..100. No value was clamped or written.",
                )
            })?;
        if !(1..=18).contains(&output) {
            return Err(Diagnostic::new(
                "unsupported_mood_output_index",
                "output-index",
                "This recovered numeric editor path supports Q1..Q18. Use a 1-based --output-index within that bound.",
            )
            .into());
        }
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                "Numeric mood edits require LightController2. Inspect the selected block before editing.",
            )
            .into());
        }
        let collections: Vec<_> = block
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| element.name == "LightscenesC")
                    .map(|element| (index, element))
            })
            .collect();
        let [(collection_index, collection)] = collections.as_slice() else {
            return Err(Diagnostic::new(
                "unsupported_mood_container_shape",
                selector,
                "Numeric mood edits require exactly one direct LightscenesC container. Inspect the XML before editing.",
            )
            .into());
        };
        let matches: Vec<_> = collection
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| {
                        element.name == "LightsceneC"
                            && ["SID", "Name"].iter().any(|key| {
                                element.attributes.get(*key).map(String::as_str) == Some(mood_ref)
                            })
                    })
                    .map(|element| (index, element))
            })
            .collect();
        let [(mood_index, mood)] = matches.as_slice() else {
            return Err(Diagnostic::new(
                "ambiguous_or_missing_mood",
                mood_ref,
                format!(
                    "Expected one direct mood matching SID or Name, found {}. Choose an unambiguous stored mood.",
                    matches.len()
                ),
            )
            .into());
        };
        let width = mood_output_width(mood, mood_ref)?;
        let key = format!("Q{output}");
        let before = mood.attributes.get(&key).cloned().ok_or_else(|| {
            Diagnostic::new(
                "unsupported_mood_output_value",
                &key,
                "The selected Qn must already be stored. Missing values are not synthesized by a numeric edit.",
            )
        })?;
        let stored = before.parse::<i32>().map_err(|_| {
            Diagnostic::new(
                "unsupported_mood_output_value",
                &key,
                "Expected a stored signed-32-bit decimal. Opaque or out-of-range values are not interpreted.",
            )
        })?;
        if i32::try_from(output)? > width {
            return Err(Diagnostic::new(
                "unsupported_mood_output_width",
                &key,
                "Selected Qn lies outside the stored native reader width. No output vector was resized.",
            )
            .into());
        }
        let type_tag = (stored >> 28) & 15;
        let capability = Self::numeric_mood_capability(block, output, type_tag)?;
        // Native numeric handler 0x1400e6460 preserves the tag, not RGB packing.
        let encoded = (stored & -0x1000_0000) | i32::from(level);
        let changed = encoded != stored;
        let after = if changed {
            encoded.to_string()
        } else {
            before.clone()
        };
        let collection_index = *collection_index;
        let mood_index = *mood_index;
        if changed {
            let mood = self.get_element_mut(&path).children[collection_index]
                .as_mut_element()
                .unwrap()
                .children[mood_index]
                .as_mut_element()
                .unwrap();
            mood.attributes.insert(key.clone(), after.clone());
        }
        Ok(LightingMoodLevelEdit {
            mood: mood_ref.into(),
            output: key,
            before,
            after,
            level,
            type_tag,
            capability,
            changed,
        })
    }

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

    fn numeric_mood_capability(
        block: &Element,
        output: usize,
        type_tag: i32,
    ) -> Result<Option<StoredValue<u16>>> {
        let editor = classify_mood_editor(block, output, type_tag);
        if let Some(issue) = editor.issue {
            return Err(issue.into());
        }
        if editor.kind == LightingEditorKind::Numeric {
            return Ok(editor.capability);
        }
        Err(Diagnostic::new(
            "incompatible_mood_output_kind",
            &format!("Q{output}"),
            format!(
                "Stored type tag {type_tag} and its needed capability do not select the recovered numeric editor. Inspect the channel settings; packed-color, switch and unresolved modes are not converted to numeric levels."
            ),
        )
        .into())
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
                "AQn and Qn suffixes are storage indices only. Stored per-mood editor classification uses Q type tags and needed CapAI, not physical channel roles, units or graph-dependent pre-modal refresh. Other metadata remains uninterpreted.",
            ),
            Diagnostic::new(
                "lighting_edit_operation_boundary",
                selector,
                "Use `config lighting set-mood-level` for supported numeric editor kinds; it validates the target before writing. `config set-mood-color` remains a raw packed-value edit, not a mode-aware equivalent, even for a color-classified output.",
            ),
        ];
        let collections: Vec<_> = block
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "LightscenesC")
            .collect();
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
            let direct = matches!(collections.as_slice(), [collection] if collection.children
                .iter()
                .filter_map(XMLNode::as_element)
                .any(|child| std::ptr::eq(child, element)));
            let mut outputs: Vec<_> = element
                .attributes
                .iter()
                .filter_map(|(key, stored)| {
                    storage_index(key, "Q").map(|index| {
                        let editor = inspect_mood_editor(block, element, index, stored, direct);
                        if let Some(issue) = &editor.issue {
                            let mut issue = issue.clone();
                            issue.path = format!(
                                "mood '{}' ({}) / {}",
                                summary.name, summary.uuid, issue.path
                            );
                            diagnostics.push(issue);
                        }
                        LightingMoodOutput {
                            key: key.clone(),
                            index,
                            stored: stored.clone(),
                            unit: None,
                            editor,
                        }
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
            inspect_duplicate_mood_ids(collection, &mut diagnostics);
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

fn mood_output_width(mood: &Element, path: &str) -> Result<i32, Diagnostic> {
    match mood.attributes.get("Outputs") {
        None => Ok(12),
        Some(raw) => match raw.parse::<i32>() {
            Ok(0) => Ok(12),
            Ok(width) if width > 0 => Ok(width),
            _ => Err(Diagnostic::new(
                "unsupported_mood_output_width",
                path,
                "Outputs must be a nonnegative signed-32-bit decimal or absent. No width was inferred from Qn fields.",
            )),
        },
    }
}

fn classify_mood_editor(block: &Element, output: usize, type_tag: i32) -> LightingMoodEditor {
    let mut editor = LightingMoodEditor {
        kind: match type_tag {
            1..=3 | 7 => LightingEditorKind::Numeric,
            4 | 5 | 9 => LightingEditorKind::Color,
            _ => LightingEditorKind::Other,
        },
        type_tag: Some(type_tag),
        capability: None,
        issue: None,
    };
    if type_tag == 6 {
        let key = format!("CapAI{}", output - 1);
        let stored = block.attributes.get(&key).cloned();
        let effective = match stored.as_deref() {
            None => Some(0),
            Some(raw) => raw.parse::<u16>().ok(),
        };
        editor.capability = Some(StoredValue {
            stored,
            effective,
            default: Some(0),
        });
        editor.kind = match effective {
            Some(mask) if mask & 0x40 != 0 || mask & 7 == 7 => LightingEditorKind::Color,
            Some(_) => LightingEditorKind::Numeric,
            None => {
                editor.issue = Some(Diagnostic::new(
                    "unsupported_mood_capability",
                    &key,
                    "Tag-6 classification requires a supported nonnegative 16-bit capability mask. Malformed or out-of-range values are not truncated or defaulted.",
                ));
                LightingEditorKind::Unresolved
            }
        };
    }
    editor
}

fn inspect_mood_editor(
    block: &Element,
    mood: &Element,
    output: usize,
    raw: &str,
    direct: bool,
) -> LightingMoodEditor {
    let type_tag = raw.parse::<i32>().ok().map(|value| (value >> 28) & 15);
    let key = format!("Q{output}");
    let issue = if !direct {
        Some(Diagnostic::new(
            "unsupported_mood_container_shape",
            &key,
            "Editor classification requires a direct mood in exactly one direct LightscenesC. Nested lookalikes remain raw data.",
        ))
    } else if !(1..=18).contains(&output) {
        Some(Diagnostic::new(
            "unsupported_mood_output_index",
            &key,
            "This recovered editor path covers Q1..Q18; higher stored fields remain uninterpreted.",
        ))
    } else if type_tag.is_none() {
        Some(Diagnostic::new(
            "unsupported_mood_output_value",
            &key,
            "Editor classification requires a stored signed-32-bit decimal; opaque data is retained.",
        ))
    } else {
        match mood_output_width(mood, &key) {
            Err(issue) => Some(issue),
            Ok(width) if output > usize::try_from(width).unwrap() => Some(Diagnostic::new(
                "unsupported_mood_output_width",
                &key,
                "Stored Qn lies outside the native reader width; no loaded editor kind is inferred.",
            )),
            Ok(_) => None,
        }
    };
    match issue {
        Some(issue) => LightingMoodEditor {
            kind: LightingEditorKind::Unresolved,
            type_tag,
            capability: None,
            issue: Some(issue),
        },
        None => classify_mood_editor(block, output, type_tag.unwrap()),
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

fn inspect_duplicate_mood_ids(collection: &Element, diagnostics: &mut Vec<Diagnostic>) {
    for (key, code) in [("SID", "duplicate_mood_sid"), ("CID", "duplicate_mood_cid")] {
        let mut first_records = BTreeMap::new();
        for (index, mood) in collection
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "LightsceneC")
            .enumerate()
        {
            let Some((raw, value)) = mood.attributes.get(key).and_then(|raw| {
                raw.parse::<i32>()
                    .ok()
                    .filter(|value| {
                        if key == "SID" {
                            *value > 0
                        } else {
                            *value != 0
                        }
                    })
                    .map(|value| (raw, value))
            }) else {
                continue;
            };
            if let Some(first) = first_records.get(&value) {
                diagnostics.push(Diagnostic::new(
                    code,
                    &format!("LightscenesC/LightsceneC[{}]/@{key}", index + 1),
                    format!("Stored {key}='{raw}' is numerically equal to {key}={value} on direct mood {first}. Native lookup returns the first matching record; selection/exchange may be ambiguous. Raw values are retained, not repaired. This check covers positive signed-32-bit SIDs and nonzero signed-32-bit CIDs only; absence of duplicates is not import or editability admission."),
                ));
            } else {
                first_records.insert(value, index + 1);
            }
        }
    }
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
