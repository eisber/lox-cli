use lox_core::configuration::custom_listing::{ListingError, parse_custom_listing};

fn rows(input: &[&str]) -> Vec<String> {
    input.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn strict_custom_listing_keeps_raw_names_and_legacy_quirks() {
    assert_eq!(parse_custom_listing(&[]).unwrap(), Vec::<String>::new());
    assert_eq!(
        parse_custom_listing(&rows(&[
            "d /ignored.txt",
            "l ignored\\path.txt",
            "- a.ℓoxcc",
            "- a.loxccé",
            "- z.loXcC",
            "- A.LoxCC",
            "- z.loXcC",
            "D upper.loxcc",
            "L upper-link.loxcc",
            " d leading.loxcc",
            " l leading-link.loxcc",
            "not numeric\u{2003}unicode.loxcc",
            "- nul\0.loxcc",
            "- 12345678-1234-1234-1234567890123456.LoxCC",
            "- not-a-uuid.loxcc",
            "- first.loxcc last-token",
        ]))
        .unwrap(),
        [
            "12345678-1234-1234-1234567890123456.LoxCC",
            "A.LoxCC",
            "leading-link.loxcc",
            "leading.loxcc",
            "not-a-uuid.loxcc",
            "nul\0.loxcc",
            "unicode.loxcc",
            "upper-link.loxcc",
            "upper.loxcc",
            "z.loXcC",
            "z.loXcC",
        ]
    );
}

#[test]
fn strict_custom_listing_fails_atomically_at_first_bad_row_without_payload() {
    for (bad, expected) in [
        ("", ListingError::Malformed { row: 2 }),
        ("\u{2003}\t ", ListingError::Malformed { row: 2 }),
        ("d private.loxcc", ListingError::Unsupported { row: 2 }),
        ("l private.loxcc", ListingError::Unsupported { row: 2 }),
        ("- private/path.loxcc", ListingError::Unsupported { row: 2 }),
        (
            "- private\\path.loxcc",
            ListingError::Unsupported { row: 2 },
        ),
    ] {
        let error =
            parse_custom_listing(&rows(&["- valid.loxcc", "- unrelated", bad, ""])).unwrap_err();
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains("private"));
        assert!(std::error::Error::source(&error).is_none());
    }
    assert_eq!(
        parse_custom_listing(&rows(&["", "- x/y.loxcc"])),
        Err(ListingError::Malformed { row: 0 })
    );
    assert_eq!(
        parse_custom_listing(&rows(&["- ignored", "\t"])),
        Err(ListingError::Malformed { row: 1 })
    );
}
