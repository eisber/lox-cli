//! Small typed storage bindings shared by validated config edits.

use std::fmt;

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StoredValue<T> {
    /// Exact serialized spelling; omission is never replaced with a default.
    pub stored: Option<String>,
    pub effective: Option<T>,
    pub default: Option<T>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
}

impl Diagnostic {
    pub(crate) fn new(code: &str, path: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for Diagnostic {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NumericBinding {
    pub key: &'static str,
    pub unit: &'static str,
    pub default: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl NumericBinding {
    pub fn parse(self, raw: &str, path: &str) -> Result<f64> {
        let unsigned = raw.strip_prefix(['-', '+']).unwrap_or(raw);
        let decimal_syntax = !unsigned.is_empty()
            && unsigned.bytes().any(|byte| byte.is_ascii_digit())
            && unsigned
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'.')
            && unsigned.bytes().filter(|byte| *byte == b'.').count() <= 1;
        if !decimal_syntax {
            return Err(Diagnostic::new(
                "invalid_number",
                path,
                "Use a finite invariant decimal (for example -1.25); scientific notation, decimal commas and GUI input filtering are not supported.",
            )
            .into());
        }
        let value = raw.parse::<f64>().ok().filter(|value| value.is_finite());
        let Some(value) = value else {
            return Err(Diagnostic::new("invalid_number", path, "Value must be finite.").into());
        };
        if value == 0.0 && unsigned.bytes().any(|byte| matches!(byte, b'1'..=b'9')) {
            return Err(Diagnostic::new(
                "numeric_underflow",
                path,
                "A nonzero decimal cannot be represented without rounding to zero; choose a representable value. Zero-default omission and power substitution will not be applied.",
            ).into());
        }
        if self.min.is_some_and(|min| value < min) || self.max.is_some_and(|max| value > max) {
            return Err(Diagnostic::new(
                "out_of_range",
                path,
                format!(
                    "{} must be within {}..{} {}.",
                    self.key,
                    self.min.map_or("unbounded".into(), |v| v.to_string()),
                    self.max.map_or("unbounded".into(), |v| v.to_string()),
                    self.unit
                ),
            )
            .into());
        }
        Ok(value)
    }
}

pub(crate) fn minutes_to_seconds(minutes: i64, path: &str) -> Result<i32> {
    minutes
        .checked_mul(60)
        .and_then(|seconds| i32::try_from(seconds).ok())
        .ok_or_else(|| {
            Diagnostic::new(
                "integer_overflow",
                path,
                "Integer minutes must fit signed 32-bit storage seconds after multiplication by 60; GUI overflow behavior is unverified.",
            )
            .into()
        })
}
