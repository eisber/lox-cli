//! Thin native archive, XML inspection/statistics, numeric, clock, control and color adapters.
//! All behavior lives in lox-core.
//! Generated code and its native library must come from the same build.

use lox_core::configuration::archive;
use std::error::Error;

mod clock;
mod color;
mod configuration;
mod control;
mod control_catalog;
mod numeric_state;
mod statistics;

uniffi::setup_scaffolding!();

/// Language-native exceptions retain domain variants, context and source text.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ArchiveError {
    #[error("LoxCC data too short ({actual} bytes)")]
    HeaderTooShort { actual: u64 },
    #[error("Not a valid LoxCC file (magic: 0x{actual:08x}, expected 0xaabbccee)")]
    InvalidMagic { actual: u32 },
    #[error("LoxCC: zero back-reference offset at byte {position}")]
    ZeroBackReference { position: u64 },
    #[error("LoxCC: back-reference offset {offset} exceeds output size {output_len}")]
    BackReferenceOutOfBounds { offset: u64, output_len: u64 },
    #[error("Invalid ZIP archive: {source_message}")]
    InvalidZip { source_message: String },
    #[error("Cannot open ZIP entry {index}: {source_message}")]
    ZipEntry { index: u64, source_message: String },
    #[error("Cannot read ZIP entry {index} ({name}): {source_message}")]
    ReadZipEntry {
        index: u64,
        name: String,
        source_message: String,
    },
    #[error("No sps0.LoxCC found in backup ZIP")]
    MissingConfigEntry,
    /// Forward-compatible fallback for new non-exhaustive core variants.
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<archive::ArchiveError> for ArchiveError {
    fn from(error: archive::ArchiveError) -> Self {
        match error {
            archive::ArchiveError::HeaderTooShort { actual } => Self::HeaderTooShort {
                actual: actual as u64,
            },
            archive::ArchiveError::InvalidMagic { actual } => Self::InvalidMagic { actual },
            archive::ArchiveError::ZeroBackReference { position } => Self::ZeroBackReference {
                position: position as u64,
            },
            archive::ArchiveError::BackReferenceOutOfBounds { offset, output_len } => {
                Self::BackReferenceOutOfBounds {
                    offset: offset as u64,
                    output_len: output_len as u64,
                }
            }
            archive::ArchiveError::InvalidZip { source } => Self::InvalidZip {
                source_message: source.to_string(),
            },
            archive::ArchiveError::ZipEntry { index, source } => Self::ZipEntry {
                index: index as u64,
                source_message: source.to_string(),
            },
            archive::ArchiveError::ReadZipEntry {
                index,
                name,
                source,
            } => Self::ReadZipEntry {
                index: index as u64,
                name,
                source_message: source.to_string(),
            },
            archive::ArchiveError::MissingConfigEntry => Self::MissingConfigEntry,
            other => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Compress opaque bytes without XML parsing or normalization.
#[uniffi::export]
pub fn compress_loxcc(data: Vec<u8>) -> Vec<u8> {
    archive::compress_loxcc(&data)
}

/// Decode raw LoxCC with the core's existing permissive policy.
#[uniffi::export]
pub fn decompress_loxcc(data: Vec<u8>) -> Result<Vec<u8>, ArchiveError> {
    archive::decompress_loxcc(&data).map_err(Into::into)
}

/// Decode raw LoxCC or select an archive member using the core's policy.
#[uniffi::export]
pub fn extract_and_decompress(data: Vec<u8>) -> Result<Vec<u8>, ArchiveError> {
    archive::extract_and_decompress(&data).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_bytes_and_permissive_header_policy_survive_the_adapter() {
        let raw: Vec<u8> = (0..=255).collect();
        let mut encoded = compress_loxcc(raw.clone());
        assert_eq!(encoded, archive::compress_loxcc(&raw));
        assert_eq!(decompress_loxcc(encoded.clone()).unwrap(), raw);
        // Decoding is not strict CRC/declared-size verification.
        encoded[4..16].fill(0);
        assert_eq!(extract_and_decompress(encoded).unwrap(), raw);
    }

    #[test]
    fn typed_header_and_back_reference_context_survives_conversion() {
        assert!(matches!(
            decompress_loxcc(vec![0; 3]),
            Err(ArchiveError::HeaderTooShort { actual: 3 })
        ));
        assert!(matches!(
            decompress_loxcc(vec![0; 16]),
            Err(ArchiveError::InvalidMagic { actual: 0 })
        ));
        let mut bad = compress_loxcc(vec![]);
        bad.truncate(16);
        bad.extend_from_slice(&[0, 0, 0]);
        assert!(matches!(
            decompress_loxcc(bad.clone()),
            Err(ArchiveError::ZeroBackReference { position: 1 })
        ));
        bad[17] = 1;
        assert!(matches!(
            decompress_loxcc(bad),
            Err(ArchiveError::BackReferenceOutOfBounds {
                offset: 1,
                output_len: 0
            })
        ));
    }

    #[test]
    fn zip_failures_retain_a_variant_and_original_source_text() {
        let invalid = b"not a zip".to_vec();
        let core_error = archive::extract_and_decompress(&invalid).unwrap_err();
        let expected = core_error.source().unwrap().to_string();
        match extract_and_decompress(invalid).unwrap_err() {
            ArchiveError::InvalidZip { source_message } => assert_eq!(source_message, expected),
            error => panic!("wrong variant: {error:?}"),
        }
        let empty_zip = b"PK\x05\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert!(matches!(
            extract_and_decompress(empty_zip.to_vec()),
            Err(ArchiveError::MissingConfigEntry)
        ));
    }
}
