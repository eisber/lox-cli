use super::ConfigEditor;

const FIXTURE: &[u8] = include_bytes!("../../tests/ux/light-controller2/synthetic.Loxone");

#[test]
fn lighting_inspection_exposes_all_stored_outputs_without_defaults_or_roles() {
    let editor = ConfigEditor::load(FIXTURE).unwrap();
    let before = editor.to_bytes().unwrap();
    let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
    assert_eq!(editor.to_bytes().unwrap(), before);
    assert_eq!(
        inspection
            .channels
            .iter()
            .map(|channel| channel.index)
            .collect::<Vec<_>>(),
        [1, 2, 10]
    );
    assert_eq!(
        inspection.moods[0]
            .outputs
            .iter()
            .map(|output| output.key.as_str())
            .collect::<Vec<_>>(),
        ["Q1", "Q2", "Q10"]
    );
    assert_eq!(inspection.moods[1].outputs[1].stored, "future-encoding");
    assert!(
        inspection
            .moods
            .iter()
            .flat_map(|mood| &mood.outputs)
            .all(|output| output.unit.is_none())
    );
    assert!(
        inspection
            .channels
            .iter()
            .all(|channel| channel.role.is_none())
    );
    let target = &inspection.channels[1].downstream[0];
    assert_eq!(target.block_uuid.as_deref(), Some("target"));
    assert_eq!(target.connector_uuid.as_deref(), Some("target-ai"));
    assert_eq!(target.wire_attributes["FutureWire"], "retain");
    assert_eq!(inspection.moods[0].summary.sid, Some(2));
    assert_eq!(inspection.moods[0].attributes["FutureMood"], "opaque");
    assert!(
        inspection
            .nested_storage
            .iter()
            .any(|element| element.element == "UnknownLighting")
    );
    assert_eq!(
        inspection.document_versions[1]["ConfigVersion"],
        "unverified-fixture"
    );
    assert!(!inspection.gui_verified);
}

#[test]
fn lighting_inspection_reports_bad_reference_identities_and_malformed_mood_ids() {
    let xml = std::str::from_utf8(FIXTURE)
        .unwrap()
        .replace("U=\"target-ai\"", "")
        .replace("SID=\"2\"", "SID=\"bad-id\"")
        .replace("U=\"output-10\"", "U=\"output-2\"");
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
    for code in [
        "ambiguous_connector_uuid",
        "unresolved_wire_target",
        "invalid_mood_id",
    ] {
        assert!(
            inspection
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "{code}"
        );
    }
    assert_eq!(inspection.moods[0].attributes["SID"], "bad-id");
    assert!(inspection.moods[0].summary.sid.is_none());
    assert!(
        inspection.channels[1].downstream[0]
            .connector_uuid
            .is_none()
    );
}

#[test]
fn lighting_inspection_rejects_wrong_type_and_reports_missing_storage() {
    let editor = ConfigEditor::load(FIXTURE).unwrap();
    assert!(editor.inspect_lighting("uuid:target").is_err());
    assert!(editor.inspect_lighting("uuid:missing").is_err());
    let editor = ConfigEditor::load(
        br#"<ControlList><C Type="LightController2" U="empty"><Co K="AQ1"/></C></ControlList>"#,
    )
    .unwrap();
    let inspection = editor.inspect_lighting("uuid:empty").unwrap();
    assert!(inspection.moods.is_empty());
    assert!(inspection.channels[0].uuid.is_none());
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsupported_mood_container_shape")
    );
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing_identity")
    );
}
