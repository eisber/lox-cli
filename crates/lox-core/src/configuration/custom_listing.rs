//! Strict interpretation of an already successful `/sys/custom` FTP LIST.
//!
//! This is not the permissive backup catalog or the runtime fragment filename
//! validator. The caller retains transport failures rather than passing an
//! assumed empty listing.

use std::fmt;

/// Atomic listing failure, with a zero-based index into the caller's rows.
/// Raw listing contents are not retained or rendered by this error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListingError {
    Malformed { row: usize },
    Unsupported { row: usize },
}

impl fmt::Display for ListingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Malformed { .. } => "Malformed /sys/custom FTP listing",
            Self::Unsupported { .. } => "Unsupported runtime overlay listing",
        })
    }
}

impl std::error::Error for ListingError {}

/// Return sorted raw overlay names, retaining duplicates, or no names on error.
///
/// Every row must contain a whitespace-delimited token. Only the last token's
/// ASCII-case-insensitive `.loxcc` suffix selects a row. Selected rows reject
/// raw lowercase `d`/`l` prefixes and filename slashes; leading whitespace and
/// uppercase type letters deliberately retain the existing producer's behavior.
/// No UUID, permission, size, Unicode normalization or fragment checks occur.
pub fn parse_custom_listing(rows: &[String]) -> Result<Vec<String>, ListingError> {
    let mut names = Vec::new();
    for (row, line) in rows.iter().enumerate() {
        let name = line
            .split_whitespace()
            .last()
            .ok_or(ListingError::Malformed { row })?;
        if name.to_ascii_lowercase().ends_with(".loxcc") {
            if line.starts_with('d') || line.starts_with('l') || name.contains(['/', '\\']) {
                return Err(ListingError::Unsupported { row });
            }
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}
