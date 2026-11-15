//! Deterministic AES bytes for the existing CLI's legacy token profile only.
//!
//! This is unauthenticated AES-256-CBC with PKCS7 padding, not a session or
//! modern salted-command implementation. Callers supply and manage the key/IV;
//! this module does not retain them or promise secret zeroization.

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{error::Error, fmt};

/// Ciphertext has invalid CBC length or PKCS7 padding. No partial output is returned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecryptError;

impl fmt::Display for DecryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&block_padding::UnpadError, f)
    }
}

impl Error for DecryptError {}

/// Encrypt opaque bytes, always adding PKCS7 padding, including for empty input.
///
/// Identical supplied inputs produce identical output. No salt, authentication,
/// framing or IV generation is added.
pub fn encrypt(key: &[u8; 32], iv: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    cbc::Encryptor::<aes::Aes256>::new(key.into(), iv.into())
        .encrypt_padded_vec_mut::<block_padding::Pkcs7>(plaintext)
}

/// Decrypt opaque bytes atomically, rejecting invalid length or padding.
///
/// Success does not establish authenticity or UTF-8 validity.
pub fn decrypt(key: &[u8; 32], iv: &[u8; 16], ciphertext: &[u8]) -> Result<Vec<u8>, DecryptError> {
    cbc::Decryptor::<aes::Aes256>::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<block_padding::Pkcs7>(ciphertext)
        .map_err(|_| DecryptError)
}

/// Encrypt command UTF-8 bytes and prepend `jdev/sys/enc/` to STANDARD padded
/// Base64 ciphertext. No percent encoding, salt or other framing is applied.
pub fn encrypt_command(key: &[u8; 32], iv: &[u8; 16], command: &str) -> String {
    let encoded = STANDARD.encode(encrypt(key, iv, command.as_bytes()));
    format!("jdev/sys/enc/{encoded}")
}
