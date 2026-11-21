use super::super::archive::ArchiveError;
use std::{fmt, str::Utf8Error, string::FromUtf8Error};

/// Whether an XML reader failure occurred in the validated document pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlContext {
    Document,
    Bare,
}

/// Atomic runtime-overlay and three-way merge failures.
///
/// Identifiers and filenames retain their original diagnostic spelling. Native
/// XML/UTF-8/archive causes are available through [`std::error::Error::source`].
#[derive(Debug)]
#[non_exhaustive]
pub enum OverlayError {
    DocumentUtf8 {
        source: Utf8Error,
    },
    Utf8 {
        source: Utf8Error,
    },
    StringUtf8 {
        source: FromUtf8Error,
    },
    Xml {
        source: quick_xml::Error,
        context: XmlContext,
    },
    Attribute {
        source: quick_xml::events::attributes::AttrError,
        context: XmlContext,
    },
    Archive {
        source: ArchiveError,
    },
    FilenameSuffix,
    InvalidFilename {
        name: String,
    },
    DuplicateUuid {
        id: String,
    },
    MissingControlType,
    UnexpectedClosingTag,
    Dtd,
    OutsideRoot,
    IncompleteRoot,
    FragmentIntegrity,
    DuplicateOverlay {
        id: String,
    },
    ParentChildOverlap,
    FragmentRead {
        name: String,
        source: Box<OverlayError>,
    },
    NotControl,
    FragmentOverlap {
        name: String,
    },
    UnsupportedRoot {
        name: String,
    },
    InvalidControlRoot {
        name: String,
    },
    TypeChange {
        name: String,
    },
    RulePlacement {
        name: String,
    },
    UnknownControl {
        name: String,
        kind: String,
    },
    UuidConsistency {
        name: String,
        source: Box<OverlayError>,
    },
    ShellConflict {
        id: String,
    },
    ControlConflict {
        id: String,
    },
    Placement,
    MergedConsistency {
        source: Box<OverlayError>,
    },
}

impl fmt::Display for OverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocumentUtf8 { .. } => f.write_str("Config XML is not valid UTF-8"),
            Self::Utf8 { source } => source.fmt(f),
            Self::StringUtf8 { source } => source.fmt(f),
            Self::Xml { context: XmlContext::Document, .. } => f.write_str("Malformed config XML"),
            Self::Xml { source, .. } => source.fmt(f),
            Self::Attribute { context: XmlContext::Document, .. } => f.write_str("Malformed XML attribute"),
            Self::Attribute { source, .. } => source.fmt(f),
            Self::Archive { source } => source.fmt(f),
            Self::FilenameSuffix => f.write_str("Runtime overlay must end in .LoxCC"),
            Self::InvalidFilename { name } => write!(f, "Invalid runtime overlay filename {name}; expected a control UUID.LoxCC."),
            Self::DuplicateUuid { id } => write!(f, "Empty or duplicate UUID {id:?} in config XML."),
            Self::MissingControlType => f.write_str("Control has no Type"),
            Self::UnexpectedClosingTag => f.write_str("Unexpected XML closing tag"),
            Self::Dtd => f.write_str("DTD in config XML is not supported."),
            Self::OutsideRoot => f.write_str("Unexpected content outside config XML root."),
            Self::IncompleteRoot => f.write_str("Config XML must contain exactly one complete root element."),
            Self::FragmentIntegrity => f.write_str("Runtime overlay LoxCC length or CRC32 mismatch; refusing a partial/corrupt fragment."),
            Self::DuplicateOverlay { id } => write!(f, "Multiple runtime overlays identify UUID {id}."),
            Self::ParentChildOverlap => f.write_str("Overlapping parent/child runtime overlays have no evidenced precedence. Reconcile with Loxone Config."),
            Self::FragmentRead { name, .. } => write!(f, "Cannot read /sys/custom/{name}"),
            Self::NotControl => f.write_str("Runtime overlay is not a control"),
            Self::FragmentOverlap { name } => write!(f, "Runtime overlay {name} overlaps another control's fragment. Reconcile with Loxone Config."),
            Self::UnsupportedRoot { name } => write!(f, "Unsupported runtime overlay root in {name}."),
            Self::InvalidControlRoot { name } => write!(f, "Runtime overlay {name} must be a standalone C with the filename's UUID."),
            Self::TypeChange { name } => write!(f, "Runtime overlay {name} changes control Type; refusing ambiguous replacement."),
            Self::RulePlacement { name } => write!(f, "Cannot place new AutopilotRule {name}: expected exactly one Page with SpNo=\"1\". Use Loxone Config to reconcile."),
            Self::UnknownControl { name, kind } => write!(f, "Runtime overlay {name} has no matching control (Type {kind}). Only new AutopilotRule placement is evidenced; use Loxone Config."),
            Self::UuidConsistency { name, .. } => write!(f, "UUID consistency failed after {name}"),
            Self::ShellConflict { id } => write!(f, "Three-way merge conflict at UUID {id} (local and live both changed). Reconcile in Loxone Config; no deployment was performed."),
            Self::ControlConflict { id } => write!(f, "Three-way merge conflict for UUID {id}: concurrent addition, deletion, or modification. Reconcile in Loxone Config."),
            Self::Placement => f.write_str("Three-way merge cannot determine placement of a concurrently added control. Reconcile in Loxone Config."),
            Self::MergedConsistency { .. } => f.write_str("Merged config has inconsistent UUIDs"),
        }
    }
}

impl std::error::Error for OverlayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DocumentUtf8 { source } | Self::Utf8 { source } => Some(source),
            Self::StringUtf8 { source } => Some(source),
            Self::Xml { source, .. } => Some(source),
            Self::Attribute { source, .. } => Some(source),
            Self::Archive { source } => Some(source),
            Self::FragmentRead { source, .. }
            | Self::UuidConsistency { source, .. }
            | Self::MergedConsistency { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<quick_xml::Error> for OverlayError {
    fn from(source: quick_xml::Error) -> Self {
        Self::Xml {
            source,
            context: XmlContext::Bare,
        }
    }
}

impl From<quick_xml::events::attributes::AttrError> for OverlayError {
    fn from(source: quick_xml::events::attributes::AttrError) -> Self {
        Self::Attribute {
            source,
            context: XmlContext::Bare,
        }
    }
}

impl From<Utf8Error> for OverlayError {
    fn from(source: Utf8Error) -> Self {
        Self::Utf8 { source }
    }
}

impl From<FromUtf8Error> for OverlayError {
    fn from(source: FromUtf8Error) -> Self {
        Self::StringUtf8 { source }
    }
}
