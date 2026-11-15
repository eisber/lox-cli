use lox_core::configuration::{
    archive::{ArchiveError, compress_loxcc},
    runtime_overlay::{OverlayError, merge_overlays, three_way, validate_document},
};
use std::{collections::BTreeMap, error::Error};

const A: &str = "abcdef00-0000-0000-ffff000000000001";
const B: &str = "abcdef00-0000-0000-ffff000000000002";

fn overlay(id: &str, xml: &[u8]) -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([(format!("{id}.LoxCC"), compress_loxcc(xml))])
}

#[test]
fn replacement_and_empty_page_insertion_preserve_exact_source_spans() {
    assert_eq!(
        merge_overlays(b"\xff<", &BTreeMap::new()).unwrap(),
        b"\xff<"
    );
    let before = format!(
        "\u{feff}<?xml version=\"1.0\"?>\r\n<Root><!--keep--><C Type=\"And\" U=\"{A}\" Title=\"literal\r\nvalue\"/><C U=\"page\" Type=\"Page\" SpNo=\"1\"/></Root>\r\n"
    );
    let replacement = format!(r#"<C U="{A}" Type="And" Title="changed"/>"#);
    let rule = format!(r#"<C Type="AutopilotRule" U="{B}"/>"#);
    let mut overlays = overlay(B, rule.as_bytes());
    overlays.extend(overlay(A, replacement.as_bytes()));
    let expected = before
        .replace(
            &format!("<C Type=\"And\" U=\"{A}\" Title=\"literal\r\nvalue\"/>"),
            &replacement,
        )
        .replace(
            r#"<C U="page" Type="Page" SpNo="1"/>"#,
            &format!(r#"<C U="page" Type="Page" SpNo="1">{rule}</C>"#),
        );
    assert_eq!(
        merge_overlays(before.as_bytes(), &overlays).unwrap(),
        expected.as_bytes()
    );
}

#[test]
fn typed_refusals_retain_archive_and_xml_causes_without_mutating_inputs() {
    let base = b"<Root/>";
    let data = BTreeMap::from([(format!("{A}.LoxCC"), vec![0; 15])]);
    let original = data.clone();
    let error = merge_overlays(base, &data).unwrap_err();
    match &error {
        OverlayError::FragmentRead { source, .. } => assert!(matches!(
            source.as_ref(),
            OverlayError::Archive {
                source: ArchiveError::HeaderTooShort { actual: 15 }
            }
        )),
        other => panic!("unexpected failure: {other:?}"),
    }
    assert!(
        error
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<ArchiveError>()
    );
    assert_eq!(data, original);
    let malformed = validate_document(b"<Root a=\"1\" a=\"2\"/>").unwrap_err();
    assert!(matches!(malformed, OverlayError::Attribute { .. }));
    assert!(
        malformed
            .source()
            .unwrap()
            .is::<quick_xml::events::attributes::AttrError>()
    );
    let mut corrupt = overlay(A, format!(r#"<C Type="And" U="{A}"/>"#).as_bytes());
    corrupt.values_mut().next().unwrap()[12] ^= 1;
    assert!(matches!(
        merge_overlays(base, &corrupt),
        Err(OverlayError::FragmentRead { source, .. }) if matches!(*source, OverlayError::FragmentIntegrity)
    ));
}

#[test]
fn uuid_aliases_and_nested_fragment_overlaps_are_atomic_refusals() {
    let base = format!(r#"<Root><C Type="And" U="{A}"/><C Type="Or" U="{B}"/></Root>"#);
    let alias = "ABCDEF00-0000-0000-FFFF-000000000001";
    let mut aliases = overlay(A, format!(r#"<C Type="And" U="{A}"/>"#).as_bytes());
    aliases.extend(overlay(
        alias,
        format!(r#"<C Type="And" U="{alias}"/>"#).as_bytes(),
    ));
    assert!(matches!(
        merge_overlays(base.as_bytes(), &aliases),
        Err(OverlayError::DuplicateOverlay { .. })
    ));
    let mut nested = overlay(
        A,
        format!(r#"<C Type="And" U="{A}"><C Type="Or" U="{B}"/></C>"#).as_bytes(),
    );
    nested.extend(overlay(B, format!(r#"<C Type="Or" U="{B}"/>"#).as_bytes()));
    assert!(matches!(
        merge_overlays(base.as_bytes(), &nested),
        Err(OverlayError::FragmentOverlap { .. })
    ));
    assert!(validate_document(base.as_bytes()).is_ok());
}

#[test]
fn three_way_preserves_selected_shell_and_refuses_concurrent_edits() {
    let base = br#"<Root><C Type="And" U="a" V="0"/><C Type="Or" U="b" V="0"/></Root>"#;
    let local = b"\xef\xbb\xbf<Root>\r\n<C V=\"1\" U=\"a\" Type=\"And\"></C><C Type=\"Or\" U=\"b\" V=\"0\"/>\r\n</Root>";
    let live = br#"<Root><C Type="And" U="a" V="0"/><C Type="Or" U="b" V="2"/></Root>"#;
    let expected = br#"<Root><C V="1" U="a" Type="And"></C><C Type="Or" U="b" V="2"/></Root>"#;
    assert_eq!(three_way(base, local, live).unwrap(), expected);
    let conflict = br#"<Root><C Type="And" U="a" V="3"/><C Type="Or" U="b" V="0"/></Root>"#;
    assert!(
        matches!(three_way(base, local, conflict), Err(OverlayError::ShellConflict { id }) if id == "a")
    );
    let deleted = br#"<Root><C Type="Or" U="b" V="0"/></Root>"#;
    assert!(
        matches!(three_way(base, deleted, local), Err(OverlayError::ControlConflict { id }) if id == "a")
    );
    assert_eq!(three_way(base, deleted, base).unwrap(), deleted);
}
