use chrono::{DateTime, FixedOffset};
use std::{error::Error, fmt};

use super::{
    EditError, LoadManagerInspection, LoadManagerInspectionError, State, inspect_positions,
    load_positions, resolve_io_index,
};
use crate::configuration::editor::transaction_scope::{
    ScopeError, require_ordinary_owner, require_plain_xml,
};

mod xml;

#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerMaxOffChange {
    pub io_index: u8,
    /// Loader-decoded `maxOff`; omission is `None`, distinct from an explicit zero.
    pub before: Option<String>,
    /// Decimal seconds on a change; the original storage spelling on a no-op.
    pub after: String,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LoadManagerMaxOffOutcome {
    /// Original bytes on a no-op, otherwise the complete canonical XML.
    pub bytes: Vec<u8>,
    pub change: LoadManagerMaxOffChange,
    pub inspection: LoadManagerInspection,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadManagerMaxOffError {
    Edit(EditError),
    Inspection(LoadManagerInspectionError),
    InvalidRequestedIoIndex { io_index: u8 },
    InvalidSeconds,
    SignedSeconds,
    InvalidStoredSeconds { io_index: u8, raw: String },
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
    CanonicalReadmission(Box<LoadManagerMaxOffError>),
    CanonicalPreservation,
}

impl fmt::Display for LoadManagerMaxOffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Inspection(error) => error.fmt(f),
            Self::InvalidRequestedIoIndex { .. } => {
                f.write_str("Load Manager io-index must be 0..11, not a 1-based priority.")
            }
            Self::InvalidSeconds => f.write_str(
                "Load Manager maximum off time must be whole seconds in 10..2147483647, using decimal digits only. Zero's UI meaning is unqualified; native values 1..9 clamp to 10, so this conservative CLI subset rejects them instead. No rounding, clamping or unit conversion; nothing was changed.",
            ),
            Self::SignedSeconds => f.write_str(
                "Load Manager maximum off time requires decimal digits only, in whole seconds; nothing was changed.",
            ),
            Self::InvalidStoredSeconds { io_index, .. } => write!(
                f,
                "Unsupported Load Manager maxOff storage for io-index {io_index}: expected a nonnegative signed-32-bit integer in seconds (an optional leading plus is supported). Explicit empty, negative, fractional, exponent, whitespace and malformed storage are rejected; nothing was changed."
            ),
            Self::NamespacedXml => f.write_str(
                "Load Manager maximum off time editing does not support XML namespaces.",
            ),
            Self::UnsupportedXml => f.write_str(
                "Load Manager maximum off time editing requires one UTF-8 XML 1.0 root, no standalone declaration and no document type declaration.",
            ),
            Self::DocumentScope => f.write_str(
                "Load Manager maximum off time editing requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => f.write_str(
                "Load Manager maximum off time editing does not support template or inherited owner state.",
            ),
            Self::CanonicalReadmission(error) => {
                write!(f, "Canonical Load Manager output was refused: {error}")
            }
            Self::CanonicalPreservation => f.write_str(
                "Canonical XML would change data outside the selected Load Manager maximum off time and owning Document stamp; nothing was changed.",
            ),
        }
    }
}

impl Error for LoadManagerMaxOffError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Inspection(error) => Some(error),
            Self::CanonicalReadmission(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<ScopeError> for LoadManagerMaxOffError {
    fn from(error: ScopeError) -> Self {
        match error {
            ScopeError::Edit(error) => Self::Edit(error),
            ScopeError::NamespacedXml => Self::NamespacedXml,
            ScopeError::UnsupportedXml => Self::UnsupportedXml,
            ScopeError::DocumentScope => Self::DocumentScope,
            ScopeError::TemplateOrInherited => Self::TemplateOrInherited,
        }
    }
}

/// Edit only `maxOff` on one direct `Load` by its resolved connector `ioIdx`.
///
/// The request is ASCII decimal digits for whole seconds in 10..=2147483647.
/// Zero requests, rounding, clamping and unit conversion are unsupported.
/// Omitted storage is interpreted as zero; explicit storage must be a
/// nonnegative signed-32-bit decimal (an optional leading `+` is accepted).
/// Only the selected record's value is interpreted. Equal values return the
/// original bytes and spelling without stamping.
///
/// After request validation, admission checks XML, selection, ordinary owner
/// scope, the complete twelve-load identity shape, then selected storage.
/// Changed output is canonical, not a raw patch: it permits layout and
/// equivalent XML spelling changes but refuses unsupported payload changes
/// and must pass the same admission checks. `now` is used only to stamp a
/// changed owning Document. No filesystem access or runtime control occurs.
pub fn set_load_manager_load_max_off(
    bytes: &[u8],
    selector: &str,
    io_index: u8,
    max_off_seconds: &str,
    now: &DateTime<FixedOffset>,
) -> Result<LoadManagerMaxOffOutcome, LoadManagerMaxOffError> {
    use LoadManagerMaxOffError as Refusal;

    let seconds = validate_request(io_index, max_off_seconds)?;
    let (mut state, path, plan) = admit(bytes, selector, io_index, seconds)?;
    let (output, document_stamped) = if plan.change.changed {
        state.apply_load_manager_max_off(&path, &plan);
        let document_stamped = state.stamp_document(now);
        (state.to_bytes().map_err(Refusal::Edit)?, document_stamped)
    } else {
        (bytes.to_vec(), false)
    };
    let inspection = inspect_positions(state.get_element(&path), &plan.positions);
    if plan.change.changed {
        let (reread, reread_path, reread_plan) = admit(&output, selector, io_index, seconds)
            .map_err(|error| Refusal::CanonicalReadmission(Box::new(error)))?;
        if reread_path != path
            || reread_plan.change.changed
            || reread_plan.change.after != plan.change.after
            || inspect_positions(reread.get_element(&reread_path), &reread_plan.positions)
                != inspection
            || !xml::preserves_payload(bytes, &output, &state, &path, &plan, document_stamped)
        {
            return Err(Refusal::CanonicalPreservation);
        }
    }
    Ok(LoadManagerMaxOffOutcome {
        bytes: output,
        change: plan.change,
        inspection,
        document_stamped,
    })
}

fn admit(
    bytes: &[u8],
    selector: &str,
    io_index: u8,
    seconds: i32,
) -> Result<(State, Vec<usize>, MaxOffPlan), LoadManagerMaxOffError> {
    use LoadManagerMaxOffError as Refusal;

    if !xml::has_trusted_declaration(bytes) {
        return Err(Refusal::UnsupportedXml);
    }
    require_plain_xml(bytes)?;
    let state = State::load(bytes).map_err(Refusal::Edit)?;
    let path = state
        .load_manager_path(selector)
        .map_err(Refusal::Inspection)?;
    require_ordinary_owner(&state, &path)?;
    let plan = state.plan_load_manager_max_off(&path, io_index, seconds)?;
    Ok((state, path, plan))
}

fn validate_request(io_index: u8, raw: &str) -> Result<i32, LoadManagerMaxOffError> {
    use LoadManagerMaxOffError as Refusal;

    if io_index > 11 {
        return Err(Refusal::InvalidRequestedIoIndex { io_index });
    }
    let seconds = storage_integer(raw)
        .filter(|seconds| *seconds >= 10)
        .ok_or(Refusal::InvalidSeconds)?;
    if raw.starts_with('+') {
        return Err(Refusal::SignedSeconds);
    }
    Ok(seconds)
}

fn storage_integer(raw: &str) -> Option<i32> {
    let digits = raw.strip_prefix('+').unwrap_or(raw);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

struct MaxOffPlan {
    position: usize,
    positions: Vec<usize>,
    change: LoadManagerMaxOffChange,
}

impl State {
    fn plan_load_manager_max_off(
        &self,
        path: &[usize],
        io_index: u8,
        seconds: i32,
    ) -> Result<MaxOffPlan, LoadManagerMaxOffError> {
        let block = self.get_element(path);
        let positions = load_positions(block).map_err(LoadManagerMaxOffError::Inspection)?;
        // Twelve distinct resolved indices in 0..=11 always include the request.
        let position = positions
            .iter()
            .copied()
            .find(|&position| {
                resolve_io_index(block.children[position].as_element().unwrap()).unwrap()
                    == io_index
            })
            .unwrap();
        let load = block.children[position].as_element().unwrap();
        let before = load.attributes.get("maxOff").cloned();
        let previous = match &before {
            None => 0,
            Some(raw) => storage_integer(raw).ok_or_else(|| {
                LoadManagerMaxOffError::InvalidStoredSeconds {
                    io_index,
                    raw: raw.clone(),
                }
            })?,
        };
        let changed = previous != seconds;
        let after = if changed {
            seconds.to_string()
        } else {
            // A positive request cannot be a no-op against omission.
            before.as_ref().unwrap().clone()
        };
        Ok(MaxOffPlan {
            position,
            positions,
            change: LoadManagerMaxOffChange {
                io_index,
                before,
                after,
                changed,
            },
        })
    }

    fn apply_load_manager_max_off(&mut self, path: &[usize], plan: &MaxOffPlan) {
        if plan.change.changed {
            self.get_element_mut(path).children[plan.position]
                .as_mut_element()
                .unwrap()
                .attributes
                .insert("maxOff".into(), plan.change.after.clone());
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_load_manager_load_max_off(
        &mut self,
        selector: &str,
        io_index: u8,
        max_off_seconds: &str,
    ) -> Result<bool, LoadManagerMaxOffError> {
        let seconds = validate_request(io_index, max_off_seconds)?;
        let path = self
            .load_manager_path(selector)
            .map_err(LoadManagerMaxOffError::Inspection)?;
        let plan = self.plan_load_manager_max_off(&path, io_index, seconds)?;
        self.apply_load_manager_max_off(&path, &plan);
        Ok(plan.change.changed)
    }
}
