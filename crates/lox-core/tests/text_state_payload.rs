use lox_core::numeric_state::{StateUuid, decode_value_state_payload};
use lox_core::text_state::{TextStatePayloadError, decode_text_state_payload};

const STATE: [u8; 16] = [
    0x4e, 0x83, 0xa2, 0xf9, 0xc1, 0x65, 0xb7, 0xd0, 0x0a, 0xef, 0x32, 0x96, 0x58, 0x41, 0xcb, 0x7d,
];
const ICON: [u8; 16] = [
    0x17, 0xbc, 0x49, 0x86, 0x35, 0xe2, 0x6a, 0x90, 0xf0, 0x12, 0xaa, 0x43, 0x8e, 0x59, 0xd6, 0x07,
];

fn header(state: [u8; 16], icon: [u8; 16], declared: u32) -> Vec<u8> {
    [state.as_slice(), icon.as_slice(), &declared.to_le_bytes()].concat()
}

fn record(state: [u8; 16], icon: [u8; 16], text: &[u8]) -> Vec<u8> {
    let mut bytes = header(state, icon, u32::try_from(text.len()).unwrap());
    bytes.extend_from_slice(text);
    bytes.resize(bytes.len().next_multiple_of(4), 0xff);
    bytes
}

#[test]
fn public_consumer_preserves_text_lengths_order_duplicates_and_every_padding_extent() {
    let texts = ["", "R", "ß", "水", "🌗", "\0a\0\0", "é🦀"];
    let payload: Vec<u8> = texts
        .iter()
        .flat_map(|text| record(STATE, ICON, text.as_bytes()))
        .collect();
    let states = decode_text_state_payload(&payload).unwrap();
    drop(payload);

    assert_eq!(states.len(), texts.len());
    for (state, text) in states.iter().zip(texts) {
        assert_eq!(state.text(), text);
        assert_eq!(state.text_byte_len() as usize, text.len());
        let uuid: StateUuid = state.uuid();
        assert_eq!(uuid.wire_bytes(), &STATE);
        assert_eq!(uuid.to_native35(), "f9a2834e-65c1-d0b7-0aef32965841cb7d");
        assert_eq!(state.icon_uuid().wire_bytes(), &ICON);
        assert_eq!(
            state.icon_uuid().to_native35(),
            "8649bc17-e235-906a-f012aa438e59d607"
        );
    }
    assert_eq!(states[6].text().chars().count(), 2);
    assert_eq!(states[6].text_byte_len(), 6);
}

#[test]
fn zero_and_all_one_ids_use_the_existing_numeric_uuid_contract() {
    assert!(decode_text_state_payload(&[]).unwrap().is_empty());
    let payload = [
        record([0; 16], [0xff; 16], b""),
        record([0xff; 16], [0; 16], b""),
    ]
    .concat();
    let text = decode_text_state_payload(&payload).unwrap();
    let numeric: Vec<u8> = [[0; 16], [0xff; 16]]
        .iter()
        .flat_map(|uuid| [uuid.as_slice(), &[0; 8]].concat())
        .collect();
    let numeric = decode_value_state_payload(&numeric).unwrap();
    assert_eq!(text[0].uuid(), numeric[0].uuid());
    assert_eq!(text[0].icon_uuid(), numeric[1].uuid());
    assert_eq!(text[1].uuid(), numeric[1].uuid());
    assert_eq!(text[1].icon_uuid(), numeric[0].uuid());
    assert_eq!(text[1].text_byte_len(), 0);
}

#[test]
fn every_truncated_header_is_rejected_alone_and_after_a_complete_prefix() {
    let prefix = record(STATE, ICON, b"sealed");
    let next = record(ICON, STATE, b"next");
    for available in 1..36 {
        for leading in [Vec::new(), prefix.clone()] {
            let record_offset = leading.len();
            let payload = [leading, next[..available].to_vec()].concat();
            assert_eq!(
                decode_text_state_payload(&payload),
                Err(TextStatePayloadError::TruncatedHeader {
                    record_offset,
                    available,
                })
            );
        }
    }
}

#[test]
fn truncated_text_and_all_padding_extents_never_return_a_decoded_prefix() {
    let prefix = record(STATE, ICON, b"prefix");
    for text in [b"j".as_slice(), b"kl", b"mno", b"pqrs"] {
        let next = record(ICON, STATE, text);
        for present in 0..text.len() {
            let payload = [&prefix, &next[..36 + present]].concat();
            assert_eq!(
                decode_text_state_payload(&payload),
                Err(TextStatePayloadError::TruncatedText {
                    record_offset: prefix.len(),
                    declared: text.len() as u32,
                    available: present,
                })
            );
        }
        let expected = next.len() - 36 - text.len();
        for available in 0..expected {
            let payload = [&prefix, &next[..36 + text.len() + available]].concat();
            assert_eq!(
                decode_text_state_payload(&payload),
                Err(TextStatePayloadError::TruncatedPadding {
                    record_offset: prefix.len(),
                    expected,
                    available,
                })
            );
        }
    }
}

#[test]
fn unsigned_maximum_and_high_bit_lengths_are_bounded_before_allocation() {
    let prefix = record(STATE, ICON, b"first");
    for declared in [0x8000_0000, u32::MAX] {
        let payload = [prefix.clone(), header(ICON, STATE, declared)].concat();
        assert_eq!(
            decode_text_state_payload(&payload),
            Err(TextStatePayloadError::TruncatedText {
                record_offset: prefix.len(),
                declared,
                available: 0,
            })
        );
    }
}

#[test]
fn malformed_utf8_is_an_explicit_payload_free_atomic_error() {
    let prefix = record(STATE, ICON, b"private-prefix");
    for (text, valid_up_to, error_len) in [
        (b"secret\xff".as_slice(), 6, Some(1)),
        (b"\xc0\xaf", 0, Some(1)),
        (b"\xed\xa0\x80", 0, Some(1)),
        (b"a\xf0\x9f", 1, None),
    ] {
        let payload = [prefix.clone(), record(ICON, STATE, text)].concat();
        let error = decode_text_state_payload(&payload).unwrap_err();
        assert_eq!(
            error,
            TextStatePayloadError::InvalidUtf8 {
                record_offset: prefix.len(),
                valid_up_to,
                error_len,
            }
        );
        assert!(!format!("{error} {error:?}").contains("secret"));
        assert!(!format!("{error} {error:?}").contains("private-prefix"));
        let _: &dyn std::error::Error = &error;
    }
}
