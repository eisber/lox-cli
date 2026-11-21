//! Owned inspection results; the distinct XML contracts belong to core.

use lox_core::configuration::{controls, rooms, summary};
use std::collections::HashMap;

/// Offline XML inventory, not runtime Control or summary ConfigControlEntry.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct XmlControlInfo {
    pub control_type: String,
    pub title: String,
    pub uuid: String,
    pub room: String,
    pub category: String,
}

impl From<controls::ControlInfo> for XmlControlInfo {
    fn from(value: controls::ControlInfo) -> Self {
        Self {
            control_type: value.control_type,
            title: value.title,
            uuid: value.uuid,
            room: value.room,
            category: value.category,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct RoomInfo {
    pub uuid: String,
    pub name: String,
    pub item_count: u64,
}

impl From<rooms::RoomInfo> for RoomInfo {
    fn from(value: rooms::RoomInfo) -> Self {
        // Every supported native usize fits without truncation in the FFI u64.
        const { assert!(usize::BITS <= u64::BITS) };
        Self {
            uuid: value.uuid,
            name: value.name,
            item_count: value.item_count as u64,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigControlEntry {
    pub name: String,
    pub control_type: String,
    pub room_uuid: String,
}

impl From<summary::ControlEntry> for ConfigControlEntry {
    fn from(value: summary::ControlEntry) -> Self {
        Self {
            name: value.name,
            control_type: value.control_type,
            room_uuid: value.room_uuid,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ConfigSummary {
    pub version: String,
    pub date: String,
    pub controls: HashMap<String, ConfigControlEntry>,
    pub rooms: HashMap<String, String>,
    pub categories: HashMap<String, String>,
    pub users: Vec<String>,
}

impl From<summary::ConfigSummary> for ConfigSummary {
    fn from(value: summary::ConfigSummary) -> Self {
        Self {
            version: value.version,
            date: value.date,
            controls: value
                .controls
                .into_iter()
                .map(|(key, entry)| (key, entry.into()))
                .collect(),
            rooms: value.rooms,
            categories: value.categories,
            users: value.users,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ConfigurationError {
    #[error("XML reader error: {source_message}")]
    Xml { source_message: String },
}

impl From<rooms::RoomInventoryError> for ConfigurationError {
    fn from(value: rooms::RoomInventoryError) -> Self {
        match value {
            rooms::RoomInventoryError::Xml { source } => Self::Xml {
                source_message: source.to_string(),
            },
        }
    }
}

impl From<summary::ConfigSummaryError> for ConfigurationError {
    fn from(value: summary::ConfigSummaryError) -> Self {
        match value {
            summary::ConfigSummaryError::Xml { source } => Self::Xml {
                source_message: source.to_string(),
            },
        }
    }
}

impl From<controls::ControlInventoryError> for ConfigurationError {
    fn from(value: controls::ControlInventoryError) -> Self {
        match value {
            controls::ControlInventoryError::Xml { source } => Self::Xml {
                source_message: source.to_string(),
            },
        }
    }
}

/// Read raw XML using core's pending-next-IoData inventory and filter admission.
#[uniffi::export]
pub fn parse_controls(
    xml: Vec<u8>,
    type_filter: Option<String>,
    room_filter: Option<String>,
) -> Result<Vec<XmlControlInfo>, ConfigurationError> {
    controls::parse_controls(&xml, type_filter.as_deref(), room_filter.as_deref())
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

/// Read raw XML bytes using core's permissive inventory, not archive detection.
#[uniffi::export]
pub fn parse_rooms(xml: Vec<u8>) -> Result<Vec<RoomInfo>, ConfigurationError> {
    rooms::parse_rooms(&xml)
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

/// Read raw XML bytes using core's distinct direct-parent summary contract.
#[uniffi::export]
pub fn parse_config_summary(xml: Vec<u8>) -> Result<ConfigSummary, ConfigurationError> {
    summary::parse_config_summary(&xml)
        .map(Into::into)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn xml_controls_preserve_every_field_filter_admission_and_owned_results() {
        let xml = include_bytes!("../tests/fixtures/xml_controls.xml").to_vec();
        let before = xml.clone();
        for typ in [None, Some("sWiTcH"), Some(""), Some("ä"), Some("Sw")] {
            for room in [
                None,
                Some("küc"),
                Some("raw&amp;"),
                Some(""),
                Some("absent"),
            ] {
                let expected: Vec<XmlControlInfo> = controls::parse_controls(&xml, typ, room)
                    .unwrap()
                    .into_iter()
                    .map(Into::into)
                    .collect();
                assert_eq!(
                    parse_controls(xml.clone(), typ.map(str::to_owned), room.map(str::to_owned))
                        .unwrap(),
                    expected
                );
            }
        }
        let rows = parse_controls(xml.clone(), None, None).unwrap();
        assert_eq!(xml, before);
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].uuid, "");
        assert_eq!(rows[1].uuid, "inner");
        assert_eq!(
            rows.iter()
                .filter(|r| r.title == "Same")
                .map(|r| r.uuid.as_str())
                .collect::<Vec<_>>(),
            ["second", "first", "first"]
        );
        let filtered = parse_controls(xml, Some("SWITCH".into()), Some("küc".into())).unwrap();
        assert!(filtered.iter().any(|r| r.uuid == "outer"));
        assert!(!rows.iter().any(|r| r.uuid == "outer"));
        assert_eq!(rows[1].category, "Raw &amp; &#65;");
    }

    #[test]
    fn xml_controls_keep_permissive_bytes_and_exact_reader_failure() {
        for xml in [
            b"plain text".as_slice(),
            b"",
            b"<C Type=\"Switch\" U=\"c\" Title=\"Open\"><IoData Pr=\"r\">",
            b"<C Type=\"Switch\" U=\"c\" Title=\"raw&amp;\" bad /><IoData/>",
            b"<C Type=\"Switch\" U=\"c\" Title=\"\xff\"/><IoData/>",
        ] {
            assert_eq!(
                parse_controls(xml.to_vec(), None, None).unwrap(),
                controls::parse_controls(xml, None, None)
                    .unwrap()
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
            );
        }
        let xml = b"<Root><C Type=\"Switch\" U=\"c\" Title=\"emitted\"/><IoData/><secret>not-an-error-payload</Root>";
        let ConfigurationError::Xml { source_message } =
            parse_controls(xml.to_vec(), None, None).unwrap_err();
        assert_eq!(
            source_message,
            controls::parse_controls(xml, None, None)
                .unwrap_err()
                .source()
                .unwrap()
                .to_string()
        );
        assert!(!source_message.contains("not-an-error-payload"));
    }

    #[test]
    fn every_owned_field_matches_core_without_changing_input_or_order() {
        let xml = include_bytes!("../tests/fixtures/config_inspection.xml").to_vec();
        let before = xml.clone();
        let expected_rooms: Vec<RoomInfo> = rooms::parse_rooms(&xml)
            .unwrap()
            .into_iter()
            .map(Into::into)
            .collect();
        let expected_summary: ConfigSummary = summary::parse_config_summary(&xml).unwrap().into();
        let actual_rooms = parse_rooms(xml.clone()).unwrap();
        assert_eq!(
            actual_rooms.iter().map(|r| &r.name).collect::<Vec<_>>(),
            expected_rooms.iter().map(|r| &r.name).collect::<Vec<_>>()
        );
        // Separate parser calls may choose different orders for equal names.
        assert_eq!(
            actual_rooms
                .into_iter()
                .map(|r| (r.uuid.clone(), r))
                .collect::<HashMap<_, _>>(),
            expected_rooms
                .into_iter()
                .map(|r| (r.uuid.clone(), r))
                .collect::<HashMap<_, _>>()
        );
        assert_eq!(parse_config_summary(xml.clone()).unwrap(), expected_summary);
        assert_eq!(xml, before);
    }

    #[test]
    fn permissive_inputs_and_exact_reader_errors_are_not_revalidated() {
        for xml in [
            b"plain text".as_slice(),
            b"",
            b"<C Type=\"Place\" U=\"r\" Title=\"Open\"><IoData Pr=\"r\">",
            b"<C Type=\"Switch\" U=\"c\" Title=\"Open\"><IoData Pr=\"r\">",
            b"<C Type=\"Place\" U=\"r\" Title=\"raw&amp;\" bad />",
            b"<C Type=\"Place\" U=\"r\" Title=\"\xff\"/>",
        ] {
            assert_eq!(
                parse_rooms(xml.to_vec()).unwrap(),
                rooms::parse_rooms(xml)
                    .unwrap()
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                parse_config_summary(xml.to_vec()).unwrap(),
                summary::parse_config_summary(xml).unwrap().into()
            );
        }
        let xml = b"<root><secret>not-an-error-payload</root>";
        let ConfigurationError::Xml { source_message } = parse_rooms(xml.to_vec()).unwrap_err();
        assert_eq!(
            source_message,
            rooms::parse_rooms(xml)
                .unwrap_err()
                .source()
                .unwrap()
                .to_string()
        );
        let ConfigurationError::Xml { source_message } =
            parse_config_summary(xml.to_vec()).unwrap_err();
        assert_eq!(
            source_message,
            summary::parse_config_summary(xml)
                .unwrap_err()
                .source()
                .unwrap()
                .to_string()
        );
        assert!(!source_message.contains("not-an-error-payload"));
    }

    #[test]
    fn count_conversion_handles_the_native_width_without_narrowing() {
        let value: RoomInfo = rooms::RoomInfo {
            uuid: String::new(),
            name: String::new(),
            item_count: usize::MAX,
        }
        .into();
        assert_eq!(u128::from(value.item_count), usize::MAX as u128);
    }
}
