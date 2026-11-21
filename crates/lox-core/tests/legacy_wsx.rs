use lox_core::legacy_wsx::{Credentials, Nonce, encode_handshake};

#[path = "support/legacy_wsx_cases.rs"]
mod fixtures;

#[test]
fn legacy_wsx_typed_consumer_matches_pre_extraction_packets() {
    let password = String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
    for (index, case) in fixtures::cases(&password).into_iter().enumerate() {
        let credentials = Credentials::new(case.username, case.password, case.locale);
        let nonce = Nonce::from_random_bytes(case.timestamp, fixtures::randomness(index));
        let packet = encode_handshake(&credentials, &nonce);
        assert_eq!(
            packet,
            fixtures::decode_packet(case.packet),
            "{}",
            case.name
        );
        assert_eq!(nonce.timestamp(), case.timestamp, "{}", case.name);
        let captured = Nonce::from_bytes(packet[..16].try_into().unwrap());
        assert_eq!(encode_handshake(&credentials, &captured), packet);
    }
}

#[test]
fn legacy_wsx_nonce_randomness_only_affects_twelve_nonce_bytes() {
    let credentials = Credentials::new("synthetic", "password", "ENU");
    for timestamp in [0, 1, 0x01020304, 0x80000000, u32::MAX] {
        let nonce = Nonce::from_random_bytes(timestamp, [0; 16]);
        let mut overwritten = [0; 16];
        for pos in [3, 4, 9, 14] {
            overwritten[pos] = 0xff;
        }
        assert_eq!(
            nonce.as_bytes(),
            Nonce::from_random_bytes(timestamp, overwritten).as_bytes()
        );
        let different = Nonce::from_random_bytes(timestamp, [0xff; 16]);
        assert_ne!(nonce.as_bytes(), different.as_bytes());
        assert_eq!(
            &encode_handshake(&credentials, &nonce)[16..],
            &encode_handshake(&credentials, &different)[16..]
        );
        assert_eq!(different.timestamp(), timestamp);
    }
}

#[test]
fn legacy_wsx_debug_does_not_disclose_fields_or_timestamp_key() {
    let credentials = Credentials::new("secret-user", "secret-password", "secret-locale");
    let nonce = Nonce::from_random_bytes(0x12345678, [0xab; 16]);
    assert_eq!(format!("{credentials:?}"), "Credentials([REDACTED])");
    assert_eq!(format!("{nonce:?}"), "Nonce([REDACTED])");
}
