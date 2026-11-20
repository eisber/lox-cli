//! Provisional password response calculation for the typed `getkey2` challenge.
//!
//! Based on "Hashing" and "Acquiring tokens" in the pinned
//! `CommunicatingWithMiniserver.md`, not on the legacy APP or CONFIG profiles.

use super::login_challenge::{HashAlgorithm, LoginChallenge};
use crate::authentication::{HashError, HashKey, salted_password_hash};
use hmac::Hmac;
use sha1::Sha1;
use sha2::Sha256;
use std::fmt;

enum ResponseMac {
    Sha1([u8; 20]),
    Sha256([u8; 32]),
}

/// Secret MAC; only explicit accessors expose it. Debug is redacted.
///
/// The response and any copies or hex encodings are not zeroized.
pub struct LoginResponse(ResponseMac);

impl LoginResponse {
    /// Borrow the binary MAC (20 bytes for SHA1, 32 for SHA256).
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            ResponseMac::Sha1(bytes) => bytes,
            ResponseMac::Sha256(bytes) => bytes,
        }
    }

    /// Render this secret as lowercase hex, directly from bytes without a
    /// case conversion. The vendor specifies hex without a case conversion
    /// but does not fix the encoder's case. This local encoding does not
    /// establish firmware acceptance, in particular for SHA1.
    pub fn to_lower_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }
}

impl fmt::Debug for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoginResponse([redacted])")
    }
}

/// Derive a password response without I/O or command formatting.
///
/// Hash `password:salt`, uppercase its hex, then HMAC `account:password_hash`
/// with the once-decoded key. Both steps use the advertised algorithm. Text
/// uses its original UTF-8 bytes with literal colons; nothing is normalized,
/// trimmed, or escaped. No zeroization is promised for inputs, temporary key
/// bytes, password hash, or other intermediate values.
///
/// This is not a trust or authenticity check or permission to respond. Do not
/// use it for an untrusted, uncorrelated, or route-mismatched challenge. The
/// caller must independently verify the peer, route, account, reply/status,
/// correlation, and permitted algorithm before using credentials.
///
/// `HashError::InvalidHexKey` is returned if key decoding fails; a valid
/// `LoginChallenge` already enforces the hex input invariant.
pub fn derive(
    challenge: &LoginChallenge,
    account: &str,
    password: &str,
) -> Result<LoginResponse, HashError> {
    let key = HashKey::from_hex(challenge.key_hex())?;
    let salt = challenge.salt();
    let password_hash = match challenge.algorithm() {
        HashAlgorithm::Sha1 => salted_password_hash::<Sha1>(password, salt),
        HashAlgorithm::Sha256 => salted_password_hash::<Sha256>(password, salt),
    };
    let parts: [&[u8]; 3] = [account.as_bytes(), b":", password_hash.as_bytes()];
    let mac = match challenge.algorithm() {
        HashAlgorithm::Sha1 => ResponseMac::Sha1(key.hmac_bytes::<Hmac<Sha1>>(&parts).into()),
        HashAlgorithm::Sha256 => ResponseMac::Sha256(key.hmac_bytes::<Hmac<Sha256>>(&parts).into()),
    };
    Ok(LoginResponse(mac))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::login_challenge::decode;

    // Original synthetic known answers computed with Python's hashlib and hmac.
    #[test]
    fn sha1_response_with_once_decoded_key() {
        let challenge = decode(br#"{"hashAlg":"SHA1","key":"3041","salt":" Salt:1 "}"#).unwrap();
        let response = derive(&challenge, "Alice:one", "test:pass").unwrap();
        assert_eq!(
            response.to_lower_hex(),
            "8633f6bec4c25bbf4223ed32799eefbde7b2515a"
        );
    }

    #[test]
    fn sha256_response_preserves_utf8_and_delimiters() {
        let challenge =
            decode(br#"{"hashAlg":"SHA256","key":"fF0001aB","salt":" s:\u0301\u00e4l\t "}"#)
                .unwrap();
        let response = derive(&challenge, "U\u{0308}:用户", "pa\u{0301}ss:🔑").unwrap();
        assert_eq!(
            response.to_lower_hex(),
            "402c7613a73210bdb2c0a28a502bd42ab878f04010079e5866bbe6cfe4c4d24a"
        );
    }

    #[test]
    fn response_debug_is_redacted() {
        let challenge =
            decode(br#"{"hashAlg":"SHA1","key":"fF00aB","salt":"fixture-salt"}"#).unwrap();
        let response = derive(&challenge, "fixture-account", "fixture-password").unwrap();
        assert_eq!(format!("{response:?}"), "LoginResponse([redacted])");
        assert_eq!(format!("{response:#?}"), "LoginResponse([redacted])");
    }
}
