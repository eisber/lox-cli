use super::ConfigEditor;
use anyhow::Result;

// Frozen public-7a helper body: compare the migrated cached editor with its
// original mutation, including the second call's legacy U-clearing behavior.
fn frozen_unwire(
    editor: &mut ConfigEditor,
    selector: &str,
    connector_name: &str,
) -> Result<String> {
    let path = editor.require_one(selector)?;
    let elem = editor.get_element_mut(&path);
    let title = elem.attributes.get("Title").cloned().unwrap_or_default();
    let co = elem
        .children
        .iter_mut()
        .find_map(|c| {
            c.as_mut_element().filter(|e| {
                e.name == "Co"
                    && e.attributes
                        .get("K")
                        .map(|k| k == connector_name)
                        .unwrap_or(false)
            })
        })
        .ok_or_else(|| {
            anyhow::anyhow!("Connector '{}' not found on '{}'", connector_name, title)
        })?;
    let old = co.attributes.get("U").cloned().unwrap_or_default();
    let had_in_children = co
        .children
        .iter()
        .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
    co.children
        .retain(|c| !c.as_element().map(|e| e.name == "In").unwrap_or(false));
    co.attributes.shift_remove("Nc");
    if !had_in_children {
        co.attributes.shift_remove("U");
    }
    Ok(format!(
        "Unwired {}.{} (was {})",
        title, connector_name, old
    ))
}

#[test]
fn unwire_cached_editor_matches_frozen_helper_and_public_bytes_after_prior_edit() {
    let input = br#"<C Type="Document" DateS="561020133"><C U="a" Title="Gate.dot" WF="2"><Co K="I1" U="raw" Nc="2" X="keep"><In Input="same" FLG="2"/><In Input="same"/><Unknown><In Input="nested"/></Unknown></Co><Co K="Q" U="other"/></C><C U="a" Title="mirror"><Co K="I1" U="mirror"/></C></C>"#;
    let mut frozen = ConfigEditor::load(input).unwrap();
    let mut migrated = ConfigEditor::load(input).unwrap();
    frozen
        .set_attribute("uuid:a", "Title", "Changed.dot")
        .unwrap();
    migrated
        .set_attribute("uuid:a", "Title", "Changed.dot")
        .unwrap();
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap();
    for _ in 0..3 {
        let before = migrated.to_bytes().unwrap();
        let public =
            lox_core::configuration::editor::unwire(&before, "uuid:a", "I1", &now).unwrap();
        assert_eq!(
            frozen_unwire(&mut frozen, "uuid:a", "I1").unwrap(),
            migrated.unwire("uuid:a", "I1").unwrap()
        );
        assert_eq!(frozen.to_bytes().unwrap(), migrated.to_bytes().unwrap());
        let mut stamped = ConfigEditor::load(&migrated.to_bytes().unwrap()).unwrap();
        if public.content_changed {
            stamped.stamp_document(&now);
        }
        assert_eq!(public.bytes, stamped.to_bytes().unwrap());
    }
    for (selector, connector) in [("missing", "Q"), ("uuid:a", "q")] {
        let before = migrated.to_bytes().unwrap();
        assert_eq!(
            frozen_unwire(&mut frozen, selector, connector)
                .unwrap_err()
                .to_string(),
            migrated
                .unwire(selector, connector)
                .unwrap_err()
                .to_string()
        );
        assert_eq!(before, migrated.to_bytes().unwrap());
    }
}
