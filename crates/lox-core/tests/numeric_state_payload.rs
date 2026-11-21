use lox_core::numeric_state::{NumericStatePayloadError, decode_value_state_payload};

const FIRST: [u8; 24] = [
    0x93, 0xe1, 0x4b, 0xd7, 0x26, 0xfa, 0x8c, 0xb5, 0x09, 0xde, 0xa1, 0x3f, 0x72, 0x80, 0xc6, 0x54,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x2b, 0xc0,
];

#[test]
fn public_consumer_decodes_two_ordered_records_with_mixed_endian_identifiers() {
    let second = [
        0x0d, 0x02, 0x00, 0x00, 0x73, 0x00, 0x04, 0x00, 0x80, 0x01, 0x9a, 0x02, 0xfc, 0x03, 0xbd,
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x90, 0x40,
    ];
    let payload = [FIRST, second].concat();
    let states = decode_value_state_payload(&payload).unwrap();
    drop(payload);

    assert_eq!(states.len(), 2);
    assert_eq!(
        states[0].uuid().wire_bytes(),
        &[
            0x93, 0xe1, 0x4b, 0xd7, 0x26, 0xfa, 0x8c, 0xb5, 0x09, 0xde, 0xa1, 0x3f, 0x72, 0x80,
            0xc6, 0x54
        ]
    );
    assert_eq!(
        states[0].uuid().to_native35(),
        "d74be193-fa26-b58c-09dea13f7280c654"
    );
    assert_eq!(
        states[1].uuid().to_native35(),
        "0000020d-0073-0004-80019a02fc03bd04"
    );
    assert_eq!(states[0].value(), -13.625);
    assert_eq!(states[0].value_bits(), 0xc02b_4000_0000_0000);
    assert_eq!(states[1].value(), 1024.5);
    assert_eq!(states[1].value_bits(), 0x4090_0200_0000_0000);
}

#[test]
fn duplicate_states_retain_order_and_ieee_bits_without_finite_validation() {
    let cases: [([u8; 8], u64); 10] = [
        ([0, 0, 0, 0, 0, 0, 0, 0], 0x0000_0000_0000_0000),
        ([0, 0, 0, 0, 0, 0, 0, 0x80], 0x8000_0000_0000_0000),
        ([0, 0, 0, 0, 0, 0, 0xf0, 0x7f], 0x7ff0_0000_0000_0000),
        ([0, 0, 0, 0, 0, 0, 0xf0, 0xff], 0xfff0_0000_0000_0000),
        (
            [0x91, 0x62, 0x33, 0, 0, 0, 0xf8, 0x7f],
            0x7ff8_0000_0033_6291,
        ),
        ([0x46, 0x52, 0, 0, 0, 0, 0xf0, 0x7f], 0x7ff0_0000_0000_5246),
        ([0xab, 0, 0, 0, 0, 0, 0xf8, 0xff], 0xfff8_0000_0000_00ab),
        ([1, 0, 0, 0, 0, 0, 0, 0], 0x0000_0000_0000_0001),
        ([1, 0, 0, 0, 0, 0, 0, 0x80], 0x8000_0000_0000_0001),
        (
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xef, 0x7f],
            0x7fef_ffff_ffff_ffff,
        ),
    ];
    let mut payload = Vec::new();
    for (wire, _) in cases {
        payload.extend_from_slice(&FIRST[..16]);
        payload.extend_from_slice(&wire);
    }

    let states = decode_value_state_payload(&payload).unwrap();
    assert_eq!(states.len(), cases.len());
    for (state, (_, bits)) in states.iter().zip(cases) {
        assert_eq!(state.uuid().wire_bytes().as_slice(), &FIRST[..16]);
        assert_eq!(state.value_bits(), bits);
        assert_eq!(state.value().to_bits(), bits);
    }
    assert_ne!(states[0], states[1]);
}

#[test]
fn every_incomplete_tail_is_an_atomic_error_even_after_complete_records() {
    let payload = FIRST.repeat(3);
    for actual in 0..=payload.len() {
        let result = decode_value_state_payload(&payload[..actual]);
        if actual.is_multiple_of(24) {
            assert_eq!(result.unwrap().len(), actual / 24);
        } else {
            assert_eq!(
                result,
                Err(NumericStatePayloadError::InvalidLength {
                    actual,
                    trailing: actual % 24,
                }),
                "length {actual} must not return a decoded prefix"
            );
        }
    }
}
