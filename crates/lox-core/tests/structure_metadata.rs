#![cfg(feature = "structure-json")]

use lox_core::structure_metadata::{
    Category, GlobalState, MetadataDecodeError, OperatingMode, StructureMetadata,
};

#[test]
fn permissive_roots_maps_and_typed_entries() {
    for bytes in [
        "null",
        "true",
        "3",
        "\"text\"",
        "[]",
        "{}",
        r#"{"cats":[],"globalStates":false,"operatingModes":null}"#,
        r#"{"cats":7,"globalStates":[],"operatingModes":"text"}"#,
    ] {
        let metadata = StructureMetadata::from_loxapp3_json(bytes.as_bytes()).unwrap();
        assert!(metadata.categories().is_empty());
        assert!(metadata.global_states().is_empty());
        assert!(metadata.operating_modes().is_empty());
    }
    let metadata = StructureMetadata::from_loxapp3_json(
        br#"{
        "cats":{"raw / id":{"name":""},"a":null,"b":[],"c":"x","d":{"name":7}},
        "globalStates":{" raw name ":"","a":null,"b":true,"c":{},"d":[],"e":7},
        "operatingModes":{"raw / id":"","a":null,"b":true,"c":{},"d":[],"e":7}
    }"#,
    )
    .unwrap();
    assert_eq!(
        metadata.categories(),
        &[Category {
            uuid: "raw / id".into(),
            name: "".into()
        }]
    );
    assert_eq!(
        metadata.global_states(),
        &[GlobalState {
            name: " raw name ".into(),
            uuid: "".into()
        }]
    );
    assert_eq!(
        metadata.operating_modes(),
        &[OperatingMode {
            raw_id: "raw / id".into(),
            name: "".into()
        }]
    );
}

#[test]
fn stable_ties_survive_the_cached_value_byte_bridge() {
    let value: serde_json::Value = serde_json::from_str(
        r#"{
        "cats":{"z":{"name":"same"},"a":{"name":"same"},"u":{"name":"Ä"},"c":{"name":"Z"}},
        "globalStates":{"ä":"a","z":"z","A":"raw","":""},
        "operatingModes":{"10":"ten","-2":"negative","2":"two","+1":"plus",
            "":"","+0":"plus zero","-0":"minus zero","00":"leading",
            " 0":"space","2147483648":"overflow","-2147483649":"underflow","x":"invalid"}
    }"#,
    )
    .unwrap();
    let ties: Vec<_> = value["cats"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|key| matches!(key.as_str(), "a" | "z"))
        .cloned()
        .collect();
    let zeros: Vec<_> = value["operatingModes"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|key| !matches!(key.as_str(), "-2" | "+1" | "2" | "10"))
        .cloned()
        .collect();
    let metadata =
        StructureMetadata::from_loxapp3_json(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        metadata
            .categories()
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["Z", "same", "same", "Ä"]
    );
    assert_eq!(
        metadata.categories()[1..3]
            .iter()
            .map(|c| c.uuid.clone())
            .collect::<Vec<_>>(),
        ties
    );
    assert_eq!(
        metadata
            .global_states()
            .iter()
            .map(|g| g.name.as_str())
            .collect::<Vec<_>>(),
        ["", "A", "z", "ä"]
    );
    let modes = metadata.operating_modes();
    assert_eq!(modes[0].raw_id, "-2");
    assert_eq!(
        modes[1..1 + zeros.len()]
            .iter()
            .map(|m| m.raw_id.clone())
            .collect::<Vec<_>>(),
        zeros
    );
    assert_eq!(
        modes[1 + zeros.len()..]
            .iter()
            .map(|m| m.raw_id.as_str())
            .collect::<Vec<_>>(),
        ["+1", "2", "10"]
    );
}

#[test]
fn invalid_json_has_structured_payload_free_location() {
    let error = StructureMetadata::from_loxapp3_json(b"{\n\"private secret\": ]}").unwrap_err();
    assert_eq!(
        error,
        MetadataDecodeError::InvalidJson {
            line: 2,
            column: 19
        }
    );
    assert_eq!(
        error.to_string(),
        "invalid LoxApp3 JSON at line 2 column 19"
    );
    assert!(!format!("{error:?}").contains("private"));
    assert!(StructureMetadata::from_loxapp3_json(&[0xff]).is_err());
}
