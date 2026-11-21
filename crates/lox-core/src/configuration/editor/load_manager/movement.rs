use chrono::{DateTime, FixedOffset};
use quick_xml::{Reader, events::Event};
use std::{error::Error, fmt};
use xmltree::Element;

use super::{
    EditError, LoadManagerInspection, LoadManagerInspectionError, State, inspect_positions,
    load_positions, resolve_io_index,
};

/// One step in stored XML order, not a runtime priority change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadManagerMoveDirection {
    Up,
    Down,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerMove {
    pub io_index: u8,
    /// One-based position in the stored Load-only list.
    pub from_priority: usize,
    pub to_priority: usize,
}

#[derive(Debug)]
pub struct LoadManagerMoveOutcome {
    pub bytes: Vec<u8>,
    pub movement: LoadManagerMove,
    pub inspection: LoadManagerInspection,
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadManagerMoveError {
    Edit(EditError),
    Inspection(LoadManagerInspectionError),
    InvalidRequestedIoIndex {
        io_index: u8,
    },
    Boundary {
        io_index: u8,
        direction: LoadManagerMoveDirection,
    },
    NamespacedXml,
    UnsupportedXml,
    DocumentScope,
    TemplateOrInherited,
    SelectedOwnerChanged,
}

impl fmt::Display for LoadManagerMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => error.fmt(f),
            Self::Inspection(error) => error.fmt(f),
            Self::InvalidRequestedIoIndex { .. } => {
                f.write_str("Load Manager io-index must be 0..11, not a 1-based priority.")
            }
            Self::Boundary {
                io_index,
                direction,
            } => write!(
                f,
                "Load io-index {io_index} is already at the {} priority boundary; nothing was changed.",
                match direction {
                    LoadManagerMoveDirection::Up => "first",
                    LoadManagerMoveDirection::Down => "last",
                }
            ),
            Self::NamespacedXml => f.write_str(
                "Load Manager movement does not support XML namespaces; nothing was changed.",
            ),
            Self::UnsupportedXml => f.write_str(
                "Load Manager movement requires one UTF-8 XML root, no non-UTF-8 encoding declaration and no document type declaration; nothing was changed.",
            ),
            Self::DocumentScope => f.write_str(
                "Load Manager movement requires a single owning Document at the root or directly under it, or no Document; nothing was changed.",
            ),
            Self::TemplateOrInherited => f.write_str(
                "Load Manager movement does not support Template/DocumentTemplate or inherited owner state; nothing was changed.",
            ),
            Self::SelectedOwnerChanged => f.write_str(
                "Load Manager selection would resolve to a different owner after the move; nothing was changed.",
            ),
        }
    }
}

impl Error for LoadManagerMoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Inspection(error) => Some(error),
            _ => None,
        }
    }
}

/// Move one complete direct `Load` record by its resolved connector `ioIdx`.
///
/// The legacy selector and complete twelve-record inspection rules apply. An
/// omitted zero stays omitted. Moving swaps only the adjacent Load positions,
/// not interleaved siblings, and never derives name, power or timing values.
/// A boundary move is an error, not a successful no-op. Namespaces, DTDs,
/// non-UTF-8 declarations, template or inherited owners and non-owning or
/// ambiguous Documents are refused.
///
/// Returns legacy canonical bytes (not a raw XML patch) and the post-move
/// inspection. `now` supplies the monotonic save stamp for a recognized
/// owning Document; document-less input is not stamped. The shared loader's
/// lox-n37w limitation is unchanged. On error there are no revised bytes.
pub fn move_load_manager_load(
    bytes: &[u8],
    selector: &str,
    io_index: u8,
    direction: LoadManagerMoveDirection,
    now: &DateTime<FixedOffset>,
) -> Result<LoadManagerMoveOutcome, LoadManagerMoveError> {
    use LoadManagerMoveError as Refusal;

    if io_index > 11 {
        return Err(Refusal::InvalidRequestedIoIndex { io_index });
    }
    let mut state = State::load(bytes).map_err(Refusal::Edit)?;
    require_plain_xml(bytes)?;
    let path = state
        .load_manager_path(selector)
        .map_err(Refusal::Inspection)?;
    require_ordinary_owner(&state, &path)?;
    let (movement, positions) = swap_load(&mut state, &path, io_index, direction)?;
    // The strict transaction must not report a different owner.
    if state
        .load_manager_path(selector)
        .map_err(Refusal::Inspection)?
        != path
    {
        return Err(Refusal::SelectedOwnerChanged);
    }
    let inspection = inspect_positions(state.get_element(&path), &positions);
    let document_stamped = state.stamp_document(now);
    let bytes = state.to_bytes().map_err(Refusal::Edit)?;
    Ok(LoadManagerMoveOutcome {
        bytes,
        movement,
        inspection,
        document_stamped,
    })
}

impl State {
    #[cfg(feature = "unstable-legacy-cli-editor")]
    #[doc(hidden)]
    pub fn move_load_manager_load(
        &mut self,
        selector: &str,
        io_index: u8,
        direction: LoadManagerMoveDirection,
    ) -> Result<(), LoadManagerMoveError> {
        use LoadManagerMoveError as Refusal;
        if io_index > 11 {
            return Err(Refusal::InvalidRequestedIoIndex { io_index });
        }
        let path = self
            .load_manager_path(selector)
            .map_err(Refusal::Inspection)?;
        swap_load(self, &path, io_index, direction)?;
        Ok(())
    }
}

fn swap_load(
    state: &mut State,
    path: &[usize],
    io_index: u8,
    direction: LoadManagerMoveDirection,
) -> Result<(LoadManagerMove, Vec<usize>), LoadManagerMoveError> {
    use LoadManagerMoveError as Refusal;
    let block = state.get_element(path);
    let positions = load_positions(block).map_err(Refusal::Inspection)?;
    // Twelve distinct resolved indices in 0..=11 always include the request.
    let index = positions
        .iter()
        .position(|&position| {
            resolve_io_index(block.children[position].as_element().unwrap()).unwrap() == io_index
        })
        .unwrap();
    let next = match direction {
        LoadManagerMoveDirection::Up if index > 0 => index - 1,
        LoadManagerMoveDirection::Down if index + 1 < positions.len() => index + 1,
        _ => {
            return Err(Refusal::Boundary {
                io_index,
                direction,
            });
        }
    };
    state
        .get_element_mut(path)
        .children
        .swap(positions[index], positions[next]);
    Ok((
        LoadManagerMove {
            io_index,
            from_priority: index + 1,
            to_priority: next + 1,
        },
        positions,
    ))
}

fn require_plain_xml(bytes: &[u8]) -> Result<(), LoadManagerMoveError> {
    use LoadManagerMoveError as Refusal;

    let text = std::str::from_utf8(bytes).map_err(|_| Refusal::UnsupportedXml)?;
    if bytes.contains(&0) {
        return Err(Refusal::UnsupportedXml);
    }
    // xmltree discards attribute prefixes. Check qualified names with the XML
    // reader before trusting DOM attributes; comments, CDATA and PI are opaque.
    let mut reader = Reader::from_str(text.strip_prefix('\u{feff}').unwrap_or(text));
    reader.config_mut().expand_empty_elements = true;
    let mut depth = 0usize;
    let mut root_seen = false;
    loop {
        match reader
            .read_event()
            .map_err(|error| Refusal::Edit(EditError::load(error)))?
        {
            Event::Start(element) => {
                if depth == 0 {
                    if root_seen {
                        return Err(Refusal::UnsupportedXml);
                    }
                    root_seen = true;
                }
                depth += 1;
                if element.name().as_ref().contains(&b':') {
                    return Err(Refusal::NamespacedXml);
                }
                // Identical duplicate attributes remain the shared loader's policy.
                for attr in element.attributes().with_checks(false) {
                    let attr = attr.map_err(|error| Refusal::Edit(EditError::load(error)))?;
                    let name = attr.key.as_ref();
                    if name == b"xmlns" || name.contains(&b':') {
                        return Err(Refusal::NamespacedXml);
                    }
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).ok_or(Refusal::UnsupportedXml)?;
            }
            Event::DocType(_) => return Err(Refusal::UnsupportedXml),
            Event::Decl(decl) => {
                if let Some(encoding) = decl.encoding()
                    && !encoding
                        .map_err(|error| Refusal::Edit(EditError::load(error)))?
                        .eq_ignore_ascii_case(b"utf-8")
                {
                    return Err(Refusal::UnsupportedXml);
                }
            }
            Event::Text(text) if depth == 0 => {
                if !text
                    .as_ref()
                    .iter()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
                {
                    return Err(Refusal::UnsupportedXml);
                }
            }
            Event::CData(_) if depth == 0 => return Err(Refusal::UnsupportedXml),
            Event::Eof if root_seen && depth == 0 => return Ok(()),
            Event::Eof => return Err(Refusal::UnsupportedXml),
            _ => {}
        }
    }
}

fn template(element: &Element) -> bool {
    matches!(element.name.as_str(), "Template" | "DocumentTemplate")
        || (element.name == "C"
            && matches!(
                element.attributes.get("Type").map(String::as_str),
                Some("Template" | "DocumentTemplate")
            ))
}

fn require_ordinary_owner(state: &State, path: &[usize]) -> Result<(), LoadManagerMoveError> {
    use LoadManagerMoveError as Refusal;

    for depth in 0..=path.len() {
        let element = state.get_element(&path[..depth]);
        if template(element)
            || element.attributes.contains_key("IfT")
            || element.attributes.contains_key("uuidParent")
            || element.attributes.get("WF").is_some_and(|raw| {
                raw.parse::<u32>()
                    .map_or(true, |flags| flags & 0x2000_0000 != 0)
            })
        {
            return Err(Refusal::TemplateOrInherited);
        }
    }
    let documents = state.find_elements("Type:Document");
    let document = match documents.as_slice() {
        [] => &state.root,
        [owner] if owner.len() <= 1 && path.starts_with(owner) => {
            let document = state.get_element(owner);
            if document.attributes.get("Type").map(String::as_str) != Some("Document") {
                return Err(Refusal::DocumentScope);
            }
            document
        }
        _ => return Err(Refusal::DocumentScope),
    };
    let has_direct_template = |element: &Element| {
        element
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .any(template)
    };
    if has_direct_template(document) || has_direct_template(&state.root) {
        return Err(Refusal::TemplateOrInherited);
    }
    Ok(())
}
