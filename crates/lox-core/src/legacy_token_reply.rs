//! Exact JSON projection used by the legacy token WebSocket receive helpers.
//!
//! This is not a success validator, generic envelope policy or authentication
//! session. Values remain opaque, and no secret zeroization is promised.

use serde_json::Value;
use std::{error::Error, fmt};

/// Fixed legacy reply projection. Non-string present values are compact JSON.
#[derive(Clone, Eq, PartialEq)]
pub struct Reply {
    pub code: u32,
    pub value: String,
}

impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reply")
            .field("code", &self.code)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// Payload-free failure classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplyErrorKind {
    InvalidJson,
    MissingEnvelope,
}

/// Default diagnostics never expose the reply or the serde_json diagnostic.
///
/// The original JSON error is retained only for an explicit compatibility
/// handoff through [`Self::into_json_error`], not the default error source chain.
pub struct ReplyError {
    kind: ReplyErrorKind,
    json: Option<serde_json::Error>,
}

impl ReplyError {
    pub fn kind(&self) -> ReplyErrorKind {
        self.kind
    }

    /// Original serde_json line/column, absent for a missing envelope.
    pub fn location(&self) -> Option<(usize, usize)> {
        self.json.as_ref().map(|e| (e.line(), e.column()))
    }

    /// Opt in to the original, potentially payload-bearing serde_json error.
    ///
    /// Intended for legacy head compatibility only. Its Display/Debug and
    /// downstream error chains do not have this type's redaction guarantee.
    pub fn into_json_error(self) -> Option<serde_json::Error> {
        self.json
    }
}

impl fmt::Debug for ReplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReplyError")
            .field("kind", &self.kind)
            .field("location", &self.location())
            .finish()
    }
}

impl fmt::Display for ReplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.location() {
            Some((line, column)) => {
                write!(
                    f,
                    "invalid legacy token reply JSON at line {line} column {column}"
                )
            }
            None => f.write_str("response missing 'LL' envelope"),
        }
    }
}

impl Error for ReplyError {}

/// Observe the existing permissive legacy profile without validating success.
///
/// Only lowercase `code` is read: strings use Rust `u32` parsing, unsigned JSON
/// integers narrow modulo 2^32, and all other forms become zero. Uppercase
/// `Code` is ignored. A missing top-level `LL` fails, but a present non-object
/// `LL` projects zero/empty. Missing `value` is empty; strings decode unchanged;
/// other present values use serde_json's compact serialization and enabled map
/// order. No normalization, numeric/domain validation or token policy is added.
pub fn parse(text: &str) -> Result<Reply, ReplyError> {
    let root: Value = serde_json::from_str(text).map_err(|json| ReplyError {
        kind: ReplyErrorKind::InvalidJson,
        json: Some(json),
    })?;
    let ll = root.get("LL").ok_or(ReplyError {
        kind: ReplyErrorKind::MissingEnvelope,
        json: None,
    })?;
    let code = ll
        .get("code")
        .and_then(|code| {
            code.as_str()
                .and_then(|text| text.parse::<u32>().ok())
                .or_else(|| code.as_u64().map(|number| number as u32))
        })
        .unwrap_or(0);
    let value = match ll.get("value") {
        Some(Value::String(value)) => value.clone(),
        Some(value) => value.to_string(),
        None => String::new(),
    };
    Ok(Reply { code, value })
}
