//! RC6 block cipher (16 rounds, 32-bit words).
//!
//! Used by the Loxone /wsx binary protocol to obfuscate the initial
//! handshake payload (credentials + locale).  The key is a single u32
//! derived from a millisecond timestamp that is also embedded in the
//! 16-byte nonce prepended to every message.
//!
//! The client applies RC6 *decryption* to the plaintext before sending;
//! the server applies RC6 *encryption* to recover the original payload.

const ROUNDS: usize = 16;
const NUM_SUBKEYS: usize = 2 * ROUNDS + 4; // 36

const P32: u32 = 0xB7E1_5163;
const Q32: u32 = 0x9E37_79B9;

/// Expanded key schedule (36 × u32).
pub struct Rc6Key {
    s: [u32; NUM_SUBKEYS],
}

impl Rc6Key {
    /// Derive an RC6 key schedule from a single 32-bit timestamp.
    pub fn from_timestamp(ts: u32) -> Self {
        let mut s = [0u32; NUM_SUBKEYS];
        s[0] = P32;
        for i in 1..NUM_SUBKEYS {
            s[i] = s[i - 1].wrapping_add(Q32);
        }

        // Scramble the timestamp into a single-word user key
        let mut l: u32 = ((ts & 0x1000) | (ts >> 16)) >> 8 | ((ts << 16) | (ts & 1)) << 8;

        // 3 × max(36, 1) = 108 mixing rounds
        let mut a: u32 = 0;
        let mut b: u32 = 0;
        let mut i = 0usize;
        for _ in 0..108 {
            a = s[i].wrapping_add(a).wrapping_add(b).rotate_left(3);
            s[i] = a;
            let rot = a.wrapping_add(b) & 31; // must use OLD b
            b = l.wrapping_add(a).wrapping_add(b).rotate_left(rot);
            l = b;
            i = (i + 1) % NUM_SUBKEYS;
        }

        Rc6Key { s }
    }

    /// RC6 encryption of a single 16-byte block (in-place).
    fn encrypt_block(&self, block: &mut [u8; 16]) {
        let [mut a, mut b, mut c, mut d] = read_u32x4(block);

        b = b.wrapping_add(self.s[0]);
        d = d.wrapping_add(self.s[1]);

        for i in 1..=ROUNDS {
            let t = (b.wrapping_mul(b.wrapping_mul(2).wrapping_add(1))).rotate_left(5);
            let u = (d.wrapping_mul(d.wrapping_mul(2).wrapping_add(1))).rotate_left(5);
            a = (a ^ t).rotate_left(u).wrapping_add(self.s[2 * i]);
            c = (c ^ u).rotate_left(t).wrapping_add(self.s[2 * i + 1]);
            let tmp = a;
            a = b;
            b = c;
            c = d;
            d = tmp;
        }

        a = a.wrapping_add(self.s[2 * ROUNDS + 2]);
        c = c.wrapping_add(self.s[2 * ROUNDS + 3]);

        write_u32x4(block, [a, b, c, d]);
    }

    /// RC6 decryption of a single 16-byte block (in-place).
    fn decrypt_block(&self, block: &mut [u8; 16]) {
        let [mut a, mut b, mut c, mut d] = read_u32x4(block);

        c = c.wrapping_sub(self.s[2 * ROUNDS + 3]);
        a = a.wrapping_sub(self.s[2 * ROUNDS + 2]);

        for i in (1..=ROUNDS).rev() {
            let tmp = d;
            d = c;
            c = b;
            b = a;
            a = tmp;
            let u = (d.wrapping_mul(d.wrapping_mul(2).wrapping_add(1))).rotate_left(5);
            let t = (b.wrapping_mul(b.wrapping_mul(2).wrapping_add(1))).rotate_left(5);
            c = (c.wrapping_sub(self.s[2 * i + 1])).rotate_right(t) ^ u;
            a = (a.wrapping_sub(self.s[2 * i])).rotate_right(u) ^ t;
        }

        d = d.wrapping_sub(self.s[1]);
        b = b.wrapping_sub(self.s[0]);

        write_u32x4(block, [a, b, c, d]);
    }

    /// Encrypt `data` in-place (ECB mode, must be multiple of 16 bytes).
    pub fn encrypt(&self, data: &mut [u8]) {
        assert!(
            data.len().is_multiple_of(16),
            "data must be a multiple of 16 bytes"
        );
        for chunk in data.chunks_exact_mut(16) {
            let block: &mut [u8; 16] = chunk.try_into().unwrap();
            self.encrypt_block(block);
        }
    }

    /// Decrypt `data` in-place (ECB mode, must be multiple of 16 bytes).
    pub fn decrypt(&self, data: &mut [u8]) {
        assert!(
            data.len().is_multiple_of(16),
            "data must be a multiple of 16 bytes"
        );
        for chunk in data.chunks_exact_mut(16) {
            let block: &mut [u8; 16] = chunk.try_into().unwrap();
            self.decrypt_block(block);
        }
    }
}

fn read_u32x4(b: &[u8; 16]) -> [u32; 4] {
    [
        u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
        u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
        u32::from_le_bytes([b[8], b[9], b[10], b[11]]),
        u32::from_le_bytes([b[12], b[13], b[14], b[15]]),
    ]
}

fn write_u32x4(b: &mut [u8; 16], vals: [u32; 4]) {
    b[0..4].copy_from_slice(&vals[0].to_le_bytes());
    b[4..8].copy_from_slice(&vals[1].to_le_bytes());
    b[8..12].copy_from_slice(&vals[2].to_le_bytes());
    b[12..16].copy_from_slice(&vals[3].to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validate RC6 encrypt/decrypt with known test vectors.
    ///
    /// The plaintext is `"admin\0<password>\0DEU\0..."` (credentials).
    /// The client applies RC6 *decrypt* to produce the ciphertext sent on the wire.
    /// RC6 encrypt should produce the expected ciphertext.
    #[test]
    fn encrypt_decrypt_test_vectors() {
        // Test credentials
        let pw = String::from_utf8(vec![76, 48, 118, 101, 46, 109, 97, 114, 105, 101]).unwrap();
        let expected: Vec<u8> = {
            let mut v = Vec::new();
            v.extend_from_slice(b"admin\0");
            v.extend_from_slice(pw.as_bytes());
            v.extend_from_slice(b"\0DEU\0");
            v.resize(32, 0);
            v
        };
        assert_eq!(expected.len(), 32);

        let samples: &[(&str, u32, &str)] = &[
            // (name, timestamp, tail_hex)
            (
                "HS1",
                0x34FDE6A4,
                "ec22eb7bcfc676f229420875226c1a5c4db9a5b92e84509093b3d9280d314a83",
            ),
            (
                "HS2",
                0x351474DF,
                "27d39c95ecba503f2ba34c6cd8735af5bebf9e06529d2c729139f6bcab6e4955",
            ),
            (
                "HS3",
                0x35149FAF,
                "1cb00679ce23a0d5d6c1fbe58544c0d57d48767c2f161baebaabee8c397920d1",
            ),
        ];

        for (name, ts, tail_hex) in samples {
            let key = Rc6Key::from_timestamp(*ts);
            let mut tail = hex::decode(tail_hex).unwrap();
            assert_eq!(tail.len(), 32);

            // Client did decrypt → we encrypt to recover plaintext
            key.encrypt(&mut tail);
            assert_eq!(
                &tail,
                expected.as_slice(),
                "{name}: decrypted tail does not match expected plaintext"
            );
        }
    }

    /// Round-trip: encrypt then decrypt should recover the original.
    #[test]
    fn round_trip() {
        let key = Rc6Key::from_timestamp(0x12345678);
        let original = b"sixteen byte blk";
        let mut data = *original;
        key.encrypt(&mut data);
        assert_ne!(&data, original);
        key.decrypt(&mut data);
        assert_eq!(&data, original);
    }

    /// Verify timestamp extraction from nonce.
    #[test]
    fn extract_timestamp_from_nonce() {
        // HS1 nonce (16 bytes)
        let nonce = hex::decode("29000034a4be18846ce6e14ad62cfdae").unwrap();
        let ts = nonce_to_timestamp(&nonce);
        assert_eq!(ts, 0x34FDE6A4);
    }

    fn nonce_to_timestamp(nonce: &[u8]) -> u32 {
        let t0 = nonce[4] as u32;
        let t1 = nonce[9] as u32;
        let t2 = nonce[14] as u32;
        let t3 = nonce[3] as u32;
        t0 | (t1 << 8) | (t2 << 16) | (t3 << 24)
    }
}
