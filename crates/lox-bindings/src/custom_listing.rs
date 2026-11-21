//! Offline interpretation of an already successful listing; transport stays caller-owned.

use lox_core::configuration::custom_listing as core;

/// Data-free errors retain the zero-based index into the supplied rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum CustomListingError {
    #[error("Malformed /sys/custom FTP listing")]
    Malformed { row: u64 },
    #[error("Unsupported runtime overlay listing")]
    Unsupported { row: u64 },
}

impl From<core::ListingError> for CustomListingError {
    fn from(error: core::ListingError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match error {
            core::ListingError::Malformed { row } => Self::Malformed { row: row as u64 },
            core::ListingError::Unsupported { row } => Self::Unsupported { row: row as u64 },
        }
    }
}

/// Return Core's sorted raw names with duplicates, or an atomic typed error.
/// This neither acquires a listing nor validates fragment filenames.
#[uniffi::export]
pub fn parse_custom_listing(rows: Vec<String>) -> Result<Vec<String>, CustomListingError> {
    core::parse_custom_listing(&rows).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_names_preserve_core_order_duplicates_and_legacy_quirks() {
        let rows = [
            "d ignored/path.txt",
            "l ignored\\path.txt",
            "- false.ℓoxcc",
            "- false.loxccé",
            "- z.loXcC",
            "- A.LoxCC",
            "- z.loXcC",
            "D upper.loxcc",
            "L upper-link.loxcc",
            " d leading.loxcc",
            " l leading-link.loxcc",
            "nonnumeric\u{2003}unicode.loxcc",
            "- nul\0.loxcc",
            "- not-a-uuid.loxcc",
            "- first.loxcc last-token",
        ]
        .map(String::from)
        .to_vec();
        let expected = core::parse_custom_listing(&rows).unwrap();
        let mut actual = parse_custom_listing(rows.clone()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            actual,
            [
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
        actual[0].clear();
        assert_eq!(parse_custom_listing(rows).unwrap(), expected);
        assert!(parse_custom_listing(vec![]).unwrap().is_empty());
    }

    #[test]
    fn first_error_is_atomic_typed_and_does_not_retain_row_contents() {
        for (bad, expected) in [
            ("", CustomListingError::Malformed { row: 2 }),
            ("\u{2003}\t ", CustomListingError::Malformed { row: 2 }),
            (
                "d private.loxcc",
                CustomListingError::Unsupported { row: 2 },
            ),
            (
                "l private.loxcc",
                CustomListingError::Unsupported { row: 2 },
            ),
            (
                "- private/path.loxcc",
                CustomListingError::Unsupported { row: 2 },
            ),
            (
                "- private\\path.loxcc",
                CustomListingError::Unsupported { row: 2 },
            ),
        ] {
            let rows = ["- valid.loxcc", "- unrelated", bad, ""]
                .map(String::from)
                .to_vec();
            let error = parse_custom_listing(rows.clone()).unwrap_err();
            assert_eq!(error, expected);
            assert_eq!(error, core::parse_custom_listing(&rows).unwrap_err().into());
            assert!(!format!("{error:?} {error}").contains("private"));
            assert!(std::error::Error::source(&error).is_none());
        }
        assert_eq!(
            parse_custom_listing(vec!["".into(), "- x/y.loxcc".into()]),
            Err(CustomListingError::Malformed { row: 0 })
        );
    }
}
