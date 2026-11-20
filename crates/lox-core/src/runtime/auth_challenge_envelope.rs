//! Provisional `getkey2` reply admission for the pinned upstream mock subset.
//!
//! A matching echo does not prove a request was sent or that a reply arrived
//! on the expected channel, in order, or from a trusted peer.

use super::auth_path_component::{AuthPathComponent, AuthPathComponentError};
use super::auth_reply_framing::AuthReplyText;
use super::login_challenge::{self, LoginChallenge, LoginChallengeError};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde_json::value::RawValue;
use std::fmt;

/// Expected full `getkey2` command, including an encoded principal.
pub struct GetKey2Command(String);

impl GetKey2Command {
    /// Encode the principal as a single path component without normalizing it.
    pub fn for_principal(principal: &str) -> Result<Self, AuthPathComponentError> {
        let principal = AuthPathComponent::encode(principal)?;
        Ok(Self(format!(
            "jdev/sys/getkey2/{}",
            principal.as_encoded_str()
        )))
    }

    /// The exact command text. It may contain private data; do not log it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for GetKey2Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GetKey2Command([redacted])")
    }
}

/// Payload-free refusal. The nested challenge error is also redacted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthChallengeEnvelopeError {
    InvalidEnvelope,
    EchoMismatch,
    StatusNotAccepted,
    InvalidChallenge(LoginChallengeError),
}

impl fmt::Display for AuthChallengeEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidEnvelope => "invalid authentication challenge reply envelope",
            Self::EchoMismatch => "authentication challenge reply command mismatch",
            Self::StatusNotAccepted => "authentication challenge reply status was not accepted",
            Self::InvalidChallenge(_) => "invalid authentication challenge reply value",
        })
    }
}

impl std::error::Error for AuthChallengeEnvelopeError {}

/// Admit the lowercase `code` profile with exactly one `LL` object and
/// exactly `control`, `code` and `value` inside. Only status 200 as a JSON
/// integer or the decoded string `"200"` is admitted.
///
/// The original `value` JSON token is passed to the existing challenge
/// decoder, so duplicate members and the value-size limit are not lost
/// through reserialization. This operation never reads or sends anything.
pub fn admit_getkey2(
    reply: &AuthReplyText<'_>,
    expected: &GetKey2Command,
) -> Result<LoginChallenge, AuthChallengeEnvelopeError> {
    use AuthChallengeEnvelopeError as Error;

    let envelope: Envelope<'_> =
        serde_json::from_slice(reply.as_bytes()).map_err(|_| Error::InvalidEnvelope)?;
    if envelope.0.control != expected.as_str() {
        return Err(Error::EchoMismatch);
    }
    let status = envelope.0.code.get();
    let success = status == "200"
        || (status.starts_with('"')
            && serde_json::from_str::<String>(status).is_ok_and(|value| value == "200"));
    if !success {
        return Err(Error::StatusNotAccepted);
    }
    login_challenge::decode(envelope.0.value.get().as_bytes()).map_err(Error::InvalidChallenge)
}

struct Envelope<'a>(Fields<'a>);

impl<'de> Deserialize<'de> for Envelope<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(EnvelopeVisitor)
    }
}

struct EnvelopeVisitor;

impl<'de> Visitor<'de> for EnvelopeVisitor {
    type Value = Envelope<'de>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an authentication reply with only LL")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        if map.next_key::<String>()?.as_deref() != Some("LL") {
            return Err(de::Error::custom("expected LL"));
        }
        let fields = map.next_value()?;
        if map.next_key::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom(
                "unexpected extra authentication reply member",
            ));
        }
        Ok(Envelope(fields))
    }
}

struct Fields<'a> {
    control: String,
    code: &'a RawValue,
    value: &'a RawValue,
}

impl<'de> Deserialize<'de> for Fields<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(FieldsVisitor)
    }
}

struct FieldsVisitor;

impl<'de> Visitor<'de> for FieldsVisitor {
    type Value = Fields<'de>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a getkey2 reply body")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let (mut control, mut code, mut value) = (None, None, None);
        while let Some(name) = map.next_key::<String>()? {
            match name.as_str() {
                "control" if control.is_none() => control = Some(map.next_value()?),
                "code" if code.is_none() => code = Some(map.next_value()?),
                "value" if value.is_none() => value = Some(map.next_value()?),
                _ => {
                    return Err(de::Error::custom(
                        "unexpected or repeated getkey2 reply member",
                    ));
                }
            }
        }
        Ok(Fields {
            control: control.ok_or_else(|| de::Error::missing_field("control"))?,
            code: code.ok_or_else(|| de::Error::missing_field("code"))?,
            value: value.ok_or_else(|| de::Error::missing_field("value"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_header::{MessageHeader, MessageType};
    use crate::runtime::auth_reply_framing::{AuthReplyMessage, admit_exact_text_reply};
    use crate::runtime::login_challenge::HashAlgorithm;
    use std::error::Error;

    const VALUE: &str = r#"{"key":"aBcD","salt":" private:s a l t ","hashAlg":"SHA256"}"#;

    fn admit(
        body: &str,
        expected: &GetKey2Command,
    ) -> Result<LoginChallenge, AuthChallengeEnvelopeError> {
        use AuthReplyMessage::{Binary, Text};

        let size = body.len().try_into().unwrap();
        let header = MessageHeader::new(MessageType::Text as u8, 0, 0, size).to_wire_bytes();
        let text = admit_exact_text_reply(Binary(&header), Some(Text(body))).unwrap();
        admit_getkey2(&text, expected)
    }

    fn envelope(control: &str, code: &str, value: &str) -> String {
        format!(r#"{{"LL":{{"value":{value},"control":"{control}","code":{code}}}}}"#)
    }

    #[test]
    fn checks_the_full_encoded_echo_and_both_success_status_forms() {
        let expected = GetKey2Command::for_principal("user /\u{e9}%").unwrap();
        let command = "jdev/sys/getkey2/user%20%2F%C3%A9%25";
        assert_eq!(expected.as_str(), command);
        assert_eq!(format!("{expected:?}"), "GetKey2Command([redacted])");
        for code in ["200", r#""200""#, r#""2\u00300""#] {
            let reply = admit(&envelope(command, code, VALUE), &expected).unwrap();
            assert_eq!(reply.key_hex(), "aBcD");
            assert_eq!(reply.salt(), " private:s a l t ");
            assert_eq!(reply.algorithm(), HashAlgorithm::Sha256);
        }
        assert!(GetKey2Command::for_principal("..").is_err());
    }

    #[test]
    fn refuses_wrong_echo_and_non_success_without_accepting_the_value() {
        use AuthChallengeEnvelopeError as Error;

        let expected = GetKey2Command::for_principal("user/name").unwrap();
        for echo in [
            "jdev/sys/getkey2/user",
            "jdev/sys/getkey2/user%2fname",
            "jdev/sys/getkey2/user%2Fname/more",
            "jdev/sys/getkey2/user/name",
        ] {
            assert_eq!(
                admit(&envelope(echo, "200", "null"), &expected).unwrap_err(),
                Error::EchoMismatch
            );
        }
        for code in [
            "401",
            "200.0",
            "2e2",
            r#""0200""#,
            r#""200 ""#,
            "null",
            "true",
            "{}",
        ] {
            assert_eq!(
                admit(&envelope(expected.as_str(), code, "null"), &expected).unwrap_err(),
                Error::StatusNotAccepted
            );
        }
    }

    #[test]
    fn refuses_duplicate_unknown_and_incomplete_envelope_members() {
        let expected = GetKey2Command::for_principal("fixture").unwrap();
        let valid = envelope(expected.as_str(), "200", VALUE);
        for invalid in [
            valid.replacen(r#""LL":"#, r#""LL":{},"LL":"#, 1),
            valid.replacen(r#""LL":"#, r#""extra":null,"LL":"#, 1),
            valid.replacen(r#""control":"#, r#""control":"secret","control":"#, 1),
            valid.replacen(r#""code":"#, r#""co\u0064e":401,"code":"#, 1),
            valid.replacen(r#""value":"#, r#""value":null,"value":"#, 1),
            valid.replacen(r#""code":"#, r#""Code":"#, 1),
            valid.replacen(r#""code":"#, r#""extra":0,"code":"#, 1),
            format!("{valid} {{}}"),
            r#"{"LL":{"code":200}}"#.to_owned(),
            r#"{"LL":[]}"#.to_owned(),
            "{}".to_owned(),
        ] {
            let err = admit(&invalid, &expected).unwrap_err();
            assert_eq!(err, AuthChallengeEnvelopeError::InvalidEnvelope);
            assert_eq!(
                err.to_string(),
                "invalid authentication challenge reply envelope"
            );
            assert!(err.source().is_none());
        }
    }

    #[test]
    fn preserves_duplicate_and_oversize_value_refusals() {
        use AuthChallengeEnvelopeError as Error;

        let expected = GetKey2Command::for_principal("fixture").unwrap();
        for value in [
            VALUE.replacen(r#""key":"#, r#""k\u0065y":"deadbeef","key":"#, 1),
            serde_json::to_string(VALUE).unwrap(),
        ] {
            assert_eq!(
                admit(&envelope(expected.as_str(), "200", &value), &expected).unwrap_err(),
                Error::InvalidChallenge(LoginChallengeError::InvalidObject)
            );
        }
        let oversized = VALUE.replacen(' ', &" ".repeat(8192), 1);
        assert_eq!(
            admit(&envelope(expected.as_str(), "200", &oversized), &expected).unwrap_err(),
            Error::InvalidChallenge(LoginChallengeError::TooLarge)
        );
    }
}
