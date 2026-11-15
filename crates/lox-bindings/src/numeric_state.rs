//! Owned native projections of Core's payload-only numeric decoder.

use lox_core::numeric_state as core;

/// `value_bits` is authoritative, even if a host floating ABI quiets a NaN.
/// UniFFI maps u64 to a nonnegative Python int and a signed Java long carrying
/// the same 64 bits (Kotlin Long); no numeric signed conversion is intended.
#[derive(Clone, Debug, uniffi::Record)]
pub struct NumericState {
    pub uuid_wire_bytes: Vec<u8>,
    pub uuid_native35: String,
    pub value: f64,
    pub value_bits: u64,
}

impl From<core::NumericState> for NumericState {
    fn from(state: core::NumericState) -> Self {
        Self {
            uuid_wire_bytes: state.uuid().wire_bytes().to_vec(),
            uuid_native35: state.uuid().to_native35(),
            value: state.value(),
            value_bits: state.value_bits(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum NumericStatePayloadError {
    #[error(
        "numeric value-state payload has {actual} bytes with {trailing} trailing bytes; expected whole 24-byte records"
    )]
    InvalidLength { actual: u64, trailing: u64 },
}

impl From<core::NumericStatePayloadError> for NumericStatePayloadError {
    fn from(error: core::NumericStatePayloadError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match error {
            core::NumericStatePayloadError::InvalidLength { actual, trailing } => {
                Self::InvalidLength {
                    actual: actual as u64,
                    trailing: trailing as u64,
                }
            }
        }
    }
}

/// Decode an already-identified, assembled numeric table with headers removed.
/// Empty is valid; incomplete tails reject the entire payload. No framing,
/// transport, type detection, subscription or finite-value validation.
#[uniffi::export]
pub fn decode_value_state_payload(
    payload: Vec<u8>,
) -> Result<Vec<NumericState>, NumericStatePayloadError> {
    Ok(core::decode_value_state_payload(&payload)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_owned_uuid_and_ieee_projection_preserves_order_and_duplicates() {
        let bits = [
            0x3ff0_0000_0000_0000_u64,
            1,
            0x8000_0000_0000_0000,
            0x7ff0_0000_0000_0001,
            0xfff8_1234_5678_9abc,
            0xffff_ffff_ffff_ffff,
        ];
        let mut payload = Vec::new();
        for value in bits {
            payload.extend(0_u8..16);
            payload.extend(value.to_le_bytes());
        }
        let expected = core::decode_value_state_payload(&payload).unwrap();
        let actual = decode_value_state_payload(payload).unwrap();
        for ((actual, expected), bits) in actual.iter().zip(expected).zip(bits) {
            assert_eq!(actual.uuid_wire_bytes, expected.uuid().wire_bytes());
            assert_eq!(actual.uuid_native35, "03020100-0504-0706-08090a0b0c0d0e0f");
            assert_eq!(actual.value_bits, bits);
            assert_eq!(actual.value.to_bits(), bits);
        }
    }

    #[test]
    fn malformed_payloads_are_atomic_structured_and_recoverable() {
        for length in (1..=49).filter(|length| length % 24 != 0) {
            assert_eq!(
                decode_value_state_payload(vec![0xa5; length]).unwrap_err(),
                NumericStatePayloadError::InvalidLength {
                    actual: length as u64,
                    trailing: length as u64 % 24,
                }
            );
            assert!(decode_value_state_payload(vec![]).unwrap().is_empty());
        }
        let error = NumericStatePayloadError::from(core::NumericStatePayloadError::InvalidLength {
            actual: usize::MAX,
            trailing: 23,
        });
        assert_eq!(
            error,
            NumericStatePayloadError::InvalidLength {
                actual: usize::MAX as u64,
                trailing: 23
            }
        );
    }
}
