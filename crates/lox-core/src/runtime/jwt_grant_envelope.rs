//! Provisional reply admission for one logical (already decrypted) `getjwt`
//! mock dialect. This neither decrypts nor authenticates a reply.

use super::auth_reply_envelope::{EnvelopeError, admit_value};
use super::auth_reply_framing::AuthReplyText;
use super::jwt_grant_value::{self, JwtGrantValue, JwtGrantValueError};
use super::login_response::LoginResponse;
use std::fmt;

/// Payload-free refusal. No parser diagnostics or error source are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JwtGrantEnvelopeError {
    InvalidEnvelope,
    EchoMismatch,
    StatusNotAccepted,
    InvalidGrant(JwtGrantValueError),
}

impl fmt::Display for JwtGrantEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidEnvelope => "invalid token grant reply envelope",
            Self::EchoMismatch => "token grant reply control mismatch",
            Self::StatusNotAccepted => "token grant reply status was not accepted",
            Self::InvalidGrant(_) => "invalid token grant reply value",
        })
    }
}

impl std::error::Error for JwtGrantEnvelopeError {}

/// Admit the pinned mock's logical control: `jdev/sys/getjwt/` followed by
/// the exact lowercase hex of `response`, not the complete outbound command.
/// The strict LL parser checks the shape, then control, then lowercase `code`
/// (only integer 200 or the decoded string `"200"`). The original JSON value
/// bytes are decoded without reserialization.
///
/// The control check does not prove that a request was sent or that a reply
/// came from a trusted peer or channel. No control or response is retained in
/// the grant or error. This performs no decryption, I/O, dispatch, token
/// verification or installation.
pub fn admit_getjwt(
    reply: &AuthReplyText<'_>,
    response: &LoginResponse,
) -> Result<JwtGrantValue, JwtGrantEnvelopeError> {
    use JwtGrantEnvelopeError as Error;

    let value = admit_value(
        reply,
        &format!("jdev/sys/getjwt/{}", response.to_lower_hex()),
    )
    .map_err(|error| match error {
        EnvelopeError::InvalidEnvelope => Error::InvalidEnvelope,
        EnvelopeError::EchoMismatch => Error::EchoMismatch,
        EnvelopeError::StatusNotAccepted => Error::StatusNotAccepted,
    })?;
    jwt_grant_value::decode(value).map_err(Error::InvalidGrant)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_header::{MessageHeader, MessageType};
    use crate::runtime::auth_reply_framing::{AuthReplyMessage, admit_exact_text_reply};
    use crate::runtime::jwt_grant_value::JwtGrantKey;
    use crate::runtime::{login_challenge, login_response};
    use std::error::Error;

    const MAC: &str = "8633f6bec4c25bbf4223ed32799eefbde7b2515a";
    const CONTROL: &str = "jdev/sys/getjwt/8633f6bec4c25bbf4223ed32799eefbde7b2515a";
    const TOKEN: &str = "synthetic.opaque.mock-grant";
    const VALUE: &str = r#"{"token":"synthetic.opaque.mock-grant","validUntil":123456,"key":""}"#;

    fn response() -> LoginResponse {
        // The independent SHA1 known answer also covered by login_response.
        let challenge =
            login_challenge::decode(br#"{"hashAlg":"SHA1","key":"3041","salt":" Salt:1 "}"#)
                .unwrap();
        login_response::derive(&challenge, "Alice:one", "test:pass").unwrap()
    }

    fn admit(body: &str, response: &LoginResponse) -> Result<JwtGrantValue, JwtGrantEnvelopeError> {
        let size = body.len().try_into().unwrap();
        let header = MessageHeader::new(MessageType::Text as u8, 0, 0, size).to_wire_bytes();
        let reply = admit_exact_text_reply(
            AuthReplyMessage::Binary(&header),
            Some(AuthReplyMessage::Text(body)),
        )
        .unwrap();
        admit_getjwt(&reply, response)
    }

    fn envelope(control: &str, code: &str, value: &str) -> String {
        format!(r#"{{"LL":{{"control":"{control}","code":{code},"value":{value}}}}}"#)
    }

    #[test]
    fn admits_exact_logical_mock_echo_and_reported_empty_key() {
        let response = response();
        for status in [r#""200""#, "200", r#""2\u00300""#] {
            let grant = admit(&envelope(CONTROL, status, VALUE), &response).unwrap();
            assert_eq!(grant.token(), TOKEN);
            assert_eq!(grant.reported_key(), JwtGrantKey::ReportedEmpty);
            assert_eq!(grant.key_hex(), None);
            assert_eq!(format!("{grant:#?}"), "JwtGrantValue([redacted])");
        }
    }

    #[test]
    fn rejects_wrong_ciphertext_full_reflected_and_incomplete_echoes() {
        use JwtGrantEnvelopeError as Refusal;

        let response = response();
        for (case, echo) in [
            ("wrong MAC", format!("jdev/sys/getjwt/0{}", &MAC[1..])),
            ("encrypted control", "jdev/sys/enc/c3ludGhldGlj".to_owned()),
            (
                "full command",
                format!("{CONTROL}/fixture/2/fixture-id/fixture-info"),
            ),
            ("reflected MAC", MAC.to_owned()),
            ("embedded echo", format!("reflect/{CONTROL}")),
            ("prefix only", "jdev/sys/getjwt/".to_owned()),
            ("different prefix", format!("dev/sys/getjwt/{MAC}")),
            (
                "URL escaped MAC",
                format!("jdev/sys/getjwt/%38{}", &MAC[1..]),
            ),
            (
                "uppercase MAC",
                format!("jdev/sys/getjwt/{}", MAC.to_uppercase()),
            ),
            ("trailing space", format!("{CONTROL} ")),
        ] {
            let error = admit(&envelope(&echo, "200", VALUE), &response).unwrap_err();
            assert_eq!(error, Refusal::EchoMismatch, "{case}");
            assert_redacted(error, &[&echo, MAC, TOKEN]);
        }
        assert_eq!(
            admit("c3ludGhldGlj", &response).unwrap_err(),
            Refusal::InvalidEnvelope
        );
    }

    #[test]
    fn rejects_shape_then_echo_then_status_before_value() {
        use JwtGrantEnvelopeError as Refusal;

        let response = response();
        let wrong_echo = envelope("fixture-other", "401", "null");
        for invalid in [
            wrong_echo.replace(r#""code""#, r#""Code""#),
            wrong_echo.replacen(r#""code":"#, r#""code":200,"co\u0064e":"#, 1),
            wrong_echo.replacen(r#""value":"#, r#""extra":0,"value":"#, 1),
        ] {
            assert_eq!(
                admit(&invalid, &response).unwrap_err(),
                Refusal::InvalidEnvelope
            );
        }
        assert_eq!(
            admit(&wrong_echo, &response).unwrap_err(),
            Refusal::EchoMismatch
        );
        for status in [
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
                admit(&envelope(CONTROL, status, "null"), &response).unwrap_err(),
                Refusal::StatusNotAccepted
            );
        }
    }

    #[test]
    fn preserves_raw_nested_duplicate_and_size_refusals_without_diagnostics() {
        use JwtGrantEnvelopeError as Refusal;

        let response = response();
        let duplicate = VALUE.replacen(r#""token":"#, r#""tok\u0065n":"other","token":"#, 1);
        let oversized = VALUE.replacen('{', &format!("{{{}", " ".repeat(8192)), 1);
        for (raw, expected) in [
            (&duplicate, JwtGrantValueError::InvalidObject),
            (&oversized, JwtGrantValueError::TooLarge),
        ] {
            let error = admit(&envelope(CONTROL, "200", raw), &response).unwrap_err();
            assert_eq!(error, Refusal::InvalidGrant(expected));
            assert_eq!(error.to_string(), "invalid token grant reply value");
            assert_redacted(error, &[CONTROL, MAC, TOKEN, raw]);
        }
    }

    fn assert_redacted(error: JwtGrantEnvelopeError, private: &[&str]) {
        assert!(error.source().is_none());
        let debug = format!("{error:?}\n{error:#?}");
        let display = error.to_string();
        for &value in private {
            assert!(!debug.contains(value));
            assert!(!display.contains(value));
        }
    }
}
