use super::{EditError, ElementIdentity, legacy::State};
use std::{error::Error, fmt};
use xmltree::{Element, XMLNode};

#[cfg(feature = "configuration-xml")]
mod replacement;
#[cfg(feature = "configuration-xml")]
pub use replacement::{
    ProgramTextReplacementError, ProgramTextReplacementOutcome, replace_program_text,
};

/// Failures specific to reading program text.
#[derive(Debug)]
pub enum ProgramTextError {
    /// The existing editor failed to load the configuration or resolve the selector.
    Edit(EditError),
    /// The selected element is not a SequenceController.
    Unsupported { element: ElementIdentity },
}

impl fmt::Display for ProgramTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::Unsupported { element } => write!(
                f,
                "'{}' is a {} — get-program is only for SequenceController blocks",
                element.title, element.block_type
            ),
        }
    }
}

impl Error for ProgramTextError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Unsupported { .. } => None,
        }
    }
}

/// Read a selected SequenceController's first direct Configuration field.
///
/// Only direct text nodes are concatenated, in source order; nested elements,
/// CDATA and differently spelled names are ignored. A missing field is empty.
/// The legacy loader and selector run before exact block-type eligibility.
/// No program validation, normalization, serialization or stamping is performed.
/// The shared loader's quoted-text/digit-name limitation (lox-n37w) is unchanged.
pub fn get_program_text(bytes: &[u8], selector: &str) -> Result<String, ProgramTextError> {
    State::load(bytes)
        .map_err(ProgramTextError::Edit)?
        .program_text_selected(selector)
}

impl State {
    pub fn program_text_selected(&self, selector: &str) -> Result<String, ProgramTextError> {
        let (path, _) = self.select_program_owner(selector)?;
        let field = self
            .get_element(&path)
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .find(|field| is_configuration_field(field));
        Ok(field
            .into_iter()
            .flat_map(|field| &field.children)
            .filter_map(XMLNode::as_text)
            .collect())
    }

    fn select_program_owner(
        &self,
        selector: &str,
    ) -> Result<(Vec<usize>, ElementIdentity), ProgramTextError> {
        let path = self
            .require_one(selector)
            .map_err(|error| ProgramTextError::Edit(error.into()))?;
        let element = ElementIdentity::from_element(self.get_element(&path));
        if element.block_type != "SequenceController" {
            return Err(ProgramTextError::Unsupported { element });
        }
        Ok((path, element))
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_program_text_legacy<E>(
        &mut self,
        selector: &str,
        program: &str,
        validate: impl FnOnce(&str) -> Result<(), E>,
        map_selection: impl FnOnce(ProgramTextError) -> E,
    ) -> Result<ElementIdentity, E> {
        let (path, element) = self.select_program_owner(selector).map_err(map_selection)?;
        validate(program)?;
        self.write_program_field(&path, program.to_owned());
        Ok(element)
    }

    #[cfg(any(feature = "configuration-xml", feature = "unstable-legacy-cli-editor"))]
    fn write_program_field(&mut self, path: &[usize], text: String) {
        let element = self.get_element_mut(path);
        if let Some(field) = element
            .children
            .iter_mut()
            .filter_map(XMLNode::as_mut_element)
            .find(|field| is_configuration_field(field))
        {
            field.children = vec![XMLNode::Text(text)];
        } else {
            let mut field = Element::new("Field");
            field.attributes.insert("Name".into(), "Configuration".into());
            field.children.push(XMLNode::Text(text));
            element.children.push(XMLNode::Element(field));
        }
    }
}

fn is_configuration_field(field: &Element) -> bool {
    field.name == "Field"
        && field.attributes.get("Name").map(String::as_str) == Some("Configuration")
}
