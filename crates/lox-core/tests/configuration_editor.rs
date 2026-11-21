use chrono::{DateTime, Duration, FixedOffset};
use lox_core::configuration::editor::{
    ControlMoveSelector, EditError, SelectionError, move_control, set_param,
};
use std::error::Error;

mod central_light_links {
    use chrono::{DateTime, FixedOffset};
    use lox_core::configuration::editor::{
        CentralLightLink,
        CentralLightLinkDirection::{Down, Up},
        CentralLightLinkMove, CentralLightLinkMoveError as MoveRefusal, CentralLightLinkRemoval,
        CentralLightLinkRemovalError as RemovalRefusal, CentralLightLinks,
        CentralLightLinksError as Refusal, EditError, SelectionError,
        inspect_central_light_links as inspect, move_central_light_link as move_link,
        remove_central_light_link as remove_link,
    };
    use std::error::Error;

    const OWNER: &str = "ABCDEF01-0002-0003-0000000000000001";
    const FIRST: &str = "ABCDEF01-0002-0003-000000000000000A";
    const SECOND: &str = "abcdef01-0002-0003-000000000000000b";
    const MISSING: &str = "abcdef01-0002-0003-000000000000000c";
    const THIRD: &str = "abCDef01-0002-0003-000000000000000d";
    const HIDDEN: &str = "abcdef01-0002-0003-000000000000000e";

    fn document() -> String {
        let first = FIRST.to_ascii_lowercase();
        format!(
            r#"<ControlList><C Type="Document">
<C Type="Place" U="den" Title="Den"/>
<C Type="CentralLight" U="{OWNER}" Title="Central" gid="primary" linkC="{FIRST},{SECOND}">
  <IoData Visu="true" Pr="den"/>
</C>
<C Type="Not" U="{SECOND}" Title="Sensor &amp; status" WF="268451840"/>
<Wrapper><C U="{first}" WF="268435456"/></Wrapper>
<C Type="Other" U="unrelated"/>
</C></ControlList>"#
        )
    }

    #[test]
    fn reads_owned_stored_order_and_defaults_with_legacy_selectors() {
        let expected = CentralLightLinks {
            uuid: OWNER.into(),
            title: "Central".into(),
            links: vec![
                CentralLightLink {
                    position: 1,
                    uuid: FIRST.into(),
                    title: String::new(),
                    control_type: String::new(),
                },
                CentralLightLink {
                    position: 2,
                    uuid: SECOND.into(),
                    title: "Sensor & status".into(),
                    control_type: "Not".into(),
                },
            ],
        };
        for selector in [
            "cent",
            "Central [dEn]",
            "gid:PRIMARY",
            &format!("uuid:{OWNER}"),
        ] {
            let actual = inspect(document().as_bytes(), selector).unwrap();
            assert_eq!(actual, expected);
        }

        let empty = format!(
            r#"<C Type="document"><C Type="CentralLight" U="{OWNER}"><IoData Visu="true"/></C></C>"#
        );
        let empty = inspect(empty.as_bytes(), &format!("uuid:{OWNER}")).unwrap();
        assert!(empty.title.is_empty() && empty.links.is_empty());
    }

    #[test]
    fn selection_and_load_errors_precede_admission_and_keep_causes() {
        let xml = document().replace(
            r#"<C Type="Other" U="unrelated"/>"#,
            r#"<C Type="Other" U="unrelated" Title="Central"/>"#,
        );
        let ambiguous = inspect(xml.as_bytes(), "Central").unwrap_err();
        assert!(matches!(
            ambiguous.source().and_then(|cause| cause.downcast_ref::<EditError>()),
            Some(EditError::Selection(SelectionError::Ambiguous { matches, .. }))
                if matches.len() == 2 && matches[1].block_type == "Other"
        ));
        let missing = inspect(
            xml.as_bytes(),
            &format!("uuid:{}", OWNER.to_ascii_lowercase()),
        );
        assert!(matches!(
            missing,
            Err(Refusal::Edit(EditError::Selection(
                SelectionError::NotFound { .. }
            )))
        ));
        let lower_type = xml.replace(r#"Type="CentralLight""#, r#"Type="centrallight""#);
        assert!(matches!(
            inspect(lower_type.as_bytes(), "Type:CentralLight"),
            Err(Refusal::NotPlainCentralLight { selector }) if selector == "Type:CentralLight"
        ));
        let malformed = inspect(b"<C A='1' A='2'/>", "Central").unwrap_err();
        assert!(matches!(malformed, Refusal::Edit(EditError::Load(_))));
        assert!(malformed.source().and_then(Error::source).is_some());
    }

    #[test]
    fn document_and_direct_owner_admission_are_not_fallbacks() {
        let inspect_xml = |xml: String| inspect(xml.as_bytes(), "Central").unwrap_err();
        let xml = document();
        assert!(matches!(
            inspect_xml(xml.replace("</ControlList>", r#"<C Type="Document"/></ControlList>"#)),
            Refusal::NoSingleOwningDocument
        ));
        assert!(matches!(
            inspect_xml(
                xml.replace(
                    r#"<C Type="Document">"#,
                    r#"<x:C Type="Document" xmlns:x="urn:test">"#
                )
                .replace("</C></ControlList>", "</x:C></ControlList>")
            ),
            Refusal::NotPlainDocument
        ));
        assert!(matches!(
            inspect_xml(xml.replace(r#"Type="Document">"#, r#"Type="Document"><Template/>"#)),
            Refusal::DocumentTemplate
        ));
        assert!(matches!(
            inspect_xml(xml.replace(
                r#"Type="CentralLight""#,
                r#"Type="CentralLight" xmlns="urn:test""#
            )),
            Refusal::NotPlainCentralLight { .. }
        ));
        for io in [
            r#"<IoData Visu="True"/>"#,
            r#"<IoData xmlns="urn:test" Visu="true"/>"#,
            r#"<Nested><IoData Visu="true"/></Nested>"#,
            r#"<IoData Visu="true"/><IoData Visu="true"/>"#,
        ] {
            assert!(matches!(
                inspect_xml(xml.replace(r#"<IoData Visu="true" Pr="den"/>"#, io)),
                Refusal::NotDirectVisibleOwner
            ));
        }
        assert!(matches!(
            inspect_xml(xml.replace(&format!(r#" U="{OWNER}""#), "")),
            Refusal::MissingOwnerIdentity
        ));
    }

    #[test]
    fn validates_document_wide_link_graph_and_wf_without_repair() {
        let inspect_xml = |xml: String| inspect(xml.as_bytes(), "Central").unwrap_err();
        let xml = document();
        let first = FIRST.to_ascii_lowercase();
        for invalid in [
            format!(" {SECOND}"),
            format!("uuid:{SECOND}"),
            "00000000-0000-0000-0000000000000000".into(),
        ] {
            assert!(matches!(
                inspect_xml(xml.replacen(SECOND, &invalid, 1)),
                Refusal::InvalidIdentity { uuid } if uuid == invalid
            ));
        }
        assert!(matches!(
            inspect_xml(xml.replacen(SECOND, &first, 1)),
            Refusal::DuplicateReference { uuid } if uuid == first
        ));
        for other in [
            format!(
                r#"<C Type="Other" linkC="{}"/>"#,
                MISSING.to_ascii_uppercase()
            ),
            format!(r#"<C Type="Other" U="{FIRST}"/>"#),
        ] {
            let expected = if other.contains("linkC=") {
                MISSING
            } else {
                &first
            };
            assert!(matches!(
                inspect_xml(xml.replace(r#"<C Type="Other" U="unrelated"/>"#, &other)),
                Refusal::UnresolvedControl { uuid } if uuid == expected
            ));
        }
        let bad_wf = inspect_xml(xml.replace("268451840", "-1"));
        assert!(matches!(&bad_wf, Refusal::InvalidFlags { value, .. } if value == "-1"));
        assert!(
            bad_wf
                .source()
                .and_then(|cause| cause.downcast_ref::<std::num::ParseIntError>())
                .is_some()
        );
        assert!(matches!(
            inspect_xml(xml.replace("268451840", "16384")),
            Refusal::InconsistentMarker { identity: Some(uuid) } if uuid == SECOND
        ));
    }

    fn move_document() -> String {
        let first = FIRST.to_ascii_lowercase();
        let third = THIRD.to_ascii_uppercase();
        format!(
            "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<ControlList>\r\n\
\t<C Type=\"Document\" Date=\"2009-01-01 01:00:02\" DateS=\"2\">\r\n\
\t\t<C Type=\"Place\" U=\"den\" Title=\"Den\"/>\r\n\
\t\t<C Type=\"CentralLight\" U=\"{OWNER}\" Title=\"Central\" linkC=\"{FIRST},{SECOND},{THIRD}\" Future=\"keep\">\r\n\
\t\t\t<IoData Visu=\"true\" Pr=\"den\" Rating=\"7\"/>\r\n\
\t\t\t<Unknown Key=\"first\"/>\r\n\
\t\t\t<Co K=\"I\">\r\n\
\t\t\t\t<In Input=\"opaque\"/>\r\n\
\t\t\t</Co>\r\n\
\t\t\t<Unknown Key=\"last\"/>\r\n\
\t\t</C>\r\n\
\t\t<C Type=\"ModbusServer\" U=\"{first}\" Title=\"A\" WF=\"268451840\"/>\r\n\
\t\t<C Type=\"ModbusServer\" U=\"{SECOND}\" Title=\"B\" WF=\"268435456\"/>\r\n\
\t\t<C Type=\"ModbusServer\" U=\"{third}\" Title=\"C\" WF=\"268435456\"/>\r\n\
\t\t<C Type=\"Other\" linkC=\"{first},{HIDDEN}\" iObjS=\"unrelated\"/>\r\n\
\t\t<C Type=\"Not\" U=\"{HIDDEN}\" Title=\"Hidden\" WF=\"268435456\"/>\r\n\
\t</C>\r\n\
</ControlList>\r\n"
        )
    }

    fn move_now() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2009-01-01T01:00:01+01:00").unwrap()
    }

    #[test]
    fn moves_in_both_directions_preserving_stored_spelling_and_collateral_exactly() {
        let source = move_document();
        let up = move_link(
            source.as_bytes(),
            "Central [dEn]",
            &SECOND.to_ascii_uppercase(),
            Up,
            &move_now(),
        )
        .unwrap();
        assert_eq!(
            up.movement,
            CentralLightLinkMove {
                uuid: SECOND.into(),
                from_position: 2,
                to_position: 1,
                changed: true,
            }
        );
        assert!(up.document_stamped);
        assert_eq!(up.inspection, inspect(&up.bytes, "Central").unwrap());
        assert_eq!(up.inspection.links[0].uuid, SECOND);
        assert_eq!(up.inspection.links[1].uuid, FIRST);
        assert_eq!(up.inspection.links[2].uuid, THIRD);
        let expected = source
            .replace(
                &format!("{FIRST},{SECOND},{THIRD}"),
                &format!("{SECOND},{FIRST},{THIRD}"),
            )
            .replace(r#"01:00:02" DateS="2""#, r#"01:00:03" DateS="3""#);
        assert_eq!(up.bytes, expected.as_bytes());
        let down = move_link(
            &up.bytes,
            &format!("uuid:{OWNER}"),
            SECOND,
            Down,
            &move_now(),
        )
        .unwrap();
        assert_eq!(
            (down.movement.from_position, down.movement.to_position),
            (1, 2)
        );
        assert!(down.movement.changed && down.document_stamped);
        assert_eq!(
            down.bytes,
            source
                .replace(r#"01:00:02" DateS="2""#, r#"01:00:04" DateS="4""#)
                .as_bytes()
        );
    }

    #[test]
    fn move_boundaries_keep_noncanonical_original_bytes_and_do_not_stamp() {
        let source = move_document().replace(r#"Future="keep">"#, r#"Future="keep" >"#);
        let source = source.trim_end().as_bytes();
        for (uuid, direction, position) in [(FIRST, Up, 1), (THIRD, Down, 3)] {
            let result = move_link(source, "Central", uuid, direction, &move_now()).unwrap();
            assert_eq!(result.bytes, source);
            assert_eq!(result.movement.from_position, position);
            assert_eq!(result.movement.to_position, position);
            assert!(!result.movement.changed && !result.document_stamped);
            assert_eq!(result.inspection, inspect(source, "Central").unwrap());
        }
    }

    #[test]
    fn move_preserves_changed_but_unstamped_legacy_document_branch() {
        let source = move_document().replace(r#"Type="Document""#, r#"Type="document""#);
        let moved = move_link(source.as_bytes(), "Central", SECOND, Up, &move_now()).unwrap();
        assert!(moved.movement.changed && !moved.document_stamped);
        assert_eq!(
            moved.bytes,
            source
                .replace(
                    &format!("{FIRST},{SECOND},{THIRD}"),
                    &format!("{SECOND},{FIRST},{THIRD}"),
                )
                .as_bytes()
        );
    }

    #[test]
    fn move_refusals_keep_legacy_validation_order_and_typed_causes() {
        let source = move_document();
        let move_up = |source: &str, owner, uuid| {
            move_link(source.as_bytes(), owner, uuid, Up, &move_now()).unwrap_err()
        };
        assert!(matches!(
            move_up("not XML", "No such owner", "1"),
            MoveRefusal::Edit(EditError::Load(_))
        ));
        let prefixed = format!("uuid:{FIRST}");
        assert!(matches!(
            move_up(&source, "No such owner", &prefixed),
            MoveRefusal::Links(Refusal::InvalidIdentity { uuid }) if uuid == prefixed
        ));
        assert!(matches!(
            move_up(&source, "No such owner", FIRST),
            MoveRefusal::Links(Refusal::Edit(EditError::Selection(
                SelectionError::NotFound { .. }
            )))
        ));
        let missing = MISSING.to_ascii_uppercase();
        assert!(matches!(
            move_up(&source, "Central", &missing),
            MoveRefusal::NotLinked { uuid } if uuid == missing
        ));
        for unproved in [
            source.replacen(r#"Title="A""#, r#"Title="""#, 1),
            source.replacen(r#"Type="ModbusServer""#, r#"Type="Not""#, 1),
        ] {
            assert!(matches!(
                move_up(&unproved, "Central", &missing),
                MoveRefusal::UnsupportedTarget { uuid } if uuid == FIRST
            ));
            assert!(matches!(
                move_link(unproved.as_bytes(), "Central", THIRD, Down, &move_now()),
                Err(MoveRefusal::UnsupportedTarget { uuid }) if uuid == FIRST
            ));
        }
        let graph_error = move_up(&source.replace("268451840", "16384"), "Central", &missing);
        assert!(matches!(
            graph_error.source().and_then(|cause| cause.downcast_ref::<Refusal>()),
            Some(Refusal::InconsistentMarker { identity: Some(uuid) })
                if uuid == &FIRST.to_ascii_lowercase()
        ));
    }

    fn removal_document() -> String {
        move_document().replace("<C Type=", r#"<C V="178" Type="#)
    }

    #[test]
    fn removal_preserves_other_incoming_links_and_only_changes_the_selected_marker() {
        let source = removal_document()
            .replace("WF=\"268451840\"", "WF=\"+2415935488\"")
            .replace(
                "Title=\"B\" WF=\"268435456\"",
                "Title=\"B\" WF=\"318783488\"",
            );
        let first = remove_link(
            source.as_bytes(),
            "Central [dEn]",
            &FIRST.to_ascii_lowercase(),
            &move_now(),
        )
        .unwrap();
        assert_eq!(
            first.removal,
            CentralLightLinkRemoval {
                uuid: FIRST.into(),
                from_position: 1,
                remaining_links: 2,
                reference_marker_cleared: false,
            }
        );
        assert!(first.document_stamped);
        assert_eq!(first.inspection, inspect(&first.bytes, "Central").unwrap());
        assert_eq!(first.inspection.links[0].uuid, SECOND);
        assert_eq!(first.inspection.links[1].uuid, THIRD);
        let expected = source
            .replace(
                &format!("{FIRST},{SECOND},{THIRD}"),
                &format!("{SECOND},{THIRD}"),
            )
            .replace(r#"01:00:02" DateS="2""#, r#"01:00:03" DateS="3""#);
        assert_eq!(first.bytes, expected.as_bytes());

        let second = remove_link(&first.bytes, "Central", SECOND, &move_now()).unwrap();
        assert!(second.removal.reference_marker_cleared && second.document_stamped);
        assert_eq!(
            second.bytes,
            expected
                .replace(&format!("{SECOND},{THIRD}"), THIRD)
                .replace("WF=\"318783488\"", "WF=\"50348032\"")
                .replace(r#"01:00:03" DateS="3""#, r#"01:00:04" DateS="4""#)
                .as_bytes()
        );
    }

    #[test]
    fn removal_of_last_entry_omits_empty_values_and_can_be_changed_but_unstamped() {
        let source = removal_document()
            .replace(r#"Type="Document""#, r#"Type="document""#)
            .replace(&format!("{FIRST},{SECOND},{THIRD}"), THIRD)
            .replace("Title=\"B\" WF=\"268435456\"", "Title=\"B\"");
        let removed = remove_link(source.as_bytes(), "Central", THIRD, &move_now()).unwrap();
        assert!(!removed.document_stamped && removed.removal.reference_marker_cleared);
        assert_eq!(removed.removal.remaining_links, 0);
        assert!(removed.inspection.links.is_empty());
        assert_eq!(
            removed.bytes,
            source
                .replace(&format!(r#" linkC="{THIRD}""#), "")
                .replace("Title=\"C\" WF=\"268435456\"", "Title=\"C\"")
                .as_bytes()
        );
        assert_eq!(
            removed.inspection,
            inspect(&removed.bytes, "Central").unwrap()
        );
        assert!(matches!(
            remove_link(&removed.bytes, "Central", THIRD, &move_now()),
            Err(RemovalRefusal::NotLinked { uuid }) if uuid == THIRD
        ));
    }

    #[test]
    fn removal_refusal_order_includes_foreign_identity_versions_and_affected_wf() {
        let remove = |source: &str, owner, uuid| {
            remove_link(source.as_bytes(), owner, uuid, &move_now()).unwrap_err()
        };
        assert!(matches!(
            remove("not XML", "missing", "1"),
            RemovalRefusal::Edit(EditError::Load(_))
        ));
        let source = removal_document();
        let invalid = format!("uuid:{SECOND}");
        assert!(matches!(
            remove(&source, "missing", &invalid),
            RemovalRefusal::Links(Refusal::InvalidIdentity { uuid }) if uuid == invalid
        ));
        let old = source.replace(r#"<C V="178" Type="Other""#, r#"<C V="89" Type="Other""#);
        assert!(matches!(
            remove(
                &old.replace(HIDDEN, &format!(" {HIDDEN}")),
                "Central",
                MISSING
            ),
            RemovalRefusal::Links(Refusal::InvalidIdentity { .. })
        ));
        assert!(matches!(
            remove(
                &old.replace(r#"Title="A""#, r#"Title="""#),
                "Central",
                MISSING
            ),
            RemovalRefusal::UnsupportedTarget { uuid } if uuid == FIRST
        ));
        assert!(matches!(
            remove(&old, "Central", MISSING),
            RemovalRefusal::NotLinked { uuid } if uuid == MISSING
        ));
        let high_wf = old.replace(
            "Title=\"B\" WF=\"268435456\"",
            "Title=\"B\" WF=\"2415919104\"",
        );
        assert!(matches!(
            remove(&high_wf, "Central", SECOND),
            RemovalRefusal::UnsupportedVersion { identity: None, version: Some(version) }
                if version == "89"
        ));
        assert!(matches!(
            remove(
                &high_wf.replace(r#"V="89""#, r#"V="178""#),
                "Central",
                SECOND
            ),
            RemovalRefusal::AffectedFlagsOutOfRange { value: 2415919104 }
        ));
        let shadowed = source.replacen(
            "<ControlList>",
            &format!(r#"<ControlList><C U="{SECOND}"/>"#),
            1,
        );
        assert!(matches!(
            remove(&shadowed, "Central", SECOND),
            RemovalRefusal::Links(Refusal::InconsistentMarker { identity: Some(uuid) })
                if uuid == SECOND
        ));
    }
}

#[test]
fn principal_permissions_are_direct_ordered_and_raw() {
    use lox_core::configuration::editor::{DirectPermission, PrincipalKind, principal_permissions};

    let xml = r#"<Root>
<NotC Type="User" Title="Sämple" U="wrong-tag"/>
<C Type="user" Title="Sämple" U="wrong-type"/>
<NotC U="Object" Title="Not metadata"/>
<C Type="Place" U="Object" Title="First"/>
<C Type="VirtualIn" U="Object" Title="Later"/>
<C U="" Title="Empty reference"/>
<Folder><C Type="User" Title="Sämple" U="User-01">
    <Wrapper><PH><PRM REF="nested-block"/></PH></Wrapper>
    <PH NUM="99">
        <PRM REF="Object" ACT="0" TIM="slot 12hTF=alpha"/>
        <Wrapper><PRM REF="nested-entry"/></Wrapper>
        <PRM REF="OBJECT" ACT="" TIM=""/>
        <PRM/>
        <PRM REF=""/>
        <PRM REF="Object"/>
    </PH>
    <PH><PRM REF="ignored-second-block"/></PH>
</C></Folder>
<C Type="UserGroup" Title="Sämple" U="Group-01"><PH><PRM REF="group-only"/></PH></C>
<C Type="User" U="no-title"/>
<C Type="User" Title="No ID"/>
</Root>"#;
    let read = principal_permissions(xml.as_bytes(), "SÄMPLE", PrincipalKind::User).unwrap();
    assert_eq!(read.principal.title, "Sämple");
    assert_eq!(read.principal.uuid, "User-01");
    assert_eq!(
        read.permissions,
        [
            DirectPermission {
                reference: Some("Object".into()),
                title: Some("First".into()),
                element_type: Some("Place".into()),
                act: Some("0".into()),
                tim: Some("slot _12hTF=alpha".into()),
            },
            DirectPermission {
                reference: Some("OBJECT".into()),
                title: None,
                element_type: None,
                act: Some(String::new()),
                tim: Some(String::new()),
            },
            DirectPermission {
                reference: None,
                title: Some("Empty reference".into()),
                element_type: None,
                act: None,
                tim: None,
            },
            DirectPermission {
                reference: Some(String::new()),
                title: Some("Empty reference".into()),
                element_type: None,
                act: None,
                tim: None,
            },
            DirectPermission {
                reference: Some("Object".into()),
                title: Some("First".into()),
                element_type: Some("Place".into()),
                act: None,
                tim: None,
            },
        ]
    );
    assert_eq!(
        principal_permissions(xml.as_bytes(), "uuid:User-01", PrincipalKind::User).unwrap(),
        read
    );
    let group = principal_permissions(xml.as_bytes(), "sämple", PrincipalKind::UserGroup).unwrap();
    assert_eq!(group.principal.uuid, "Group-01");
    assert_eq!(group.permissions.len(), 1);
    assert_eq!(
        group.permissions[0].reference.as_deref(),
        Some("group-only")
    );
    let no_title =
        principal_permissions(xml.as_bytes(), "uuid:no-title", PrincipalKind::User).unwrap();
    assert!(no_title.principal.title.is_empty() && no_title.permissions.is_empty());
    let no_id = principal_permissions(xml.as_bytes(), "No ID", PrincipalKind::User).unwrap();
    assert!(no_id.principal.uuid.is_empty());
}

#[test]
fn principal_permissions_keep_typed_refusals_and_parser_cause() {
    use lox_core::configuration::editor::{
        PrincipalKind, PrincipalPermissionsError, PrincipalSelectionError, principal_permissions,
    };

    let xml = br#"<Root>
<C Type="UserGroup" Title="Group only" U="GROUP"/>
<C Type="user" Title="Wrong kind" U="wrong-case"/>
<C Type="User" Title="Sample" U="Sample-id"/>
<C Type="User" Title="SAMPLE"/>
<C Type="User" Title="Sample" U="Sample-id"/>
<C Type="User" Title="" U="no-name"/>
</Root>"#;
    for query in ["Sam", "uuid:sample-id", "UUID:Sample-id", "Group only"] {
        assert!(matches!(
            principal_permissions(xml, query, PrincipalKind::User),
            Err(PrincipalPermissionsError::Selection(PrincipalSelectionError::NotFound {
                kind: PrincipalKind::User, selector, titles,
            })) if selector == query && titles.iter().map(String::as_str).eq(["", "SAMPLE", "Sample"])
        ));
    }
    for (query, expected) in [
        ("sample", &["Sample-id", "", "Sample-id"][..]),
        ("uuid:Sample-id", &["Sample-id", "Sample-id"][..]),
    ] {
        let error = principal_permissions(xml, query, PrincipalKind::User).unwrap_err();
        assert!(matches!(
            error.source().and_then(|cause| cause.downcast_ref::<PrincipalSelectionError>()),
            Some(PrincipalSelectionError::Ambiguous {
                kind: PrincipalKind::User, selector, uuids,
            }) if selector == query && uuids.iter().map(String::as_str).eq(expected.iter().copied())
        ));
    }
    let error = principal_permissions(
        b"<C Type='User' Type='UserGroup'/>",
        "missing",
        PrincipalKind::User,
    )
    .unwrap_err();
    let PrincipalPermissionsError::Load(EditError::Load(cause)) = error else {
        panic!("load must precede principal selection");
    };
    assert!(cause.into_inner().downcast::<xmltree::ParseError>().is_ok());
}

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
fn mood_inventory_keeps_recursive_order_duplicates_and_legacy_field_defaults() {
    use lox_core::configuration::editor::{MoodInfo, list_moods};

    let xml = br#"<Root xmlns:ext="urn:future">
      <C Type="Place" U="r" Title="Kitchen"/>
      <C Type="LightController2" U="lc" Title="Kitchen lamps" gid="light">
        <IoData Pr="r"/>
        <Unknown>
          <LightsceneC Name="Outer" UUID="same" SID="+002" CID="-3" Q1="00037">
            <LightsceneC Name="Inner" UUID="same" SID="2" CID=" 3" Q1=""/>
          </LightsceneC>
        </Unknown>
        <LightscenesC Num="99">
          <LightsceneC SID="9223372036854775808" CID="future" Q2="ignored"/>
          <ext:LightsceneC Name="" UUID="" SID="" Q1="opaque&amp;unparsed"/>
          <lightscenec Name="wrong spelling"/>
        </LightscenesC>
      </C>
      <C Type="LightController2" U="lc" Title="Kitchen lamps">
        <LightsceneC Name="mirror"/>
      </C>
      <C Type="LightController2" U="empty"/>
    </Root>"#;
    let expected = vec![
        MoodInfo {
            name: "Outer".into(),
            uuid: "same".into(),
            sid: Some(2),
            cid: Some(-3),
            q1: Some("00037".into()),
        },
        MoodInfo {
            name: "Inner".into(),
            uuid: "same".into(),
            sid: Some(2),
            cid: None,
            q1: Some(String::new()),
        },
        MoodInfo {
            name: String::new(),
            uuid: String::new(),
            sid: None,
            cid: None,
            q1: None,
        },
        MoodInfo {
            name: String::new(),
            uuid: String::new(),
            sid: None,
            cid: None,
            q1: Some("opaque&unparsed".into()),
        },
    ];
    for selector in ["uuid:lc", "gid:LIGHT", "Lamps [kit]"] {
        assert_eq!(list_moods(xml, selector).unwrap(), expected);
    }
    assert!(list_moods(xml, "uuid:empty").unwrap().is_empty());
}

#[test]
fn mood_inventory_retains_load_and_selection_causes_before_exact_type_refusal() {
    use lox_core::configuration::editor::{MoodInventoryError, list_moods};

    let xml = br#"<Root>
      <C Type="And" U="a" Title="Lamps"/>
      <C Type="LightController2" U="b" Title="Lamps"/>
      <C Type="lightcontroller2" U="case" Title="Different title"/>
      <C U="sparse"/>
    </Root>"#;
    let ambiguous = list_moods(xml, "Lamps").unwrap_err();
    assert!(matches!(
        ambiguous
            .source()
            .and_then(|cause| cause.downcast_ref::<EditError>())
            .and_then(Error::source)
            .and_then(|cause| cause.downcast_ref::<SelectionError>()),
        Some(SelectionError::Ambiguous { matches, .. })
            if matches.len() == 2 && matches[0].block_type == "And"
    ));
    assert!(matches!(
        list_moods(xml, "uuid:CASE"),
        Err(MoodInventoryError::Edit(EditError::Selection(
            SelectionError::NotFound { selector, .. }
        ))) if selector == "uuid:CASE"
    ));
    assert!(matches!(
        list_moods(xml, "uuid:case"),
        Err(MoodInventoryError::NotLightController { selector, block_type })
            if selector == "uuid:case" && block_type == "lightcontroller2"
    ));
    assert!(matches!(
        list_moods(xml, "uuid:sparse"),
        Err(MoodInventoryError::NotLightController { block_type, .. }) if block_type.is_empty()
    ));

    let load = list_moods(b"<C A='1' A='2'/>", "Lamps").unwrap_err();
    assert!(matches!(
        load.source()
            .and_then(|cause| cause.downcast_ref::<EditError>()),
        Some(EditError::Load(_))
    ));
    assert!(
        load.source()
            .and_then(Error::source)
            .is_some_and(|cause| cause.is::<xmltree::ParseError>())
    );
}

mod lighting_inspection {
    use super::{EditError, SelectionError};
    use lox_core::configuration::editor::{LightingInspectionError, inspect_lighting, list_moods};

    const XML: &[u8] = br#"<ControlList Version="+0267">
  <C Type="Document" ConfigVersion="17.2" V="opaque">
    <C Type="Place" U="den" Title="Study"/>
    <C Type="LightController2" U="light" gid="primary" Title="Lamps" CapAI0="+0007">
      <Co K="AQ10" U="src" Future="keep &amp; stay"/>
      <Co K="AQ2" U=""/>
      <Co K="AQ01" U="ignored"/>
      <IoData Pr="den"/>
      <LightscenesC Num="99" Outputs="0002">
        <LightsceneC UUID="same" Name="Direct" SID="bad" CID="+09" Q10="opaque" Q2="+0006" Q1="" Q01="unprojected">
          <SeqConf Flag="0"><LightsceneC UUID="same" Name="Nested" Q1="-1"/></SeqConf>
        </LightsceneC>
      </LightscenesC>
      <HCL T="2"/>
    </C>
    <C Type="And" U="wrong" Title="Lamps">
      <Co K="I1" U="in"><In Input="src" FLG="02"/><In Input="SRC"/></Co>
      <In Input="src" Future="unresolved"/>
    </C>
    <C Type="Other" U="src"/>
  </C>
</ControlList>"#;

    #[test]
    fn projects_raw_controller_data_and_reference_observations() {
        let inspection = inspect_lighting(XML, "Lamps [study]").unwrap();
        assert_eq!(inspection, inspect_lighting(XML, "gid:PRIMARY").unwrap());
        assert_eq!(inspection.uuid.as_deref(), Some("light"));
        assert_eq!(inspection.attributes["CapAI0"], "+0007");
        assert_eq!(inspection.document_versions[0]["Version"], "+0267");
        assert_eq!(inspection.document_versions[1]["ConfigVersion"], "17.2");
        assert_eq!(inspection.direct_scene_collection_count, 1);
        let collection = &inspection.nested_storage[inspection.sole_scene_collection.unwrap()];
        assert_eq!(collection.attributes["Outputs"], "0002");
        assert_eq!(collection.children[0].children[0].attributes["Flag"], "0");
        assert_eq!(inspection.nested_storage[2].element, "HCL");

        assert_eq!(
            inspection
                .moods
                .iter()
                .map(|mood| mood.summary.clone())
                .collect::<Vec<_>>(),
            list_moods(XML, "uuid:light").unwrap()
        );
        let [direct, nested] = inspection.moods.as_slice() else {
            panic!("expected direct and nested mood records");
        };
        assert!(direct.in_sole_direct_collection && !nested.in_sole_direct_collection);
        assert_eq!(direct.attributes["SID"], "bad");
        assert_eq!(direct.attributes["Q01"], "unprojected");
        assert_eq!(
            direct
                .outputs
                .iter()
                .map(|output| (output.index, output.stored.as_str()))
                .collect::<Vec<_>>(),
            [(10, "opaque"), (2, "+0006"), (1, "")]
        );

        let [connected, missing] = inspection.channels.as_slice() else {
            panic!("expected two indexed channels");
        };
        assert_eq!(connected.key, "AQ10");
        assert_eq!(connected.attributes["Future"], "keep & stay");
        assert_eq!(connected.uuid_occurrences, Some(2));
        let [target, unresolved] = connected.downstream.as_slice() else {
            panic!("expected two exact references");
        };
        assert_eq!(target.connector_uuid.as_deref(), Some("in"));
        assert_eq!(target.wire_attributes["FLG"], "02");
        assert_eq!(unresolved.connector_uuid, None);
        assert_eq!(unresolved.wire_attributes["Future"], "unresolved");
        assert_eq!(missing.key, "AQ2");
        assert_eq!(missing.uuid_occurrences, None);
    }

    #[cfg(feature = "configuration-xml")]
    mod lighting_mood_count {
        use chrono::{DateTime, FixedOffset};
        use lox_core::configuration::editor::{
            EditError, LightingMoodCountChange, LightingMoodCountError as Refusal, SelectionError,
            recount_lighting_moods as recount,
        };

        const COLLECTION: &str = r#"<LightscenesC Num="future" Outputs="0003" FC="opaque"><!-- <x:LightsceneC Num='91'/> --><LightsceneC UUID="first" SID="+02" CID="09" Q1="00037" Q10="opaque" Outputs="2"><Empty/></LightsceneC><Opaque><![CDATA[<LightsceneC xml:Name='lookalike'/>]]><LightsceneC Num="91"/></Opaque><LightsceneC/></LightscenesC>"#;

        fn document() -> String {
            format!(
                "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
                 <ControlList><C Type=\"Document\" Date=\"old\" DateS=\"86400\" NumO=\"99\">\
                 <C Type=\"Place\" U=\"room\" Title=\"Office\"/><C Type=\"Program\"><C Type=\"Page\">\
                 <C Type=\"LightController2\" U=\"lamp\" Title=\"Lamps\"><IoData Pr=\"room\"/>\
                 {COLLECTION}<LSConfig OpM=\"opaque\"/><HCL T=\"2\"/></C>\
                 <C Type=\"LightController2\" U=\"other\" Title=\"Lamps\"><LightscenesC Num=\"70\"/></C>\
                 </C></C></C></ControlList>\r\n"
            )
        }

        fn clock() -> DateTime<FixedOffset> {
            DateTime::parse_from_rfc3339("2009-01-02T01:59:59+02:00").unwrap()
        }

        #[test]
        fn repairs_only_the_direct_stored_count_and_stamps_owning_document() {
            let xml = document();
            let result = recount(xml.as_bytes(), "Lamps [Off]", &clock()).unwrap();
            assert_eq!(
                result.change,
                LightingMoodCountChange {
                    before: Some("future".into()),
                    before_effective: None,
                    after: "2".into(),
                    scenes: 2,
                    changed: true,
                }
            );
            assert!(result.document_stamped);
            let saved = std::str::from_utf8(&result.bytes).unwrap();
            assert!(saved.starts_with("\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n"));
            assert!(saved.ends_with("</ControlList>\r\n"));
            assert!(!saved.replace("\r\n", "").contains('\n'));
            assert!(saved.contains("<Empty></Empty>"));
            assert!(saved.contains("<LightsceneC></LightsceneC>"));
            assert!(saved.contains(r#"<LightsceneC Num="91"/>"#));
            let expected = xml.replace(r#"Num="future""#, r#"Num="2""#).replace(
                r#"Date="old" DateS="86400""#,
                r#"Date="2009-01-02 02:00:01" DateS="86401""#,
            );
            let parse = |text: &str| {
                xmltree::Element::parse(text.trim_start_matches('\u{feff}').as_bytes()).unwrap()
            };
            assert_eq!(parse(saved), parse(&expected));
            let repeated = recount(&result.bytes, "uuid:lamp", &clock()).unwrap();
            assert!(!repeated.change.changed);
            assert!(!repeated.document_stamped);
            assert_eq!(repeated.bytes, result.bytes);
        }

        #[test]
        fn inserts_and_repairs_zero_in_fragments_but_never_serializes_equivalent_values() {
            for (attr, before, effective, after) in [
                ("", None, None, "0"),
                (r#" Num="""#, Some(""), None, "0"),
                (r#" Num="2147483648""#, Some("2147483648"), None, "0"),
                (r#" Num="-1""#, Some("-1"), Some(-1), "0"),
                (" Num = '&#43;000'", Some("+000"), Some(0), "+000"),
                (r#" Num="-0""#, Some("-0"), Some(0), "-0"),
            ] {
                let xml = format!(
                    r#"<C Type="LightController2" U="lamp"><LightscenesC{attr}/><Empty/></C>"#
                );
                let result = recount(xml.as_bytes(), "uuid:lamp", &clock()).unwrap();
                assert_eq!(result.change.before.as_deref(), before);
                assert_eq!(result.change.before_effective, effective);
                assert_eq!(result.change.after, after);
                assert_eq!(result.change.scenes, 0);
                assert_eq!(result.change.changed, effective != Some(0));
                assert!(!result.document_stamped);
                if result.change.changed {
                    let saved = std::str::from_utf8(&result.bytes).unwrap();
                    assert!(saved.contains(r#"<LightscenesC Num="0"/>"#));
                    assert!(saved.contains("<Empty></Empty>"));
                    assert!(!saved.contains("DateS"));
                } else {
                    assert_eq!(result.bytes, xml.as_bytes());
                }
            }

            let xml = document().replace(r#"Num="future""#, "Num = '+0002'");
            let result = recount(xml.as_bytes(), "uuid:lamp", &clock()).unwrap();
            assert_eq!(result.change.after, "+0002");
            assert!(!result.change.changed);
            assert!(!result.document_stamped);
            assert_eq!(result.bytes, xml.as_bytes());
        }

        #[test]
        fn types_load_selection_type_and_direct_container_refusals() {
            let fail =
                |xml: &str, selector| recount(xml.as_bytes(), selector, &clock()).unwrap_err();
            assert!(matches!(
                fail("<ControlList><C", "missing"),
                Refusal::Edit(EditError::Load(_))
            ));
            let xml = document().replace(
                r#"Type="LightController2" U="other""#,
                r#"Type="And" U="other""#,
            );
            assert!(matches!(
                fail(&xml, "Lamps"),
                Refusal::Edit(EditError::Selection(SelectionError::Ambiguous { .. }))
            ));
            assert!(matches!(fail(&xml, "uuid:other"), Refusal::WrongBlockType));
            assert!(matches!(
                fail(&xml, "uuid:missing"),
                Refusal::Edit(EditError::Selection(SelectionError::NotFound { .. }))
            ));
            for (storage, found) in [
                ("<Nested><LightscenesC/></Nested>".to_owned(), 0),
                (format!("<LightscenesC/>{COLLECTION}"), 2),
            ] {
                let xml = xml.replace(COLLECTION, &storage);
                assert!(matches!(
                    fail(&xml, "uuid:lamp"),
                    Refusal::ContainerCount { found: actual } if actual == found
                ));
            }
        }

        #[test]
        fn refuses_unsupported_xml_and_nonordinary_ownership_even_on_noop() {
            let xml = document().replace(r#"Num="future""#, r#"Num="+02""#);
            let fail = |xml: String| recount(xml.as_bytes(), "uuid:lamp", &clock()).unwrap_err();
            for namespaced in [
                xml.replace(r#"Num="+02""#, r#"xml:Num="+02""#),
                xml.replace("<ControlList>", r#"<ControlList xmlns="urn:default">"#),
                xml.replace("<LightsceneC/>", r#"<x:LightsceneC xmlns:x="urn:mood"/>"#),
            ] {
                assert!(matches!(fail(namespaced), Refusal::NamespacedXml));
            }
            for unsupported in [
                xml.replace("<ControlList>", "<!DOCTYPE ControlList><ControlList>"),
                xml.replace("</ControlList>", "</ControlList><Hidden/>"),
            ] {
                assert!(matches!(fail(unsupported), Refusal::UnsupportedXml));
            }
            for template in [
                xml.replace(r#"NumO="99">"#, r#"NumO="99"><Template/>"#),
                xml.replace(r#"NumO="99">"#, r#"NumO="99"><C Type="DocumentTemplate"/>"#),
                xml.replace(r#"Type="Program""#, r#"Type="DocumentTemplate""#),
                xml.replace(r#"Type="Page""#, r#"Type="Page" WF="536870912""#),
                xml.replace(r#"Type="Page""#, r#"Type="Page" uuidParent="inherited""#),
                xml.replace(r#"U="lamp""#, r#"U="lamp" IfT="false""#),
            ] {
                assert!(matches!(fail(template), Refusal::TemplateOrInherited));
            }
            for document in [
                xml.replace("</ControlList>", r#"<C Type="Document"/></ControlList>"#),
                format!(
                    r#"<ControlList><C Type="Document"/><C Type="LightController2" U="lamp">{COLLECTION}</C></ControlList>"#
                ),
            ] {
                assert!(matches!(fail(document), Refusal::DocumentScope));
            }
        }
    }

    #[test]
    fn keeps_shape_observations_and_typed_load_selection_and_type_failures() {
        let xml = br#"<Root><C Type="LightController2" U="light">
          <LightscenesC><LightsceneC Name="One"/></LightscenesC>
          <LightscenesC><LightsceneC Name="Two"/></LightscenesC>
        </C><C Type="LightController2" U="empty"/></Root>"#;
        let inspection = inspect_lighting(xml, "uuid:light").unwrap();
        assert_eq!(inspection.direct_scene_collection_count, 2);
        assert_eq!(inspection.sole_scene_collection, None);
        assert_eq!(inspection.moods.len(), 2);
        assert!(
            inspection
                .moods
                .iter()
                .all(|mood| !mood.in_sole_direct_collection)
        );
        assert!(
            inspect_lighting(xml, "uuid:empty")
                .unwrap()
                .moods
                .is_empty()
        );

        assert!(matches!(
            inspect_lighting(b"<C A='1' A='2'/>", "uuid:missing"),
            Err(LightingInspectionError::Edit(EditError::Load(_)))
        ));
        assert!(matches!(
            inspect_lighting(XML, "Lamps"),
            Err(LightingInspectionError::Edit(EditError::Selection(
                SelectionError::Ambiguous { matches, .. }
            ))) if matches.len() == 2
        ));
        assert!(matches!(
            inspect_lighting(XML, "uuid:wrong"),
            Err(LightingInspectionError::NotLightController { selector, block_type })
                if selector == "uuid:wrong" && block_type == "And"
        ));
    }
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

fn status_monitor_fixture() -> String {
    let rows: String = [5, 3, 0, 7, 9, 1, 6, 2, 8, 4, 10]
        .into_iter()
        .map(|index| {
            let attrs = match index {
                5 => r#" Na=" A &amp; B " Val="+5e0" Cl="1,2,3" Future="""#.to_owned(),
                3 => r#" Na="" Val="3""#.to_owned(),
                0 => r#" Val="0""#.to_owned(),
                7 => r#" Na="no value""#.to_owned(),
                10 => r#" Na="Rest""#.to_owned(),
                _ => format!(r#" Na="Active" Val="{index}""#),
            };
            let output = if index == 5 {
                "05".to_owned()
            } else {
                index.to_string()
            };
            format!(r#"<State Id="ABCDEF01-0002-0003-{index:016x}" OIdx="{output}"{attrs}/>"#)
        })
        .collect();
    format!(
        r#"<Loxone><C Type="Place" Title="Workshop" U="place"/>
        <C Type="StatusMonitor" Title="Status" U="Mon-1"><IoData Pr="place"/>
        <Co K="AQ1"><State Id="nested-is-ignored"/></Co>{rows}<Other/>
        </C><C Type="Switch" Title="Other" U="not-monitor"/></Loxone>"#
    )
}

#[test]
fn status_monitor_inspection_retains_raw_records_without_defaults() {
    use lox_core::configuration::editor::inspect_status_monitor;
    use std::collections::BTreeMap;

    let xml = status_monitor_fixture();
    let inspection = inspect_status_monitor(xml.as_bytes(), "uuid:Mon-1").unwrap();
    assert_eq!(
        inspect_status_monitor(xml.as_bytes(), "stat [work]").unwrap(),
        inspection
    );
    assert_eq!(
        (inspection.uuid.as_str(), inspection.title.as_str()),
        ("Mon-1", "Status")
    );
    assert_eq!(inspection.states.len(), 11);
    let first = &inspection.states[0];
    assert_eq!(
        (first.priority, first.output_index, first.configured),
        (1, 5, true)
    );
    assert_eq!(first.id, "ABCDEF01-0002-0003-0000000000000005");
    assert_eq!(
        first.attributes,
        BTreeMap::from([
            ("Cl".into(), "1,2,3".into()),
            ("Future".into(), "".into()),
            ("Id".into(), first.id.clone()),
            ("Na".into(), " A & B ".into()),
            ("OIdx".into(), "05".into()),
            ("Val".into(), "+5e0".into()),
        ])
    );
    let [empty_name, absent_name, absent_value] = &inspection.states[1..4] else {
        unreachable!();
    };
    assert!(!empty_name.configured && !absent_name.configured && !absent_value.configured);
    assert_eq!(empty_name.attributes["Na"], "");
    assert!(!absent_name.attributes.contains_key("Na"));
    assert!(!absent_value.attributes.contains_key("Val"));
    let rest = &inspection.states[10];
    assert_eq!(
        (rest.priority, rest.output_index, rest.configured),
        (11, 10, false)
    );
    assert!(!rest.attributes.contains_key("Val"));

    let unnamed = xml.replace(r#" Title="Status" U="Mon-1""#, "");
    let unnamed = inspect_status_monitor(unnamed.as_bytes(), "Type:StatusMonitor").unwrap();
    assert!(unnamed.uuid.is_empty() && unnamed.title.is_empty());
}

#[test]
fn status_monitor_inspection_refuses_in_original_order_with_causes() {
    use lox_core::configuration::editor::{
        EditError, SelectionError, StatusMonitorInspectionError as Failure, inspect_status_monitor,
    };

    let error = inspect_status_monitor(b"<Loxone><C></Loxone>", "Missing").unwrap_err();
    assert!(matches!(&error, Failure::Edit(EditError::Load(_))));
    assert!(error.source().and_then(|cause| cause.source()).is_some());

    let inherited = br#"<Loxone iObjS=""><C Type="StatusMonitor" Title="Status"/><C Type="statusmonitor" Title="Wrong"/><C Type="Switch" Title="Status"/></Loxone>"#;
    let error = inspect_status_monitor(inherited, "Missing").unwrap_err();
    assert!(matches!(
        &error,
        Failure::Edit(EditError::Selection(SelectionError::NotFound { selector, .. }))
            if selector == "Missing"
    ));
    assert!(error.source().and_then(|cause| cause.source()).is_some());
    assert!(matches!(
        inspect_status_monitor(inherited, "Status"),
        Err(Failure::Edit(EditError::Selection(
            SelectionError::Ambiguous { .. }
        )))
    ));
    assert!(matches!(
        inspect_status_monitor(inherited, "Wrong"),
        Err(Failure::NotStatusMonitor { selector }) if selector == "Wrong"
    ));
    assert!(matches!(
        inspect_status_monitor(
            br#"<Loxone><C U="mon" Type="StatusMonitor"/><Other iObjS=""/></Loxone>"#,
            "uuid:mon"
        ),
        Err(Failure::InheritedConfiguration)
    ));
}

#[test]
fn status_monitor_inspection_keeps_strict_state_identity_and_shape() {
    use lox_core::configuration::editor::{
        StatusMonitorInspectionError as Failure, inspect_status_monitor,
    };

    let xml = status_monitor_fixture();
    let failure = |old: &str, new: &str| {
        let changed = xml.replacen(old, new, 1);
        assert_ne!(changed, xml);
        inspect_status_monitor(changed.as_bytes(), "Status").unwrap_err()
    };
    assert!(matches!(
        failure(r#"<State Id="ABCDEF01"#, r#"<Other Id="ABCDEF01"#),
        Failure::StateCount { found: 10 }
    ));
    assert!(matches!(
        failure(
            r#"<State Id="ABCDEF01"#,
            r#"<s:State xmlns:s="urn:synthetic" Id="ABCDEF01"#
        ),
        Failure::NamespacedState
    ));
    assert!(matches!(
        failure(r#" Id="ABCDEF01-0002-0003-0000000000000005""#, ""),
        Failure::MissingStateId
    ));
    assert!(matches!(
        failure("ABCDEF01-0002-0003-0000000000000003", "abcdef01-0002-0003-0000000000000005"),
        Failure::DuplicateStateId { id } if id == "abcdef01-0002-0003-0000000000000005"
    ));
    for rejected in [
        "00000000-0000-0000-0000000000000000",
        "abcdef01-0002-0003-0000-000000000003",
    ] {
        assert!(matches!(
            failure("ABCDEF01-0002-0003-0000000000000003", rejected),
            Failure::InvalidStateId { id } if id == rejected
        ));
    }
    assert!(matches!(
        failure(r#" OIdx="3""#, ""),
        Failure::InvalidOutputIndex
    ));
    assert!(matches!(
        failure(r#"OIdx="3""#, r#"OIdx="5""#),
        Failure::DuplicateOutputIndex { output_index: 5 }
    ));
    assert!(matches!(
        failure(r#"OIdx="05""#, r#"OIdx="10""#),
        Failure::RestNotLast
    ));
    assert!(matches!(
        failure(r#"Na="Rest""#, r#"Na="Rest" Val="42""#),
        Failure::RestHasValue
    ));
    for raw in ["NaN".to_owned(), "inf".to_owned(), f64::MAX.to_string()] {
        assert!(matches!(
            failure(r#"Val="3""#, &format!(r#"Val="{raw}""#)),
            Failure::InvalidValue { raw: observed } if observed == raw
        ));
    }
    assert!(matches!(
        failure(r#"Val="3""#, r#"Val="-0""#),
        Failure::DuplicateValue { value } if value == 0.0
    ));
}

mod load_manager {
    use lox_core::configuration::editor::{
        EditError, LoadManagerInspectionError as Failure, SelectionError,
        inspect_load_manager as inspect,
    };
    use std::{collections::BTreeMap, error::Error};

    fn document() -> String {
        let rows: String = [5, 0, 3, 1, 2, 4, 6, 7, 8, 9, 10, 11]
                .into_iter()
                .map(|index| match index {
                    5 => r#"<l:Load xmlns:l="urn:synthetic" ioIdx="05" nm=" A &amp; B " pwr="02.5000e0" minOn="+0012" Future=""/>"#.to_owned(),
                    0 => r#"<Load nm="" maxOff="" pwr="unknown" Note="at 7am=ok"/>"#.to_owned(),
                    _ => format!(r#"<Load ioIdx="{index}"/>"#),
                })
                .collect();
        format!(
            "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
                 <Loxone iObjS=\"\"><C Type=\"Place\" Title=\"Workshop\" U=\"room\"/>\
                 <C Type=\"LoadShed\" U=\"Manager\" Title=\"Loads\" gid=\"primary\">\
                 <IoData Pr=\"room\"/><Other><Load ioIdx=\"invalid\"/></Other>{rows}</C>\
                 <C Type=\"Switch\" U=\"Manager\" Title=\"Loads\"/></Loxone>\r\n"
        )
    }

    #[test]
    fn inspection_observes_xml_order_resolved_identity_and_loader_values() {
        let xml = document();
        let inspection = inspect(xml.as_bytes(), "uuid:Manager").unwrap();
        for selector in ["load", "load [work]", "gid:PRIMARY", "Type:loadshed"] {
            assert_eq!(inspect(xml.as_bytes(), selector).unwrap(), inspection);
        }
        assert_eq!(
            (inspection.uuid.as_str(), inspection.title.as_str()),
            ("Manager", "Loads")
        );
        assert_eq!(
            inspection
                .loads
                .iter()
                .map(|load| (load.priority, load.io_index))
                .collect::<Vec<_>>(),
            [
                (1, 5),
                (2, 0),
                (3, 3),
                (4, 1),
                (5, 2),
                (6, 4),
                (7, 6),
                (8, 7),
                (9, 8),
                (10, 9),
                (11, 10),
                (12, 11),
            ]
        );
        assert_eq!(
            inspection.loads[0].attributes,
            BTreeMap::from([
                ("ioIdx".into(), "05".into()),
                ("nm".into(), " A & B ".into()),
                ("pwr".into(), "02.5000e0".into()),
                ("minOn".into(), "+0012".into()),
                ("Future".into(), "".into()),
            ])
        );
        assert_eq!(
            inspection.loads[1].attributes,
            BTreeMap::from([
                ("nm".into(), "".into()),
                ("maxOff".into(), "".into()),
                ("pwr".into(), "unknown".into()),
                ("Note".into(), "at _7am=ok".into()),
            ])
        );
        assert_eq!(inspection.loads[2].attributes.len(), 1);

        let unnamed = xml.replace(r#" U="Manager" Title="Loads" gid="primary""#, "");
        let unnamed = inspect(unnamed.as_bytes(), "Type:LoadShed").unwrap();
        assert!(unnamed.uuid.is_empty() && unnamed.title.is_empty());
    }

    #[test]
    fn inspection_preserves_loader_and_selector_refusals_before_shape_checks() {
        let error = inspect(b"<Loxone><C></Loxone>", "Missing").unwrap_err();
        assert!(matches!(&error, Failure::Edit(EditError::Load(_))));
        assert!(error.source().and_then(|cause| cause.source()).is_some());

        let xml = document();
        assert!(matches!(
            inspect(xml.as_bytes(), "uuid:manager"),
            Err(Failure::Edit(EditError::Selection(SelectionError::NotFound { selector, .. })))
                if selector == "uuid:manager"
        ));
        let ambiguous = xml.replace(r#"Type="Switch" U="Manager""#, r#"Type="Switch" U="Other""#);
        assert!(matches!(
            inspect(ambiguous.as_bytes(), "Loads"),
            Err(Failure::Edit(EditError::Selection(SelectionError::Ambiguous { matches, .. })))
                if matches.len() == 2 && matches[1].block_type == "Switch"
        ));
        let wrong_type = xml.replace(r#"Type="LoadShed""#, r#"Type="loadshed""#);
        assert!(matches!(
            inspect(wrong_type.as_bytes(), "Type:LoadShed"),
            Err(Failure::NotLoadManager { selector }) if selector == "Type:LoadShed"
        ));
    }

    #[test]
    fn inspection_types_complete_shape_and_identity_refusals() {
        let xml = document();
        let invalid = |replacement: &str| xml.replace(r#"ioIdx="1""#, replacement);
        let missing_row = invalid(r#"ioIdx="+1""#).replace(r#"<Load ioIdx="11"/>"#, "");
        assert!(matches!(
            inspect(missing_row.as_bytes(), "Loads"),
            Err(Failure::RecordCount { found: 11 })
        ));
        for raw in ["", "+1", " 1", "12", "other"] {
            let changed = invalid(&format!(r#"ioIdx="{raw}""#));
            assert!(matches!(
                inspect(changed.as_bytes(), "Loads"),
                Err(Failure::InvalidIoIndex { raw: observed }) if observed == raw
            ));
        }
        for replacement in [r#"ioIdx="000""#, ""] {
            let changed = invalid(replacement);
            assert!(matches!(
                inspect(changed.as_bytes(), "Loads"),
                Err(Failure::DuplicateIoIndex { io_index: 0 })
            ));
        }
    }

    #[cfg(feature = "configuration-xml")]
    mod movement {
        use super::inspect;
        use chrono::{DateTime, FixedOffset};
        use lox_core::configuration::editor::{
            LoadManagerInspectionError, LoadManagerMove,
            LoadManagerMoveDirection::{Down, Up},
            LoadManagerMoveError as Refusal, move_load_manager_load as move_load,
        };

        fn clock() -> DateTime<FixedOffset> {
            DateTime::parse_from_rfc3339("2009-01-02T01:59:59+02:00").unwrap()
        }

        fn document() -> String {
            let rest: String = [3, 1, 2, 4, 6, 7, 8, 9, 10, 11]
                .into_iter()
                .map(|index| format!(r#"<Load ioIdx="{index}"/>"#))
                .collect();
            format!(
                "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
                <Loxone><C Type=\"Document\" Date=\"old\" DateS=\"86400\">\
                <C Type=\"Place\" U=\"room\" Title=\"Workshop\"/><C Type=\"Page\" WF=\"16384\">\
                <C Type=\"LoadShed\" U=\"manager\" Title=\"Loads\" WF=\"16384\">\
                <IoData Pr=\"room\"/><Co K=\"AQl1\" U=\"wire\" U=\"wire\" 12hTF=\"true\"/>\
                <Load ioIdx=\"05\" nm=\"Five\" pwr=\"2.5000e0\" maxOff=\"123\" minOn=\"+0045\"><Extra U=\"Five\"/></Load>\
                <Between/><Other><Load ioIdx=\"invalid\"/></Other>\
                <!-- <x:Load xmlns:x='urn:comment'/> <Fake xml:Type='Document'/> -->\
                <Opaque><![CDATA[<Fake xml:Type='Document'/>]]></Opaque>\
                <Load nm=\" A &amp; B \" Unknown=\"zero\"><Empty/></Load>\
                {rest}</C></C></C></Loxone>\r\n"
            )
        }

        #[test]
        fn swaps_adjacent_record_slots_by_connector_and_serializes_with_explicit_clock() {
            let xml = document();
            let source = xml.as_bytes();
            let before = inspect(source, "Loads [Work]").unwrap();
            let result = move_load(source, "Loads [Work]", 0, Up, &clock()).unwrap();
            assert_eq!(
                result.movement,
                LoadManagerMove {
                    io_index: 0,
                    from_priority: 2,
                    to_priority: 1,
                }
            );
            assert!(result.document_stamped);
            assert_eq!(
                result.inspection.loads[0].attributes,
                before.loads[1].attributes
            );
            assert_eq!(
                result.inspection.loads[1].attributes,
                before.loads[0].attributes
            );
            assert!(!result.inspection.loads[0].attributes.contains_key("ioIdx"));
            assert_eq!(result.inspection.loads[2..], before.loads[2..]);
            assert_eq!(
                inspect(&result.bytes, "uuid:manager").unwrap(),
                result.inspection
            );

            let saved = std::str::from_utf8(&result.bytes).unwrap();
            assert!(saved.starts_with("\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n"));
            assert!(saved.ends_with("</Loxone>\r\n"));
            assert!(saved.contains(r#"Date="2009-01-02 02:00:01" DateS="86401""#));
            assert!(saved.contains(r#"<Co K="AQl1" U="wire" 12hTF="true"/>"#));
            let positions = [
                r#"<Load nm=" A &amp; B " Unknown="zero">"#,
                "<Empty></Empty>",
                "<Between></Between>",
                r#"<Load ioIdx="invalid"/>"#,
                "<!-- <x:Load",
                "<![CDATA[<Fake xml:Type='Document'/>]]>",
                r#"<Load ioIdx="05" nm="Five" pwr="2.5000e0" maxOff="123" minOn="+0045">"#,
                r#"<Extra U="Five"/>"#,
                r#"<Load ioIdx="3"/>"#,
            ]
            .map(|part| saved.find(part).unwrap());
            assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        }

        #[test]
        fn types_boundaries_and_preserves_existing_shape_validation() {
            let xml = document();
            assert!(matches!(
                move_load(xml.as_bytes(), "Loads", 5, Up, &clock()),
                Err(Refusal::Boundary {
                    io_index: 5,
                    direction: Up
                })
            ));
            assert!(matches!(
                move_load(xml.as_bytes(), "Loads", 11, Down, &clock()),
                Err(Refusal::Boundary {
                    io_index: 11,
                    direction: Down
                })
            ));
            assert!(matches!(
                move_load(xml.as_bytes(), "Loads", 12, Down, &clock()),
                Err(Refusal::InvalidRequestedIoIndex { io_index: 12 })
            ));
            let check =
                |xml: String| move_load(xml.as_bytes(), "Loads", 0, Up, &clock()).unwrap_err();
            assert!(matches!(
                check(xml.replace(
                    r#"<Load ioIdx="11"/>"#,
                    r#"<Other><Load ioIdx="11"/></Other>"#
                )),
                Refusal::Inspection(LoadManagerInspectionError::RecordCount { found: 11 })
            ));
            assert!(matches!(
                check(xml.replace(r#"ioIdx="05""#, r#"ioIdx="000""#)),
                Refusal::Inspection(LoadManagerInspectionError::DuplicateIoIndex { io_index: 0 })
            ));
            assert!(matches!(
                check(xml.replace(r#"ioIdx="05""#, r#"ioIdx="+5""#)),
                Refusal::Inspection(LoadManagerInspectionError::InvalidIoIndex { raw })
                    if raw == "+5"
            ));
        }

        #[test]
        fn refuses_namespaces_template_ownership_and_multiple_roots() {
            let xml = document();
            let check =
                |xml: String| move_load(xml.as_bytes(), "Loads", 0, Up, &clock()).unwrap_err();
            for namespaced in [
                xml.replace(r#"Type="LoadShed""#, r#"xml:Type="LoadShed""#),
                xml.replace(r#"ioIdx="05""#, r#"xml:ioIdx="05""#),
                xml.replace(r#"DateS="86400""#, r#"xml:DateS="86400""#),
                xml.replace("<Loxone>", r#"<Loxone xmlns="urn:inherited">"#),
                xml.replace(
                    r#"<Load ioIdx="11"/>"#,
                    r#"<x:Load xmlns:x="urn:load" ioIdx="11"/>"#,
                ),
            ] {
                assert!(matches!(check(namespaced), Refusal::NamespacedXml));
            }
            for template in [
                xml.replace(
                    r#"DateS="86400">"#,
                    r#"DateS="86400"><Template IfT="false"/>"#,
                ),
                xml.replace(
                    r#"DateS="86400">"#,
                    r#"DateS="86400"><C Type="DocumentTemplate"/>"#,
                ),
                xml.replace(r#"Type="Document""#, r#"Type="DocumentTemplate""#),
                xml.replace("</Loxone>", r#"<C Type="DocumentTemplate"/></Loxone>"#),
                xml.replace(r#"Type="Page" WF="16384""#, r#"Type="Page" WF="536887296""#),
                xml.replace(r#"Type="Page""#, r#"Type="Page" uuidParent="inherited""#),
                xml.replace(r#"Type="LoadShed""#, r#"Type="LoadShed" IfT="false""#),
            ] {
                assert!(matches!(check(template), Refusal::TemplateOrInherited));
            }
            assert!(matches!(
                check(xml.replace("</Loxone>", r#"<C Type="Document"/></Loxone>"#)),
                Refusal::DocumentScope
            ));
            assert!(matches!(
                check(xml.replace("</Loxone>", "</Loxone><Hidden/>")),
                Refusal::UnsupportedXml
            ));
            assert!(matches!(
                check(xml.replace("<Loxone>", "<!DOCTYPE Loxone><Loxone>")),
                Refusal::UnsupportedXml
            ));
            let latin1 = xml
                .trim_start_matches('\u{feff}')
                .replace("utf-8", "ISO-8859-1")
                .replace("Five", "Fünf");
            assert!(matches!(check(latin1), Refusal::UnsupportedXml));
        }

        #[test]
        fn refuses_when_mixed_row_children_would_reselect_another_owner() {
            let xml = document()
                .replace(
                    r#"<Extra U="Five"/>"#,
                    r#"<C Type="Place" U="room" Title="Garage"/><C Type="Place" U="room2" Title="Workshop"/>"#,
                )
                .replace(
                    "<Empty/>",
                    r#"<C Type="Place" U="room" Title="Workshop"/><C Type="Place" U="room2" Title="Garage"/>"#,
                )
                .replace(
                    "</C></C></C></Loxone>",
                    r#"</C><C Type="LoadShed" U="other" Title="Loads"><IoData Pr="room2"/></C></C></C></Loxone>"#,
                );
            assert_eq!(
                inspect(xml.as_bytes(), "Loads [Work]").unwrap().uuid,
                "manager"
            );
            assert!(matches!(
                move_load(xml.as_bytes(), "Loads [Work]", 0, Up, &clock()),
                Err(Refusal::SelectedOwnerChanged)
            ));
        }
    }

    #[cfg(feature = "configuration-xml")]
    mod name {
        use super::{EditError, Failure, SelectionError, inspect};
        use chrono::{DateTime, FixedOffset};
        use lox_core::configuration::editor::{
            LoadManagerNameChange, LoadManagerNameError as Refusal,
            set_load_manager_load_name as set_name,
        };

        fn clock() -> DateTime<FixedOffset> {
            DateTime::parse_from_rfc3339("2009-01-02T01:59:59+02:00").unwrap()
        }

        fn document() -> String {
            let rest: String = [3, 1, 2, 4, 6, 7, 8, 9, 10, 11]
                .into_iter()
                .map(|index| format!(r#"<Load ioIdx="{index}"/>"#))
                .collect();
            format!(
                "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
                <Loxone><C Type=\"Document\" Date=\"old\" DateS=\"86400\">\
                <C Type=\"Place\" U=\"room\" Title=\"Workshop\"/>\
                <C Type=\"LoadShed\" U=\"manager\" Title=\"Loads\" WF=\"16384\">\
                <IoData Pr=\"room\"/><Co K=\"AQl1\" U=\"wire\" 12hTF=\"true\"/>\
                <Load ioIdx=\"05\" nm=\"Five\" pwr=\"2.5000e0\"/>\
                <Between/><Other><Load ioIdx=\"invalid\" nm=\"Nested\"/></Other>\
                <Load nm=\" A &amp; B \" pwr=\"invalid\" minOn=\"+0045\" Future=\"\"><Extra U=\"keep\"/></Load>\
                {rest}</C></C></Loxone>\r\n"
            )
        }

        #[test]
        fn edits_the_resolved_record_without_changing_order_or_other_data() {
            let xml = document();
            let selector = "Loads [Work]";
            let mut expected = inspect(xml.as_bytes(), selector).unwrap();
            let name = "  Wärmepumpe 東京 🔌 & \"Haus\" <ok>  ";
            let result = set_name(xml.as_bytes(), selector, 0, name, &clock()).unwrap();
            assert_eq!(
                result.change,
                LoadManagerNameChange {
                    io_index: 0,
                    before: Some(" A & B ".into()),
                    after: Some(name.into()),
                    changed: true,
                }
            );
            assert_eq!(expected.loads[1].io_index, 0);
            expected.loads[1]
                .attributes
                .insert("nm".into(), name.into());
            assert_eq!(result.inspection, expected);
            assert_eq!(inspect(&result.bytes, selector).unwrap(), expected);
            assert!(result.document_stamped);
            let saved = std::str::from_utf8(&result.bytes).unwrap();
            assert!(saved.starts_with("\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n"));
            assert!(saved.ends_with("</Loxone>\r\n"));
            assert!(saved.contains(r#"Date="2009-01-02 02:00:01" DateS="86401""#));
            let positions = [
                r#"<Co K="AQl1" U="wire" 12hTF="true"/>"#,
                r#"<Load ioIdx="05" nm="Five" pwr="2.5000e0"/>"#,
                "<Between></Between>",
                r#"<Load ioIdx="invalid" nm="Nested"/>"#,
                r#"<Load nm="  Wärmepumpe 東京 🔌 &amp; &quot;Haus&quot; "#,
                r#"<Extra U="keep"/>"#,
                r#"<Load ioIdx="3"/>"#,
            ]
            .map(|part| saved.find(part).unwrap());
            assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        }

        #[test]
        fn clear_omits_nonempty_name_and_exact_noops_return_original_bytes() {
            let xml = document();
            let without_document = xml
                .replace(r#"<C Type="Document" Date="old" DateS="86400">"#, "")
                .replace("</C></C></Loxone>", "</C></Loxone>");
            let result = set_name(without_document.as_bytes(), "Loads", 0, "", &clock()).unwrap();
            assert!(result.change.changed);
            assert_eq!(result.change.before.as_deref(), Some(" A & B "));
            assert_eq!(result.change.after, None);
            assert!(!result.document_stamped);
            assert!(!result.inspection.loads[1].attributes.contains_key("nm"));
            assert_eq!(inspect(&result.bytes, "Loads").unwrap(), result.inspection);

            for (stored, request, observed) in [
                (r#" nm=" A &#38; B ""#, " A & B ", Some(" A & B ")),
                (r#" nm="""#, "", Some("")),
                ("", "", None),
            ] {
                let source = xml.replacen(r#" nm=" A &amp; B ""#, stored, 1);
                let result = set_name(source.as_bytes(), "Loads", 0, request, &clock()).unwrap();
                assert!(!result.change.changed);
                assert_eq!(result.change.before.as_deref(), observed);
                assert_eq!(result.change.after.as_deref(), observed);
                assert_eq!(
                    result.inspection.loads[1]
                        .attributes
                        .get("nm")
                        .map(String::as_str),
                    observed
                );
                assert!(!result.document_stamped);
                assert_eq!(result.bytes, source.as_bytes());
            }
        }

        #[test]
        fn orders_request_load_selection_scope_and_shape_refusals() {
            let check = |xml: &str, selector: &str, index, name: &str| {
                set_name(xml.as_bytes(), selector, index, name, &clock()).unwrap_err()
            };
            let malformed = "<Loxone><C></Loxone>";
            assert!(matches!(
                check(malformed, "Missing", 12, "\t"),
                Refusal::InvalidRequestedIoIndex { io_index: 12 }
            ));
            assert!(matches!(
                check(malformed, "Missing", 0, "\t"),
                Refusal::InvalidName
            ));
            assert!(matches!(
                check(malformed, "Missing", 0, "New"),
                Refusal::Edit(EditError::Load(_))
            ));
            let xml = document();
            assert!(matches!(
                check(&xml, "Missing", 0, "New"),
                Refusal::Inspection(Failure::Edit(EditError::Selection(
                    SelectionError::NotFound { .. }
                )))
            ));
            assert!(matches!(
                check(&xml, "uuid:room", 0, "New"),
                Refusal::Inspection(Failure::NotLoadManager { .. })
            ));
            let duplicate = xml.replace(r#"ioIdx="05""#, r#"ioIdx="000""#);
            assert!(matches!(
                check(&duplicate, "Loads", 0, "New"),
                Refusal::Inspection(Failure::DuplicateIoIndex { io_index: 0 })
            ));
            let inherited = duplicate.replace(r#"WF="16384""#, r#"IfT="false""#);
            assert!(matches!(
                check(&inherited, "Loads", 0, " A & B "),
                Refusal::TemplateOrInherited
            ));
            let namespaced = xml.replace(r#" nm=" A"#, r#" xml:nm=" A"#);
            assert!(matches!(
                check(&namespaced, "Missing", 0, "New"),
                Refusal::NamespacedXml
            ));
            let nonowning = xml
                .replace(
                    r#"<C Type="Document" Date="old" DateS="86400">"#,
                    r#"<C Type="Document" Date="old" DateS="86400"/>"#,
                )
                .replace("</C></C></Loxone>", "</C></Loxone>");
            assert!(matches!(
                check(&nonowning, "Loads", 0, ""),
                Refusal::DocumentScope
            ));
        }

        #[test]
        fn rejects_changed_names_that_cannot_round_trip_exactly() {
            let xml = document();
            for name in ["at 7am=on", "at _12hTF=on"] {
                assert!(matches!(
                    set_name(xml.as_bytes(), "Loads", 0, name, &clock()),
                    Err(Refusal::NameTextCollision)
                ));
            }
            let stored = xml.replacen(" A &amp; B ", "at _12hTF=on", 1);
            let noop = set_name(stored.as_bytes(), "Loads", 0, "at _12hTF=on", &clock()).unwrap();
            assert!(!noop.change.changed);
            assert_eq!(noop.bytes, stored.as_bytes());
        }

        #[test]
        fn refuses_non_1_0_or_untrustworthy_declarations_even_for_a_disjoint_name() {
            let xml = document();
            let disjoint = xml.replacen(r#"nm="Five""#, r#"nm="&#x1;""#, 1);
            let xml_1_1 = disjoint.replacen("1.0", "1.1", 1);
            let before = inspect(xml_1_1.as_bytes(), "Loads").unwrap();
            assert_eq!(before.loads[0].attributes["nm"], "\u{1}");
            for name in ["New", " A & B "] {
                assert!(matches!(
                    set_name(xml_1_1.as_bytes(), "Loads", 0, name, &clock()),
                    Err(Refusal::UnsupportedXml)
                ));
            }

            for declaration in [
                r#"<?xml version="1.2"?>"#,
                r#"<?xml encoding="utf-8"?>"#,
                r#"<?xml version="1.0" version="1.1"?>"#,
                r#"<?xml version="1.0"?><?xml version="1.1"?>"#,
            ] {
                let xml = xml.replacen(r#"<?xml version="1.0" encoding="utf-8"?>"#, declaration, 1);
                assert!(matches!(
                    set_name(xml.as_bytes(), "Loads", 0, "New", &clock()),
                    Err(Refusal::UnsupportedXml)
                ));
            }

            let xml_1_0 = disjoint.replacen("&#x1;", "&#x41;", 1);
            let result = set_name(xml_1_0.as_bytes(), "Loads", 0, "New", &clock()).unwrap();
            assert_eq!(result.inspection.loads[0].attributes["nm"], "A");
            assert_eq!(result.inspection.loads[1].attributes["nm"], "New");
            assert_eq!(inspect(&result.bytes, "Loads").unwrap(), result.inspection);
        }
    }

    #[cfg(feature = "configuration-xml")]
    mod power {
        use super::{EditError, Failure, SelectionError, inspect};
        use chrono::{DateTime, FixedOffset};
        use lox_core::configuration::editor::{
            LoadManagerPowerChange, LoadManagerPowerError as Refusal,
            set_load_manager_load_power as set_power,
        };

        fn clock() -> DateTime<FixedOffset> {
            DateTime::parse_from_rfc3339("2009-01-02T01:59:59+02:00").unwrap()
        }

        fn document() -> String {
            let rest: String = [3, 1, 2, 4, 6, 7, 8, 9, 10, 11]
                .into_iter()
                .map(|index| format!("\t\t\t<Load ioIdx=\"{index}\"/>\r\n"))
                .collect();
            format!(
                "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
                <Loxone>\r\n\
                \t<C Type=\"Document\" Date=\"old\" DateS=\"86400\" 12hTF=\"true\">\r\n\
                \t\t<C Type=\"Place\" U=\"room\" Title=\"Workshop\"/>\r\n\
                \t\t<C Type=\"LoadShed\" U=\"manager\" Title=\"Loads\" WF=\"16384\">\r\n\
                \t\t\t<IoData Pr=\"room\"/>\r\n\
                \t\t\t<Co K=\"AQl1\" U=\"wire\"/>\r\n\
                \t\t\t<Load ioIdx=\"05\" nm=\"Five\" pwr=\"invalid\"/>\r\n\
                \t\t\t<Between></Between>\r\n\
                \t\t\t<Load nm=\" A &amp; B \" pwr=\"+0008.5000E-000\" minOn=\"+0045\" Future=\"\">\r\n\
                \t\t\t\t<Extra U=\"keep\"/>\r\n\
                \t\t\t</Load>\r\n\
                {rest}\t\t</C>\r\n\
                \t</C>\r\n\
                </Loxone>\r\n"
            )
        }

        #[test]
        fn edits_only_resolved_power_and_stamps_with_the_explicit_clock() {
            let xml = document();
            let selector = "Loads [Work]";
            let mut expected = inspect(xml.as_bytes(), selector).unwrap();
            let result = set_power(xml.as_bytes(), selector, 0, "000.001", &clock()).unwrap();

            assert_eq!(
                result.change,
                LoadManagerPowerChange {
                    io_index: 0,
                    before: "+0008.5000E-000".into(),
                    after: "0.001".into(),
                    changed: true,
                }
            );
            assert_eq!(expected.loads[1].io_index, 0);
            expected.loads[1]
                .attributes
                .insert("pwr".into(), "0.001".into());
            assert_eq!(result.inspection, expected);
            assert_eq!(inspect(&result.bytes, selector).unwrap(), expected);
            assert!(result.document_stamped);
            let expected_bytes = xml.replace("+0008.5000E-000", "0.001").replace(
                r#"Date="old" DateS="86400""#,
                r#"Date="2009-01-02 02:00:01" DateS="86401""#,
            );
            assert_eq!(result.bytes, expected_bytes.as_bytes());
            let reread = set_power(&result.bytes, selector, 0, "0.001", &clock()).unwrap();
            assert!(!reread.change.changed);
            assert!(!reread.document_stamped);
            assert_eq!(reread.bytes, result.bytes);
        }

        #[test]
        fn retains_commercial_spellings_on_exact_noops_but_not_double_aliases() {
            let check = |stored: &str, request: &str| {
                let xml = document().replace("+0008.5000E-000", stored);
                let result = set_power(xml.as_bytes(), "Loads", 0, request, &clock());
                (xml, result)
            };
            for (stored, request, decoded, bits) in [
                (
                    "+0002.5000E-000",
                    "2.500",
                    "+0002.5000E-000",
                    0x4004_0000_0000_0000,
                ),
                ("&#50;.500e+0", "0002.5", "2.500e+0", 0x4004_0000_0000_0000),
                ("-0.0000e+0", "0.000", "-0.0000e+0", 0x8000_0000_0000_0000),
            ] {
                let (xml, result) = check(stored, request);
                let result = result.unwrap();
                assert!(!result.change.changed);
                assert!(!result.document_stamped);
                assert_eq!(result.change.before, decoded);
                assert_eq!(result.change.after, decoded);
                assert_eq!(result.change.after.parse::<f64>().unwrap().to_bits(), bits);
                assert_eq!(result.bytes, xml.as_bytes());
            }
            let positive_underflow = format!("0.{}1", "0".repeat(330));
            for (stored, request) in [
                ("2.50000000000000001", "2.5"),
                ("9007199254740993", "9007199254740992"),
                (positive_underflow.as_str(), "0"),
            ] {
                assert_eq!(
                    stored.parse::<f64>().unwrap().to_bits(),
                    request.parse::<f64>().unwrap().to_bits()
                );
                assert!(matches!(
                    check(stored, request).1,
                    Err(Refusal::StoredPowerAliasesRequest { io_index: 0 })
                ));
            }
            let negative_underflow = format!("-0.{}1", "0".repeat(330));
            let above_maximum = format!("{:.0}.001", f64::MAX);
            for stored in [&negative_underflow, &above_maximum] {
                assert!(matches!(
                    check(stored, "1").1,
                    Err(Refusal::UnsupportedStoredPower { io_index: 0 })
                ));
            }
        }

        #[test]
        fn validates_the_exact_kw_grid_finite_range_and_binary_formatting_ties() {
            let xml = document()
                .replace(r#" Type="Document""#, r#" Type="Page""#)
                .replace("DateS=\"86400\"", "DateS=\"keep\"");
            let edit = |value: &str| set_power(xml.as_bytes(), "Loads", 0, value, &clock());
            let largest = format!("{:.0}", f64::MAX);
            for (value, canonical, bits) in [
                ("000.000", "0", 0x0000_0000_0000_0000),
                ("00.001", "0.001", 0x3f50_624d_d2f1_a9fc),
                ("2.125", "2.125", 0x4001_0000_0000_0000),
                (
                    "281474976710656.125",
                    "281474976710656.125",
                    0x42f0_0000_0000_0002,
                ),
                (
                    "9007199254740992",
                    "9007199254740992",
                    0x4340_0000_0000_0000,
                ),
                (largest.as_str(), largest.as_str(), 0x7fef_ffff_ffff_ffff),
            ] {
                let result = edit(value).unwrap();
                assert!(!result.document_stamped);
                let observed = inspect(&result.bytes, "Loads").unwrap();
                let saved = &observed.loads[1].attributes["pwr"];
                assert_eq!(saved, canonical);
                assert_eq!(saved.parse::<f64>().unwrap().to_bits(), bits);
                let saved = std::str::from_utf8(&result.bytes).unwrap();
                assert!(saved.contains(r#"DateS="keep""#));
            }
            for value in [
                "", "-0", "+1", ".5", "2.", "1.0000", "0.0005", "1e3", "1 W", "1,5", " 1", "١",
            ] {
                assert!(matches!(edit(value), Err(Refusal::InvalidPower)), "{value}");
            }
            let above_maximum = format!("{largest}.001");
            for value in [
                "281474976710656.062",
                "9007199254740993",
                "9007199254740992.001",
                above_maximum.as_str(),
            ] {
                assert!(
                    matches!(edit(value), Err(Refusal::PowerPrecisionLoss)),
                    "{value}"
                );
            }
            assert!(matches!(
                edit(&format!("1{}", "0".repeat(309))),
                Err(Refusal::NonFinitePower)
            ));
        }

        #[test]
        fn orders_request_load_plain_xml_selector_owner_shape_and_storage_errors() {
            let check = |xml: &str, selector: &str, index, value: &str| {
                set_power(xml.as_bytes(), selector, index, value, &clock()).unwrap_err()
            };
            let malformed = "<Loxone><C></Loxone>";
            assert!(matches!(
                check(malformed, "Missing", 12, "invalid"),
                Refusal::InvalidRequestedIoIndex { io_index: 12 }
            ));
            assert!(matches!(
                check(malformed, "Missing", 0, "invalid"),
                Refusal::InvalidPower
            ));
            assert!(matches!(
                check(malformed, "Missing", 0, "1"),
                Refusal::Edit(EditError::Load(_))
            ));
            let xml = document();
            let namespaced = xml.replace(r#"Future="""#, r#"xml:lang="en""#);
            assert!(matches!(
                check(&namespaced, "Missing", 0, "1"),
                Refusal::NamespacedXml
            ));
            let disjoint_xml_1_1 =
                xml.replacen("1.0", "1.1", 1)
                    .replacen(r#"nm="Five""#, r#"nm="&#x1;""#, 1);
            assert!(matches!(
                check(&disjoint_xml_1_1, "Loads", 0, "8.5"),
                Refusal::UnsupportedXml
            ));
            let other_root = format!("{xml}<Other/>");
            assert!(matches!(
                check(&other_root, "Loads", 0, "1"),
                Refusal::UnsupportedXml
            ));
            assert!(matches!(
                check(&xml, "Missing", 0, "1"),
                Refusal::Inspection(Failure::Edit(EditError::Selection(
                    SelectionError::NotFound { .. }
                )))
            ));
            assert!(matches!(
                check(&xml, "uuid:room", 0, "1"),
                Refusal::Inspection(Failure::NotLoadManager { .. })
            ));
            let missing_power = xml.replace(r#" pwr="+0008.5000E-000""#, "");
            let duplicate = missing_power.replace(r#"ioIdx="05""#, r#"ioIdx="000""#);
            assert!(matches!(
                check(&duplicate, "Loads", 0, "1"),
                Refusal::Inspection(Failure::DuplicateIoIndex { io_index: 0 })
            ));
            let inherited = duplicate.replace(r#"WF="16384""#, r#"IfT="false""#);
            assert!(matches!(
                check(&inherited, "Loads", 0, "1"),
                Refusal::TemplateOrInherited
            ));
            let nonowning = xml.replace("</Loxone>", r#"<C Type="Document"/></Loxone>"#);
            assert!(matches!(
                check(&nonowning, "Loads", 0, "8.5"),
                Refusal::DocumentScope
            ));
            for bad in [missing_power, xml.replace("+0008.5000E-000", "2.5e1")] {
                assert!(matches!(
                    check(&bad, "Loads", 0, "1"),
                    Refusal::UnsupportedStoredPower { io_index: 0 }
                ));
            }
        }

        #[test]
        fn refuses_loss_on_load_or_canonical_reopen_of_unrelated_xml() {
            let xml = document();
            let check =
                |xml: &str, value: &str| set_power(xml.as_bytes(), "Loads", 0, value, &clock());
            let numeric_reference = xml.replace(" A &amp; B ", "wake &#55;am=on");
            assert_eq!(
                inspect(numeric_reference.as_bytes(), "Loads")
                    .unwrap()
                    .loads[1]
                    .attributes["nm"],
                "wake 7am=on"
            );
            for source in [
                xml.replace(" A &amp; B ", "wake 7am=on"),
                xml.replace(" A &amp; B ", "wake _12hTF=on"),
                xml.replace(r#"Future="""#, r#"_12hTF="unrelated""#),
                numeric_reference,
                xml.replace(r#"<Extra U="keep"/>"#, r#"<Extra U="keep"> </Extra>"#),
                xml.replace(
                    r#"<Extra U="keep"/>"#,
                    r#"<Extra U="keep"><Nested/>&#32;</Extra>"#,
                ),
                xml.replace("<Loxone>", "<!--keep outside root--><Loxone>"),
                xml.replace(
                    r#"encoding="utf-8""#,
                    r#"encoding="utf-8" standalone="yes""#,
                ),
            ] {
                assert!(matches!(
                    check(&source, "1"),
                    Err(Refusal::CanonicalStorageLoss)
                ));
            }
            let outer = format!("{xml}<!--keep outside root-->");
            let result = check(&outer, "8.5").unwrap();
            assert_eq!(result.bytes, outer.as_bytes());
            assert!(!result.document_stamped);
        }
    }
}
