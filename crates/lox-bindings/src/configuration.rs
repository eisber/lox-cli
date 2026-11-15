//! Owned inspection results; the distinct XML contracts belong to core.

use lox_core::configuration::{controls, devices, rooms, summary, users};
use std::collections::HashMap;

/// Raw user inventory only, not credentials or permission evaluation.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct XmlUserInfo {
    pub name: String,
    pub uuid: String,
    pub nfc: bool,
    pub description: String,
}

impl From<users::UserInfo> for XmlUserInfo {
    fn from(value: users::UserInfo) -> Self {
        Self {
            name: value.name,
            uuid: value.uuid,
            nfc: value.nfc,
            description: value.description,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum XmlDeviceBus {
    Tree,
    Air,
    Network,
}

/// Read-only SAX inventory, not DOM device identity or port discovery.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct XmlDeviceInfo {
    pub name: String,
    pub bus: XmlDeviceBus,
    pub serial: Option<String>,
    /// Widened from core u32 so Java's signed Long retains every value.
    pub sub_type: Option<u64>,
    pub type_label: String,
    pub mac: Option<String>,
    pub address: Option<String>,
}

impl From<devices::DeviceInfo> for XmlDeviceInfo {
    fn from(value: devices::DeviceInfo) -> Self {
        Self {
            name: value.name,
            bus: match value.bus {
                devices::DeviceBus::Tree => XmlDeviceBus::Tree,
                devices::DeviceBus::Air => XmlDeviceBus::Air,
                devices::DeviceBus::Network => XmlDeviceBus::Network,
            },
            serial: value.serial,
            sub_type: value.sub_type.map(u64::from),
            type_label: value.type_label,
            mac: value.mac,
            address: value.address,
        }
    }
}

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

impl From<ConfigControlEntry> for summary::ControlEntry {
    fn from(value: ConfigControlEntry) -> Self {
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

impl From<ConfigSummary> for summary::ConfigSummary {
    fn from(value: ConfigSummary) -> Self {
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

impl From<devices::DeviceInventoryError> for ConfigurationError {
    fn from(value: devices::DeviceInventoryError) -> Self {
        match value {
            devices::DeviceInventoryError::Xml { source } => Self::Xml {
                source_message: source.to_string(),
            },
        }
    }
}

impl From<users::UserInventoryError> for ConfigurationError {
    fn from(value: users::UserInventoryError) -> Self {
        match value {
            users::UserInventoryError::Xml { source } => Self::Xml {
                source_message: source.to_string(),
            },
        }
    }
}

/// Preserve core's raw user fields, independent defaults and encounter order.
#[uniffi::export]
pub fn parse_users(xml: Vec<u8>) -> Result<Vec<XmlUserInfo>, ConfigurationError> {
    users::parse_users(&xml)
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

/// Preserve core's raw-byte, document-order device inventory without enrichment.
#[uniffi::export]
pub fn parse_devices(xml: Vec<u8>) -> Result<Vec<XmlDeviceInfo>, ConfigurationError> {
    devices::parse_devices(&xml)
        .map(|values| values.into_iter().map(Into::into).collect())
        .map_err(Into::into)
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
    fn users_preserve_raw_fields_first_attributes_order_and_independent_defaults() {
        let xml = include_bytes!("../tests/fixtures/xml_users.xml").to_vec();
        let before = xml.clone();
        let values = parse_users(xml.clone()).unwrap();
        assert_eq!(xml, before);
        assert_eq!(values.len(), 6);
        assert_eq!(
            values[0],
            XmlUserInfo {
                name: " raw&amp;&#65;&custom; ".into(),
                uuid: "not_a_uuid".into(),
                nfc: true,
                description: "one  two &#95; &amp;".into(),
            }
        );
        assert_eq!(values[1].name, "");
        assert_eq!(values[1].uuid, "");
        assert_eq!(values[1].description, "");
        assert!(!values[1].nfc);
        assert_eq!(values[2], values[3]);
        assert_eq!(values[2].description, " ");
        assert!(!values[2].nfc);
        assert_eq!(values[4].name, "first");
        assert!(values[4].nfc);
        assert_eq!(values[5].name, "Grün");
        assert_eq!(values[5].description, "Grüße 世界");
        assert_eq!(
            values,
            users::parse_users(&xml)
                .unwrap()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
        );
        let invalid = parse_users(
            b"<C Type=\"User\" Title=\"\xff\" U=\"valid_raw\" Desc=\"ok_value\" NFCArr=\"x\"/>\
              <C Type=\"User\" Title=\"valid\" U=\"\xff\" Desc=\"\xff\" NFCArr=\"\xff\"/>\
              <C Type=\"\xff\" Title=\"excluded\"/>"
                .to_vec(),
        )
        .unwrap();
        assert_eq!(
            invalid,
            vec![
                XmlUserInfo {
                    name: "".into(),
                    uuid: "valid_raw".into(),
                    nfc: true,
                    description: "ok value".into(),
                },
                XmlUserInfo {
                    name: "valid".into(),
                    uuid: "".into(),
                    nfc: false,
                    description: "".into(),
                }
            ]
        );
    }

    #[test]
    fn users_keep_permissive_attributes_eof_and_atomic_original_reader_error() {
        for xml in [
            b"plain text".as_slice(),
            b"",
            b"<C Type=\"User\" Title=\"open\">",
            b"<C Type=\"User\" Title=\"before-bad\" bad />",
            b"<C Type=\"User\" Title=\"\xff\" Title=\"not-a-fallback\"/>",
        ] {
            assert_eq!(
                parse_users(xml.to_vec()).unwrap(),
                users::parse_users(xml)
                    .unwrap()
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
            );
        }
        let xml =
            b"<Root><C Type=\"User\" Title=\"not partial\"/><secret>not-an-error-payload</Root>";
        let ConfigurationError::Xml { source_message } = parse_users(xml.to_vec()).unwrap_err();
        assert_eq!(
            source_message,
            users::parse_users(xml)
                .unwrap_err()
                .source()
                .unwrap()
                .to_string()
        );
        assert!(!source_message.contains("not-an-error-payload"));
    }

    #[test]
    fn devices_preserve_raw_branch_fields_defaults_order_and_unsigned_subtypes() {
        let xml = br#"<Root>
          <C Type="TreeDevice" Title=" raw&amp;&#65; " Title="ignored" Serial="" SubType="4294967295" MAC="ignored"/>
          <C Type="LoxAIRDevice" Title="same" Serial="ignored" SubType="7" Addr="ignored"/>
          <C Type="NetworkDevice" Title="same" Serial="ignored" SubType="4294967295" MAC="" Addr="raw&amp;"/>
          <C Type="TreeDevice" SubType=" 7"/>
          <C Type="treeDevice" Title="excluded"/>
          <C Type="TreeDevice" Title="repeat"/><C Type="TreeDevice" Title="repeat"/>
        </Root>"#;
        let values = parse_devices(xml.to_vec()).unwrap();
        assert_eq!(values.len(), 6);
        assert_eq!(
            values[0],
            XmlDeviceInfo {
                name: " raw&amp;&#65; ".into(),
                bus: XmlDeviceBus::Tree,
                serial: Some("".into()),
                sub_type: Some(4_294_967_295),
                type_label: "Unknown (4294967295)".into(),
                mac: None,
                address: None,
            }
        );
        assert_eq!(values[1].bus, XmlDeviceBus::Air);
        assert_eq!(values[1].serial, None);
        assert_eq!(values[1].type_label, "Nano IO Air");
        assert_eq!(values[2].bus, XmlDeviceBus::Network);
        assert_eq!(values[2].sub_type, Some(4_294_967_295));
        assert_eq!(values[2].type_label, "Network device");
        assert_eq!(values[2].mac.as_deref(), Some(""));
        assert_eq!(values[2].address.as_deref(), Some("raw&amp;"));
        assert_eq!(values[3].name, "");
        assert_eq!(values[3].sub_type, None);
        assert_eq!(values[3].type_label, "Unknown");
        assert_eq!(values[4], values[5]);
    }

    #[test]
    fn devices_keep_permissive_attributes_eof_and_original_reader_errors() {
        for xml in [
            b"plain text".as_slice(),
            b"",
            b"<C Type=\"TreeDevice\" Title=\"open\">",
            b"<C Type=\"TreeDevice\" Title=\"raw&amp;\" bad />",
            b"<C Type=\"TreeDevice\" Title=\"\xff\" Serial=\"\xff\"/>",
            b"<C Type=\"NetworkDevice\" SubType=\"4294967296\"/>",
        ] {
            let expected: Vec<XmlDeviceInfo> = devices::parse_devices(xml)
                .unwrap()
                .into_iter()
                .map(Into::into)
                .collect();
            assert_eq!(parse_devices(xml.to_vec()).unwrap(), expected);
        }
        let xml = b"<Root><C Type=\"TreeDevice\" Title=\"not partial\"/><secret>not-an-error-payload</Root>";
        let ConfigurationError::Xml { source_message } = parse_devices(xml.to_vec()).unwrap_err();
        assert_eq!(
            source_message,
            devices::parse_devices(xml)
                .unwrap_err()
                .source()
                .unwrap()
                .to_string()
        );
        assert!(!source_message.contains("not-an-error-payload"));
    }

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
