//! Raw observations of complete, already-identified Weather-State payloads.
//!
//! No framing, transport, date conversion, units or weather classification is
//! inferred. This is distinct from the HTTP `weatheru.bin` 108-byte format.

use crate::numeric_state::StateUuid;
use std::fmt;

const HEADER_BYTES: usize = 24;
const ENTRY_BYTES: usize = 68;

/// One state in wire order, including empty states and duplicate identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherState {
    uuid: StateUuid,
    last_update: u32,
    entry_count: i32,
    entries: Vec<WeatherEntry>,
}

impl WeatherState {
    pub fn uuid(&self) -> StateUuid {
        self.uuid
    }

    /// Raw unsigned wire value, not narrowed to a signed timestamp.
    pub fn last_update(&self) -> u32 {
        self.last_update
    }

    /// Original nonnegative signed count; negative declarations are rejected.
    pub fn entry_count(&self) -> i32 {
        self.entry_count
    }

    pub fn entries(&self) -> &[WeatherEntry] {
        &self.entries
    }
}

/// Raw fields without range, epoch, unit or domain validation.
///
/// The six `u64` fields are authoritative IEEE 754 bits read directly from the
/// wire. Equality is bitwise, retaining signed zero and every NaN payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl WeatherEntry {
    pub fn temperature(&self) -> f64 {
        f64::from_bits(self.temperature_bits)
    }

    pub fn perceived_temperature(&self) -> f64 {
        f64::from_bits(self.perceived_temperature_bits)
    }

    pub fn dew_point(&self) -> f64 {
        f64::from_bits(self.dew_point_bits)
    }

    pub fn precipitation(&self) -> f64 {
        f64::from_bits(self.precipitation_bits)
    }

    pub fn wind_speed(&self) -> f64 {
        f64::from_bits(self.wind_speed_bits)
    }

    pub fn barometric_pressure(&self) -> f64 {
        f64::from_bits(self.barometric_pressure_bits)
    }
}

/// Atomic rejection containing byte offsets/counts, never identifiers or data.
///
/// `record_offset` is the state's header start in the supplied unframed payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherStatePayloadError {
    TruncatedHeader {
        record_offset: usize,
        available: usize,
    },
    NegativeEntryCount {
        record_offset: usize,
        declared: i32,
    },
    LengthOverflow {
        record_offset: usize,
        declared: i32,
    },
    TruncatedEntries {
        record_offset: usize,
        declared: i32,
        expected: usize,
        available: usize,
    },
}

impl fmt::Display for WeatherStatePayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader {
                record_offset,
                available,
            } => write!(
                f,
                "weather-state header at byte {record_offset} needs 24 bytes, has {available}"
            ),
            Self::NegativeEntryCount {
                record_offset,
                declared,
            } => write!(
                f,
                "weather-state record at byte {record_offset} declares negative entry count {declared}"
            ),
            Self::LengthOverflow {
                record_offset,
                declared,
            } => write!(
                f,
                "weather-state entry count {declared} at byte {record_offset} exceeds addressable length"
            ),
            Self::TruncatedEntries {
                record_offset,
                declared,
                expected,
                available,
            } => write!(
                f,
                "weather-state record at byte {record_offset} declares {declared} entries needing {expected} bytes, has {available}"
            ),
        }
    }
}

impl std::error::Error for WeatherStatePayloadError {}

/// Decode a complete Weather-State table payload, with transport/protocol headers removed.
///
/// Each state is UUID16 + unsigned last-update LE32 + signed entry-count LE32,
/// then that many 68-byte entries (five signed LE32 fields and six LE64 f64s).
/// Empty input is valid. Order, duplicate IDs, zero-entry states, signed fields
/// and all IEEE bits are preserved. No semantic ranges or arbitrary caps apply.
///
/// The entire input's headers, counts and lengths are checked before allocating
/// any results. Any malformed suffix rejects the whole input, not a prefix.
/// This strict policy is an original API choice, not native/firmware validation
/// or compatibility with upstream's lenient prefix reader.
///
/// ```
/// use lox_core::weather_state::{decode_weather_state_payload, WeatherStatePayloadError};
/// assert!(decode_weather_state_payload(&[])?.is_empty());
/// assert_eq!(decode_weather_state_payload(&[0; 23]),
///     Err(WeatherStatePayloadError::TruncatedHeader {
///         record_offset: 0, available: 23
///     }));
/// # Ok::<(), WeatherStatePayloadError>(())
/// ```
pub fn decode_weather_state_payload(
    payload: &[u8],
) -> Result<Vec<WeatherState>, WeatherStatePayloadError> {
    let mut remaining = payload;
    let mut record_count = 0;
    while !remaining.is_empty() {
        let body_bytes = checked_body_bytes(remaining, payload.len() - remaining.len())?;
        remaining = &remaining[HEADER_BYTES..][body_bytes..];
        record_count += 1;
    }

    let mut states = Vec::with_capacity(record_count);
    remaining = payload;
    while !remaining.is_empty() {
        let (header, body) = remaining.split_at(HEADER_BYTES);
        let entry_count = i32::from_le_bytes(word(header, 20));
        // The first pass proved this count, multiplication and slice extent.
        let body_bytes = entry_count as usize * ENTRY_BYTES;
        let (entries_bytes, tail) = body.split_at(body_bytes);
        let mut uuid = [0; 16];
        uuid.copy_from_slice(&header[..16]);
        let entries = entries_bytes
            .as_chunks::<ENTRY_BYTES>()
            .0
            .iter()
            .map(|entry| WeatherEntry {
                timestamp: i32::from_le_bytes(word(entry, 0)),
                weather_type: i32::from_le_bytes(word(entry, 4)),
                wind_direction: i32::from_le_bytes(word(entry, 8)),
                solar_radiation: i32::from_le_bytes(word(entry, 12)),
                relative_humidity: i32::from_le_bytes(word(entry, 16)),
                temperature_bits: bits(entry, 20),
                perceived_temperature_bits: bits(entry, 28),
                dew_point_bits: bits(entry, 36),
                precipitation_bits: bits(entry, 44),
                wind_speed_bits: bits(entry, 52),
                barometric_pressure_bits: bits(entry, 60),
            })
            .collect();
        states.push(WeatherState {
            uuid: StateUuid::from_wire_bytes(uuid),
            last_update: u32::from_le_bytes(word(header, 16)),
            entry_count,
            entries,
        });
        remaining = tail;
    }
    Ok(states)
}

fn checked_body_bytes(
    remaining: &[u8],
    record_offset: usize,
) -> Result<usize, WeatherStatePayloadError> {
    if remaining.len() < HEADER_BYTES {
        return Err(WeatherStatePayloadError::TruncatedHeader {
            record_offset,
            available: remaining.len(),
        });
    }
    let declared = i32::from_le_bytes(word(remaining, 20));
    if declared < 0 {
        return Err(WeatherStatePayloadError::NegativeEntryCount {
            record_offset,
            declared,
        });
    }
    let expected = usize::try_from(declared)
        .ok()
        .and_then(|count| count.checked_mul(ENTRY_BYTES))
        .ok_or(WeatherStatePayloadError::LengthOverflow {
            record_offset,
            declared,
        })?;
    let available = remaining.len() - HEADER_BYTES;
    if expected > available {
        return Err(WeatherStatePayloadError::TruncatedEntries {
            record_offset,
            declared,
            expected,
            available,
        });
    }
    Ok(expected)
}

fn word(bytes: &[u8], offset: usize) -> [u8; 4] {
    let mut word = [0; 4];
    word.copy_from_slice(&bytes[offset..offset + 4]);
    word
}

fn bits(bytes: &[u8], offset: usize) -> u64 {
    let mut word = [0; 8];
    word.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(word)
}
