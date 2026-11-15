//! Existing legacy `/wsx` handshake compatibility, not RFC6455 or token auth.
//!
//! The caller supplies timestamp and randomness. This reuses the repository's
//! timestamp-derived 16-round RC6 transform, in the decryption direction before
//! transmission; it is not a security or firmware-compatibility guarantee.
//! Inputs and output are not logged or zeroized.

use std::fmt;

mod rc6;

use rc6::Rc6Key;

/// Borrowed UTF-8 fields, concatenated verbatim with NUL separators.
///
/// Embedded NULs are accepted. The concatenated bytes, including separators,
/// are truncated to 32 bytes (even within a UTF-8 character), then zero-padded.
pub struct Credentials<'a> {
    username: &'a str,
    password: &'a str,
    locale: &'a str,
}

impl<'a> Credentials<'a> {
    pub fn new(username: &'a str, password: &'a str, locale: &'a str) -> Self {
        Self {
            username,
            password,
            locale,
        }
    }
}

impl fmt::Debug for Credentials<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials([REDACTED])")
    }
}

/// Exactly 16 legacy nonce bytes, including the timestamp used by the transform.
///
/// No clock or RNG is accessed. Length errors are excluded by the input type:
///
/// ```compile_fail
/// use lox_core::legacy_wsx::Nonce;
/// let nonce = Nonce::from_bytes([0; 15]);
/// ```
pub struct Nonce([u8; 16]);

impl Nonce {
    /// Embed the timestamp into a caller-provided 16-byte random buffer.
    ///
    /// Positions 3, 4, 9 and 14 are overwritten, leaving 12 random bytes.
    /// Accepting all 16 bytes preserves the existing producer's RNG consumption.
    pub fn from_random_bytes(timestamp: u32, mut random_bytes: [u8; 16]) -> Self {
        let ts = timestamp.to_le_bytes();
        random_bytes[3] = ts[3];
        random_bytes[4] = ts[0];
        random_bytes[9] = ts[1];
        random_bytes[14] = ts[2];
        Self(random_bytes)
    }

    /// Use an already assembled or captured nonce without altering its bytes.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Recover the full 32-bit timestamp without accepting an unchecked slice.
    pub const fn timestamp(&self) -> u32 {
        u32::from_le_bytes([self.0[4], self.0[9], self.0[14], self.0[3]])
    }
}

impl fmt::Debug for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Nonce([REDACTED])")
    }
}

/// Encode the complete 64-byte legacy handshake, without transport framing.
///
/// The timestamp is recovered from `nonce`, so it cannot disagree with the
/// transform's input. The fixed header is type 1, payload length 32, sequence 0,
/// and magic `0xFEED_BEEF`, all little-endian. Every string is valid input:
/// truncation is deliberate legacy behavior, not a validation error.
///
/// Returned bytes contain recoverable credentials; callers control disclosure.
pub fn encode_handshake(credentials: &Credentials<'_>, nonce: &Nonce) -> [u8; 64] {
    let mut buf = [0u8; 64];
    buf[..16].copy_from_slice(nonce.as_bytes());
    buf[16..20].copy_from_slice(&1u32.to_le_bytes());
    buf[20..24].copy_from_slice(&32u32.to_le_bytes());
    buf[24..28].copy_from_slice(&0u32.to_le_bytes());
    buf[28..32].copy_from_slice(&0xFEED_BEEFu32.to_le_bytes());

    let mut payload = [0u8; 32];
    let mut pos = 0;
    for part in [
        credentials.username.as_bytes(),
        &[0],
        credentials.password.as_bytes(),
        &[0],
        credentials.locale.as_bytes(),
        &[0],
    ] {
        let end = (pos + part.len()).min(32);
        let n = end - pos;
        payload[pos..end].copy_from_slice(&part[..n]);
        pos = end;
    }

    Rc6Key::from_timestamp(nonce.timestamp()).decrypt(&mut payload);
    buf[32..64].copy_from_slice(&payload);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_handshake_with_ts(user: &str, password: &str, locale: &str, ts: u32) -> [u8; 64] {
        encode_handshake(
            &Credentials::new(user, password, locale),
            &Nonce::from_random_bytes(ts, [0; 16]),
        )
    }

    #[test]
    fn round_trip_handshake() {
        let pw = String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
        let pkt = build_handshake_with_ts("admin", &pw, "DEU", 0x34FDE6A4);
        assert_eq!(pkt.len(), 64);
        assert_eq!(&pkt[16..20], &1u32.to_le_bytes());
        assert_eq!(&pkt[20..24], &32u32.to_le_bytes());
        assert_eq!(&pkt[24..28], &0u32.to_le_bytes());
        assert_eq!(&pkt[28..32], &0xFEED_BEEFu32.to_le_bytes());

        let ts = Nonce::from_bytes(pkt[..16].try_into().unwrap()).timestamp();
        assert_eq!(ts, 0x34FDE6A4);
        let key = Rc6Key::from_timestamp(ts);
        let mut payload = [0u8; 32];
        payload.copy_from_slice(&pkt[32..64]);
        key.encrypt(&mut payload);
        let mut expected = Vec::new();
        expected.extend_from_slice(b"admin\0");
        expected.extend_from_slice(pw.as_bytes());
        expected.extend_from_slice(b"\0DEU\0");
        expected.resize(32, 0);
        assert_eq!(&payload[..], expected.as_slice());
    }

    #[test]
    fn handshake_construction() {
        let pw = String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
        let ts = 0x34FDE6A4;
        let pkt = build_handshake_with_ts("admin", &pw, "DEU", ts);
        let expected_header = decode::<16>("010000002000000000000000efbeedfe");
        assert_eq!(&pkt[16..32], &expected_header);
        let key = Rc6Key::from_timestamp(ts);
        let mut our_payload = [0u8; 32];
        our_payload.copy_from_slice(&pkt[32..64]);
        key.encrypt(&mut our_payload);
        let mut expected_payload =
            decode::<32>("ec22eb7bcfc676f229420875226c1a5c4db9a5b92e84509093b3d9280d314a83");
        key.encrypt(&mut expected_payload);
        assert_eq!(our_payload, expected_payload);
    }

    #[test]
    fn truncation_is_across_segments_and_can_split_utf8() {
        let cases: &[(&str, &str, &str, &[u8])] = &[
            ("a\0b", "\0p\0", "E\0N", b"a\0b\0\0p\0\0E\0N\0"),
            (
                "1234567890123456789012345678901",
                "ignored",
                "ENU",
                b"1234567890123456789012345678901\0",
            ),
            (
                "12345678901234567890123456789012",
                "ignored",
                "ENU",
                b"12345678901234567890123456789012",
            ),
            (
                "1234567890123456789012345678901é",
                "ignored",
                "ENU",
                b"1234567890123456789012345678901\xc3",
            ),
            (
                "user",
                "12345678901234567890123456🙂",
                "ENU",
                b"user\x0012345678901234567890123456\xf0",
            ),
            (
                "user",
                "123456789012345678901234",
                "日本語",
                b"user\x00123456789012345678901234\0\xe6\x97",
            ),
        ];
        for &(user, password, locale, expected) in cases {
            let pkt = build_handshake_with_ts(user, password, locale, u32::MAX);
            let mut payload: [u8; 32] = pkt[32..].try_into().unwrap();
            Rc6Key::from_timestamp(u32::MAX).encrypt(&mut payload);
            let mut padded = [0; 32];
            padded[..expected.len()].copy_from_slice(expected);
            assert_eq!(payload, padded);
        }
    }

    pub(super) fn decode<const N: usize>(encoded: &str) -> [u8; N] {
        assert_eq!(encoded.len(), N * 2);
        std::array::from_fn(|i| u8::from_str_radix(&encoded[i * 2..i * 2 + 2], 16).unwrap())
    }
}
