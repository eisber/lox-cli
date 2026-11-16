use chrono::{DateTime, Duration, FixedOffset};
use lox_core::configuration::editor::{EditError, SelectionError, remove_element, set_param};
use std::error::Error;

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap()
}

#[test]
fn canonical_bytes_preserve_native_collateral_and_first_connector_semantics() {
    let before = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<ControlList Version=\"267\">\r\n\
\t<C Type=\"VirtualIn\" U=\"a\" Title=\"Control\" 12hTF=\"true\" NTXT=\"first\nsecond\">\r\n\
\t\t<Co K=\"I1\" Def=\"1\"/>\r\n\
\t\t<Co K=\"I1\" Def=\"mirror\"/>\r\n\
\t\t<IoData></IoData>\r\n\
\t\t<Co K=\"Q\" U=\"wire\">\r\n\
\t\t\t<In Input=\"first\"/>\r\n\
\t\t\t<In Input=\"second\" FLG=\"2\"/>\r\n\
\t\t</Co>\r\n\
\t</C>\r\n\
\t<C Type=\"Future\" U=\"collateral\" Title=\"Other\" Value=\"&quot; &amp; &lt;\"/>\r\n\
</ControlList>\r\n";
    let outcome = set_param(before.as_bytes(), "uuid:a", "I1", "42", &now()).unwrap();
    assert_eq!(
        outcome.bytes,
        before.replacen("Def=\"1\"", "Def=\"42\"", 1).as_bytes()
    );
    assert_eq!(outcome.change.old_value.as_deref(), Some("1"));
    assert_eq!(outcome.change.element.uuid, "a");
    assert!(outcome.content_changed);
    assert!(!outcome.document_stamped);
}

#[test]
fn explicit_stamps_are_monotonic_and_canonical_noops_do_not_stamp() {
    let xml = br#"<C Type="Document" Date="2026-10-12 07:55:33" DateS="561020133"><C Type="And" U="a" Title="Gate"><Co K="I1" Def="1"/></C></C>"#;
    let noop = set_param(xml, "uuid:a", "I1", "1", &now()).unwrap();
    assert!(!noop.content_changed);
    assert!(!noop.document_stamped);
    assert_ne!(noop.bytes, xml);
    for (input, clock, expected) in [
        (xml.as_slice(), now(), 561_020_134),
        (xml.as_slice(), now() - Duration::days(1), 561_020_134),
        (xml.as_slice(), now() + Duration::seconds(5), 561_020_138),
    ] {
        let changed = set_param(input, "uuid:a", "I1", "2", &clock).unwrap();
        assert!(changed.document_stamped);
        let text = String::from_utf8(changed.bytes).unwrap();
        assert!(text.contains(&format!("DateS=\"{expected}\"")));
    }
}

#[test]
fn legacy_selection_room_and_uuid_mirrors_share_one_kernel() {
    let xml = br#"<ControlList>
<C Type="Place" U="r" Title="Kitchen"/>
<C Type="And" U="a" Title="Gate"><Co K="I1"/><IoData Pr="r"/></C>
<C Type="And" U="a" Title="Gate"><Co K="I1" Def="mirror"/><IoData Pr="r"/></C>
<C Type="And" U="b" Title="Gate"><Co K="I1"/><IoData Pr="elsewhere"/></C>
</ControlList>"#;
    let changed = set_param(xml, "gate [kitchen]", "I1", "2", &now()).unwrap();
    assert_eq!(changed.change.old_value, None);
    assert_eq!(changed.change.element.uuid, "a");
    assert!(
        String::from_utf8(changed.bytes)
            .unwrap()
            .contains("Def=\"mirror\"")
    );
    assert!(matches!(
        set_param(xml, "Gate", "I1", "2", &now()),
        Err(EditError::Selection(SelectionError::Ambiguous { matches, .. })) if matches.len() == 2
    ));
}

#[test]
fn refusals_are_typed_and_leave_borrowed_input_unchanged() {
    let xml = br#"<C Type="VirtualIn" U="a" Title="Gate" MaxVal="100"><Co K="I1"/></C>"#;
    let saved = xml.to_vec();
    assert!(matches!(
        set_param(xml, "uuid:a", "MaxVal", "200", &now()),
        Err(EditError::ParameterMissing { parameter, .. }) if parameter == "MaxVal"
    ));
    assert!(matches!(
        set_param(xml, "missing", "I1", "2", &now()),
        Err(EditError::Selection(SelectionError::NotFound { titles, .. })) if titles == ["Gate"]
    ));
    let error = set_param(b"<C A='1' A='2'/>", "", "I1", "2", &now()).unwrap_err();
    assert!(matches!(error, EditError::Load(_)));
    assert!(error.source().is_some());
    assert_eq!(xml.as_slice(), saved);
    let identical = br#"<C Type='And' U='a' Title='Gate' Title="Gate"><Co K='I1' Def='1'/></C>"#;
    assert!(set_param(identical, "Type:And", "I1", "2", &now()).is_ok());
}

#[test]
fn removal_is_sibling_first_exact_and_does_not_clean_references() {
    let xml = br#"<Root U="root"><Container><C U="x" Title="nested"><C U="descendant"/></C></Container><Other U="x" Title="direct"><C U="subtree"/></Other><C U="x" Title="mirror"/><Co U="wire"><In Input="x"/></Co><PRM REF="x"/></Root>"#;
    let first = remove_element(xml, "x", &now()).unwrap();
    assert_eq!(first.removed_title, "direct");
    let text = String::from_utf8(first.bytes.clone()).unwrap();
    assert!(!text.contains("subtree"));
    assert!(text.contains("Title=\"nested\""));
    assert!(text.contains("Title=\"mirror\""));
    assert!(text.contains("<In Input=\"x\"/>"));
    assert!(text.contains("<PRM REF=\"x\"/>"));
    let second = remove_element(&first.bytes, "x", &now()).unwrap();
    assert_eq!(second.removed_title, "mirror");
    let third = remove_element(&second.bytes, "x", &now()).unwrap();
    assert_eq!(third.removed_title, "nested");
    assert!(
        !String::from_utf8(third.bytes)
            .unwrap()
            .contains("descendant")
    );
    for raw in ["root", "X", "uuid:x"] {
        assert!(matches!(
            remove_element(xml, raw, &now()),
            Err(EditError::ExactUuidNotFound { uuid }) if uuid == raw
        ));
    }
}

#[test]
fn removal_empty_unicode_missing_title_and_recursive_branch_priority() {
    let xml = "<R><A><B><Deep U=\"ü\" Title=\"first branch\"/></B></A><B><Shallow U=\"ü\" Title=\"second branch\"/></B><Missing/><Empty U=\"\"/><Literal U=\"uuid:x\"/></R>";
    let removed = remove_element(xml.as_bytes(), "ü", &now()).unwrap();
    assert_eq!(removed.removed_title, "first branch");
    assert!(
        String::from_utf8(removed.bytes)
            .unwrap()
            .contains("second branch")
    );
    let empty = remove_element(xml.as_bytes(), "", &now()).unwrap();
    assert_eq!(empty.removed_title, "");
    let text = String::from_utf8(empty.bytes).unwrap();
    assert!(text.contains("<Missing></Missing>"));
    assert!(!text.contains("<Empty"));
    assert!(remove_element(xml.as_bytes(), "uuid:x", &now()).is_ok());
}

#[test]
fn removal_preserves_canonical_bom_eol_stamps_and_load_refusal_order() {
    let xml = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<C Type=\"Document\" Date=\"2026-10-12 07:55:33\" DateS=\"561020133\">\r\n\t<Other U=\"gone\"/>\r\n\t<C U=\"keep\" NTXT=\"first\nsecond\"/>\r\n</C>\r\n";
    let before = xml.as_bytes().to_vec();
    let outcome = remove_element(&before, "gone", &(now() - Duration::days(1))).unwrap();
    assert_eq!(
        outcome.bytes,
        xml.replace("\t<Other U=\"gone\"/>\r\n", "")
            .replace("DateS=\"561020133\"", "DateS=\"561020134\"")
            .replace(
                "Date=\"2026-10-12 07:55:33\"",
                "Date=\"2026-10-12 07:55:34\""
            )
            .as_bytes()
    );
    assert!(outcome.content_changed && outcome.document_stamped);
    let missing = remove_element(&before, "absent", &now()).unwrap_err();
    assert_eq!(missing.to_string(), "Element with UUID 'absent' not found");
    assert!(missing.source().is_none());
    assert_eq!(before, xml.as_bytes());
    let load = remove_element(b"<R A='1' A='2'/>", "absent", &now()).unwrap_err();
    assert!(matches!(load, EditError::Load(_)));
    assert!(load.source().is_some());
}
