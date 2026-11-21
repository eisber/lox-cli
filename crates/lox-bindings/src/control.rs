//! Owned native values around the core's borrowed, two-stage control query.

use lox_core::control as core;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Control {
    pub name: String,
    pub uuid: String,
    pub typ: String,
    pub room: Option<String>,
    pub cat: Option<String>,
    pub is_favorite: bool,
    pub is_secured: bool,
}

impl From<core::Control> for Control {
    fn from(value: core::Control) -> Self {
        Self {
            name: value.name,
            uuid: value.uuid,
            typ: value.typ,
            room: value.room,
            cat: value.cat,
            is_favorite: value.is_favorite,
            is_secured: value.is_secured,
        }
    }
}

impl From<Control> for core::Control {
    fn from(value: Control) -> Self {
        Self {
            name: value.name,
            uuid: value.uuid,
            typ: value.typ,
            room: value.room,
            cat: value.cat,
            is_favorite: value.is_favorite,
            is_secured: value.is_secured,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ResolvedControl {
    pub uuid: String,
}

impl From<core::ResolvedControl> for ResolvedControl {
    fn from(value: core::ResolvedControl) -> Self {
        Self { uuid: value.uuid }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum ControlResolutionError {
    #[error("control query {query:?} matched no controls")]
    NotFound { query: String },
    #[error("control query {query:?} matched {} controls", candidates.len())]
    Ambiguous {
        query: String,
        candidates: Vec<Control>,
    },
}

impl From<core::ResolutionError> for ControlResolutionError {
    fn from(value: core::ResolutionError) -> Self {
        match value {
            core::ResolutionError::NotFound { query } => Self::NotFound { query },
            core::ResolutionError::Ambiguous { query, candidates } => Self::Ambiguous {
                query,
                candidates: candidates.into_iter().map(Into::into).collect(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum IdentifierError {
    #[error("identifier length must be between 1 and 50 bytes")]
    InvalidLength,
    #[error("identifier contains unsafe path characters")]
    InvalidCharacters,
    #[error("identifier contains non-hex characters")]
    NonHex,
}

impl From<core::IdentifierError> for IdentifierError {
    fn from(value: core::IdentifierError) -> Self {
        match value {
            core::IdentifierError::InvalidLength => Self::InvalidLength,
            core::IdentifierError::InvalidCharacters => Self::InvalidCharacters,
            core::IdentifierError::NonHex => Self::NonHex,
        }
    }
}

/// A resolved opaque ID or an owned request snapshot, with no native handle.
/// Catalog snapshots retain the original input, not a second parsed query.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum PreparedControlQuery {
    Resolved {
        control: ResolvedControl,
    },
    Catalog {
        query: String,
        room_filter: Option<String>,
    },
}

/// Prepare before acquiring a catalog. Alias values are opaque, not validated.
#[uniffi::export]
pub fn prepare_control_query(
    query: String,
    room_filter: Option<String>,
    aliases: HashMap<String, String>,
) -> PreparedControlQuery {
    match core::ControlQuery::prepare(&query, room_filter.as_deref(), &aliases) {
        core::PreparedControlQuery::Resolved(control) => PreparedControlQuery::Resolved {
            control: control.into(),
        },
        core::PreparedControlQuery::Catalog(_) => {
            PreparedControlQuery::Catalog { query, room_filter }
        }
    }
}

/// Complete a prepared query against a caller-owned, ordered catalog.
#[uniffi::export]
pub fn resolve_control_query(
    prepared: PreparedControlQuery,
    controls: Vec<Control>,
) -> Result<ResolvedControl, ControlResolutionError> {
    let (query, room_filter) = match prepared {
        PreparedControlQuery::Resolved { control } => return Ok(control),
        PreparedControlQuery::Catalog { query, room_filter } => (query, room_filter),
    };
    // Reborrow the owned snapshot; parsing and matching still belong to core.
    // A caller-constructed Catalog variant is also interpreted by core.
    match core::ControlQuery::prepare(&query, room_filter.as_deref(), &HashMap::new()) {
        core::PreparedControlQuery::Resolved(control) => Ok(control.into()),
        core::PreparedControlQuery::Catalog(query) => query
            .resolve(&controls.into_iter().map(Into::into).collect::<Vec<_>>())
            .map(Into::into)
            .map_err(Into::into),
    }
}

/// Legacy identifier detection, deliberately distinct from path validation.
#[uniffi::export]
pub fn is_uuid(value: String) -> bool {
    core::is_uuid(&value)
}

#[uniffi::export]
pub fn validate_uuid(value: String) -> Result<(), IdentifierError> {
    core::validate_uuid(&value).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control(name: &str, uuid: &str, room: Option<&str>) -> Control {
        Control {
            name: name.into(),
            uuid: uuid.into(),
            typ: "Switch".into(),
            room: room.map(str::to_owned),
            cat: Some("Category".into()),
            is_favorite: true,
            is_secured: false,
        }
    }

    #[test]
    fn bypasses_keep_raw_values_and_need_no_catalog() {
        let long = "-".repeat(51);
        for query in ["---------------------", &long] {
            let prepared = prepare_control_query(
                query.into(),
                Some("ignored".into()),
                HashMap::from([(query.into(), "must-not-win".into())]),
            );
            assert!(matches!(prepared, PreparedControlQuery::Resolved { .. }));
            assert_eq!(resolve_control_query(prepared, vec![]).unwrap().uuid, query);
        }
        let prepared = prepare_control_query(
            "Alias [Room]".into(),
            Some("ignored".into()),
            HashMap::from([
                ("Alias [Room]".into(), "../raw-alias".into()),
                ("../raw-alias".into(), "must-not-recurse".into()),
            ]),
        );
        assert_eq!(
            resolve_control_query(prepared.clone(), vec![])
                .unwrap()
                .uuid,
            "../raw-alias"
        );
        assert_eq!(
            resolve_control_query(prepared, vec![]).unwrap().uuid,
            "../raw-alias"
        );
    }

    #[test]
    fn typed_errors_preserve_original_query_and_every_ordered_candidate_field() {
        let mut controls = vec![
            control("Licht Extra", "opaque-first", Some("KÜCHE")),
            control("Licht", "opaque-exact", None),
        ];
        controls[1].cat = None;
        controls[1].is_favorite = false;
        controls[1].is_secured = true;
        let prepared = prepare_control_query("Licht".into(), None, HashMap::new());
        assert_eq!(
            resolve_control_query(prepared, controls.clone()),
            Err(ControlResolutionError::Ambiguous {
                query: "Licht".into(),
                candidates: controls,
            })
        );
        let original = " Missing [ Küche ]";
        assert_eq!(
            resolve_control_query(
                prepare_control_query(original.into(), None, HashMap::new()),
                vec![]
            ),
            Err(ControlResolutionError::NotFound {
                query: original.into(),
            })
        );
    }

    #[test]
    fn owned_snapshot_reuses_the_core_parser_and_matching() {
        let controls = vec![
            control("Panel [West]", "nested", Some("KÜCHE")),
            control("Licht", "spaced", Some(" Küche ")),
            control("Other", "roomless", None),
        ];
        for (query, expected) in [
            (" Panel [West] [küche]", "nested"),
            (" Licht [ küche ]", "spaced"),
            ("Other []", "roomless"),
        ] {
            let prepared = prepare_control_query(
                query.into(),
                Some("ignored".into()),
                HashMap::from([("alias".into(), "unused".into())]),
            );
            assert_eq!(
                resolve_control_query(prepared, controls.clone())
                    .unwrap()
                    .uuid,
                expected
            );
        }
        let prepared = prepare_control_query(" Licht ".into(), None, HashMap::new());
        assert!(matches!(
            resolve_control_query(prepared, controls),
            Err(ControlResolutionError::NotFound { .. })
        ));
    }

    #[test]
    fn identifier_errors_are_exhaustive_and_not_resolution_policy() {
        for value in ["a", "-", &"-".repeat(50)] {
            validate_uuid(value.into()).unwrap();
        }
        for (value, expected) in [
            ("".to_owned(), IdentifierError::InvalidLength),
            ("-".repeat(51), IdentifierError::InvalidLength),
            ("é".repeat(26), IdentifierError::InvalidLength),
            ("..".into(), IdentifierError::InvalidCharacters),
            ("a/b".into(), IdentifierError::InvalidCharacters),
            ("a\\b".into(), IdentifierError::InvalidCharacters),
            ("a%b".into(), IdentifierError::InvalidCharacters),
            ("é".into(), IdentifierError::NonHex),
            ("a.b".into(), IdentifierError::NonHex),
        ] {
            assert_eq!(validate_uuid(value), Err(expected));
        }
        assert!(!is_uuid("a".into()));
        assert!(is_uuid("-".repeat(51)));
    }
}
