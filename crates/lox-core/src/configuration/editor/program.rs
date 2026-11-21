use super::{EditError, ElementIdentity, legacy::State};
use xmltree::XMLNode;

/// Read a selected SequenceController's first direct Configuration field.
///
/// Only direct text nodes are concatenated, in source order; nested elements,
/// CDATA and differently spelled names are ignored. A missing field is empty.
/// The legacy loader and selector run before exact block-type eligibility.
/// No program validation, normalization, serialization or stamping is performed.
/// The shared loader's quoted-text/digit-name limitation (lox-n37w) is unchanged.
pub fn get_program_text(bytes: &[u8], selector: &str) -> Result<String, EditError> {
    State::load(bytes)?.program_text_selected(selector)
}

impl State {
    pub fn program_text_selected(&self, selector: &str) -> Result<String, EditError> {
        let path = self.require_one(selector)?;
        let element = self.get_element(&path);
        let identity = ElementIdentity::from_element(element);
        if identity.block_type != "SequenceController" {
            return Err(EditError::ProgramTextUnsupported { element: identity });
        }
        for field in element.children.iter().filter_map(XMLNode::as_element) {
            if field.name == "Field"
                && field
                    .attributes
                    .get("Name")
                    .is_some_and(|name| name == "Configuration")
            {
                return Ok(field
                    .children
                    .iter()
                    .filter_map(|node| match node {
                        XMLNode::Text(text) => Some(text.as_str()),
                        _ => None,
                    })
                    .collect());
            }
        }
        Ok(String::new())
    }
}
