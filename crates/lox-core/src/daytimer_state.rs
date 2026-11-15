//! Observe complete runtime Daytimer-State payloads without transport or I/O.
//!
//! The caller identifies and assembles the payload and removes all headers.
//! Each record is UUID16, default IEEE f64 LE8, signed entry count LE4, then
//! that many entries: signed mode/from/to/activation LE4 each and IEEE f64 LE8.
//! These are observations, not Config DayTimer editing or schedule validation.

use crate::numeric_state::StateUuid;
use std::fmt;

const HEADER_BYTES: usize = 28;
const ENTRY_BYTES: usize = 24;

/// One runtime entry. Equality compares raw fields and IEEE bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaytimerEntry {
    mode: i32,
    from_minutes: i32,
    to_minutes: i32,
    activation_word: i32,
    value_bits: u64,
}

impl DaytimerEntry {
    /// Observed signed mode number, not a mode UUID or validated mode.
    pub fn mode(&self) -> i32 {
        self.mode
    }

    /// Observed minutes since midnight, without range checks or normalization.
    pub fn from_minutes(&self) -> i32 {
        self.from_minutes
    }

    /// Observed minutes since midnight, without range checks or normalization.
    pub fn to_minutes(&self) -> i32 {
        self.to_minutes
    }

    /// Exact signed activation word, including values other than zero and one.
    pub fn activation_word(&self) -> i32 {
        self.activation_word
    }

    /// Convenience interpretation only; the raw word remains authoritative.
    pub fn needs_activation(&self) -> bool {
        self.activation_word != 0
    }

    pub fn value_bits(&self) -> u64 {
        self.value_bits
    }

    /// Interpret the stored IEEE bits without arithmetic or finite validation.
    pub fn value(&self) -> f64 {
        f64::from_bits(self.value_bits)
    }
}

/// One owned runtime state, retaining entry order and duplicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaytimerState {
    uuid: StateUuid,
    default_value_bits: u64,
    entries: Vec<DaytimerEntry>,
}

impl DaytimerState {
    pub fn uuid(&self) -> StateUuid {
        self.uuid
    }

    pub fn default_value_bits(&self) -> u64 {
        self.default_value_bits
    }

    /// Interpret the stored IEEE bits without arithmetic or finite validation.
    pub fn default_value(&self) -> f64 {
        f64::from_bits(self.default_value_bits)
    }

    pub fn entries(&self) -> &[DaytimerEntry] {
        &self.entries
    }
}

/// Atomic rejection with byte offsets/counts only, never payload contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaytimerStatePayloadError {
    TruncatedHeader {
        record_offset: usize,
        available: usize,
    },
    NegativeEntryCount {
        record_offset: usize,
        declared: i32,
    },
    /// The nonnegative entry count or its byte extent cannot fit `usize`.
    LengthOverflow {
        record_offset: usize,
        declared: i32,
    },
    TruncatedEntries {
        record_offset: usize,
        declared: i32,
        /// Available bytes after this record's 28-byte header.
        available: usize,
    },
}

impl fmt::Display for DaytimerStatePayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader {
                record_offset,
                available,
            } => write!(
                f,
                "daytimer-state header at byte {record_offset} needs 28 bytes, has {available}"
            ),
            Self::NegativeEntryCount {
                record_offset,
                declared,
            } => write!(
                f,
                "daytimer-state record at byte {record_offset} declares negative entry count {declared}"
            ),
            Self::LengthOverflow {
                record_offset,
                declared,
            } => write!(
                f,
                "daytimer-state entry count {declared} at byte {record_offset} has an unrepresentable byte extent"
            ),
            Self::TruncatedEntries {
                record_offset,
                declared,
                available,
            } => write!(
                f,
                "daytimer-state record at byte {record_offset} declares {declared} entries of 24 bytes, has {available} bytes"
            ),
        }
    }
}

impl std::error::Error for DaytimerStatePayloadError {}

/// Decode an already-identified, complete runtime Daytimer-State table payload.
///
/// Empty input and zero-entry states are valid. State/entry order, duplicates,
/// UUID wire bytes, all signed fields and every IEEE default/entry bit pattern
/// are preserved. No mode, time, activation or finite-value validator is applied.
///
/// A negative count, unrepresentable entry extent, incomplete record or trailing
/// fragment returns an error, never a partial vector. Signed counts and checked
/// byte extents are validated against the remaining input before allocating
/// entries; no untrusted count is used to reserve storage in advance.
/// Strict complete-input rejection is an original local policy, unlike upstream
/// prefix-success parsing, and is not a firmware-equivalence claim.
///
/// This API does not parse the separate eight-byte type-4 message header or
/// classify frames. Arbitrary bytes (including framed data) can coincide with a
/// valid payload shape; successful decoding does not establish their type.
///
/// ```
/// use lox_core::daytimer_state::{decode_daytimer_state_payload, DaytimerStatePayloadError};
///
/// assert!(decode_daytimer_state_payload(&[])?.is_empty());
/// assert!(matches!(decode_daytimer_state_payload(&[0; 27]),
///     Err(DaytimerStatePayloadError::TruncatedHeader {
///         record_offset: 0, available: 27
///     })));
/// # Ok::<(), DaytimerStatePayloadError>(())
/// ```
pub fn decode_daytimer_state_payload(
    payload: &[u8],
) -> Result<Vec<DaytimerState>, DaytimerStatePayloadError> {
    let mut remaining = payload;
    let mut states = Vec::new();
    while !remaining.is_empty() {
        let record_offset = payload.len() - remaining.len();
        if remaining.len() < HEADER_BYTES {
            return Err(DaytimerStatePayloadError::TruncatedHeader {
                record_offset,
                available: remaining.len(),
            });
        }
        let (header, body) = remaining.split_at(HEADER_BYTES);
        let declared = read_i32(header, 24);
        if declared < 0 {
            return Err(DaytimerStatePayloadError::NegativeEntryCount {
                record_offset,
                declared,
            });
        }
        let entry_bytes = usize::try_from(declared)
            .ok()
            .and_then(|count| count.checked_mul(ENTRY_BYTES))
            .ok_or(DaytimerStatePayloadError::LengthOverflow {
                record_offset,
                declared,
            })?;
        if entry_bytes > body.len() {
            return Err(DaytimerStatePayloadError::TruncatedEntries {
                record_offset,
                declared,
                available: body.len(),
            });
        }
        let (entry_data, tail) = body.split_at(entry_bytes);
        let (records, _) = entry_data.as_chunks::<ENTRY_BYTES>();
        let entries = records
            .iter()
            .map(|record| DaytimerEntry {
                mode: read_i32(record, 0),
                from_minutes: read_i32(record, 4),
                to_minutes: read_i32(record, 8),
                activation_word: read_i32(record, 12),
                value_bits: read_bits(record, 16),
            })
            .collect();
        let mut uuid = [0; 16];
        uuid.copy_from_slice(&header[..16]);
        states.push(DaytimerState {
            uuid: StateUuid::from_wire_bytes(uuid),
            default_value_bits: read_bits(header, 16),
            entries,
        });
        remaining = tail;
    }
    Ok(states)
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_bits(bytes: &[u8], offset: usize) -> u64 {
    let mut bits = [0; 8];
    bits.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(bits)
}
