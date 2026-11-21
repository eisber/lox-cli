//! Offline update-server metadata observations, not update eligibility or trust validation.
//!
//! See the crate README for the pinned upstream mapping and deliberate strictness.

use quick_xml::Reader;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use std::{cmp::Ordering, fmt, str::FromStr};

/// Numeric dotted components, compared lexicographically without zero padding.
///
/// Leading zeroes are accepted and normalized. Empty, signed, non-ASCII,
/// overflowing and nonnumeric components are errors, never silently zero.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FirmwareVersion(Vec<u32>);

impl FirmwareVersion {
    pub fn components(&self) -> &[u32] {
        &self.0
    }

    fn same_major_minor(&self, other: &Self) -> bool {
        self.0.iter().take(2).eq(other.0.iter().take(2))
    }
}

impl FromStr for FirmwareVersion {
    type Err = FirmwareMetadataError;

    fn from_str(value: &str) -> Result<Self> {
        let invalid = || FirmwareMetadataError::InvalidVersion {
            value: value.into(),
        };
        let components = value
            .split('.')
            .map(|part| {
                if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(invalid());
                }
                part.parse().map_err(|_| invalid())
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(Self(components))
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, part) in self.0.iter().enumerate() {
            if index != 0 {
                f.write_str(".")?;
            }
            write!(f, "{part}")?;
        }
        Ok(())
    }
}

/// A Gregorian date in years 0001..9999; no timezone or system clock access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalendarDate {
    year: u16,
    month: u8,
    day: u8,
}

impl FromStr for CalendarDate {
    type Err = FirmwareMetadataError;

    fn from_str(value: &str) -> Result<Self> {
        let invalid = || FirmwareMetadataError::InvalidDate {
            value: value.into(),
        };
        let bytes = value.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || !bytes
                .iter()
                .enumerate()
                .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
        {
            return Err(invalid());
        }
        let year = value[..4].parse::<u16>().map_err(|_| invalid())?;
        let month = value[5..7].parse::<u8>().map_err(|_| invalid())?;
        let day = value[8..].parse::<u8>().map_err(|_| invalid())?;
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return Err(invalid()),
        };
        if year == 0 || day == 0 || day > days {
            return Err(invalid());
        }
        Ok(Self { year, month, day })
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseMetadata {
    pub version: FirmwareVersion,
    /// Opaque, entity-decoded Config installer location; not a firmware URL.
    pub config_download: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteConnectMetadata {
    pub minimum_version: FirmwareVersion,
    pub deadline: CalendarDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMetadata {
    pub release: Option<ReleaseMetadata>,
    pub remote_connect: Option<RemoteConnectMetadata>,
}

/// How the offered release compares with the caller's current version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseStatus {
    Unavailable,
    Newer,
    Equal,
    Older,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramCompatibility {
    SameMajorMinor,
    DifferentMajorMinor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadlineRelation {
    Before,
    On,
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteConnectObservation {
    pub below_minimum: bool,
    pub deadline: DeadlineRelation,
    /// Matches the upstream note predicate: below minimum AND on/before deadline.
    /// False after the deadline does not mean Remote Connect is usable.
    pub warning_due: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAssessment {
    pub release_status: ReleaseStatus,
    /// Only assessed for a newer offered release and a supplied program version.
    /// This observes a major/minor match, not actual Config/program compatibility.
    pub program_compatibility: Option<ProgramCompatibility>,
    pub remote_connect: Option<RemoteConnectObservation>,
}

#[derive(Debug)]
pub enum FirmwareMetadataError {
    Xml(quick_xml::Error),
    Attribute(AttrError),
    InvalidDocument(&'static str),
    DuplicateElement(&'static str),
    MissingAttribute {
        element: &'static str,
        attribute: &'static str,
    },
    InvalidVersion {
        value: String,
    },
    InvalidDate {
        value: String,
    },
}

impl fmt::Display for FirmwareMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(source) => write!(f, "invalid update XML: {source}"),
            Self::Attribute(source) => write!(f, "invalid update XML attribute: {source}"),
            Self::InvalidDocument(reason) => write!(f, "invalid update document: {reason}"),
            Self::DuplicateElement(element) => write!(f, "duplicate update element {element}"),
            Self::MissingAttribute { element, attribute } => {
                write!(f, "update element {element} has no {attribute}")
            }
            Self::InvalidVersion { value } => {
                write!(f, "invalid numeric firmware version {value:?}")
            }
            Self::InvalidDate { value } => write!(f, "invalid calendar date {value:?}"),
        }
    }
}

impl std::error::Error for FirmwareMetadataError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(source) => Some(source),
            Self::Attribute(source) => Some(source),
            _ => None,
        }
    }
}

type Result<T> = std::result::Result<T, FirmwareMetadataError>;

impl UpdateMetadata {
    /// Parse UTF-8 XML without I/O. Only direct children of `Miniserversoftware`
    /// named `Release` or `RemoteConnect` participate. Other elements are ignored.
    ///
    /// Known elements must be unique and have valid required attributes.
    /// Malformed attributes, mismatched/unclosed tags, extra roots and DTDs fail.
    /// This is not XSD, certificate, signature, checksum or download validation.
    pub fn parse(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        let mut depth = 0usize;
        let mut seen_root = false;
        let mut metadata = Self {
            release: None,
            remote_connect: None,
        };
        loop {
            match reader.read_event().map_err(FirmwareMetadataError::Xml)? {
                Event::Start(element) => {
                    metadata.read_element(&element, depth, &mut seen_root)?;
                    depth += 1;
                }
                Event::Empty(element) => {
                    metadata.read_element(&element, depth, &mut seen_root)?;
                }
                Event::End(_) => {
                    depth = depth
                        .checked_sub(1)
                        .ok_or(FirmwareMetadataError::InvalidDocument(
                            "unexpected closing element",
                        ))?;
                }
                Event::Text(text)
                    if depth == 0 && !text.as_ref().iter().all(u8::is_ascii_whitespace) =>
                {
                    return Err(FirmwareMetadataError::InvalidDocument("text outside root"));
                }
                Event::CData(_) if depth == 0 => {
                    return Err(FirmwareMetadataError::InvalidDocument("CDATA outside root"));
                }
                Event::DocType(_) => {
                    return Err(FirmwareMetadataError::InvalidDocument("DTD is unsupported"));
                }
                Event::Eof => {
                    if !seen_root || depth != 0 {
                        return Err(FirmwareMetadataError::InvalidDocument(
                            "missing or unclosed root",
                        ));
                    }
                    return Ok(metadata);
                }
                _ => {}
            }
        }
    }

    fn read_element(
        &mut self,
        element: &BytesStart<'_>,
        depth: usize,
        seen_root: &mut bool,
    ) -> Result<()> {
        // quick-xml accepts adjacent quoted attributes and literal '<' in values.
        let raw_attributes = element.attributes_raw();
        let mut quote = None;
        for (index, &byte) in raw_attributes.iter().enumerate() {
            if let Some(delimiter) = quote {
                if byte == b'<' {
                    return Err(FirmwareMetadataError::InvalidDocument(
                        "raw '<' in attribute value",
                    ));
                }
                if byte == delimiter {
                    if raw_attributes
                        .get(index + 1)
                        .is_some_and(|next| !matches!(next, b' ' | b'\t' | b'\r' | b'\n'))
                    {
                        return Err(FirmwareMetadataError::InvalidDocument(
                            "expected XML whitespace after attribute value",
                        ));
                    }
                    quote = None;
                }
            } else if matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
            }
        }
        for attr in element.attributes() {
            attr.map_err(FirmwareMetadataError::Attribute)?
                .unescape_value()
                .map_err(FirmwareMetadataError::Xml)?;
        }
        if depth == 0 {
            if *seen_root || element.name().as_ref() != b"Miniserversoftware" {
                return Err(FirmwareMetadataError::InvalidDocument(
                    "expected one Miniserversoftware root",
                ));
            }
            *seen_root = true;
            return Ok(());
        }
        if depth != 1 {
            return Ok(());
        }
        match element.name().as_ref() {
            b"Release" => {
                if self.release.is_some() {
                    return Err(FirmwareMetadataError::DuplicateElement("Release"));
                }
                self.release = Some(ReleaseMetadata {
                    version: required(element, "Release", "Version")?.parse()?,
                    config_download: attribute(element, "Path")?,
                });
            }
            b"RemoteConnect" => {
                if self.remote_connect.is_some() {
                    return Err(FirmwareMetadataError::DuplicateElement("RemoteConnect"));
                }
                self.remote_connect = Some(RemoteConnectMetadata {
                    minimum_version: required(element, "RemoteConnect", "MinVersion")?.parse()?,
                    deadline: required(element, "RemoteConnect", "Deadline")?.parse()?,
                });
            }
            _ => {}
        }
        Ok(())
    }

    /// Observe release ordering and upstream note predicates with explicit inputs.
    /// No install permission, device readiness or download authenticity is implied.
    pub fn assess(
        &self,
        current: &FirmwareVersion,
        program: Option<&FirmwareVersion>,
        as_of: CalendarDate,
    ) -> UpdateAssessment {
        let release_status = match self.release.as_ref().map(|r| r.version.cmp(current)) {
            None => ReleaseStatus::Unavailable,
            Some(Ordering::Greater) => ReleaseStatus::Newer,
            Some(Ordering::Equal) => ReleaseStatus::Equal,
            Some(Ordering::Less) => ReleaseStatus::Older,
        };
        let program_compatibility = self
            .release
            .as_ref()
            .filter(|_| release_status == ReleaseStatus::Newer)
            .zip(program)
            .map(|(release, program)| {
                if release.version.same_major_minor(program) {
                    ProgramCompatibility::SameMajorMinor
                } else {
                    ProgramCompatibility::DifferentMajorMinor
                }
            });
        let remote_connect = self.remote_connect.as_ref().map(|remote| {
            let below_minimum = current < &remote.minimum_version;
            let deadline = match as_of.cmp(&remote.deadline) {
                Ordering::Less => DeadlineRelation::Before,
                Ordering::Equal => DeadlineRelation::On,
                Ordering::Greater => DeadlineRelation::Past,
            };
            RemoteConnectObservation {
                below_minimum,
                deadline,
                warning_due: below_minimum && deadline != DeadlineRelation::Past,
            }
        });
        UpdateAssessment {
            release_status,
            program_compatibility,
            remote_connect,
        }
    }
}

fn attribute(element: &BytesStart<'_>, name: &str) -> Result<Option<String>> {
    for attr in element.attributes() {
        let attr = attr.map_err(FirmwareMetadataError::Attribute)?;
        if attr.key.as_ref() == name.as_bytes() {
            return attr
                .unescape_value()
                .map(|v| Some(v.into_owned()))
                .map_err(FirmwareMetadataError::Xml);
        }
    }
    Ok(None)
}

fn required(element: &BytesStart<'_>, tag: &'static str, name: &'static str) -> Result<String> {
    attribute(element, name)?.ok_or(FirmwareMetadataError::MissingAttribute {
        element: tag,
        attribute: name,
    })
}
