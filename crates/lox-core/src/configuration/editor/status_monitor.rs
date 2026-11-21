use super::{EditError, legacy::State};
use chrono::{DateTime, FixedOffset};
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

/// Direction in the stored order of existing Status Monitor states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusMonitorPriorityDirection {
    Up,
    Down,
}

/// Movement of one stored state, including success at a movable boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMonitorPriorityMove {
    /// Persistent `Id` in its stored spelling.
    pub id: String,
    /// One-based priority before the operation.
    pub from_priority: usize,
    /// One-based priority after the operation.
    pub to_priority: usize,
    pub changed: bool,
}

/// Successful offline byte transaction.
#[derive(Debug)]
pub struct StatusMonitorPriorityMoveOutcome {
    /// Exact input on a boundary no-op; shared writer output on a real move.
    pub bytes: Vec<u8>,
    pub movement: StatusMonitorPriorityMove,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum StatusMonitorPriorityMoveError {
    Edit(EditError),
    Inspection(StatusMonitorInspectionError),
    InvalidStateId,
    StateNotFound { id: String },
    RestState,
    UnconfiguredState,
    UnsupportedDocument,
    NamespacedContext,
    TemplateConfiguration,
    InvalidContextFlags { value: String },
}

impl fmt::Display for StatusMonitorPriorityMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::Inspection(error) => fmt::Display::fmt(error, f),
            Self::InvalidStateId => f.write_str(
                "Status Monitor state Id must be a nonzero native UUID (8-4-4-16 hexadecimal digits). Use `config status-monitor inspect` to find it; nothing was changed.",
            ),
            Self::StateNotFound { id } => write!(
                f,
                "No Status Monitor state with Id '{id}'. Select an existing Id, not OIdx, name or priority; nothing was changed."
            ),
            Self::RestState => f.write_str(
                "The Status Monitor Rest state (OIdx=10) cannot move; nothing was changed.",
            ),
            Self::UnconfiguredState => f.write_str(
                "Select a named Status Monitor state with an explicit finite Val; nothing was changed.",
            ),
            Self::UnsupportedDocument => f.write_str(
                "Status Monitor priority moves require one owning Document at the root or directly under it; nothing was changed.",
            ),
            Self::NamespacedContext => f.write_str(
                "Namespaced Status Monitor owners or ancestors are unsupported; nothing was changed.",
            ),
            Self::TemplateConfiguration => f.write_str(
                "Status Monitor priority moves do not support Document Template or template-owned state; nothing was changed.",
            ),
            Self::InvalidContextFlags { value } => write!(
                f,
                "Unsupported Status Monitor context WF '{value}'; nothing was changed."
            ),
        }
    }
}

impl Error for StatusMonitorPriorityMoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Inspection(error) => Some(error),
            _ => None,
        }
    }
}

/// Move one configured state by one position in a standalone Status Monitor.
///
/// The legacy selector and stored shape are those of [`inspect_status_monitor`].
/// `id` is a non-nil native 8-4-4-16 State ID, matched case-insensitively; the
/// loader-observed ID, `OIdx`, `Val`, and rest of each record travel together,
/// without changing connector identities or references. Rest cannot be selected.
///
/// The byte transaction additionally requires one plain, stampable owning
/// Document (root or direct child). Direct Document `Template` and template
/// elements, `IfT`, `uuidParent`, and `WF` bit 29 on the owner or its ancestors
/// are out of scope, even for no-ops. Present context `WF` must parse as a
/// nonnegative signed 32-bit decimal.
/// A boundary no-op returns the original bytes. A real swap gets the shared
/// monotonic Document stamp using `now`, then the shared canonical serializer
/// (including its BOM, line-ending and lox-n37w behavior). Errors return no
/// revised bytes. This API performs no filesystem I/O or native validation.
pub fn move_status_monitor_state(
    bytes: &[u8],
    selector: &str,
    id: &str,
    direction: StatusMonitorPriorityDirection,
    now: &DateTime<FixedOffset>,
) -> Result<StatusMonitorPriorityMoveOutcome, StatusMonitorPriorityMoveError> {
    use StatusMonitorPriorityMoveError as Failure;

    let mut state = State::load(bytes).map_err(Failure::Edit)?;
    let plan = state.plan_status_monitor_move(selector, id, direction)?;
    state.require_status_monitor_move_context(&plan.path)?;
    let movement = state.apply_status_monitor_move(plan);
    let bytes = if movement.changed {
        if !state.stamp_document(now) {
            return Err(Failure::UnsupportedDocument);
        }
        state.to_bytes().map_err(Failure::Edit)?
    } else {
        bytes.to_vec()
    };
    Ok(StatusMonitorPriorityMoveOutcome { bytes, movement })
}

struct MatchedStatusMonitor {
    path: Vec<usize>,
    states: Vec<(usize, StatusMonitorState)>,
}

struct StatusMonitorMovePlan {
    path: Vec<usize>,
    from: usize,
    to: usize,
    movement: StatusMonitorPriorityMove,
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

    fn match_status_monitor(
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

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    // The binary keeps its historical admission and separate file writer.
    pub fn move_status_monitor_state_legacy(
        &mut self,
        selector: &str,
        id: &str,
        direction: StatusMonitorPriorityDirection,
    ) -> Result<StatusMonitorPriorityMove, StatusMonitorPriorityMoveError> {
        let plan = self.plan_status_monitor_move(selector, id, direction)?;
        Ok(self.apply_status_monitor_move(plan))
    }

    fn plan_status_monitor_move(
        &self,
        selector: &str,
        id: &str,
        direction: StatusMonitorPriorityDirection,
    ) -> Result<StatusMonitorMovePlan, StatusMonitorPriorityMoveError> {
        use StatusMonitorPriorityMoveError as Failure;

        let key = status_monitor_identity_key(id).ok_or(Failure::InvalidStateId)?;
        let MatchedStatusMonitor { path, states } = self
            .match_status_monitor(selector)
            .map_err(Failure::Inspection)?;
        let index = states
            .iter()
            .position(|(_, state)| status_monitor_identity_key(&state.id).as_ref() == Some(&key))
            .ok_or_else(|| Failure::StateNotFound { id: id.into() })?;
        let (from, selected) = &states[index];
        if selected.output_index == 10 {
            return Err(Failure::RestState);
        }
        if !selected.configured {
            return Err(Failure::UnconfiguredState);
        }
        // Priority is the State ordinal, not OIdx. Rest never participates.
        let destination = match direction {
            StatusMonitorPriorityDirection::Up => index.saturating_sub(1),
            StatusMonitorPriorityDirection::Down => (index + 1).min(9),
        };
        Ok(StatusMonitorMovePlan {
            path,
            from: *from,
            to: states[destination].0,
            movement: StatusMonitorPriorityMove {
                id: selected.id.clone(),
                from_priority: index + 1,
                to_priority: destination + 1,
                changed: index != destination,
            },
        })
    }

    fn apply_status_monitor_move(&mut self, plan: StatusMonitorMovePlan) -> StatusMonitorPriorityMove {
        if plan.movement.changed {
            self.get_element_mut(&plan.path)
                .children
                .swap(plan.from, plan.to);
        }
        plan.movement
    }

    fn require_status_monitor_move_context(
        &self,
        owner: &[usize],
    ) -> Result<(), StatusMonitorPriorityMoveError> {
        use StatusMonitorPriorityMoveError as Failure;

        let documents = self.find_elements("Type:Document");
        let [document] = documents.as_slice() else {
            return Err(Failure::UnsupportedDocument);
        };
        if document.len() > 1 || !owner.starts_with(document) {
            return Err(Failure::UnsupportedDocument);
        }
        let document = self.get_element(document);
        if document.attributes.get("Type").map(String::as_str) != Some("Document") {
            return Err(Failure::UnsupportedDocument);
        }
        // A direct Template configures the Document even when it is a sibling
        // of the owner's branch, and regardless of its IfT value.
        if document
            .children
            .iter()
            .filter_map(|node| node.as_element())
            .any(is_template_element)
        {
            return Err(Failure::TemplateConfiguration);
        }
        for depth in 0..=owner.len() {
            let element = self.get_element(&owner[..depth]);
            if element.namespace.is_some() || element.prefix.is_some() {
                return Err(Failure::NamespacedContext);
            }
            if is_template_element(element)
                || element.attributes.contains_key("IfT")
                || element.attributes.contains_key("uuidParent")
            {
                return Err(Failure::TemplateConfiguration);
            }
            if let Some(raw) = element.attributes.get("WF") {
                let flags = raw
                    .parse::<u32>()
                    .ok()
                    .filter(|value| i32::try_from(*value).is_ok())
                    .ok_or_else(|| Failure::InvalidContextFlags { value: raw.clone() })?;
                if flags & 0x2000_0000 != 0 {
                    return Err(Failure::TemplateConfiguration);
                }
            }
        }
        Ok(())
    }
}

fn is_template_element(element: &Element) -> bool {
    element.name == "Template"
        || element.attributes.get("Type").map(String::as_str) == Some("DocumentTemplate")
}

fn has_inheritance(element: &Element) -> bool {
    element.attributes.contains_key("iObjS")
        || element
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .any(has_inheritance)
}

fn status_monitor_identity_key(raw: &str) -> Option<String> {
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
