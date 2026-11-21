use chrono::{DateTime, FixedOffset};
use std::{error::Error, fmt};
use xmltree::{Element, XMLNode};

use super::{
    EditError,
    legacy::State,
    transaction_scope::{ScopeError, require_ordinary_owner, require_plain_xml},
};

/// Reconciliation of one direct `LightscenesC/@Num` value.
#[derive(Debug, PartialEq, Eq)]
pub struct LightingMoodCountChange {
    /// Loader-decoded stored value, distinct from an absent attribute.
    pub before: Option<String>,
    /// Parsed signed-32-bit value, or `None` for absent or invalid storage.
    pub before_effective: Option<i32>,
    /// Stored spelling after the transaction.
    pub after: String,
    /// Count of direct `LightsceneC` elements, ignoring nested records.
    pub scenes: i32,
    pub changed: bool,
}

#[derive(Debug)]
pub struct LightingMoodCountOutcome {
    /// Original bytes on a no-op, otherwise the complete canonical XML.
    pub bytes: Vec<u8>,
    pub change: LightingMoodCountChange,
    /// True only when a change advanced the owning Document's save stamp.
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LightingMoodCountError {
    Edit(EditError),
    WrongBlockType,
    ContainerCount { found: usize },
    CountOutOfRange,
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
}

impl fmt::Display for LightingMoodCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::WrongBlockType => f.write_str("Mood recount requires LightController2."),
            Self::ContainerCount { found } => write!(
                f,
                "Mood recount requires one direct LightscenesC container, found {found}."
            ),
            Self::CountOutOfRange => f.write_str(
                "The direct scene count does not fit the native signed 32-bit Num field.",
            ),
            Self::NamespacedXml => f.write_str("Mood recount does not support XML namespaces."),
            Self::UnsupportedXml => f.write_str(
                "Mood recount requires one UTF-8 XML root and no document type declaration.",
            ),
            Self::DocumentScope => f.write_str(
                "Mood recount requires one owning Document at the root or directly under it, or no Document.",
            ),
            Self::TemplateOrInherited => {
                f.write_str("Mood recount does not support template or inherited owner state.")
            }
        }
    }
}

impl Error for LightingMoodCountError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ScopeError> for LightingMoodCountError {
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

/// Reconcile the stored `Num` of one legacy-selected `LightController2`.
///
/// The selected controller must have exactly one direct `LightscenesC`.
/// Missing, invalid or unequal signed-32-bit `Num` is set to the number of
/// direct `LightsceneC` records (including zero). Equivalent spellings such
/// as `+2` are preserved without serialization or stamping, returning the
/// original bytes. No other lighting storage is interpreted or changed.
///
/// This byte transaction is limited to plain UTF-8 XML with one root, no DTD,
/// and either no Document or one stampable owning Document at the root or
/// directly under it. Template or inherited scope is refused even on a no-op.
/// On a change, `now` supplies the monotonic save stamp if there is a Document.
/// Changed bytes use the legacy canonical writer (including BOM and line
/// endings, not a byte patch); the loader's lox-n37w limitation is unchanged.
/// No file or clock is accessed, no revised bytes are returned on error, and
/// no native save or runtime reconciliation is modeled.
pub fn recount_lighting_moods(
    bytes: &[u8],
    selector: &str,
    now: &DateTime<FixedOffset>,
) -> Result<LightingMoodCountOutcome, LightingMoodCountError> {
    let mut state = State::load(bytes).map_err(LightingMoodCountError::Edit)?;
    require_plain_xml(bytes)?;
    let plan = state.plan_lighting_mood_count(selector)?;
    require_ordinary_owner(&state, &plan.path)?;
    if !plan.change.changed {
        return Ok(LightingMoodCountOutcome {
            bytes: bytes.to_vec(),
            change: plan.change,
            document_stamped: false,
        });
    }
    let change = state.apply_lighting_mood_count(plan);
    let document_stamped = state.stamp_document(now);
    let bytes = state.to_bytes().map_err(LightingMoodCountError::Edit)?;
    Ok(LightingMoodCountOutcome {
        bytes,
        change,
        document_stamped,
    })
}

struct MoodCountPlan {
    path: Vec<usize>,
    change: LightingMoodCountChange,
}

impl State {
    fn plan_lighting_mood_count(
        &self,
        selector: &str,
    ) -> Result<MoodCountPlan, LightingMoodCountError> {
        use LightingMoodCountError as Refusal;

        let mut path = self
            .require_one(selector)
            .map_err(|error| Refusal::Edit(error.into()))?;
        let block = self.get_element(&path);
        if block.attributes.get("Type").map(String::as_str) != Some("LightController2") {
            return Err(Refusal::WrongBlockType);
        }
        let containers: Vec<_> = block
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|element| element.name == "LightscenesC")
                    .map(|element| (index, element))
            })
            .collect();
        let [(container_index, collection)] = containers.as_slice() else {
            return Err(Refusal::ContainerCount {
                found: containers.len(),
            });
        };
        // The existing recount follows scene-vector size, independently of
        // either Outputs field (append 0x1405b48e0, reconciliation 0x1405c6770).
        let scenes = i32::try_from(Self::direct_lighting_mood_count(collection))
            .map_err(|_| Refusal::CountOutOfRange)?;
        let before = collection.attributes.get("Num").cloned();
        let before_effective = before.as_ref().and_then(|raw| raw.parse::<i32>().ok());
        let changed = before_effective != Some(scenes);
        let after = if changed {
            scenes.to_string()
        } else {
            before.as_ref().unwrap().clone()
        };
        path.push(*container_index);
        Ok(MoodCountPlan {
            path,
            change: LightingMoodCountChange {
                before,
                before_effective,
                after,
                scenes,
                changed,
            },
        })
    }

    fn apply_lighting_mood_count(&mut self, plan: MoodCountPlan) -> LightingMoodCountChange {
        if plan.change.changed {
            self.get_element_mut(&plan.path)
                .attributes
                .insert("Num".into(), plan.change.after.clone());
        }
        plan.change
    }

    #[doc(hidden)]
    pub fn direct_lighting_mood_count(collection: &Element) -> usize {
        collection
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|element| element.name == "LightsceneC")
            .count()
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn recount_lighting_moods_selected(
        &mut self,
        selector: &str,
    ) -> Result<LightingMoodCountChange, LightingMoodCountError> {
        let plan = self.plan_lighting_mood_count(selector)?;
        Ok(self.apply_lighting_mood_count(plan))
    }

    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn plan_lighting_mood_count_selected(
        &self,
        selector: &str,
    ) -> Result<(LightingMoodCountChange, Vec<usize>), LightingMoodCountError> {
        let plan = self.plan_lighting_mood_count(selector)?;
        Ok((plan.change, plan.path))
    }
}
