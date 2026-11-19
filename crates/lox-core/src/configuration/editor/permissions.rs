use super::{EditError, legacy::State};
use std::{collections::HashMap, error::Error, fmt};
use xmltree::Element;

/// Exact `C Type` of the selected permission principal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalKind {
    User,
    UserGroup,
}

impl PrincipalKind {
    /// The exact XML type spelling.
    pub fn xml_type(self) -> &'static str {
        match self {
            Self::User => "User",
            Self::UserGroup => "UserGroup",
        }
    }

    /// A human-readable name for this kind.
    pub fn noun(self) -> &'static str {
        match self {
            Self::User => "User",
            Self::UserGroup => "User group",
        }
    }
}

/// Identity of the selected principal; absent `Title` or `U` is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionPrincipal {
    pub kind: PrincipalKind,
    pub title: String,
    pub uuid: String,
}

/// One direct `PRM`. These fields are uninterpreted, not an access decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectPermission {
    /// Raw `REF`, distinct from an explicitly empty attribute.
    pub reference: Option<String>,
    /// `Title` from the first `C` with the exact lookup value as `U`, if present.
    pub title: Option<String>,
    /// `Type` from the same `C`, if present.
    pub element_type: Option<String>,
    pub act: Option<String>,
    pub tim: Option<String>,
}

/// The selected principal and its direct entries in document order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrincipalPermissions {
    pub principal: PermissionPrincipal,
    pub permissions: Vec<DirectPermission>,
}

/// Why an exact, kind-scoped permission principal selection was refused.
#[derive(Debug)]
pub enum PrincipalSelectionError {
    NotFound {
        kind: PrincipalKind,
        selector: String,
        /// Available titles of this kind, sorted and deduplicated.
        titles: Vec<String>,
    },
    Ambiguous {
        kind: PrincipalKind,
        selector: String,
        /// Matching `U` values in document order, empty where absent.
        uuids: Vec<String>,
    },
}

impl fmt::Display for PrincipalSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { kind, selector, .. } => {
                write!(f, "{} {selector:?} not found", kind.noun())
            }
            Self::Ambiguous {
                kind,
                selector,
                uuids,
            } => write!(
                f,
                "{} {selector:?} matches {} elements (expected 1)",
                kind.noun(),
                uuids.len()
            ),
        }
    }
}

impl Error for PrincipalSelectionError {}

/// A configuration load failure or a refusal to select one principal.
#[derive(Debug)]
pub enum PrincipalPermissionsError {
    Load(EditError),
    Selection(PrincipalSelectionError),
}

impl fmt::Display for PrincipalPermissionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => fmt::Display::fmt(error, f),
            Self::Selection(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl Error for PrincipalPermissionsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::Selection(error) => Some(error),
        }
    }
}

/// Read one principal's direct permissions from caller-supplied configuration XML.
///
/// Selects exactly one `C` of `kind` by case-insensitive exact title or a
/// lowercase `uuid:` prefix followed by an exact case-sensitive `U`.
/// Missing or duplicate matches are errors; matches are not deduplicated.
/// Only direct `PRM` children of the first direct `PH` are returned. `NUM`
/// is not used. For legacy metadata lookup only, absent `REF` acts as empty.
/// The shared loader's quoted-text/digit-name limitation (lox-n37w) is unchanged.
/// No document edits, inherited grants, deny evaluation or access decisions occur.
pub fn principal_permissions(
    bytes: &[u8],
    selector: &str,
    kind: PrincipalKind,
) -> Result<PrincipalPermissions, PrincipalPermissionsError> {
    let state = State::load(bytes).map_err(PrincipalPermissionsError::Load)?;
    let path = state
        .find_principal(selector, kind)
        .map_err(PrincipalPermissionsError::Selection)?;
    Ok(state.direct_permissions_at(&path, kind, &state.permission_index()))
}

type PermissionIndex = HashMap<String, (Option<String>, Option<String>)>;

impl State {
    pub fn principal_paths(&self, kind: PrincipalKind) -> Vec<Vec<usize>> {
        fn walk(
            elem: &Element,
            kind: PrincipalKind,
            path: &mut Vec<usize>,
            out: &mut Vec<Vec<usize>>,
        ) {
            if elem.name == "C"
                && elem
                    .attributes
                    .get("Type")
                    .is_some_and(|t| t == kind.xml_type())
            {
                out.push(path.clone());
            }
            for (i, child) in elem.children.iter().enumerate() {
                if let Some(e) = child.as_element() {
                    path.push(i);
                    walk(e, kind, path, out);
                    path.pop();
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.root, kind, &mut Vec::new(), &mut out);
        out
    }

    pub fn find_principal(
        &self,
        selector: &str,
        kind: PrincipalKind,
    ) -> Result<Vec<usize>, PrincipalSelectionError> {
        let all = self.principal_paths(kind);
        let wanted_uuid = selector.strip_prefix("uuid:");
        let wanted_lower = selector.to_lowercase();
        let matches: Vec<_> = all
            .iter()
            .filter(|path| {
                let e = self.get_element(path);
                match wanted_uuid {
                    Some(u) => e.attributes.get("U").is_some_and(|x| x == u),
                    None => e
                        .attributes
                        .get("Title")
                        .is_some_and(|t| t.to_lowercase() == wanted_lower),
                }
            })
            .cloned()
            .collect();

        match matches.len() {
            0 => {
                let mut titles: Vec<_> = all
                    .iter()
                    .filter_map(|path| self.get_element(path).attributes.get("Title").cloned())
                    .collect();
                titles.sort();
                titles.dedup();
                Err(PrincipalSelectionError::NotFound {
                    kind,
                    selector: selector.to_owned(),
                    titles,
                })
            }
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(PrincipalSelectionError::Ambiguous {
                kind,
                selector: selector.to_owned(),
                uuids: matches
                    .iter()
                    .map(|path| {
                        self.get_element(path)
                            .attributes
                            .get("U")
                            .cloned()
                            .unwrap_or_default()
                    })
                    .collect(),
            }),
        }
    }

    pub fn permission_principal(&self, path: &[usize], kind: PrincipalKind) -> PermissionPrincipal {
        let e = self.get_element(path);
        PermissionPrincipal {
            kind,
            title: e.attributes.get("Title").cloned().unwrap_or_default(),
            uuid: e.attributes.get("U").cloned().unwrap_or_default(),
        }
    }

    pub fn permission_index(&self) -> PermissionIndex {
        fn walk(elem: &Element, map: &mut PermissionIndex) {
            if elem.name == "C"
                && let Some(u) = elem.attributes.get("U")
            {
                map.entry(u.clone()).or_insert_with(|| {
                    (
                        elem.attributes.get("Type").cloned(),
                        elem.attributes.get("Title").cloned(),
                    )
                });
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    walk(e, map);
                }
            }
        }
        let mut map = HashMap::new();
        walk(&self.root, &mut map);
        map
    }

    pub fn permission_block_index(elem: &Element) -> Option<usize> {
        elem.children
            .iter()
            .position(|c| c.as_element().is_some_and(|e| e.name == "PH"))
    }

    pub fn direct_permissions_at(
        &self,
        path: &[usize],
        kind: PrincipalKind,
        index: &PermissionIndex,
    ) -> PrincipalPermissions {
        let principal = self.permission_principal(path, kind);
        let elem = self.get_element(path);
        let mut permissions = Vec::new();
        if let Some(ph) =
            Self::permission_block_index(elem).and_then(|i| elem.children[i].as_element())
        {
            for prm in ph
                .children
                .iter()
                .filter_map(|c| c.as_element())
                .filter(|e| e.name == "PRM")
            {
                let reference = prm.attributes.get("REF").cloned();
                let (element_type, title) = index
                    .get(reference.as_deref().unwrap_or_default())
                    .cloned()
                    .unwrap_or((None, None));
                permissions.push(DirectPermission {
                    reference,
                    title,
                    element_type,
                    act: prm.attributes.get("ACT").cloned(),
                    tim: prm.attributes.get("TIM").cloned(),
                });
            }
        }
        PrincipalPermissions {
            principal,
            permissions,
        }
    }
}
