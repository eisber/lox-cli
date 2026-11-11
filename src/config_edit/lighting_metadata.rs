//! Selected-mood metadata transactions, not complete native dialog acceptance.

use anyhow::Result;
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;
use super::schema::{Diagnostic, StoredValue};

#[derive(Debug, Serialize)]
pub struct LightingMoodIdentity {
    pub uuid: Option<String>,
    pub sid: Option<String>,
    pub cid: Option<String>,
}

impl LightingMoodIdentity {
    fn from_element(mood: &Element) -> Self {
        Self {
            uuid: mood.attributes.get("UUID").cloned(),
            sid: mood.attributes.get("SID").cloned(),
            cid: mood.attributes.get("CID").cloned(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LightingMoodRenameEdit {
    pub mood: LightingMoodIdentity,
    pub before: Option<String>,
    pub after: String,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodDisplayNameEdit {
    pub mood: LightingMoodIdentity,
    pub before: Option<String>,
    pub after: String,
    pub changed: bool,
    pub normalized_from: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodMixEdit {
    pub mood: LightingMoodIdentity,
    pub scope: &'static str,
    pub cid_effective: i32,
    pub before: Option<String>,
    pub before_effective: bool,
    pub default: bool,
    pub after: Option<String>,
    pub enabled: bool,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodSkipEdit {
    pub mood: LightingMoodIdentity,
    pub before: StoredValue<bool>,
    pub after: Option<String>,
    pub skip: bool,
    pub checkbox_checked: bool,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodFlag {
    pub key: &'static str,
    pub value: StoredValue<bool>,
    pub checkbox_checked: Option<bool>,
    pub projection: &'static str,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodOptions {
    pub mood: LightingMoodIdentity,
    pub read_only: bool,
    pub flags: Vec<LightingMoodFlag>,
    pub display_name: StoredValue<String>,
    pub icon_id: StoredValue<i32>,
    pub icon_resolution: &'static str,
    pub diagnostics: Vec<Diagnostic>,
}

impl ConfigEditor {
    pub fn inspect_lighting_mood_options(
        &self,
        selector: &str,
        mood_ref: &str,
    ) -> Result<LightingMoodOptions> {
        let path = self.lighting_metadata_mood_path(selector, mood_ref)?;
        let mood = self.get_element(&path);
        let mut diagnostics = vec![Diagnostic::new(
            "stored_mood_options_only",
            mood_ref,
            "Stored values and recovered checkbox projections only, not loaded-row/import, current graph/role state or GUI permissions. IID is a numeric icon reference, not an input binding; catalog/image resolution is not established. Each edit operation independently validates its target.",
        )];
        let mut flags = Vec::new();
        for key in ["SkipNext", "MixMod", "ReadOnly", "Sequencing", "Move"] {
            let value = Self::stored_mood_flag(mood, key);
            if value.effective.is_none() {
                diagnostics.push(Diagnostic::new(
                    "unsupported_mood_option_value", &format!("{mood_ref}.{key}"),
                    "Only canonical true/false or omission is interpreted here. Unsupported explicit storage is retained and unresolved, not defaulted.",
                ));
            }
            let (mut projection, checkbox_checked) = match key {
                "SkipNext" => ("inverse", value.effective.map(|skip| !skip)),
                "MixMod" => {
                    let sid = mood
                        .attributes
                        .get("SID")
                        .and_then(|raw| raw.parse::<i32>().ok());
                    let cid = Self::stored_mood_integer(mood, "CID")
                        .effective
                        .filter(|cid| *cid >= 0);
                    match (sid, cid) {
                        (Some(sid), _) if sid > 31 => ("not_checkbox_row", None),
                        (Some(sid), Some(cid)) if sid > 0 => {
                            if (12..=19).contains(&cid) {
                                diagnostics.push(Diagnostic::new(
                                    "role_dependent_mood_mix", &format!("{mood_ref}.MixMod"),
                                    "CID12..19 can force and disable the displayed mix checkbox through current graph/role state. Stored MixMod is not its displayed state.",
                                ));
                                ("role_dependent", None)
                            } else {
                                ("direct", value.effective)
                            }
                        }
                        _ => {
                            diagnostics.push(Diagnostic::new(
                                "unresolved_mood_option_identity", &format!("{mood_ref}.MixMod"),
                                "Mix projection needs a supported positive signed32 SID and nonnegative signed32 CID (omitted CID has reader default0). No identity was defaulted from unsupported explicit storage.",
                            ));
                            ("unresolved_identity", None)
                        }
                    }
                }
                _ => ("stored_only", None),
            };
            if value.effective.is_none() && matches!(projection, "inverse" | "direct") {
                projection = "unresolved_value";
            }
            flags.push(LightingMoodFlag {
                key,
                value,
                checkbox_checked,
                projection,
            });
        }
        let stored = mood.attributes.get("SName").cloned();
        let icon_id = Self::stored_mood_integer(mood, "IID");
        if icon_id.effective.is_none() {
            diagnostics.push(Diagnostic::new(
                "unsupported_mood_icon_id", &format!("{mood_ref}.IID"),
                "Stored IID is not a supported signed32 decimal. It is retained and unresolved, not defaulted or treated as an input binding.",
            ));
        }
        Ok(LightingMoodOptions {
            mood: LightingMoodIdentity::from_element(mood),
            read_only: true,
            flags,
            display_name: StoredValue {
                effective: Some(stored.clone().unwrap_or_default()),
                stored,
                default: Some(String::new()),
            },
            icon_id,
            icon_resolution: "unresolved_catalog_mapping",
            diagnostics,
        })
    }

    /// Set SName using a no-normalization uppercase ASCII subset; empty clears it.
    pub fn set_lighting_mood_display_name(
        &mut self,
        selector: &str,
        mood_ref: &str,
        name: &str,
    ) -> Result<LightingMoodDisplayNameEdit> {
        let alphanumeric = |byte: &u8| byte.is_ascii_uppercase() || byte.is_ascii_digit();
        if !name.is_empty()
            && (!name.as_bytes().first().is_some_and(alphanumeric)
                || !name.as_bytes().last().is_some_and(alphanumeric)
                || !name.bytes().all(|byte| alphanumeric(&byte) || byte == b' '))
        {
            return Err(Diagnostic::new(
                "unsupported_mood_display_name",
                "name",
                "Use uppercase ASCII letters/digits with ordinary internal spaces and alphanumeric edges, or explicitly clear. --normalize-ascii opts into recovered lowercase ASCII conversion only; transliteration, trimming and punctuation replacement are not approximated.",
            )
            .into());
        }
        let path = self.lighting_metadata_mood_path(selector, mood_ref)?;
        let mood = self.get_element(&path);
        let identity = LightingMoodIdentity::from_element(mood);
        let before = mood.attributes.get("SName").cloned();
        let changed = before.as_deref().unwrap_or("") != name;
        if changed {
            self.get_element_mut(&path)
                .attributes
                .insert("SName".into(), name.into());
        }
        Ok(LightingMoodDisplayNameEdit {
            mood: identity,
            before,
            after: name.into(),
            changed,
            normalized_from: None,
        })
    }

    pub fn set_lighting_mood_display_name_ascii(
        &mut self,
        selector: &str,
        mood_ref: &str,
        name: &str,
    ) -> Result<LightingMoodDisplayNameEdit> {
        let normalized = name.to_ascii_uppercase();
        let mut edit = self.set_lighting_mood_display_name(selector, mood_ref, &normalized)?;
        edit.normalized_from = (normalized != name).then(|| name.to_string());
        Ok(edit)
    }

    /// Set only SkipNext; the native cycling checkbox displays its inverse.
    pub fn set_lighting_mood_skip(
        &mut self,
        selector: &str,
        mood_ref: &str,
        skip: bool,
    ) -> Result<LightingMoodSkipEdit> {
        let path = self.lighting_metadata_mood_path(selector, mood_ref)?;
        let mood = self.get_element(&path);
        let sid = mood_identity_integer(mood, "SID", mood_ref)?;
        if sid <= 0 {
            return Err(Diagnostic::new(
                "unsupported_mood_skip_scope",
                mood_ref,
                "Cycling flag edits require an explicit positive signed-32-bit SID. Native import's zero-SID reconciliation is not simulated.",
            )
            .into());
        }
        require_unique_metadata_sid(
            self.get_element(&path[..path.len() - 1]),
            sid,
            selector,
            mood_ref,
        )?;
        let before = Self::stored_mood_flag(mood, "SkipNext");
        let previous = before.effective.ok_or_else(|| {
                Diagnostic::new(
                    "unsupported_mood_skip_value",
                    &format!("{mood_ref}.SkipNext"),
                    "SkipNext must be canonical true/false or absent (recovered native default false). Unsupported explicit values are not defaulted or normalized.",
                )
        })?;
        let identity = LightingMoodIdentity::from_element(mood);
        let changed = previous != skip;
        let after = if changed {
            Some(skip.to_string())
        } else {
            before.stored.clone()
        };
        if let Some(after) = &after
            && changed
        {
            self.get_element_mut(&path)
                .attributes
                .insert("SkipNext".into(), after.clone());
        }
        Ok(LightingMoodSkipEdit {
            mood: identity,
            before,
            after,
            skip,
            checkbox_checked: !skip,
            changed,
        })
    }

    /// Rename one direct mood using a no-normalization subset of the native name editor.
    ///
    /// New names contain only ASCII letters, digits and internal ordinary spaces,
    /// and start/end with a letter or digit. Other native name normalization and
    /// character conversion are not reproduced. Duplicate names are permitted,
    /// as in the recovered row-bound setter; ambiguous selectors are not.
    pub fn rename_lighting_mood(
        &mut self,
        selector: &str,
        mood_ref: &str,
        name: &str,
    ) -> Result<LightingMoodRenameEdit> {
        if !name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
            || !name
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b' ')
        {
            return Err(Diagnostic::new(
                "unsupported_mood_name",
                "name",
                "Use a nonempty ASCII letter/digit name with internal ordinary spaces only and a letter/digit at each end. Native normalization and character conversion outside this subset are not implemented; no name was rewritten.",
            )
            .into());
        }
        let path = self.lighting_metadata_mood_path(selector, mood_ref)?;
        let mood = self.get_element(&path);
        let identity = LightingMoodIdentity::from_element(mood);
        let before = mood.attributes.get("Name").cloned();
        let changed = before.as_deref() != Some(name);
        if changed {
            self.get_element_mut(&path)
                .attributes
                .insert("Name".into(), name.into());
        }
        Ok(LightingMoodRenameEdit {
            mood: identity,
            before,
            after: name.into(),
            changed,
        })
    }

    /// Set the requested state of the recovered per-mood MixMod checkbox.
    ///
    /// Only unique SID1..31 records with nonnegative CID outside12..19 (omitted=0)
    /// and true/false or omitted (default false) are supported. CID12..19 has a graph/role-dependent
    /// forced checkbox state; special/non-checkbox rows are not raw flag edits.
    pub fn set_lighting_mood_mix(
        &mut self,
        selector: &str,
        mood_ref: &str,
        enabled: bool,
    ) -> Result<LightingMoodMixEdit> {
        let path = self.lighting_metadata_mood_path(selector, mood_ref)?;
        let mood = self.get_element(&path);
        let sid = mood_identity_integer(mood, "SID", mood_ref)?;
        let cid = Self::stored_mood_integer(mood, "CID").effective.ok_or_else(|| {
            Diagnostic::new(
                "unsupported_mood_identity", &format!("{mood_ref}.CID"),
                "Mix option edits require a supported nonnegative signed32 CID or omission (recovered default0). Unsupported explicit data is not defaulted or truncated.",
            )
        })?;
        if !(1..=31).contains(&sid) {
            return Err(Diagnostic::new(
                "unsupported_mood_mix_scope",
                mood_ref,
                "Mix option edits support only SID1..31 checkbox rows. Select a supported ordinary mood; special, zero or non-checkbox rows are not raw boolean edits.",
            )
            .into());
        }
        if cid < 0 {
            return Err(Diagnostic::new(
                "unsupported_mood_identity",
                &format!("{mood_ref}.CID"),
                "Mix option edits require nonnegative signed32 CID or omission (recovered default0). Negative identity was not rewritten.",
            )
            .into());
        }
        if (12..=19).contains(&cid) {
            return Err(Diagnostic::new(
                "role_dependent_mood_mix",
                mood_ref,
                "CID12..19 can force and disable the native mix checkbox through owner/role state. This transaction does not infer that state or alter its cohort; edit this option in native Config.",
            )
            .into());
        }
        require_unique_metadata_sid(
            self.get_element(&path[..path.len() - 1]),
            sid,
            selector,
            mood_ref,
        )?;
        let before = Self::stored_mood_flag(mood, "MixMod");
        let previous = before.effective.ok_or_else(|| {
                Diagnostic::new(
                    "unsupported_mood_mix_value",
                    &format!("{mood_ref}.MixMod"),
                    "MixMod must be canonical true/false or absent (recovered native default false). Unsupported explicit values are not defaulted or normalized.",
                )
        })?;
        let identity = LightingMoodIdentity::from_element(mood);
        let changed = previous != enabled;
        let after = if changed {
            Some(enabled.to_string())
        } else {
            before.stored.clone()
        };
        if let Some(after) = &after
            && changed
        {
            self.get_element_mut(&path)
                .attributes
                .insert("MixMod".into(), after.clone());
        }
        Ok(LightingMoodMixEdit {
            mood: identity,
            scope: "selected_record_role_independent",
            cid_effective: cid,
            before: before.stored,
            before_effective: previous,
            default: false,
            after,
            enabled,
            changed,
        })
    }

    fn stored_mood_flag(mood: &Element, key: &str) -> StoredValue<bool> {
        let stored = mood.attributes.get(key).cloned();
        let effective = match stored.as_deref() {
            None | Some("false") => Some(false),
            Some("true") => Some(true),
            _ => None,
        };
        StoredValue {
            stored,
            effective,
            default: Some(false),
        }
    }

    fn stored_mood_integer(mood: &Element, key: &str) -> StoredValue<i32> {
        let stored = mood.attributes.get(key).cloned();
        let effective = match stored.as_deref() {
            None => Some(0),
            Some(raw) => raw.parse::<i32>().ok(),
        };
        StoredValue {
            stored,
            effective,
            default: Some(0),
        }
    }

    pub(super) fn lighting_metadata_mood_path(
        &self,
        selector: &str,
        mood_ref: &str,
    ) -> Result<Vec<usize>> {
        let mut path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                "Mood metadata operations require LightController2. Inspect the selected block.",
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
                "Mood metadata operations require exactly one direct LightscenesC container. Inspect the XML before proceeding.",
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
                    .map(|_| index)
            })
            .collect();
        let [mood_index] = matches.as_slice() else {
            return Err(Diagnostic::new(
                "ambiguous_or_missing_mood",
                mood_ref,
                format!(
                    "Expected one direct mood matching exact stored SID or Name, found {}. Choose an unambiguous stored mood; no fuzzy matching or identity default was applied.",
                    matches.len()
                ),
            )
            .into());
        };
        path.extend([*collection_index, *mood_index]);
        Ok(path)
    }
}

fn mood_identity_integer(mood: &Element, key: &str, path: &str) -> Result<i32> {
    mood.attributes
        .get(key)
        .and_then(|raw| raw.parse::<i32>().ok())
        .ok_or_else(|| {
            Diagnostic::new(
                "unsupported_mood_identity",
                &format!("{path}.{key}"),
                "Checkbox option edits require an explicit signed-32-bit decimal identity. Inspect the stored moods; missing, malformed or overflowing values are not defaulted or truncated.",
            )
            .into()
        })
}

fn require_unique_metadata_sid(
    collection: &Element,
    sid: i32,
    selector: &str,
    mood_ref: &str,
) -> Result<()> {
    let mut matching_sids = 0;
    for (index, node) in collection.children.iter().enumerate() {
        let Some(sibling) = node.as_element().filter(|e| e.name == "LightsceneC") else {
            continue;
        };
        let sibling_sid = mood_identity_integer(
            sibling,
            "SID",
            &format!("{selector}.LightscenesC.LightsceneC[{index}]"),
        )?;
        if sibling_sid == sid {
            matching_sids += 1;
        }
    }
    if matching_sids != 1 {
        return Err(Diagnostic::new(
            "ambiguous_mood_sid",
            mood_ref,
            "The native checkbox refresh reselects by numeric SID. The selected SID must be unique among direct moods, including equivalent decimal spellings; no cohort was changed.",
        )
        .into());
    }
    Ok(())
}
