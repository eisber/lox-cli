use lox_core::configuration::devices::{
    DeviceBus, DeviceInfo, DeviceInventoryError, parse_devices,
};
use std::error::Error;

#[test]
fn raw_inventory_is_ordered_typed_and_independent_of_device_hierarchy() {
    let xml = br#"<Root><C Type="TreeExtension">
<C Type="NetworkDevice" Title="Network" Serial="ignored" SubType="19" MAC="" Addr="A&amp;B"/>
<C Type="Page"><C Type="LoxAIRDevice" Title="Air" Serial="ignored" SubType="+7">
<C Type="TreeDevice" Title='has 12hTF="x"' 12hTF="true" Serial="S" SubType="32784"/>
</C></C></C>
<C Type="AirExtension"><C Type="TreeDevice" Title="First" Title="Ignored" Serial="" SubType="99999"/></C>
<C Type="TreeDevice"/><C Type="TreeDevice"/>
<C Type="TreeAsensor"/><C Type="treedevice"/><C Type="Tree&#68;evice"/>
<Other Type="NetworkDevice"/></Root>"#;
    let before = xml.to_vec();
    let rows: Vec<DeviceInfo> = parse_devices(xml).unwrap();
    assert_eq!(
        rows.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        ["Network", "Air", "has 12hTF=\"x\"", "First", "", ""]
    );
    assert_eq!(
        rows.iter().map(|d| d.bus).collect::<Vec<_>>(),
        [
            DeviceBus::Network,
            DeviceBus::Air,
            DeviceBus::Tree,
            DeviceBus::Tree,
            DeviceBus::Tree,
            DeviceBus::Tree
        ]
    );
    assert_eq!(rows[0].sub_type, Some(19));
    assert_eq!(rows[0].type_label, "Network device");
    assert_eq!(rows[0].serial, None);
    assert_eq!(rows[0].mac.as_deref(), Some(""));
    assert_eq!(rows[0].address.as_deref(), Some("A&amp;B"));
    assert_eq!(rows[1].type_label, "Nano IO Air");
    assert_eq!(rows[1].serial, None);
    assert_eq!(rows[2].serial.as_deref(), Some("S"));
    assert_eq!(rows[2].type_label, "Room climate sensor");
    assert_eq!(rows[2].mac, None);
    assert_eq!(rows[2].address, None);
    assert_eq!(rows[3].serial.as_deref(), Some(""));
    assert_eq!(rows[3].type_label, "Unknown (99999)");
    assert_eq!(rows[4].type_label, "Unknown");
    assert_eq!(rows[4], rows[5]);
    assert_eq!(xml.as_slice(), before);
}

#[test]
fn unreadable_and_malformed_attributes_keep_legacy_defaults_and_recovery() {
    let mut xml = b"\xef\xbb\xbf<Root>\r\n".to_vec();
    xml.extend_from_slice(b"<C Type=\"TreeDevice\" Title=\"\xff\" Title=\"fallback\" Serial=\"\xff\" SubType=\"\xff\"/>");
    xml.extend_from_slice(b"<C Type=\"NetworkDevice\" MAC=\"\xff\" Addr=\"\xff\"/>");
    xml.extend_from_slice(b"<C Type=\"\xff\" Type=\"TreeDevice\"/>");
    xml.extend_from_slice(b"<C Type=\"TreeDevice\" broken Title=\"A&amp;&#65;&custom;\"/>");
    for sub in ["", "-1", " 7", "7 ", "7&amp;", "4294967296"] {
        xml.extend_from_slice(format!("<C Type=\"TreeDevice\" SubType=\"{sub}\"/>").as_bytes());
    }
    xml.extend_from_slice(b"<C Type=\"LoxAIRDevice\" SubType=\"0007\"/>");
    xml.extend_from_slice(b"<C Type=\"TreeDevice\" SubType=\"4294967295\"/>");
    let before = xml.clone();
    let rows = parse_devices(&xml).unwrap();
    assert_eq!(rows.len(), 11);
    assert_eq!(rows[0].name, "");
    assert_eq!(rows[0].serial, None);
    assert_eq!(rows[0].sub_type, None);
    assert_eq!(rows[1].mac, None);
    assert_eq!(rows[1].address, None);
    assert_eq!(rows[2].name, "A&amp;&#65;&custom;");
    assert!(rows[3..9].iter().all(|d| d.sub_type.is_none()));
    assert_eq!(rows[9].sub_type, Some(7));
    assert_eq!(rows[10].type_label, "Unknown (4294967295)");
    assert_eq!(xml, before);
}

#[test]
fn incomplete_eof_is_permissive_but_reader_errors_are_typed_and_atomic() {
    for bytes in [b"".as_slice(), b"not XML", b"<Root>"] {
        assert!(parse_devices(bytes).unwrap().is_empty());
    }
    assert_eq!(
        parse_devices(b"<Root><C Type=\"TreeDevice\">")
            .unwrap()
            .len(),
        1
    );
    let error = parse_devices(b"<Root><C Type=\"TreeDevice\"></Root>").unwrap_err();
    let DeviceInventoryError::Xml { source } = &error;
    assert!(matches!(source, quick_xml::Error::IllFormed(_)));
    assert_eq!(error.to_string(), source.to_string());
    assert!(error.source().unwrap().is::<quick_xml::Error>());
    assert_eq!(
        source.to_string(),
        "ill-formed document: expected `</C>`, but `</Root>` was found"
    );
}
