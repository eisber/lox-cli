use lox_core::daytimer_state::{DaytimerStatePayloadError, decode_daytimer_state_payload};

const UUID: [u8; 16] = [
    0x7d, 0x92, 0x16, 0xe3, 0x5b, 0xa0, 0xe8, 0x42, 0x91, 0x37, 0xcb, 0x06, 0x84, 0xfa, 0x25, 0x6e,
];

fn header(default_bits: u64, count: i32) -> Vec<u8> {
    let mut bytes = UUID.to_vec();
    bytes.extend_from_slice(&default_bits.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes
}

fn entry(fields: [i32; 4], value_bits: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    for field in fields {
        bytes.extend_from_slice(&field.to_le_bytes());
    }
    bytes.extend_from_slice(&value_bits.to_le_bytes());
    bytes
}

#[test]
fn public_observations_preserve_signed_fields_activation_order_and_duplicates() {
    let fields = [
        [i32::MIN, -83, i32::MAX, 0],
        [17, 2001, -6, -7],
        [17, 2001, -6, 2],
        [17, 2001, -6, 1],
    ];
    let mut payload = header(0x8000_0000_0000_0000, 5);
    for field in fields {
        payload.extend(entry(field, 0xc031_4000_0000_0000));
    }
    payload.extend(entry(fields[1], 0xc031_4000_0000_0000));
    payload.extend(header(0x7ff0_0000_0000_3ba5, 0));
    let states = decode_daytimer_state_payload(&payload).unwrap();
    drop(payload);

    assert_eq!(states.len(), 2);
    assert_eq!(states[0].uuid(), states[1].uuid());
    assert_eq!(states[0].uuid().wire_bytes(), &UUID);
    assert_eq!(
        states[0].uuid().to_native35(),
        "e316927d-a05b-42e8-9137cb0684fa256e"
    );
    assert_eq!(states[0].default_value_bits(), 0x8000_0000_0000_0000);
    assert_eq!(states[0].default_value().to_bits(), 0x8000_0000_0000_0000);
    assert_eq!(states[1].default_value_bits(), 0x7ff0_0000_0000_3ba5);
    assert!(states[1].entries().is_empty());
    let entries = states[0].entries();
    assert_eq!(entries.len(), 5);
    for (decoded, expected) in entries.iter().zip(fields) {
        assert_eq!(
            [
                decoded.mode(),
                decoded.from_minutes(),
                decoded.to_minutes(),
                decoded.activation_word()
            ],
            expected
        );
        assert_eq!(decoded.needs_activation(), expected[3] != 0);
        assert_eq!(decoded.value_bits(), 0xc031_4000_0000_0000);
        assert_eq!(decoded.value(), -17.25);
    }
    assert_eq!(entries[1], entries[4]);
    assert_ne!(entries[1], entries[2]);
}

#[test]
fn ieee_bits_are_authoritative_for_both_defaults_and_entries() {
    let patterns = [
        0,
        0x8000_0000_0000_0000,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_4e27,
        0x7ff0_0000_0000_67b1,
        0xfff8_0000_0000_9512,
        1,
        0x8000_0000_0000_0001,
        0x7fef_ffff_ffff_ffff,
    ];
    let mut payload = Vec::new();
    for bits in patterns {
        payload.extend(header(bits, 1));
        payload.extend(entry([4, 5, 6, i32::MIN], bits));
    }
    let states = decode_daytimer_state_payload(&payload).unwrap();
    assert_eq!(states.len(), patterns.len());
    for (state, bits) in states.iter().zip(patterns) {
        assert_eq!(state.default_value_bits(), bits);
        assert_eq!(state.entries()[0].value_bits(), bits);
        assert_eq!(state.entries()[0].activation_word(), i32::MIN);
        assert!(state.entries()[0].needs_activation());
    }
    assert_ne!(states[0], states[1]);
}

#[test]
fn every_truncation_is_atomic_even_after_a_complete_state() {
    let mut payload = header(0, 2);
    payload.extend(entry([-3, 8, 7, 0], 1));
    payload.extend(entry([9, 0, -1, 1], 2));
    let boundary = payload.len();
    payload.extend(header(3, 0));
    for actual in 0..=payload.len() {
        let result = decode_daytimer_state_payload(&payload[..actual]);
        match actual {
            0 => assert!(result.unwrap().is_empty()),
            n if n == boundary => assert_eq!(result.unwrap().len(), 1),
            n if n == payload.len() => assert_eq!(result.unwrap().len(), 2),
            n if n < 28 => assert_eq!(
                result,
                Err(DaytimerStatePayloadError::TruncatedHeader {
                    record_offset: 0,
                    available: n,
                })
            ),
            n if n < boundary => assert_eq!(
                result,
                Err(DaytimerStatePayloadError::TruncatedEntries {
                    record_offset: 0,
                    declared: 2,
                    available: n - 28,
                })
            ),
            n => assert_eq!(
                result,
                Err(DaytimerStatePayloadError::TruncatedHeader {
                    record_offset: boundary,
                    available: n - boundary,
                })
            ),
        }
    }
}

#[test]
fn signed_counts_and_maximum_extent_are_rejected_without_count_based_allocation() {
    for prefix in [Vec::new(), header(0, 0)] {
        for declared in [-1, i32::MIN] {
            let payload = [prefix.clone(), header(0, declared)].concat();
            assert_eq!(
                decode_daytimer_state_payload(&payload),
                Err(DaytimerStatePayloadError::NegativeEntryCount {
                    record_offset: prefix.len(),
                    declared,
                })
            );
        }
        let payload = [prefix.clone(), header(0, i32::MAX)].concat();
        let expected = if usize::try_from(i32::MAX)
            .ok()
            .and_then(|n| n.checked_mul(24))
            .is_none()
        {
            DaytimerStatePayloadError::LengthOverflow {
                record_offset: prefix.len(),
                declared: i32::MAX,
            }
        } else {
            DaytimerStatePayloadError::TruncatedEntries {
                record_offset: prefix.len(),
                declared: i32::MAX,
                available: 0,
            }
        };
        assert_eq!(decode_daytimer_state_payload(&payload), Err(expected));
    }
}

#[test]
fn declared_count_controls_record_boundary_without_ignoring_extra_bytes() {
    let mut payload = header(0, 1);
    payload.extend(entry([0, 1, 2, 0], 0));
    payload.extend(entry([3, 4, 5, 0], 0));
    assert_eq!(
        decode_daytimer_state_payload(&payload),
        Err(DaytimerStatePayloadError::TruncatedHeader {
            record_offset: 52,
            available: 24,
        })
    );
    payload[24..28].copy_from_slice(&2_i32.to_le_bytes());
    assert_eq!(
        decode_daytimer_state_payload(&payload).unwrap()[0]
            .entries()
            .len(),
        2
    );
    payload[24..28].copy_from_slice(&3_i32.to_le_bytes());
    assert_eq!(
        decode_daytimer_state_payload(&payload),
        Err(DaytimerStatePayloadError::TruncatedEntries {
            record_offset: 0,
            declared: 3,
            available: 48,
        })
    );
}
