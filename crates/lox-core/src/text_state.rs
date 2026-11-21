//! Decode complete text-state table payloads without transport or I/O.
//!
//! The caller must identify and assemble the payload and remove all protocol and
//! transport headers first. This module cannot classify or reassemble frames.
//! Each record is UUID16, icon UUID16, unsigned little-endian text-byte length4,
//! UTF-8 text, then 0..=3 ignored bytes aligning the next record to four bytes.
//! See the upstream crosswalk for evidence and intentional policy differences.

use crate::numeric_state::StateUuid;
use std::fmt;

const HEADER_BYTES: usize = 36;

/// One owned update, retaining both identifiers and the exact valid UTF-8 text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextState {
    uuid: StateUuid,
    icon_uuid: StateUuid,
    text_byte_len: u32,
    text: String,
}

impl TextState {
    pub fn uuid(&self) -> StateUuid {
        self.uuid
    }

    pub fn icon_uuid(&self) -> StateUuid {
        self.icon_uuid
    }

    /// Declared byte length, excluding padding; not a Unicode character count.
    pub fn text_byte_len(&self) -> u32 {
        self.text_byte_len
    }

    /// Exact text, including embedded and trailing NULs; no trimming/normalization.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Complete-input rejection. Errors contain positions/counts, never payload data.
///
/// `record_offset` is the record's start in the supplied payload, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStatePayloadError {
    TruncatedHeader {
        record_offset: usize,
        available: usize,
    },
    LengthOverflow {
        record_offset: usize,
        declared: u32,
    },
    TruncatedText {
        record_offset: usize,
        declared: u32,
        available: usize,
    },
    TruncatedPadding {
        record_offset: usize,
        expected: usize,
        available: usize,
    },
    InvalidUtf8 {
        record_offset: usize,
        /// Valid prefix length relative to the start of this record's text.
        valid_up_to: usize,
        /// Malformed sequence length; `None` means an incomplete final sequence.
        error_len: Option<usize>,
    },
}

impl fmt::Display for TextStatePayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader {
                record_offset,
                available,
            } => write!(
                f,
                "text-state header at byte {record_offset} needs 36 bytes, has {available}"
            ),
            Self::LengthOverflow {
                record_offset,
                declared,
            } => write!(
                f,
                "text-state length {declared} at byte {record_offset} cannot be represented"
            ),
            Self::TruncatedText {
                record_offset,
                declared,
                available,
            } => write!(
                f,
                "text-state text at record byte {record_offset} declares {declared} bytes, has {available}"
            ),
            Self::TruncatedPadding {
                record_offset,
                expected,
                available,
            } => write!(
                f,
                "text-state padding at record byte {record_offset} needs {expected} bytes, has {available}"
            ),
            Self::InvalidUtf8 {
                record_offset,
                valid_up_to,
                error_len,
            } => write!(
                f,
                "invalid UTF-8 in text-state record at byte {record_offset}, after {valid_up_to} text bytes (sequence length {error_len:?})"
            ),
        }
    }
}

impl std::error::Error for TextStatePayloadError {}

/// Decode an already-identified, complete text-state table payload, not a frame.
///
/// Empty input is valid. Order, duplicates, both UUIDs, empty text and all
/// in-length NULs are preserved. Padding must be present but may contain any
/// bytes; it is ignored. Lengths count UTF-8 bytes, not characters.
///
/// Any incomplete record, trailing fragment or invalid UTF-8 returns an error,
/// never a partial result. Strict atomic rejection, lossless UTF-8 and no NUL
/// trimming are original API choices, not compatibility with lenient readers.
/// Bounds are checked before slicing or allocating text; no declared length is
/// used to reserve storage in advance. No application-specific size cap is set.
///
/// ```
/// use lox_core::text_state::{TextStatePayloadError, decode_text_state_payload};
///
/// assert!(decode_text_state_payload(&[])?.is_empty());
/// assert!(matches!(decode_text_state_payload(&[0; 35]),
///     Err(TextStatePayloadError::TruncatedHeader {
///         record_offset: 0, available: 35
///     })));
/// # Ok::<(), TextStatePayloadError>(())
/// ```
pub fn decode_text_state_payload(payload: &[u8]) -> Result<Vec<TextState>, TextStatePayloadError> {
    let mut remaining = payload;
    let mut states = Vec::new();
    while !remaining.is_empty() {
        let record_offset = payload.len() - remaining.len();
        if remaining.len() < HEADER_BYTES {
            return Err(TextStatePayloadError::TruncatedHeader {
                record_offset,
                available: remaining.len(),
            });
        }
        let (header, body) = remaining.split_at(HEADER_BYTES);
        let declared = u32::from_le_bytes([header[32], header[33], header[34], header[35]]);
        let text_len =
            usize::try_from(declared).map_err(|_| TextStatePayloadError::LengthOverflow {
                record_offset,
                declared,
            })?;
        if text_len > body.len() {
            return Err(TextStatePayloadError::TruncatedText {
                record_offset,
                declared,
                available: body.len(),
            });
        }
        let (text_bytes, tail) = body.split_at(text_len);
        let padding = ((4 - declared % 4) % 4) as usize;
        if padding > tail.len() {
            return Err(TextStatePayloadError::TruncatedPadding {
                record_offset,
                expected: padding,
                available: tail.len(),
            });
        }
        let text = std::str::from_utf8(text_bytes).map_err(|error| {
            TextStatePayloadError::InvalidUtf8 {
                record_offset,
                valid_up_to: error.valid_up_to(),
                error_len: error.error_len(),
            }
        })?;
        let mut uuid = [0; 16];
        uuid.copy_from_slice(&header[..16]);
        let mut icon_uuid = [0; 16];
        icon_uuid.copy_from_slice(&header[16..32]);
        states.push(TextState {
            uuid: StateUuid::from_wire_bytes(uuid),
            icon_uuid: StateUuid::from_wire_bytes(icon_uuid),
            text_byte_len: declared,
            text: text.to_owned(),
        });
        // Consume checked slices instead of adding untrusted lengths to offsets.
        remaining = &tail[padding..];
    }
    Ok(states)
}
