use chrono::{DateTime, FixedOffset};
use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

/// Exact components of the caller's Core `DateTime<FixedOffset>`.
#[derive(Clone, Copy, Debug, uniffi::Record)]
pub struct SetAttributeClock {
    pub unix_seconds: i64,
    pub nanoseconds: u32,
    pub offset_seconds: i32,
}

impl SetAttributeClock {
    fn into_datetime(self) -> Option<DateTime<FixedOffset>> {
        let utc = DateTime::from_timestamp(self.unix_seconds, self.nanoseconds)?;
        let offset = FixedOffset::east_opt(self.offset_seconds)?;
        Some(utc.with_timezone(&offset))
    }
}

/// Lower XML and all three fallible strings in one request buffer.
#[derive(uniffi::Record)]
pub struct SetAttributeRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
    pub attribute: String,
    pub value: String,
    pub clock: SetAttributeClock,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct SetAttributeElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl From<core::ElementIdentity> for SetAttributeElementIdentity {
    fn from(identity: core::ElementIdentity) -> Self {
        Self {
            title: identity.title,
            block_type: identity.block_type,
            uuid: identity.uuid,
        }
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct SetAttributeChange {
    pub element: SetAttributeElementIdentity,
    pub attribute: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct SetAttributeOutcome {
    pub bytes: Vec<u8>,
    pub change: SetAttributeChange,
    pub content_changed: bool,
    pub document_stamped: bool,
}

impl From<core::SetAttributeOutcome> for SetAttributeOutcome {
    fn from(outcome: core::SetAttributeOutcome) -> Self {
        Self {
            bytes: outcome.bytes,
            change: SetAttributeChange {
                element: outcome.change.element.into(),
                attribute: outcome.change.attribute,
                old_value: outcome.change.old_value,
                new_value: outcome.change.new_value,
            },
            content_changed: outcome.content_changed,
            document_stamped: outcome.document_stamped,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum SetAttributeError {
    #[error("Clock components cannot form a fixed-offset instant")]
    InvalidClock,
    #[error("Configuration load failed: {source_message}")]
    Load { source_message: String },
    #[error("No element matches {selector:?}")]
    NotFound {
        selector: String,
        titles: Vec<String>,
    },
    #[error("Multiple elements match {selector:?}")]
    Ambiguous {
        selector: String,
        matches: Vec<SetAttributeElementIdentity>,
    },
    #[error("Configuration serialization failed: {source_message}")]
    Serialize { source_message: String },
    #[error("Configuration serialization is not UTF-8: {source_message}")]
    InvalidUtf8 { source_message: String },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<EditError> for SetAttributeError {
    fn from(error: EditError) -> Self {
        match error {
            EditError::Load(cause) => Self::Load {
                source_message: cause.to_string(),
            },
            EditError::Selection(SelectionError::NotFound { selector, titles }) => {
                Self::NotFound { selector, titles }
            }
            EditError::Selection(SelectionError::Ambiguous { selector, matches }) => {
                Self::Ambiguous {
                    selector,
                    matches: matches.into_iter().map(Into::into).collect(),
                }
            }
            EditError::Serialize(cause) => Self::Serialize {
                source_message: cause.to_string(),
            },
            EditError::InvalidUtf8(cause) => Self::InvalidUtf8 {
                source_message: cause.to_string(),
            },
            other => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Set one raw attribute in caller-supplied XML through the published Core transaction.
///
/// The legacy selector, name/value admission, canonical bytes, and conditional save
/// stamp are Core's. No file I/O, schema validation, reference repair, or machine
/// clock read is performed. Errors carry no revised XML.
#[uniffi::export]
pub fn set_attribute(
    request: SetAttributeRequest,
) -> Result<SetAttributeOutcome, SetAttributeError> {
    let clock = request
        .clock
        .into_datetime()
        .ok_or(SetAttributeError::InvalidClock)?;
    core::set_attribute(
        &request.bytes,
        &request.selector,
        &request.attribute,
        &request.value,
        &clock,
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: SetAttributeClock = SetAttributeClock {
        unix_seconds: 1_230_768_000,
        nanoseconds: 123_456_789,
        offset_seconds: 3661,
    };

    fn request(bytes: &[u8], selector: &str, attribute: &str, value: &str) -> SetAttributeRequest {
        SetAttributeRequest {
            bytes: bytes.to_vec(),
            selector: selector.into(),
            attribute: attribute.into(),
            value: value.into(),
            clock: CLOCK,
        }
    }

    #[test]
    fn projects_exact_core_bytes_and_distinct_change_observations() {
        let clock = CLOCK.into_datetime().unwrap();
        assert_eq!(clock.timestamp(), CLOCK.unix_seconds);
        assert_eq!(clock.timestamp_subsec_nanos(), CLOCK.nanoseconds);
        assert_eq!(clock.offset().local_minus_utc(), CLOCK.offset_seconds);
        let mut bytes = b"\xef\xbb\xbf<C Type=\"Document\" DateS=\"30\">\r\n<C Type=\"And\" U=\"a\" Title=\"Old\" Empty=\"\"/><C U=\"a\" Title=\"Mirror\"/>\r\n</C>\r\n".to_vec();
        for (attr, value, old, title, changed) in [
            ("Title", "New", Some("Old"), "Old", true),
            ("Empty", "", Some(""), "New", false),
            ("future", "<&\"\n室", None, "New", true),
        ] {
            let expected = core::set_attribute(&bytes, "uuid:a", attr, value, &clock).unwrap();
            let actual = set_attribute(request(&bytes, "uuid:a", attr, value)).unwrap();
            assert_eq!(actual.bytes, expected.bytes);
            assert_eq!(actual.change.element.title, title);
            assert_eq!(actual.change.element.block_type, "And");
            assert_eq!(actual.change.element.uuid, "a");
            assert_eq!(actual.change.old_value.as_deref(), old);
            assert_eq!(actual.change.attribute, attr);
            assert_eq!(actual.change.new_value, value);
            assert_eq!(
                (actual.content_changed, actual.document_stamped),
                (changed, changed)
            );
            bytes = actual.bytes;
        }
    }

    #[test]
    fn preserves_clock_load_and_ordered_selection_refusals() {
        let invalid = b"<C attr='1' attr='2'/>";
        let mut bad = request(invalid, "missing", "", "");
        bad.clock.offset_seconds = 86_400;
        assert_eq!(set_attribute(bad), Err(SetAttributeError::InvalidClock));
        let EditError::Load(cause) =
            core::set_attribute(invalid, "missing", "", "", &CLOCK.into_datetime().unwrap())
                .unwrap_err()
        else {
            panic!("expected Core load error before selection");
        };
        assert_eq!(
            set_attribute(request(invalid, "missing", "", "")),
            Err(SetAttributeError::Load {
                source_message: cause.to_string()
            })
        );
        let xml = br#"<Root><C U="a" Type="And" Title="Gate Z"/><C U="b" Title="Gate A"/><C U="a" Title="Gate Mirror"/></Root>"#;
        assert_eq!(
            set_attribute(request(xml, "missing", "", "")),
            Err(SetAttributeError::NotFound {
                selector: "missing".into(),
                titles: ["Gate A", "Gate Mirror", "Gate Z"]
                    .map(str::to_owned)
                    .to_vec(),
            })
        );
        assert_eq!(
            set_attribute(request(xml, "gate", "", "")),
            Err(SetAttributeError::Ambiguous {
                selector: "gate".into(),
                matches: vec![
                    SetAttributeElementIdentity {
                        title: "Gate Z".into(),
                        block_type: "And".into(),
                        uuid: "a".into(),
                    },
                    SetAttributeElementIdentity {
                        title: "Gate A".into(),
                        block_type: "".into(),
                        uuid: "b".into(),
                    },
                ],
            })
        );
    }
}
