use super::ClockEndpoint;
use serde_json::Value;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonKind {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

fn kind(value: &Value) -> JsonKind {
    match value {
        Value::Null => JsonKind::Null,
        Value::Bool(_) => JsonKind::Boolean,
        Value::Number(_) => JsonKind::Number,
        Value::String(_) => JsonKind::String,
        Value::Array(_) => JsonKind::Array,
        Value::Object(_) => JsonKind::Object,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvelopeState {
    Present,
    Missing,
    Unsupported(JsonKind),
}

#[derive(Clone, PartialEq)]
pub enum ClockValue {
    Text(String),
    Number(f64),
    Missing,
    Unsupported(JsonKind),
}

impl fmt::Debug for ClockValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(_) => f.write_str("Text([redacted])"),
            Self::Number(value) => f.debug_tuple("Number").field(value).finish(),
            Self::Missing => f.write_str("Missing"),
            Self::Unsupported(kind) => f.debug_tuple("Unsupported").field(kind).finish(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ProtocolCode {
    Missing,
    Text(String),
    Integer(i64),
    Unsupported(JsonKind),
}

impl fmt::Debug for ProtocolCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(_) => f.write_str("Text([redacted])"),
            Self::Integer(value) => f.debug_tuple("Integer").field(value).finish(),
            Self::Missing => f.write_str("Missing"),
            Self::Unsupported(kind) => f.debug_tuple("Unsupported").field(kind).finish(),
        }
    }
}

/// Observed wire fields, not a claim that the server returned a valid clock.
#[derive(Clone, Debug, PartialEq)]
pub struct ClockField {
    pub envelope: EnvelopeState,
    pub value: ClockValue,
    pub code: ProtocolCode,
    pub lowercase_code: ProtocolCode,
}

impl ClockField {
    pub(crate) fn decode(response: Value) -> Self {
        let ll = response.get("LL");
        let envelope = match ll {
            None => EnvelopeState::Missing,
            Some(Value::Object(_)) => EnvelopeState::Present,
            Some(value) => EnvelopeState::Unsupported(kind(value)),
        };
        let field = |name| ll.and_then(|ll| ll.get(name));
        let value = match field("value") {
            None => ClockValue::Missing,
            Some(Value::String(text)) => ClockValue::Text(text.clone()),
            Some(value) => match value.as_f64() {
                Some(number) => ClockValue::Number(number),
                None => ClockValue::Unsupported(kind(value)),
            },
        };
        let code = |name| match field(name) {
            None => ProtocolCode::Missing,
            Some(Value::String(text)) => ProtocolCode::Text(text.clone()),
            Some(value) => match value.as_i64() {
                Some(number) => ProtocolCode::Integer(number),
                None => ProtocolCode::Unsupported(kind(value)),
            },
        };
        Self {
            envelope,
            value,
            code: code("Code"),
            lowercase_code: code("code"),
        }
    }

    fn legacy_text(&self) -> String {
        match &self.value {
            ClockValue::Text(text) => text.clone(),
            ClockValue::Number(number) => number.to_string(),
            ClockValue::Missing | ClockValue::Unsupported(_) => String::new(),
        }
    }

    fn validate(&self, endpoint: ClockEndpoint) -> Result<String, ClockValidationError> {
        let fail = |issue| ClockValidationError { endpoint, issue };
        if self.envelope != EnvelopeState::Present {
            return Err(fail(ClockValidationIssue::Envelope));
        }
        let code = if self.code == ProtocolCode::Missing {
            &self.lowercase_code
        } else {
            &self.code
        };
        match code {
            ProtocolCode::Text(text) if text == "200" => {}
            ProtocolCode::Integer(200) => {}
            ProtocolCode::Missing => return Err(fail(ClockValidationIssue::MissingCode)),
            ProtocolCode::Unsupported(_) => {
                return Err(fail(ClockValidationIssue::UnsupportedCode));
            }
            _ => return Err(fail(ClockValidationIssue::RejectedCode)),
        }
        match &self.value {
            ClockValue::Text(text) if !text.is_empty() => Ok(text.clone()),
            ClockValue::Number(number) if number.is_finite() => Ok(number.to_string()),
            ClockValue::Missing => Err(fail(ClockValidationIssue::MissingValue)),
            ClockValue::Text(_) => Err(fail(ClockValidationIssue::EmptyValue)),
            _ => Err(fail(ClockValidationIssue::UnsupportedValue)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClockObservation {
    pub date: ClockField,
    pub time: ClockField,
}

/// Textual clock fields, not parsed calendar/timezone/epoch values.
#[derive(Clone, Eq, PartialEq)]
pub struct ClockText {
    pub date: String,
    pub time: String,
}

impl fmt::Debug for ClockText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClockText { date: [redacted], time: [redacted] }")
    }
}

impl ClockObservation {
    /// Requires LL objects, successful (200) codes, and nonempty string or finite
    /// numeric values. This is envelope validation, NOT calendar validation.
    /// An absent uppercase Code falls back to lowercase code.
    pub fn validate(&self) -> Result<ClockText, ClockValidationError> {
        Ok(ClockText {
            date: self.date.validate(ClockEndpoint::Date)?,
            time: self.time.validate(ClockEndpoint::Time)?,
        })
    }

    /// Explicit CLI compatibility: ignore protocol codes and map missing or
    /// unsupported values to empty text. This does not certify a valid reply.
    pub fn legacy_text(&self) -> ClockText {
        ClockText {
            date: self.date.legacy_text(),
            time: self.time.legacy_text(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockValidationIssue {
    Envelope,
    MissingCode,
    UnsupportedCode,
    RejectedCode,
    MissingValue,
    EmptyValue,
    UnsupportedValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockValidationError {
    pub endpoint: ClockEndpoint,
    pub issue: ClockValidationIssue,
}

impl fmt::Display for ClockValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid clock envelope at {}: {:?}",
            self.endpoint.path(),
            self.issue
        )
    }
}

impl std::error::Error for ClockValidationError {}
