//! Shared strict LL policy; callers supply the exact command-specific echo.

use super::auth_reply_framing::AuthReplyText;
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde_json::value::RawValue;
use std::fmt;

pub(super) enum EnvelopeError {
    InvalidEnvelope,
    EchoMismatch,
    StatusNotAccepted,
}

pub(super) fn admit_value<'a>(
    reply: &AuthReplyText<'a>,
    matches_control: impl FnOnce(&str) -> bool,
) -> Result<&'a [u8], EnvelopeError> {
    let envelope: Envelope<'_> =
        serde_json::from_slice(reply.as_bytes()).map_err(|_| EnvelopeError::InvalidEnvelope)?;
    if !matches_control(&envelope.0.control) {
        return Err(EnvelopeError::EchoMismatch);
    }
    let status = envelope.0.code.get();
    let success = status == "200"
        || (status.starts_with('"')
            && serde_json::from_str::<String>(status).is_ok_and(|value| value == "200"));
    if !success {
        return Err(EnvelopeError::StatusNotAccepted);
    }
    Ok(envelope.0.value.get().as_bytes())
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
        f.write_str("an authentication reply body")
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
                        "unexpected or repeated authentication reply member",
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
