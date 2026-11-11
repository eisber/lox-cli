//! Selected-mood metadata transactions, not complete native dialog acceptance.

use anyhow::Result;
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;
use super::schema::Diagnostic;

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
pub struct LightingMoodMixEdit {
    pub mood: LightingMoodIdentity,
    pub scope: &'static str,
    pub before: Option<String>,
    pub after: String,
    pub enabled: bool,
    pub changed: bool,
}

impl ConfigEditor {
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
    /// Only unique SID1..31 records with explicit nonnegative CID outside12..19
    /// and stored true/false are supported. CID12..19 has a graph/role-dependent
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
        let cid = mood_identity_integer(mood, "CID", mood_ref)?;
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
                "Mix option edits require an explicit nonnegative signed-32-bit CID. No identity was defaulted or rewritten.",
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
        let collection = self.get_element(&path[..path.len() - 1]);
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
        let before = mood.attributes.get("MixMod").cloned();
        let previous = match before.as_deref() {
            Some("true") => true,
            Some("false") => false,
            _ => {
                return Err(Diagnostic::new(
                    "unsupported_mood_mix_value",
                    &format!("{mood_ref}.MixMod"),
                    "The selected MixMod must explicitly store true or false. Missing, numeric or opaque flags are not defaulted or normalized by this transaction.",
                )
                .into());
            }
        };
        let identity = LightingMoodIdentity::from_element(mood);
        let after = enabled.to_string();
        let changed = previous != enabled;
        if changed {
            self.get_element_mut(&path)
                .attributes
                .insert("MixMod".into(), after.clone());
        }
        Ok(LightingMoodMixEdit {
            mood: identity,
            scope: "selected_record_role_independent",
            before,
            after,
            enabled,
            changed,
        })
    }

    fn lighting_metadata_mood_path(&self, selector: &str, mood_ref: &str) -> Result<Vec<usize>> {
        let mut path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                "Mood metadata edits require LightController2. Inspect the selected block before editing.",
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
                "Mood metadata edits require exactly one direct LightscenesC container. Inspect the XML before editing.",
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
                "Mix option edits require an explicit signed-32-bit decimal identity. Inspect the stored moods; missing, malformed or overflowing values are not defaulted or truncated.",
            )
            .into()
        })
}
