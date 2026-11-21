use chrono::{DateTime, FixedOffset};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    num::ParseIntError,
};
use xmltree::Element;

use super::{EditError, legacy::State};

const LINKED: u32 = 0x1000_0000;

/// One stored `linkC` reference, not a row in native Config.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CentralLightLink {
    /// One-based position in the stored list.
    pub position: usize,
    /// The UUID spelling from `linkC`, not the target's `U` spelling.
    pub uuid: String,
    /// The target's stored `Title`, or an empty string if absent.
    pub title: String,
    /// The target's stored `Type`, or an empty string if absent.
    pub control_type: String,
}

/// Stored links on one selected CentralLight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CentralLightLinks {
    /// The selected owner's stored `U`.
    pub uuid: String,
    /// The owner's stored `Title`, or an empty string if absent.
    pub title: String,
    pub links: Vec<CentralLightLink>,
}

/// Loading, selection or admission failure for CentralLight link inspection.
#[derive(Debug)]
#[non_exhaustive]
pub enum CentralLightLinksError {
    Edit(EditError),
    NotPlainCentralLight { selector: String },
    NoSingleOwningDocument,
    NotPlainDocument,
    DocumentTemplate,
    NotDirectVisibleOwner,
    MissingOwnerIdentity,
    InvalidIdentity { uuid: String },
    DuplicateReference { uuid: String },
    UnresolvedControl { uuid: String },
    InvalidFlags { value: String, cause: ParseIntError },
    InconsistentMarker { identity: Option<String> },
}

impl fmt::Display for CentralLightLinksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::NotPlainCentralLight { selector } => {
                write!(f, "'{selector}' is not a plain CentralLight")
            }
            Self::NoSingleOwningDocument => {
                f.write_str("CentralLight links require one owning Document")
            }
            Self::NotPlainDocument => {
                f.write_str("CentralLight links require a plain owning Document")
            }
            Self::DocumentTemplate => {
                f.write_str("CentralLight links do not support Document Template configurations")
            }
            Self::NotDirectVisibleOwner => f.write_str(
                "CentralLight links require one direct IoData with explicit Visu=\"true\"",
            ),
            Self::MissingOwnerIdentity => f.write_str("CentralLight has no U identity"),
            Self::InvalidIdentity { uuid } => write!(
                f,
                "'{uuid}' is not a nonzero native UUID (8-4-4-16 hexadecimal digits)"
            ),
            Self::DuplicateReference { uuid } => {
                write!(f, "Duplicate linked-function UUID '{uuid}'")
            }
            Self::UnresolvedControl { uuid } => write!(
                f,
                "Linked-function UUID '{uuid}' does not resolve to exactly one plain control"
            ),
            Self::InvalidFlags { value, .. } => write!(f, "Unsupported decimal WF '{value}'"),
            Self::InconsistentMarker { identity } => write!(
                f,
                "Inconsistent document linked-function WF marker on '{}'",
                identity.as_deref().unwrap_or("(no U)")
            ),
        }
    }
}

impl Error for CentralLightLinksError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::InvalidFlags { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

type LinkResult<T> = Result<T, CentralLightLinksError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CentralLightLinkDirection {
    Up,
    Down,
}

/// Movement of one stored member, even when it is already at the boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CentralLightLinkMove {
    /// UUID spelling as stored in the owner's `linkC`.
    pub uuid: String,
    /// One-based positions in the stored list.
    pub from_position: usize,
    pub to_position: usize,
    pub changed: bool,
}

#[derive(Debug)]
pub struct CentralLightLinkMoveOutcome {
    pub bytes: Vec<u8>,
    pub movement: CentralLightLinkMove,
    pub inspection: CentralLightLinks,
    /// Whether a changed list received the legacy writer's Document stamp.
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CentralLightLinkMoveError {
    Edit(EditError),
    Links(CentralLightLinksError),
    UnsupportedTarget { uuid: String },
    NotLinked { uuid: String },
}

impl fmt::Display for CentralLightLinkMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::Links(error) => fmt::Display::fmt(error, f),
            Self::UnsupportedTarget { uuid } => write!(
                f,
                "Linked UUID '{uuid}' is not a direct ModbusServer target with a nonempty Title"
            ),
            Self::NotLinked { uuid } => {
                write!(
                    f,
                    "'{uuid}' is not an existing linked-function UUID on this CentralLight"
                )
            }
        }
    }
}

impl Error for CentralLightLinkMoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Links(error) => Some(error),
            _ => None,
        }
    }
}

/// Removal of one stored member, without reordering the remaining links.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CentralLightLinkRemoval {
    /// UUID spelling as stored in the owner's `linkC`.
    pub uuid: String,
    /// One-based position in the original stored list.
    pub from_position: usize,
    pub remaining_links: usize,
    /// True only when the target's last incoming generic reference was removed.
    pub reference_marker_cleared: bool,
}

#[derive(Debug)]
pub struct CentralLightLinkRemovalOutcome {
    pub bytes: Vec<u8>,
    pub removal: CentralLightLinkRemoval,
    pub inspection: CentralLightLinks,
    /// Whether the shared writer recognized and stamped the Document.
    pub document_stamped: bool,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CentralLightLinkRemovalError {
    Edit(EditError),
    Links(CentralLightLinksError),
    UnsupportedTarget {
        uuid: String,
    },
    NotLinked {
        uuid: String,
    },
    UnsupportedVersion {
        identity: Option<String>,
        version: Option<String>,
    },
    AffectedFlagsOutOfRange {
        value: u32,
    },
}

impl fmt::Display for CentralLightLinkRemovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::Links(error) => fmt::Display::fmt(error, f),
            Self::UnsupportedTarget { uuid } => write!(
                f,
                "Linked UUID '{uuid}' is not a direct ModbusServer target with a nonempty Title"
            ),
            Self::NotLinked { uuid } => {
                write!(
                    f,
                    "'{uuid}' is not an existing linked-function UUID on this CentralLight"
                )
            }
            Self::UnsupportedVersion { identity, version } => write!(
                f,
                "Document control '{}' has V '{}'; removal requires explicit decimal V >= 90",
                identity.as_deref().unwrap_or("(no U)"),
                version.as_deref().unwrap_or("(missing)")
            ),
            Self::AffectedFlagsOutOfRange { value } => write!(
                f,
                "Affected WF '{value}' is outside the admitted nonnegative signed32 format"
            ),
        }
    }
}

impl Error for CentralLightLinkRemovalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::Links(error) => Some(error),
            _ => None,
        }
    }
}

/// Inspect the stored `linkC` order of one legacy-selected standalone CentralLight.
///
/// Loading and legacy selection precede admission. The selected owner must be a
/// plain `C` of exact type `CentralLight`, within the one plain owning Document
/// (no direct `Template`), with one plain direct `IoData Visu="true"`.
/// Within the Document, UUID references are checked as nonzero 8-4-4-16 hex
/// identifiers, case-insensitively for graph resolution (unlike the legacy
/// `uuid:` selector). Every stored `linkC` reference must resolve to one plain
/// control, without duplicates within a list. A document-wide inconsistent
/// linked-function `WF` marker is refused, not repaired.
///
/// This read does not infer native rows, follow parent-owner fallbacks, apply
/// write eligibility, or alter the document. The shared legacy loader's
/// digit-name/quoted-text limitation (lox-n37w) is unchanged.
pub fn inspect_central_light_links(
    bytes: &[u8],
    selector: &str,
) -> Result<CentralLightLinks, CentralLightLinksError> {
    let (_, inspection) = State::load(bytes)
        .map_err(CentralLightLinksError::Edit)?
        .match_central_light_links(selector)?;
    Ok(inspection)
}

/// Move one existing member of a standalone CentralLight's stored `linkC`.
///
/// The selected member is a raw nonzero native UUID (case-insensitive, no
/// `uuid:` prefix); the owner uses the same legacy selector and graph admission
/// as [`inspect_central_light_links`]. Every member on that owner must resolve
/// directly to a `ModbusServer` with a nonempty stored `Title`, including at a
/// boundary. An adjacent swap preserves stored UUID spelling and membership,
/// does not update `WF`, and is reinspected before serialization.
///
/// A boundary no-op returns the exact borrowed input contents without stamping.
/// A changed list uses the shared writer and an explicit monotonic save stamp
/// when the writer recognizes the Document (root or direct child, exact type).
/// On failure no revised bytes are returned. This is not a native Config
/// validator; the shared loader limitation (lox-n37w) still applies.
pub fn move_central_light_link(
    bytes: &[u8],
    selector: &str,
    uuid: &str,
    direction: CentralLightLinkDirection,
    now: &DateTime<FixedOffset>,
) -> Result<CentralLightLinkMoveOutcome, CentralLightLinkMoveError> {
    use CentralLightLinkMoveError::Edit;

    let mut state = State::load(bytes).map_err(Edit)?;
    let (movement, inspection) = state.move_central_light_link(selector, uuid, direction)?;
    let document_stamped = movement.changed && state.stamp_document(now);
    let bytes = if movement.changed {
        state.to_bytes().map_err(Edit)?
    } else {
        bytes.to_vec()
    };
    Ok(CentralLightLinkMoveOutcome {
        bytes,
        movement,
        inspection,
        document_stamped,
    })
}

/// Remove one existing member of a standalone CentralLight's stored `linkC`.
///
/// The legacy owner selector and graph admission are the same as for
/// [`inspect_central_light_links`]. All selected-owner members must resolve to
/// direct `ModbusServer` targets with nonempty stored Titles. The selected
/// member is a raw nonzero native UUID (case-insensitive, no `uuid:` prefix).
/// Each Document control needs explicit decimal `V >= 90`. If no other
/// generic list refers to the target, only its `WF` reference bit is cleared,
/// and the affected stored `WF` must fit in nonnegative signed 32 bits. Empty
/// `linkC` and marker-derived zero `WF` are omitted; other raw bits and list
/// spelling/order are preserved. The resulting graph is validated again.
///
/// Success always removes a member; a repeat or absent member is a refusal,
/// not a no-op. Success returns canonical bytes and uses `now` for a monotonic
/// save stamp when the shared writer recognizes the Document (root or direct
/// child, exact type). Failure returns no revised bytes. This is not native
/// full-resave normalization or a general repair; the shared loader
/// limitation (lox-n37w) still applies.
pub fn remove_central_light_link(
    bytes: &[u8],
    selector: &str,
    uuid: &str,
    now: &DateTime<FixedOffset>,
) -> Result<CentralLightLinkRemovalOutcome, CentralLightLinkRemovalError> {
    use CentralLightLinkRemovalError::Edit;

    let mut state = State::load(bytes).map_err(Edit)?;
    let (removal, inspection) = state.remove_central_light_link(selector, uuid)?;
    let document_stamped = state.stamp_document(now);
    let bytes = state.to_bytes().map_err(Edit)?;
    Ok(CentralLightLinkRemovalOutcome {
        bytes,
        removal,
        inspection,
        document_stamped,
    })
}

impl State {
    #[doc(hidden)]
    pub fn move_central_light_link(
        &mut self,
        selector: &str,
        uuid: &str,
        direction: CentralLightLinkDirection,
    ) -> Result<(CentralLightLinkMove, CentralLightLinks), CentralLightLinkMoveError> {
        use CentralLightLinkMoveError as Refusal;

        let selected = identity(uuid).map_err(Refusal::Links)?;
        let (path, inspection) = self
            .match_central_light_links(selector)
            .map_err(Refusal::Links)?;
        // Native buttons use visible positions but callbacks move raw indices.
        // Requiring every member to be directly visible makes the orders agree.
        for link in &inspection.links {
            if link.control_type != "ModbusServer" || link.title.is_empty() {
                return Err(Refusal::UnsupportedTarget {
                    uuid: link.uuid.clone(),
                });
            }
        }
        let index = inspection
            .links
            .iter()
            .position(|link| link.uuid.eq_ignore_ascii_case(&selected))
            .ok_or_else(|| Refusal::NotLinked { uuid: uuid.into() })?;
        let destination = match direction {
            CentralLightLinkDirection::Up => index.saturating_sub(1),
            CentralLightLinkDirection::Down => (index + 1).min(inspection.links.len() - 1),
        };
        let movement = CentralLightLinkMove {
            uuid: inspection.links[index].uuid.clone(),
            from_position: index + 1,
            to_position: destination + 1,
            changed: index != destination,
        };
        if !movement.changed {
            return Ok((movement, inspection));
        }
        let mut order: Vec<_> = inspection
            .links
            .iter()
            .map(|link| link.uuid.as_str())
            .collect();
        order.swap(index, destination);
        let previous = self
            .get_element_mut(&path)
            .attributes
            .insert("linkC".into(), order.join(","));
        match self.match_central_light_links(selector) {
            Ok((_, inspection)) => Ok((movement, inspection)),
            Err(error) => {
                let attrs = &mut self.get_element_mut(&path).attributes;
                if let Some(previous) = previous {
                    attrs.insert("linkC".into(), previous);
                } else {
                    attrs.shift_remove("linkC");
                }
                Err(Refusal::Links(error))
            }
        }
    }

    #[doc(hidden)]
    pub fn remove_central_light_link(
        &mut self,
        selector: &str,
        uuid: &str,
    ) -> Result<(CentralLightLinkRemoval, CentralLightLinks), CentralLightLinkRemovalError> {
        use CentralLightLinkRemovalError as Refusal;

        let selected = identity(uuid).map_err(Refusal::Links)?;
        let (path, inspection) = self
            .match_central_light_links(selector)
            .map_err(Refusal::Links)?;
        for link in &inspection.links {
            if link.control_type != "ModbusServer" || link.title.is_empty() {
                return Err(Refusal::UnsupportedTarget {
                    uuid: link.uuid.clone(),
                });
            }
        }
        let index = inspection
            .links
            .iter()
            .position(|link| link.uuid.eq_ignore_ascii_case(&selected))
            .ok_or_else(|| Refusal::NotLinked { uuid: uuid.into() })?;

        let owner = self.get_element(&path);
        let documents = self.find_elements("Type:Document");
        let mut controls = Vec::new();
        collect_controls(self.get_element(&documents[0]), &mut controls);
        let mut other_incoming = false;
        let mut target = None;
        for control in controls {
            let version = control.attributes.get("V");
            let stored_uuid = control.attributes.get("U");
            if !version
                .and_then(|raw| raw.parse::<i32>().ok())
                .is_some_and(|version| version >= 90)
            {
                return Err(Refusal::UnsupportedVersion {
                    identity: stored_uuid.cloned(),
                    version: version.cloned(),
                });
            }
            if stored_uuid.is_some_and(|raw| raw.eq_ignore_ascii_case(&selected)) {
                target = Some(control);
            }
            for link in links(control).map_err(Refusal::Links)? {
                if !std::ptr::eq(control, owner) && link.eq_ignore_ascii_case(&selected) {
                    other_incoming = true;
                }
            }
        }
        let marker = if other_incoming {
            None
        } else {
            // Admission proves unique resolution and correct prior membership,
            // so the only possible delta is the bit on this selected target.
            let target = target.ok_or_else(|| {
                Refusal::Links(CentralLightLinksError::UnresolvedControl {
                    uuid: selected.clone(),
                })
            })?;
            let before = flags(target).map_err(Refusal::Links)?;
            if before > i32::MAX as u32 {
                return Err(Refusal::AffectedFlagsOutOfRange { value: before });
            }
            // The legacy exact-U selector can choose an earlier mirror. The
            // post-edit graph check must reject that case rather than redirect.
            let marker_path = self
                .require_one(&format!("uuid:{}", target.attributes["U"]))
                .map_err(|error| Refusal::Edit(error.into()))?;
            Some((marker_path, before & !LINKED))
        };
        let removal = CentralLightLinkRemoval {
            uuid: inspection.links[index].uuid.clone(),
            from_position: index + 1,
            remaining_links: inspection.links.len() - 1,
            reference_marker_cleared: marker.is_some(),
        };
        let mut remaining: Vec<_> = inspection.links.into_iter().map(|link| link.uuid).collect();
        remaining.remove(index);
        // Native save omits both an empty generic list and a zero WF word.
        let attributes = &mut self.get_element_mut(&path).attributes;
        let previous_owner = attributes.clone();
        if remaining.is_empty() {
            attributes.shift_remove("linkC");
        } else {
            attributes.insert("linkC".into(), remaining.join(","));
        }
        let previous_marker = marker.map(|(target, after)| {
            let attributes = &mut self.get_element_mut(&target).attributes;
            let previous = attributes.clone();
            if after == 0 {
                attributes.shift_remove("WF");
            } else {
                attributes.insert("WF".into(), after.to_string());
            }
            (target, previous)
        });

        match self.match_central_light_links(selector) {
            Ok((_, inspection)) => Ok((removal, inspection)),
            Err(error) => {
                self.get_element_mut(&path).attributes = previous_owner;
                if let Some((target, previous)) = previous_marker {
                    self.get_element_mut(&target).attributes = previous;
                }
                Err(Refusal::Links(error))
            }
        }
    }

    #[doc(hidden)]
    pub fn match_central_light_links(
        &self,
        selector: &str,
    ) -> LinkResult<(Vec<usize>, CentralLightLinks)> {
        use CentralLightLinksError as Refusal;

        let path = self
            .require_one(selector)
            .map_err(|error| Refusal::Edit(error.into()))?;
        let owner = self.get_element(&path);
        if owner.name != "C"
            || owner.attributes.get("Type").map(String::as_str) != Some("CentralLight")
            || owner.namespace.is_some()
            || owner.prefix.is_some()
        {
            return Err(Refusal::NotPlainCentralLight {
                selector: selector.into(),
            });
        }
        let documents = self.find_elements("Type:Document");
        if documents.len() != 1 || !path.starts_with(&documents[0]) {
            return Err(Refusal::NoSingleOwningDocument);
        }
        let document = self.get_element(&documents[0]);
        if document.name != "C" || document.namespace.is_some() || document.prefix.is_some() {
            return Err(Refusal::NotPlainDocument);
        }
        if document
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .any(|child| child.name == "Template")
        {
            return Err(Refusal::DocumentTemplate);
        }
        let io: Vec<_> = owner
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .filter(|child| child.name == "IoData")
            .collect();
        if io.len() != 1
            || io[0].attributes.get("Visu").map(String::as_str) != Some("true")
            || io[0].namespace.is_some()
            || io[0].prefix.is_some()
        {
            return Err(Refusal::NotDirectVisibleOwner);
        }
        let uuid = owner
            .attributes
            .get("U")
            .ok_or(Refusal::MissingOwnerIdentity)?;
        let owner_key = identity(uuid)?;
        let mut controls = Vec::new();
        collect_controls(document, &mut controls);
        let mut by_uuid: HashMap<String, Vec<&Element>> = HashMap::new();
        for control in &controls {
            if let Some(key) = control
                .attributes
                .get("U")
                .and_then(|raw| identity(raw).ok())
            {
                by_uuid.entry(key).or_default().push(control);
            }
        }
        resolve(&by_uuid, &owner_key)?;
        let mut referenced = HashSet::new();
        for control in &controls {
            for link in links(control)? {
                let key = identity(link)?;
                resolve(&by_uuid, &key)?;
                referenced.insert(key);
            }
        }
        // Native accept recomputes this marker document-wide; require a graph
        // needing no incidental repair before using the stored projection.
        for control in &controls {
            let marked = flags(control)? & LINKED != 0;
            let expected = control
                .attributes
                .get("U")
                .and_then(|raw| identity(raw).ok())
                .is_some_and(|key| referenced.contains(&key));
            if marked != expected {
                return Err(Refusal::InconsistentMarker {
                    identity: control.attributes.get("U").cloned(),
                });
            }
        }
        let mut result = Vec::new();
        for (index, link) in links(owner)?.into_iter().enumerate() {
            let control = resolve(&by_uuid, &identity(link)?)?;
            result.push(CentralLightLink {
                position: index + 1,
                uuid: link.into(),
                title: control.attributes.get("Title").cloned().unwrap_or_default(),
                control_type: control.attributes.get("Type").cloned().unwrap_or_default(),
            });
        }
        Ok((
            path,
            CentralLightLinks {
                uuid: uuid.clone(),
                title: owner.attributes.get("Title").cloned().unwrap_or_default(),
                links: result,
            },
        ))
    }
}

fn collect_controls<'a>(element: &'a Element, controls: &mut Vec<&'a Element>) {
    if element.name == "C" {
        controls.push(element);
    }
    for child in element
        .children
        .iter()
        .filter_map(|child| child.as_element())
    {
        collect_controls(child, controls);
    }
}

fn resolve<'a>(controls: &HashMap<String, Vec<&'a Element>>, key: &str) -> LinkResult<&'a Element> {
    match controls.get(key).map(Vec::as_slice) {
        Some([control]) if control.namespace.is_none() && control.prefix.is_none() => Ok(control),
        _ => Err(CentralLightLinksError::UnresolvedControl { uuid: key.into() }),
    }
}

fn links(control: &Element) -> LinkResult<Vec<&str>> {
    let raw = control.attributes.get("linkC").map_or("", String::as_str);
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for link in raw.split(',') {
        if !seen.insert(identity(link)?) {
            return Err(CentralLightLinksError::DuplicateReference { uuid: link.into() });
        }
        result.push(link);
    }
    Ok(result)
}

fn flags(control: &Element) -> LinkResult<u32> {
    match control.attributes.get("WF") {
        None => Ok(0),
        Some(raw) => raw
            .parse()
            .map_err(|cause| CentralLightLinksError::InvalidFlags {
                value: raw.clone(),
                cause,
            }),
    }
}

fn identity(raw: &str) -> LinkResult<String> {
    if raw.len() == 35
        && raw.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
        && raw.bytes().any(|byte| byte != b'0' && byte != b'-')
    {
        Ok(raw.to_ascii_lowercase())
    } else {
        Err(CentralLightLinksError::InvalidIdentity { uuid: raw.into() })
    }
}
