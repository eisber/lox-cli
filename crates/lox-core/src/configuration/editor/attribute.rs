use super::{EditError, ElementIdentity, legacy::State};
use chrono::{DateTime, FixedOffset};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeChange {
    /// Identity captured before inserting the attribute, including a new `Title`.
    pub element: ElementIdentity,
    pub attribute: String,
    /// `None` is distinct from an explicitly empty attribute.
    pub old_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug)]
pub struct SetAttributeOutcome {
    pub bytes: Vec<u8>,
    pub change: AttributeChange,
    /// Compared with an unedited canonical serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Insert a raw attribute on the first legacy-selected element.
///
/// This migrates the existing CLI operation, not a native configuration contract.
/// Names and values are not validated, references are not repaired, and UUID
/// mirrors retain the shared selector's first-occurrence behavior. The caller
/// supplies the save instant. Canonical no-ops do not stamp but can normalize
/// input bytes; the shared loader's lox-n37w limitation remains unchanged.
/// Errors return no revised bytes and cannot mutate the borrowed input.
pub fn set_attribute(
    bytes: &[u8],
    selector: &str,
    attribute: &str,
    value: &str,
    now: &DateTime<FixedOffset>,
) -> Result<SetAttributeOutcome, EditError> {
    let mut state = State::load(bytes)?;
    let change = state.edit_attribute(selector, attribute, value)?;
    let mut output = state.to_bytes()?;
    let baseline = State::load(bytes)?.to_bytes()?;
    let content_changed = baseline != output;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    Ok(SetAttributeOutcome {
        bytes: output,
        change,
        content_changed,
        document_stamped,
    })
}

impl State {
    fn edit_attribute(
        &mut self,
        selector: &str,
        attribute: &str,
        value: &str,
    ) -> Result<AttributeChange, EditError> {
        let path = self.require_one(selector)?;
        let element = self.get_element_mut(&path);
        let identity = ElementIdentity::from_element(element);
        let old_value = element
            .attributes
            .insert(attribute.to_owned(), value.to_owned());
        Ok(AttributeChange {
            element: identity,
            attribute: attribute.to_owned(),
            old_value,
            new_value: value.to_owned(),
        })
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub fn set_attribute_legacy(
        &mut self,
        selector: &str,
        attribute: &str,
        value: &str,
    ) -> Result<AttributeChange, EditError> {
        self.edit_attribute(selector, attribute, value)
    }
}
