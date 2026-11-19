use super::{EditError, ElementIdentity, legacy::State};
use chrono::{DateTime, FixedOffset};
use std::{error::Error, fmt};

/// A connector lookup refusal or an existing editor failure.
#[derive(Debug)]
pub enum UnwireError {
    ConnectorMissing {
        element: ElementIdentity,
        connector: String,
    },
    Edit(EditError),
}

impl fmt::Display for UnwireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectorMissing { element, connector } => {
                write!(
                    f,
                    "Connector '{connector}' not found on '{}'",
                    element.title
                )
            }
            Self::Edit(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl Error for UnwireError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConnectorMissing { .. } => None,
            Self::Edit(error) => error.source(),
        }
    }
}

impl From<EditError> for UnwireError {
    fn from(error: EditError) -> Self {
        Self::Edit(error)
    }
}

/// Observations of the first direct connector selected by the legacy mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnwireChange {
    pub element: ElementIdentity,
    pub connector: String,
    pub previous_uuid: Option<String>,
    pub removed_references: usize,
}

#[derive(Debug)]
pub struct UnwireOutcome {
    pub bytes: Vec<u8>,
    pub change: UnwireChange,
    /// Compared with the unedited canonical serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Disconnect the first direct `Co` whose case-sensitive `K` matches `connector`.
///
/// Removes all direct `In` children and `Nc`. If any direct `In` existed, `U`
/// remains the connector identity; otherwise `U` is removed as legacy wiring.
/// Other children, reference flags and other connectors are not repaired or
/// cascaded. Selection uses the existing UUID-mirror/room/title rules.
///
/// Changed canonical content receives the existing monotonic Document stamp
/// using the explicit instant. A canonical no-op is not stamped, although raw
/// bytes may normalize. Refusals return no revised bytes. The legacy loader's
/// quoted-text/digit-name limitation (lox-n37w) is unchanged.
pub fn unwire(
    bytes: &[u8],
    selector: &str,
    connector: &str,
    now: &DateTime<FixedOffset>,
) -> Result<UnwireOutcome, UnwireError> {
    let mut state = State::load(bytes)?;
    let change = state.unwire_connector(selector, connector)?;
    let mut output = state.to_bytes()?;
    let baseline = State::load(bytes)?.to_bytes()?;
    let content_changed = baseline != output;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    Ok(UnwireOutcome {
        bytes: output,
        change,
        content_changed,
        document_stamped,
    })
}

impl State {
    #[cfg_attr(feature = "unstable-legacy-cli-editor", doc(hidden))]
    pub fn unwire_connector(
        &mut self,
        selector: &str,
        connector: &str,
    ) -> Result<UnwireChange, UnwireError> {
        let path = self.require_one(selector).map_err(EditError::from)?;
        let element = self.get_element_mut(&path);
        let identity = ElementIdentity::from_element(element);
        let co = element
            .children
            .iter_mut()
            .find_map(|child| {
                child.as_mut_element().filter(|element| {
                    element.name == "Co"
                        && element.attributes.get("K").map(String::as_str) == Some(connector)
                })
            })
            .ok_or_else(|| UnwireError::ConnectorMissing {
                element: identity.clone(),
                connector: connector.to_owned(),
            })?;
        let previous_uuid = co.attributes.get("U").cloned();
        let before = co.children.len();
        co.children
            .retain(|child| !child.as_element().map(|e| e.name == "In").unwrap_or(false));
        let removed_references = before - co.children.len();
        co.attributes.shift_remove("Nc");
        if removed_references == 0 {
            co.attributes.shift_remove("U");
        }
        Ok(UnwireChange {
            element: identity,
            connector: connector.to_owned(),
            previous_uuid,
            removed_references,
        })
    }
}
