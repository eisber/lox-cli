//! LoxCC compression and decompression — Loxone Miniserver config archives.
//!
//! Backup ZIPs contain `sps0.LoxCC`, a custom LZ-compressed binary. These
//! operations return raw bytes, not validated or normalized XML.
//!
//! ## Binary format
//!
//! ```text
//!   [0..4]   u32_le  magic (0xaabbccee)
//!   [4..8]   u32_le  compressed payload size
//!   [8..12]  u32_le  uncompressed size
//!   [12..16] u32_le  CRC32 of uncompressed data
//!   [16..]   compressed data (LZ4-style tokens)
//! ```
//!
//! Each token byte encodes `high nibble = literal count`, `low nibble =
//! match_length - 4`. A nibble of 15 is extended by following bytes until
//! a byte below 255. Literals precede a two-byte little-endian back-reference
//! offset; the final literal block may omit the back-reference.
//!
//! The compressor writes CRC32 over the exact input bytes. This field is
//! required for the Miniserver to trust encrypted config fields (`t="15"`).
//! A zero CRC32 causes it to ignore sensitive fields.
//!
//! ## Compatibility limits
//!
//! Decoding intentionally retains the CLI's permissive behavior: LoxCC CRC32
//! and declared sizes are not verified; truncated final tokens/literals can
//! produce partial output. The size hint only caps initial allocation, not
//! total output. ZIP extraction chooses the first case-insensitive name
//! containing `sps0.loxcc`, including nested names and suffixes. It is not a
//! deployment validator or an untrusted-input resource limiter. A future
//! strict verification policy must be a separate, explicit API.
//!
//! ## Provenance
//!
//! Extracted from eisber/lox-cli's own `src/loxcc.rs`; project licensing and
//! attribution are unchanged (see repository LICENSE and NOTICE).
//! Format reference: <https://github.com/sarnau/Inside-The-Loxone-Miniserver>.
//! CRC32 discovery and `t="11"` plaintext trick: eisber/lox-cli contributors.

use crc32fast::Hasher as Crc32Hasher;
use std::fmt;
use std::io::{self, Cursor, Read};

#[cfg(test)]
mod tests;

const LOXCC_MAGIC: u32 = 0xaabbccee;

/// Result of an archive decoding operation.
pub type Result<T> = std::result::Result<T, ArchiveError>;

/// Structured codec failures, with ZIP entry context and underlying sources.
#[derive(Debug)]
#[non_exhaustive]
pub enum ArchiveError {
    /// The input is shorter than the sixteen-byte LoxCC header.
    HeaderTooShort { actual: usize },
    /// The LoxCC magic does not match the supported format.
    InvalidMagic { actual: u32 },
    /// A back-reference has a zero offset at this compressed-payload position.
    ZeroBackReference { position: usize },
    /// A back-reference points before the decoded output.
    BackReferenceOutOfBounds { offset: usize, output_len: usize },
    /// The non-LoxCC input could not be opened as a ZIP.
    InvalidZip { source: zip::result::ZipError },
    /// A ZIP member could not be opened during the sequential search.
    ZipEntry {
        index: usize,
        source: zip::result::ZipError,
    },
    /// A selected ZIP member could not be read.
    ReadZipEntry {
        index: usize,
        name: String,
        source: io::Error,
    },
    /// No ZIP member name contains `sps0.loxcc` (case-insensitive).
    MissingConfigEntry,
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeaderTooShort { actual } => write!(f, "LoxCC data too short ({actual} bytes)"),
            Self::InvalidMagic { actual } => write!(
                f,
                "Not a valid LoxCC file (magic: 0x{actual:08x}, expected 0x{LOXCC_MAGIC:08x})"
            ),
            Self::ZeroBackReference { position } => {
                write!(f, "LoxCC: zero back-reference offset at byte {position}")
            }
            Self::BackReferenceOutOfBounds { offset, output_len } => write!(
                f,
                "LoxCC: back-reference offset {offset} exceeds output size {output_len}"
            ),
            Self::InvalidZip { .. } => write!(f, "Invalid ZIP archive"),
            Self::ZipEntry { index, .. } => write!(f, "Cannot open ZIP entry {index}"),
            Self::ReadZipEntry { index, name, .. } => {
                write!(f, "Cannot read ZIP entry {index} ({name})")
            }
            Self::MissingConfigEntry => write!(f, "Backup ZIP does not contain sps0.LoxCC"),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidZip { source } | Self::ZipEntry { source, .. } => Some(source),
            Self::ReadZipEntry { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Decompress a raw LoxCC/LxRes blob without interpreting its decoded bytes.
///
/// Header size and magic errors precede payload errors. Positions in
/// [`ArchiveError::ZeroBackReference`] are relative to the compressed payload.
/// See the module's compatibility limits before decoding untrusted input.
pub fn decompress_loxcc(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 16 {
        return Err(ArchiveError::HeaderTooShort { actual: data.len() });
    }
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != LOXCC_MAGIC {
        return Err(ArchiveError::InvalidMagic { actual: magic });
    }
    let _comp_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let uncomp_hint = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

    let compressed = &data[16..];
    // This preserves the existing initial-allocation cap, not an output limit.
    let capped_hint = uncomp_hint.clamp(1024, 64 * 1024 * 1024);
    let mut output = Vec::with_capacity(capped_hint);
    let mut pos = 0;

    while pos < compressed.len() {
        let token = compressed[pos];
        pos += 1;

        let mut lit_len = (token >> 4) as usize;
        if lit_len == 15 {
            loop {
                if pos >= compressed.len() {
                    break;
                }
                let extra = compressed[pos] as usize;
                pos += 1;
                lit_len += extra;
                if extra != 255 {
                    break;
                }
            }
        }

        if pos + lit_len > compressed.len() {
            // Allow partial final literal block (end of stream).
            let remaining = compressed.len() - pos;
            output.extend_from_slice(&compressed[pos..pos + remaining]);
            break;
        }
        output.extend_from_slice(&compressed[pos..pos + lit_len]);
        pos += lit_len;

        if pos >= compressed.len() {
            break;
        }

        if pos + 2 > compressed.len() {
            break;
        }
        let offset = u16::from_le_bytes([compressed[pos], compressed[pos + 1]]) as usize;
        pos += 2;

        if offset == 0 {
            return Err(ArchiveError::ZeroBackReference { position: pos - 2 });
        }

        let mut match_len = (token & 0x0F) as usize + 4;
        if (token & 0x0F) == 15 {
            loop {
                if pos >= compressed.len() {
                    break;
                }
                let extra = compressed[pos] as usize;
                pos += 1;
                match_len += extra;
                if extra != 255 {
                    break;
                }
            }
        }

        if offset > output.len() {
            return Err(ArchiveError::BackReferenceOutOfBounds {
                offset,
                output_len: output.len(),
            });
        }
        let start = output.len() - offset;
        for i in 0..match_len {
            let byte = output[start + (i % offset)];
            output.push(byte);
        }
    }

    Ok(output)
}

/// Decode raw LoxCC/LxRes by magic, otherwise search a backup ZIP.
///
/// Returns raw bytes from the first member whose lowercased name contains
/// `sps0.loxcc`. No XML validation, normalization or deployment checks occur.
pub fn extract_and_decompress(data: &[u8]) -> Result<Vec<u8>> {
    if data.starts_with(&LOXCC_MAGIC.to_le_bytes()) {
        return decompress_loxcc(data);
    }

    let cursor = Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|source| ArchiveError::InvalidZip { source })?;

    let mut loxcc_data = None;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|source| ArchiveError::ZipEntry { index: i, source })?;
        if file.name().to_lowercase().contains("sps0.loxcc") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|source| ArchiveError::ReadZipEntry {
                    index: i,
                    name: file.name().to_owned(),
                    source,
                })?;
            loxcc_data = Some(buf);
            break;
        }
    }

    let data = loxcc_data.ok_or(ArchiveError::MissingConfigEntry)?;
    decompress_loxcc(&data)
}

/// Compress exact input bytes into LoxCC with LZ4 block compression and CRC32.
///
/// No XML parsing or text conversion occurs. As in the original CLI codec,
/// header sizes are stored using `u32` casts; inputs exceeding that format's
/// size range are not validated by this compatibility API.
pub fn compress_loxcc(data: &[u8]) -> Vec<u8> {
    let compressed = lz4_flex::block::compress(data);

    let mut hasher = Crc32Hasher::new();
    hasher.update(data);
    let checksum = hasher.finalize();

    let mut blob = Vec::with_capacity(16 + compressed.len());
    blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
    blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    blob.extend_from_slice(&(data.len() as u32).to_le_bytes());
    blob.extend_from_slice(&checksum.to_le_bytes());
    blob.extend_from_slice(&compressed);
    blob
}
