//! Owned HTML statistics index rows; all parsing semantics remain in Core.

use lox_core::statistics as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct StatEntry {
    pub file: String,
    pub uuid: String,
    pub group: Option<u32>,
    pub month: String,
    pub name: String,
    pub label: String,
}

impl From<core::StatEntry> for StatEntry {
    fn from(value: core::StatEntry) -> Self {
        Self {
            file: value.file,
            uuid: value.uuid,
            group: value.group,
            month: value.month,
            name: value.name,
            label: value.label,
        }
    }
}

/// Parse an acquired HTML index, preserving Core's omissions, order and duplicates.
#[uniffi::export]
pub fn parse_statistics_index(html: String) -> Vec<StatEntry> {
    core::parse_index(&html)
        .into_iter()
        .map(Into::into)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_rows_preserve_core_index_semantics() {
        let input = include_str!("../tests/fixtures/statistics_index.html");
        let expected: Vec<StatEntry> = core::parse_index(input)
            .into_iter()
            .map(Into::into)
            .collect();
        let mut actual = parse_statistics_index(input.to_owned());
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 7);
        assert_eq!(actual[0].group, None);
        assert_eq!(actual[1].group, Some(0));
        assert_eq!(actual[2].group, Some(u32::MAX));
        assert_eq!(actual[3], actual[4]);
        actual[0].name.push_str("changed");
        assert_ne!(actual, expected);
    }

    #[test]
    fn empty_malformed_and_embedded_nul_are_infallible() {
        for input in ["", "<LI><a href=\"bad\">bad</a>", "<li>broken", "\0é"] {
            assert!(parse_statistics_index(input.to_owned()).is_empty());
        }
        let input = "<li><a href=\"01234567-89ab-cdef-0123456789abcdef.000000.xml\">raw\0é e\u{301}\r\nvalue</a>";
        let rows = parse_statistics_index(input.to_owned());
        assert_eq!(rows[0].name, "raw\0é e\u{301}\r\nvalue");
        assert_eq!(rows[0].month, "0000-00");
    }
}
