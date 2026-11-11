use std::collections::BTreeSet;

use anyhow::Result;
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;
use super::schema::Diagnostic;

#[derive(Debug, Serialize)]
pub struct LightingIdentityChange {
    pub record: usize,
    pub uuid: Option<String>,
    pub before_sid: String,
    pub after_sid: String,
    pub before_cid: Option<String>,
    pub after_cid: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LightingSidRemap {
    pub before: i32,
    pub after: i32,
}

#[derive(Debug, Serialize)]
pub struct LightingSelectionEdit {
    pub before: String,
    pub after: String,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct LightingMoodCidEdit {
    pub mood: String,
    pub requested_cid: i32,
    pub identities: Vec<LightingIdentityChange>,
    pub selection_remaps: Vec<LightingSidRemap>,
    pub selection_refs: Option<LightingSelectionEdit>,
    pub changed: bool,
}

#[derive(Clone)]
struct SceneIdentity {
    index: usize,
    sid: i32,
    cid: i32,
}

impl ConfigEditor {
    pub fn set_lighting_mood_cid(
        &mut self,
        selector: &str,
        mood_ref: &str,
        cid: u32,
    ) -> Result<LightingMoodCidEdit> {
        let cid = i32::try_from(cid).map_err(|_| {
            Diagnostic::new(
                "mood_cid_out_of_range",
                "cid",
                "CID must be a nonnegative signed-32-bit integer. No value was truncated or written.",
            )
        })?;
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                "CID editing requires LightController2. Inspect the selected block before editing.",
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
                "CID editing requires exactly one direct LightscenesC container.",
            )
            .into());
        };
        let records: Vec<_> = collection
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| element.name == "LightsceneC")
                    .map(|element| (index, element))
            })
            .collect();
        let count = collection
            .attributes
            .get("Num")
            .and_then(|raw| raw.parse::<i32>().ok())
            .and_then(|count| usize::try_from(count).ok());
        if count != Some(records.len()) {
            return Err(Diagnostic::new(
                "unsupported_mood_count",
                "LightscenesC/@Num",
                "Num must match the direct scene count before a CID transaction. Inspect or explicitly recount the collection; this edit does not simulate native import.",
            )
            .into());
        }
        let selected: Vec<_> = records
            .iter()
            .enumerate()
            .filter(|(_, (_, scene))| {
                ["SID", "Name"]
                    .iter()
                    .any(|key| scene.attributes.get(*key).map(String::as_str) == Some(mood_ref))
            })
            .map(|(index, _)| index)
            .collect();
        let [selected] = selected.as_slice() else {
            return Err(Diagnostic::new(
                "ambiguous_or_missing_mood",
                mood_ref,
                "Choose exactly one direct mood by its exact stored SID spelling or Name.",
            )
            .into());
        };
        let selected = *selected;
        let mut scenes = Vec::new();
        for (index, scene) in &records {
            let parse = |key: &str| -> Result<i32> {
                if key == "CID" && !scene.attributes.contains_key(key) {
                    return Ok(0);
                }
                let value = scene
                    .attributes
                    .get(key)
                    .and_then(|raw| raw.parse::<i32>().ok());
                match value {
                    Some(value) if value >= 0 && (key != "SID" || value > 0) => Ok(value),
                    _ => Err(Diagnostic::new(
                        "unsupported_mood_identity_value",
                        &format!("LightsceneC/@{key}"),
                        "CID transactions require explicit positive signed-32-bit SIDs and nonnegative signed-32-bit CIDs. Absent CID has the recovered reader default0; present malformed values and missing/zero SIDs are not guessed.",
                    )
                    .into()),
                }
            };
            scenes.push(SceneIdentity {
                index: *index,
                sid: parse("SID")?,
                cid: parse("CID")?,
            });
        }
        validate_unique_identities(&scenes)?;
        let original = scenes.clone();
        let other = scenes.iter().position(|scene| scene.cid == cid);
        if let Some(other) = other.filter(|other| *other != selected && cid != 0) {
            scenes[other].cid = scenes[selected].cid;
            scenes[selected].cid = cid;
            if scenes[other].sid >= 776 {
                // This early native SID write intentionally has no ScMv remap.
                scenes[selected].sid = free_sid(&scenes)?;
            }
        } else {
            scenes[selected].cid = cid;
        }
        let mut remaps = Vec::new();
        normalize_role(&mut scenes, selected, &mut remaps)?;
        if let Some(other) = other {
            normalize_role(&mut scenes, other, &mut remaps)?;
        }
        validate_unique_identities(&scenes)?;
        let selection = selection_plan(block, &remaps)?;
        let mut identities = Vec::new();
        for (position, (before, after)) in original.iter().zip(&scenes).enumerate() {
            if before.sid == after.sid && before.cid == after.cid {
                continue;
            }
            let scene = records[position].1;
            let before_sid = scene.attributes["SID"].clone();
            let before_cid = scene.attributes.get("CID").cloned();
            identities.push(LightingIdentityChange {
                record: position + 1,
                uuid: scene.attributes.get("UUID").cloned(),
                after_sid: if before.sid == after.sid {
                    before_sid.clone()
                } else {
                    after.sid.to_string()
                },
                after_cid: if before.cid == after.cid {
                    before_cid.clone()
                } else {
                    Some(after.cid.to_string())
                },
                before_sid,
                before_cid,
            });
        }
        let changed =
            !identities.is_empty() || selection.as_ref().is_some_and(|(_, edit)| edit.changed);
        let collection_index = *collection_index;
        if changed {
            let block = self.get_element_mut(&path);
            let collection = block.children[collection_index].as_mut_element().unwrap();
            for change in &identities {
                let scene = collection.children[scenes[change.record - 1].index]
                    .as_mut_element()
                    .unwrap();
                scene
                    .attributes
                    .insert("SID".into(), change.after_sid.clone());
                if let Some(cid) = &change.after_cid {
                    scene.attributes.insert("CID".into(), cid.clone());
                }
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
        Ok(LightingMoodCidEdit {
            mood: mood_ref.into(),
            requested_cid: cid,
            identities,
            selection_remaps: remaps,
            selection_refs: selection.map(|(_, edit)| edit),
            changed,
        })
    }
}

fn validate_unique_identities(scenes: &[SceneIdentity]) -> Result<()> {
    let mut sids = BTreeSet::new();
    let mut cids = BTreeSet::new();
    for scene in scenes {
        if scene.sid <= 0 || !sids.insert(scene.sid) {
            return Err(Diagnostic::new(
                "ambiguous_mood_sid",
                "LightscenesC",
                "CID editing requires distinct positive SIDs before and after the transaction. Existing role inconsistencies are not silently repaired.",
            )
            .into());
        }
        if scene.cid != 0 && !cids.insert(scene.cid) {
            return Err(Diagnostic::new(
                "ambiguous_mood_cid",
                "LightscenesC",
                "Nonzero CIDs must be unique for a deterministic swap. Repeated CID0 is allowed and uses the first stored match, as in the recovered handler.",
            )
            .into());
        }
    }
    Ok(())
}

fn free_sid(scenes: &[SceneIdentity]) -> Result<i32> {
    (1..=99)
        .find(|sid| !scenes.iter().any(|scene| scene.sid == *sid))
        .ok_or_else(|| {
            Diagnostic::new(
                "mood_identity_exhausted",
                "LightscenesC",
                "The CID transaction requires a free SID in 1..99, but none remains. No identities or references were written; native zero-SID exhaustion is not reproduced.",
            )
            .into()
        })
}

fn normalize_role(
    scenes: &mut [SceneIdentity],
    index: usize,
    remaps: &mut Vec<LightingSidRemap>,
) -> Result<()> {
    let before = scenes[index].sid;
    let after = match scenes[index].cid {
        22 => 777,
        24 => 776,
        _ if before >= 776 => free_sid(scenes)?,
        _ => before,
    };
    scenes[index].sid = after;
    if before != after {
        remaps.push(LightingSidRemap { before, after });
    }
    Ok(())
}

pub(super) fn selection_plan(
    block: &Element,
    remaps: &[LightingSidRemap],
) -> Result<Option<(usize, LightingSelectionEdit)>> {
    if remaps.is_empty() {
        return Ok(None);
    }
    let configs: Vec<_> = block
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.as_element()
                .filter(|element| element.name == "LSConfig")
                .map(|element| (index, element))
        })
        .collect();
    if configs.is_empty() {
        return Ok(None);
    }
    let [(index, config)] = configs.as_slice() else {
        return Err(Diagnostic::new(
            "unsupported_lighting_selection_shape",
            "LSConfig",
            "SID remapping requires at most one direct LSConfig. Ambiguous selection records are not rewritten.",
        )
        .into());
    };
    let Some(operations) = config.attributes.get("OpM").filter(|raw| !raw.is_empty()) else {
        return Ok(None);
    };
    let Some(before) = config.attributes.get("ScMv").filter(|raw| !raw.is_empty()) else {
        return Ok(None);
    };
    let parse_list = |raw: &str, key: &str, limit: usize| -> Result<Vec<i32>> {
        raw.split(',')
            .take(limit)
            .map(|token| {
                token.trim_matches(|c: char| c.is_ascii_whitespace()).parse::<i32>().map_err(|_| {
                    Diagnostic::new(
                        "unsupported_lighting_selection_value",
                        &format!("LSConfig/@{key}"),
                        "SID remapping requires comma-separated signed-32-bit decimals without empty tokens. Opaque lists are not truncated or normalized.",
                    )
                    .into()
                })
            })
            .collect()
    };
    let count = parse_list(operations, "OpM", usize::MAX)?.len();
    let mut values = parse_list(before, "ScMv", count)?;
    let mut tokens: Vec<_> = before.split(',').map(String::from).collect();
    for remap in remaps {
        for (index, value) in values.iter_mut().take(count).enumerate() {
            if *value == remap.before {
                *value = remap.after;
                let token = &tokens[index];
                let prefix = &token[..token.len()
                    - token
                        .trim_start_matches(|c: char| c.is_ascii_whitespace())
                        .len()];
                let suffix = &token[token
                    .trim_end_matches(|c: char| c.is_ascii_whitespace())
                    .len()..];
                tokens[index] = format!("{prefix}{}{suffix}", remap.after);
            }
        }
    }
    let after = tokens.join(",");
    Ok(Some((
        *index,
        LightingSelectionEdit {
            before: before.clone(),
            changed: after != *before,
            after,
        },
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(records: &str, selection: &str) -> String {
        format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\" Title=\"Lamp\"><LightscenesC Num=\"{}\" Outputs=\"18\" Future=\"keep\">{records}</LightscenesC>{selection}<HCL T=\"2\" FROM=\"0\"/></C></ControlList>",
            records.matches("<LightsceneC ").count()
        )
    }

    const ORDINARY: &str = r#"<LightsceneC Name="A" UUID="a" SID="1" CID="12" Q1="future"><SeqConf Future="keep"/></LightsceneC><LightsceneC Name="B" UUID="b" SID="2" CID="13" Q2="-1"/>"#;

    #[test]
    fn lighting_cid_swaps_only_existing_ids_and_preserves_opaque_unneeded_selection() {
        let xml = fixture(
            ORDINARY,
            r#"<LSConfig OpM="opaque" ScMv="future" ScAc="unknown"/>"#,
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_cid("uuid:lighting", "A", 13)
            .unwrap();
        assert_eq!(edit.identities.len(), 2);
        assert!(edit.selection_remaps.is_empty());
        assert!(edit.selection_refs.is_none());
        let expected = xml
            .replace("SID=\"1\" CID=\"12\"", "SID=\"1\" CID=\"13\"")
            .replace("SID=\"2\" CID=\"13\"", "SID=\"2\" CID=\"12\"");
        assert_eq!(
            editor.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }

    #[test]
    fn lighting_cid_role_swap_retains_native_early_write_and_ordered_reference_cascade() {
        let records = r#"<LightsceneC Name="A" SID="2" CID="13"/><LightsceneC Name="Role" SID="777" CID="22"/><LightsceneC Name="B" SID="1" CID="12"/>"#;
        let xml = fixture(
            records,
            r#"<LSConfig OpM="0,1,2" ScMv="2, 777 ,+03,opaque" ScAc="future"/>"#,
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_cid("uuid:lighting", "A", 22)
            .unwrap();
        assert_eq!(
            edit.selection_remaps
                .iter()
                .map(|m| (m.before, m.after))
                .collect::<Vec<_>>(),
            [(3, 777), (777, 2)]
        );
        assert_eq!(
            edit.selection_refs.as_ref().unwrap().after,
            "2, 2 ,2,opaque"
        );
        let expected = xml
            .replace(
                "Name=\"A\" SID=\"2\" CID=\"13\"",
                "Name=\"A\" SID=\"777\" CID=\"22\"",
            )
            .replace(
                "Name=\"Role\" SID=\"777\" CID=\"22\"",
                "Name=\"Role\" SID=\"2\" CID=\"13\"",
            )
            .replace("ScMv=\"2, 777 ,+03,opaque\"", "ScMv=\"2, 2 ,2,opaque\"");
        assert_eq!(
            editor.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }

    #[test]
    fn lighting_cid_zero_does_not_swap_but_can_demote_first_zero_cid_role() {
        let xml = fixture(
            r#"<LightsceneC Name="A" SID="1" CID="12"/><LightsceneC Name="Role" SID="778" CID="0"/><LightsceneC Name="B" SID="2" CID="0"/>"#,
            r#"<LSConfig OpM="0" ScMv="778,opaque"/>"#,
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_cid("uuid:lighting", "A", 0)
            .unwrap();
        assert_eq!(edit.identities.len(), 2);
        assert_eq!(edit.identities[0].after_sid, "1");
        assert_eq!(edit.identities[0].after_cid.as_deref(), Some("0"));
        assert_eq!(edit.identities[1].after_sid, "3");
        assert_eq!(edit.selection_refs.unwrap().after, "3,opaque");
        let xml = xml.replace("SID=\"778\" CID=\"0\"", "SID=\"778\"");
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_cid("uuid:lighting", "A", 0)
            .unwrap();
        assert_eq!(edit.identities[1].before_cid, None);
        assert_eq!(edit.identities[1].after_cid, None);
        assert_eq!(edit.identities[1].after_sid, "3");
    }

    #[test]
    fn lighting_cid_leaving_a_special_role_remaps_both_affected_scene_references() {
        for (role_sid, role_cid) in [(777, 22), (776, 24)] {
            let xml = fixture(
                &format!(
                    "<LightsceneC Name=\"A\" SID=\"{role_sid}\" CID=\"{role_cid}\" Q1=\"future\"/><LightsceneC Name=\"B\" SID=\"1\" CID=\"12\"/>"
                ),
                &format!("<LSConfig OpM=\"0,1,2\" ScMv=\"{role_sid},1,2\"/>"),
            );
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let edit = editor
                .set_lighting_mood_cid("uuid:lighting", "A", 12)
                .unwrap();
            assert_eq!(edit.identities[0].after_sid, "2");
            assert_eq!(edit.identities[1].after_sid, role_sid.to_string());
            assert_eq!(
                edit.selection_refs.unwrap().after,
                format!("2,{role_sid},2")
            );
        }
    }

    #[test]
    fn lighting_cid_promotes_roles_and_preserves_noop_spelling_without_selection_defaults() {
        for (cid, sid) in [(22, "777"), (24, "776")] {
            let xml = fixture(ORDINARY, r#"<LSConfig ScMv="opaque"/>"#);
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let edit = editor
                .set_lighting_mood_cid("uuid:lighting", "A", cid)
                .unwrap();
            assert_eq!(edit.identities[0].after_sid, sid);
            assert_eq!(edit.identities[0].after_cid, Some(cid.to_string()));
            assert!(edit.selection_refs.is_none());
        }
        let xml = fixture(&ORDINARY.replace("CID=\"12\"", "CID=\"+00012\""), "");
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let edit = editor
            .set_lighting_mood_cid("uuid:lighting", "A", 12)
            .unwrap();
        assert!(!edit.changed);
        assert!(edit.identities.is_empty());
        assert_eq!(editor.to_bytes().unwrap(), before);
        let xml = fixture(
            r#"<LightsceneC Name="A" SID="1"/><LightsceneC SID="778"/>"#,
            "",
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_cid("uuid:lighting", "A", 0)
            .unwrap();
        assert_eq!(edit.identities.len(), 0);
        assert!(!edit.changed);
    }

    #[test]
    fn lighting_cid_rejects_ambiguous_roles_invalid_refs_and_exhaustion_atomically() {
        let mut cases = vec![
            (fixture(&ORDINARY.replace("SID=\"2\"", "SID=\"1\""), ""), 13),
            (
                fixture(&ORDINARY.replace("CID=\"13\"", "CID=\"12\""), ""),
                13,
            ),
            (fixture(&ORDINARY.replace("SID=\"2\"", "SID=\"0\""), ""), 13),
            (
                fixture(&ORDINARY.replace("CID=\"13\"", "CID=\"opaque\""), ""),
                13,
            ),
            (fixture(ORDINARY, "").replace("Num=\"2\"", "Num=\"99\""), 13),
            (fixture(ORDINARY, ""), u32::MAX),
            (
                fixture(&ORDINARY.replace("Name=\"B\"", "Name=\"A\""), ""),
                13,
            ),
            (
                fixture(ORDINARY, r#"<LSConfig OpM="0" ScMv="opaque"/>"#),
                22,
            ),
            (fixture(ORDINARY, "<LSConfig/><LSConfig/>"), 22),
            (
                fixture(
                    r#"<LightsceneC Name="A" SID="1" CID="12"/><LightsceneC SID="777" CID="11"/>"#,
                    "",
                ),
                22,
            ),
        ];
        let mut records: String = (1..=99)
            .map(|sid| format!("<LightsceneC SID=\"{sid}\" CID=\"0\"/>"))
            .collect();
        records.push_str("<LightsceneC Name=\"A\" SID=\"778\" CID=\"0\"/>");
        cases.push((fixture(&records, ""), 10));
        for (xml, cid) in cases {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .set_lighting_mood_cid("uuid:lighting", "A", cid)
                    .is_err(),
                "{xml}/{cid}"
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
}
