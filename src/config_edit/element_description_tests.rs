use super::{ConfigEditor, ConnectorInfo, ElementDescription, PropertyValue};
use std::collections::HashMap;
use xmltree::Element;

// Frozen public 5bf766ff producer, retained only as a characterization oracle.
fn legacy_description(elem: &Element) -> ElementDescription {
    let mut properties = HashMap::new();
    let mut connectors = Vec::new();
    let mut children = Vec::new();
    let mut room = String::new();
    let mut category = String::new();
    for child in &elem.children {
        if let Some(child_elem) = child.as_element() {
            match child_elem.name.as_str() {
                "SET" => {
                    for prop_node in &child_elem.children {
                        if let Some(prop) = prop_node.as_element() {
                            let val = prop.attributes.get("v").cloned().unwrap_or_default();
                            let t = prop.attributes.get("t").cloned().unwrap_or_default();
                            properties.insert(
                                prop.name.clone(),
                                PropertyValue {
                                    value: val,
                                    type_code: t,
                                },
                            );
                        }
                    }
                }
                "Co" => {
                    connectors.push(ConnectorInfo {
                        kind: child_elem.attributes.get("K").cloned().unwrap_or_default(),
                        target: child_elem.attributes.get("U").cloned().unwrap_or_default(),
                    });
                }
                "IoData" => {
                    room = child_elem.attributes.get("Pr").cloned().unwrap_or_default();
                    category = child_elem.attributes.get("Cr").cloned().unwrap_or_default();
                }
                "C" => {
                    let ct = child_elem
                        .attributes
                        .get("Type")
                        .cloned()
                        .unwrap_or_default();
                    let ct_title = child_elem
                        .attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_default();
                    children.push(format!("{}: {}", ct, ct_title));
                }
                _ => {}
            }
        }
    }
    ElementDescription {
        element_type: elem.attributes.get("Type").cloned().unwrap_or_default(),
        title: elem.attributes.get("Title").cloned().unwrap_or_default(),
        uuid: elem.attributes.get("U").cloned().unwrap_or_default(),
        gid: elem.attributes.get("gid").cloned().unwrap_or_default(),
        room_uuid: room,
        category_uuid: category,
        properties,
        connectors,
        children,
    }
}

#[test]
fn selected_description_cached_head_and_public_core_match_frozen_legacy_walk() {
    let xml = r#"<ControlList><C Type="Place" U="r" Title="Küche"/>
<C Type="Future" U="a" Title="Gate" gid="g" 12hTF="true"><SET><X v="first" t="11"/><X v="next"/><Empty/></SET><Co K="Q" U="RAW" Def="ignored"/><Co/><IoData Pr="r" Cr="c"/><C Type="Child" Title="室 &amp; é"><SET><Y v="nested"/></SET></C><Unknown><Co K="hidden"/><IoData Pr="hidden"/></Unknown><SET><X v="last" t="15"/><Y v="&quot; &amp; &lt;"/></SET><IoData/></C>
<C Type="Future" U="a" Title="Gate"><IoData Pr="r"/></C>
<C U="sparse" Title="Sparse"/><C U="other" Title="Gate"><IoData Pr="elsewhere"/></C></ControlList>"#;
    for newline in ["\n", "\r\n"] {
        for bom in ["", "\u{feff}"] {
            let bytes = format!("{bom}{}", xml.replace('\n', newline)).into_bytes();
            let editor = ConfigEditor::load(&bytes).unwrap();
            for selector in ["uuid:a", "gid:g", "Gate [KÜCHE]", "Sparse", "Type:Child"] {
                let path = editor.require_one(selector).unwrap();
                let frozen = legacy_description(editor.get_element(&path));
                let head = editor.describe(selector).unwrap();
                let core =
                    lox_core::configuration::editor::describe_element(&bytes, selector).unwrap();
                let adapted = ElementDescription::from(core);
                // Object equality ignores randomized HashMap order, not value/array order.
                assert_eq!(
                    serde_json::to_value(&head).unwrap(),
                    serde_json::to_value(&frozen).unwrap()
                );
                assert_eq!(
                    serde_json::to_value(&adapted).unwrap(),
                    serde_json::to_value(&frozen).unwrap()
                );
            }
        }
    }
}

#[test]
fn selected_description_head_errors_preserve_selection_and_native_boxed_load_causes() {
    fn observation(error: &anyhow::Error) -> (Vec<String>, Vec<bool>) {
        (
            vec![
                format!("{error}"),
                format!("{error:?}"),
                format!("{error:#}"),
                format!("{}", error.root_cause()),
                format!("{:?}", error.root_cause()),
                error
                    .chain()
                    .map(|cause| format!("{cause}\n{cause:?}"))
                    .collect::<Vec<_>>()
                    .join("\n---\n"),
            ],
            vec![
                error.downcast_ref::<String>().is_some(),
                error.downcast_ref::<&'static str>().is_some(),
                error.downcast_ref::<xmltree::ParseError>().is_some(),
                error
                    .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
                    .is_some(),
                error.root_cause().is::<xmltree::ParseError>(),
                error.chain().any(|cause| cause.is::<xmltree::ParseError>()),
            ],
        )
    }

    let editor =
        ConfigEditor::load(br#"<Root><C U="a" Title="Gate"/><C U="b" Title="Gate"/></Root>"#)
            .unwrap();
    for selector in ["Gate", "Gat", "missing"] {
        assert_eq!(
            observation(&editor.describe(selector).unwrap_err()),
            observation(&editor.require_one(selector).unwrap_err())
        );
    }
    let bytes = b"<C A='1' A='2'/>";
    let public = lox_core::configuration::editor::describe_element(bytes, "").unwrap_err();
    let adapted = super::editor_error(public);
    let cached = ConfigEditor::load(bytes).err().unwrap();
    assert_eq!(format!("{adapted:#}"), format!("{cached:#}"));
    // The unchanged public loader is the oracle for anyhow's boxed-cause chain.
    // ParseError need not be the root cause or directly downcastable in anyhow.
    assert_eq!(observation(&adapted), observation(&cached));
    let lox_core::configuration::editor::EditError::Load(cause) =
        lox_core::configuration::editor::describe_element(bytes, "").unwrap_err()
    else {
        panic!("expected the native Core load cause");
    };
    assert!(cause.into_inner().downcast::<xmltree::ParseError>().is_ok());
}
