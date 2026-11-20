//! Local encoding policy for one authentication path component, not a request.
//!
//! The vendor explicitly documents URL encoding for token acquisition's client
//! info field. That does not establish support for arbitrary escaped principals
//! or delimiters. This module does not qualify firmware or any hash profile.

use std::fmt;

/// One owned encoded component. The text may be private and is not zeroized.
pub struct AuthPathComponent(String);

/// Refusal reasons that retain no input text, bytes, or positions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthPathComponentError {
    Empty,
    DotSegment,
    ControlCharacter,
}

impl AuthPathComponent {
    /// Encode raw UTF-8 without normalization, trimming, or case folding.
    ///
    /// The caller retains the original input; any hash preimage must use that
    /// original text, never this representation. Only RFC 3986 unreserved
    /// bytes are preserved. Other bytes use uppercase percent escapes, so a
    /// space is `%20` and a literal `%` becomes `%25`, even in escape-like text.
    /// Empty input, exact `.` or `..`, and Unicode control characters (Cc,
    /// including DEL and C1) are refused before encoding.
    pub fn encode(raw: &str) -> Result<Self, AuthPathComponentError> {
        if raw.is_empty() {
            return Err(AuthPathComponentError::Empty);
        }
        if matches!(raw, "." | "..") {
            return Err(AuthPathComponentError::DotSegment);
        }
        if raw.chars().any(char::is_control) {
            return Err(AuthPathComponentError::ControlCharacter);
        }

        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let mut encoded = String::with_capacity(raw.len());
        for byte in raw.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                encoded.push(char::from(byte));
            } else {
                encoded.push('%');
                encoded.push(char::from(HEX[usize::from(byte >> 4)]));
                encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
        Ok(Self(encoded))
    }

    /// Explicitly borrow the already encoded component, not the original text.
    /// The borrowed text may be private; do not log it or encode it again.
    pub fn as_encoded_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AuthPathComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthPathComponent([REDACTED])")
    }
}

impl fmt::Display for AuthPathComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "authentication path component cannot be empty",
            Self::DotSegment => "authentication path component cannot be a dot segment",
            Self::ControlCharacter => "authentication path component cannot contain controls",
        })
    }
}

impl std::error::Error for AuthPathComponentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_unreserved_and_escapes_all_reserved_octets() {
        let encoded = AuthPathComponent::encode("Az09-._~:/?#[]@!$&'()*+,;=\\ %2f ").unwrap();
        assert_eq!(
            encoded.as_encoded_str(),
            "Az09-._~%3A%2F%3F%23%5B%5D%40%21%24%26%27%28%29%2A%2B%2C%3B%3D%5C%20%252f%20"
        );
    }

    #[test]
    fn encodes_exact_utf8_without_unicode_normalization() {
        let composed = AuthPathComponent::encode("caf\u{e9} \u{1f30d}").unwrap();
        let decomposed = AuthPathComponent::encode("cafe\u{301} \u{1f30d}").unwrap();
        assert_eq!(composed.as_encoded_str(), "caf%C3%A9%20%F0%9F%8C%8D");
        assert_eq!(decomposed.as_encoded_str(), "cafe%CC%81%20%F0%9F%8C%8D");
    }

    #[test]
    fn refuses_empty_dot_segments_and_unicode_controls() {
        use AuthPathComponentError::{ControlCharacter, DotSegment, Empty};

        for (raw, expected) in [
            ("", Empty),
            (".", DotSegment),
            ("..", DotSegment),
            ("example\0", ControlCharacter),
            ("exam\nple", ControlCharacter),
            ("\u{7f}example", ControlCharacter),
            ("example\u{85}", ControlCharacter),
        ] {
            assert_eq!(AuthPathComponent::encode(raw).unwrap_err(), expected);
        }
        assert_eq!(
            AuthPathComponent::encode("...").unwrap().as_encoded_str(),
            "..."
        );
    }

    #[test]
    fn debug_redacts_the_encoded_text() {
        let encoded = AuthPathComponent::encode("example / fixture").unwrap();
        assert_eq!(format!("{encoded:?}"), "AuthPathComponent([REDACTED])");
        assert_eq!(format!("{encoded:#?}"), "AuthPathComponent([REDACTED])");
    }
}
