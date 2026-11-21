use lox_core::configuration::editor::{self as core, PrincipalSelectionError};
use std::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum PrincipalKind {
    User,
    UserGroup,
}

impl From<PrincipalKind> for core::PrincipalKind {
    fn from(kind: PrincipalKind) -> Self {
        match kind {
            PrincipalKind::User => Self::User,
            PrincipalKind::UserGroup => Self::UserGroup,
        }
    }
}

impl From<core::PrincipalKind> for PrincipalKind {
    fn from(kind: core::PrincipalKind) -> Self {
        match kind {
            core::PrincipalKind::User => Self::User,
            core::PrincipalKind::UserGroup => Self::UserGroup,
        }
    }
}

/// One lowering transaction for XML bytes and the UTF-8 selector.
#[derive(uniffi::Record)]
pub struct PrincipalPermissionsRequest {
    pub bytes: Vec<u8>,
    pub selector: String,
    pub kind: PrincipalKind,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct PermissionPrincipal {
    pub kind: PrincipalKind,
    pub title: String,
    pub uuid: String,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct DirectPermission {
    pub reference: Option<String>,
    pub title: Option<String>,
    pub element_type: Option<String>,
    pub act: Option<String>,
    pub tim: Option<String>,
}

#[derive(Debug, PartialEq, Eq, uniffi::Record)]
pub struct PrincipalPermissions {
    pub principal: PermissionPrincipal,
    pub permissions: Vec<DirectPermission>,
}

impl From<core::PrincipalPermissions> for PrincipalPermissions {
    fn from(value: core::PrincipalPermissions) -> Self {
        Self {
            principal: PermissionPrincipal {
                kind: value.principal.kind.into(),
                title: value.principal.title,
                uuid: value.principal.uuid,
            },
            permissions: value
                .permissions
                .into_iter()
                .map(|entry| DirectPermission {
                    reference: entry.reference,
                    title: entry.title,
                    element_type: entry.element_type,
                    act: entry.act,
                    tim: entry.tim,
                })
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum PrincipalPermissionsError {
    #[error("{message}")]
    Load {
        message: String,
        source_message: Option<String>,
    },
    #[error("No {kind:?} principal matches {selector:?}")]
    NotFound {
        kind: PrincipalKind,
        selector: String,
        titles: Vec<String>,
    },
    #[error("Multiple {kind:?} principals match {selector:?}")]
    Ambiguous {
        kind: PrincipalKind,
        selector: String,
        uuids: Vec<String>,
    },
}

impl From<core::PrincipalPermissionsError> for PrincipalPermissionsError {
    fn from(error: core::PrincipalPermissionsError) -> Self {
        match error {
            core::PrincipalPermissionsError::Load(error) => Self::Load {
                message: error.to_string(),
                source_message: error.source().map(ToString::to_string),
            },
            core::PrincipalPermissionsError::Selection(PrincipalSelectionError::NotFound {
                kind,
                selector,
                titles,
            }) => Self::NotFound {
                kind: kind.into(),
                selector,
                titles,
            },
            core::PrincipalPermissionsError::Selection(PrincipalSelectionError::Ambiguous {
                kind,
                selector,
                uuids,
            }) => Self::Ambiguous {
                kind: kind.into(),
                selector,
                uuids,
            },
        }
    }
}

/// Return an owned principal and raw direct permissions in Core document order.
///
/// This shares Core's selector and loader (including lox-n37w). It does not
/// perform file I/O, edit XML, or evaluate inherited or effective access.
#[uniffi::export]
pub fn principal_permissions(
    request: PrincipalPermissionsRequest,
) -> Result<PrincipalPermissions, PrincipalPermissionsError> {
    core::principal_permissions(&request.bytes, &request.selector, request.kind.into())
        .map(Into::into)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &[u8] = include_bytes!("../tests/fixtures/principal_permissions.xml");

    fn read(
        bytes: &[u8],
        selector: &str,
        kind: PrincipalKind,
    ) -> Result<PrincipalPermissions, PrincipalPermissionsError> {
        principal_permissions(PrincipalPermissionsRequest {
            bytes: bytes.to_vec(),
            selector: selector.into(),
            kind,
        })
    }

    #[test]
    fn owns_ordered_direct_entries_without_collapsing_optional_values() {
        let value = read(XML, "SÄMPLE & 灯", PrincipalKind::User).unwrap();
        assert_eq!(value.principal.title, "Sämple & 灯");
        assert_eq!(value.principal.uuid, "user-01");
        let rows: Vec<_> = value
            .permissions
            .iter()
            .map(|entry| {
                (
                    entry.reference.as_deref(),
                    entry.title.as_deref(),
                    entry.element_type.as_deref(),
                    entry.act.as_deref(),
                    entry.tim.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            rows,
            [
                (
                    Some("target"),
                    Some("Erste & Küche"),
                    Some("Button"),
                    Some("0007"),
                    Some("opaque & 01"),
                ),
                (
                    Some("target"),
                    Some("Erste & Küche"),
                    Some("Button"),
                    Some(""),
                    Some(""),
                ),
                (Some("Target"), None, None, Some("text"), None),
                (Some(""), Some(""), Some(""), None, None),
                (None, Some(""), Some(""), None, None),
            ]
        );
        assert_eq!(
            read(XML, "uuid:user-01", PrincipalKind::User).unwrap(),
            value
        );
        let group = read(XML, "sämple & 灯", PrincipalKind::UserGroup).unwrap();
        assert_eq!(group.principal.kind, PrincipalKind::UserGroup);
        assert_eq!(group.principal.uuid, "group-01");
        assert_eq!(group.permissions[0].tim.as_deref(), Some("01"));
        let empty = read(XML, "Empty", PrincipalKind::User).unwrap();
        assert!(empty.principal.uuid.is_empty() && empty.permissions.is_empty());
    }

    #[test]
    fn preserves_error_kind_original_selector_and_ordered_payloads() {
        assert_eq!(
            read(XML, "uuid:USER-01", PrincipalKind::User).unwrap_err(),
            PrincipalPermissionsError::NotFound {
                kind: PrincipalKind::User,
                selector: "uuid:USER-01".into(),
                titles: ["Duplicate", "Empty", "Sämple & 灯", "duplicate"]
                    .map(str::to_owned)
                    .to_vec(),
            }
        );
        assert_eq!(
            read(XML, "dUpLiCaTe", PrincipalKind::User).unwrap_err(),
            PrincipalPermissionsError::Ambiguous {
                kind: PrincipalKind::User,
                selector: "dUpLiCaTe".into(),
                uuids: ["z", "", "z"].map(str::to_owned).to_vec(),
            }
        );
        let malformed = b"<C A='1' A='2'/>";
        let core::PrincipalPermissionsError::Load(error) =
            core::principal_permissions(malformed, "missing", core::PrincipalKind::UserGroup)
                .unwrap_err()
        else {
            panic!("expected Core load error before selection");
        };
        assert_eq!(
            read(malformed, "missing", PrincipalKind::UserGroup).unwrap_err(),
            PrincipalPermissionsError::Load {
                message: error.to_string(),
                source_message: Some(error.source().unwrap().to_string()),
            }
        );
    }
}
