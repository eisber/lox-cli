use lox_core::configuration::summary::{
    ConfigSummary, ConfigSummaryError, ControlEntry, parse_config_summary,
};
use std::error::Error;

#[test]
fn typed_summary_retains_raw_metadata_identity_and_direct_parent_relationships() {
    let bytes = br#"<ControlList Version="old">
<C Type="Document" Date="first"/><C Type="Document" Date="last"/>
<C Type="Place" U="r" Title="Old"/><C Type="Place" U="r" Title=" A&amp;&#65;&custom; "/>
<C Type="Category" U="c" Title="Old"/><C Type="Category" U="c" Title="New"/>
<C Type="User" Title="Ada"/><C Type="User" Title=""/><C Type="User" Title="Ada"/>
<C Type="Page" U="page" Title="Ignored"><IoData Pr="r"/></C>
<C Type="Switch" U="a" Title="Old"><IoData Pr="discarded"/></C>
<C Type="FutureBlock" U="a" Title=" A&amp;&#65; ">
  <IoData Pr="first"/><Other><IoData Pr="ignored"/></Other>
  <C Type="InputRef" U="ref" Title="Ignored"><IoData Pr="ignored"/></C>
  <C Type="Switch" U="b" Title="Nested"><IoData Pr="b-room"/></C>
  <IoData Pr="r"/><IoData/>
</C>
<IoData Pr="orphan"/><C Type="Switch" U="a" Title=""/>
<C Type="Switch" U="missing-title"/><C Type="Switch" Title="missing-id"/>
<C Type="Switch" U="" Title="Empty ID"/><C Type="Place" U="" Title=""/>
<C Type="Switch" U="first" U="second" Title="First" Title="Second"/>
<ControlList Version="new"/>
</ControlList>"#;
    let before = bytes.to_vec();
    let summary: ConfigSummary = parse_config_summary(bytes).unwrap();
    assert_eq!(
        (summary.version.as_str(), summary.date.as_str()),
        ("new", "last")
    );
    assert_eq!(summary.controls.len(), 4);
    assert_eq!(
        summary.controls["a"],
        ControlEntry {
            name: " A&amp;&#65; ".into(),
            control_type: "FutureBlock".into(),
            room_uuid: "r".into(),
        }
    );
    assert_eq!(summary.controls["b"].room_uuid, "b-room");
    assert_eq!(summary.controls[""].name, "Empty ID");
    assert_eq!(summary.controls["first"].name, "First");
    assert_eq!(summary.rooms.len(), 2);
    assert_eq!(
        summary.rooms[&summary.controls["a"].room_uuid],
        " A&amp;&#65;&custom; "
    );
    assert_eq!(summary.categories["c"], "New");
    assert_eq!(summary.users, ["Ada", "", "Ada"]);
    assert_eq!(bytes.as_slice(), before);
}

#[test]
fn borrowed_bytes_keep_bom_unicode_leniency_and_typed_reader_errors() {
    let mut xml = b"\xef\xbb\xbf<Root>\r\n".to_vec();
    xml.extend_from_slice("<C Type=\"Switch\" U=\"室\" Title=\"Grüße\"/>".as_bytes());
    xml.extend_from_slice(
        b"<C Type=\"Switch\" U=\"bad\" Title=\"\xff\"/><C Type=\"Switch\" U=\"ok\" Title=\"A\" Extra=\"\xff\"/>",
    );
    xml.extend_from_slice(
        b"<C Type=\"Place\" U=\"r\" Title=\"\xff\"/><C Type=\"User\" Title=\"\xff\"/>",
    );
    let before = xml.clone();
    let summary = parse_config_summary(&xml).unwrap();
    assert_eq!(summary.controls.len(), 2);
    assert_eq!(summary.controls["室"].name, "Grüße");
    assert_eq!(summary.controls["ok"].name, "A");
    assert!(summary.rooms.is_empty());
    assert!(summary.users.is_empty());
    assert_eq!(xml, before);
    for bytes in [b"".as_slice(), b"not XML", b"<Root>"] {
        assert!(parse_config_summary(bytes).unwrap().controls.is_empty());
    }
    let error = parse_config_summary(b"<Root><C></Root>").unwrap_err();
    let ConfigSummaryError::Xml { source } = &error;
    assert!(matches!(source, quick_xml::Error::IllFormed(_)));
    assert_eq!(error.to_string(), source.to_string());
    assert!(error.source().unwrap().is::<quick_xml::Error>());
}
