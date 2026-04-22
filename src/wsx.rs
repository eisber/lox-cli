//! Loxone `/wsx` binary protocol handshake.
//!
//! The `/wsx` endpoint uses hixie-76 WebSocket framing with a custom binary
//! message layer on top.  Each outgoing binary message has this layout:
//!
//! ```text
//! [0..16]   16-byte nonce (rand() + timestamp)
//! [16..20]  message type  (u32 LE, 0x01 for initial handshake)
//! [20..24]  payload size  (u32 LE, 0x20 = 32)
//! [24..28]  sequence num  (u32 LE, 0 for first message)
//! [28..32]  magic         (u32 LE, 0xFEED_BEEF)
//! [32..64]  RC6-encrypted payload (username\0password\0locale\0...\0)
//! ```
//!
//! The RC6 key is derived from a millisecond uptime counter whose bytes are
//! interleaved into the nonce at positions [3], [4], [9], [14].

use crate::rc6::Rc6Key;

const MAGIC_CONTROL: u32 = 0xFEED_BEEF;
const MSG_TYPE_HANDSHAKE: u32 = 0x0000_0001;
const PAYLOAD_LEN: u32 = 0x20; // 32 bytes

/// Build a 64-byte /wsx binary handshake message.
///
/// # Arguments
/// * `username` – Miniserver username (e.g. "admin")
/// * `password` – Miniserver password (plaintext)
/// * `locale`   – Locale / language code (e.g. "DEU", "ENU")
///
/// # Returns
/// A 64-byte buffer ready to send on the raw TCP connection after the
/// HTTP 101 WebSocket upgrade.
pub fn build_handshake(username: &str, password: &str, locale: &str) -> [u8; 64] {
    // Get uptime-like timestamp (milliseconds).
    // Loxone uses QueryPerformanceCounter / GetTickCount on Windows.
    // Any monotonic millisecond counter works; the server only cares that
    // it can extract the same value from the nonce to derive the RC6 key.
    let ts = monotonic_ms();

    build_handshake_with_ts(username, password, locale, ts)
}

/// Build a handshake with an explicit timestamp (for testing).
pub fn build_handshake_with_ts(username: &str, password: &str, locale: &str, ts: u32) -> [u8; 64] {
    let mut buf = [0u8; 64];

    // ── 1. Nonce (bytes 0..16) ──────────────────────────────────────────
    build_nonce(&mut buf[0..16], ts);

    // ── 2. Header (bytes 16..32) ────────────────────────────────────────
    buf[16..20].copy_from_slice(&MSG_TYPE_HANDSHAKE.to_le_bytes());
    buf[20..24].copy_from_slice(&PAYLOAD_LEN.to_le_bytes());
    buf[24..28].copy_from_slice(&0u32.to_le_bytes()); // sequence = 0
    buf[28..32].copy_from_slice(&MAGIC_CONTROL.to_le_bytes());

    // ── 3. Payload: credentials ─────────────────────────────────────────
    let mut payload = [0u8; 32];
    let mut pos = 0;
    for part in [
        username.as_bytes(),
        &[0],
        password.as_bytes(),
        &[0],
        locale.as_bytes(),
        &[0],
    ] {
        let end = (pos + part.len()).min(32);
        let n = end - pos;
        payload[pos..end].copy_from_slice(&part[..n]);
        pos = end;
    }
    // Rest is zero-padded (already initialized)

    // ── 4. RC6 encrypt → the client applies "decrypt" direction ─────────
    let key = Rc6Key::from_timestamp(ts);
    key.decrypt(&mut payload);

    buf[32..64].copy_from_slice(&payload);
    buf
}

/// Build a 16-byte nonce with timestamp bytes interleaved into rand() output.
///
/// Timestamp byte positions in the nonce:
///   - `ts[3]` (bits 24-31) → nonce[3]
///   - `ts[0]` (bits  0-7)  → nonce[4]
///   - `ts[1]` (bits  8-15) → nonce[9]
///   - `ts[2]` (bits 16-23) → nonce[14]
///
/// The remaining 12 bytes are random.
fn build_nonce(out: &mut [u8], ts: u32) {
    // Fill with random bytes first
    let mut rng = rand::thread_rng();
    rand::RngCore::fill_bytes(&mut rng, out);

    // Embed timestamp bytes at the fixed positions
    let ts_bytes = ts.to_le_bytes(); // [t0, t1, t2, t3]
    out[3] = ts_bytes[3]; // t3 at position 3
    out[4] = ts_bytes[0]; // t0 at position 4
    out[9] = ts_bytes[1]; // t1 at position 9
    out[14] = ts_bytes[2]; // t2 at position 14
}

/// Extract the timestamp from a 16-byte nonce.
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

    #[test]
    fn round_trip_handshake() {
        // Test credentials
        let pw = String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
        let pkt = build_handshake_with_ts("admin", &pw, "DEU", 0x34FDE6A4);
        assert_eq!(pkt.len(), 64);

        // Header checks
        assert_eq!(&pkt[16..20], &1u32.to_le_bytes()); // msg type 1
        assert_eq!(&pkt[20..24], &32u32.to_le_bytes()); // payload size 32
        assert_eq!(&pkt[24..28], &0u32.to_le_bytes()); // seq 0
        assert_eq!(&pkt[28..32], &MAGIC_CONTROL.to_le_bytes());

        // Decrypt the payload to verify credentials
        let ts = nonce_to_timestamp(&pkt[0..16]);
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

    /// Verify handshake packet construction.
    #[test]
    fn handshake_construction() {
        let pw = String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
        let ts: u32 = 0x34FDE6A4;
        let pkt = build_handshake_with_ts("admin", &pw, "DEU", ts);

        // The nonce has random bytes, so we can't compare those.
        // But the header and encrypted payload (given the same ts) must match.
        let expected_header = hex::decode("010000002000000000000000efbeedfe").unwrap();
        assert_eq!(&pkt[16..32], &expected_header[..]);

        // Decrypt our generated payload to verify it's the same credentials
        let key = Rc6Key::from_timestamp(ts);
        let mut our_payload = [0u8; 32];
        our_payload.copy_from_slice(&pkt[32..64]);
        key.encrypt(&mut our_payload);

        let mut expected_payload =
            hex::decode("ec22eb7bcfc676f229420875226c1a5c4db9a5b92e84509093b3d9280d314a83")
                .unwrap();
        key.encrypt(&mut expected_payload);

        assert_eq!(our_payload, expected_payload.as_slice());
    }
}
