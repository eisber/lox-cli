use chrono::{DateTime, Duration, FixedOffset};
use lox_core::configuration::editor::{
    ControlMoveSelector, EditError, SelectionError, move_control, set_param,
};
use std::error::Error;

const DAYTIMER_U: &str = "ABCDEF00-0000-0004-FFFF000000000000";
const MODE_NATIVE: &str = "ABCDEF01-ABCD-0004-15000000ABCDEF00";
const MODE_RFC: &str = "abcdef01-abcd-0004-1500-0000abcdef00";
const MODE_B: &str = "00000000-0000-0005-1500000000000000";
const SELECTED_ENTRY: &str = "\t\t\t\t<Entry To=\"180\" V=\"+005\" Marker=\"remove\"/>\r\n";
const SURVIVING_ENTRY: &str =
    "\t\t\t\t<Entry To=\"180\" Ix=\"+01\" V=\"+004\" NeedActivate=\"TrUe\"/>\r\n";

fn daytimer_xml() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
         <Loxone>\r\n\
         \t<C Type=\"Document\" Date=\"2009-01-02 00:00:00\" DateS=\"86400\" 12hTF=\"true\">\r\n\
         \t\t<C Type=\"Page\">\r\n\
         \t\t\t<C Type=\"DayTimer\" U=\"{DAYTIMER_U}\" Title=\"Schedule\" Analog=\"false\" DefValue=\"+00\" N=\"+02\" Modes=\"{MODE_NATIVE},{MODE_B}\" Unknown=\"KeEp\">\r\n\
         {SURVIVING_ENTRY}\
         \t\t\t\t<Co K=\"AQ\" U=\"wire\">\r\n\
         \t\t\t\t\t<In Input=\"source\"/>\r\n\
         \t\t\t\t</Co>\r\n\
         {SELECTED_ENTRY}\
         \t\t\t\t<Unknown Preserve=\"yes\">\r\n\
         \t\t\t\t\t<Entry To=\"300\"/>\r\n\
         \t\t\t\t</Unknown>\r\n\
         \t\t\t</C>\r\n\
         \t\t</C>\r\n\
         \t\t<C Type=\"Mode\" U=\"{MODE_RFC}\"/>\r\n\
         \t\t<C Type=\"Mode\" U=\"{MODE_B}\"/>\r\n\
         \t</C>\r\n\
         </Loxone>\r\n"
    )
}

fn daytimer_clock() -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339("2009-01-01T10:00:00+02:00").unwrap()
}

#[test]
fn daytimer_removal_shared_identity_cardinality_and_explicit_stamp() {
    use lox_core::configuration::editor::remove_daytimer_entry;

    let input = daytimer_xml();
    let now = daytimer_clock();
    let selector = format!("uuid:{DAYTIMER_U}");
    let first = remove_daytimer_entry(input.as_bytes(), &selector, MODE_RFC, 0, 180, &now).unwrap();
    assert_eq!(first.removal.block_uuid, DAYTIMER_U);
    assert_eq!(first.removal.requested_mode_uuid, MODE_RFC);
    assert_eq!(first.removal.stored_mode_uuid, MODE_NATIVE);
    assert_eq!(first.removal.remaining_entries, 1);
    assert_eq!(first.removal.removed_attributes["V"], "+005");
    assert_eq!(first.removal.removed_attributes["Marker"], "remove");
    assert!(!first.removal.removed_attributes.contains_key("From"));
    let expected = input
        .replacen(SELECTED_ENTRY, "", 1)
        .replacen(" N=\"+02\"", " N=\"1\"", 1)
        .replacen(
            "Date=\"2009-01-02 00:00:00\" DateS=\"86400\"",
            "Date=\"2009-01-02 02:00:01\" DateS=\"86401\"",
            1,
        );
    assert_eq!(first.bytes, expected.as_bytes());

    let last = remove_daytimer_entry(&first.bytes, "Schedule", MODE_B, 0, 180, &now).unwrap();
    assert_eq!(last.removal.remaining_entries, 0);
    assert_eq!(last.removal.removed_attributes["NeedActivate"], "TrUe");
    assert_eq!(
        last.bytes,
        expected
            .replacen(SURVIVING_ENTRY, "", 1)
            .replacen(" N=\"1\"", "", 1)
            .replacen(
                "02:00:01\" DateS=\"86401\"",
                "02:00:02\" DateS=\"86402\"",
                1
            )
            .as_bytes()
    );
}

#[test]
fn daytimer_removal_page_admission_and_typed_atomic_refusals() {
    use lox_core::configuration::editor::{DayTimerRemovalError as Failure, remove_daytimer_entry};

    let input = daytimer_xml();
    let now = daytimer_clock();
    let remove =
        |xml: &str| remove_daytimer_entry(xml.as_bytes(), "Schedule", MODE_RFC, 0, 180, &now);
    let control_rfc = "abcdef00-0000-0004-ffff-000000000000";
    let program = input
        .replace("<C Type=\"Page\">", "<C Type=\"Program\"><C Type=\"Page\">")
        .replace(
            "\t\t</C>\r\n\t\t<C Type=\"Mode\"",
            "\t\t</C></C>\r\n\t\t<C Type=\"Mode\"",
        )
        .replace(DAYTIMER_U, control_rfc);
    assert_eq!(
        remove_daytimer_entry(
            program.as_bytes(),
            &format!("uuid:{control_rfc}"),
            MODE_NATIVE,
            0,
            180,
            &now,
        )
        .unwrap()
        .removal
        .block_uuid,
        control_rfc
    );
    assert!(
        remove(
            &input
                .replace("<Loxone>\r\n", "")
                .replace("</Loxone>\r\n", "")
        )
        .is_ok()
    );
    for outside in [
        input.replace("Type=\"Page\"", "Type=\"Folder\""),
        input.replace("Type=\"Page\"", "Type=\"page\""),
        input.replace("Type=\"Document\"", "Type=\"Folder\""),
    ] {
        assert!(matches!(remove(&outside), Err(Failure::OutsidePage)));
    }

    assert!(matches!(
        remove(&input.replace("N=\"+02\"", "N=\"3\"")),
        Err(Failure::EntryCountMismatch)
    ));
    assert!(matches!(
        remove(&input.replace(
            &format!("{MODE_NATIVE},{MODE_B}"),
            &format!("{MODE_NATIVE},{MODE_RFC}")
        )),
        Err(Failure::DuplicateModes)
    ));
    assert!(matches!(
        remove(&input.replace(SURVIVING_ENTRY, "<Entry From=\"180\" To=\"240\"/>")),
        Err(Failure::TouchingOrOverlappingEntries)
    ));
    let duplicate_mode_object = input.replace(
        &format!("<C Type=\"Mode\" U=\"{MODE_RFC}\"/>"),
        &format!("<C Type=\"Mode\" U=\"{MODE_RFC}\"/><C Type=\"Mode\" U=\"{MODE_NATIVE}\"/>"),
    );
    assert!(matches!(
        remove(&duplicate_mode_object),
        Err(Failure::UnresolvedMode { stored_mode_uuid }) if stored_mode_uuid == MODE_NATIVE
    ));
    assert!(matches!(
        remove(&input.replace("Analog=\"false\"", "Analog=\"true\"")),
        Err(Failure::AnalogSchedule)
    ));
    assert!(matches!(
        remove_daytimer_entry(
            input.as_bytes(),
            "uuid:abcdef00-0000-0004-ffff000000000000",
            MODE_RFC,
            0,
            180,
            &now
        ),
        Err(Failure::Edit(EditError::Selection(
            SelectionError::NotFound { .. }
        )))
    ));
    let failure = remove("<broken").unwrap_err();
    assert!(matches!(&failure, Failure::Edit(EditError::Load(_))));
    assert!(failure.source().and_then(Error::source).is_some());
}

#[test]
fn daytimer_removal_refuses_direct_namespaced_entry_lookalikes_without_output() {
    use lox_core::configuration::editor::{DayTimerRemovalError, remove_daytimer_entry};

    let input = daytimer_xml();
    let last = input
        .replace("N=\"+02\"", "N=\"1\"")
        .replace(SURVIVING_ENTRY, "");
    let now = daytimer_clock();
    let remove =
        |xml: &str| remove_daytimer_entry(xml.as_bytes(), "Schedule", MODE_RFC, 0, 180, &now);
    assert_eq!(remove(&last).unwrap().removal.remaining_entries, 0);

    for foreign in [
        r#"<vendor:Entry xmlns:vendor="urn:vendor" To="180"/>"#,
        r#"<Entry xmlns="urn:vendor" To="180"/>"#,
    ] {
        // The legacy match sees local Entry, count 1, and defaults From/Ix to zero.
        let selected_foreign = last.replace(SELECTED_ENTRY, foreign);
        assert!(matches!(
            remove(&selected_foreign),
            Err(DayTimerRemovalError::NamespacedEntry)
        ));
    }
    let surviving_foreign = input.replace(
        SURVIVING_ENTRY,
        r#"<vendor:Entry xmlns:vendor="urn:vendor" To="180" Ix="1"/>"#,
    );
    assert!(matches!(
        remove(&surviving_foreign),
        Err(DayTimerRemovalError::NamespacedEntry)
    ));
}

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
fn unwire_removes_direct_references_only_and_retains_identity_then_legacy_uuid_wiring() {
    use lox_core::configuration::editor::unwire;
    let input = br#"<C Type="Document" DateS="561020133"><C Type="Future" U="a" Title="Gate.dot" Nio="9" WF="18432"><Co K="I1" U="raw35-not-a-validated-uuid" Nc="2" Other="keep"><In Input="same" FLG="2"/><In Input="same"/><Future><In Input="nested"/></Future></Co><Co K="I1" U="duplicate"/><Co K="Q" U="other"/></C><C U="a" Title="mirror"><Co K="I1" U="mirror"/></C></C>"#;
    let first = unwire(input, "uuid:a", "I1", &now()).unwrap();
    assert_eq!(first.change.removed_references, 2);
    assert_eq!(
        first.change.previous_uuid.as_deref(),
        Some("raw35-not-a-validated-uuid")
    );
    let text = String::from_utf8(first.bytes.clone()).unwrap();
    assert!(text.contains("U=\"raw35-not-a-validated-uuid\" Other=\"keep\""));
    assert!(text.contains("<In Input=\"nested\"/>"));
    assert!(text.contains("<Co K=\"I1\" U=\"duplicate\"/>"));
    assert!(text.contains("<Co K=\"Q\" U=\"other\"/>"));
    assert!(text.contains("<Co K=\"I1\" U=\"mirror\"/>"));
    assert!(text.contains("Nio=\"9\" WF=\"18432\""));
    assert!(text.contains("DateS=\"561020134\""));
    assert!(!text.contains("Nc="));
    let second = unwire(&first.bytes, "uuid:a", "I1", &now()).unwrap();
    assert!(second.content_changed);
    assert_eq!(second.change.removed_references, 0);
    let third = unwire(&second.bytes, "uuid:a", "I1", &now()).unwrap();
    assert!(!third.content_changed);
    assert!(!third.document_stamped);
    assert_eq!(third.bytes, second.bytes);
}

#[test]
fn unwire_normalization_noop_and_explicit_clock_preserve_bom_eol_and_attribute_lf() {
    use lox_core::configuration::editor::unwire;
    for ending in ["\n", "\r\n"] {
        for bom in ["", "\u{feff}"] {
            let input = format!(
                "{bom}<?xml version=\"1.0\" encoding=\"utf-8\"?>{ending}<C Type=\"Document\" DateS=\"561020133\" Note=\"first\nsecond\" 12hTF=\"true\"><C Title=\"Gate\" U=\"a\"><Co K=\"Q\" Nc=\"0\"/></C></C>{ending}"
            );
            let first = unwire(input.as_bytes(), "Gate", "Q", &now()).unwrap();
            assert!(first.content_changed && first.document_stamped);
            assert_eq!(first.change.previous_uuid, None);
            let text = std::str::from_utf8(&first.bytes).unwrap();
            assert_eq!(text.starts_with('\u{feff}'), !bom.is_empty());
            assert!(text.contains(&format!("?>{ending}")));
            assert!(text.contains("Note=\"first\nsecond\" 12hTF=\"true\""));
            let noop = unwire(&first.bytes, "Gate", "Q", &(now() + Duration::days(1))).unwrap();
            assert!(!noop.content_changed && !noop.document_stamped);
            assert_eq!(noop.bytes, first.bytes);
        }
    }
}

#[test]
fn unwire_selection_and_connector_refusals_are_typed_and_atomic() {
    use lox_core::configuration::editor::{UnwireError, unwire};
    let input = br#"<Root><C Type="Place" U="r" Title="Kitchen"/><C U="a" Title="Gate"><Co K="I1" U="wire"/><IoData Pr="r"/></C><C U="b" Title="Gate"><Co K="I1"/></C></Root>"#;
    let before = input.to_vec();
    assert_eq!(
        unwire(input, "gate [KITCHEN]", "I1", &now())
            .unwrap()
            .change
            .element
            .uuid,
        "a"
    );
    assert!(matches!(
        unwire(input, "Gate", "missing", &now()),
        Err(UnwireError::Edit(EditError::Selection(
            SelectionError::Ambiguous { .. }
        )))
    ));
    assert!(matches!(
        unwire(input, "absent", "missing", &now()),
        Err(UnwireError::Edit(EditError::Selection(
            SelectionError::NotFound { .. }
        )))
    ));
    assert!(matches!(
        unwire(input, "uuid:a", "i1", &now()),
        Err(UnwireError::ConnectorMissing { element, connector })
            if element.uuid == "a" && connector == "i1"
    ));
    let error = unwire(b"<C>", "absent", "", &now()).unwrap_err();
    assert!(matches!(error, UnwireError::Edit(EditError::Load(_))));
    assert!(error.source().unwrap().is::<xmltree::ParseError>());
    assert_eq!(input.as_slice(), before);
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
