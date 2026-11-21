use chrono::{DateTime, Duration, FixedOffset};
use lox_core::configuration::editor::{
    ControlMoveSelector, EditError, SelectionError, move_control, set_param,
};
use std::error::Error;

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
fn room_add_explicit_entropy_time_and_direct_insertion_preserve_canonical_collateral() {
    use lox_core::configuration::editor::{RoomAddEntropy, add_room};
    let input = b"\xef\xbb\xbf<ControlList Future=\"+003\">\r\n<C Type=\"Document\" Date=\"old\" DateS=\"1\"/>\r\n<C Type=\"PlaceCaption\" WF=\"\"/>\r\n<C Type=\"Page\"><C Type=\"PlaceCaption\"/><IoData Pr=\"old\"/><Co U=\"wire\"><In Input=\"ref\"/></Co></C>\r\n<Tail Marker=\"keep\"/>\r\n</ControlList>\r\n";
    let calls = std::cell::Cell::new(0);
    let outcome = add_room(
        input,
        "  Room & 室  ",
        || {
            calls.set(calls.get() + 1);
            RoomAddEntropy {
                first: 0x1234abcd,
                second: 0x0001,
                third: 0x00ef,
            }
        },
        &now(),
    )
    .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(outcome.change.name, "  Room & 室  ");
    assert_eq!(outcome.change.uuid, "1234abcd-0001-00ef-ffff000000000000");
    assert!(outcome.content_changed && outcome.document_stamped);
    let expected = format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList Future=\"+003\">\r\n\t<C Type=\"Document\" Date=\"2026-10-12 07:55:33\" DateS=\"{}\"/>\r\n\t<C Type=\"PlaceCaption\" WF=\"\"/>\r\n\t<C Type=\"Place\" V=\"175\" U=\"1234abcd-0001-00ef-ffff000000000000\" Title=\"  Room &amp; 室  \" WF=\"16384\"/>\r\n\t<C Type=\"Page\">\r\n\t\t<C Type=\"PlaceCaption\"/>\r\n\t\t<IoData Pr=\"old\"/>\r\n\t\t<Co U=\"wire\">\r\n\t\t\t<In Input=\"ref\"/>\r\n\t\t</Co>\r\n\t</C>\r\n\t<Tail Marker=\"keep\"/>\r\n</ControlList>\r\n",
        now().timestamp() - 1_230_768_000
    );
    assert_eq!(outcome.bytes, expected.as_bytes());
    let second = add_room(
        &outcome.bytes,
        "Next",
        || RoomAddEntropy {
            first: 0,
            second: 0,
            third: 0,
        },
        &now(),
    )
    .unwrap();
    assert!(String::from_utf8(second.bytes).unwrap().contains(&format!(
        "DateS=\"{}\"",
        now().timestamp() - 1_230_768_000 + 1
    )));
}

#[test]
fn room_add_refusals_precede_entropy_and_retain_legacy_substring_predicate() {
    use lox_core::configuration::editor::{RoomAddEntropy, add_room};
    for (xml, name, duplicate) in [
        (
            r#"<C Type="Place" U="" Title="Kitchen Long"/>"#,
            "KITCHEN",
            true,
        ),
        (
            r#"<R><C Type="Page"><C Type="Place" U="a" Title="Nested"/></C></R>"#,
            "est",
            true,
        ),
        (r#"<R><C Type="Place" U="a" Title=""/></R>"#, "", true),
        (r#"<R><C Type="Place" Title="Alpha"/></R>"#, "alpha", false),
        (r#"<R><C Type="Place" U="a"/></R>"#, "", false),
        (
            r#"<R><C Type="place" U="a" Title="Alpha"/></R>"#,
            "Alpha",
            false,
        ),
        (
            r#"<R><C Type="Place" U="a" Title="Kitchen"/></R>"#,
            "Kitchen Long",
            false,
        ),
        (
            r#"<R><C Type="Place" U="a" Title="Straße"/></R>"#,
            "STRASSE",
            false,
        ),
    ] {
        let calls = std::cell::Cell::new(0);
        let outcome = add_room(
            xml.as_bytes(),
            name,
            || {
                calls.set(calls.get() + 1);
                RoomAddEntropy {
                    first: 0,
                    second: 0,
                    third: 0,
                }
            },
            &now(),
        );
        assert_eq!(calls.get(), usize::from(!duplicate));
        if duplicate {
            let error = outcome.unwrap_err();
            assert!(matches!(&error, EditError::RoomAlreadyExists { room } if room == name));
            assert_eq!(error.to_string(), format!("Room '{name}' already exists"));
        } else {
            assert!(outcome.unwrap().content_changed);
        }
    }
    assert!(matches!(
        add_room(
            b"<",
            "",
            || panic!("load refusal must not request entropy"),
            &now()
        ),
        Err(EditError::Load(_))
    ));
}

#[test]
fn room_add_allows_collisions_empty_names_and_appends_past_only_nested_captions() {
    use lox_core::configuration::editor::{RoomAddEntropy, add_room};
    let xml = br#"<R><C Type="Page"><C Type="PlaceCaption"/><C Type="Place" U="00000000-0000-0000-ffff000000000000"/></C><Tail/></R>"#;
    let outcome = add_room(
        xml,
        "",
        || RoomAddEntropy {
            first: 0,
            second: 0,
            third: 0,
        },
        &now(),
    )
    .unwrap();
    assert!(outcome.content_changed && !outcome.document_stamped);
    let text = String::from_utf8(outcome.bytes).unwrap();
    assert_eq!(
        text.matches("U=\"00000000-0000-0000-ffff000000000000\"")
            .count(),
        2
    );
    assert!(text.contains("\t<Tail></Tail>\n\t<C Type=\"Place\" V=\"175\" U=\"00000000-0000-0000-ffff000000000000\" Title=\"\" WF=\"16384\"/>\n</R>"));
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
