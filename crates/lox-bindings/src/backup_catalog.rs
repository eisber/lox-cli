//! Owned observations of supplied backup names and listing rows, without I/O.

use lox_core::backup_catalog as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct BackupInfo {
    pub filename: String,
    pub version: u32,
    pub timestamp: String,
    pub size: u64,
}

impl From<core::BackupInfo> for BackupInfo {
    fn from(value: core::BackupInfo) -> Self {
        Self {
            filename: value.filename,
            version: value.version,
            timestamp: value.timestamp,
            size: value.size,
        }
    }
}

/// Observe a supplied name, not calendar, path or file-type validity.
#[uniffi::export]
pub fn parse_backup_name(name: String, size: u64) -> Option<BackupInfo> {
    core::parse_backup_name(&name, size).map(Into::into)
}

/// Observe supplied listing rows with Core's permissive sizes and stable raw ordering.
#[uniffi::export]
pub fn parse_backup_listing(listing: Vec<String>) -> Vec<BackupInfo> {
    core::parse_backup_listing(&listing)
        .into_iter()
        .map(Into::into)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_owned_metadata_preserves_unsigned_fields_and_raw_text() {
        let name = "sps_4294967295_20261399259999999.zip";
        let expected = BackupInfo::from(core::parse_backup_name(name, u64::MAX).unwrap());
        let mut actual = parse_backup_name(name.to_owned(), u64::MAX).unwrap();
        assert_eq!(actual, expected);
        actual.filename.clear();
        assert_eq!(parse_backup_name(name.to_owned(), u64::MAX), Some(expected));
        assert_eq!(
            parse_backup_name("SPS_0_20260101120000.zip".into(), 0),
            None
        );
        assert_eq!(
            parse_backup_name("sps_+0000_00000000000000.zip".into(), 0)
                .unwrap()
                .version,
            0
        );
    }

    #[test]
    fn owned_list_conversion_preserves_core_projection_and_duplicate_order() {
        let listing = [
            "d a b c 18446744073709551615 sps_3_99999999999999.zip",
            "l a b c 18446744073709551616 sps_1_20261399259999.zip",
            "? a b c invalid sps_4_20261399259999.zip",
            "a b c sps_1_20261399259999.zip",
            "a b c sps_2_100000000000000.zip",
        ]
        .map(String::from)
        .to_vec();
        let expected: Vec<BackupInfo> = core::parse_backup_listing(&listing)
            .into_iter()
            .map(Into::into)
            .collect();
        let mut actual = parse_backup_listing(listing);
        assert_eq!(actual, expected);
        assert_eq!(
            actual.iter().map(|v| v.version).collect::<Vec<_>>(),
            [3, 1, 4, 1, 2]
        );
        actual[0].timestamp.clear();
        assert_ne!(actual, expected);
        assert!(parse_backup_listing(vec![]).is_empty());
    }
}
