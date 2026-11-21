use chrono::{DateTime, FixedOffset};
use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

/// Exact components of the caller's Core `DateTime<FixedOffset>`.
#[derive(Clone, Copy, Debug, uniffi::Record)]
pub struct UnwireClock {
    pub unix_seconds: i64,
    pub nanoseconds: u32,
    pub offset_seconds: i32,
}

impl UnwireClock {
    fn into_datetime(self) -> Option<DateTime<FixedOffset>> {
        let utc = DateTime::from_timestamp(self.unix_seconds, self.nanoseconds)?;
        let offset = FixedOffset::east_opt(self.offset_seconds)?;
        Some(utc.with_timezone(&offset))
    }
}

/// One lowering transaction for bytes and both independently fallible strings.
#[derive(uniffi::Record)]
pub struct UnwireRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
    pub connector: String,
    pub clock: UnwireClock,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct UnwireElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl From<core::ElementIdentity> for UnwireElementIdentity {
    fn from(value: core::ElementIdentity) -> Self {
        Self {
            title: value.title,
            block_type: value.block_type,
            uuid: value.uuid,
        }
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct UnwireChange {
    pub element: UnwireElementIdentity,
    pub connector: String,
    pub previous_uuid: Option<String>,
    pub removed_references: u64,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct UnwireOutcome {
    pub bytes: Vec<u8>,
    pub change: UnwireChange,
    pub content_changed: bool,
    pub document_stamped: bool,
}

impl From<core::UnwireOutcome> for UnwireOutcome {
    fn from(value: core::UnwireOutcome) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        Self {
            bytes: value.bytes,
            change: UnwireChange {
                element: value.change.element.into(),
                connector: value.change.connector,
                previous_uuid: value.change.previous_uuid,
                removed_references: value.change.removed_references as u64,
            },
            content_changed: value.content_changed,
            document_stamped: value.document_stamped,
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum UnwireError {
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
        matches: Vec<UnwireElementIdentity>,
    },
    #[error("Connector '{connector}' not found on '{title}'", title = .element.title)]
    ConnectorMissing {
        element: UnwireElementIdentity,
        connector: String,
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

impl From<core::UnwireError> for UnwireError {
    fn from(value: core::UnwireError) -> Self {
        match value {
            core::UnwireError::ConnectorMissing { element, connector } => Self::ConnectorMissing {
                element: element.into(),
                connector,
            },
            core::UnwireError::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            core::UnwireError::Edit(EditError::Selection(SelectionError::NotFound {
                selector,
                titles,
            })) => Self::NotFound { selector, titles },
            core::UnwireError::Edit(EditError::Selection(SelectionError::Ambiguous {
                selector,
                matches,
            })) => Self::Ambiguous {
                selector,
                matches: matches.into_iter().map(Into::into).collect(),
            },
            core::UnwireError::Edit(EditError::Serialize(cause)) => Self::Serialize {
                source_message: cause.to_string(),
            },
            core::UnwireError::Edit(EditError::InvalidUtf8(cause)) => Self::InvalidUtf8 {
                source_message: cause.to_string(),
            },
            core::UnwireError::Edit(other) => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Disconnect a connector in caller-supplied configuration bytes, without file I/O.
///
/// The clock is an already supplied value, not a request to read a machine clock.
/// After validating its representation, Core alone decides whether to stamp.
/// A canonical no-op can normalize bytes without stamping. Refusals carry no bytes.
#[uniffi::export]
pub fn unwire(request: UnwireRequest) -> Result<UnwireOutcome, UnwireError> {
    let clock = request
        .clock
        .into_datetime()
        .ok_or(UnwireError::InvalidClock)?;
    core::unwire(
        &request.bytes,
        &request.selector,
        &request.connector,
        &clock,
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: UnwireClock = UnwireClock {
        unix_seconds: 1_230_768_000,
        nanoseconds: 123_456_789,
        offset_seconds: 3661,
    };

    fn run(bytes: &[u8], selector: &str, connector: &str) -> Result<UnwireOutcome, UnwireError> {
        unwire(UnwireRequest {
            bytes: bytes.to_vec(),
            selector: selector.into(),
            connector: connector.into(),
            clock: CLOCK,
        })
    }

    #[test]
    fn exact_clock_and_raw_bytes_match_core_for_each_pass() {
        let now = CLOCK.into_datetime().unwrap();
        assert_eq!(now.timestamp(), CLOCK.unix_seconds);
        assert_eq!(now.timestamp_subsec_nanos(), CLOCK.nanoseconds);
        assert_eq!(now.offset().local_minus_utc(), CLOCK.offset_seconds);
        let mut bytes = b"\xef\xbb\xbf<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<C Type=\"Document\" DateS=\"30\"><C U=\"a\"><Co K=\"I1\" Nc=\"1\" U=\"raw\"><In Input=\"a\"/></Co></C></C>\r\n".to_vec();
        for (reference_count, uuid, change, stamp) in [
            (1, Some("raw"), true, "31"),
            (0, Some("raw"), true, "32"),
            (0, None, false, "32"),
        ] {
            let expected = core::unwire(&bytes, "uuid:a", "I1", &now).unwrap();
            let actual = run(&bytes, "uuid:a", "I1").unwrap();
            assert_eq!(actual.bytes, expected.bytes);
            assert!(String::from_utf8_lossy(&actual.bytes).contains(&format!("DateS=\"{stamp}\"")));
            assert_eq!(actual.change.previous_uuid.as_deref(), uuid);
            assert_eq!(actual.change.removed_references, reference_count);
            assert_eq!(
                (actual.content_changed, actual.document_stamped),
                (change, change)
            );
            bytes = actual.bytes;
        }
    }

    #[test]
    fn boundary_clock_refusal_is_distinct_from_core_load_and_selection() {
        for clock in [
            UnwireClock {
                unix_seconds: i64::MAX,
                ..CLOCK
            },
            UnwireClock {
                nanoseconds: u32::MAX,
                ..CLOCK
            },
            UnwireClock {
                offset_seconds: 86_400,
                ..CLOCK
            },
        ] {
            assert_eq!(
                unwire(UnwireRequest {
                    bytes: b"<C>".to_vec(),
                    selector: "absent".into(),
                    connector: "I1".into(),
                    clock,
                }),
                Err(UnwireError::InvalidClock)
            );
        }
        let core::UnwireError::Edit(EditError::Load(cause)) =
            core::unwire(b"<C>", "absent", "I1", &CLOCK.into_datetime().unwrap()).unwrap_err()
        else {
            panic!("expected Core load error");
        };
        assert_eq!(
            run(b"<C>", "absent", "I1"),
            Err(UnwireError::Load {
                source_message: cause.to_string()
            })
        );
        let source = br#"<C Type="And" Title="Gate" U="a"><Co K="I1"/></C>"#;
        assert_eq!(
            run(source, "absent", "I1"),
            Err(UnwireError::NotFound {
                selector: "absent".into(),
                titles: vec!["Gate".into()]
            })
        );
        assert_eq!(
            run(source, "uuid:a", "i1"),
            Err(UnwireError::ConnectorMissing {
                element: UnwireElementIdentity {
                    title: "Gate".into(),
                    block_type: "And".into(),
                    uuid: "a".into(),
                },
                connector: "i1".into(),
            })
        );
    }
}
