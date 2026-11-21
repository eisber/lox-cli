//! Owned projections of Core's exact application-header observation.

use lox_core::message_header as core;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum MessageType {
    Text,
    BinaryFile,
    ValueStates,
    TextStates,
    DaytimerStates,
    OutOfService,
    KeepAlive,
    WeatherStates,
}

impl From<core::MessageType> for MessageType {
    fn from(value: core::MessageType) -> Self {
        match value {
            core::MessageType::Text => Self::Text,
            core::MessageType::BinaryFile => Self::BinaryFile,
            core::MessageType::ValueStates => Self::ValueStates,
            core::MessageType::TextStates => Self::TextStates,
            core::MessageType::DaytimerStates => Self::DaytimerStates,
            core::MessageType::OutOfService => Self::OutOfService,
            core::MessageType::KeepAlive => Self::KeepAlive,
            core::MessageType::WeatherStates => Self::WeatherStates,
        }
    }
}

/// Host-owned observation, not a transport frame or payload validation.
/// Python represents unsigned fields as nonnegative ints. Java/Kotlin byte
/// and int fields carry the same u8/u32 bits, not signed protocol values.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct MessageHeader {
    pub message_type: u8,
    pub known_type: Option<MessageType>,
    pub info_flags: u8,
    pub reserved: u8,
    pub payload_size: u32,
    pub size_is_estimated: bool,
    pub wire_bytes: Vec<u8>,
}

impl From<core::MessageHeader> for MessageHeader {
    fn from(header: core::MessageHeader) -> Self {
        Self {
            message_type: header.message_type(),
            known_type: header.known_type().map(Into::into),
            info_flags: header.info_flags(),
            reserved: header.reserved(),
            payload_size: header.payload_size(),
            size_is_estimated: header.size_is_estimated(),
            wire_bytes: header.to_wire_bytes().to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum MessageHeaderError {
    #[error("application message header has {actual} bytes; expected exactly 8")]
    InvalidLength { actual: u64 },
    #[error("application message header starts with 0x{actual:02x}; expected 0x03")]
    InvalidMagic { actual: u8 },
}

impl From<core::MessageHeaderError> for MessageHeaderError {
    fn from(error: core::MessageHeaderError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match error {
            core::MessageHeaderError::InvalidLength { actual } => Self::InvalidLength {
                actual: actual as u64,
            },
            core::MessageHeaderError::InvalidMagic { actual } => Self::InvalidMagic { actual },
        }
    }
}

/// Observe exactly eight application-header bytes; Core checks length first.
#[uniffi::export]
pub fn decode_message_header(bytes: Vec<u8>) -> Result<MessageHeader, MessageHeaderError> {
    Ok(core::decode_message_header(&bytes)?.into())
}

/// Preserve all raw fields using Core's constructor and exact wire encoder.
#[uniffi::export]
pub fn encode_message_header(
    message_type: u8,
    info_flags: u8,
    reserved: u8,
    payload_size: u32,
) -> Vec<u8> {
    core::MessageHeader::new(message_type, info_flags, reserved, payload_size)
        .to_wire_bytes()
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_unsigned_fields_and_all_known_types_roundtrip_through_core() {
        for kind in (0..=7).chain([0xe7, 0xff]) {
            for flags in [0x80, 0xff] {
                let expected = core::MessageHeader::new(kind, flags, 0x91, u32::MAX);
                let bytes = encode_message_header(kind, flags, 0x91, u32::MAX);
                assert_eq!(bytes, expected.to_wire_bytes());
                let actual = decode_message_header(bytes.clone()).unwrap();
                assert_eq!(actual, MessageHeader::from(expected));
                assert_eq!(actual.wire_bytes, bytes);
                assert_eq!(actual.known_type.is_some(), kind < 8);
            }
        }
    }

    #[test]
    fn strict_core_errors_preserve_length_precedence_magic_and_unsigned_context() {
        for length in [0, 1, 7, 9, 32] {
            let bytes = vec![0x83; length];
            assert_eq!(
                decode_message_header(bytes.clone()).unwrap_err(),
                MessageHeaderError::from(core::decode_message_header(&bytes).unwrap_err())
            );
        }
        assert_eq!(
            decode_message_header(vec![0x83; 8]).unwrap_err(),
            MessageHeaderError::InvalidMagic { actual: 0x83 }
        );
        assert_eq!(
            MessageHeaderError::from(core::MessageHeaderError::InvalidLength {
                actual: usize::MAX
            }),
            MessageHeaderError::InvalidLength {
                actual: usize::MAX as u64
            }
        );
    }
}
