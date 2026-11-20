//! Provisional, value-only admission for `getkey2`.
//!
//! The pinned `CommunicatingWithMiniserver.md` ("Hashing" and "Acquiring
//! tokens") identifies a hex-encoded key, a user salt, and the algorithm names
//! SHA1 and SHA256. The legacy `src/token.rs` consumer recognizes an object
//! or a key-only reply; `legacy_token_reply.rs` can flatten LL values and
//! collapse duplicate members. Neither legacy path supplies strict admission.
//!
//! This local policy admits only the three-field JSON object. Wrapped,
//! stringified, key-only, and extended objects are unsupported; envelope
//! shape, echo/correlation, and cross-firmware compatibility are unverified.

use serde::de::{self, Deserializer, MapAccess, Visitor};
use std::fmt;

const MAX_SERIALIZED_BYTES: usize = 8 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
}

/// Shape-validated only: not authenticated, authorized, trusted, bound to an
/// endpoint or principal, or safe to answer without independent checks.
/// Values are only exposed through explicit borrows. No zeroization is promised.
pub struct LoginChallenge {
    key_hex: String,
    salt: String,
    algorithm: HashAlgorithm,
}

impl LoginChallenge {
    /// Nonempty, even-length ASCII hex in its original decoded case.
    pub fn key_hex(&self) -> &str {
        &self.key_hex
    }

    /// Nonempty UTF-8 as decoded, with no trimming or normalization.
    pub fn salt(&self) -> &str {
        &self.salt
    }

    /// What the peer advertised, not a recommendation or a trust decision.
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }
}

impl fmt::Debug for LoginChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoginChallenge([redacted])")
    }
}

/// Payload-free refusal. No parser diagnostics or error source are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginChallengeError {
    TooLarge,
    InvalidObject,
    InvalidKey,
    EmptySalt,
    UnsupportedAlgorithm,
}

impl fmt::Display for LoginChallengeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "login challenge exceeds the local size limit",
            Self::InvalidObject => "invalid login challenge object",
            Self::InvalidKey => "invalid login challenge key",
            Self::EmptySalt => "empty login challenge salt",
            Self::UnsupportedAlgorithm => "unsupported login challenge algorithm",
        })
    }
}

impl std::error::Error for LoginChallengeError {}

/// Decode the original serialized value bytes, not a reserialized JSON map
/// or a legacy LL projection. The caller must independently correlate and
/// admit the reply and status before handing off the value.
///
/// The 8 KiB serialized byte ceiling is local policy, not a firmware limit.
/// JSON string escapes are decoded; key and salt text is otherwise unchanged.
pub fn decode(value: &[u8]) -> Result<LoginChallenge, LoginChallengeError> {
    use LoginChallengeError as Issue;

    if value.len() > MAX_SERIALIZED_BYTES {
        return Err(Issue::TooLarge);
    }
    let mut json = serde_json::Deserializer::from_slice(value);
    let [key_hex, salt, hash_alg] = json
        .deserialize_map(Fields)
        .and_then(|fields| json.end().map(|()| fields))
        .map_err(|_| Issue::InvalidObject)?;

    if key_hex.is_empty()
        || !key_hex.len().is_multiple_of(2)
        || !key_hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(Issue::InvalidKey);
    }
    if salt.is_empty() {
        return Err(Issue::EmptySalt);
    }
    let algorithm = match hash_alg.as_str() {
        "SHA1" => HashAlgorithm::Sha1,
        "SHA256" => HashAlgorithm::Sha256,
        _ => return Err(Issue::UnsupportedAlgorithm),
    };
    Ok(LoginChallenge {
        key_hex,
        salt,
        algorithm,
    })
}

struct Fields;

impl<'de> Visitor<'de> for Fields {
    type Value = [String; 3];

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a login challenge object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let (mut key, mut salt, mut hash_alg) = (None, None, None);
        while let Some(name) = map.next_key::<String>()? {
            let slot = match name.as_str() {
                "key" => &mut key,
                "salt" => &mut salt,
                "hashAlg" => &mut hash_alg,
                _ => return Err(de::Error::custom("unexpected login challenge field")),
            };
            if slot.is_some() {
                return Err(de::Error::custom("duplicate login challenge field"));
            }
            *slot = Some(map.next_value::<String>()?);
        }
        Ok([
            key.ok_or_else(|| de::Error::missing_field("key"))?,
            salt.ok_or_else(|| de::Error::missing_field("salt"))?,
            hash_alg.ok_or_else(|| de::Error::missing_field("hashAlg"))?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn admits_exact_strings_for_both_algorithms() {
        let sha1 = decode(br#"{"hashAlg":"SHA1","key":"aBcD","salt":" "}"#).unwrap();
        assert_eq!(sha1.algorithm(), HashAlgorithm::Sha1);
        assert_eq!(sha1.key_hex(), "aBcD");
        assert_eq!(sha1.salt(), " ");

        let sha256 = decode(
            r#"{"k\u0065y":"\u0041a00","salt":" é:\t\u0301 ","hashAlg":"SHA256"}"#.as_bytes(),
        )
        .unwrap();
        assert_eq!(sha256.algorithm(), HashAlgorithm::Sha256);
        assert_eq!(sha256.key_hex(), "Aa00");
        assert_eq!(sha256.salt(), " é:\t\u{0301} ");
    }

    #[test]
    fn refuses_unsupported_shapes_and_types() {
        for value in [
            "aBcD",
            r#""aBcD""#,
            r#""{\"key\":\"00\",\"salt\":\"s\",\"hashAlg\":\"SHA1\"}""#,
            r#"["00","s","SHA1"]"#,
            r#"{"LL":{"value":{"key":"00","salt":"s","hashAlg":"SHA1"}}}"#,
            r#"{"key":"00"}"#,
            r#"{"key":"00","salt":"s"}"#,
            r#"{"key":"00","hashAlg":"SHA1"}"#,
            r#"{"salt":"s","hashAlg":"SHA1"}"#,
            r#"{"Key":"00","salt":"s","hashAlg":"SHA1"}"#,
            r#"{"key":"00","salt":"s","hashAlg":"SHA1","extra":0}"#,
            r#"{"key":42,"salt":"s","hashAlg":"SHA1"}"#,
            r#"{"key":"00","salt":null,"hashAlg":"SHA1"}"#,
            r#"{"key":"00","salt":"s","hashAlg":["SHA1"]}"#,
            r#"{"key":"00","salt":"s","hashAlg":"SHA1"} {}"#,
        ] {
            assert_eq!(
                decode(value.as_bytes()).unwrap_err(),
                LoginChallengeError::InvalidObject
            );
        }
        let invalid_utf8 = b"{\"key\":\"00\",\"salt\":\"\xff\",\"hashAlg\":\"SHA1\"}";
        assert_eq!(
            decode(invalid_utf8).unwrap_err(),
            LoginChallengeError::InvalidObject
        );
    }

    #[test]
    fn refuses_duplicate_decoded_fields() {
        for value in [
            br#"{"key":"00","key":"11","salt":"s","hashAlg":"SHA1"}"#.as_slice(),
            br#"{"key":"00","salt":"s","s\u0061lt":"t","hashAlg":"SHA1"}"#,
            br#"{"key":"00","salt":"s","hashAlg":"SHA1","hash\u0041lg":"SHA256"}"#,
        ] {
            assert_eq!(
                decode(value).unwrap_err(),
                LoginChallengeError::InvalidObject
            );
        }
    }

    #[test]
    fn refuses_invalid_key_salt_and_algorithm_without_fallback() {
        for key in ["", "abc", "Aé", "gg", " aa", "aa ", "0x12"] {
            let value = format!(r#"{{"key":"{key}","salt":"s","hashAlg":"SHA1"}}"#);
            assert_eq!(
                decode(value.as_bytes()).unwrap_err(),
                LoginChallengeError::InvalidKey
            );
        }
        let no_salt = br#"{"key":"00","salt":"","hashAlg":"SHA1"}"#;
        assert_eq!(decode(no_salt).unwrap_err(), LoginChallengeError::EmptySalt);
        for alg in ["", "SHA512", "sha1", "SHA-256", " SHA256"] {
            let value = format!(r#"{{"key":"00","salt":"s","hashAlg":"{alg}"}}"#);
            assert_eq!(
                decode(value.as_bytes()).unwrap_err(),
                LoginChallengeError::UnsupportedAlgorithm
            );
        }
    }

    #[test]
    fn bounds_serialized_bytes_before_decoding() {
        let prefix = br#"{"key":"00","hashAlg":"SHA256","salt":""#;
        let suffix = br#""}"#;
        let mut boundary = prefix.to_vec();
        boundary.resize(8192 - suffix.len(), b's');
        boundary.extend_from_slice(suffix);
        assert!(decode(&boundary).is_ok());
        boundary.push(b' ');
        assert_eq!(
            decode(&boundary).unwrap_err(),
            LoginChallengeError::TooLarge
        );
        assert_eq!(
            decode(&[b'!'; 8193]).unwrap_err(),
            LoginChallengeError::TooLarge
        );
    }

    #[test]
    fn diagnostics_are_redacted_and_source_free() {
        let accepted =
            decode(br#"{"key":"a012ff","salt":"fixture-salt","hashAlg":"SHA1"}"#).unwrap();
        assert_eq!(format!("{accepted:?}"), "LoginChallenge([redacted])");
        assert_eq!(format!("{accepted:#?}"), "LoginChallenge([redacted])");

        let refused = decode(br#"{"key":"a012ff","salt":"fixture-salt","hashAlg":{"fixture-algorithm":"fixture-value"}}"#)
            .unwrap_err();
        assert_eq!(format!("{refused:?}"), "InvalidObject");
        assert_eq!(refused.to_string(), "invalid login challenge object");
        assert!(refused.source().is_none());
    }
}
