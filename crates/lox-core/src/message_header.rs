//! Observe an eight-byte Loxone application message header, not a WebSocket frame.
//!
//! The caller supplies exactly the separate binary header message, after transport
//! processing. This module neither finds headers in arbitrary bytes nor assembles,
//! dispatches or validates the following payload. In particular, an estimated size
//! is not a payload length guarantee. See the upstream crosswalk for source,
//! reference and caller differences.

use std::fmt;

/// Message types documented by the pinned protocol reference.
///
/// Classification is not a payload decoder or a promise that a payload follows:
/// the reference describes out-of-service as header-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MessageType {
    Text = 0,
    BinaryFile = 1,
    ValueStates = 2,
    TextStates = 3,
    DaytimerStates = 4,
    OutOfService = 5,
    KeepAlive = 6,
    WeatherStates = 7,
}

/// Lossless header observation, including uninterpreted flags and reserved data.
///
/// Unknown type bytes are retained, not approved for dispatch. No relationship
/// between type, flags and payload size is enforced by this observation API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageHeader {
    message_type: u8,
    info_flags: u8,
    reserved: u8,
    payload_size: u32,
}

impl MessageHeader {
    /// Construct raw header fields without imposing undocumented validation.
    ///
    /// `payload_size` counts bytes; bit zero of `info_flags` marks an estimate.
    /// Nonzero reserved data and unknown types/bits have no interpreted semantics.
    pub const fn new(message_type: u8, info_flags: u8, reserved: u8, payload_size: u32) -> Self {
        Self {
            message_type,
            info_flags,
            reserved,
            payload_size,
        }
    }

    pub const fn message_type(&self) -> u8 {
        self.message_type
    }

    /// Interpret only the eight documented identifiers; retain all others raw.
    pub const fn known_type(&self) -> Option<MessageType> {
        match self.message_type {
            0 => Some(MessageType::Text),
            1 => Some(MessageType::BinaryFile),
            2 => Some(MessageType::ValueStates),
            3 => Some(MessageType::TextStates),
            4 => Some(MessageType::DaytimerStates),
            5 => Some(MessageType::OutOfService),
            6 => Some(MessageType::KeepAlive),
            7 => Some(MessageType::WeatherStates),
            _ => None,
        }
    }

    pub const fn info_flags(&self) -> u8 {
        self.info_flags
    }

    /// An estimated header is followed by an exact header according to the
    /// reference. This stateless observation does not enforce that sequence.
    pub const fn size_is_estimated(&self) -> bool {
        self.info_flags & 1 != 0
    }

    pub const fn reserved(&self) -> u8 {
        self.reserved
    }

    /// Unsigned byte count, possibly estimated; never used here to allocate.
    pub const fn payload_size(&self) -> u32 {
        self.payload_size
    }

    /// Encode exactly eight application-header bytes, retaining all raw fields.
    ///
    /// This is not a WebSocket encoder or a guarantee a peer accepts unknown data.
    pub const fn to_wire_bytes(&self) -> [u8; 8] {
        let size = self.payload_size.to_le_bytes();
        [
            0x03,
            self.message_type,
            self.info_flags,
            self.reserved,
            size[0],
            size[1],
            size[2],
            size[3],
        ]
    }
}

/// Exact-header admission failures. No partial observation is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHeaderError {
    InvalidLength { actual: usize },
    InvalidMagic { actual: u8 },
}

impl fmt::Display for MessageHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => write!(
                f,
                "application message header has {actual} bytes; expected exactly 8"
            ),
            Self::InvalidMagic { actual } => write!(
                f,
                "application message header starts with 0x{actual:02x}; expected 0x03"
            ),
        }
    }
}

impl std::error::Error for MessageHeaderError {}

/// Decode exactly one application header, with length checked before magic.
///
/// Extra bytes (including an appended payload) are rejected, not consumed.
/// Unknown type bytes, flag bits and the reserved byte are preserved unchanged;
/// observation is deliberately broader than the upstream enum-indexing reader.
/// All unsigned 32-bit sizes are admitted without allocation or signed narrowing.
///
/// ```
/// use lox_core::message_header::{MessageHeader, MessageType, decode_message_header};
///
/// let bytes = MessageHeader::new(MessageType::TextStates as u8, 1, 0, 900).to_wire_bytes();
/// let header = decode_message_header(&bytes)?;
/// assert_eq!(header.known_type(), Some(MessageType::TextStates));
/// assert!(header.size_is_estimated()); // not yet an exact payload size
/// # Ok::<(), lox_core::message_header::MessageHeaderError>(())
/// ```
pub fn decode_message_header(bytes: &[u8]) -> Result<MessageHeader, MessageHeaderError> {
    let bytes: &[u8; 8] = bytes
        .try_into()
        .map_err(|_| MessageHeaderError::InvalidLength {
            actual: bytes.len(),
        })?;
    if bytes[0] != 0x03 {
        return Err(MessageHeaderError::InvalidMagic { actual: bytes[0] });
    }
    Ok(MessageHeader::new(
        bytes[1],
        bytes[2],
        bytes[3],
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
    ))
}
