use chrono::{DateTime, FixedOffset};
use lox_core::configuration::editor::{self as core, EditError, SelectionError};
use std::error::Error;

#[derive(Clone, Copy, Debug, uniffi::Record)]
pub struct ControlMoveClock {
    pub unix_seconds: i64,
    pub nanoseconds: u32,
    pub offset_seconds: i32,
}

impl ControlMoveClock {
    fn into_datetime(self) -> Option<DateTime<FixedOffset>> {
        let utc = DateTime::from_timestamp(self.unix_seconds, self.nanoseconds)?;
        let offset = FixedOffset::east_opt(self.offset_seconds)?;
        Some(utc.with_timezone(&offset))
    }
}

/// One lowering transaction for bytes, optional selectors, room, exclusions and clock.
#[derive(uniffi::Record)]
pub struct ControlMoveRequest {
    pub bytes: Vec<u8>,
    pub type_filter: Option<String>,
    pub title: Option<String>,
    pub target_room: String,
    pub exclude: Vec<String>,
    pub clock: ControlMoveClock,
}

#[derive(Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ControlMoveSelector {
    Type { value: String },
    Title { value: String },
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct ControlMoveChange {
    pub selector: ControlMoveSelector,
    pub target_room: String,
    pub room_uuid: String,
    pub selected_count: u64,
    pub association_changed: bool,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct ControlMoveOutcome {
    pub bytes: Vec<u8>,
    pub change: ControlMoveChange,
    pub content_changed: bool,
    pub document_stamped: bool,
}

impl From<core::ControlMoveOutcome> for ControlMoveOutcome {
    fn from(outcome: core::ControlMoveOutcome) -> Self {
        const { assert!(usize::BITS <= u64::BITS) };
        let change = outcome.change;
        Self {
            bytes: outcome.bytes,
            change: ControlMoveChange {
                selector: match change.selector {
                    core::ControlMoveSelector::Type(value) => ControlMoveSelector::Type { value },
                    core::ControlMoveSelector::Title(value) => ControlMoveSelector::Title { value },
                },
                target_room: change.target_room,
                room_uuid: change.room_uuid,
                selected_count: change.selected_count as u64,
                association_changed: change.association_changed,
            },
            content_changed: outcome.content_changed,
            document_stamped: outcome.document_stamped,
        }
    }
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct ControlMoveElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum ControlMoveError {
    #[error("Clock components cannot form a fixed-offset instant")]
    InvalidClock,
    #[error("Configuration load failed: {source_message}")]
    Load { source_message: String },
    #[error("A type or title selector is required")]
    SelectorMissing,
    #[error("No element matches {selector:?}")]
    NotFound {
        selector: String,
        titles: Vec<String>,
    },
    #[error("Multiple elements match {selector:?}")]
    Ambiguous {
        selector: String,
        matches: Vec<ControlMoveElementIdentity>,
    },
    #[error("Room {room:?} was not found")]
    RoomNotFound { room: String },
    #[error("Multiple rooms match {room:?}")]
    RoomAmbiguous { room: String, matches: Vec<String> },
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

impl From<EditError> for ControlMoveError {
    fn from(error: EditError) -> Self {
        match error {
            EditError::Load(cause) => Self::Load {
                source_message: cause.to_string(),
            },
            EditError::ControlMoveSelectorMissing => Self::SelectorMissing,
            EditError::Selection(SelectionError::NotFound { selector, titles }) => {
                Self::NotFound { selector, titles }
            }
            EditError::Selection(SelectionError::Ambiguous { selector, matches }) => {
                Self::Ambiguous {
                    selector,
                    matches: matches
                        .into_iter()
                        .map(|element| ControlMoveElementIdentity {
                            title: element.title,
                            block_type: element.block_type,
                            uuid: element.uuid,
                        })
                        .collect(),
                }
            }
            EditError::RoomNotFound { room } => Self::RoomNotFound { room },
            EditError::RoomAmbiguous { room, matches } => Self::RoomAmbiguous { room, matches },
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

/// Change selected controls' direct `IoData.Pr` associations in supplied XML bytes.
///
/// The clock is validated before delegating to Core. `None` and an empty selector
/// are distinct, and the room in the outcome is the supplied query, not a room
/// name inferred from the UUID. The canonical output is returned as-is, even on a
/// no-op; refusals return no revised bytes. This API never opens a file or device.
#[uniffi::export]
pub fn move_control(request: ControlMoveRequest) -> Result<ControlMoveOutcome, ControlMoveError> {
    let clock = request
        .clock
        .into_datetime()
        .ok_or(ControlMoveError::InvalidClock)?;
    let exclude: Vec<_> = request.exclude.iter().map(String::as_str).collect();
    core::move_control(
        &request.bytes,
        request.type_filter.as_deref(),
        request.title.as_deref(),
        &request.target_room,
        &exclude,
        &clock,
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: ControlMoveClock = ControlMoveClock {
        unix_seconds: 1_230_768_000,
        nanoseconds: 123_456_789,
        offset_seconds: 3661,
    };
    const ROOM: &str = "KÜCHE 室";
    const MALFORMED: &[u8] = b"<C A='1' A='2'/>";

    fn fixture() -> Vec<u8> {
        format!(
            "\u{feff}{}",
            include_str!("../tests/fixtures/control_move.xml")
                .replace("\r\n", "\n")
                .replace('\n', "\r\n")
        )
        .into_bytes()
    }

    fn request(
        bytes: &[u8],
        type_filter: Option<&str>,
        title: Option<&str>,
        target_room: &str,
        exclude: &[&str],
    ) -> ControlMoveRequest {
        ControlMoveRequest {
            bytes: bytes.to_vec(),
            type_filter: type_filter.map(str::to_owned),
            title: title.map(str::to_owned),
            target_room: target_room.into(),
            exclude: exclude.iter().map(|value| (*value).to_owned()).collect(),
            clock: CLOCK,
        }
    }

    #[test]
    fn retains_raw_core_result_and_direct_change_for_both_selector_branches() {
        let data = fixture();
        let now = CLOCK.into_datetime().unwrap();
        assert_eq!(now.timestamp(), CLOCK.unix_seconds);
        assert_eq!(now.timestamp_subsec_nanos(), CLOCK.nanoseconds);
        assert_eq!(now.offset().local_minus_utc(), CLOCK.offset_seconds);
        for (kind, title, exclude, selector, count, changed) in [
            (
                None,
                Some("uuid:a"),
                &["FUTURE"][..],
                ControlMoveSelector::Title {
                    value: "uuid:a".into(),
                },
                1,
                true,
            ),
            (
                Some("fUtUrE"),
                Some("ignored"),
                &[][..],
                ControlMoveSelector::Type {
                    value: "fUtUrE".into(),
                },
                2,
                true,
            ),
            (
                Some("Future"),
                None,
                &["FUTURE"][..],
                ControlMoveSelector::Type {
                    value: "Future".into(),
                },
                0,
                false,
            ),
        ] {
            let actual = move_control(request(&data, kind, title, ROOM, exclude)).unwrap();
            let expected = core::move_control(&data, kind, title, ROOM, exclude, &now).unwrap();
            assert_eq!(actual.bytes, expected.bytes);
            assert_eq!(
                actual.change,
                ControlMoveChange {
                    selector,
                    target_room: ROOM.into(),
                    room_uuid: "Raw-r".into(),
                    selected_count: count,
                    association_changed: changed,
                }
            );
            assert_eq!(
                (actual.content_changed, actual.document_stamped),
                (changed, changed)
            );
        }
    }

    #[test]
    fn keeps_clock_load_selection_and_room_refusals_distinct() {
        let data = fixture();
        for clock in [
            ControlMoveClock {
                unix_seconds: i64::MAX,
                ..CLOCK
            },
            ControlMoveClock {
                nanoseconds: u32::MAX,
                ..CLOCK
            },
            ControlMoveClock {
                offset_seconds: 86_400,
                ..CLOCK
            },
        ] {
            let mut input = request(MALFORMED, None, None, "", &[]);
            input.clock = clock;
            assert_eq!(move_control(input), Err(ControlMoveError::InvalidClock));
        }
        let EditError::Load(cause) = core::move_control(
            MALFORMED,
            None,
            None,
            "",
            &[],
            &CLOCK.into_datetime().unwrap(),
        )
        .unwrap_err() else {
            panic!("expected Core load refusal");
        };
        assert_eq!(
            move_control(request(MALFORMED, None, None, "", &[])),
            Err(ControlMoveError::Load {
                source_message: cause.to_string(),
            })
        );
        assert_eq!(
            move_control(request(&data, None, None, "absent", &[])),
            Err(ControlMoveError::SelectorMissing)
        );
        assert_eq!(
            move_control(request(
                &data,
                Some("Future"),
                Some("absent"),
                "absent",
                &[]
            )),
            Err(ControlMoveError::RoomNotFound {
                room: "absent".into(),
            })
        );
        assert!(matches!(
            move_control(request(&data, None, Some("absent"), "absent", &[])),
            Err(ControlMoveError::NotFound { selector, titles })
                if selector == "absent" && titles.iter().any(|title| title == "Lamp Ω")
        ));
        let duplicate = br#"<Root><C Type="Place" U="r" Title="Room"/><C Type="Place" U="r" Title="Room"/></Root>"#;
        assert_eq!(
            move_control(request(duplicate, Some(""), None, "room", &[])),
            Err(ControlMoveError::RoomAmbiguous {
                room: "room".into(),
                matches: vec!["Room".into(), "Room".into()],
            })
        );
    }
}
