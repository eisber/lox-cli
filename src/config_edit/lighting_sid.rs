//! Ordinary selected-mood SID edits, not role/import or complete dialog normalization.

use std::collections::BTreeSet;

use super::ConfigEditor;
use super::lighting_cid::{
    LightingIdentityChange, LightingSelectionEdit, LightingSidRemap, selection_plan,
};
use super::schema::Diagnostic;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LightingMoodSidEdit {
    pub mood: String,
    pub requested_sid: i32,
    pub swapped: bool,
    pub identities: Vec<LightingIdentityChange>,
    pub selection_remaps: Vec<LightingSidRemap>,
    pub selection_refs: Option<LightingSelectionEdit>,
    pub changed: bool,
}

impl ConfigEditor {
    pub fn set_lighting_mood_sid(
        &mut self,
        selector: &str,
        mood_ref: &str,
        sid: u32,
        swap: bool,
    ) -> Result<LightingMoodSidEdit> {
        let sid = i32::try_from(sid).ok().filter(|sid| (1..=31).contains(sid))
            .ok_or_else(|| Diagnostic::new(
                "unsupported_mood_sid_scope", "sid",
                "This ordinary SID transaction supports requested SID1..31 only. Role/display-ID conversion and the CID-clearing confirmation for higher SIDs are not simulated.",
            ))?;
        let mood_path = self.lighting_metadata_mood_path(selector, mood_ref)?;
        let collection_path = &mood_path[..mood_path.len() - 1];
        let block_path = &mood_path[..mood_path.len() - 2];
        let collection = self.get_element(collection_path);
        let records: Vec<_> = collection
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|e| e.name == "LightsceneC")
                    .map(|e| (index, e))
            })
            .collect();
        let count = collection
            .attributes
            .get("Num")
            .and_then(|raw| raw.parse::<i32>().ok())
            .and_then(|count| usize::try_from(count).ok());
        if count != Some(records.len()) {
            return Err(Diagnostic::new("unsupported_mood_count", "LightscenesC/@Num",
                "Num must match the direct scene count before a SID transaction. Inspect or explicitly recount the collection; this edit does not simulate native import.").into());
        }
        let mut seen = BTreeSet::new();
        let mut sids = Vec::new();
        for (_, record) in &records {
            let value = record.attributes.get("SID").and_then(|raw| raw.parse::<i32>().ok())
                .filter(|value| *value > 0).ok_or_else(|| Diagnostic::new(
                    "unsupported_mood_identity_value", "LightsceneC/@SID",
                    "SID transactions require explicit positive signed-32-bit SIDs on all direct records. Missing, zero, malformed and overflowing identities are not defaulted or truncated.",
                ))?;
            if !seen.insert(value) {
                return Err(Diagnostic::new("ambiguous_mood_sid", "LightscenesC",
                    "SID transactions require numerically distinct direct SIDs, including equivalent decimal spellings. No first-match ambiguity is silently accepted.").into());
            }
            sids.push(value);
        }
        let selected = records
            .iter()
            .position(|(index, _)| Some(index) == mood_path.last())
            .unwrap();
        let old_sid = sids[selected];
        if !(1..=31).contains(&old_sid) {
            return Err(Diagnostic::new("unsupported_mood_sid_scope", mood_ref,
                "The selected stored SID must be in1..31. Special/nonordinary rows are not raw identity edits.").into());
        }
        let other = sids
            .iter()
            .position(|value| *value == sid)
            .filter(|index| *index != selected);
        if other.is_some() && !swap {
            return Err(Diagnostic::new("mood_sid_occupied", "sid",
                "The requested SID is occupied. Native Config asks for exchange confirmation; pass --swap to explicitly accept exchanging both existing SIDs and their ordered selection references.").into());
        }
        if other.is_some() && seen.contains(&99) {
            return Err(Diagnostic::new("mood_sid_temporary_collision", "LightscenesC",
                "Native exchange uses temporary SID99. An existing SID99 makes that transaction unsafe; no IDs or references were written.").into());
        }
        let mut remaps = Vec::new();
        if old_sid != sid {
            if other.is_some() {
                remaps.push(LightingSidRemap {
                    before: sid,
                    after: 99,
                });
            }
            remaps.push(LightingSidRemap {
                before: old_sid,
                after: sid,
            });
            if other.is_some() {
                remaps.push(LightingSidRemap {
                    before: 99,
                    after: old_sid,
                });
            }
        }
        let selection = selection_plan(self.get_element(block_path), &remaps)?;
        let mut identities = Vec::new();
        for (position, (_, record)) in records.iter().enumerate() {
            let after = if position == selected {
                sid
            } else if Some(position) == other {
                old_sid
            } else {
                sids[position]
            };
            if after == sids[position] {
                continue;
            }
            let cid = record.attributes.get("CID").cloned();
            identities.push(LightingIdentityChange {
                record: position + 1,
                uuid: record.attributes.get("UUID").cloned(),
                before_sid: record.attributes["SID"].clone(),
                after_sid: after.to_string(),
                before_cid: cid.clone(),
                after_cid: cid,
            });
        }
        let changed =
            !identities.is_empty() || selection.as_ref().is_some_and(|(_, edit)| edit.changed);
        let collection_index = mood_path[mood_path.len() - 2];
        let changed_records: Vec<_> = identities
            .iter()
            .map(|change| (records[change.record - 1].0, change.after_sid.clone()))
            .collect();
        if changed {
            let block = self.get_element_mut(block_path);
            let collection = block.children[collection_index].as_mut_element().unwrap();
            for (index, after) in changed_records {
                collection.children[index]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("SID".into(), after);
            }
            if let Some((index, edit)) = &selection
                && edit.changed
            {
                block.children[*index]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("ScMv".into(), edit.after.clone());
            }
        }
        Ok(LightingMoodSidEdit {
            mood: mood_ref.into(),
            requested_sid: sid,
            swapped: other.is_some(),
            identities,
            selection_remaps: remaps,
            selection_refs: selection.map(|(_, edit)| edit),
            changed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<ControlList><C Type="LightController2" U="lighting"><LightscenesC Num="2" Outputs="18" Future="keep">
      <LightsceneC Name="A" UUID="a" SID="+0001" CID="opaque" SkipNext="future" Q1="future"><SeqConf X="keep"/></LightsceneC>
      <Unknown SID="99"/><LightsceneC Name="B" UUID="b" SID="2" CID="13" Q2="-1"/>
    </LightscenesC><LSConfig OpM="0,1,2" ScMv=" +01 ,2,99,opaque" ScAc="future"/><HCL T="2"/></C></ControlList>"#;

    fn assert_expected(editor: &ConfigEditor, expected: &str) {
        assert_eq!(
            editor.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }

    #[test]
    fn lighting_sid_free_edit_and_noop_preserve_unneeded_opaque_storage() {
        let mut editor = ConfigEditor::load(XML.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let edit = editor
            .set_lighting_mood_sid("uuid:lighting", "A", 1, false)
            .unwrap();
        assert!(!edit.changed && edit.selection_refs.is_none());
        assert_eq!(editor.to_bytes().unwrap(), before);
        let edit = editor
            .set_lighting_mood_sid("uuid:lighting", "A", 3, false)
            .unwrap();
        assert!(!edit.swapped);
        assert_eq!(edit.identities.len(), 1);
        assert_eq!(edit.selection_refs.unwrap().after, " 3 ,2,99,opaque");
        assert_expected(
            &editor,
            &XML.replace("SID=\"+0001\"", "SID=\"3\"")
                .replace("ScMv=\" +01 ,2,99,opaque\"", "ScMv=\" 3 ,2,99,opaque\""),
        );
    }

    #[test]
    fn lighting_sid_exchange_uses_ordered_temporary99_remaps_and_preserves_tail() {
        let mut editor = ConfigEditor::load(XML.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_sid("uuid:lighting", "A", 2, true)
            .unwrap();
        assert!(edit.swapped);
        assert_eq!(
            edit.selection_remaps
                .iter()
                .map(|r| (r.before, r.after))
                .collect::<Vec<_>>(),
            [(2, 99), (1, 2), (99, 1)]
        );
        assert_eq!(edit.selection_refs.unwrap().after, " 2 ,1,1,opaque");
        let expected = XML
            .replace(
                "Name=\"A\" UUID=\"a\" SID=\"+0001\"",
                "Name=\"A\" UUID=\"a\" SID=\"2\"",
            )
            .replace(
                "Name=\"B\" UUID=\"b\" SID=\"2\"",
                "Name=\"B\" UUID=\"b\" SID=\"1\"",
            )
            .replace("ScMv=\" +01 ,2,99,opaque\"", "ScMv=\" 2 ,1,1,opaque\"");
        assert_expected(&editor, &expected);
    }

    #[test]
    fn lighting_sid_unsupported_or_unconfirmed_transactions_are_atomic() {
        for (xml, sid, swap) in [
            (XML.into(), 2, false),
            (XML.into(), 32, true),
            (XML.replace("SID=\"+0001\"", "SID=\"777\""), 3, false),
            (XML.replace("SID=\"2\"", "SID=\"+01\""), 3, true),
            (XML.replace("SID=\"2\"", "SID=\"future\""), 3, true),
            (XML.replace("Num=\"2\"", "Num=\"3\""), 3, false),
            (XML.replace("OpM=\"0,1,2\"", "OpM=\"opaque\""), 3, false),
            (
                XML.replace("ScMv=\" +01 ,2,99,opaque\"", "ScMv=\"1,future\""),
                2,
                true,
            ),
            (
                XML.replace(
                    "<Unknown SID=\"99\"/>",
                    "<LightsceneC Name=\"C\" SID=\"99\"/>",
                )
                .replace("Num=\"2\"", "Num=\"3\""),
                2,
                true,
            ),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .set_lighting_mood_sid("uuid:lighting", "A", sid, swap)
                    .is_err()
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
}
