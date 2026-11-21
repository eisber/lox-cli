use lox_core::configuration::rooms::{RoomInfo, RoomInventoryError, parse_rooms};
use std::error::Error;

#[test]
fn raw_room_identity_attributes_and_all_iodata_counts_are_preserved() {
    let xml = br#"<ControlList>
<IoData Pr="r&amp;1"/><IoData Pr="unknown"/>
<C Type="Place" U="r&amp;1" Title="Old"/>
<C Type="Page"><C Type="Place" U="r&amp;1" Title=" A&amp;&#65;&custom; "/></C>
<IoData Pr="r&amp;1"></IoData><IoData Pr="r&#38;1"/>
<C Type="Place" U="" Title=""/>
<IoData Pr=""/><IoData/><Other Pr="r&amp;1"/>
<C Type="Place" Title="Missing ID"/><C Type="Place" U="missing-title"/>
<C Type="place" U="lower" Title="Ignored"/>
<C Type="Pl&#97;ce" U="entity-type" Title="Ignored"/>
<C Type="Place" U="duplicate" U="second" Title="Z" Title="Second"/>
<C Type="Place" U="quoted" Title='has 12hTF="x"'/>
</ControlList>"#;
    let before = xml.to_vec();
    assert_eq!(
        parse_rooms(xml).unwrap(),
        [
            RoomInfo {
                uuid: "".into(),
                name: "".into(),
                item_count: 1
            },
            RoomInfo {
                uuid: "r&amp;1".into(),
                name: " A&amp;&#65;&custom; ".into(),
                item_count: 2
            },
            RoomInfo {
                uuid: "duplicate".into(),
                name: "Z".into(),
                item_count: 0
            },
            RoomInfo {
                uuid: "quoted".into(),
                name: "has 12hTF=\"x\"".into(),
                item_count: 0
            },
        ]
    );
    assert_eq!(xml.as_slice(), before);
}

#[test]
fn bom_unicode_and_invalid_utf8_attributes_retain_legacy_acceptance() {
    let mut xml = b"\xef\xbb\xbf<Root>\r\n".to_vec();
    xml.extend_from_slice("<C Type=\"Place\" U=\"室\" Title=\"Grüße\"/>".as_bytes());
    xml.extend_from_slice(
        b"<C Type=\"Place\" U=\"bad\" Title=\"\xff\"/><C Type=\"Place\" U=\"valid\" Title=\"A\" Future=\"\xff\"/>",
    );
    xml.extend_from_slice(b"<C Type=\"Place\" U=\"\xff\" Title=\"Missing\"/></Root>");
    let before = xml.clone();
    assert_eq!(
        parse_rooms(&xml).unwrap(),
        [
            RoomInfo {
                uuid: "valid".into(),
                name: "A".into(),
                item_count: 0
            },
            RoomInfo {
                uuid: "室".into(),
                name: "Grüße".into(),
                item_count: 0
            },
        ]
    );
    assert_eq!(xml, before);
}

#[test]
fn permissive_eof_is_not_validation_and_reader_failures_are_typed() {
    for xml in [b"".as_slice(), b"not XML", b"<ControlList/>"] {
        assert!(parse_rooms(xml).unwrap().is_empty());
    }
    assert_eq!(
        parse_rooms(br#"<C Type="Place" U="open" Title="Open">"#).unwrap(),
        [RoomInfo {
            uuid: "open".into(),
            name: "Open".into(),
            item_count: 0
        }]
    );
    let error = parse_rooms(b"<ControlList><C></ControlList>").unwrap_err();
    let RoomInventoryError::Xml { source } = &error;
    assert!(matches!(source, quick_xml::Error::IllFormed(_)));
    assert_eq!(error.to_string(), source.to_string());
    assert!(error.source().unwrap().is::<quick_xml::Error>());
    assert!(source.to_string().contains("expected `</C>`"));
}

#[test]
fn equal_titles_remain_present_without_promising_a_tie_break_order() {
    let rooms = parse_rooms(
        br#"<Root><C Type="Place" U="b" Title="Same"/><C Type="Place" U="a" Title="Same"/><IoData Pr="b"/></Root>"#,
    ).unwrap();
    assert_eq!(rooms.len(), 2);
    assert!(rooms.iter().all(|room| room.name == "Same"));
    assert_eq!(
        rooms
            .iter()
            .find(|room| room.uuid == "b")
            .unwrap()
            .item_count,
        1
    );
    assert_eq!(
        rooms
            .iter()
            .find(|room| room.uuid == "a")
            .unwrap()
            .item_count,
        0
    );
}
