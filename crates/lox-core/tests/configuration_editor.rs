use chrono::{DateTime, Duration, FixedOffset};
use lox_core::configuration::editor::{
    ControlMoveSelector, EditError, SelectionError, move_control, set_param, set_property,
};
use std::error::Error;

fn now() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap()
}

#[test]
fn set_property_keeps_first_set_first_property_and_native_collateral() {
    let before = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<ControlList>\r\n\
\t<C Type=\"GenTActor\" U=\"a\" Title=\"Küche 室\" 12hTF=\"true\" Note=\"first\nsecond\">\r\n\
\t\t<SET>\r\n\
\t\t\t<mqtt_topic Extra=\"+003\" t=\"old\" v=\"first\"><Unknown/></mqtt_topic>\r\n\
\t\t\t<mqtt_topic t=\"11\" v=\"second\"/>\r\n\
\t\t</SET>\r\n\
\t\t<SET><mqtt_topic v=\"last\"/><mqtt_qos v=\"+01\"/></SET>\r\n\
\t\t<Unknown Value=\"&quot; &amp; &lt;\"/>\r\n\
\t</C>\r\n\
</ControlList>\r\n";
    let out = set_property(
        before.as_bytes(),
        "uuid:a",
        "mqtt_topic",
        "雪 & <",
        "-007",
        &now(),
    )
    .unwrap();
    assert_eq!(
        out.bytes,
        before
            .replace("t=\"old\" v=\"first\"", "t=\"-007\" v=\"雪 &amp; &lt;\"")
            .replace(
                "\"><Unknown/></mqtt_topic>",
                "\">\r\n\t\t\t\t<Unknown></Unknown>\r\n\t\t\t</mqtt_topic>",
            )
            .replace("<SET><mqtt_topic v=\"last\"/><mqtt_qos v=\"+01\"/></SET>",
                "<SET>\r\n\t\t\t<mqtt_topic v=\"last\"/>\r\n\t\t\t<mqtt_qos v=\"+01\"/>\r\n\t\t</SET>")
            .as_bytes()
    );
    assert!(!out.change.created);
    assert_eq!(out.change.old_value.as_deref(), Some("first"));
    assert_eq!(out.change.old_type_code.as_deref(), Some("old"));
    #[cfg(feature = "configuration-xml")]
    {
        let topics = lox_core::configuration::mqtt::list_topics(&out.bytes).unwrap();
        assert_eq!(topics[0].topic, "last");
        assert_eq!(topics[0].qos, "+01");
    }
}

#[test]
fn set_property_create_missing_value_noop_and_type_only_stamp_are_distinct() {
    let xml = br#"<C Type="Document" Date="2026-10-12 07:55:33" DateS="561020133"><C Type="And" U="a"><Unknown/><SET><P Other="keep"><Child/></P></SET><SET><P v="later"/></SET></C></C>"#;
    let out = set_property(xml, "uuid:a", "P", "", "+11", &now()).unwrap();
    assert!(!out.change.created);
    assert_eq!(out.change.old_value, None);
    assert!(out.content_changed && out.document_stamped);
    assert!(String::from_utf8_lossy(&out.bytes).contains("DateS=\"561020134\""));
    let noop = set_property(&out.bytes, "uuid:a", "P", "", "+11", &now()).unwrap();
    assert!(!noop.content_changed && !noop.document_stamped);
    assert_eq!(noop.bytes, out.bytes);
    let typed = set_property(&out.bytes, "uuid:a", "P", "", "15", &now()).unwrap();
    assert!(typed.content_changed && typed.document_stamped);
    assert_eq!(typed.change.old_value.as_deref(), Some(""));
    let appended = set_property(xml, "uuid:a", "New", "+0003", "anything", &now()).unwrap();
    assert!(appended.change.created);
    let text = String::from_utf8(appended.bytes).unwrap();
    assert!(text.find("<New ").unwrap() < text.find("v=\"later\"").unwrap());
    let created =
        set_property(br#"<C U="a"><Unknown/></C>"#, "uuid:a", "P", "", "", &now()).unwrap();
    assert!(created.change.created);
    assert_eq!(created.change.element.title, "");
    assert_eq!(created.bytes, b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<C U=\"a\">\n\t<Unknown></Unknown>\n\t<SET>\n\t\t<P t=\"\" v=\"\"/>\n\t</SET>\n</C>\n");
}

#[test]
fn set_property_preserves_load_selection_order_and_raw_name_acceptance() {
    let xml = br#"<ControlList><C Type="And" U="a" Title="Gate"/><C Type="And" U="b" Title="Gate"/></ControlList>"#;
    let saved = xml.to_vec();
    assert!(matches!(
        set_property(xml, "missing", "", "", "", &now()),
        Err(EditError::Selection(SelectionError::NotFound { .. }))
    ));
    assert!(matches!(
        set_property(xml, "Gate", "", "", "", &now()),
        Err(EditError::Selection(SelectionError::Ambiguous { .. }))
    ));
    let error = set_property(b"<C A='1' A='2'/>", "missing", "", "", "", &now()).unwrap_err();
    assert!(matches!(error, EditError::Load(_)));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<xmltree::ParseError>()
            .is_some()
    );
    assert_eq!(xml.as_slice(), saved);
    for property in ["", "a/b", "prefix:name", "12name"] {
        // The emitter's legacy acceptance is intentionally not an XML-name validator.
        let out = set_property(xml, "uuid:a", property, "+003", "-1", &now()).unwrap();
        assert_eq!(out.change.property, property);
    }
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
