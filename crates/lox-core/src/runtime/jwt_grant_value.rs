//! Provisional, value-only admission for the pinned `getjwt` mock subset.
//!
//! This does not admit a reply status or echo, validate a JWT, grant rights,
//! install a token, or choose any command encryption or transport policy.

use super::session_token::TokenExpiry;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use std::fmt;

const MAX_SERIALIZED_BYTES: usize = 8 * 1024;

/// Shape-validated grant fields, not proof of authenticity or usable expiry.
pub struct JwtGrantValue {
    token: String,
    expiry: TokenExpiry,
    token_rights: Option<u32>,
    unsecure_pass: Option<bool>,
    key_hex: Option<String>,
}

impl JwtGrantValue {
    /// Opaque peer-provided text, not a verified JWT. Do not log it.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Reported expiry, converted from the Loxone epoch without using a clock.
    pub fn expiry(&self) -> TokenExpiry {
        self.expiry
    }

    /// Reported bitmap only; it does not confer any rights.
    pub fn token_rights(&self) -> Option<u32> {
        self.token_rights
    }

    /// Reported weak-password advisory. Missing and false are distinct.
    pub fn unsecure_pass(&self) -> Option<bool> {
        self.unsecure_pass
    }

    /// Optional reported key, preserved as supplied. Not selected for use.
    pub fn key_hex(&self) -> Option<&str> {
        self.key_hex.as_deref()
    }
}

impl fmt::Debug for JwtGrantValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JwtGrantValue([redacted])")
    }
}

/// Payload-free refusal. No parser diagnostics or error source are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JwtGrantValueError {
    TooLarge,
    InvalidObject,
    InvalidToken,
    InvalidExpiry,
    InvalidKey,
}

impl fmt::Display for JwtGrantValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "token grant value exceeds the local size limit",
            Self::InvalidObject => "invalid token grant value object",
            Self::InvalidToken => "invalid token grant text",
            Self::InvalidExpiry => "token grant expiry is out of range",
            Self::InvalidKey => "invalid token grant key",
        })
    }
}

impl std::error::Error for JwtGrantValueError {}

/// Decode the original JSON value token, not a reserialized or legacy reply.
///
/// The object requires `token` and unsigned integer `validUntil`. Optional
/// `tokenRights`, `unsecurePass` and `key` have exact types. Unknown and
/// repeated fields are refused. The local profile rejects empty tokens and
/// controls or whitespace in token text, and requires any reported key to
/// be nonempty even-length ASCII hex. No token, key, or original JSON is logged.
pub fn decode(value: &[u8]) -> Result<JwtGrantValue, JwtGrantValueError> {
    use JwtGrantValueError as Error;

    if value.len() > MAX_SERIALIZED_BYTES {
        return Err(Error::TooLarge);
    }
    let mut json = serde_json::Deserializer::from_slice(value);
    let fields = json
        .deserialize_map(FieldsVisitor)
        .and_then(|fields| json.end().map(|()| fields))
        .map_err(|_| Error::InvalidObject)?;
    let token = fields.token.ok_or(Error::InvalidObject)?;
    if token.is_empty() || token.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(Error::InvalidToken);
    }
    let seconds = fields.valid_until.ok_or(Error::InvalidObject)?;
    let expiry = TokenExpiry::from_loxone_seconds(seconds).map_err(|_| Error::InvalidExpiry)?;
    if fields.key_hex.as_ref().is_some_and(|key| {
        key.is_empty()
            || !key.len().is_multiple_of(2)
            || !key.bytes().all(|byte| byte.is_ascii_hexdigit())
    }) {
        return Err(Error::InvalidKey);
    }
    Ok(JwtGrantValue {
        token,
        expiry,
        token_rights: fields.token_rights,
        unsecure_pass: fields.unsecure_pass,
        key_hex: fields.key_hex,
    })
}

#[derive(Default)]
struct Fields {
    token: Option<String>,
    valid_until: Option<u64>,
    token_rights: Option<u32>,
    unsecure_pass: Option<bool>,
    key_hex: Option<String>,
}

struct FieldsVisitor;

impl<'de> Visitor<'de> for FieldsVisitor {
    type Value = Fields;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a token grant value object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut fields = Fields::default();
        while let Some(name) = map.next_key::<String>()? {
            match name.as_str() {
                "token" if fields.token.is_none() => fields.token = Some(map.next_value()?),
                "validUntil" if fields.valid_until.is_none() => {
                    fields.valid_until = Some(map.next_value()?);
                }
                "tokenRights" if fields.token_rights.is_none() => {
                    fields.token_rights = Some(map.next_value()?);
                }
                "unsecurePass" if fields.unsecure_pass.is_none() => {
                    fields.unsecure_pass = Some(map.next_value()?);
                }
                "key" if fields.key_hex.is_none() => fields.key_hex = Some(map.next_value()?),
                _ => {
                    return Err(de::Error::custom(
                        "unexpected or repeated token grant member",
                    ));
                }
            }
        }
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::time::{Duration, UNIX_EPOCH};

    const REQUIRED: &[u8] = br#"{"token":"synthetic.opaque.reply","validUntil":123456}"#;

    #[test]
    fn preserves_opaque_text_and_converts_reported_expiry() {
        let grant = decode(REQUIRED).unwrap();
        assert_eq!(grant.token(), "synthetic.opaque.reply");
        assert_eq!(
            grant.expiry(),
            TokenExpiry::At(UNIX_EPOCH + Duration::from_secs(1_230_768_000 + 123_456))
        );
        assert_eq!(grant.token_rights(), None);
        assert_eq!(grant.unsecure_pass(), None);
        assert_eq!(grant.key_hex(), None);

        let grant = decode(
            br#"{"key":"aBcD","unsecurePass":false,"tokenRights":6,"validUntil":0,"token":"fixture"}"#,
        )
        .unwrap();
        assert_eq!(grant.key_hex(), Some("aBcD"));
        assert_eq!(grant.token_rights(), Some(6));
        assert_eq!(grant.unsecure_pass(), Some(false));
        assert_eq!(format!("{grant:?}"), "JwtGrantValue([redacted])");
    }

    #[test]
    fn rejects_bad_shape_missing_unknown_or_repeated_fields() {
        use JwtGrantValueError as Error;

        for value in [
            r#"{"token":"fixture"}"#,
            r#"{"validUntil":0}"#,
            r#"{"token":"fixture","validUntil":0,"extra":false}"#,
            r#"{"token":"fixture","tok\u0065n":"other","validUntil":0}"#,
            r#"{"token":"fixture","validUntil":0,"validUntil":5}"#,
            r#"{"token":"fixture","validUntil":0,"tokenRights":0,"tokenRights":1}"#,
            r#"{"token":"fixture","validUntil":0,"unsecurePass":"true"}"#,
            r#"{"token":"fixture","validUntil":0,"key":null}"#,
            r#"{"token":"fixture","validUntil":0,"tokenRights":4294967296}"#,
            r#"{"token":"fixture","validUntil":-1}"#,
            r#"{"token":"fixture","validUntil":1.5}"#,
            r#"{"token":"fixture","validUntil":"1"}"#,
            r#"{"token":"fixture","validUntil":0}{}"#,
            r#"{"token":"fixture","validUntil":"#,
            r#"{"LL":{"value":{"token":"fixture","validUntil":0}}}"#,
            "[]",
        ] {
            let error = decode(value.as_bytes()).unwrap_err();
            assert_eq!(error, Error::InvalidObject);
            assert!(error.source().is_none());
            assert_eq!(error.to_string(), "invalid token grant value object");
        }
    }

    #[test]
    fn refuses_unsupported_token_key_and_expiry_values() {
        use JwtGrantValueError as Error;

        for (value, wanted) in [
            (r#"{"token":"","validUntil":0}"#, Error::InvalidToken),
            (
                r#"{"token":" private","validUntil":0}"#,
                Error::InvalidToken,
            ),
            (
                r#"{"token":"a\u0000b","validUntil":0}"#,
                Error::InvalidToken,
            ),
            (
                r#"{"token":"a\u00a0b","validUntil":0}"#,
                Error::InvalidToken,
            ),
            (
                r#"{"token":"fixture","validUntil":18446744073709551615}"#,
                Error::InvalidExpiry,
            ),
            (
                r#"{"token":"fixture","validUntil":0,"key":""}"#,
                Error::InvalidKey,
            ),
            (
                r#"{"token":"fixture","validUntil":0,"key":"abc"}"#,
                Error::InvalidKey,
            ),
            (
                r#"{"token":"fixture","validUntil":0,"key":"xx"}"#,
                Error::InvalidKey,
            ),
        ] {
            assert_eq!(decode(value.as_bytes()).unwrap_err(), wanted);
        }
    }

    #[test]
    fn enforces_the_original_serialized_byte_limit() {
        let mut bytes = REQUIRED.to_vec();
        bytes.resize(MAX_SERIALIZED_BYTES, b' ');
        assert!(decode(&bytes).is_ok());
        bytes.push(b' ');
        assert_eq!(decode(&bytes).unwrap_err(), JwtGrantValueError::TooLarge);
    }
}
