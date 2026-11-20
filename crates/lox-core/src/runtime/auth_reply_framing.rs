//! Provisional framing for one exact-length Loxone authentication reply.
//!
//! The caller supplies complete, already separated WebSocket messages. This
//! module does not read a transport, enforce adjacency, correlate a reply or
//! verify its contents. It does not authorize or send a credential.

use crate::message_header::{MessageType, decode_message_header};
use std::fmt;

const MAX_REPLY_BYTES: usize = 16 * 1024;

/// A complete WebSocket message, not bytes from an individual wire fragment.
pub enum AuthReplyMessage<'a> {
    Binary(&'a [u8]),
    Text(&'a str),
    Other,
}

/// The original text of a structurally admitted pair. Contents are unverified.
#[must_use]
pub struct AuthReplyText<'a>(&'a str);

impl<'a> AuthReplyText<'a> {
    /// Original UTF-8 bytes for a subsequent envelope decoder. Not for logging.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for AuthReplyText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthReplyText([redacted])")
    }
}

/// No payload text, malformed bytes, or parser error chain is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthReplyFramingError {
    HeaderMessageKind,
    MalformedHeader,
    UnexpectedHeaderType,
    UnsupportedHeaderMetadata,
    SizeOutsideLimit,
    MissingPayload,
    PayloadMessageKind,
    PayloadSizeMismatch,
}

impl fmt::Display for AuthReplyFramingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::HeaderMessageKind => "authentication reply requires a separate binary header",
            Self::MalformedHeader => "invalid authentication reply header",
            Self::UnexpectedHeaderType => "authentication reply header is not a text identifier",
            Self::UnsupportedHeaderMetadata => "unsupported authentication reply header metadata",
            Self::SizeOutsideLimit => "authentication reply size is outside the local limit",
            Self::MissingPayload => "authentication reply text message is missing",
            Self::PayloadMessageKind => "authentication reply payload requires a text message",
            Self::PayloadSizeMismatch => {
                "authentication reply payload size differs from its header"
            }
        })
    }
}

impl std::error::Error for AuthReplyFramingError {}

/// Admits the exact-size subset: a binary header with the text identifier,
/// zero flags and reserved byte, then one nonempty text message of exactly
/// the declared byte length. The 16 KiB envelope bound is a local policy,
/// not a firmware limit or an allocation request. `None` represents an
/// incomplete pair. Pass only the immediately following message as `payload`.
pub fn admit_exact_text_reply<'a>(
    header: AuthReplyMessage<'_>,
    payload: Option<AuthReplyMessage<'a>>,
) -> Result<AuthReplyText<'a>, AuthReplyFramingError> {
    use AuthReplyFramingError as Error;

    let AuthReplyMessage::Binary(bytes) = header else {
        return Err(Error::HeaderMessageKind);
    };
    let header = decode_message_header(bytes).map_err(|_| Error::MalformedHeader)?;
    if header.known_type() != Some(MessageType::Text) {
        return Err(Error::UnexpectedHeaderType);
    }
    if header.info_flags() != 0 || header.reserved() != 0 {
        return Err(Error::UnsupportedHeaderMetadata);
    }
    let size = usize::try_from(header.payload_size()).map_err(|_| Error::SizeOutsideLimit)?;
    if !(1..=MAX_REPLY_BYTES).contains(&size) {
        return Err(Error::SizeOutsideLimit);
    }
    let Some(payload) = payload else {
        return Err(Error::MissingPayload);
    };
    let AuthReplyMessage::Text(text) = payload else {
        return Err(Error::PayloadMessageKind);
    };
    if text.len() != size {
        return Err(Error::PayloadSizeMismatch);
    }
    Ok(AuthReplyText(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_header::MessageHeader;
    use std::error::Error;

    fn text_header(size: u32) -> [u8; 8] {
        MessageHeader::new(MessageType::Text as u8, 0, 0, size).to_wire_bytes()
    }

    #[test]
    fn admits_original_text_by_utf8_byte_count() {
        let text = " {\"value\":\"caf\u{e9}\"}\n";
        let header = text_header(text.len().try_into().unwrap());
        let reply = admit_exact_text_reply(
            AuthReplyMessage::Binary(&header),
            Some(AuthReplyMessage::Text(text)),
        )
        .unwrap();
        assert_eq!(reply.as_bytes(), text.as_bytes());
        assert_eq!(format!("{reply:?}"), "AuthReplyText([redacted])");

        let character_count = text_header(text.chars().count().try_into().unwrap());
        let error = admit_exact_text_reply(
            AuthReplyMessage::Binary(&character_count),
            Some(AuthReplyMessage::Text(text)),
        )
        .unwrap_err();
        assert_eq!(error, AuthReplyFramingError::PayloadSizeMismatch);
        assert!(error.source().is_none());
    }

    #[test]
    fn requires_a_binary_header_and_separate_text_message() {
        use AuthReplyFramingError as Error;
        use AuthReplyMessage::{Binary, Other, Text};

        let header = text_header(2);
        assert_eq!(
            admit_exact_text_reply(Text("{}"), Some(Binary(&header))).unwrap_err(),
            Error::HeaderMessageKind
        );
        let mut appended = header.to_vec();
        appended.extend_from_slice(b"{}");
        let mut wrong_magic = header;
        wrong_magic[0] = 0xff;
        for bytes in [&header[..7], &appended, &wrong_magic] {
            assert_eq!(
                admit_exact_text_reply(Binary(bytes), Some(Text("{}"))).unwrap_err(),
                Error::MalformedHeader
            );
        }
        assert_eq!(
            admit_exact_text_reply(Binary(&header), None).unwrap_err(),
            Error::MissingPayload
        );
        for payload in [Binary(b"{}"), Binary(&header), Other] {
            assert_eq!(
                admit_exact_text_reply(Binary(&header), Some(payload)).unwrap_err(),
                Error::PayloadMessageKind
            );
        }
    }

    #[test]
    fn refuses_unsupported_identifier_flags_and_reserved_data() {
        use AuthReplyFramingError as Error;
        use AuthReplyMessage::{Binary, Text};

        for (kind, flags, reserved, expected) in [
            (
                MessageType::BinaryFile as u8,
                0,
                0,
                Error::UnexpectedHeaderType,
            ),
            (u8::MAX, 0, 0, Error::UnexpectedHeaderType),
            (
                MessageType::Text as u8,
                1,
                0,
                Error::UnsupportedHeaderMetadata,
            ),
            (
                MessageType::Text as u8,
                0x80,
                0,
                Error::UnsupportedHeaderMetadata,
            ),
            (
                MessageType::Text as u8,
                0,
                1,
                Error::UnsupportedHeaderMetadata,
            ),
        ] {
            let header = MessageHeader::new(kind, flags, reserved, 2).to_wire_bytes();
            assert_eq!(
                admit_exact_text_reply(Binary(&header), Some(Text("{}"))).unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn bounds_the_exact_envelope_without_allocating_from_the_header() {
        use AuthReplyFramingError as Error;
        use AuthReplyMessage::{Binary, Text};

        let max: u32 = MAX_REPLY_BYTES.try_into().unwrap();
        for declared in [0, max + 1, u32::MAX] {
            let header = text_header(declared);
            assert_eq!(
                admit_exact_text_reply(Binary(&header), None).unwrap_err(),
                Error::SizeOutsideLimit
            );
        }
        let text = "a".repeat(MAX_REPLY_BYTES);
        assert_eq!(
            admit_exact_text_reply(Binary(&text_header(max)), Some(Text(&text)))
                .unwrap()
                .as_bytes(),
            text.as_bytes()
        );
        assert_eq!(
            admit_exact_text_reply(Binary(&text_header(2)), Some(Text(&text))).unwrap_err(),
            Error::PayloadSizeMismatch
        );
        assert_eq!(
            admit_exact_text_reply(Binary(&text_header(2)), Some(Text("x"))).unwrap_err(),
            Error::PayloadSizeMismatch
        );
    }
}
