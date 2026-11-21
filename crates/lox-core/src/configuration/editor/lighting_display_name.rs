use chrono::{DateTime, FixedOffset};
use std::{error::Error, fmt};

use super::{
    EditError,
    legacy::State,
    transaction_scope::{
        ScopeError, require_ordinary_owner, require_round_trip_xml, require_xml_1_0_declaration,
    },
};

/// Exact, case-sensitive decoded attribute values, not legacy selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightingMoodDisplayNameTarget<'a> {
    /// The controller's stored `C/@U`, with no selector prefix.
    pub controller_uuid: &'a str,
    /// `LightsceneC/@UUID` in that controller's sole direct scene collection.
    pub mood_uuid: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightingMoodDisplayNameValue<'a> {
    /// Replace a nonempty `SName` with an explicit empty attribute.
    Clear,
    /// A nonempty uppercase ASCII literal with letters, digits and internal
    /// ordinary spaces. No case conversion, trimming or other normalization.
    Literal(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct LightingMoodDisplayNameChange {
    /// Loader-decoded `SName` before the edit; absent and empty are distinct.
    pub before: Option<String>,
    /// Stored `SName` after the edit, still absent if clearing absent storage.
    pub after: Option<String>,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LightingMoodDisplayNameOutcome {
    /// Original bytes on a no-op, otherwise complete canonical XML.
    pub bytes: Vec<u8>,
    pub change: LightingMoodDisplayNameChange,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LightingMoodDisplayNameError {
    InvalidLiteral,
    EmptyControllerUuid,
    EmptyMoodUuid,
    Edit(EditError),
    UnsupportedXml,
    NamespacedXml,
    AmbiguousWhitespace,
    LoaderSubstitution,
    ControllerCount { found: usize },
    WrongBlockType,
    ContainerCount { found: usize },
    MoodCount { found: usize },
    DocumentScope,
    TemplateOrInherited,
    CanonicalRoundTrip,
}

impl fmt::Display for LightingMoodDisplayNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLiteral => f.write_str(
                "Use a nonempty uppercase ASCII literal with alphanumeric edges and internal ordinary spaces, or explicitly clear.",
            ),
            Self::EmptyControllerUuid => f.write_str("Controller U must not be empty."),
            Self::EmptyMoodUuid => f.write_str("Mood UUID must not be empty."),
            Self::Edit(error) => error.fmt(f),
            Self::UnsupportedXml => f.write_str(
                "Mood display-name editing requires one UTF-8 XML 1.0 root without a DTD, duplicate attributes, or comments or instructions outside the root.",
            ),
            Self::NamespacedXml => f.write_str("Mood display-name editing does not support XML namespaces."),
            Self::AmbiguousWhitespace => f.write_str(
                "The XML contains whitespace-only text that the writer cannot safely treat as indentation.",
            ),
            Self::LoaderSubstitution => f.write_str(
                "The shared loader's digit-attribute substitution prevents an exact mood display-name byte transaction.",
            ),
            Self::ControllerCount { found } => write!(
                f,
                "Expected one C with the exact controller U, found {found}."
            ),
            Self::WrongBlockType => f.write_str("Mood display-name editing requires LightController2."),
            Self::ContainerCount { found } => write!(
                f,
                "Expected one direct LightscenesC container, found {found}."
            ),
            Self::MoodCount { found } => write!(
                f,
                "Expected one direct LightsceneC with the exact mood UUID, found {found}."
            ),
            Self::DocumentScope => f.write_str(
                "Mood display-name editing requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => f.write_str(
                "Mood display-name editing does not support template or inherited owners or moods."
            ),
            Self::CanonicalRoundTrip => f.write_str(
                "The canonical XML writer could not preserve the edited document tree."
            ),
        }
    }
}

impl Error for LightingMoodDisplayNameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ScopeError> for LightingMoodDisplayNameError {
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

/// Edit only `SName` on one exact stored mood, not the mood's `Name`.
///
/// Refusals are ordered: value, nonempty target IDs, XML declaration/load/scope
/// and loader substitutions, unique controller `C/@U` and its type, unique
/// direct `LightscenesC` and direct mood `@UUID` within it, then ordinary
/// ownership. No fuzzy matching, duplicate-controller folding, nested mood
/// fallback, or SID/CID/Name interpretation is performed.
///
/// Absent and explicit-empty `SName` are both no-ops when clearing. An
/// identical supported literal is also a no-op. No-ops return the original
/// bytes without stamping, but still enforce admission. On a change, `now`
/// supplies the shared save stamp for a recognized owning Document, if any.
/// The returned bytes use the canonical XML 1.0 writer, not a byte patch.
/// Duplicate attributes and comments or instructions outside the root are
/// refused rather than lost; the result must reopen as the edited root.
/// Whitespace-only text is refused except literal multiline indentation
/// around element children of a parent with no direct text or CDATA payload.
/// The shared loader's lox-n37w digit-attribute rewriting is refused, not
/// repaired. No filesystem, clock or runtime is accessed, and no bytes are
/// returned on error.
pub fn set_lighting_mood_display_name(
    bytes: &[u8],
    target: LightingMoodDisplayNameTarget<'_>,
    value: LightingMoodDisplayNameValue<'_>,
    now: &DateTime<FixedOffset>,
) -> Result<LightingMoodDisplayNameOutcome, LightingMoodDisplayNameError> {
    use LightingMoodDisplayNameError as Refusal;

    let name = match value {
        LightingMoodDisplayNameValue::Clear => "",
        LightingMoodDisplayNameValue::Literal(name) if State::is_lighting_display_name(name) => {
            name
        }
        LightingMoodDisplayNameValue::Literal(_) => return Err(Refusal::InvalidLiteral),
    };
    if target.controller_uuid.is_empty() {
        return Err(Refusal::EmptyControllerUuid);
    }
    if target.mood_uuid.is_empty() {
        return Err(Refusal::EmptyMoodUuid);
    }
    require_xml_1_0_declaration(bytes)?;
    let mut state = State::load(bytes).map_err(Refusal::Edit)?;
    if require_round_trip_xml(bytes)? {
        return Err(Refusal::AmbiguousWhitespace);
    }
    if !state.digit_attr_renames.is_empty() {
        return Err(Refusal::LoaderSubstitution);
    }
    let path = state.exact_lighting_display_name_path(target)?;
    require_ordinary_owner(&state, &path)?;
    let change = state.plan_lighting_display_name(&path, name);
    if !change.changed {
        return Ok(LightingMoodDisplayNameOutcome {
            bytes: bytes.to_vec(),
            change,
            document_stamped: false,
        });
    }

    state.apply_lighting_display_name(&path, &change);
    let document_stamped = state.stamp_document(now);
    let bytes = state.to_bytes().map_err(Refusal::Edit)?;
    let reopened = State::load(&bytes).map_err(Refusal::Edit)?;
    if !reopened.digit_attr_renames.is_empty() {
        return Err(Refusal::LoaderSubstitution);
    }
    if reopened.root != state.root {
        return Err(Refusal::CanonicalRoundTrip);
    }
    Ok(LightingMoodDisplayNameOutcome {
        bytes,
        change,
        document_stamped,
    })
}

impl State {
    fn exact_lighting_display_name_path(
        &self,
        target: LightingMoodDisplayNameTarget<'_>,
    ) -> Result<Vec<usize>, LightingMoodDisplayNameError> {
        use LightingMoodDisplayNameError as Refusal;

        // require_one would fold distinct C elements carrying the same U.
        let matches = self.find_elements(&format!("uuid:{}", target.controller_uuid));
        let [controller_path] = matches.as_slice() else {
            return Err(Refusal::ControllerCount {
                found: matches.len(),
            });
        };
        let controller = self.get_element(controller_path);
        if controller.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Refusal::WrongBlockType);
        }
        let collections: Vec<_> = controller
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| element.name == "LightscenesC")
                    .map(|element| (index, element))
            })
            .collect();
        let [(collection_index, collection)] = collections.as_slice() else {
            return Err(Refusal::ContainerCount {
                found: collections.len(),
            });
        };
        let matches: Vec<_> = collection
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| {
                        element.name == "LightsceneC"
                            && element.attributes.get("UUID").map(String::as_str)
                                == Some(target.mood_uuid)
                    })
                    .map(|_| index)
            })
            .collect();
        let [mood_index] = matches.as_slice() else {
            return Err(Refusal::MoodCount {
                found: matches.len(),
            });
        };
        let mut path = controller_path.to_vec();
        path.extend([*collection_index, *mood_index]);
        Ok(path)
    }

    #[doc(hidden)]
    pub fn is_lighting_display_name(name: &str) -> bool {
        let alphanumeric = |byte: &u8| byte.is_ascii_uppercase() || byte.is_ascii_digit();
        name.as_bytes().first().is_some_and(alphanumeric)
            && name.as_bytes().last().is_some_and(alphanumeric)
            && name.bytes().all(|byte| alphanumeric(&byte) || byte == b' ')
    }

    fn plan_lighting_display_name(
        &self,
        path: &[usize],
        name: &str,
    ) -> LightingMoodDisplayNameChange {
        let before = self.get_element(path).attributes.get("SName").cloned();
        let changed = before.as_deref().unwrap_or("") != name;
        let after = if changed {
            Some(name.into())
        } else {
            before.clone()
        };
        LightingMoodDisplayNameChange {
            before,
            after,
            changed,
        }
    }

    fn apply_lighting_display_name(
        &mut self,
        path: &[usize],
        change: &LightingMoodDisplayNameChange,
    ) {
        if change.changed {
            let after = change
                .after
                .as_ref()
                .expect("a changed display name is always stored");
            self.get_element_mut(path)
                .attributes
                .insert("SName".into(), after.clone());
        }
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn plan_lighting_display_name_selected(
        &self,
        path: &[usize],
        name: &str,
    ) -> LightingMoodDisplayNameChange {
        self.plan_lighting_display_name(path, name)
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn set_lighting_display_name_selected(
        &mut self,
        path: &[usize],
        name: &str,
    ) -> LightingMoodDisplayNameChange {
        let change = self.plan_lighting_display_name(path, name);
        self.apply_lighting_display_name(path, &change);
        change
    }
}
