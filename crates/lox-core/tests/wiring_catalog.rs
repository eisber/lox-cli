#![cfg(feature = "configuration-xml")]

use lox_core::configuration::{
    editor::{EditError, SelectionError},
    wiring::{ConnectorDirection, connector_wiring},
};
use std::error::Error;

#[test]
fn wiring_catalog_retains_raw_values_order_and_direct_child_policy() {
    let bytes = br#"<C Title="Gate"><Co K="AIQ" U="00000000-0000-0000-0000000000000000"/><Co K="AQI" U="00000000-0000-0000-0000-000000000000"/><Co K="output"><Other><In/></Other></Co><Co><In/></Co><Co K="AIQ"/><Other><Co K="I"/></Other></C>"#;
    let saved = bytes.to_vec();
    let wires = connector_wiring(bytes, "Gate").unwrap();
    assert_eq!(
        wires
            .iter()
            .map(|w| (w.connector.as_str(), w.direction, w.connected))
            .collect::<Vec<_>>(),
        [
            ("AIQ", ConnectorDirection::Input, false),
            ("AQI", ConnectorDirection::Output, true),
            ("output", ConnectorDirection::Parameter, false),
            ("", ConnectorDirection::Parameter, true),
            ("AIQ", ConnectorDirection::Input, false),
        ]
    );
    assert_eq!(wires[1].target_uuid, "00000000-0000-0000-0000-000000000000");
    assert_eq!(bytes.as_slice(), saved);
}

#[test]
fn wiring_catalog_shares_mirror_selection_and_typed_load_causes() {
    let bytes = br#"<R><C U="a" Title="Gate" Type="And"><Co K="I"/></C><C U="a" Title="Gate" Type="And"><Co K="mirror"/></C></R>"#;
    assert_eq!(
        connector_wiring(bytes, "Type:And").unwrap()[0].connector,
        "I"
    );
    assert!(matches!(
        connector_wiring(bytes, "missing"),
        Err(EditError::Selection(SelectionError::NotFound { .. }))
    ));
    let error = connector_wiring(b"<C>", "").unwrap_err();
    assert!(matches!(error, EditError::Load(_)));
    assert!(error.source().unwrap().is::<xmltree::ParseError>());
    assert!(connector_wiring(b"<C Title='\xff'/>", "").is_err());
}
