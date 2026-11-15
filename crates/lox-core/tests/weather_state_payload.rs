use lox_core::weather_state::{
    WeatherStatePayloadError as Error, decode_weather_state_payload as decode,
};

const UUID: [u8; 16] = [
    0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
];

fn header(last_update: u32, count: i32) -> Vec<u8> {
    let mut bytes = UUID.to_vec();
    bytes.extend(last_update.to_le_bytes());
    bytes.extend(count.to_le_bytes());
    bytes
}

fn entry(ints: [i32; 5], bits: [u64; 6]) -> Vec<u8> {
    ints.into_iter()
        .flat_map(i32::to_le_bytes)
        .chain(bits.into_iter().flat_map(u64::to_le_bytes))
        .collect()
}

#[test]
fn raw_signed_fields_unsigned_update_and_ieee_identity() {
    let ints = [i32::MIN, -17, i32::MAX, -3, -1024];
    let raw = [
        0x8000_0000_0000_0000,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff0_0000_0000_0042,
        0xfff8_0000_0000_abcd,
        1,
    ];
    let mut bytes = header(u32::MAX, 1);
    bytes.extend(entry(ints, raw));
    let states = decode(&bytes).unwrap();
    let state = &states[0];
    assert_eq!(state.last_update(), u32::MAX);
    assert_eq!(state.uuid().wire_bytes(), &UUID);
    assert_eq!(
        state.uuid().to_native35(),
        "76543210-ba98-fedc-0123456789abcdef"
    );
    assert_eq!(state.entry_count(), 1);
    let e = &state.entries()[0];
    assert_eq!(
        [
            e.timestamp,
            e.weather_type,
            e.wind_direction,
            e.solar_radiation,
            e.relative_humidity
        ],
        ints
    );
    assert_eq!(
        [
            e.temperature_bits,
            e.perceived_temperature_bits,
            e.dew_point_bits,
            e.precipitation_bits,
            e.wind_speed_bits,
            e.barometric_pressure_bits
        ],
        raw
    );
    assert!(e.temperature().is_sign_negative());
    assert_eq!(e.perceived_temperature(), f64::INFINITY);
    assert_eq!(e.dew_point(), f64::NEG_INFINITY);
    assert!(e.precipitation().is_nan());
    assert!(e.wind_speed().is_nan());
    assert_eq!(e.barometric_pressure().to_bits(), 1);
    assert_eq!(decode(&bytes).unwrap(), states);
    // A distinct NaN payload is a distinct observation, irrespective of float equality.
    bytes[24 + 44] ^= 1;
    assert_ne!(decode(&bytes).unwrap(), states);
}

#[test]
fn preserves_order_duplicate_ids_zero_entries_and_entry_order() {
    assert!(decode(&[]).unwrap().is_empty());
    let mut bytes = header(0x8000_0000, 0);
    bytes.extend(header(7, 2));
    bytes.extend(entry([19, 2, 3, 4, 5], [0; 6]));
    bytes.extend(entry([-29, 6, 7, 8, 9], [0x3ff8_0000_0000_0000; 6]));
    bytes.extend(header(0, 0));
    let states = decode(&bytes).unwrap();
    assert_eq!(states.len(), 3);
    assert!(states.iter().all(|s| s.uuid() == states[0].uuid()));
    assert_eq!(states[0].last_update(), 0x8000_0000);
    assert!(states[0].entries().is_empty());
    assert_eq!(states[1].entry_count(), 2);
    assert_eq!(states[1].entries()[0].timestamp, 19);
    assert_eq!(states[1].entries()[1].timestamp, -29);
    assert_eq!(states[1].entries()[1].temperature(), 1.5);
    assert_eq!(states[2].last_update(), 0);
    assert!(states[2].entries().is_empty());
}

#[test]
fn every_truncation_is_atomic_with_exact_offsets_and_counts() {
    let prefix = header(43, 0);
    let mut record = header(99, 2);
    record.extend(entry([0; 5], [0; 6]));
    record.extend(entry([0; 5], [0; 6]));
    for len in 1..record.len() {
        let mut bytes = prefix.clone();
        bytes.extend_from_slice(&record[..len]);
        let expected = if len < 24 {
            Error::TruncatedHeader {
                record_offset: 24,
                available: len,
            }
        } else {
            Error::TruncatedEntries {
                record_offset: 24,
                declared: 2,
                expected: 136,
                available: len - 24,
            }
        };
        assert_eq!(decode(&bytes), Err(expected), "cut at {len}");
    }
    for len in 1..24 {
        assert_eq!(
            decode(&record[..len]),
            Err(Error::TruncatedHeader {
                record_offset: 0,
                available: len
            })
        );
    }
    let mut complete = prefix;
    complete.extend(record);
    assert_eq!(decode(&complete).unwrap().len(), 2);
}

#[test]
fn rejects_negative_and_impossible_counts_without_payload_in_errors() {
    for count in [-1, i32::MIN] {
        let mut bytes = header(0, 0);
        bytes.extend(header(0, count));
        let error = Error::NegativeEntryCount {
            record_offset: 24,
            declared: count,
        };
        assert_eq!(decode(&bytes), Err(error));
        let rendered = format!("{error} {error:?}");
        assert!(!rendered.contains("76543210"));
    }
    let error = decode(&header(0, i32::MAX)).unwrap_err();
    match (
        usize::try_from(i32::MAX)
            .ok()
            .and_then(|n| n.checked_mul(68)),
        error,
    ) {
        (
            Some(expected),
            Error::TruncatedEntries {
                record_offset: 0,
                declared: i32::MAX,
                expected: actual,
                available: 0,
            },
        ) => assert_eq!(actual, expected),
        (
            None,
            Error::LengthOverflow {
                record_offset: 0,
                declared: i32::MAX,
            },
        ) => {}
        other => panic!("unexpected checked length result: {other:?}"),
    }
}
