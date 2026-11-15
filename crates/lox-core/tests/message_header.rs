use lox_core::message_header::{
    MessageHeader, MessageHeaderError, MessageType, decode_message_header,
};
use lox_core::numeric_state::decode_value_state_payload;
use lox_core::text_state::decode_text_state_payload;

#[test]
fn observes_unsigned_fields_and_preserves_uninterpreted_information() {
    let bytes = [3, 0xe7, 0xfe, 0x91, 0x76, 0x54, 0x32, 0xf0];
    let header = decode_message_header(&bytes).unwrap();
    assert_eq!(header.message_type(), 0xe7);
    assert_eq!(header.known_type(), None);
    assert_eq!(header.info_flags(), 0xfe);
    assert_eq!(header.reserved(), 0x91);
    assert_eq!(header.payload_size(), 0xf032_5476);
    assert!(!header.size_is_estimated());
    assert_eq!(header.to_wire_bytes(), bytes);

    let estimated = decode_message_header(&[3, 0xff, 0xff, 0xff, 255, 255, 255, 255]).unwrap();
    assert!(estimated.size_is_estimated());
    assert_eq!(estimated.payload_size(), u32::MAX);
    assert_eq!(
        estimated.to_wire_bytes(),
        [3, 0xff, 0xff, 0xff, 255, 255, 255, 255]
    );
}

#[test]
fn rejects_incomplete_and_overlong_headers_before_checking_magic() {
    for len in [0, 1, 2, 7, 9, 32] {
        assert_eq!(
            decode_message_header(&vec![0; len]),
            Err(MessageHeaderError::InvalidLength { actual: len })
        );
    }
    assert_eq!(
        decode_message_header(&[0x83, 2, 0, 0, 24, 0, 0, 0]),
        Err(MessageHeaderError::InvalidMagic { actual: 0x83 })
    );
}

#[test]
fn classifies_documented_types_without_inventing_payload_rules() {
    let types = [
        MessageType::Text,
        MessageType::BinaryFile,
        MessageType::ValueStates,
        MessageType::TextStates,
        MessageType::DaytimerStates,
        MessageType::OutOfService,
        MessageType::KeepAlive,
        MessageType::WeatherStates,
    ];
    for kind in types {
        let header = MessageHeader::new(kind as u8, 0x81, 0x42, 0);
        assert_eq!(decode_message_header(&header.to_wire_bytes()), Ok(header));
        assert_eq!(header.known_type(), Some(kind));
        assert!(header.size_is_estimated());
    }
    // Observing a nonzero keepalive size is not certifying a valid keepalive.
    let header = decode_message_header(&[3, 6, 0, 0, 17, 0, 0, 0]).unwrap();
    assert_eq!(header.known_type(), Some(MessageType::KeepAlive));
    assert_eq!(header.payload_size(), 17);
}

#[test]
fn caller_explicitly_selects_already_assembled_payload_decoders() {
    let numeric_header = decode_message_header(&[3, 2, 0, 0, 24, 0, 0, 0]).unwrap();
    let mut numeric_payload = [0; 24];
    numeric_payload[16..].copy_from_slice(&12.5_f64.to_le_bytes());
    assert_eq!(numeric_header.known_type(), Some(MessageType::ValueStates));
    assert_eq!(
        numeric_header.payload_size() as usize,
        numeric_payload.len()
    );
    let values = decode_value_state_payload(&numeric_payload).unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].value(), 12.5);

    let text_header = decode_message_header(&[3, 3, 0, 0, 40, 0, 0, 0]).unwrap();
    let mut text_payload = [0; 40];
    text_payload[32..36].copy_from_slice(&3_u32.to_le_bytes());
    text_payload[36..39].copy_from_slice(b"new");
    text_payload[39] = 0xa5;
    assert_eq!(text_header.known_type(), Some(MessageType::TextStates));
    let texts = decode_text_state_payload(&text_payload).unwrap();
    assert_eq!(texts[0].text(), "new");

    let mut combined = numeric_header.to_wire_bytes().to_vec();
    combined.extend_from_slice(&numeric_payload);
    assert_eq!(
        decode_message_header(&combined),
        Err(MessageHeaderError::InvalidLength { actual: 32 })
    );
}
