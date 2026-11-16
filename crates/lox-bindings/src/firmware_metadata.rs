//! Offline observations only; parsing, ordering and calendar policy stay in Core.

use lox_core::firmware_metadata as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct FirmwareVersion {
    pub normalized: String,
    pub components: Vec<u32>,
}

impl From<core::FirmwareVersion> for FirmwareVersion {
    fn from(value: core::FirmwareVersion) -> Self {
        Self {
            normalized: value.to_string(),
            components: value.components().to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ReleaseMetadata {
    pub version: FirmwareVersion,
    pub config_download: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct RemoteConnectMetadata {
    pub minimum_version: FirmwareVersion,
    pub deadline: String,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct UpdateMetadata {
    pub release: Option<ReleaseMetadata>,
    pub remote_connect: Option<RemoteConnectMetadata>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FirmwareOrdering {
    Less,
    Equal,
    Greater,
}

macro_rules! observation_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
        pub enum $name { $($variant),+ }
        impl From<core::$name> for $name {
            fn from(value: core::$name) -> Self {
                match value { $(core::$name::$variant => Self::$variant),+ }
            }
        }
    };
}

observation_enum!(ReleaseStatus {
    Unavailable,
    Newer,
    Equal,
    Older,
});
observation_enum!(ProgramCompatibility {
    SameMajorMinor,
    DifferentMajorMinor,
});
observation_enum!(DeadlineRelation { Before, On, Past });

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct RemoteConnectObservation {
    pub below_minimum: bool,
    pub deadline: DeadlineRelation,
    pub warning_due: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct UpdateAssessment {
    pub release_status: ReleaseStatus,
    pub program_compatibility: Option<ProgramCompatibility>,
    pub remote_connect: Option<RemoteConnectObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum FirmwareMetadataError {
    #[error("invalid update XML: {source_message}")]
    Xml { source_message: String },
    #[error("invalid update XML attribute: {source_message}")]
    Attribute { source_message: String },
    #[error("invalid update document: {reason}")]
    InvalidDocument { reason: String },
    #[error("duplicate update element {element}")]
    DuplicateElement { element: String },
    #[error("update element {element} has no {attribute}")]
    MissingAttribute { element: String, attribute: String },
    #[error("{message}")]
    InvalidVersion { value: String, message: String },
    #[error("{message}")]
    InvalidDate { value: String, message: String },
}

impl From<core::FirmwareMetadataError> for FirmwareMetadataError {
    fn from(error: core::FirmwareMetadataError) -> Self {
        let message = error.to_string();
        match error {
            core::FirmwareMetadataError::Xml(source) => Self::Xml {
                source_message: source.to_string(),
            },
            core::FirmwareMetadataError::Attribute(source) => Self::Attribute {
                source_message: source.to_string(),
            },
            core::FirmwareMetadataError::InvalidDocument(reason) => Self::InvalidDocument {
                reason: reason.into(),
            },
            core::FirmwareMetadataError::DuplicateElement(element) => Self::DuplicateElement {
                element: element.into(),
            },
            core::FirmwareMetadataError::MissingAttribute { element, attribute } => {
                Self::MissingAttribute {
                    element: element.into(),
                    attribute: attribute.into(),
                }
            }
            core::FirmwareMetadataError::InvalidVersion { value } => {
                Self::InvalidVersion { value, message }
            }
            core::FirmwareMetadataError::InvalidDate { value } => {
                Self::InvalidDate { value, message }
            }
        }
    }
}

#[uniffi::export]
pub fn parse_firmware_version(value: String) -> Result<FirmwareVersion, FirmwareMetadataError> {
    value
        .parse::<core::FirmwareVersion>()
        .map(Into::into)
        .map_err(Into::into)
}

/// Numeric lexicographic ordering, without zero padding.
#[uniffi::export]
pub fn compare_firmware_versions(
    left: String,
    right: String,
) -> Result<FirmwareOrdering, FirmwareMetadataError> {
    let left: core::FirmwareVersion = left.parse()?;
    let right: core::FirmwareVersion = right.parse()?;
    Ok(match left.cmp(&right) {
        std::cmp::Ordering::Less => FirmwareOrdering::Less,
        std::cmp::Ordering::Equal => FirmwareOrdering::Equal,
        std::cmp::Ordering::Greater => FirmwareOrdering::Greater,
    })
}

#[uniffi::export]
pub fn parse_firmware_metadata(xml: String) -> Result<UpdateMetadata, FirmwareMetadataError> {
    let value = core::UpdateMetadata::parse(&xml)?;
    Ok(UpdateMetadata {
        release: value.release.map(|release| ReleaseMetadata {
            version: release.version.into(),
            config_download: release.config_download,
        }),
        remote_connect: value.remote_connect.map(|remote| RemoteConnectMetadata {
            minimum_version: remote.minimum_version.into(),
            deadline: remote.deadline.to_string(),
        }),
    })
}

/// Explicit date required. Neither warning_due nor its absence implies usability.
#[uniffi::export]
pub fn assess_firmware_metadata(
    xml: String,
    current: String,
    program: Option<String>,
    as_of: String,
) -> Result<UpdateAssessment, FirmwareMetadataError> {
    let metadata = core::UpdateMetadata::parse(&xml)?;
    let current = current.parse()?;
    let program = program.as_deref().map(str::parse).transpose()?;
    let value = metadata.assess(&current, program.as_ref(), as_of.parse()?);
    Ok(UpdateAssessment {
        release_status: value.release_status.into(),
        program_compatibility: value.program_compatibility.map(Into::into),
        remote_connect: value.remote_connect.map(|remote| RemoteConnectObservation {
            below_minimum: remote.below_minimum,
            deadline: remote.deadline.into(),
            warning_due: remote.warning_due,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<Miniserversoftware><Release Version="016.2.4294967295" Path="a&amp;b"/><RemoteConnect MinVersion="16.2.0" Deadline="2028-02-29"/></Miniserversoftware>"#;

    #[test]
    fn owned_values_unsigned_components_and_numeric_order_survive() {
        let value = parse_firmware_metadata(XML.into()).unwrap();
        let release = value.release.unwrap();
        assert_eq!(release.version.components, [16, 2, u32::MAX]);
        assert_eq!(release.version.normalized, "16.2.4294967295");
        assert_eq!(release.config_download.as_deref(), Some("a&b"));
        assert_eq!(
            compare_firmware_versions("1.2".into(), "1.2.0".into()).unwrap(),
            FirmwareOrdering::Less
        );
        assert_eq!(
            compare_firmware_versions("01.2".into(), "1.2".into()).unwrap(),
            FirmwareOrdering::Equal
        );
    }

    #[test]
    fn explicit_calendar_and_optional_observations_preserve_core_predicates() {
        for (date, relation, warning) in [
            ("2028-02-28", DeadlineRelation::Before, true),
            ("2028-02-29", DeadlineRelation::On, true),
            ("2028-03-01", DeadlineRelation::Past, false),
        ] {
            let value = assess_firmware_metadata(
                XML.into(),
                "16.1".into(),
                Some("16.2".into()),
                date.into(),
            )
            .unwrap();
            assert_eq!(value.release_status, ReleaseStatus::Newer);
            assert_eq!(
                value.program_compatibility,
                Some(ProgramCompatibility::SameMajorMinor)
            );
            let remote = value.remote_connect.unwrap();
            assert!(remote.below_minimum);
            assert_eq!((remote.deadline, remote.warning_due), (relation, warning));
        }
        let absent = assess_firmware_metadata(
            "<Miniserversoftware/>".into(),
            "1".into(),
            None,
            "2000-01-01".into(),
        )
        .unwrap();
        assert_eq!(absent.release_status, ReleaseStatus::Unavailable);
        assert!(absent.remote_connect.is_none() && absent.program_compatibility.is_none());
    }

    #[test]
    fn fallible_core_errors_keep_raw_fields_and_source_display() {
        let raw = "invalid-version-".to_owned() + &"x".repeat(65536);
        let expected = raw
            .parse::<core::FirmwareVersion>()
            .unwrap_err()
            .to_string();
        assert_eq!(
            parse_firmware_version(raw.clone()),
            Err(FirmwareMetadataError::InvalidVersion {
                value: raw,
                message: expected
            })
        );
        assert!(
            matches!(assess_firmware_metadata(XML.into(), "1".into(), None, "2027-02-29".into()), Err(FirmwareMetadataError::InvalidDate { value, .. }) if value == "2027-02-29")
        );
        assert!(
            matches!(parse_firmware_metadata("<Miniserversoftware><Release/></Miniserversoftware>".into()), Err(FirmwareMetadataError::MissingAttribute { element, attribute }) if element == "Release" && attribute == "Version")
        );
        assert!(matches!(parse_firmware_metadata("<Miniserversoftware><Release Version=\"1\"/><Release Version=\"2\"/></Miniserversoftware>".into()), Err(FirmwareMetadataError::DuplicateElement { .. })));
    }
}
