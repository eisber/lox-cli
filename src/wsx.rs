//! CLI-owned time/randomness adapter for the legacy `/wsx` handshake.
//!
//! The deterministic framing, payload truncation and timestamp-derived
//! compatibility transform live in `lox_core::legacy_wsx`.

use lox_core::legacy_wsx::{Credentials, Nonce, encode_handshake};

/// Build a 64-byte /wsx binary handshake message.
///
/// The returned buffer is sent after the HTTP 101 upgrade and start frame.
pub fn build_handshake(username: &str, password: &str, locale: &str) -> [u8; 64] {
    let ts = monotonic_ms();
    build_handshake_with_ts(username, password, locale, ts)
}

/// Build a handshake with a caller-owned timestamp and fresh random nonce.
pub fn build_handshake_with_ts(username: &str, password: &str, locale: &str, ts: u32) -> [u8; 64] {
    build_handshake_with_rng(username, password, locale, ts, &mut rand::thread_rng())
}

fn build_handshake_with_rng(
    username: &str,
    password: &str,
    locale: &str,
    ts: u32,
    rng: &mut impl rand::RngCore,
) -> [u8; 64] {
    let mut random_bytes = [0; 16];
    rng.fill_bytes(&mut random_bytes);
    encode_handshake(
        &Credentials::new(username, password, locale),
        &Nonce::from_random_bytes(ts, random_bytes),
    )
}

/// Extract the timestamp from a legacy nonce slice.
///
/// Retains the old head-only unchecked contract; Core accepts exactly 16 bytes.
pub fn nonce_to_timestamp(nonce: &[u8]) -> u32 {
    let t0 = nonce[4] as u32;
    let t1 = nonce[9] as u32;
    let t2 = nonce[14] as u32;
    let t3 = nonce[3] as u32;
    t0 | (t1 << 8) | (t2 << 16) | (t3 << 24)
}

/// Monotonic millisecond counter.
fn monotonic_ms() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    (ms & 0xFFFF_FFFF) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        include!("wsx_fixtures.rs");
    }

    struct FixedRng {
        bytes: [u8; 16],
        calls: usize,
    }

    impl rand::RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 {
            panic!("the existing producer fills one 16-byte buffer")
        }

        fn next_u64(&mut self) -> u64 {
            panic!("the existing producer fills one 16-byte buffer")
        }

        fn fill_bytes(&mut self, out: &mut [u8]) {
            out.copy_from_slice(&self.bytes);
            self.calls += 1;
        }

        fn try_fill_bytes(&mut self, out: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(out);
            Ok(())
        }
    }

    #[test]
    fn legacy_wsx_head_producer_matches_pre_extraction_packets() {
        let password =
            String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
        for (index, case) in fixtures::cases(&password).into_iter().enumerate() {
            let mut rng = FixedRng {
                bytes: fixtures::randomness(index),
                calls: 0,
            };
            let packet = build_handshake_with_rng(
                case.username,
                case.password,
                case.locale,
                case.timestamp,
                &mut rng,
            );
            let expected = fixtures::decode_packet(case.packet);
            assert_eq!(packet, expected, "{}", case.name);
            assert_eq!(rng.calls, 1);
            let fresh =
                build_handshake_with_ts(case.username, case.password, case.locale, case.timestamp);
            assert_eq!(&fresh[16..], &expected[16..], "{}", case.name);
            assert_eq!(nonce_to_timestamp(&fresh), case.timestamp);
        }
    }

    #[test]
    fn legacy_wsx_head_nonce_slice_compatibility() {
        let nonce = hex::decode("29000034a4be18846ce6e14ad62cfdae").unwrap();
        assert_eq!(nonce_to_timestamp(&nonce), 0x34fde6a4);
        assert_eq!(nonce_to_timestamp(&nonce[..15]), 0x34fde6a4);
        assert!(std::panic::catch_unwind(|| nonce_to_timestamp(&nonce[..14])).is_err());
    }
}
