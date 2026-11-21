//! Owned native projections of Core's strict, payload-only weather decoder.

use lox_core::weather_state as core;

/// Host-owned snapshot; no native handle or close obligation.
/// `last_update` is an unsigned Python int; JVM int/Int carries its raw 32 bits.
#[derive(Clone, Debug, uniffi::Record)]
pub struct WeatherState {
    pub uuid_wire_bytes: Vec<u8>,
    pub uuid_native35: String,
    pub last_update: u32,
    pub entry_count: i32,
    pub entries: Vec<WeatherEntry>,
}

/// All six IEEE fields are authoritative raw bits, including NaN payloads.
/// Python uses unsigned int; Java long/Kotlin Long carries identical 64 bits.
#[derive(Clone, Debug, uniffi::Record)]
pub struct WeatherEntry {
    pub timestamp: i32,
    pub weather_type: i32,
    pub wind_direction: i32,
    pub solar_radiation: i32,
    pub relative_humidity: i32,
    pub temperature_bits: u64,
    pub perceived_temperature_bits: u64,
    pub dew_point_bits: u64,
    pub precipitation_bits: u64,
    pub wind_speed_bits: u64,
    pub barometric_pressure_bits: u64,
}

impl From<core::WeatherEntry> for WeatherEntry {
    fn from(entry: core::WeatherEntry) -> Self {
        Self {
            timestamp: entry.timestamp,
            weather_type: entry.weather_type,
            wind_direction: entry.wind_direction,
            solar_radiation: entry.solar_radiation,
            relative_humidity: entry.relative_humidity,
            temperature_bits: entry.temperature_bits,
            perceived_temperature_bits: entry.perceived_temperature_bits,
            dew_point_bits: entry.dew_point_bits,
            precipitation_bits: entry.precipitation_bits,
            wind_speed_bits: entry.wind_speed_bits,
            barometric_pressure_bits: entry.barometric_pressure_bits,
        }
    }
}

impl From<core::WeatherState> for WeatherState {
    fn from(state: core::WeatherState) -> Self {
        Self {
            uuid_wire_bytes: state.uuid().wire_bytes().to_vec(),
            uuid_native35: state.uuid().to_native35(),
            last_update: state.last_update(),
            entry_count: state.entry_count(),
            entries: state.entries().iter().copied().map(Into::into).collect(),
        }
    }
}

/// Complete-input failures without identifiers or payload contents.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum WeatherStatePayloadError {
    #[error("weather-state header at byte {record_offset} needs 24 bytes, has {available}")]
    TruncatedHeader { record_offset: u64, available: u64 },
    #[error(
        "weather-state record at byte {record_offset} declares negative entry count {declared}"
    )]
    NegativeEntryCount { record_offset: u64, declared: i32 },
    #[error(
        "weather-state entry count {declared} at byte {record_offset} exceeds addressable length"
    )]
    LengthOverflow { record_offset: u64, declared: i32 },
    #[error(
        "weather-state record at byte {record_offset} declares {declared} entries needing {expected} bytes, has {available}"
    )]
    TruncatedEntries {
        record_offset: u64,
        declared: i32,
        expected: u64,
        available: u64,
    },
}

impl From<core::WeatherStatePayloadError> for WeatherStatePayloadError {
    fn from(error: core::WeatherStatePayloadError) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        match error {
            core::WeatherStatePayloadError::TruncatedHeader {
                record_offset,
                available,
            } => Self::TruncatedHeader {
                record_offset: record_offset as u64,
                available: available as u64,
            },
            core::WeatherStatePayloadError::NegativeEntryCount {
                record_offset,
                declared,
            } => Self::NegativeEntryCount {
                record_offset: record_offset as u64,
                declared,
            },
            core::WeatherStatePayloadError::LengthOverflow {
                record_offset,
                declared,
            } => Self::LengthOverflow {
                record_offset: record_offset as u64,
                declared,
            },
            core::WeatherStatePayloadError::TruncatedEntries {
                record_offset,
                declared,
                expected,
                available,
            } => Self::TruncatedEntries {
                record_offset: record_offset as u64,
                declared,
                expected: expected as u64,
                available: available as u64,
            },
        }
    }
}

/// Decode an identified complete Weather-State table after removing headers.
/// Core alone validates counts and lengths atomically. No units, epoch, timezone,
/// finiteness, framing or HTTP weatheru.bin interpretation is added.
#[uniffi::export]
pub fn decode_weather_state_payload(
    payload: Vec<u8>,
) -> Result<Vec<WeatherState>, WeatherStatePayloadError> {
    Ok(core::decode_weather_state_payload(&payload)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(count: i32) -> Vec<u8> {
        let mut bytes: Vec<u8> = (0..16).collect();
        bytes.extend(u32::MAX.to_le_bytes());
        bytes.extend(count.to_le_bytes());
        bytes
    }

    #[test]
    fn owned_projection_preserves_every_signed_field_ieee_bits_and_duplicate_order() {
        let ints = [i32::MIN, -83, i32::MAX, -7, -1024];
        let bits = [
            0x8000_0000_0000_0000,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff0_0000_0000_1234,
            0xfff8_1234_5678_9abc,
            1_u64,
        ];
        let mut payload = header(0);
        payload.extend(header(2));
        for _ in 0..2 {
            payload.extend(ints.into_iter().flat_map(i32::to_le_bytes));
            payload.extend(bits.into_iter().flat_map(u64::to_le_bytes));
        }
        payload.extend(header(0));
        let actual = decode_weather_state_payload(payload.clone()).unwrap();
        let expected = core::decode_weather_state_payload(&payload).unwrap();
        assert_eq!(actual.len(), 3);
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.uuid_wire_bytes, expected.uuid().wire_bytes());
            assert_eq!(actual.uuid_native35, "03020100-0504-0706-08090a0b0c0d0e0f");
            assert_eq!(actual.last_update, u32::MAX);
            assert_eq!(actual.entry_count, expected.entry_count());
            for entry in &actual.entries {
                assert_eq!(
                    [
                        entry.timestamp,
                        entry.weather_type,
                        entry.wind_direction,
                        entry.solar_radiation,
                        entry.relative_humidity
                    ],
                    ints
                );
                assert_eq!(
                    [
                        entry.temperature_bits,
                        entry.perceived_temperature_bits,
                        entry.dew_point_bits,
                        entry.precipitation_bits,
                        entry.wind_speed_bits,
                        entry.barometric_pressure_bits
                    ],
                    bits
                );
            }
        }
        assert!(decode_weather_state_payload(vec![]).unwrap().is_empty());
    }

    #[test]
    fn malformed_suffixes_are_atomic_and_all_error_fields_widen_losslessly() {
        for suffix in [
            vec![0xa5; 23],
            header(-1),
            header(i32::MIN),
            header(i32::MAX),
        ] {
            let payload = [header(0), suffix].concat();
            let expected = WeatherStatePayloadError::from(
                core::decode_weather_state_payload(&payload).unwrap_err(),
            );
            let actual = decode_weather_state_payload(payload).unwrap_err();
            assert_eq!(actual, expected);
            assert!(!actual.to_string().contains("03020100"));
            assert_eq!(decode_weather_state_payload(header(0)).unwrap().len(), 1);
        }
        let max = usize::MAX;
        assert_eq!(
            WeatherStatePayloadError::from(core::WeatherStatePayloadError::LengthOverflow {
                record_offset: max,
                declared: i32::MAX,
            }),
            WeatherStatePayloadError::LengthOverflow {
                record_offset: max as u64,
                declared: i32::MAX,
            }
        );
        assert_eq!(
            WeatherStatePayloadError::from(core::WeatherStatePayloadError::TruncatedEntries {
                record_offset: max,
                declared: i32::MAX,
                expected: max,
                available: max,
            }),
            WeatherStatePayloadError::TruncatedEntries {
                record_offset: max as u64,
                declared: i32::MAX,
                expected: max as u64,
                available: max as u64,
            }
        );
    }
}
