use anyhow::{Context, Result, bail};
use chrono::Local;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use xmltree::{Element, XMLNode};

use crate::config_edit::ConfigEditor;

const SUPPORTED_VERSION: &str = "178";

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectorSpec {
    uuid: String,
    title: String,
    version: String,
    visualized: bool,
    room: Option<String>,
    category: Option<String>,
    reset: Option<String>,
    min: Option<String>,
    max: Option<String>,
    step: Option<String>,
    value: Option<String>,
}

fn child<'a>(element: &'a Element, name: &str) -> Option<&'a Element> {
    element
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .find(|e| e.name == name)
}

fn connector_uuid(element: &Element, key: &str) -> Option<String> {
    element
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .find(|e| e.name == "Co" && e.attributes.get("K").map(String::as_str) == Some(key))
        .and_then(|e| e.attributes.get("U").cloned())
}

fn visit_elements(element: &Element, f: &mut impl FnMut(&Element)) {
    f(element);
    for child in element.children.iter().filter_map(XMLNode::as_element) {
        visit_elements(child, f);
    }
}

fn selectors(root: &Element) -> BTreeMap<String, SelectorSpec> {
    let mut out = BTreeMap::new();
    visit_elements(root, &mut |element| {
        if element.name != "C"
            || element.attributes.get("Type").map(String::as_str) != Some("PushButton2Sel")
        {
            return;
        }
        let Some(uuid) = element.attributes.get("U").cloned() else {
            return;
        };
        let io = child(element, "IoData");
        out.insert(
            uuid.clone(),
            SelectorSpec {
                uuid,
                title: element.attributes.get("Title").cloned().unwrap_or_default(),
                version: element.attributes.get("V").cloned().unwrap_or_default(),
                visualized: io
                    .and_then(|e| e.attributes.get("Visu"))
                    .map(String::as_str)
                    == Some("true"),
                room: io.and_then(|e| e.attributes.get("Pr").cloned()),
                category: io.and_then(|e| e.attributes.get("Cr").cloned()),
                reset: connector_uuid(element, "Reset"),
                min: connector_uuid(element, "Min"),
                max: connector_uuid(element, "Max"),
                step: connector_uuid(element, "Step"),
                value: connector_uuid(element, "AQ"),
            },
        );
    });
    out
}

fn right_group(root: &Element, element_type: &str, uuid: &str) -> Option<String> {
    let mut found = None;
    visit_elements(root, &mut |element| {
        if found.is_some() || element.name != "C" {
            return;
        }
        if element.attributes.get("Type").map(String::as_str) == Some(element_type)
            && element.attributes.get("U").map(String::as_str) == Some(uuid)
        {
            found = element.attributes.get("RGR").cloned();
        }
    });
    found
}

fn required<'a>(value: &'a Option<String>, what: &str, uuid: &str) -> Result<&'a str> {
    value
        .as_deref()
        .with_context(|| format!("PushButton2Sel {uuid}: missing {what}"))
}

fn jlocked_uuid(reset_uuid: &str) -> Result<String> {
    let (first, rest) = reset_uuid
        .split_once('-')
        .with_context(|| format!("invalid Reset UUID {reset_uuid}"))?;
    if first.len() != 8 {
        bail!("invalid Reset UUID {reset_uuid}");
    }
    let first = u32::from_str_radix(first, 16)
        .with_context(|| format!("invalid Reset UUID {reset_uuid}"))?
        | 0x8000_0000;
    Ok(format!("{first:08x}-{rest}"))
}

fn expected_entry(spec: &SelectorSpec) -> Result<Value> {
    if spec.version != SUPPORTED_VERSION {
        bail!(
            "PushButton2Sel {} uses V{}; ValueSelector sidecar synthesis is verified only for V{}",
            spec.uuid,
            spec.version,
            SUPPORTED_VERSION
        );
    }
    let room = required(&spec.room, "IoData.Pr room", &spec.uuid)?;
    let category = required(&spec.category, "IoData.Cr category", &spec.uuid)?;
    let reset = required(&spec.reset, "Reset connector UUID", &spec.uuid)?;
    let min = required(&spec.min, "Min connector UUID", &spec.uuid)?;
    let max = required(&spec.max, "Max connector UUID", &spec.uuid)?;
    let step = required(&spec.step, "Step connector UUID", &spec.uuid)?;
    let value = required(&spec.value, "AQ connector UUID", &spec.uuid)?;
    let jlocked = jlocked_uuid(reset)?;

    Ok(json!({
        "name": spec.title,
        "type": "ValueSelector",
        "uuidAction": spec.uuid,
        "room": room,
        "cat": category,
        "defaultRating": 0,
        "isFavorite": false,
        "isSecured": false,
        "defaultIcon": null,
        "details": {
            "jLockable": true,
            "controlHistory": true,
            "hasHistory": 0,
            "increaseOnly": false,
            "format": "%.0f"
        },
        "states": {
            "jLocked": jlocked,
            "min": min,
            "max": max,
            "step": step,
            "value": value
        },
        "restrictions": 0
    }))
}

fn uuid_bytes(uuid: &str) -> Result<[u8; 16]> {
    let parts: Vec<&str> = uuid.split('-').collect();
    if parts.len() != 4
        || parts[0].len() != 8
        || parts[1].len() != 4
        || parts[2].len() != 4
        || parts[3].len() != 16
    {
        bail!("invalid Loxone UUID {uuid}");
    }
    let a = u32::from_str_radix(parts[0], 16)?;
    let b = u16::from_str_radix(parts[1], 16)?;
    let c = u16::from_str_radix(parts[2], 16)?;
    let d = hex::decode(parts[3])?;
    if d.len() != 8 {
        bail!("invalid Loxone UUID {uuid}");
    }
    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a.to_le_bytes());
    out[4..6].copy_from_slice(&b.to_le_bytes());
    out[6..8].copy_from_slice(&c.to_le_bytes());
    out[8..16].copy_from_slice(&d);
    Ok(out)
}

fn marker(byte: u8) -> [u8; 16] {
    [byte; 16]
}

fn value_str<'a>(entry: &'a Value, pointer: &str, what: &str) -> Result<&'a str> {
    entry
        .pointer(pointer)
        .and_then(Value::as_str)
        .with_context(|| format!("ValueSelector sidecar missing {what}"))
}

fn permission_record(entry: &Value, root: &Element) -> Result<Vec<u8>> {
    if entry.get("type").and_then(Value::as_str) != Some("ValueSelector") {
        bail!("sidecar entry is not ValueSelector");
    }
    let action = value_str(entry, "/uuidAction", "uuidAction")?;
    let room = value_str(entry, "/room", "room")?;
    let category = value_str(entry, "/cat", "cat")?;
    let room_right = right_group(root, "Place", room)
        .with_context(|| format!("room {room} has no existing RightGroup"))?;
    let category_right = right_group(root, "Category", category)
        .with_context(|| format!("category {category} has no existing RightGroup"))?;

    let state_names = ["jLocked", "min", "max", "step", "value"];
    let mut out = Vec::with_capacity(176);
    out.extend_from_slice(&uuid_bytes(action)?);
    out.extend_from_slice(&marker(0xfe));
    for name in state_names {
        let state = value_str(entry, &format!("/states/{name}"), name)?;
        out.extend_from_slice(&uuid_bytes(state)?);
    }
    out.extend_from_slice(&marker(0xfc));
    out.extend_from_slice(&uuid_bytes(&room_right)?);
    out.extend_from_slice(&uuid_bytes(&category_right)?);
    out.extend_from_slice(&marker(0xff));
    debug_assert_eq!(out.len(), 176);
    Ok(out)
}

fn aligned_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len())
        .step_by(16)
        .find(|&offset| &haystack[offset..offset + needle.len()] == needle)
}

fn control_key(controls: &Map<String, Value>, uuid: &str) -> Option<String> {
    let prefixed = format!("U:{uuid}");
    if controls.contains_key(&prefixed) {
        return Some(prefixed);
    }
    if controls.contains_key(uuid) {
        return Some(uuid.to_string());
    }
    controls.iter().find_map(|(key, value)| {
        (value.get("uuidAction").and_then(Value::as_str) == Some(uuid)).then(|| key.clone())
    })
}

fn weather_insert_at(app: &Value, permissions: &[u8]) -> usize {
    let Some(weather) = app
        .pointer("/weatherServer/states/actual")
        .and_then(Value::as_str)
    else {
        return permissions.len();
    };
    let Ok(token) = uuid_bytes(weather) else {
        return permissions.len();
    };
    aligned_find(permissions, &token).unwrap_or(permissions.len())
}

/// Reconcile the independently verified V178 PushButton2Sel -> ValueSelector sidecars.
///
/// The function is deliberately narrow. It does not infer sidecars for any other
/// control type or PushButton2Sel version. Existing room/category RightGroups are
/// required; missing metadata is an error rather than a guessed binary record.
pub(crate) fn reconcile_pushbutton2sel_sidecars(
    live_xml: &[u8],
    candidate_xml: &[u8],
    app_bytes: &[u8],
    permission_bytes: &[u8],
) -> Result<(Vec<u8>, Vec<u8>, usize)> {
    let live_editor = ConfigEditor::load(live_xml)?;
    let candidate_editor = ConfigEditor::load(candidate_xml)?;
    let live_specs = selectors(&live_editor.root);
    let candidate_specs = selectors(&candidate_editor.root);

    // Keep unrelated repacks byte-semantically identical to the old behavior.
    if live_specs.is_empty() && candidate_specs.is_empty() {
        return Ok((app_bytes.to_vec(), permission_bytes.to_vec(), 0));
    }
    if !permission_bytes.len().is_multiple_of(16) {
        bail!("permissions.bin is not 16-byte aligned");
    }

    let mut app: Value =
        serde_json::from_slice(app_bytes).context("failed to parse LoxAPP3.json")?;
    let mut permissions = permission_bytes.to_vec();
    let mut changed = 0usize;

    let mut ids = BTreeSet::new();
    ids.extend(live_specs.keys().cloned());
    ids.extend(candidate_specs.keys().cloned());

    for uuid in ids {
        let live_spec = live_specs.get(&uuid);
        let candidate_spec = candidate_specs.get(&uuid);
        let expected = match candidate_spec {
            Some(spec) if spec.visualized => Some(expected_entry(spec)?),
            _ => None,
        };

        let existing_key = {
            let controls = app
                .get("controls")
                .and_then(Value::as_object)
                .context("LoxAPP3.json has no controls object")?;
            control_key(controls, &uuid)
        };
        let existing = existing_key.as_ref().and_then(|key| {
            app.get("controls")
                .and_then(Value::as_object)
                .and_then(|controls| controls.get(key))
                .cloned()
        });

        if existing == expected {
            continue;
        }

        // If neither side has an app control, this PushButton2Sel change is not
        // visualization-bearing and does not require sidecar work.
        if existing.is_none() && expected.is_none() {
            continue;
        }

        if let Some(old_entry) = existing.as_ref() {
            if old_entry.get("type").and_then(Value::as_str) != Some("ValueSelector") {
                bail!(
                    "PushButton2Sel {uuid} maps to unsupported existing app type {:?}",
                    old_entry.get("type")
                );
            }
            let old_root = live_spec
                .map(|_| &live_editor.root)
                .unwrap_or(&candidate_editor.root);
            let old_record = permission_record(old_entry, old_root)?;
            let pos = aligned_find(&permissions, &old_record).with_context(|| {
                format!(
                    "exact ValueSelector permission record for {uuid} not found; refusing heuristic rewrite"
                )
            })?;
            permissions.drain(pos..pos + old_record.len());

            if let Some(key) = existing_key.as_ref() {
                app.get_mut("controls")
                    .and_then(Value::as_object_mut)
                    .expect("controls checked above")
                    .remove(key);
            }
        }

        if let Some(new_entry) = expected {
            let new_record = permission_record(&new_entry, &candidate_editor.root)?;
            let action = uuid_bytes(&uuid)?;

            if aligned_find(&permissions, &new_record).is_none() {
                if aligned_find(&permissions, &action).is_some() {
                    bail!(
                        "permissions.bin already contains action UUID {uuid} but not the verified ValueSelector record"
                    );
                }
                let insert_at = weather_insert_at(&app, &permissions);
                permissions.splice(insert_at..insert_at, new_record);
            }

            app.get_mut("controls")
                .and_then(Value::as_object_mut)
                .expect("controls checked above")
                .insert(format!("U:{uuid}"), new_entry);
        }

        changed += 1;
    }

    if changed == 0 {
        return Ok((app_bytes.to_vec(), permission_bytes.to_vec(), 0));
    }

    app["lastModified"] = Value::String(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    let mut app_out = serde_json::to_vec(&app)?;
    app_out.push(b'\n');
    Ok((app_out, permissions, changed))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOM: &str = "11111111-1111-1111-1111111111111111";
    const ROOM_RIGHT: &str = "11111111-1111-1112-1111111111111111";
    const CAT: &str = "22222222-2222-2222-2222222222222222";
    const CAT_RIGHT: &str = "22222222-2222-2223-2222222222222222";
    const ACTION: &str = "33333333-3333-3333-3333333333333333";
    const RESET: &str = "33333333-3333-3334-3333333333333333";
    const MIN: &str = "33333333-3333-3335-3333333333333333";
    const MAX: &str = "33333333-3333-3336-3333333333333333";
    const STEP: &str = "33333333-3333-3337-3333333333333333";
    const VALUE: &str = "33333333-3333-3338-3333333333333333";

    fn xml(with_selector: bool, visualized: bool, version: &str) -> Vec<u8> {
        let selector = if with_selector {
            format!(
                r#"<C Type="PushButton2Sel" V="{version}" U="{ACTION}" Title="Selector">
<Co K="Reset" U="{RESET}"/><Co K="Min" U="{MIN}"/><Co K="Max" U="{MAX}"/>
<Co K="Step" U="{STEP}"/><Co K="AQ" U="{VALUE}"/>
<IoData Visu="{}" Pr="{ROOM}" Cr="{CAT}"/></C>"#,
                if visualized { "true" } else { "false" }
            )
        } else {
            String::new()
        };
        format!(
            r#"<?xml version="1.0"?><C Type="Document" U="aaaaaaaa-aaaa-aaaa-aaaaaaaaaaaaaaaa">
<C Type="Place" U="{ROOM}" Title="Room" RGR="{ROOM_RIGHT}"/>
<C Type="Category" U="{CAT}" Title="Category" RGR="{CAT_RIGHT}"/>
<C Type="Page" U="bbbbbbbb-bbbb-bbbb-bbbbbbbbbbbbbbbb">{selector}</C>
</C>"#
        )
        .into_bytes()
    }

    fn empty_app() -> Vec<u8> {
        br#"{"lastModified":"2026-01-01 00:00:00","controls":{}}"#.to_vec()
    }

    #[test]
    fn adds_verified_value_selector_and_176_byte_permission_record() {
        let live = xml(false, false, "178");
        let candidate = xml(true, true, "178");
        let (app, permissions, changed) =
            reconcile_pushbutton2sel_sidecars(&live, &candidate, &empty_app(), &[]).unwrap();

        assert_eq!(changed, 1);
        assert_eq!(permissions.len(), 176);

        let parsed: Value = serde_json::from_slice(&app).unwrap();
        let entry = &parsed["controls"][format!("U:{ACTION}")];
        assert_eq!(entry["type"], "ValueSelector");
        assert_eq!(entry["uuidAction"], ACTION);
        assert_eq!(entry["room"], ROOM);
        assert_eq!(entry["cat"], CAT);
        assert_eq!(entry["restrictions"], 0);
        assert_eq!(entry["details"]["increaseOnly"], false);
        assert_eq!(entry["states"]["min"], MIN);
        assert_eq!(entry["states"]["max"], MAX);
        assert_eq!(entry["states"]["step"], STEP);
        assert_eq!(entry["states"]["value"], VALUE);

        let expected =
            permission_record(entry, &ConfigEditor::load(&candidate).unwrap().root).unwrap();
        assert_eq!(permissions, expected);
    }

    #[test]
    fn repeated_reconcile_is_idempotent() {
        let live = xml(false, false, "178");
        let candidate = xml(true, true, "178");
        let (app1, permissions1, _) =
            reconcile_pushbutton2sel_sidecars(&live, &candidate, &empty_app(), &[]).unwrap();
        let (app2, permissions2, changed) =
            reconcile_pushbutton2sel_sidecars(&candidate, &candidate, &app1, &permissions1)
                .unwrap();

        assert_eq!(changed, 0);
        assert_eq!(app2, app1);
        assert_eq!(permissions2, permissions1);
    }

    #[test]
    fn removing_selector_removes_exact_verified_record() {
        let live = xml(false, false, "178");
        let with_selector = xml(true, true, "178");
        let (app, permissions, _) =
            reconcile_pushbutton2sel_sidecars(&live, &with_selector, &empty_app(), &[]).unwrap();

        let (app2, permissions2, changed) =
            reconcile_pushbutton2sel_sidecars(&with_selector, &live, &app, &permissions).unwrap();
        assert_eq!(changed, 1);
        assert!(permissions2.is_empty());
        let parsed: Value = serde_json::from_slice(&app2).unwrap();
        assert!(parsed["controls"].as_object().unwrap().is_empty());
    }

    #[test]
    fn refuses_unverified_selector_version_when_sidecar_is_needed() {
        let live = xml(false, false, "178");
        let candidate = xml(true, true, "177");
        let err =
            reconcile_pushbutton2sel_sidecars(&live, &candidate, &empty_app(), &[]).unwrap_err();
        assert!(err.to_string().contains("verified only for V178"));
    }

    #[test]
    fn nonvisualized_selector_does_not_create_sidecars() {
        let live = xml(false, false, "178");
        let candidate = xml(true, false, "178");
        let (app, permissions, changed) =
            reconcile_pushbutton2sel_sidecars(&live, &candidate, &empty_app(), &[]).unwrap();
        assert_eq!(changed, 0);
        assert_eq!(app, empty_app());
        assert!(permissions.is_empty());
    }
}
