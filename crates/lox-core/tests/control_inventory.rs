use lox_core::configuration::controls::{ControlInfo, ControlInventoryError, parse_controls};
use std::error::Error;

#[test]
fn pending_state_is_not_parent_scoped_or_blacklisted() {
    let xml = br#"<Root>
<C Type="Page" U="page" Title="Page"/><Other><IoData Pr="page-room"/></Other>
<C Type="TreeDevice" U="device" Title="Device"><IoData/><IoData Pr="ignored"/></C>
<C Type="Switch" U="outer" Title="Outer">
  <C Type="Other" U="inner" Title="Inner"/><IoData Pr="inner-room"/>
  <IoData Pr="not-outer"/>
</C>
<C Type="Switch" U="pending" Title="Pending"/>
<C Type="Place" U="r" Title="Room"/><C Type="Category" U="c" Title="Category"/>
<C Type="Switch" Title="No ID"/><C Title="No Type"/><IoData Pr="r" Cr="c"/>
<C Type="Switch" U="lost" Title="Lost"/><C Type="Switch" U="no-title"/><IoData/>
</Root>"#;
    let rows: Vec<ControlInfo> = parse_controls(xml, None, None).unwrap();
    let actual: Vec<_> = rows
        .iter()
        .map(|c| (c.control_type.as_str(), c.uuid.as_str(), c.room.as_str()))
        .collect();
    assert_eq!(
        actual,
        [
            ("Other", "inner", "inner-room"),
            ("Page", "page", "page-room"),
            ("Switch", "pending", "Room"),
            ("TreeDevice", "device", "")
        ]
    );
    assert_eq!(rows[2].category, "Category");
    let rows = parse_controls(xml, Some("sWiTcH"), None).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!((&*rows[0].uuid, &*rows[0].room), ("outer", "inner-room"));
    assert_eq!((&*rows[1].uuid, &*rows[1].room), ("pending", "Room"));
}

#[test]
fn raw_resolution_filters_empty_values_and_stable_duplicates() {
    let xml = r#"<Root>
<C Type="Switch" U="second" Title="Same"><IoData Pr="r" Cr="c"/></C>
<C Type="Switch" U="first" U="ignored" Title="Same" Title="Ignored"><IoData Pr="r"/></C>
<C Type="Switch" U="first" Title="Same"><IoData Pr="raw&amp;id" Cr="unknown"/></C>
<C Type="" U="" Title=""><IoData/></C>
<C Type="switch" U="lower" Title="Z"><IoData/></C>
<C Type="Sw&#105;tch" U="entity" Title='has 12hTF="x"'><IoData/></C>
<C Type="Ä" U="unicode" Title=" A&amp;&#65;&custom; "><IoData/></C>
<C Type="Place" U="r" Title="Old"/><C Type="Place" U="r" Title="KÜCHE"/>
<C Type="Category" U="c" Title="Old"/><C Type="Category" U="c" Title=""/>
<C Type="Place" U="" Title="Default"/><C Type="Category" U="" Title="Empty ID"/>
</Root>"#;
    let before = xml.as_bytes().to_vec();
    let rows = parse_controls(xml.as_bytes(), None, None).unwrap();
    assert_eq!(rows.len(), 7);
    assert_eq!(
        (&*rows[0].control_type, &*rows[0].uuid, &*rows[0].title),
        ("", "", "")
    );
    assert_eq!(
        (&*rows[0].room, &*rows[0].category),
        ("Default", "Empty ID")
    );
    let switches = parse_controls(xml.as_bytes(), Some("SWITCH"), None).unwrap();
    assert_eq!(
        switches.iter().map(|c| c.uuid.as_str()).collect::<Vec<_>>(),
        ["second", "first", "first", "lower"]
    );
    assert_eq!(switches[0].room, "KÜCHE");
    assert_eq!(switches[0].category, "");
    assert_eq!(switches[2].room, "raw&amp;id");
    assert_eq!(switches[2].category, "unknown");
    assert_eq!(
        parse_controls(xml.as_bytes(), None, Some("küc"))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        parse_controls(xml.as_bytes(), None, Some(""))
            .unwrap()
            .len(),
        7
    );
    assert_eq!(
        parse_controls(xml.as_bytes(), Some(""), None)
            .unwrap()
            .len(),
        1
    );
    assert!(
        parse_controls(xml.as_bytes(), Some("ä"), None)
            .unwrap()
            .is_empty()
    );
    assert!(
        parse_controls(xml.as_bytes(), Some("Switch"), Some("absent"))
            .unwrap()
            .is_empty()
    );
    assert_eq!(rows[1].title, "has 12hTF=\"x\"");
    assert_eq!(rows[6].title, " A&amp;&#65;&custom; ");
    assert_eq!(xml.as_bytes(), before);
}

#[test]
fn borrowed_bytes_keep_bom_unicode_leniency_and_typed_reader_errors() {
    let mut xml = b"\xef\xbb\xbf<Root>\r\n".to_vec();
    xml.extend_from_slice("<C Type=\"Switch\" U=\"室\" Title=\"Grüße\"><IoData/></C>".as_bytes());
    xml.extend_from_slice(b"<C Type=\"Switch\" U=\"bad\" Title=\"\xff\"/><IoData/>");
    xml.extend_from_slice(
        b"<C Type=\"Switch\" U=\"valid\" Title=\"A\" Extra=\"\xff\"/><IoData Pr=\"\xff\"/>",
    );
    xml.extend_from_slice(b"<C Type=\"Switch\" U=\"\xff\" Title=\"Unreadable ID\"/><IoData/>");
    let before = xml.clone();
    let rows = parse_controls(&xml, None, None).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!((&*rows[0].uuid, &*rows[0].room), ("valid", ""));
    assert_eq!((&*rows[1].uuid, &*rows[1].title), ("室", "Grüße"));
    assert_eq!(xml, before);
    for bytes in [b"".as_slice(), b"not XML", b"<Root>"] {
        assert!(parse_controls(bytes, None, None).unwrap().is_empty());
    }
    let error = parse_controls(b"<Root><C></Root>", None, None).unwrap_err();
    let ControlInventoryError::Xml { source } = &error;
    assert!(matches!(source, quick_xml::Error::IllFormed(_)));
    assert_eq!(error.to_string(), source.to_string());
    assert!(error.source().unwrap().is::<quick_xml::Error>());
    assert_eq!(
        source.to_string(),
        "ill-formed document: expected `</C>`, but `</Root>` was found"
    );
}
