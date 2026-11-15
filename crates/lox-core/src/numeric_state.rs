//! Decode complete numeric value-state table payloads without transport or I/O.
//!
//! The caller must already have identified and assembled a value-state table
//! payload, with any transport/protocol headers removed. Length alone cannot
//! identify its type: arbitrary bytes of a matching length will decode.
//! This module does not authenticate, parse or reassemble modern WebSocket frames,
//! encode the legacy WSX handshake, or decode statistics, text, daytimer or weather.
//!
//! Each record contains a 16-byte UUID followed by a little-endian IEEE 754 f64.
//! UUID fields use u32 LE, u16 LE, u16 LE, then eight bytes in wire order.
//! See the repository's upstream crosswalk for format evidence and policy differences.

use std::fmt;

const RECORD_BYTES: usize = 24;

/// A state identifier retaining its exact 16 wire bytes, not a config UUID parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateUuid([u8; 16]);

impl StateUuid {
    pub(crate) const fn from_wire_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Original wire bytes, including the little-endian first three fields.
    pub fn wire_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Lowercase native35 form: `xxxxxxxx-xxxx-xxxx-xxxxxxxxxxxxxxxx`.
    ///
    /// This does not normalize identifiers in other APIs to or from RFC36.
    pub fn to_native35(&self) -> String {
        let b = &self.0;
        format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            u16::from_le_bytes([b[4], b[5]]),
            u16::from_le_bytes([b[6], b[7]]),
            b[8],
            b[9],
            b[10],
            b[11],
            b[12],
            b[13],
            b[14],
            b[15],
        )
    }
}

/// One numeric state update. Equality compares UUID bytes and value bits,
/// distinguishing signed zero and preserving NaN payload identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericState {
    uuid: StateUuid,
    value_bits: u64,
}

impl NumericState {
    pub fn uuid(&self) -> StateUuid {
        self.uuid
    }

    /// Interpret the value bits without arithmetic, rounding or finite validation.
    pub fn value(&self) -> f64 {
        f64::from_bits(self.value_bits)
    }

    /// Exact IEEE 754 bits, including signed zero and signaling/quiet NaN payloads.
    pub fn value_bits(&self) -> u64 {
        self.value_bits
    }
}

/// A complete payload must contain only whole 24-byte records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericStatePayloadError {
    /// No records are returned, even if a complete prefix precedes the tail.
    InvalidLength { actual: usize, trailing: usize },
}

impl fmt::Display for NumericStatePayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual, trailing } => write!(
                f,
                "numeric value-state payload has {actual} bytes with {trailing} trailing bytes; expected whole 24-byte records"
            ),
        }
    }
}

impl std::error::Error for NumericStatePayloadError {}

/// Decode an already-assembled numeric value-state table payload, not a frame.
///
/// Empty input is valid. Records retain input order and duplicate identifiers.
/// All f64 bit patterns are accepted and preserved. A non-multiple-of-24 length
/// returns an error before allocating/decoding records, never partial success.
/// This is a strict, new API policy, not compatibility with lenient prefix readers.
///
/// ```
/// use lox_core::numeric_state::{NumericStatePayloadError, decode_value_state_payload};
///
/// fn observe(payload: &[u8]) -> Result<Vec<(String, u64)>, NumericStatePayloadError> {
///     Ok(decode_value_state_payload(payload)?
///         .iter()
///         .map(|state| (state.uuid().to_native35(), state.value_bits()))
///         .collect())
/// }
///
/// assert!(observe(&[])?.is_empty());
/// assert!(matches!(observe(&[0; 25]),
///     Err(NumericStatePayloadError::InvalidLength { actual: 25, trailing: 1 })));
/// # Ok::<(), NumericStatePayloadError>(())
/// ```
pub fn decode_value_state_payload(
    payload: &[u8],
) -> Result<Vec<NumericState>, NumericStatePayloadError> {
    let (records, trailing) = payload.as_chunks::<RECORD_BYTES>();
    if !trailing.is_empty() {
        return Err(NumericStatePayloadError::InvalidLength {
            actual: payload.len(),
            trailing: trailing.len(),
        });
    }

    Ok(records
        .iter()
        .map(|record| {
            let mut uuid = [0; 16];
            uuid.copy_from_slice(&record[..16]);
            let mut value = [0; 8];
            value.copy_from_slice(&record[16..]);
            NumericState {
                uuid: StateUuid::from_wire_bytes(uuid),
                value_bits: u64::from_le_bytes(value),
            }
        })
        .collect())
}
