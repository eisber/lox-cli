use lox_core::configuration::users::{UserInfo, UserInventoryError, parse_users};
use std::error::Error;

#[test]
fn raw_user_fields_defaults_and_duplicate_identities_keep_encounter_order() {
    let xml = br#"<Root><C Type="User" Title="Z&amp;&#65;&custom;" U="same" NFCArr=" " Desc="_A__&foo_bar;"/>
<C Type="Page"><C Type="User" Title="First" Title="Second" U="same" NFCArr="0">
<C Type="User"/></C></C><C Type="User"/>
<C Type="user"/><C Type="UserGroup"/><C Type="Us&#101;r"/><Other Type="User"/></Root>"#;
    let before = xml.to_vec();
    let rows: Vec<UserInfo> = parse_users(xml).unwrap();
    assert_eq!(
        rows,
        [
            UserInfo {
                name: "Z&amp;&#65;&custom;".into(),
                uuid: "same".into(),
                nfc: true,
                description: " A  &foo bar;".into()
            },
            UserInfo {
                name: "First".into(),
                uuid: "same".into(),
                nfc: true,
                description: String::new()
            },
            UserInfo {
                name: String::new(),
                uuid: String::new(),
                nfc: false,
                description: String::new()
            },
            UserInfo {
                name: String::new(),
                uuid: String::new(),
                nfc: false,
                description: String::new()
            }
        ]
    );
    assert_eq!(xml.as_slice(), before);
}

#[test]
fn utf8_attribute_errors_and_quoted_digit_text_do_not_invoke_dom_repair() {
    let mut xml = b"\xef\xbb\xbf<Root>\r\n".to_vec();
    xml.extend_from_slice("<C Type=\"User\" Title=\"Jörg_東京\" Desc=\"Grüße_世界\"/>".as_bytes());
    xml.extend_from_slice(b"<C Type=\"User\" Title=\"\xff\" Title=\"fallback\" U=\"\xff\" NFCArr=\"\xff\" Desc=\"\xff\"/>");
    xml.extend_from_slice(b"<C Type=\"\xff\" Type=\"User\"/>");
    xml.extend_from_slice(b"<C Type=\"User\" broken Title=\"after malformed\"/>");
    xml.extend_from_slice(br#"<C Type="User" Title='has 12hTF="x"' 12hTF="true"/></Root>"#);
    let before = xml.clone();
    let rows = parse_users(&xml).unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].name, "Jörg_東京");
    assert_eq!(rows[0].description, "Grüße 世界");
    assert_eq!(
        rows[1],
        UserInfo {
            name: String::new(),
            uuid: String::new(),
            nfc: false,
            description: String::new()
        }
    );
    assert_eq!(rows[2].name, "after malformed");
    assert_eq!(rows[3].name, "has 12hTF=\"x\"");
    assert_eq!(xml, before);
}

#[test]
fn incomplete_eof_is_permissive_but_reader_errors_are_typed_and_atomic() {
    for xml in [b"".as_slice(), b"not XML", b"<Root>"] {
        assert!(parse_users(xml).unwrap().is_empty());
    }
    assert_eq!(parse_users(b"<Root><C Type=\"User\">").unwrap().len(), 1);
    let error = parse_users(b"<Root><C Type=\"User\"></Root>").unwrap_err();
    let UserInventoryError::Xml { source } = &error;
    assert!(matches!(source, quick_xml::Error::IllFormed(_)));
    assert_eq!(error.to_string(), source.to_string());
    assert!(error.source().unwrap().is::<quick_xml::Error>());
    assert_eq!(
        source.to_string(),
        "ill-formed document: expected `</C>`, but `</Root>` was found"
    );
}
