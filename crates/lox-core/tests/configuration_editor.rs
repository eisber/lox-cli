use chrono::{DateTime, Duration, FixedOffset};
use lox_core::configuration::editor::{
    ControlMoveSelector, EditError, SelectionError, move_control, set_param,
};
use std::error::Error;

#[test]
fn program_reader_first_direct_field_text_and_shared_legacy_selection() {
    use lox_core::configuration::editor::get_program_text;
    let xml = br#"<Root Date="fixed" DateS="99"><C Type="Place" U="r" Title="Kitchen"/>
<C Type="SequenceController" U="s" Title="Program" gid="g" foreign="keep"><IoData Pr="r"/>
<Unknown><Field Name="Configuration">nested</Field></Unknown><Field Name="configuration">wrong</Field>
<Field Name="Configuration">first<Nested>hidden</Nested>second<![CDATA[hidden]]>third</Field>
<Field Name="Configuration">later</Field></C>
<C Type="SequenceController" U="s" Title="Program"><Field Name="Configuration">mirror</Field></C>
<C Type="SequenceController" U="empty" Title="Empty"><Field Name="configuration">wrong</Field></C></Root>"#;
    let before = xml.to_vec();
    for selector in ["uuid:s", "gid:g", "Program [Kitchen]"] {
        assert_eq!(get_program_text(xml, selector).unwrap(), "firstsecondthird");
    }
    assert_eq!(get_program_text(xml, "Empty").unwrap(), "");
    assert_eq!(xml.as_slice(), before);
    let legacy = br#"<C Type="SequenceController" U="s" 12hTF="true"><Field Name="Configuration">text 12hTF=value</Field></C>"#;
    assert_eq!(
        get_program_text(legacy, "uuid:s").unwrap(),
        "text _12hTF=value"
    );
}

#[test]
fn program_reader_typed_load_selection_and_exact_type_error_priority() {
    use lox_core::configuration::editor::{ProgramTextError, get_program_text};
    let xml = br#"<Root><C Type="And" U="a" Title="Program"/><C Type="SequenceController" U="b" Title="Program"/><C Type="sequencecontroller" U="c" Title="Wrong"/><C U="sparse"/></Root>"#;
    assert!(matches!(
        get_program_text(xml, "Program")
            .unwrap_err()
            .source()
            .and_then(|cause| cause.downcast_ref::<EditError>()),
        Some(EditError::Selection(SelectionError::Ambiguous { .. }))
    ));
    assert!(matches!(
        get_program_text(xml, "missing"),
        Err(ProgramTextError::Edit(EditError::Selection(
            SelectionError::NotFound { .. }
        )))
    ));
    assert!(
        matches!(get_program_text(xml, "Wrong"), Err(ProgramTextError::Unsupported { element }) if element.block_type == "sequencecontroller")
    );
    assert!(
        matches!(get_program_text(xml, "uuid:sparse"), Err(ProgramTextError::Unsupported { element }) if element.block_type.is_empty() && element.title.is_empty())
    );
    let error = get_program_text(b"<C A='1' A='2'/>", "missing").unwrap_err();
    assert!(
        error
            .source()
            .and_then(|cause| cause.downcast_ref::<EditError>())
            .and_then(Error::source)
            .unwrap()
            .downcast_ref::<xmltree::ParseError>()
            .is_some()
    );
    let ProgramTextError::Edit(EditError::Load(cause)) = error else {
        panic!("load must precede selection")
    };
    assert!(cause.into_inner().downcast::<xmltree::ParseError>().is_ok());
}

#[test]
fn selected_description_preserves_direct_raw_observations_and_last_precedence() {
    use lox_core::configuration::editor::describe_element;
    let xml = include_bytes!("fixtures/element-description.Loxone");
    let before = xml.to_vec();
    let description = describe_element(xml, "uuid:a").unwrap();
    assert_eq!(description.properties["Repeated"].value, "last");
    assert_eq!(description.properties["Repeated"].type_code, "raw");
    assert_eq!(description.properties["Empty"].value, "");
    assert_eq!(description.properties["Entity"].value, "\" & <");
    assert!(!description.properties.contains_key("Hidden"));
    assert_eq!(description.room_uuid, "last-room");
    assert_eq!(description.category_uuid, "");
    assert_eq!(
        description
            .connectors
            .iter()
            .map(|c| (c.kind.as_str(), c.target.as_str()))
            .collect::<Vec<_>>(),
        [("AQ1", "{RAW-ID}"), ("", ""), ("AQ1", "second")]
    );
    assert_eq!(description.children.len(), 2);
    assert_eq!(description.children[0].title, "Nested & 室");
    assert_eq!(description.children[1].element_type, "");
    assert_eq!(
        describe_element(xml, "gate [KÜCHE 室]").unwrap(),
        description
    );
    assert_eq!(describe_element(xml, "gid:raw-gid").unwrap(), description);
    assert_eq!(
        describe_element(xml, "uuid:nested").unwrap().properties["Own"].value,
        "child"
    );
    let sparse = describe_element(xml, "Sparse").unwrap();
    assert!(sparse.room_uuid.is_empty() && sparse.connectors.is_empty());
    assert_eq!(xml.as_slice(), before);
    for ending in ["\n", "\r\n"] {
        let text = std::str::from_utf8(xml).unwrap().replace('\n', ending);
        for bom in ["", "\u{feff}"] {
            assert_eq!(
                describe_element(format!("{bom}{text}").as_bytes(), "uuid:a").unwrap(),
                description
            );
        }
    }
}

#[test]
fn selected_description_typed_failures_keep_concrete_parser_cause() {
    use lox_core::configuration::editor::describe_element;
    let xml = include_bytes!("fixtures/element-description.Loxone");
    assert!(matches!(
        describe_element(xml, "Gate"),
        Err(EditError::Selection(SelectionError::Ambiguous { matches, .. })) if matches.len() == 2
    ));
    assert!(matches!(
        describe_element(xml, "missing"),
        Err(EditError::Selection(SelectionError::NotFound { titles, .. })) if titles.contains(&"Sparse".to_owned())
    ));
    for bytes in [b"".as_slice(), b"<C A='1' A='2'/>", b"<C>"] {
        let error = describe_element(bytes, "missing").unwrap_err();
        assert!(matches!(error, EditError::Load(_)));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<xmltree::ParseError>()
                .is_some()
        );
        let EditError::Load(cause) = error else {
            unreachable!()
        };
        assert!(cause.into_inner().downcast::<xmltree::ParseError>().is_ok());
    }
    let legacy = br#"<C U="a" Title="text 12hTF=value" 12hTF="true"/>"#;
    assert_eq!(
        describe_element(legacy, "uuid:a").unwrap().title,
        "text _12hTF=value"
    );
}

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap()
}

#[test]
fn legacy_attribute_identity_previous_option_and_raw_values() {
    use lox_core::configuration::editor::{describe_element, set_attribute};
    let xml = br#"<ControlList><C Type="And" U="a" Title="Old" Empty=""/><C Type="And" U="a" Title="Mirror"/></ControlList>"#;
    let renamed = set_attribute(xml, "uuid:a", "Title", "New", &now()).unwrap();
    assert_eq!(renamed.change.element.title, "Old");
    assert_eq!(renamed.change.old_value.as_deref(), Some("Old"));
    assert_eq!(renamed.change.new_value, "New");
    assert!(String::from_utf8_lossy(&renamed.bytes).contains("Title=\"Mirror\""));
    let empty = set_attribute(&renamed.bytes, "uuid:a", "Empty", "", &now()).unwrap();
    assert_eq!(empty.change.old_value.as_deref(), Some(""));
    assert!(!empty.content_changed);
    let raw = set_attribute(&empty.bytes, "uuid:a", "Future", "<&\"\n室", &now()).unwrap();
    assert_eq!(raw.change.old_value, None);
    assert_eq!(raw.change.attribute, "Future");
    assert_eq!(raw.change.new_value, "<&\"\n室");
    assert_eq!(describe_element(&raw.bytes, "uuid:a").unwrap().title, "New");
    assert!(String::from_utf8_lossy(&raw.bytes).contains("Future=\"&lt;&amp;&quot;\n室\""));
}

#[test]
fn legacy_attribute_canonical_noop_stamp_and_bom_crlf() {
    use lox_core::configuration::editor::set_attribute;
    let xml = "\u{feff}<C Type=\"Document\" DateS=\"561020133\">\r\n<C U=\"a\" Title=\"Old\"/>\r\n</C>\r\n";
    let noop = set_attribute(xml.as_bytes(), "uuid:a", "Title", "Old", &now()).unwrap();
    assert!(!noop.content_changed && !noop.document_stamped);
    assert_ne!(noop.bytes, xml.as_bytes());
    let edit = set_attribute(xml.as_bytes(), "uuid:a", "NTXT", "one\ntwo", &now()).unwrap();
    assert!(edit.content_changed && edit.document_stamped);
    assert!(edit.bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    let text = String::from_utf8(edit.bytes).unwrap();
    assert!(text.contains("DateS=\"561020134\""));
    assert!(text.contains("Date=\"2026-10-12 07:55:34\""));
    assert!(text.contains("NTXT=\"one\ntwo\""));
    assert!(text.ends_with("\r\n"));
}

#[test]
fn legacy_attribute_refusals_preserve_input_and_parser_cause() {
    use lox_core::configuration::editor::set_attribute;
    let xml = br#"<ControlList><C U="a" Title="Gate"/><C U="b" Title="Gate"/></ControlList>"#;
    let before = xml.to_vec();
    assert!(matches!(
        set_attribute(xml, "missing", "", "", &now()),
        Err(EditError::Selection(SelectionError::NotFound { .. }))
    ));
    assert!(matches!(
        set_attribute(xml, "Gate", "", "", &now()),
        Err(EditError::Selection(SelectionError::Ambiguous { matches, .. })) if matches.len() == 2
    ));
    let error = set_attribute(b"<C>", "missing", "", "", &now()).unwrap_err();
    assert!(matches!(error, EditError::Load(_)));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<xmltree::ParseError>()
            .is_some()
    );
    assert_eq!(xml.as_slice(), before);
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
fn control_move_preserves_direct_associations_collateral_and_explicit_stamp() {
    let before = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<ControlList>\r\n\
\t<C Type=\"Document\" Date=\"2026-10-12 07:55:33\" DateS=\"561020133\"/>\r\n\
\t<C Type=\"Place\" U=\"r\" Title=\"Küche 室\"/>\r\n\
\t<C Type=\"Future\" U=\"a\" Title=\"Lamp\" 12hTF=\"true\">\r\n\
\t\t<IoData Pr=\"old\" Cr=\"cat\" Future=\"+003\"/>\r\n\
\t\t<IoData Pr=\"\"/>\r\n\
\t\t<IoData/>\r\n\
\t\t<Unknown>\r\n\
\t\t\t<IoData Pr=\"nested\"/>\r\n\
\t\t</Unknown>\r\n\
\t\t<Co K=\"AQ\" U=\"wire\">\r\n\
\t\t\t<In Input=\"source\" FLG=\"01\"/>\r\n\
\t\t</Co>\r\n\
\t</C>\r\n\
</ControlList>\r\n";
    let moved = move_control(
        before.as_bytes(),
        None,
        Some("Lamp"),
        "KÜCHE 室",
        &["Future"],
        &(now() - Duration::days(1)),
    )
    .unwrap();
    assert!(moved.content_changed && moved.document_stamped);
    assert_eq!(moved.change.selected_count, 1);
    assert!(moved.change.association_changed);
    let expected = before
        .replace(
            "Date=\"2026-10-12 07:55:33\"",
            "Date=\"2026-10-12 07:55:34\"",
        )
        .replace("DateS=\"561020133\"", "DateS=\"561020134\"")
        .replace("Pr=\"old\"", "Pr=\"r\"")
        .replace("Pr=\"\"", "Pr=\"r\"")
        .replace("<IoData/>", "<IoData Pr=\"r\"/>");
    assert_eq!(moved.bytes, expected.as_bytes());
    let noop = move_control(&moved.bytes, None, Some("Lamp"), "Küche 室", &[], &now()).unwrap();
    assert!(!noop.content_changed && !noop.document_stamped);
    assert_eq!(noop.bytes, moved.bytes);
}

#[test]
fn control_move_type_and_title_keep_distinct_mirror_exclusion_and_sparse_rules() {
    let xml = br#"<ControlList><C Type="Place" U="r" Title="Room"/>
<C Type="Future" U="a" Title="Lamp"><IoData Pr="r"/></C>
<C Type="Future" U="a" Title="Mirror"><IoData Pr="old"/></C>
<C Type="Future" U="dry" Title="Dry"><Unknown><IoData Pr="old"/></Unknown></C>
</ControlList>"#;
    let typed = move_control(xml, Some("fUtUrE"), Some("missing"), "Room", &[], &now()).unwrap();
    assert_eq!(
        typed.change.selector,
        ControlMoveSelector::Type("fUtUrE".into())
    );
    assert_eq!(typed.change.selected_count, 2);
    assert!(typed.content_changed && !typed.document_stamped);
    let excluded = move_control(xml, Some("Future"), None, "Room", &["FUTURE"], &now()).unwrap();
    assert_eq!(excluded.change.selected_count, 0);
    assert!(!excluded.content_changed && !excluded.document_stamped);
    assert_ne!(excluded.bytes, xml);
    let title = move_control(xml, None, Some("uuid:a"), "Room", &["Future"], &now()).unwrap();
    assert_eq!(title.change.selected_count, 1);
    assert!(!title.content_changed);
    assert!(
        String::from_utf8(title.bytes)
            .unwrap()
            .contains("Pr=\"old\"")
    );
    let dry = move_control(xml, None, Some("Dry"), "Room", &[], &now()).unwrap();
    assert_eq!(dry.change.selected_count, 1);
    assert!(!dry.change.association_changed && !dry.content_changed);
}

#[test]
fn control_move_typed_refusals_preserve_branch_error_order_and_room_duplicates() {
    let xml = br#"<ControlList><C Type="Place" U="r" Title="Room"/>
<C Type="Place" U="long" Title="Room Long"/>
<C Type="Future" U="a" Title="Lamp"/><C Type="Future" U="b" Title="Lamp"/></ControlList>"#;
    let saved = xml.to_vec();
    assert!(matches!(
        move_control(xml, Some("Future"), Some("Lamp"), "missing", &[], &now()),
        Err(EditError::RoomNotFound { room }) if room == "missing"
    ));
    assert!(matches!(
        move_control(xml, None, Some("Lamp"), "missing", &[], &now()),
        Err(EditError::Selection(SelectionError::Ambiguous { .. }))
    ));
    assert!(matches!(
        move_control(xml, None, Some("missing"), "missing", &[], &now()),
        Err(EditError::Selection(SelectionError::NotFound { .. }))
    ));
    assert!(matches!(
        move_control(xml, None, None, "missing", &[], &now()),
        Err(EditError::ControlMoveSelectorMissing)
    ));
    assert_eq!(
        move_control(xml, Some(""), None, "Room", &[], &now())
            .unwrap()
            .change
            .room_uuid,
        "r"
    );
    let mirrors = br#"<ControlList><C Type="Place" U="r" Title="Room"/><C Type="Place" U="r" Title="Room"/></ControlList>"#;
    assert!(matches!(
        move_control(mirrors, Some("Future"), None, "Room", &[], &now()),
        Err(EditError::RoomAmbiguous { matches, .. }) if matches == ["Room", "Room"]
    ));
    let error = move_control(b"<C A='1' A='2'/>", None, None, "", &[], &now()).unwrap_err();
    assert!(matches!(error, EditError::Load(_)));
    assert!(error.source().is_some());
    assert_eq!(xml.as_slice(), saved);
}
