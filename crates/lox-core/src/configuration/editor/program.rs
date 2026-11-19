use super::{EditError, ElementIdentity, legacy::State};
use std::{error::Error, fmt};
use xmltree::XMLNode;

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
        let path = self
            .require_one(selector)
            .map_err(|error| ProgramTextError::Edit(error.into()))?;
        let element = self.get_element(&path);
        let identity = ElementIdentity::from_element(element);
        if identity.block_type != "SequenceController" {
            return Err(ProgramTextError::Unsupported { element: identity });
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
