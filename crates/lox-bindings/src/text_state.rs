//! Owned native projections of Core's strict, payload-only text decoder.

use lox_core::text_state as core;

/// Host-owned snapshot; no native handle or close obligation.
#[derive(Clone, Debug, uniffi::Record)]
pub struct TextState {
    pub uuid_wire_bytes: Vec<u8>,
    pub uuid_native35: String,
    pub icon_uuid_wire_bytes: Vec<u8>,
    pub icon_uuid_native35: String,
    pub text_byte_len: u32,
    pub text: String,
}

impl From<core::TextState> for TextState {
    fn from(state: core::TextState) -> Self {
        Self {
            uuid_wire_bytes: state.uuid().wire_bytes().to_vec(),
            uuid_native35: state.uuid().to_native35(),
            icon_uuid_wire_bytes: state.icon_uuid().wire_bytes().to_vec(),
            icon_uuid_native35: state.icon_uuid().to_native35(),
            text_byte_len: state.text_byte_len(),
            text: state.text().to_owned(),
        }
    }
}

/// Payload-free, complete-input failures; all positions count bytes.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum TextStatePayloadError {
    #[error("text-state header at byte {record_offset} needs 36 bytes, has {available}")]
    TruncatedHeader { record_offset: u64, available: u64 },
    #[error("text-state length {declared} at byte {record_offset} cannot be represented")]
    LengthOverflow { record_offset: u64, declared: u32 },
    #[error(
        "text-state text at record byte {record_offset} declares {declared} bytes, has {available}"
    )]
    TruncatedText {
        record_offset: u64,
        declared: u32,
        available: u64,
    },
    #[error(
        "text-state padding at record byte {record_offset} needs {expected} bytes, has {available}"
    )]
    TruncatedPadding {
        record_offset: u64,
        expected: u64,
        available: u64,
    },
    #[error(
        "invalid UTF-8 in text-state record at byte {record_offset}, after {valid_up_to} text bytes (sequence length {error_len:?})"
    )]
    InvalidUtf8 {
        record_offset: u64,
        valid_up_to: u64,
        error_len: Option<u64>,
    },
}

impl From<core::TextStatePayloadError> for TextStatePayloadError {
    fn from(error: core::TextStatePayloadError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match error {
            core::TextStatePayloadError::TruncatedHeader {
                record_offset,
                available,
            } => Self::TruncatedHeader {
                record_offset: record_offset as u64,
                available: available as u64,
            },
            core::TextStatePayloadError::LengthOverflow {
                record_offset,
                declared,
            } => Self::LengthOverflow {
                record_offset: record_offset as u64,
                declared,
            },
            core::TextStatePayloadError::TruncatedText {
                record_offset,
                declared,
                available,
            } => Self::TruncatedText {
                record_offset: record_offset as u64,
                declared,
                available: available as u64,
            },
            core::TextStatePayloadError::TruncatedPadding {
                record_offset,
                expected,
                available,
            } => Self::TruncatedPadding {
                record_offset: record_offset as u64,
                expected: expected as u64,
                available: available as u64,
            },
            core::TextStatePayloadError::InvalidUtf8 {
                record_offset,
                valid_up_to,
                error_len,
            } => Self::InvalidUtf8 {
                record_offset: record_offset as u64,
                valid_up_to: valid_up_to as u64,
                error_len: error_len.map(|length| length as u64),
            },
        }
    }
}

/// Caller identifies/assembles the table and removes headers. Core exclusively
/// validates bounds and UTF-8; order, duplicates and all in-length NULs survive.
#[uniffi::export]
pub fn decode_text_state_payload(
    payload: Vec<u8>,
) -> Result<Vec<TextState>, TextStatePayloadError> {
    Ok(core::decode_text_state_payload(&payload)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(text: &[u8]) -> Vec<u8> {
        let mut bytes: Vec<u8> = (0..16).chain(240..=255).collect();
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text);
        bytes.extend(vec![0xa5; (4 - text.len() % 4) % 4]);
        bytes
    }

    #[test]
    fn owned_projection_preserves_both_uuids_exact_utf8_bytes_order_and_duplicates() {
        let texts = ["", "A", "é", "😀\0", "é e\u{301} 😀\0tail\0", "é"];
        let payload: Vec<u8> = texts
            .iter()
            .flat_map(|text| record(text.as_bytes()))
            .collect();
        let expected = core::decode_text_state_payload(&payload).unwrap();
        let actual = decode_text_state_payload(payload).unwrap();
        assert_eq!(actual.len(), texts.len());
        for ((actual, expected), text) in actual.iter().zip(expected).zip(texts) {
            assert_eq!(actual.uuid_wire_bytes, expected.uuid().wire_bytes());
            assert_eq!(actual.uuid_native35, "03020100-0504-0706-08090a0b0c0d0e0f");
            assert_eq!(
                actual.icon_uuid_wire_bytes,
                expected.icon_uuid().wire_bytes()
            );
            assert_eq!(
                actual.icon_uuid_native35,
                "f3f2f1f0-f5f4-f7f6-f8f9fafbfcfdfeff"
            );
            assert_eq!(actual.text_byte_len, expected.text_byte_len());
            assert_eq!(actual.text_byte_len as usize, text.len());
            assert_eq!(actual.text.as_bytes(), text.as_bytes());
        }
        assert!(decode_text_state_payload(vec![]).unwrap().is_empty());
    }

    #[test]
    fn malformed_suffixes_reject_atomically_and_recover_with_exact_core_errors() {
        let prefix = record(b"private-text");
        let mut missing_text = record(b"");
        missing_text[32..36].copy_from_slice(&u32::MAX.to_le_bytes());
        let mut missing_padding = record(b"x");
        missing_padding.truncate(38);
        for suffix in [
            vec![0xa5; 35],
            missing_text,
            missing_padding,
            record(b"private-text\xff"),
            record(b"private-text\xf0\x9f"),
        ] {
            let payload = [prefix.clone(), suffix].concat();
            let expected =
                TextStatePayloadError::from(core::decode_text_state_payload(&payload).unwrap_err());
            let actual = decode_text_state_payload(payload).unwrap_err();
            assert_eq!(actual, expected);
            assert!(!actual.to_string().contains("private-text"));
            assert_eq!(
                decode_text_state_payload(prefix.clone()).unwrap()[0].text,
                "private-text"
            );
        }
    }

    #[test]
    fn all_error_fields_widen_losslessly_including_unreachable_64_bit_length_overflow() {
        let max = usize::MAX;
        let cases = [
            (
                core::TextStatePayloadError::TruncatedHeader {
                    record_offset: max,
                    available: max,
                },
                TextStatePayloadError::TruncatedHeader {
                    record_offset: max as u64,
                    available: max as u64,
                },
            ),
            (
                core::TextStatePayloadError::LengthOverflow {
                    record_offset: max,
                    declared: u32::MAX,
                },
                TextStatePayloadError::LengthOverflow {
                    record_offset: max as u64,
                    declared: u32::MAX,
                },
            ),
            (
                core::TextStatePayloadError::TruncatedText {
                    record_offset: max,
                    declared: u32::MAX,
                    available: max,
                },
                TextStatePayloadError::TruncatedText {
                    record_offset: max as u64,
                    declared: u32::MAX,
                    available: max as u64,
                },
            ),
            (
                core::TextStatePayloadError::TruncatedPadding {
                    record_offset: max,
                    expected: max,
                    available: max,
                },
                TextStatePayloadError::TruncatedPadding {
                    record_offset: max as u64,
                    expected: max as u64,
                    available: max as u64,
                },
            ),
            (
                core::TextStatePayloadError::InvalidUtf8 {
                    record_offset: max,
                    valid_up_to: max,
                    error_len: Some(max),
                },
                TextStatePayloadError::InvalidUtf8 {
                    record_offset: max as u64,
                    valid_up_to: max as u64,
                    error_len: Some(max as u64),
                },
            ),
            (
                core::TextStatePayloadError::InvalidUtf8 {
                    record_offset: max,
                    valid_up_to: max,
                    error_len: None,
                },
                TextStatePayloadError::InvalidUtf8 {
                    record_offset: max as u64,
                    valid_up_to: max as u64,
                    error_len: None,
                },
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(TextStatePayloadError::from(input), expected);
        }
    }
}
