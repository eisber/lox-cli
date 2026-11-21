//! Legacy-compatible, headless configuration reads and editor transactions.
//!
//! This is not a native Config validator or a lossless arbitrary-XML editor.
//! Loading and canonical serialization intentionally retain the CLI's existing
//! digit-name/quoted-text limitation (lox-n37w), duplicate handling and formatting.
//! No file, clock, home directory, terminal or process state is accessed.

mod attribute;
mod central_light_links;
mod control_move;
mod daytimer;
mod description;
pub(super) mod legacy;
mod moods;
#[cfg(all(feature = "configuration-xml", feature = "unstable-legacy-cli-editor"))]
#[doc(hidden)]
pub mod operating_modes;
mod permissions;
mod program;
mod selector;
mod unwire;
#[cfg(any(test, feature = "unstable-legacy-cli-editor"))]
mod virtual_input;
mod write;

use chrono::{DateTime, FixedOffset};
use std::{error::Error, fmt};

pub use attribute::{AttributeChange, SetAttributeOutcome, set_attribute};
pub use central_light_links::{
    CentralLightLink, CentralLightLinks, CentralLightLinksError, inspect_central_light_links,
};
#[cfg(feature = "unstable-legacy-cli-editor")]
#[doc(hidden)]
pub use control_move::ControlMovePlan;
pub use control_move::{ControlMoveChange, ControlMoveOutcome, ControlMoveSelector, move_control};
pub use daytimer::{
    DayTimerEntryRemoval, DayTimerRemovalError, DayTimerRemovalOutcome, remove_daytimer_entry,
};
#[cfg(feature = "unstable-legacy-cli-editor")]
#[doc(hidden)]
pub use daytimer::{SelectedDayTimerEntry, daytimer_integer, daytimer_mode_uuid_key};
pub use description::{
    ChildDescription, ConnectorInfo, ElementDescription, PropertyValue, describe_element,
};
pub use moods::{MoodInfo, MoodInventoryError, list_moods};
pub use permissions::{
    DirectPermission, PermissionPrincipal, PrincipalKind, PrincipalPermissions,
    PrincipalPermissionsError, PrincipalSelectionError, principal_permissions,
};
pub use program::{ProgramTextError, get_program_text};
pub use unwire::{UnwireChange, UnwireError, UnwireOutcome, unwire};

/// Existing graph primitives used by the legacy binary's link removal code.
#[cfg(feature = "unstable-legacy-cli-editor")]
#[doc(hidden)]
pub mod legacy_central_light_links {
    pub use super::central_light_links::{LINKED, collect_controls, flags, identity, links};
}

/// Reversible bridge for the binary's unmigrated editor impl modules.
///
/// UNSTABLE: not a supported document model, not enabled by default, and never
/// exported through foreign bindings. The normal API exposes no mutable tree.
#[cfg(feature = "unstable-legacy-cli-editor")]
#[doc(hidden)]
pub use legacy::State as LegacyState;
#[cfg(feature = "unstable-legacy-cli-editor")]
#[doc(hidden)]
pub use virtual_input::{InputRefIds, LegacyVirtualInputCreation, VirtualInIds, VirtualInOpts};

/// Value-only identity, independent of the editor's storage representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementIdentity {
    pub title: String,
    pub block_type: String,
    pub uuid: String,
}

impl ElementIdentity {
    fn from_element(element: &xmltree::Element) -> Self {
        Self {
            title: element.attributes.get("Title").cloned().unwrap_or_default(),
            block_type: element.attributes.get("Type").cloned().unwrap_or_default(),
            uuid: element.attributes.get("U").cloned().unwrap_or_default(),
        }
    }
}

#[derive(Debug)]
pub enum SelectionError {
    NotFound {
        selector: String,
        titles: Vec<String>,
    },
    Ambiguous {
        selector: String,
        matches: Vec<ElementIdentity>,
    },
}

impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { selector, .. } => write!(f, "No element matches {selector:?}"),
            Self::Ambiguous { selector, matches } => {
                write!(f, "{} elements match {selector:?}", matches.len())
            }
        }
    }
}

impl Error for SelectionError {}

/// Retains the original parser/emitter cause without making its type public API.
#[derive(Debug)]
pub struct EditCause(Box<dyn Error + Send + Sync>);

impl EditCause {
    pub fn into_inner(self) -> Box<dyn Error + Send + Sync> {
        self.0
    }
}

impl fmt::Display for EditCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for EditCause {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[derive(Debug)]
pub enum EditError {
    Load(EditCause),
    Serialize(EditCause),
    InvalidUtf8(EditCause),
    Selection(SelectionError),
    ParameterMissing {
        element: ElementIdentity,
        parameter: String,
    },
    UnsupportedLegacyParameter {
        parameter: String,
    },
    ControlMoveSelectorMissing,
    RoomNotFound {
        room: String,
    },
    RoomAmbiguous {
        room: String,
        matches: Vec<String>,
    },
}

impl EditError {
    fn load(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Load(EditCause(Box::new(error)))
    }

    fn serialize(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Serialize(EditCause(Box::new(error)))
    }

    fn invalid_utf8(error: std::string::FromUtf8Error) -> Self {
        Self::InvalidUtf8(EditCause(Box::new(error)))
    }
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(_) => f.write_str("Configuration load failed"),
            Self::Serialize(_) => f.write_str("Configuration serialization failed"),
            Self::InvalidUtf8(_) => f.write_str("Configuration serialization is not UTF-8"),
            Self::Selection(error) => error.fmt(f),
            Self::ParameterMissing { parameter, .. } => {
                write!(f, "Connector {parameter:?} is absent")
            }
            Self::UnsupportedLegacyParameter { parameter } => {
                write!(f, "Unsupported legacy parameter {parameter:?}")
            }
            Self::ControlMoveSelectorMissing => {
                f.write_str("Specify --type or --title to select controls to move")
            }
            Self::RoomNotFound { room } => write!(f, "Room '{room}' not found in config"),
            Self::RoomAmbiguous { room, matches } => {
                write!(f, "Multiple rooms match '{room}': {matches:?}")
            }
        }
    }
}

impl Error for EditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load(cause) | Self::Serialize(cause) | Self::InvalidUtf8(cause) => {
                Some(cause.0.as_ref())
            }
            Self::Selection(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SelectionError> for EditError {
    fn from(value: SelectionError) -> Self {
        Self::Selection(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterChange {
    pub element: ElementIdentity,
    pub parameter: String,
    /// `None` is distinct from an explicitly empty `Def`.
    pub old_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug)]
pub struct SetParamOutcome {
    pub bytes: Vec<u8>,
    pub change: ParameterChange,
    /// Compared against an unedited *canonical* serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Edit the first matching `Co K=parameter` on one legacy-selected block.
///
/// Mirrors the existing CLI's connector-only eligibility, including input/output
/// connectors (warnings remain the head's policy). It does not create connectors
/// or use the legacy helper's attribute whitelist. Selection de-duplicates UUID
/// mirrors and supports the existing title/type/UUID/gid/room syntax.
///
/// The explicit instant supplies local wall time for `Date` and UTC epoch seconds
/// for `DateS`; changed documents advance monotonically. A canonical no-op does
/// not stamp. Normalization may still change raw bytes on a no-op. On error no
/// revised bytes are returned and the borrowed input cannot be mutated.
pub fn set_param(
    bytes: &[u8],
    selector: &str,
    parameter: &str,
    value: &str,
    now: &DateTime<FixedOffset>,
) -> Result<SetParamOutcome, EditError> {
    let mut state = legacy::State::load(bytes)?;
    // Legacy selection/mutation refusal precedes serialization.
    let change = state.edit_parameter(selector, parameter, value, false)?;
    let mut output = state.to_bytes()?;
    let baseline = legacy::State::load(bytes)?.to_bytes()?;
    let content_changed = baseline != output;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    Ok(SetParamOutcome {
        bytes: output,
        change,
        content_changed,
        document_stamped,
    })
}

impl legacy::State {
    fn edit_parameter(
        &mut self,
        selector: &str,
        parameter: &str,
        value: &str,
        allow_legacy_attribute: bool,
    ) -> Result<ParameterChange, EditError> {
        let path = self.require_one(selector)?;
        let element = self.get_element_mut(&path);
        let identity = ElementIdentity::from_element(element);
        for child in &mut element.children {
            if let Some(connector) = child.as_mut_element()
                && connector.name == "Co"
                && connector.attributes.get("K").map(String::as_str) == Some(parameter)
            {
                let old_value = connector
                    .attributes
                    .insert("Def".to_owned(), value.to_owned());
                return Ok(ParameterChange {
                    element: identity,
                    parameter: parameter.to_owned(),
                    old_value,
                    new_value: value.to_owned(),
                });
            }
        }
        if allow_legacy_attribute {
            if ["MaxVal", "MinVal", "MinChange", "MinTime", "Step"].contains(&parameter) {
                let old_value = element
                    .attributes
                    .insert(parameter.to_owned(), value.to_owned());
                return Ok(ParameterChange {
                    element: identity,
                    parameter: parameter.to_owned(),
                    old_value,
                    new_value: value.to_owned(),
                });
            }
            return Err(EditError::UnsupportedLegacyParameter {
                parameter: parameter.to_owned(),
            });
        }
        Err(EditError::ParameterMissing {
            element: identity,
            parameter: parameter.to_owned(),
        })
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub fn set_param_legacy(
        &mut self,
        selector: &str,
        parameter: &str,
        value: &str,
    ) -> Result<(), EditError> {
        self.edit_parameter(selector, parameter, value, true)
            .map(|_| ())
    }
}
