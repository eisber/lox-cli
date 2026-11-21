use chrono::{DateTime, FixedOffset};
use quick_xml::{Reader, events::Event};
use std::{error::Error, fmt};

use super::{
    EditError, LoadManagerInspection, LoadManagerInspectionError, State, inspect_positions,
    load_positions, selected_load_position,
};
use crate::configuration::editor::transaction_scope::{
    ScopeError, require_ordinary_owner, require_plain_xml, require_xml_1_0_declaration,
};

#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerPowerChange {
    pub io_index: u8,
    /// Loader-decoded `pwr` spelling before the edit, in kW.
    pub before: String,
    /// Canonical decimal kW on a change; the original spelling on a no-op.
    pub after: String,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LoadManagerPowerOutcome {
    /// Original bytes on a no-op, otherwise the complete canonical XML.
    pub bytes: Vec<u8>,
    pub change: LoadManagerPowerChange,
    pub inspection: LoadManagerInspection,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadManagerPowerError {
    Edit(EditError),
    Inspection(LoadManagerInspectionError),
    InvalidRequestedIoIndex { io_index: u8 },
    InvalidPower,
    NonFinitePower,
    PowerPrecisionLoss,
    UnsupportedStoredPower { io_index: u8 },
    StoredPowerAliasesRequest { io_index: u8 },
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
    CanonicalStorageLoss,
}

impl fmt::Display for LoadManagerPowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Inspection(error) => error.fmt(f),
            Self::InvalidRequestedIoIndex { .. } => {
                f.write_str("Load Manager io-index must be 0..11, not a 1-based priority.")
            }
            Self::InvalidPower => f.write_str(
                "Load Manager power must be a nonnegative kW decimal with at most 3 fractional digits (for example 2.5 or 0.001), without signs, exponent, units or separators. This conservative CLI subset never clamps or rounds.",
            ),
            Self::NonFinitePower => {
                f.write_str("Load Manager power must be finite; nothing was changed.")
            }
            Self::PowerPrecisionLoss => f.write_str(
                "Load Manager power loses precision in native double/3-decimal storage; choose an exactly round-tripping value. Nothing was changed.",
            ),
            Self::UnsupportedStoredPower { io_index } => write!(
                f,
                "Unsupported Load Manager power storage for io-index {io_index}: expected an explicit finite nonnegative pwr decimal (only zero exponents are supported); missing or malformed storage is not defaulted. Nothing was changed."
            ),
            Self::StoredPowerAliasesRequest { io_index } => write!(
                f,
                "Stored Load Manager power for io-index {io_index} shares the requested double but is a different exact decimal; nothing was changed."
            ),
            Self::NamespacedXml => {
                f.write_str("Load Manager power editing does not support XML namespaces.")
            }
            Self::UnsupportedXml => f.write_str(
                "Load Manager power editing requires one UTF-8 XML 1.0 root and no document type declaration.",
            ),
            Self::DocumentScope => f.write_str(
                "Load Manager power editing requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => f.write_str(
                "Load Manager power editing does not support template or inherited owner state.",
            ),
            Self::CanonicalStorageLoss => f.write_str(
                "Load Manager power editing cannot preserve this input's data through canonical XML; nothing was changed.",
            ),
        }
    }
}

impl Error for LoadManagerPowerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Inspection(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ScopeError> for LoadManagerPowerError {
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

/// Edit the stored `pwr` of one direct `Load` by its resolved connector `ioIdx`.
///
/// The request is an unsigned dot decimal in kW, with zero to three fractional
/// digits (a 0.001 kW decimal grid, not a claimed native UI step). Its exact
/// decimal must survive binary64 conversion and fixed three-place formatting;
/// binary half-milli ties are refused. No unit conversion is performed.
///
/// The request is checked before XML loading and selection. One ordinary
/// `LoadShed` and twelve direct loads with unique resolved indices in 0..=11
/// are required. Omitted `ioIdx` means zero. The selected `pwr` must already be
/// an explicit finite nonnegative decimal; signed zero and decimal mantissas
/// with zero exponents are supported in storage. Exact decimal equality
/// preserves the original bytes and stamp. Different stored decimals that
/// would compare equal only through binary64 rounding are refused.
///
/// Namespaces, non-UTF-8 or non-1.0 declarations, DTDs, multiple roots and
/// unsupported Document or owner scope are refused even on a no-op. On a
/// change, `now` supplies the save stamp when an owning Document exists.
/// Changed bytes must reopen with the same edited tree. Canonical formatting
/// is allowed, but digit-attribute rewriting of data, outside-root markup,
/// standalone declarations and potentially significant discarded whitespace
/// are refused rather than silently losing unrelated data. Genuine Loxone
/// digit-prefixed attributes remain eligible when only their keys are rewritten.
/// This API has no filesystem or runtime effects and returns no bytes on error.
pub fn set_load_manager_load_power(
    bytes: &[u8],
    selector: &str,
    io_index: u8,
    power_kw: &str,
    now: &DateTime<FixedOffset>,
) -> Result<LoadManagerPowerOutcome, LoadManagerPowerError> {
    use LoadManagerPowerError as Refusal;

    let request = power_request(io_index, power_kw)?;
    require_xml_1_0_declaration(bytes)?;
    let mut state = State::load(bytes).map_err(Refusal::Edit)?;
    require_plain_xml(bytes)?;
    let path = state
        .load_manager_path(selector)
        .map_err(Refusal::Inspection)?;
    require_ordinary_owner(&state, &path)?;
    let plan = state.plan_load_manager_power(&path, io_index, &request)?;
    plan.require_exact_storage(&request)?;
    require_preserved_xml(bytes, &state, plan.change.changed)?;

    let (bytes, document_stamped) = if plan.change.changed {
        state.apply_load_manager_power(&path, &plan);
        let document_stamped = state.stamp_document(now);
        let output = state.to_bytes().map_err(Refusal::Edit)?;
        require_plain_xml(&output).map_err(|_| Refusal::CanonicalStorageLoss)?;
        let reread = State::load(&output).map_err(|_| Refusal::CanonicalStorageLoss)?;
        require_preserved_xml(&output, &reread, true)?;
        if reread.root != state.root
            || reread.had_bom != state.had_bom
            || reread.had_crlf != state.had_crlf
        {
            return Err(Refusal::CanonicalStorageLoss);
        }
        (output, document_stamped)
    } else {
        (bytes.to_vec(), false)
    };
    let inspection = inspect_positions(state.get_element(&path), &plan.positions);
    Ok(LoadManagerPowerOutcome {
        bytes,
        change: plan.change,
        inspection,
        document_stamped,
    })
}

struct PowerRequest {
    double: f64,
    canonical: String,
}

fn power_request(io_index: u8, raw: &str) -> Result<PowerRequest, LoadManagerPowerError> {
    use LoadManagerPowerError as Refusal;

    if io_index > 11 {
        return Err(Refusal::InvalidRequestedIoIndex { io_index });
    }
    let (whole, fraction) = decimal_parts(raw)
        .filter(|&(_, fraction)| fraction.len() <= 3)
        .ok_or(Refusal::InvalidPower)?;
    let double = raw
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or(Refusal::NonFinitePower)?;
    let (whole, fraction) = normalize_decimal(whole, fraction);
    let canonical = if fraction.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}.{fraction}")
    };
    // Keep the exact decimal as the value written. Binary64 only witnesses
    // the existing three-decimal round-trip rule; odd sixteenths are CRT ties.
    let fixed = format!("{double:.3}");
    let stored = fixed.trim_end_matches('0').trim_end_matches('.');
    if stored != canonical || (double.fract() * 16.0) % 2.0 == 1.0 {
        return Err(Refusal::PowerPrecisionLoss);
    }
    Ok(PowerRequest { double, canonical })
}

fn decimal_parts(raw: &str) -> Option<(&str, &str)> {
    let (whole, fraction) = match raw.split_once('.') {
        Some((_, "")) => return None,
        Some(parts) => parts,
        None => (raw, ""),
    };
    (!whole.is_empty()
        && whole.bytes().all(|byte| byte.is_ascii_digit())
        && fraction.bytes().all(|byte| byte.is_ascii_digit()))
    .then_some((whole, fraction))
}

fn normalize_decimal<'a>(whole: &'a str, fraction: &'a str) -> (&'a str, &'a str) {
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    (whole, fraction.trim_end_matches('0'))
}

fn stored_power_parts(raw: &str) -> Option<(&str, &str)> {
    let mantissa = if let Some((mantissa, exponent)) = raw.split_once(['e', 'E']) {
        let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        if !mantissa.contains('.')
            || exponent.is_empty()
            || !exponent.bytes().all(|byte| byte == b'0')
        {
            return None;
        }
        mantissa
    } else {
        raw
    };
    decimal_parts(mantissa.strip_prefix(['+', '-']).unwrap_or(mantissa))
}

struct PowerPlan {
    position: usize,
    positions: Vec<usize>,
    previous_double: f64,
    change: LoadManagerPowerChange,
}

impl PowerPlan {
    fn require_exact_storage(&self, request: &PowerRequest) -> Result<(), LoadManagerPowerError> {
        use LoadManagerPowerError as Refusal;

        let io_index = self.change.io_index;
        let raw = &self.change.before;
        let (whole, fraction) = stored_power_parts(raw).unwrap();
        let (whole, fraction) = normalize_decimal(whole, fraction);
        if raw.starts_with('-') && (whole != "0" || !fraction.is_empty()) {
            return Err(Refusal::UnsupportedStoredPower { io_index });
        }
        // Decimal magnitudes just above DBL_MAX can round down to DBL_MAX.
        if self.previous_double == f64::MAX {
            let max = format!("{:.0}", f64::MAX);
            if whole.len() > max.len()
                || whole.len() == max.len()
                    && (whole > max.as_str() || whole == max.as_str() && !fraction.is_empty())
            {
                return Err(Refusal::UnsupportedStoredPower { io_index });
            }
        }
        // The CLI intentionally retains legacy binary64 equality. A strict byte
        // transaction must not decide "no change" on an exact decimal collision.
        if !self.change.changed && (whole, fraction) != decimal_parts(&request.canonical).unwrap() {
            return Err(Refusal::StoredPowerAliasesRequest { io_index });
        }
        Ok(())
    }
}

impl State {
    fn plan_load_manager_power(
        &self,
        path: &[usize],
        io_index: u8,
        request: &PowerRequest,
    ) -> Result<PowerPlan, LoadManagerPowerError> {
        use LoadManagerPowerError as Refusal;

        let block = self.get_element(path);
        let positions = load_positions(block).map_err(Refusal::Inspection)?;
        let position = selected_load_position(block, &positions, io_index);
        let attributes = &block.children[position].as_element().unwrap().attributes;
        let (before, previous_double) = attributes
            .get("pwr")
            .filter(|raw| stored_power_parts(raw).is_some())
            .and_then(|raw| raw.parse::<f64>().ok().map(|value| (raw, value)))
            .filter(|(_, value)| value.is_finite() && *value >= 0.0)
            .ok_or(Refusal::UnsupportedStoredPower { io_index })?;
        let changed = previous_double != request.double;
        Ok(PowerPlan {
            position,
            positions,
            previous_double,
            change: LoadManagerPowerChange {
                io_index,
                before: before.clone(),
                after: (if changed { &request.canonical } else { before }).clone(),
                changed,
            },
        })
    }

    fn apply_load_manager_power(&mut self, path: &[usize], plan: &PowerPlan) {
        if plan.change.changed {
            self.get_element_mut(path).children[plan.position]
                .as_mut_element()
                .unwrap()
                .attributes
                .insert("pwr".into(), plan.change.after.clone());
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_load_manager_load_power(
        &mut self,
        selector: &str,
        io_index: u8,
        power_kw: &str,
    ) -> Result<bool, LoadManagerPowerError> {
        let request = power_request(io_index, power_kw)?;
        let path = self
            .load_manager_path(selector)
            .map_err(LoadManagerPowerError::Inspection)?;
        let plan = self.plan_load_manager_power(&path, io_index, &request)?;
        self.apply_load_manager_power(&path, &plan);
        Ok(plan.change.changed)
    }
}

#[derive(Default)]
struct TextScope {
    child_element: bool,
    text: bool,
    whitespace: bool,
}

fn require_preserved_xml(
    bytes: &[u8],
    state: &State,
    changing: bool,
) -> Result<(), LoadManagerPowerError> {
    use LoadManagerPowerError::CanonicalStorageLoss;

    // The global digit-name substitution is safe for real attribute names
    // only, not for alias names or literal occurrences in document data.
    let patterns: Vec<_> = state
        .digit_attr_renames
        .iter()
        .flat_map(|(alias, original)| [format!(" {alias}="), format!(" {original}=")])
        .collect();
    if patterns.is_empty() && !changing {
        return Ok(());
    }
    let check_text = |raw: &[u8]| {
        if patterns.iter().any(|pattern| {
            raw.windows(pattern.len())
                .any(|window| window == pattern.as_bytes())
        }) {
            Err(CanonicalStorageLoss)
        } else {
            Ok(())
        }
    };
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().expand_empty_elements = true;
    let mut elements: Vec<TextScope> = Vec::new();
    loop {
        match reader.read_event().map_err(|_| CanonicalStorageLoss)? {
            Event::Start(element) => {
                for attr in element.attributes().with_checks(false) {
                    let attr = attr.map_err(|_| CanonicalStorageLoss)?;
                    if state
                        .digit_attr_renames
                        .iter()
                        .any(|(alias, _)| attr.key.as_ref() == alias.as_bytes())
                    {
                        return Err(CanonicalStorageLoss);
                    }
                    check_text(attr.value.as_ref())?;
                }
                if let Some(parent) = elements.last_mut() {
                    parent.child_element = true;
                }
                elements.push(TextScope::default());
            }
            Event::End(_) => {
                let ended = elements.pop().ok_or(CanonicalStorageLoss)?;
                // Only actual child elements justify interpreting whitespace as indentation.
                if changing && ended.whitespace && (ended.text || !ended.child_element) {
                    return Err(CanonicalStorageLoss);
                }
            }
            Event::Text(text) => {
                check_text(&text)?;
                if changing && let Some(parent) = elements.last_mut() {
                    if text
                        .unescape()
                        .map_err(|_| CanonicalStorageLoss)?
                        .bytes()
                        .all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
                    {
                        if text.as_ref().contains(&b'&') {
                            return Err(CanonicalStorageLoss);
                        }
                        parent.whitespace = true;
                    } else {
                        parent.text = true;
                    }
                }
            }
            Event::CData(data) => {
                check_text(&data)?;
                if let Some(parent) = elements.last_mut() {
                    parent.text = true;
                }
            }
            Event::Comment(comment) => {
                check_text(&comment)?;
                if changing && elements.is_empty() {
                    return Err(CanonicalStorageLoss);
                }
            }
            Event::PI(pi) => {
                check_text(&pi)?;
                if changing && elements.is_empty() {
                    return Err(CanonicalStorageLoss);
                }
            }
            Event::Decl(decl) if changing && decl.standalone().is_some() => {
                return Err(CanonicalStorageLoss);
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}
