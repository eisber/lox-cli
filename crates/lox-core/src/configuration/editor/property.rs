use super::{EditError, ElementIdentity, legacy::State};
use chrono::{DateTime, FixedOffset};
use xmltree::Element;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyChange {
    pub element: ElementIdentity,
    pub property: String,
    pub created: bool,
    /// Missing `v` on an existing property remains distinct from an empty value.
    pub old_value: Option<String>,
    pub old_type_code: Option<String>,
    pub new_value: String,
    pub new_type_code: String,
}

#[derive(Debug)]
pub struct SetPropertyOutcome {
    pub bytes: Vec<u8>,
    pub change: PropertyChange,
    /// Compared against an unedited canonical serialization, not the raw input.
    pub content_changed: bool,
    pub document_stamped: bool,
}

/// Edit the first direct property in the first direct `SET` of one selected element.
///
/// Missing `SET` and property elements are appended. Both `t` and `v` are replaced
/// on an existing property; its other attributes, children and position survive.
/// Names, values and type codes retain the legacy raw-string acceptance: this is
/// not schema validation or the connector `Co.Def` operation.
///
/// Uses the shared legacy loader (including lox-n37w), selector, canonical
/// serializer and explicit monotonic document stamp. A canonical no-op does not
/// stamp, even when normalization changes raw bytes. Errors return no revised
/// bytes, and the borrowed input is never mutated.
pub fn set_property(
    bytes: &[u8],
    selector: &str,
    property: &str,
    value: &str,
    type_code: &str,
    now: &DateTime<FixedOffset>,
) -> Result<SetPropertyOutcome, EditError> {
    let mut state = State::load(bytes)?;
    let change = state.edit_property(selector, property, value, type_code)?;
    let mut output = state.to_bytes()?;
    let baseline = State::load(bytes)?.to_bytes()?;
    let content_changed = baseline != output;
    let document_stamped = content_changed && state.stamp_document(now);
    if document_stamped {
        output = state.to_bytes()?;
    }
    Ok(SetPropertyOutcome {
        bytes: output,
        change,
        content_changed,
        document_stamped,
    })
}

impl State {
    fn edit_property(
        &mut self,
        selector: &str,
        prop_name: &str,
        value: &str,
        type_code: &str,
    ) -> Result<PropertyChange, EditError> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let identity = ElementIdentity::from_element(elem);

        let set_idx = elem
            .children
            .iter()
            .position(|c| c.as_element().map(|e| e.name == "SET").unwrap_or(false));

        let set_elem = if let Some(idx) = set_idx {
            elem.children[idx].as_mut_element().unwrap()
        } else {
            let set = Element::new("SET");
            elem.children.push(xmltree::XMLNode::Element(set));
            let last = elem.children.len() - 1;
            elem.children[last].as_mut_element().unwrap()
        };

        let prop_idx = set_elem
            .children
            .iter()
            .position(|c| c.as_element().map(|e| e.name == prop_name).unwrap_or(false));

        let (old_value, old_type_code) = if let Some(idx) = prop_idx {
            let prop = set_elem.children[idx].as_mut_element().unwrap();
            let old_value = prop.attributes.get("v").cloned();
            let old_type_code = prop.attributes.get("t").cloned();
            prop.attributes
                .insert("t".to_string(), type_code.to_string());
            prop.attributes.insert("v".to_string(), value.to_string());
            (old_value, old_type_code)
        } else {
            let mut prop = Element::new(prop_name);
            prop.attributes
                .insert("t".to_string(), type_code.to_string());
            prop.attributes.insert("v".to_string(), value.to_string());
            set_elem.children.push(xmltree::XMLNode::Element(prop));
            (None, None)
        };
        Ok(PropertyChange {
            element: identity,
            property: prop_name.to_owned(),
            created: prop_idx.is_none(),
            old_value,
            old_type_code,
            new_value: value.to_owned(),
            new_type_code: type_code.to_owned(),
        })
    }

    /// Cached CLI bridge: selection/mutation precedes its report and filesystem save.
    #[cfg(feature = "unstable-legacy-cli-editor")]
    pub fn set_property_legacy(
        &mut self,
        selector: &str,
        property: &str,
        value: &str,
        type_code: &str,
    ) -> Result<PropertyChange, EditError> {
        self.edit_property(selector, property, value, type_code)
    }
}
