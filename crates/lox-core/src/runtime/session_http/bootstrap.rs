use crate::runtime::TransportFailure;
use reqwest::StatusCode;
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde_json::Value;
use std::fmt;
use std::marker::PhantomData;

/// An unauthenticated observation, not a basis for selecting a trust policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability<T> {
    Missing,
    Unknown,
    Reported(T),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlsStatus {
    Available,
    CertificateExpired,
}

/// Advisory observations from the HTTP route; not an authenticated identity,
/// a TLS verification result, or a reservation of a WebSocket event slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootstrapCapabilities {
    pub event_slots: Capability<bool>,
    pub tls_status: Capability<TlsStatus>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapPayloadIssue {
    TooLarge,
    InvalidEnvelope,
    UnexpectedControl,
    CommandStatus(u16),
    UnsupportedValueEncoding,
}

/// No route, headers, response body or backend error chain is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapError {
    WrongRoute,
    Transport(TransportFailure),
    HttpStatus(StatusCode),
    Payload(BootstrapPayloadIssue),
}

impl fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongRoute => f.write_str("session route is not the bootstrap command"),
            Self::Transport(kind) => write!(f, "bootstrap HTTP transport failure: {kind:?}"),
            Self::HttpStatus(status) => write!(f, "bootstrap HTTP status: {}", status.as_u16()),
            Self::Payload(issue) => write!(f, "bootstrap payload refused: {issue:?}"),
        }
    }
}

impl std::error::Error for BootstrapError {}

pub(super) fn decode(body: &[u8]) -> Result<BootstrapCapabilities, BootstrapPayloadIssue> {
    use BootstrapPayloadIssue as Issue;

    let [command] = read_members::<Command, 1>(body, ["LL"]).map_err(|_| Issue::InvalidEnvelope)?;
    let Command([control, status, value]) = command.ok_or(Issue::InvalidEnvelope)?;
    let status = status
        .as_ref()
        .and_then(parse_status)
        .ok_or(Issue::InvalidEnvelope)?;
    let control = control
        .as_ref()
        .and_then(Value::as_str)
        .ok_or(Issue::InvalidEnvelope)?;
    if control != "dev/cfg/api" {
        return Err(Issue::UnexpectedControl);
    }
    if status != 200 {
        return Err(Issue::CommandStatus(status));
    }

    let [event_slots, tls_status] = value
        .as_ref()
        .and_then(Value::as_str)
        .and_then(decode_value)
        .ok_or(Issue::UnsupportedValueEncoding)?;
    Ok(BootstrapCapabilities {
        event_slots: observe(event_slots.as_ref(), Value::as_bool),
        tls_status: observe(tls_status.as_ref(), |value| match value.as_u64() {
            Some(1) => Some(TlsStatus::Available),
            Some(2) => Some(TlsStatus::CertificateExpired),
            _ => None,
        }),
    })
}

struct Command([Option<Value>; 3]);

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_map(Members::new(["control", "Code", "value"]))
            .map(Self)
    }
}

fn read_members<'de, T: Deserialize<'de>, const N: usize>(
    input: &'de [u8],
    names: [&'static str; N],
) -> Result<[Option<T>; N], serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let fields = deserializer.deserialize_map(Members::new(names))?;
    deserializer.end()?;
    Ok(fields)
}

// Visit protocol-level keys as decoded by serde, before a Value map can collapse them.
struct Members<T, const N: usize> {
    names: [&'static str; N],
    marker: PhantomData<T>,
}

impl<T, const N: usize> Members<T, N> {
    fn new(names: [&'static str; N]) -> Self {
        Self {
            names,
            marker: PhantomData,
        }
    }
}

impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Members<T, N> {
    type Value = [Option<T>; N];

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bootstrap object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut fields: Self::Value = std::array::from_fn(|_| None);
        while let Some(key) = map.next_key::<String>()? {
            let Some(index) = self.names.iter().position(|&name| match name {
                "Code" => key.eq_ignore_ascii_case(name),
                _ => key == name,
            }) else {
                map.next_value::<Value>()?;
                continue;
            };
            let name = self.names[index];
            if fields[index].is_some() {
                return Err(de::Error::duplicate_field(name));
            }
            if name == "Code" && !matches!(key.as_str(), "Code" | "code") {
                return Err(de::Error::custom("unsupported bootstrap status spelling"));
            }
            fields[index] = Some(map.next_value()?);
        }
        Ok(fields)
    }
}

fn parse_status(value: &Value) -> Option<u16> {
    let code = match value {
        Value::String(text) if text.len() == 3 && text.bytes().all(|b| b.is_ascii_digit()) => {
            text.parse().ok()?
        }
        _ => u16::try_from(value.as_u64()?).ok()?,
    };
    (100..=599).contains(&code).then_some(code)
}

fn observe<T>(value: Option<&Value>, read: impl FnOnce(&Value) -> Option<T>) -> Capability<T> {
    match value {
        None => Capability::Missing,
        Some(value) => read(value).map_or(Capability::Unknown, Capability::Reported),
    }
}

fn decode_value(value: &str) -> Option<[Option<Value>; 2]> {
    let names = ["hasEventSlots", "httpsStatus"];
    if let Ok(fields) = read_members(value.as_bytes(), names) {
        return Some(fields);
    }
    read_members(&normalize_apostrophes(value)?, names).ok()
}

// Extend only simple apostrophe-delimited tokens; preserve double-quoted strings and escapes.
fn normalize_apostrophes(value: &str) -> Option<Vec<u8>> {
    let mut json = Vec::with_capacity(value.len());
    let mut bytes = value.bytes();
    let mut in_string = false;
    let mut changed = false;
    while let Some(byte) = bytes.next() {
        match byte {
            b'"' => {
                in_string = !in_string;
                json.push(byte);
            }
            b'\\' if in_string => {
                json.push(byte);
                json.push(bytes.next()?);
            }
            b'\'' if !in_string => {
                changed = true;
                json.push(b'"');
                loop {
                    match bytes.next()? {
                        b'\'' => break,
                        b'"' | b'\\' => return None,
                        plain @ b' '..=b'~' => json.push(plain),
                        _ => return None,
                    }
                }
                json.push(b'"');
            }
            _ => json.push(byte),
        }
    }
    changed.then_some(json)
}
