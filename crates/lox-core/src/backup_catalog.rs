//! Legacy backup filename and FTP LIST catalog interpretation, without transport.
//!
//! This permissive observation is not filename/path, file-type or calendar validation.

/// Owned metadata observed from an existing backup filename and listing size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInfo {
    pub filename: String,
    pub version: u32,
    pub timestamp: String,
    pub size: u64,
}

/// Observe a case-sensitive `sps_<u32>_<timestamp>.zip` filename.
///
/// Timestamps contain at least fourteen ASCII digits, with no calendar validation
/// or upper length bound. The original filename and full timestamp are retained.
pub fn parse_backup_name(name: &str, size: u64) -> Option<BackupInfo> {
    let stem = name.strip_suffix(".zip")?;
    let parts: Vec<&str> = stem.splitn(3, '_').collect();
    if parts.len() != 3 || parts[0] != "sps" {
        return None;
    }
    let version = parts[1].parse::<u32>().ok()?;
    let timestamp = parts[2].to_string();
    if timestamp.len() < 14 || !timestamp.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(BackupInfo {
        filename: name.to_string(),
        version,
        timestamp,
        size,
    })
}

/// Project the existing permissive FTP LIST profile, omitting unrecognized rows.
///
/// Rows need four whitespace-separated fields; the last is the filename and
/// field index four is the optional `u64` size (invalid or absent means zero).
/// No permission/file-type filtering is performed. Duplicates remain, ordered
/// by descending raw timestamp string; equal timestamps retain input order.
pub fn parse_backup_listing(listing: &[String]) -> Vec<BackupInfo> {
    let mut backups = Vec::new();
    for line in listing {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        let name = fields.last().unwrap();
        if !name.ends_with(".zip") || !name.starts_with("sps_") {
            continue;
        }
        let size = fields
            .get(4)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        if let Some(info) = parse_backup_name(name, size) {
            backups.push(info);
        }
    }
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    backups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_filename_observation_is_not_calendar_validation() {
        let name = "sps_+0007_20261399259999999.zip";
        let info = parse_backup_name(name, u64::MAX).unwrap();
        assert_eq!(info.filename, name);
        assert_eq!(info.version, 7);
        assert_eq!(info.timestamp, "20261399259999999");
        assert_eq!(info.size, u64::MAX);
        for invalid in [
            "SPS_7_20260101120000.zip",
            "sps_7_20260101120000.ZIP",
            "sps_-1_20260101120000.zip",
            "sps_ 7_20260101120000.zip",
            "sps_4294967296_20260101120000.zip",
            "sps_7_2026010112000.zip",
            "sps_7_２０２６0101120000.zip",
            "sps_7_20260101120000_extra.zip",
        ] {
            assert!(parse_backup_name(invalid, 0).is_none(), "{invalid}");
        }
    }

    #[test]
    fn legacy_list_row_projection_preserves_permissive_size_and_type_policy() {
        let name = "sps_1_20260101120000.zip";
        let listing = [
            format!("a b {name}"),
            format!("a b c {name}"),
            format!("a b c d {name}"),
            format!("drwx x x x +00042 {name}"),
            format!("lrwx x x x 18446744073709551616 {name}"),
            format!("-\tx\u{2003}x x 17 Jan 1 {name}  "),
            format!("- x x x 99 {name} trailing"),
        ];
        let got = parse_backup_listing(&listing);
        assert_eq!(
            got.iter().map(|b| b.size).collect::<Vec<_>>(),
            [0, 0, 42, 0, 17]
        );
        assert!(got.iter().all(|b| b.filename == name));
    }

    #[test]
    fn catalog_sort_is_stable_raw_lexical_and_retains_duplicates() {
        let listing = [
            "a b c sps_1_20260101120000.zip",
            "a b c sps_2_100000000000000.zip",
            "a b c sps_3_99999999999999.zip",
            "a b c sps_4_20260101120000.zip",
            "a b c sps_1_20260101120000.zip",
        ]
        .map(String::from);
        let got = parse_backup_listing(&listing);
        assert_eq!(
            got.iter().map(|b| b.version).collect::<Vec<_>>(),
            [3, 1, 4, 1, 2]
        );
        assert_eq!(got[0].timestamp, "99999999999999");
        assert_eq!(got[4].timestamp, "100000000000000");
    }
}
