//! Provisional `getkey2` reply admission for the pinned upstream mock subset.
//!
//! A matching echo does not prove a request was sent or that a reply arrived
//! on the expected channel, in order, or from a trusted peer.

use super::auth_path_component::{AuthPathComponent, AuthPathComponentError};
use super::auth_reply_envelope::{EnvelopeError, admit_value};
use super::auth_reply_framing::AuthReplyText;
use super::login_challenge::{self, LoginChallenge, LoginChallengeError};
use std::fmt;
#[cfg(feature = "auth-hash")]
use std::sync::Arc;

/// Expected full `getkey2` command, including an encoded principal.
pub struct GetKey2Command {
    encoded: String,
    #[cfg(feature = "auth-hash")]
    principal: Arc<str>,
}

impl GetKey2Command {
    /// Encode the principal as a single path component without normalizing it.
    /// With `auth-hash`, also retain the original UTF-8 for response preparation.
    pub fn for_principal(principal: &str) -> Result<Self, AuthPathComponentError> {
        let component = AuthPathComponent::encode(principal)?;
        Ok(Self {
            encoded: format!("jdev/sys/getkey2/{}", component.as_encoded_str()),
            #[cfg(feature = "auth-hash")]
            principal: Arc::from(principal),
        })
    }

    /// The exact command text. It may contain private data; do not log it.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }

    #[cfg(feature = "auth-hash")]
    pub(super) fn clone_principal(&self) -> Arc<str> {
        Arc::clone(&self.principal)
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

    let value = admit_value(reply, expected.as_str()).map_err(|error| match error {
        EnvelopeError::InvalidEnvelope => Error::InvalidEnvelope,
        EnvelopeError::EchoMismatch => Error::EchoMismatch,
        EnvelopeError::StatusNotAccepted => Error::StatusNotAccepted,
    })?;
    login_challenge::decode(value).map_err(Error::InvalidChallenge)
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
