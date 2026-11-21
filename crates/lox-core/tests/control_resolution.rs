use lox_core::control::{
    Control, ControlQuery, IdentifierError, PreparedControlQuery, ResolutionError, ResolvedControl,
    validate_uuid,
};
use std::collections::HashMap;

fn control(name: &str, uuid: &str, room: Option<&str>) -> Control {
    Control {
        name: name.into(),
        uuid: uuid.into(),
        typ: "Switch".into(),
        room: room.map(str::to_owned),
        cat: None,
        is_favorite: false,
        is_secured: false,
    }
}

fn resolve(
    query: &str,
    room: Option<&str>,
    controls: &[Control],
) -> Result<ResolvedControl, ResolutionError> {
    match ControlQuery::prepare(query, room, &HashMap::new()) {
        PreparedControlQuery::Resolved(resolved) => Ok(resolved),
        PreparedControlQuery::Catalog(query) => query.resolve(controls),
    }
}

#[test]
fn identifiers_and_raw_case_sensitive_aliases_need_no_catalog() {
    let mut aliases = HashMap::from([("Light [Room]".into(), "../opaque".into())]);
    for id in [
        "1fbc668c-005c-7471-ffffed57184a04d2",
        "1fbc668c-005c-7471-ffff-ed57184a04d2",
        "---------------------",
        &"-".repeat(51),
    ] {
        aliases.insert(id.into(), "must-not-win".into());
        assert_eq!(
            ControlQuery::prepare(id, Some("ignored"), &aliases),
            PreparedControlQuery::Resolved(ResolvedControl { uuid: id.into() })
        );
    }
    assert_eq!(
        ControlQuery::prepare("Light [Room]", Some("ignored"), &aliases),
        PreparedControlQuery::Resolved(ResolvedControl {
            uuid: "../opaque".into()
        })
    );
    let PreparedControlQuery::Catalog(query) =
        ControlQuery::prepare("light [Room]", None, &aliases)
    else {
        panic!("alias keys must remain case-sensitive");
    };
    assert_eq!(
        query
            .resolve(&[control("LIGHT", "opaque-id", Some("ROOM"))])
            .unwrap()
            .uuid,
        "opaque-id"
    );
}

#[test]
fn ambiguity_preserves_caller_order_and_has_no_exact_name_preference() {
    let controls = [
        control("Light Extra", "opaque-extra", Some("Room")),
        control("Light", "opaque-exact", None),
    ];
    assert_eq!(
        resolve("Light", None, &controls),
        Err(ResolutionError::Ambiguous {
            query: "Light".into(),
            candidates: controls.to_vec(),
        })
    );
    assert_eq!(
        resolve("Missing", None, &controls),
        Err(ResolutionError::NotFound {
            query: "Missing".into()
        })
    );
}

#[test]
fn inline_room_overrides_flag_with_literal_spaces_and_rightmost_bracket() {
    let controls = [
        control("Licht", "native-room", Some("KÜCHE")),
        control("Licht Extra", "spaced-room", Some(" Küche ")),
        control("Panel [West]", "nested-name", Some("KÜCHE")),
        control("Other", "roomless", None),
        control("Open [", "unclosed", None),
    ];
    for (query, room, expected) in [
        (" Licht [ küche ]", Some("missing"), "spaced-room"),
        ("Panel [West] [küche]", Some("missing"), "nested-name"),
        ("", Some(" küche "), "spaced-room"),
        ("Other []", Some("missing"), "roomless"),
        ("Open [", None, "unclosed"),
    ] {
        assert_eq!(resolve(query, room, &controls).unwrap().uuid, expected);
    }
    assert!(matches!(
        resolve(" Licht ", None, &controls),
        Err(ResolutionError::NotFound { .. })
    ));
}

#[test]
fn path_validation_remains_distinct_from_identifier_detection_and_aliases() {
    for value in ["a", "-", "1fbc668c-005c-7471-ffffed57184a04d2"] {
        validate_uuid(value).unwrap();
    }
    for (value, error) in [
        ("", IdentifierError::InvalidLength),
        (&"-".repeat(51), IdentifierError::InvalidLength),
        ("../bad", IdentifierError::InvalidCharacters),
        ("a/b", IdentifierError::InvalidCharacters),
        ("a\\b", IdentifierError::InvalidCharacters),
        ("a%2fb", IdentifierError::InvalidCharacters),
        ("has spaces", IdentifierError::NonHex),
        ("opaque-id", IdentifierError::NonHex),
    ] {
        assert_eq!(validate_uuid(value), Err(error));
    }
}
