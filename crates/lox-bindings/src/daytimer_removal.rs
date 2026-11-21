use chrono::{DateTime, FixedOffset};
use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::{collections::HashMap, error::Error};

/// Exact components of the caller's fixed-offset clock, as in Unwire.
#[derive(Clone, Copy, Debug, uniffi::Record)]
pub struct DaytimerRemovalClock {
    pub unix_seconds: i64,
    pub nanoseconds: u32,
    pub offset_seconds: i32,
}

impl DaytimerRemovalClock {
    fn into_datetime(self) -> Option<DateTime<FixedOffset>> {
        let utc = DateTime::from_timestamp(self.unix_seconds, self.nanoseconds)?;
        let offset = FixedOffset::east_opt(self.offset_seconds)?;
        Some(utc.with_timezone(&offset))
    }
}

/// One lowering transaction for XML bytes and both independently fallible strings.
#[derive(uniffi::Record)]
pub struct DaytimerRemovalRequest {
    pub bytes: Vec<u8>,
    pub block_uuid: String,
    pub mode_uuid: String,
    pub from_minutes: u32,
    pub to_minutes: u32,
    pub clock: DaytimerRemovalClock,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct DaytimerRemovedEntry {
    pub block_uuid: String,
    pub requested_mode_uuid: String,
    pub stored_mode_uuid: String,
    pub from_minutes: u32,
    pub to_minutes: u32,
    pub removed_attributes: HashMap<String, String>,
    pub remaining_entries: u64,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct DaytimerRemovalOutcome {
    pub bytes: Vec<u8>,
    pub removal: DaytimerRemovedEntry,
}

impl From<core::DayTimerRemovalOutcome> for DaytimerRemovalOutcome {
    fn from(outcome: core::DayTimerRemovalOutcome) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        let removal = outcome.removal;
        Self {
            bytes: outcome.bytes,
            removal: DaytimerRemovedEntry {
                block_uuid: removal.block_uuid,
                requested_mode_uuid: removal.requested_mode_uuid,
                stored_mode_uuid: removal.stored_mode_uuid,
                from_minutes: removal.from_minutes,
                to_minutes: removal.to_minutes,
                removed_attributes: removal.removed_attributes.into_iter().collect(),
                remaining_entries: removal.remaining_entries as u64,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum DaytimerRemovalError {
    #[error("Clock components cannot form a fixed-offset instant")]
    InvalidClock,
    #[error("Expected a non-nil native or RFC DayTimer block UUID")]
    InvalidBlockUuid,
    #[error("Configuration load failed: {source_message}")]
    Load { source_message: String },
    #[error("No element matches {selector:?}")]
    NotFound {
        selector: String,
        titles: Vec<String>,
    },
    #[error("Invalid operating mode UUID")]
    InvalidModeUuid,
    #[error("Entry times must satisfy 00:00 <= from < to <= 24:00")]
    InvalidInterval,
    #[error("'{selector}' is not a DayTimer")]
    NotDayTimer { selector: String },
    #[error("Only ordinary DayTimer M=0 schedules are supported")]
    UnsupportedScheduleMode,
    #[error("Only digital DayTimer schedules are supported")]
    AnalogSchedule,
    #[error("Unsupported DayTimer Analog spelling")]
    UnsupportedAnalog,
    #[error("DayTimer has no explicit Modes references")]
    MissingModes,
    #[error(
        "DayTimer Modes requires comma-separated non-nil native or RFC UUIDs without whitespace"
    )]
    InvalidModes,
    #[error("DayTimer Modes contains duplicate references")]
    DuplicateModes,
    #[error("DayTimer operating mode '{stored_mode_uuid}' is missing, ambiguous or not a Mode")]
    UnresolvedMode { stored_mode_uuid: String },
    #[error("Requested operating mode is not in this DayTimer's Modes")]
    ModeNotInSchedule,
    #[error("Invalid DayTimer {attribute}='{value}': expected a nonnegative signed-32-bit integer")]
    InvalidInteger { attribute: String, value: String },
    #[error("DayTimer N does not match stored entries")]
    EntryCountMismatch,
    #[error("DayTimer contains an invalid interval or unresolved Ix")]
    InvalidStoredInterval,
    #[error("DayTimer contains overlapping or touching entries in one mode")]
    TouchingOrOverlappingEntries,
    #[error("No existing entry matches that operating mode and exact interval")]
    EntryNotFound,
    #[error("DayTimer removal requires a direct Page child in one stampable Document")]
    OutsidePage,
    #[error("DayTimer removal does not admit prefixed or namespaced direct Entry elements")]
    NamespacedEntry,
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

impl DaytimerRemovalError {
    fn other(error: &dyn Error) -> Self {
        Self::Other {
            message: error.to_string(),
            source_message: error.source().map(ToString::to_string),
        }
    }
}

impl From<core::DayTimerRemovalError> for DaytimerRemovalError {
    fn from(value: core::DayTimerRemovalError) -> Self {
        use core::DayTimerRemovalError as Failure;
        match value {
            Failure::Edit(EditError::Load(cause)) => Self::Load {
                source_message: cause.to_string(),
            },
            Failure::Edit(EditError::Selection(SelectionError::NotFound { selector, titles })) => {
                Self::NotFound { selector, titles }
            }
            Failure::Edit(EditError::Serialize(cause)) => Self::Serialize {
                source_message: cause.to_string(),
            },
            Failure::Edit(EditError::InvalidUtf8(cause)) => Self::InvalidUtf8 {
                source_message: cause.to_string(),
            },
            Failure::Edit(other) => Self::other(&other),
            Failure::InvalidModeUuid => Self::InvalidModeUuid,
            Failure::InvalidInterval => Self::InvalidInterval,
            Failure::NotDayTimer { selector } => Self::NotDayTimer { selector },
            Failure::UnsupportedScheduleMode => Self::UnsupportedScheduleMode,
            Failure::AnalogSchedule => Self::AnalogSchedule,
            Failure::UnsupportedAnalog => Self::UnsupportedAnalog,
            Failure::MissingModes => Self::MissingModes,
            Failure::InvalidModes => Self::InvalidModes,
            Failure::DuplicateModes => Self::DuplicateModes,
            Failure::UnresolvedMode { stored_mode_uuid } => {
                Self::UnresolvedMode { stored_mode_uuid }
            }
            Failure::ModeNotInSchedule => Self::ModeNotInSchedule,
            Failure::InvalidInteger { attribute, value } => Self::InvalidInteger {
                attribute: attribute.to_owned(),
                value,
            },
            Failure::EntryCountMismatch => Self::EntryCountMismatch,
            Failure::InvalidStoredInterval => Self::InvalidStoredInterval,
            Failure::TouchingOrOverlappingEntries => Self::TouchingOrOverlappingEntries,
            Failure::EntryNotFound => Self::EntryNotFound,
            Failure::OutsidePage => Self::OutsidePage,
            Failure::NamespacedEntry => Self::NamespacedEntry,
            other => Self::other(&other),
        }
    }
}

/// Remove one existing digital DayTimer entry by exact block UUID, mode and interval.
///
/// The block spelling is not normalized and no CLI selectors are accepted.
/// Core supplies Page admission, mode identity, canonical bytes and the stamp.
/// Refusals carry no revised XML. There is no filesystem or clock access.
#[uniffi::export]
pub fn remove_daytimer_entry(
    request: DaytimerRemovalRequest,
) -> Result<DaytimerRemovalOutcome, DaytimerRemovalError> {
    let clock = request
        .clock
        .into_datetime()
        .ok_or(DaytimerRemovalError::InvalidClock)?;
    if core::daytimer_mode_uuid_key(&request.block_uuid).is_none() {
        return Err(DaytimerRemovalError::InvalidBlockUuid);
    }
    core::remove_daytimer_entry(
        &request.bytes,
        &format!("uuid:{}", request.block_uuid),
        &request.mode_uuid,
        request.from_minutes,
        request.to_minutes,
        &clock,
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = include_str!("../tests/fixtures/daytimer_removal.xml");
    const BLOCK: &str = "ABCDEF00-0000-0004-FFFF000000000000";
    const MODE: &str = "abcdef01-abcd-0004-1500-0000abcdef00";
    const CLOCK: DaytimerRemovalClock = DaytimerRemovalClock {
        unix_seconds: 1_230_768_000,
        nanoseconds: 123_456_789,
        offset_seconds: 3661,
    };

    fn request(xml: &str) -> DaytimerRemovalRequest {
        DaytimerRemovalRequest {
            bytes: xml.as_bytes().to_vec(),
            block_uuid: BLOCK.into(),
            mode_uuid: MODE.into(),
            from_minutes: 0,
            to_minutes: 180,
            clock: CLOCK,
        }
    }

    #[test]
    fn exact_core_projection_with_caller_clock() {
        let input = format!(
            "\u{feff}{}",
            XML.replace("\r\n", "\n").replace('\n', "\r\n")
        );
        let request = request(&input);
        let clock = CLOCK.into_datetime().unwrap();
        assert_eq!(clock.timestamp(), CLOCK.unix_seconds);
        assert_eq!(clock.timestamp_subsec_nanos(), CLOCK.nanoseconds);
        assert_eq!(clock.offset().local_minus_utc(), CLOCK.offset_seconds);
        let expected = core::remove_daytimer_entry(
            &request.bytes,
            &format!("uuid:{BLOCK}"),
            MODE,
            0,
            180,
            &clock,
        )
        .unwrap();
        let actual = remove_daytimer_entry(request).unwrap();
        assert_eq!(actual.bytes, expected.bytes);
        assert_eq!(
            actual.removal.removed_attributes,
            HashMap::from([
                ("To".into(), "180".into()),
                ("V".into(), "+005".into()),
                ("Marker".into(), "entfernt & 灯".into()),
            ])
        );
        assert_eq!(actual.removal.remaining_entries, 1);
        assert_eq!(actual.removal.block_uuid, BLOCK);
        assert_eq!(actual.removal.requested_mode_uuid, MODE);
        assert_eq!(
            actual.removal.stored_mode_uuid,
            "ABCDEF01-ABCD-0004-15000000ABCDEF00"
        );
    }

    #[test]
    fn input_and_core_refusals_are_distinct() {
        use DaytimerRemovalError as Failure;
        for clock in [
            DaytimerRemovalClock {
                unix_seconds: i64::MAX,
                ..CLOCK
            },
            DaytimerRemovalClock {
                nanoseconds: u32::MAX,
                ..CLOCK
            },
            DaytimerRemovalClock {
                offset_seconds: 86_400,
                ..CLOCK
            },
        ] {
            assert_eq!(
                remove_daytimer_entry(DaytimerRemovalRequest {
                    clock,
                    ..request("<broken")
                }),
                Err(Failure::InvalidClock)
            );
        }
        for block_uuid in [
            "Type:DayTimer",
            "Schedule [Room]",
            "00000000-0000-0000-0000000000000000",
        ] {
            assert_eq!(
                remove_daytimer_entry(DaytimerRemovalRequest {
                    block_uuid: block_uuid.into(),
                    ..request(XML)
                }),
                Err(Failure::InvalidBlockUuid)
            );
        }
        let core::DayTimerRemovalError::Edit(EditError::Load(cause)) = core::remove_daytimer_entry(
            b"<broken",
            &format!("uuid:{BLOCK}"),
            MODE,
            0,
            180,
            &CLOCK.into_datetime().unwrap(),
        )
        .unwrap_err() else {
            panic!("expected Core load error");
        };
        assert_eq!(
            remove_daytimer_entry(request("<broken")),
            Err(Failure::Load {
                source_message: cause.to_string()
            })
        );
        assert_eq!(
            remove_daytimer_entry(request(&XML.replace("Type=\"Page\"", "Type=\"Folder\""))),
            Err(Failure::OutsidePage)
        );
        assert_eq!(
            remove_daytimer_entry(request(&XML.replace(
                "<Entry To=\"180\" Ix",
                "<Entry xmlns=\"urn:test\" To=\"180\" Ix"
            ))),
            Err(Failure::NamespacedEntry)
        );
    }
}
