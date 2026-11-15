use super::{ConfigEditor, WireInfo, editor_error};

// Frozen producer from public base 9dcbbf94, not the new Core implementation.
fn old_list_wires(editor: &ConfigEditor, selector: &str) -> anyhow::Result<Vec<WireInfo>> {
    let path = editor.require_one(selector)?;
    let elem = editor.get_element(&path);
    let mut wires = Vec::new();
    for child in &elem.children {
        if let Some(co) = child.as_element()
            && co.name == "Co"
        {
            let name = co.attributes.get("K").cloned().unwrap_or_default();
            let target_uuid = co.attributes.get("U").cloned().unwrap_or_default();
            let direction = if name.starts_with('I') || name.starts_with("AI") || name == "Input" {
                "input"
            } else if name.starts_with('Q') || name.starts_with("AQ") || name.starts_with("Output")
            {
                "output"
            } else {
                "parameter"
            };
            let has_in_children = co
                .children
                .iter()
                .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
            let connected = has_in_children
                || (!target_uuid.is_empty()
                    && target_uuid != "00000000-0000-0000-0000000000000000");
            wires.push(WireInfo {
                connector: name,
                direction: direction.to_string(),
                target_uuid,
                connected,
            });
        }
    }
    Ok(wires)
}

fn public_wires(bytes: &[u8], selector: &str) -> anyhow::Result<Vec<WireInfo>> {
    lox_core::configuration::wiring::connector_wiring(bytes, selector)
        .map(|wires| wires.into_iter().map(Into::into).collect())
        .map_err(editor_error)
}

#[test]
fn wiring_catalog_frozen_loop_matches_public_bytes_and_cached_editor() {
    let mut connectors = String::new();
    for name in [
        "I",
        "Input",
        "INPUT",
        "AIQ",
        "AI",
        "Q",
        "AQI",
        "Output",
        "OutputMore",
        "output",
        "input",
        "Aq",
        " I",
        "Ω",
        "",
        "Q",
    ] {
        for target in [
            "",
            "00000000-0000-0000-0000000000000000",
            "00000000-0000-0000-000000000000000",
            "00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-0000000000000000",
            "00000000-0000-0000-0000-000000000000000",
            "not-a-uuid",
            " ",
            "零",
        ] {
            for child in ["", "<In/>", "<Other><In/></Other>", "<in/>", "<In/><In/>"] {
                connectors.push_str(&format!("<Co K='{name}' U='{target}'>{child}</Co>"));
            }
        }
    }
    connectors.push_str(
        "<Co/><Co K='I1'/><Co U='raw'/><!-- ignored --><Other><Co K='Q'/></Other><co K='Q'/>",
    );
    let xml = format!(
        "<ControlList><C Type='Place' U='room' Title='Kitchen'/><C Type='And' gid='GateGroup' U='owner' Title='Gate'>{connectors}<IoData Pr='room'/></C><C Type='And' U='owner' Title='Gate'><Co K='mirror'/><IoData Pr='room'/></C></ControlList>"
    );
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    for selector in [
        "uuid:owner",
        "gate",
        "Type:And",
        "gid:GateGroup",
        "Gate [kitchen]",
    ] {
        let expected = serde_json::to_value(old_list_wires(&editor, selector).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(public_wires(xml.as_bytes(), selector).unwrap()).unwrap(),
            expected,
            "{selector}"
        );
        assert_eq!(
            serde_json::to_value(editor.list_wires(selector).unwrap()).unwrap(),
            expected,
            "{selector}"
        );
    }
}

fn error_signature(error: anyhow::Error) -> (String, Vec<String>, String, bool, bool) {
    (
        format!("{error:#}"),
        error.chain().map(ToString::to_string).collect(),
        error.root_cause().to_string(),
        error.downcast_ref::<xmltree::ParseError>().is_some(),
        error
            .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
            .is_some(),
    )
}

#[test]
fn wiring_catalog_preserves_selection_loader_and_concrete_error_routes() {
    let parser = public_wires(b"<C>", "Gate").unwrap_err();
    assert!(parser.downcast_ref::<xmltree::ParseError>().is_none());
    assert!(
        parser
            .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
            .is_some()
    );
    assert!(!parser.root_cause().is::<xmltree::ParseError>());
    for (bytes, selector) in [
        (b"<C>".as_slice(), "Gate"),
        (b"<C Title='\xff'/>".as_slice(), "Gate"),
        (b"<C A='1' A='2'/>".as_slice(), ""),
        (b"<C Title='Gate'/>".as_slice(), "Missing"),
        (
            b"<R><C Title='Gate' U='a'/><C Title='Gate' U='b'/></R>".as_slice(),
            "Gate",
        ),
        // lox-n37w: global digit-name replacement also reaches quoted text.
        (
            b"<C Title='quoted 12hTF=x' 12hTF='true'/>".as_slice(),
            "quoted 12hTF=x",
        ),
    ] {
        let expected = ConfigEditor::load(bytes).and_then(|e| old_list_wires(&e, selector));
        assert_eq!(
            error_signature(public_wires(bytes, selector).unwrap_err()),
            error_signature(expected.unwrap_err())
        );
    }
    let xml = b"<C Title='quoted 12hTF=x' 12hTF='true'><Co K='I'/></C>";
    assert_eq!(public_wires(xml, "quoted _12hTF=x").unwrap().len(), 1);
}
