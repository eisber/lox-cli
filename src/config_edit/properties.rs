use super::{ConfigEditor, ConfigSurfaceEntry, ElementDescription};
use anyhow::{Result, bail};
use lox_core::configuration::editor::ProgramTextError;
use std::collections::{BTreeMap, BTreeSet};
use xmltree::Element;

#[cfg(test)]
mod legacy_attribute_characterization {
    use super::ConfigEditor;

    #[test]
    fn title_identity_raw_values_and_previous_option() {
        let xml = br#"<ControlList><C Type="And" U="a" Title="Old" Empty=""/><C Type="And" U="a" Title="Mirror"/></ControlList>"#;
        let mut editor = ConfigEditor::load(xml).unwrap();
        assert_eq!(
            editor.set_attribute("uuid:a", "Title", "New").unwrap(),
            "Set Old.Title: Old → 'New'"
        );
        assert_eq!(
            editor.set_attribute("uuid:a", "Empty", "").unwrap(),
            "Set New.Empty:  → ''"
        );
        assert_eq!(
            editor
                .set_attribute("uuid:a", "Future", "<&\"\n室")
                .unwrap(),
            "Set New.Future: (none) → '<&\"\n室'"
        );
        let bytes = editor.to_bytes().unwrap();
        let check = ConfigEditor::load(&bytes).unwrap();
        let path = check.require_one("uuid:a").unwrap();
        assert_eq!(check.get_element(&path).attributes["Future"], "<&\"\n室");
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .contains("Title=\"Mirror\"")
        );
    }

    #[test]
    fn selector_refusal_precedes_raw_attribute_mutation() {
        let xml = br#"<ControlList><C U="a" Title="Gate"/><C U="b" Title="Gate"/></ControlList>"#;
        let mut editor = ConfigEditor::load(xml).unwrap();
        let before = editor.to_bytes().unwrap();
        for selector in ["missing", "Gate"] {
            let expected = editor.require_one(selector).unwrap_err();
            let actual = editor.set_attribute(selector, "", "").unwrap_err();
            assert_eq!(actual.to_string(), expected.to_string());
            assert!(actual.downcast_ref::<String>().is_some());
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
}

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
        let change = self
            .kernel
            .set_attribute_legacy(selector, attr_name, value)
            .map_err(super::editor_error)?;
        Ok(format!(
            "Set {}.{}: {} → '{}'",
            change.element.title,
            change.attribute,
            change.old_value.as_deref().unwrap_or("(none)"),
            change.new_value
        ))
    }

    /// Set a parameter (Def value) on a connector of a block.
    pub fn set_param(&mut self, selector: &str, param: &str, value: &str) -> Result<()> {
        self.kernel
            .set_param_legacy(selector, param, value)
            .map_err(super::editor_error)
    }

    /// Describe an element: type, title, UUID, properties, connectors, children.
    pub fn describe(&self, selector: &str) -> Result<ElementDescription> {
        self.kernel
            .describe_selected(selector)
            .map(ElementDescription::from)
            .map_err(super::editor_error)
    }

    /// Describe field names and nested storage without exposing configuration values.
    pub fn configuration_surfaces(&self) -> Vec<ConfigSurfaceEntry> {
        fn original_attribute<'a>(name: &'a str, renames: &'a [(String, String)]) -> &'a str {
            renames
                .iter()
                .find_map(|(sanitized, original)| (sanitized == name).then_some(original.as_str()))
                .unwrap_or(name)
        }

        fn collect_paths(
            element: &Element,
            prefix: &str,
            paths: &mut BTreeSet<String>,
            renames: &[(String, String)],
        ) {
            for child in element.children.iter().filter_map(|node| node.as_element()) {
                let kind = child
                    .attributes
                    .get("Type")
                    .map(String::as_str)
                    .unwrap_or(child.name.as_str());
                if matches!(kind, "Co" | "IoData" | "In") {
                    continue;
                }
                let component = if child.name == "C" && child.attributes.contains_key("Type") {
                    format!("C[Type={kind}]")
                } else {
                    child.name.clone()
                };
                let path = if prefix.is_empty() {
                    component
                } else {
                    format!("{prefix}/{component}")
                };
                paths.insert(path.clone());
                for attribute in child.attributes.keys() {
                    let attribute = original_attribute(attribute, renames);
                    paths.insert(format!("{path}/@{attribute}"));
                }
                collect_paths(child, &path, paths, renames);
            }
        }

        let mut surfaces: BTreeMap<String, (usize, BTreeSet<String>, BTreeSet<String>)> =
            BTreeMap::new();
        for element in Self::iter_elements(&self.root) {
            let Some(kind) = element.attributes.get("Type") else {
                continue;
            };
            if matches!(kind.as_str(), "Co" | "IoData" | "In") {
                continue;
            }
            let (count, attributes, paths) = surfaces.entry(kind.clone()).or_default();
            *count += 1;
            for attribute in element.attributes.keys() {
                attributes
                    .insert(original_attribute(attribute, &self.digit_attr_renames).to_string());
            }
            collect_paths(element, "", paths, &self.digit_attr_renames);
        }
        surfaces
            .into_iter()
            .map(
                |(element_type, (occurrences, attributes, storage_paths))| ConfigSurfaceEntry {
                    element_type,
                    occurrences,
                    attributes: attributes.into_iter().collect(),
                    storage_paths: storage_paths.into_iter().collect(),
                },
            )
            .collect()
    }

    /// Read the program text from a SequenceController block's
    /// `<Field Name="Configuration">` element.
    pub fn get_program_text(&self, selector: &str) -> Result<String> {
        self.kernel
            .program_text_selected(selector)
            .map_err(program_text_error)
    }

    /// Write program text to a SequenceController block's
    /// `<Field Name="Configuration">` element.
    ///
    /// Creates the `<Field>` element if it doesn't exist.
    /// Validates the program syntax before writing (returns error with line numbers if invalid).
    pub fn set_program_text(&mut self, selector: &str, program: &str) -> Result<String> {
        use lox_sim::blocks::sequence::{count_program_lines, count_sequences, validate_program};

        let path = self.require_one(selector)?;
        let elem = self.get_element(&path);
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();

        if block_type != "SequenceController" {
            bail!(
                "'{}' is a {} — set-program is only for SequenceController blocks",
                title,
                block_type
            );
        }

        // Validate program syntax
        let errors = validate_program(program);
        if !errors.is_empty() {
            let mut msg = format!(
                "Program has {} syntax error{}:\n",
                errors.len(),
                if errors.len() == 1 { "" } else { "s" }
            );
            for err in &errors {
                msg.push_str(&format!("{}\n", err));
            }
            bail!("{}", msg.trim_end());
        }

        // Now get mutable reference and write
        let elem = self.get_element_mut(&path);

        // Find or create <Field Name="Configuration">
        let field_idx = elem.children.iter().position(|c| {
            c.as_element().is_some_and(|f| {
                f.name == "Field"
                    && f.attributes
                        .get("Name")
                        .map(|n| n == "Configuration")
                        .unwrap_or(false)
            })
        });

        if let Some(idx) = field_idx {
            let field = elem.children[idx].as_mut_element().unwrap();
            field.children.clear();
            field
                .children
                .push(xmltree::XMLNode::Text(program.to_string()));
        } else {
            let mut field = Element::new("Field");
            field
                .attributes
                .insert("Name".to_string(), "Configuration".to_string());
            field
                .children
                .push(xmltree::XMLNode::Text(program.to_string()));
            elem.children.push(xmltree::XMLNode::Element(field));
        }

        let lines = count_program_lines(program);
        let seqs = count_sequences(program);
        Ok(format!(
            "✓ Set program ({} line{}, {} sequence{}) on '{}'",
            lines,
            if lines == 1 { "" } else { "s" },
            seqs,
            if seqs == 1 { "" } else { "s" },
            title,
        ))
    }
}

pub(super) fn program_text_error(error: ProgramTextError) -> anyhow::Error {
    match error {
        ProgramTextError::Edit(error) => super::editor_error(error),
        error @ ProgramTextError::Unsupported { .. } => anyhow::anyhow!("{error}"),
    }
}
