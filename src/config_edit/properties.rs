use super::{ConfigEditor, ConnectorInfo, ElementDescription, PropertyValue};
use anyhow::{Result, bail};
use std::collections::HashMap;
use xmltree::Element;

impl ConfigEditor {
    /// Set a property in an element's SET block.
    /// Creates the SET block and property tag if they don't exist.
    pub fn set_property(
        &mut self,
        selector: &str,
        prop_name: &str,
        value: &str,
        type_code: &str,
    ) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();

        // Find or create SET child
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

        // Find or create property tag
        let prop_idx = set_elem
            .children
            .iter()
            .position(|c| c.as_element().map(|e| e.name == prop_name).unwrap_or(false));

        if let Some(idx) = prop_idx {
            let prop = set_elem.children[idx].as_mut_element().unwrap();
            let old_val = prop.attributes.get("v").cloned().unwrap_or_default();
            prop.attributes
                .insert("t".to_string(), type_code.to_string());
            prop.attributes.insert("v".to_string(), value.to_string());
            Ok(format!(
                "Updated {}.{}: '{}' → '{}'",
                title, prop_name, old_val, value
            ))
        } else {
            let mut prop = Element::new(prop_name);
            prop.attributes
                .insert("t".to_string(), type_code.to_string());
            prop.attributes.insert("v".to_string(), value.to_string());
            set_elem.children.push(xmltree::XMLNode::Element(prop));
            Ok(format!("Created {}.{} = '{}'", title, prop_name, value))
        }
    }

    /// Set an attribute on an element.
    pub fn set_attribute(
        &mut self,
        selector: &str,
        attr_name: &str,
        value: &str,
    ) -> Result<String> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
        let old = elem
            .attributes
            .insert(attr_name.to_string(), value.to_string());
        Ok(format!(
            "Set {}.{}: {} → '{}'",
            title,
            attr_name,
            old.as_deref().unwrap_or("(none)"),
            value
        ))
    }

    /// Set a parameter (Def value) on a connector of a block.
    pub fn set_param(&mut self, selector: &str, param: &str, value: &str) -> Result<()> {
        let path = self.require_one(selector)?;
        let elem = self.get_element_mut(&path);
        for child in &mut elem.children {
            if let Some(co) = child.as_mut_element()
                && co.name == "Co"
                && co.attributes.get("K").map(|k| k == param).unwrap_or(false)
            {
                co.attributes.insert("Def".to_string(), value.to_string());
                return Ok(());
            }
        }
        bail!("Connector '{}' not found on block", param)
    }

    /// Describe an element: type, title, UUID, properties, connectors, children.
    pub fn describe(&self, selector: &str) -> Result<ElementDescription> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(path.as_slice());
        self.describe_element(elem)
    }

    fn describe_element(&self, elem: &Element) -> Result<ElementDescription> {
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
                        let ct = child_elem
                            .attributes
                            .get("Type")
                            .cloned()
                            .unwrap_or_default();
                        let ct_title = child_elem
                            .attributes
                            .get("Title")
                            .cloned()
                            .unwrap_or_default();
                        children.push(format!("{}: {}", ct, ct_title));
                    }
                    _ => {}
                }
            }
        }

        Ok(ElementDescription {
            element_type: elem.attributes.get("Type").cloned().unwrap_or_default(),
            title: elem.attributes.get("Title").cloned().unwrap_or_default(),
            uuid: elem.attributes.get("U").cloned().unwrap_or_default(),
            gid: elem.attributes.get("gid").cloned().unwrap_or_default(),
            room_uuid: room,
            category_uuid: category,
            properties,
            connectors,
            children,
        })
    }
}
