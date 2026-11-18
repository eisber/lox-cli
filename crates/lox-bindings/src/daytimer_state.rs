//! Owned observations from Core's strict, payload-only Daytimer decoder.

use lox_core::daytimer_state as core;

/// IEEE bits are authoritative: Python exposes unsigned integers; Java/Kotlin
/// expose signed long/Long values carrying the identical 64-bit patterns.
#[derive(Clone, Debug, uniffi::Record)]
pub struct DaytimerEntry {
    pub mode: i32,
    pub from_minutes: i32,
    pub to_minutes: i32,
    pub activation_word: i32,
    pub needs_activation: bool,
    pub value_bits: u64,
}

impl From<&core::DaytimerEntry> for DaytimerEntry {
    fn from(entry: &core::DaytimerEntry) -> Self {
        Self {
            mode: entry.mode(),
            from_minutes: entry.from_minutes(),
            to_minutes: entry.to_minutes(),
            activation_word: entry.activation_word(),
            needs_activation: entry.needs_activation(),
            value_bits: entry.value_bits(),
        }
    }
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct DaytimerState {
    pub uuid_wire_bytes: Vec<u8>,
    pub uuid_native35: String,
    pub default_value_bits: u64,
    pub entries: Vec<DaytimerEntry>,
}

impl From<core::DaytimerState> for DaytimerState {
    fn from(state: core::DaytimerState) -> Self {
        Self {
            uuid_wire_bytes: state.uuid().wire_bytes().to_vec(),
            uuid_native35: state.uuid().to_native35(),
            default_value_bits: state.default_value_bits(),
            entries: state.entries().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum DaytimerStatePayloadError {
    #[error("daytimer-state header at byte {record_offset} needs 28 bytes, has {available}")]
    TruncatedHeader { record_offset: u64, available: u64 },
    #[error(
        "daytimer-state record at byte {record_offset} declares negative entry count {declared}"
    )]
    NegativeEntryCount { record_offset: u64, declared: i32 },
    #[error(
        "daytimer-state entry count {declared} at byte {record_offset} has an unrepresentable byte extent"
    )]
    LengthOverflow { record_offset: u64, declared: i32 },
    #[error(
        "daytimer-state record at byte {record_offset} declares {declared} entries of 24 bytes, has {available} bytes"
    )]
    TruncatedEntries {
        record_offset: u64,
        declared: i32,
        available: u64,
    },
}

impl From<core::DaytimerStatePayloadError> for DaytimerStatePayloadError {
    fn from(error: core::DaytimerStatePayloadError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match error {
            core::DaytimerStatePayloadError::TruncatedHeader {
                record_offset,
                available,
            } => Self::TruncatedHeader {
                record_offset: record_offset as u64,
                available: available as u64,
            },
            core::DaytimerStatePayloadError::NegativeEntryCount {
                record_offset,
                declared,
            } => Self::NegativeEntryCount {
                record_offset: record_offset as u64,
                declared,
            },
            core::DaytimerStatePayloadError::LengthOverflow {
                record_offset,
                declared,
            } => Self::LengthOverflow {
                record_offset: record_offset as u64,
                declared,
            },
            core::DaytimerStatePayloadError::TruncatedEntries {
                record_offset,
                declared,
                available,
            } => Self::TruncatedEntries {
                record_offset: record_offset as u64,
                declared,
                available: available as u64,
            },
        }
    }
}

/// Decode a complete already-identified payload with framing removed. Empty is
/// valid; errors reject the entire payload. No schedule/time/finite validation.
#[uniffi::export]
pub fn decode_daytimer_state_payload(
    payload: Vec<u8>,
) -> Result<Vec<DaytimerState>, DaytimerStatePayloadError> {
    Ok(core::decode_daytimer_state_payload(&payload)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(bits: u64, count: i32) -> Vec<u8> {
        let mut bytes: Vec<u8> = (0..16).collect();
        bytes.extend(bits.to_le_bytes());
        bytes.extend(count.to_le_bytes());
        bytes
    }

    #[test]
    fn owned_projection_preserves_signed_fields_bits_order_and_duplicates() {
        let bits = [
            0,
            0x8000_0000_0000_0000,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff0_0000_0000_1234,
            0xfff8_1234_5678_9abc,
            u64::MAX,
        ];
        let mut payload = Vec::new();
        for value in bits {
            payload.extend(header(value, 3));
            for activation in [0_i32, -7, -7] {
                for field in [i32::MIN, -83, i32::MAX, activation] {
                    payload.extend(field.to_le_bytes());
                }
                payload.extend(value.to_le_bytes());
            }
        }
        payload.extend(header(1, 0));
        let expected = core::decode_daytimer_state_payload(&payload).unwrap();
        let actual = decode_daytimer_state_payload(payload).unwrap();
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.uuid_wire_bytes, expected.uuid().wire_bytes());
            assert_eq!(actual.uuid_native35, expected.uuid().to_native35());
            assert_eq!(actual.default_value_bits, expected.default_value_bits());
            assert_eq!(actual.entries.len(), expected.entries().len());
            for (actual, expected) in actual.entries.iter().zip(expected.entries()) {
                assert_eq!(actual.mode, expected.mode());
                assert_eq!(actual.from_minutes, expected.from_minutes());
                assert_eq!(actual.to_minutes, expected.to_minutes());
                assert_eq!(actual.activation_word, expected.activation_word());
                assert_eq!(actual.needs_activation, expected.needs_activation());
                assert_eq!(actual.value_bits, expected.value_bits());
            }
        }
        assert!(actual.last().unwrap().entries.is_empty());
    }

    #[test]
    fn atomic_errors_preserve_every_context_and_recover() {
        let mut valid = header(0, 1);
        valid.extend([0; 24]);
        for length in 1..valid.len() {
            let input = [header(0, 0), valid[..length].to_vec()].concat();
            let expected = core::decode_daytimer_state_payload(&input).unwrap_err();
            assert_eq!(
                decode_daytimer_state_payload(input).unwrap_err(),
                DaytimerStatePayloadError::from(expected)
            );
            assert!(decode_daytimer_state_payload(vec![]).unwrap().is_empty());
        }
        for declared in [i32::MIN, -1, i32::MAX] {
            let input = [header(0, 0), header(0, declared)].concat();
            let expected = core::decode_daytimer_state_payload(&input).unwrap_err();
            assert_eq!(
                decode_daytimer_state_payload(input).unwrap_err(),
                DaytimerStatePayloadError::from(expected)
            );
        }
        assert_eq!(
            DaytimerStatePayloadError::from(core::DaytimerStatePayloadError::LengthOverflow {
                record_offset: usize::MAX,
                declared: i32::MAX,
            }),
            DaytimerStatePayloadError::LengthOverflow {
                record_offset: usize::MAX as u64,
                declared: i32::MAX,
            }
        );
    }
}
