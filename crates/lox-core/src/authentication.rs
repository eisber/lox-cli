//! Existing CLI authentication hash profiles, not a general authentication flow.
//!
//! APP uses raw credentials. CONFIG `/wsx` salts the password with the explicit
//! inner digest, but always uses HMAC-SHA256. Neither selects protocol commands,
//! permissions or firmware policy. Inputs and outputs are not logged or zeroized.

use std::borrow::Cow;
use std::fmt;

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

/// The inner password digest for the existing CONFIG `/wsx` profile.
///
/// The head selects SHA256 only for the exact advertised string `"SHA256"`;
/// missing, unknown or differently cased values retain its SHA1 fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsxPasswordDigest {
    Sha1,
    Sha256,
}

/// Explicit, distinct credential profiles already used by the CLI.
pub enum CredentialProfile<'a> {
    /// HMAC-SHA256 of `user:password`, without a salted inner digest.
    AppRawPassword,
    /// Uppercase HASH(`password:salt`), then HMAC-SHA256 of `user:HASH`.
    ConfigWsx {
        salt: &'a str,
        password_digest: WsxPasswordDigest,
    },
}

impl fmt::Debug for CredentialProfile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AppRawPassword => f.write_str("AppRawPassword"),
            Self::ConfigWsx {
                password_digest, ..
            } => f
                .debug_struct("ConfigWsx")
                .field("salt", &"[REDACTED]")
                .field("password_digest", password_digest)
                .finish(),
        }
    }
}

/// Decoded HMAC key. Borrow bytes when the head owns decoding/error policy.
///
/// Empty keys are valid HMAC inputs. Malformed hex is never treated as empty.
/// No guarantee of memory erasure is made for borrowed or owned bytes.
pub struct HashKey<'a>(Cow<'a, [u8]>);

impl<'a> HashKey<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self(Cow::Borrowed(bytes))
    }

    /// Strictly decode a key without returning encoded bytes in diagnostics.
    pub fn from_hex(encoded: &str) -> Result<Self, HashError> {
        hex::decode(encoded)
            .map(|bytes| Self(Cow::Owned(bytes)))
            .map_err(|_| HashError::InvalidHexKey)
    }
}

impl fmt::Debug for HashKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashKey([REDACTED])")
    }
}

/// A secret authentication value with profile-selected wire casing.
///
/// Obtain its wire representation explicitly with [`Self::to_hex`]. Debug never
/// includes its bytes. The returned string is also secret and is not zeroized.
pub struct AuthenticationHash {
    bytes: [u8; 32],
    uppercase: bool,
}

impl AuthenticationHash {
    pub fn to_hex(&self) -> String {
        if self.uppercase {
            hex::encode_upper(self.bytes)
        } else {
            hex::encode(self.bytes)
        }
    }
}

impl fmt::Debug for AuthenticationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthenticationHash([REDACTED])")
    }
}

/// Payload-free input errors; sources deliberately retain no backend/input data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashError {
    InvalidHexKey,
    NonUtf8WsxKey,
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidHexKey => "invalid hex authentication key",
            Self::NonUtf8WsxKey => "CONFIG wsx authentication key is not UTF-8",
        })
    }
}

impl std::error::Error for HashError {}

/// Hash credentials using one of the explicit existing CLI profiles.
///
/// Credentials/salt are UTF-8, colon-delimited verbatim, without escaping,
/// normalization or case folding. Both profiles produce lowercase hex.
pub fn hash_credentials(
    profile: CredentialProfile<'_>,
    user: &str,
    password: &str,
    key: &HashKey<'_>,
) -> AuthenticationHash {
    match profile {
        CredentialProfile::AppRawPassword => {
            hmac_sha256(key, &[user.as_bytes(), b":", password.as_bytes()], false)
        }
        CredentialProfile::ConfigWsx {
            salt,
            password_digest,
        } => {
            let password_hash = match password_digest {
                WsxPasswordDigest::Sha1 => {
                    let mut digest = sha1::Sha1::new();
                    digest.update(password.as_bytes());
                    digest.update(b":");
                    digest.update(salt.as_bytes());
                    hex::encode_upper(digest.finalize())
                }
                WsxPasswordDigest::Sha256 => {
                    let mut digest = Sha256::new();
                    digest.update(password.as_bytes());
                    digest.update(b":");
                    digest.update(salt.as_bytes());
                    hex::encode_upper(digest.finalize())
                }
            };
            hmac_sha256(
                key,
                &[user.as_bytes(), b":", password_hash.as_bytes()],
                false,
            )
        }
    }
}

/// Existing APP token HMAC-SHA256, encoded as lowercase hex.
pub fn hash_app_token(token: &str, key: &HashKey<'_>) -> AuthenticationHash {
    hmac_sha256(key, &[token.as_bytes()], false)
}

/// Existing CONFIG `/wsx` autht: HMAC-SHA256 encoded as uppercase hex.
///
/// Decoded JWT key bytes must be UTF-8, not merely ASCII, matching the existing
/// head's `String::from_utf8` boundary. Invalid UTF-8 never hashes successfully.
pub fn hash_wsx_autht(token: &str, key: &HashKey<'_>) -> Result<AuthenticationHash, HashError> {
    std::str::from_utf8(&key.0).map_err(|_| HashError::NonUtf8WsxKey)?;
    Ok(hmac_sha256(key, &[token.as_bytes()], true))
}

fn hmac_sha256(key: &HashKey<'_>, parts: &[&[u8]], uppercase: bool) -> AuthenticationHash {
    let mut mac = Hmac::<Sha256>::new_from_slice(&key.0).expect("HMAC accepts any key length");
    for part in parts {
        mac.update(part);
    }
    AuthenticationHash {
        bytes: mac.finalize().into_bytes().into(),
        uppercase,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // Original synthetic known answers computed independently with Python stdlib
    // hashlib/hmac, not copied from another Loxone client's vectors.
    #[test]
    fn distinct_credential_profiles_unicode_colons_and_forced_hmac256() {
        let key = HashKey::from_hex("001122aaccee").unwrap();
        let app = hash_credentials(
            CredentialProfile::AppRawPassword,
            "us:ér",
            "päss:word",
            &key,
        );
        assert_eq!(
            app.to_hex(),
            "88bfb35cc1a3ce183b6fcd66b3b5fe3acbdaba620c3ce041d866ce6d055e2bb2"
        );
        for (password_digest, expected) in [
            (
                WsxPasswordDigest::Sha1,
                "9d063ec0b204dd0a2368b4be99e052704579ada03e5b755e2fa4452646473790",
            ),
            (
                WsxPasswordDigest::Sha256,
                "dffd58fcc0395555f0ea22c20b7f613cf1f2cb9103864f0f340c99bb3776a79c",
            ),
        ] {
            assert_eq!(
                hash_credentials(
                    CredentialProfile::ConfigWsx {
                        salt: "s:älт",
                        password_digest,
                    },
                    "us:ér",
                    "päss:word",
                    &key,
                )
                .to_hex(),
                expected
            );
        }
    }

    #[test]
    fn token_profiles_casing_utf8_and_empty_key() {
        let key = HashKey::from_bytes("clé:key".as_bytes());
        assert_eq!(
            hash_app_token("jwt:тøken", &key).to_hex(),
            "4b59c58e38a81ab3dbe92b052651a678a6f5a175175ba01f1f1853849b25da89"
        );
        assert_eq!(
            hash_wsx_autht("jwt:тøken", &key).unwrap().to_hex(),
            "4B59C58E38A81AB3DBE92B052651A678A6F5A175175BA01F1F1853849B25DA89"
        );
        assert_eq!(
            hash_app_token("tok", &HashKey::from_hex("").unwrap()).to_hex(),
            "3d6cb1ab5aa9e075eeb3a608a885b15452f8f9c50a7d33aef07acfd8fdfdc6e1"
        );
        // APP accepts arbitrary decoded key bytes; CONFIG retains its UTF8 check.
        let non_utf8 = HashKey::from_bytes(&[0xff]);
        assert_eq!(
            hash_app_token("tok", &non_utf8).to_hex(),
            "ed4646e109c4890bf51f8bed06aa54b6a79626c69a60ae2d23e6277d940b8b1a"
        );
        assert_eq!(
            hash_wsx_autht("secret-token", &non_utf8).unwrap_err(),
            HashError::NonUtf8WsxKey
        );
    }

    #[test]
    fn secrets_never_enter_debug_or_error_chains() {
        let key = HashKey::from_bytes(b"synthetic-secret-key");
        let hash = hash_app_token("synthetic-secret-token", &key);
        assert_eq!(format!("{key:?}"), "HashKey([REDACTED])");
        assert_eq!(format!("{hash:?}"), "AuthenticationHash([REDACTED])");
        let profile = CredentialProfile::ConfigWsx {
            salt: "synthetic-secret-salt",
            password_digest: WsxPasswordDigest::Sha1,
        };
        assert!(!format!("{profile:?}").contains("synthetic-secret"));
        for invalid in ["z-synthetic-secret-key", "a", "🌸"] {
            let error = HashKey::from_hex(invalid).unwrap_err();
            assert_eq!(error, HashError::InvalidHexKey);
            assert_eq!(error.to_string(), "invalid hex authentication key");
            assert_eq!(format!("{error:?}"), "InvalidHexKey");
            assert!(error.source().is_none());
        }
        let error =
            hash_wsx_autht("synthetic-secret-token", &HashKey::from_bytes(&[0xff])).unwrap_err();
        assert_eq!(
            error.to_string(),
            "CONFIG wsx authentication key is not UTF-8"
        );
        assert_eq!(format!("{error:?}"), "NonUtf8WsxKey");
        assert!(error.source().is_none());
    }
}
