use super::{EditError, legacy::State};
use std::collections::HashMap;
use xmltree::Element;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyValue {
    pub value: String,
    pub type_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorInfo {
    pub kind: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildDescription {
    pub element_type: String,
    pub title: String,
}

/// Owned observations of one legacy-selected element, not a document model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementDescription {
    pub element_type: String,
    pub title: String,
    pub uuid: String,
    pub gid: String,
    pub room_uuid: String,
    pub category_uuid: String,
    pub properties: HashMap<String, PropertyValue>,
    pub connectors: Vec<ConnectorInfo>,
    pub children: Vec<ChildDescription>,
}

/// Describe one element without serializing, stamping or modifying the input.
///
/// Uses the existing legacy loader and selector, including UUID mirrors and
/// bracket-room matching. Direct SET fields overwrite earlier same-name fields
/// across all direct SET blocks; map iteration order is unspecified. Connectors
/// and child identities retain direct-child source order. The last direct IoData
/// supplies raw room/category UUIDs (missing attributes clear earlier values).
/// No room/category label lookup or recursive metadata aggregation is performed.
/// The legacy quoted-text/digit-name limitation (lox-n37w) is unchanged.
pub fn describe_element(bytes: &[u8], selector: &str) -> Result<ElementDescription, EditError> {
    State::load(bytes)?.describe_selected(selector)
}

impl State {
    pub fn describe_selected(&self, selector: &str) -> Result<ElementDescription, EditError> {
        let path = self.require_one(selector)?;
        Ok(describe(self.get_element(&path)))
    }
}

fn describe(elem: &Element) -> ElementDescription {
    let mut properties = HashMap::new();
    let mut connectors = Vec::new();
    let mut children = Vec::new();
    let mut room = String::new();
    let mut category = String::new();

    for child in &elem.children {
        if let Some(child_elem) = child.as_element() {
            match child_elem.name.as_str() {
                "SET" => {
                    for prop_node in &child_elem.children {
                        if let Some(prop) = prop_node.as_element() {
                            let val = prop.attributes.get("v").cloned().unwrap_or_default();
                            let t = prop.attributes.get("t").cloned().unwrap_or_default();
                            properties.insert(
                                prop.name.clone(),
                                PropertyValue {
                                    value: val,
                                    type_code: t,
                                },
                            );
                        }
                    }
                }
                "Co" => {
                    connectors.push(ConnectorInfo {
                        kind: child_elem.attributes.get("K").cloned().unwrap_or_default(),
                        target: child_elem.attributes.get("U").cloned().unwrap_or_default(),
                    });
                }
                "IoData" => {
                    room = child_elem.attributes.get("Pr").cloned().unwrap_or_default();
                    category = child_elem.attributes.get("Cr").cloned().unwrap_or_default();
                }
                "C" => {
                    children.push(ChildDescription {
                        element_type: child_elem
                            .attributes
                            .get("Type")
                            .cloned()
                            .unwrap_or_default(),
                        title: child_elem
                            .attributes
                            .get("Title")
                            .cloned()
                            .unwrap_or_default(),
                    });
                }
                _ => {}
            }
        }
    }

    ElementDescription {
        element_type: elem.attributes.get("Type").cloned().unwrap_or_default(),
        title: elem.attributes.get("Title").cloned().unwrap_or_default(),
        uuid: elem.attributes.get("U").cloned().unwrap_or_default(),
        gid: elem.attributes.get("gid").cloned().unwrap_or_default(),
        room_uuid: room,
        category_uuid: category,
        properties,
        connectors,
        children,
    }
}
