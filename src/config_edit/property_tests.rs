use super::{ConfigEditor, editor_error, property_report};
use anyhow::Result;
use lox_core::configuration::editor::set_property;
use xmltree::{Element, XMLNode};

// Frozen public 5bf766f mutation/report oracle, not used by production.
fn old_set_property(
    editor: &mut ConfigEditor,
    selector: &str,
    prop_name: &str,
    value: &str,
    type_code: &str,
) -> Result<String> {
    let path = editor.require_one(selector)?;
    let elem = editor.get_element_mut(&path);
    let title = elem.attributes.get("Title").cloned().unwrap_or_default();
    let set_idx = elem
        .children
        .iter()
        .position(|c| c.as_element().map(|e| e.name == "SET").unwrap_or(false));
    let set_elem = if let Some(idx) = set_idx {
        elem.children[idx].as_mut_element().unwrap()
    } else {
        elem.children.push(XMLNode::Element(Element::new("SET")));
        let last = elem.children.len() - 1;
        elem.children[last].as_mut_element().unwrap()
    };
    let prop_idx = set_elem
        .children
        .iter()
        .position(|c| c.as_element().map(|e| e.name == prop_name).unwrap_or(false));
    if let Some(idx) = prop_idx {
        let prop = set_elem.children[idx].as_mut_element().unwrap();
        let old_val = prop.attributes.get("v").cloned().unwrap_or_default();
        prop.attributes
            .insert("t".to_string(), type_code.to_string());
        prop.attributes.insert("v".to_string(), value.to_string());
        Ok(format!(
            "Updated {}.{}: '{}' → '{}'",
            title, prop_name, old_val, value
        ))
    } else {
        let mut prop = Element::new(prop_name);
        prop.attributes
            .insert("t".to_string(), type_code.to_string());
        prop.attributes.insert("v".to_string(), value.to_string());
        set_elem.children.push(XMLNode::Element(prop));
        Ok(format!("Created {}.{} = '{}'", title, prop_name, value))
    }
}

#[test]
fn set_property_public_transaction_cached_head_and_frozen_public_oracle_agree() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap();
    let xml = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList><C Type=\"Document\" Date=\"future\" DateS=\"999999999\"/><C Type=\"And\" U=\"a\" Title=\"Gate\"><SET><P Other=\"+003\" t=\"11\" v=\"old\"><Child/></P><P v=\"second\"/></SET><SET><P v=\"last\"/></SET></C></ControlList>\r\n";
    for (input, property, value, code) in [
        (xml.as_bytes(), "P", "old", "11"),
        (xml.as_bytes(), "P", "old", "15"),
        (xml.as_bytes(), "P", "雪 & <", "-007"),
        (xml.as_bytes(), "New", "", ""),
        (b"<C U='a'/>".as_slice(), "P", "+003", "raw"),
        (b"<C U='a'><SET><P/></SET></C>".as_slice(), "P", "", "11"),
        (b"<C U='a'/>".as_slice(), "a/b", "", "raw"),
    ] {
        let baseline = ConfigEditor::load(input).unwrap().to_bytes().unwrap();
        let mut old = ConfigEditor::load(input).unwrap();
        let expected_report = old_set_property(&mut old, "uuid:a", property, value, code).unwrap();
        let mut cached = ConfigEditor::load(input).unwrap();
        assert_eq!(
            cached
                .set_property("uuid:a", property, value, code)
                .unwrap(),
            expected_report
        );
        assert_eq!(cached.to_bytes().unwrap(), old.to_bytes().unwrap());
        let changed = baseline != old.to_bytes().unwrap();
        let stamped = changed && old.stamp_document(&now);
        let actual = set_property(input, "uuid:a", property, value, code, &now).unwrap();
        assert_eq!(property_report(&actual.change), expected_report);
        assert_eq!(actual.bytes, old.to_bytes().unwrap());
        assert_eq!(actual.content_changed, changed);
        assert_eq!(actual.document_stamped, stamped);
    }
}

#[test]
fn set_property_preserves_concrete_head_errors_and_native_boxed_cause() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap();
    for (input, selector) in [
        (b"<C>".as_slice(), "missing"),
        (b"<C A='1' A='2'/>".as_slice(), "missing"),
        (b"<C Title='\xff'/>".as_slice(), ""),
        (b"<C Title='Gate'/>".as_slice(), "missing"),
        (
            b"<R><C U='a' Title='Gate'/><C U='b' Title='Gate'/></R>".as_slice(),
            "Gate",
        ),
    ] {
        let old = ConfigEditor::load(input)
            .and_then(|mut e| old_set_property(&mut e, selector, "", "", ""))
            .unwrap_err();
        let new = editor_error(set_property(input, selector, "", "", "", &now).unwrap_err());
        assert_eq!(format!("{old:?}"), format!("{new:?}"));
        assert_eq!(format!("{old:#}"), format!("{new:#}"));
        assert_eq!(
            old.chain().map(ToString::to_string).collect::<Vec<_>>(),
            new.chain().map(ToString::to_string).collect::<Vec<_>>()
        );
        assert_eq!(old.root_cause().to_string(), new.root_cause().to_string());
        assert_eq!(
            old.downcast_ref::<String>().is_some(),
            new.downcast_ref::<String>().is_some()
        );
        assert_eq!(
            old.downcast_ref::<&'static str>().is_some(),
            new.downcast_ref::<&'static str>().is_some()
        );
        let boxed_native = |error: &anyhow::Error| {
            error
                .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
                .is_some_and(|cause| cause.as_ref().is::<xmltree::ParseError>())
        };
        assert_eq!(boxed_native(&old), boxed_native(&new));
        assert_eq!(
            old.downcast_ref::<xmltree::ParseError>().is_some(),
            new.downcast_ref::<xmltree::ParseError>().is_some()
        );
    }
}
