use super::{EditError, legacy::State};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fmt,
};
use xmltree::Element;

/// One direct State record in the selected Status Monitor.
#[derive(Debug, PartialEq, Eq)]
pub struct StatusMonitorState {
    /// One-based record ordinal, independent of the persistent `OIdx`.
    pub priority: usize,
    /// Stored native ID spelling; only validation compares it case-insensitively.
    pub id: String,
    pub output_index: u8,
    /// True only when the record has both an admitted `Val` and nonempty `Na`.
    pub configured: bool,
    /// Attributes as seen by the shared loader, without injecting defaults.
    pub attributes: BTreeMap<String, String>,
}

/// Owned standalone Status Monitor inspection. Missing block `U` or `Title` is empty.
#[derive(Debug, PartialEq, Eq)]
pub struct StatusMonitorInspection {
    pub uuid: String,
    pub title: String,
    pub states: Vec<StatusMonitorState>,
}

/// Failure to read the existing standalone Status Monitor shape.
#[derive(Debug)]
#[non_exhaustive]
pub enum StatusMonitorInspectionError {
    /// The original shared editor load or selection error.
    Edit(EditError),
    NotStatusMonitor {
        selector: String,
    },
    InheritedConfiguration,
    StateCount {
        found: usize,
    },
    NamespacedState,
    MissingStateId,
    InvalidStateId {
        id: String,
    },
    DuplicateStateId {
        id: String,
    },
    InvalidOutputIndex,
    DuplicateOutputIndex {
        output_index: u8,
    },
    RestNotLast,
    InvalidValue {
        raw: String,
    },
    RestHasValue,
    DuplicateValue {
        value: f64,
    },
}

impl fmt::Display for StatusMonitorInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::NotStatusMonitor { selector } => write!(
                f,
                "'{selector}' is not a StatusMonitor. Use `lox config controls FILE --type StatusMonitor` to find one; nothing was changed."
            ),
            Self::InheritedConfiguration => f.write_str(
                "Status Monitor priority editing supports standalone configurations only: an iObjS inheritance attribute is present in this file. Native inherited-owner redirection and propagation are not implemented; nothing was changed."
            ),
            Self::StateCount { found } => write!(
                f,
                "Unsupported Status Monitor shape: expected exactly eleven direct State records, including Rest; found {found}. Nothing was changed."
            ),
            Self::NamespacedState => f.write_str(
                "Namespaced Status Monitor State records are unsupported; nothing was changed."
            ),
            Self::MissingStateId => f.write_str(
                "Every Status Monitor State must have an explicit Id; nothing was changed."
            ),
            Self::InvalidStateId { id } => write!(
                f,
                "Unsupported Status Monitor State Id '{id}': expected a nonzero native UUID (8-4-4-16); nothing was changed."
            ),
            Self::DuplicateStateId { id } => {
                write!(f, "Duplicate Status Monitor State Id '{id}'; nothing was changed.")
            }
            Self::InvalidOutputIndex => f.write_str(
                "Every Status Monitor State must have an explicit decimal OIdx in 0..10; missing/defaulted associations are unsupported. Nothing was changed."
            ),
            Self::DuplicateOutputIndex { output_index } => {
                write!(f, "Duplicate Status Monitor OIdx {output_index}; nothing was changed.")
            }
            Self::RestNotLast => f.write_str(
                "The Status Monitor Rest state (OIdx=10) must be last; nothing was changed."
            ),
            Self::InvalidValue { raw } => write!(
                f,
                "Unsupported Status Monitor Val '{raw}': expected a finite decimal, not the native unset sentinel; nothing was changed."
            ),
            Self::RestHasValue => {
                f.write_str("The Status Monitor Rest state must omit Val; nothing was changed.")
            }
            Self::DuplicateValue { value } => write!(
                f,
                "Colliding Status Monitor Val '{value}'; resolve duplicate matching values before moving a state. Nothing was changed."
            ),
        }
    }
}

impl Error for StatusMonitorInspectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            _ => None,
        }
    }
}

/// Read one legacy-selected `C Type="StatusMonitor"` from standalone configuration bytes.
///
/// Loading and selection precede block eligibility and the document-wide `iObjS`
/// refusal. The existing inspection shape requires eleven direct, unnamespaced
/// State records, distinct non-nil native 8-4-4-16 IDs, and explicit distinct
/// decimal `OIdx` in 0..=10. Rest (`OIdx=10`) must be last and omit `Val`. Present
/// values must parse as distinct finite `f64`s other than the positive unset
/// sentinel (`f64::MAX`). No records or attributes are synthesized.
///
/// The legacy loader limitation (lox-n37w) is unchanged. This is not a native
/// Config validator and performs no editing, serialization or filesystem I/O.
pub fn inspect_status_monitor(
    bytes: &[u8],
    selector: &str,
) -> Result<StatusMonitorInspection, StatusMonitorInspectionError> {
    State::load(bytes)
        .map_err(StatusMonitorInspectionError::Edit)?
        .status_monitor_inspection_selected(selector)
}

#[doc(hidden)]
pub struct MatchedStatusMonitor {
    pub path: Vec<usize>,
    pub states: Vec<(usize, StatusMonitorState)>,
}

impl State {
    #[doc(hidden)]
    pub fn status_monitor_inspection_selected(
        &self,
        selector: &str,
    ) -> Result<StatusMonitorInspection, StatusMonitorInspectionError> {
        let MatchedStatusMonitor { path, states } = self.match_status_monitor(selector)?;
        let block = self.get_element(&path);
        Ok(StatusMonitorInspection {
            uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            title: block.attributes.get("Title").cloned().unwrap_or_default(),
            states: states.into_iter().map(|(_, state)| state).collect(),
        })
    }

    #[doc(hidden)]
    pub fn match_status_monitor(
        &self,
        selector: &str,
    ) -> Result<MatchedStatusMonitor, StatusMonitorInspectionError> {
        use StatusMonitorInspectionError as Failure;

        let path = self
            .require_one(selector)
            .map_err(|error| Failure::Edit(error.into()))?;
        let block = self.get_element(&path);
        if block.name != "C"
            || block.attributes.get("Type").map(String::as_str) != Some("StatusMonitor")
        {
            return Err(Failure::NotStatusMonitor {
                selector: selector.to_owned(),
            });
        }
        if has_inheritance(&self.root) {
            return Err(Failure::InheritedConfiguration);
        }
        let states = states(block)?;
        Ok(MatchedStatusMonitor { path, states })
    }
}

fn has_inheritance(element: &Element) -> bool {
    element.attributes.contains_key("iObjS")
        || element
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .any(has_inheritance)
}

#[doc(hidden)]
pub fn status_monitor_identity_key(raw: &str) -> Option<String> {
    if raw.len() != 35 {
        return None;
    }
    let mut key = String::with_capacity(32);
    for (index, byte) in raw.bytes().enumerate() {
        if [8, 13, 18].contains(&index) {
            if byte != b'-' {
                return None;
            }
        } else if byte.is_ascii_hexdigit() {
            key.push(char::from(byte.to_ascii_lowercase()));
        } else {
            return None;
        }
    }
    (!key.bytes().all(|byte| byte == b'0')).then_some(key)
}

fn states(
    block: &Element,
) -> Result<Vec<(usize, StatusMonitorState)>, StatusMonitorInspectionError> {
    use StatusMonitorInspectionError as Failure;

    let positions: Vec<_> = block
        .children
        .iter()
        .enumerate()
        .filter_map(|(position, child)| {
            child
                .as_element()
                .filter(|child| child.name == "State")
                .map(|child| (position, child))
        })
        .collect();
    if positions.len() != 11 {
        return Err(Failure::StateCount {
            found: positions.len(),
        });
    }
    let mut identities = HashSet::new();
    let mut outputs = HashSet::new();
    let mut values = Vec::new();
    let mut result = Vec::with_capacity(11);
    for (ordinal, (position, state)) in positions.into_iter().enumerate() {
        if state.namespace.is_some() || state.prefix.is_some() {
            return Err(Failure::NamespacedState);
        }
        let id = state.attributes.get("Id").ok_or(Failure::MissingStateId)?;
        let key = status_monitor_identity_key(id)
            .ok_or_else(|| Failure::InvalidStateId { id: id.clone() })?;
        if !identities.insert(key) {
            return Err(Failure::DuplicateStateId { id: id.clone() });
        }
        let output_index = state
            .attributes
            .get("OIdx")
            .filter(|raw| !raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|raw| raw.parse::<u8>().ok())
            .filter(|index| *index <= 10)
            .ok_or(Failure::InvalidOutputIndex)?;
        if !outputs.insert(output_index) {
            return Err(Failure::DuplicateOutputIndex { output_index });
        }
        if (ordinal == 10) != (output_index == 10) {
            return Err(Failure::RestNotLast);
        }
        let value = match state.attributes.get("Val") {
            None => None,
            Some(raw) => Some(
                raw.parse::<f64>()
                    .ok()
                    .filter(|value| value.is_finite() && *value != f64::MAX)
                    .ok_or_else(|| Failure::InvalidValue { raw: raw.clone() })?,
            ),
        };
        if output_index == 10 && value.is_some() {
            return Err(Failure::RestHasValue);
        }
        if let Some(value) = value {
            if values.contains(&value) {
                return Err(Failure::DuplicateValue { value });
            }
            values.push(value);
        }
        let configured = value.is_some()
            && state
                .attributes
                .get("Na")
                .is_some_and(|name| !name.is_empty());
        result.push((
            position,
            StatusMonitorState {
                priority: ordinal + 1,
                id: id.clone(),
                output_index,
                configured,
                attributes: state
                    .attributes
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            },
        ));
    }
    Ok(result)
}
