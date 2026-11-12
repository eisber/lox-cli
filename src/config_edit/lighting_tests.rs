use super::ConfigEditor;
use super::lighting::LightingEditorKind;

const FIXTURE: &[u8] = include_bytes!("../../tests/ux/light-controller2/synthetic.Loxone");

const LEVEL_XML: &str = r#"<ControlList><C Type="LightController2" U="lighting" Title="Lamp" CapAI0="7">
  <LightscenesC Num="2" Outputs="18" Future="retain">
    <LightsceneC Name="First" SID="2" UUID="first" Outputs="18" Q1="1640612766" Q2="1610622736" Q10="1610612736" Future="opaque">
      <SeqConf Future="keep"><Unknown Q2="nested"/></SeqConf>
    </LightsceneC>
    <LightsceneC Name="Second" SID="3" UUID="second" Outputs="18" Q2="future"/>
  </LightscenesC>
  <HCL T="2" TMIN="2000" FROM="0" TO="0"/>
</C></ControlList>"#;

#[test]
fn lighting_identity_inspection_reports_numeric_duplicates_without_repairing_raw_storage() {
    let xml = r#"<ControlList><C Type="LightController2" U="lighting"><LightscenesC Num="5">
      <LightsceneC UUID="a" SID="+0002" CID="09"/>
      <LightsceneC UUID="b" SID="2" CID="+9"/>
      <LightsceneC UUID="c" SID="3" CID="0"/>
      <LightsceneC UUID="d" SID="4" CID="+00"/>
      <LightsceneC UUID="e" SID="future" CID="2147483648"><Unknown SID="2" CID="9"/></LightsceneC>
    </LightscenesC></C></ControlList>"#;
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let result = editor.inspect_lighting("uuid:lighting").unwrap();
    for code in ["duplicate_mood_sid", "duplicate_mood_cid"] {
        let diagnostics: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.code == code)
            .collect();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].path.contains("LightsceneC[2]"));
        assert!(diagnostics[0].message.contains("direct mood 1"));
    }
    assert_eq!(result.moods[0].attributes["SID"], "+0002");
    assert_eq!(result.moods[1].attributes["CID"], "+9");
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_identity_inspection_never_joins_nested_or_separate_collections() {
    for collections in [
        r#"<LightscenesC Num="1"><LightsceneC UUID="a" SID="2" CID="9"><LightsceneC UUID="nested" SID="2" CID="9"/></LightsceneC></LightscenesC>"#,
        r#"<LightscenesC Num="1"><LightsceneC UUID="a" SID="2" CID="9"/></LightscenesC><LightscenesC Num="1"><LightsceneC UUID="b" SID="2" CID="9"/></LightscenesC>"#,
        r#"<LightscenesC Num="2"><LightsceneC SID="0" CID="0"/><LightsceneC SID="+00" CID="+00"/></LightscenesC>"#,
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\">{collections}</C></ControlList>"
        );
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        assert!(
            !editor
                .inspect_lighting("uuid:lighting")
                .unwrap()
                .diagnostics
                .iter()
                .any(|d| d.code == "duplicate_mood_sid" || d.code == "duplicate_mood_cid")
        );
    }
}

#[test]
fn lighting_editor_inspection_distinguishes_selected_tags_capabilities_and_numeric_edit() {
    for (stored, capability, kind) in [
        ("1610622736", None, LightingEditorKind::Numeric),
        ("1610622736", Some("7"), LightingEditorKind::Color),
        ("1610622736", Some("64"), LightingEditorKind::Color),
        ("1610622736", Some("32"), LightingEditorKind::Numeric),
        (
            "268435465",
            Some("opaque-unused"),
            LightingEditorKind::Numeric,
        ),
        (
            "1879048201",
            Some("opaque-unused"),
            LightingEditorKind::Numeric,
        ),
        (
            "1073741824",
            Some("opaque-unused"),
            LightingEditorKind::Color,
        ),
        ("1342177280", None, LightingEditorKind::Color),
        ("-1879048192", None, LightingEditorKind::Color),
        ("0", None, LightingEditorKind::Other),
        ("-1", None, LightingEditorKind::Other),
    ] {
        let xml = LEVEL_XML
            .replace(
                "CapAI0=\"7\"",
                &format!(
                    "CapAI0=\"7\" {}",
                    capability
                        .map(|raw| format!("CapAI1=\"{raw}\""))
                        .unwrap_or_default()
                ),
            )
            .replace("Q2=\"1610622736\"", &format!("Q2=\"{stored}\""));
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let output = inspection.moods[0]
            .outputs
            .iter()
            .find(|q| q.index == 2)
            .unwrap();
        assert_eq!(output.editor.kind, kind, "{stored}/{capability:?}");
        assert_eq!(output.stored, stored);
        assert!(output.unit.is_none());
        assert_eq!(editor.to_bytes().unwrap(), before);
        if kind == LightingEditorKind::Numeric {
            editor
                .set_lighting_mood_level("uuid:lighting", "First", 2, 37)
                .unwrap();
        } else {
            assert!(
                editor
                    .set_lighting_mood_level("uuid:lighting", "First", 2, 37)
                    .is_err()
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
}

#[test]
fn lighting_editor_inspection_retains_missing_and_unsupported_capability_distinctions() {
    for capability in [None, Some("opaque"), Some("-1"), Some("65536")] {
        let xml = LEVEL_XML.replace(
            "CapAI0=\"7\"",
            &format!(
                "CapAI0=\"7\" {}",
                capability
                    .map(|v| format!("CapAI1=\"{v}\""))
                    .unwrap_or_default()
            ),
        );
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let output = inspection.moods[0]
            .outputs
            .iter()
            .find(|q| q.index == 2)
            .unwrap();
        let cap = output.editor.capability.as_ref().unwrap();
        assert_eq!(cap.stored.as_deref(), capability);
        assert_eq!(cap.effective, capability.is_none().then_some(0));
        if capability.is_some() {
            assert_eq!(output.editor.kind, LightingEditorKind::Unresolved);
            assert_eq!(
                output.editor.issue.as_ref().unwrap().code,
                "unsupported_mood_capability"
            );
            assert!(
                inspection
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "unsupported_mood_capability")
            );
        } else {
            assert_eq!(output.editor.kind, LightingEditorKind::Numeric);
            assert!(output.editor.issue.is_none());
        }
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_editor_inspection_does_not_infer_loaded_kinds_from_opaque_or_unloaded_storage() {
    for (xml, issue) in [
        (
            LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"opaque\""),
            "unsupported_mood_output_value",
        ),
        (
            LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"1\""),
            "unsupported_mood_output_width",
        ),
        (
            LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"bad\""),
            "unsupported_mood_output_width",
        ),
        (
            LEVEL_XML
                .replace(
                    "<LightsceneC Name=\"First\"",
                    "<Unknown><LightsceneC Name=\"First\"",
                )
                .replace("</LightsceneC>", "</LightsceneC></Unknown>"),
            "unsupported_mood_container_shape",
        ),
        (
            LEVEL_XML.replace("</C>", "<LightscenesC/></C>"),
            "unsupported_mood_container_shape",
        ),
        (
            LEVEL_XML.replace("Q2=\"1610622736\"", "Q19=\"1610622736\""),
            "unsupported_mood_output_index",
        ),
    ] {
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let output = inspection.moods[0]
            .outputs
            .iter()
            .find(|q| {
                q.index
                    == if issue == "unsupported_mood_output_index" {
                        19
                    } else {
                        2
                    }
            })
            .unwrap();
        assert_eq!(output.editor.kind, LightingEditorKind::Unresolved);
        assert_eq!(output.editor.issue.as_ref().unwrap().code, issue);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_numeric_level_matches_native_encoding_without_incidental_normalization() {
    let mut editor = ConfigEditor::load(LEVEL_XML.as_bytes()).unwrap();
    let edit = editor
        .set_lighting_mood_level("uuid:lighting", "First", 2, 37)
        .unwrap();
    assert_eq!(edit.after, "1610612773");
    assert_eq!(edit.type_tag, 6);
    let capability = edit.capability.unwrap();
    assert_eq!(capability.stored, None);
    assert_eq!(capability.effective, Some(0));
    let expected = LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"1610612773\"");
    assert_eq!(
        editor.to_bytes().unwrap(),
        ConfigEditor::load(expected.as_bytes())
            .unwrap()
            .to_bytes()
            .unwrap()
    );
    let xml = LEVEL_XML.replace("Q2=\"1610622736\"", "Q2=\"+01610612773\"");
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let edit = editor
        .set_lighting_mood_level("uuid:lighting", "2", 2, 37)
        .unwrap();
    assert!(!edit.changed);
    assert_eq!(edit.after, "+01610612773");
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_numeric_level_uses_selected_tag_and_zero_based_capability_not_color_packing() {
    for (stored, capability, expected) in [
        ("268435465", "opaque-unused", "268435493"),
        ("536870921", "opaque-unused", "536870949"),
        ("805306377", "opaque-unused", "805306405"),
        ("1879048201", "opaque-unused", "1879048229"),
        ("1610622736", "6", "1610612773"),
        ("1610622736", "32", "1610612773"),
    ] {
        let xml = LEVEL_XML
            .replace(
                "CapAI0=\"7\"",
                &format!("CapAI0=\"7\" CapAI1=\"{capability}\""),
            )
            .replace("Q2=\"1610622736\"", &format!("Q2=\"{stored}\""));
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor
            .set_lighting_mood_level("uuid:lighting", "First", 2, 37)
            .unwrap();
        assert_eq!(edit.after, expected, "{stored}/{capability}");
    }
}

#[test]
fn lighting_numeric_level_preview_is_immutable_and_matches_boundary_noop_and_selected_only_writes()
{
    for (output, raw, capability, level, after, changed) in [
        (1, "+00268435465", "opaque-unused", 0, "268435456", true),
        (2, "536870921", "opaque-unused", 100, "536871012", true),
        (2, "805306377", "opaque-unused", 37, "805306405", true),
        (18, "1879048201", "opaque-unused", 100, "1879048292", true),
        (2, "1610622736", "32", 0, "1610612736", true),
        (2, "+01610612773", "+0006", 37, "+01610612773", false),
        (2, "1610612736", "", 0, "1610612736", false),
    ] {
        let key = format!("Q{output}");
        let xml = LEVEL_XML
            .replace(
                "CapAI0=\"7\"",
                &format!(
                    "{} {}",
                    if output == 1 { "" } else { "CapAI0=\"7\"" },
                    if capability.is_empty() {
                        String::new()
                    } else {
                        format!("CapAI{}=\"{capability}\"", output - 1)
                    }
                ),
            )
            .replace(
                "Name=\"First\" SID=\"2\" UUID=\"first\"",
                "Name=\"First\" SID=\"+003\" UUID=\"first\" CID=\"future\" MixMod=\"opaque\" SkipNext=\"opaque\"",
            );
        let prior = match output {
            1 => "Q1=\"1640612766\"",
            18 => "Future=\"opaque\"",
            _ => "Q2=\"1610622736\"",
        };
        let attr = if output == 18 {
            format!("Future=\"opaque\" {key}=\"{raw}\"")
        } else {
            format!("{key}=\"{raw}\"")
        };
        let xml = format!(
            "\u{feff}{}",
            xml.replace(prior, &attr).replace('\n', "\r\n")
        );
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_lighting_mood_level("uuid:lighting", "First", output, level)
            .unwrap();
        assert_eq!(editor.to_bytes().unwrap(), before);
        assert!(before.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            !String::from_utf8_lossy(&before)
                .replace("\r\n", "")
                .contains('\n')
        );
        assert_eq!(preview.selected_mood.sid.as_deref(), Some("+003"));
        assert_eq!(preview.selected_mood.cid.as_deref(), Some("future"));
        assert_eq!(preview.selected_mood.uuid.as_deref(), Some("first"));
        assert_eq!(preview.output_index, output);
        assert_eq!(preview.editor_kind, LightingEditorKind::Numeric);
        assert_eq!(
            preview.before_effective,
            raw.parse::<i32>().unwrap() & 0x0fff_ffff
        );
        assert_eq!(u32::from(preview.after_effective), level);
        assert_eq!(preview.edit.after, after);
        assert_eq!(preview.edit.changed, changed);
        let mut writer = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = writer
            .set_lighting_mood_level("uuid:lighting", "First", output, level)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&preview.edit).unwrap(),
            serde_json::to_value(&edit).unwrap()
        );
        let expected = xml.replacen(
            &format!("{key}=\"{raw}\""),
            &format!("{key}=\"{after}\""),
            1,
        );
        assert_eq!(
            writer.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }
    for width in ["", "Outputs=\"+000\""] {
        let xml = LEVEL_XML
            .replace("Outputs=\"18\"", width)
            .replace("SID=\"2\" UUID=\"first\"", "")
            .replace(
                "Q10=\"1610612736\"",
                "Q10=\"1610612736\" Q12=\"1610612736\" Q13=\"1610612736\"",
            );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_lighting_mood_level("uuid:lighting", "First", 12, 100)
            .unwrap();
        assert!(preview.selected_mood.uuid.is_none());
        assert!(preview.selected_mood.sid.is_none());
        assert!(preview.selected_mood.cid.is_none());
        assert_eq!(preview.edit.after, "1610612836");
        let error = editor
            .preview_lighting_mood_level("uuid:lighting", "First", 13, 100)
            .unwrap_err();
        assert_eq!(
            error
                .downcast_ref::<super::schema::Diagnostic>()
                .unwrap()
                .code,
            "unsupported_mood_output_width"
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
        let edit = editor
            .set_lighting_mood_level("uuid:lighting", "First", 12, 100)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&preview.edit).unwrap(),
            serde_json::to_value(&edit).unwrap()
        );
        let expected = xml.replace("Q12=\"1610612736\"", "Q12=\"1610612836\"");
        assert_eq!(
            editor.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }
}

#[test]
fn lighting_numeric_level_rejects_unsupported_inputs_before_any_mutation() {
    let mut cases: Vec<(String, usize, u32)> = [
        ("CapAI1=\"7\"", "1610622736"),
        ("CapAI1=\"64\"", "1610622736"),
        ("CapAI1=\"71\"", "1610622736"),
        ("CapAI1=\"opaque\"", "1610622736"),
        ("CapAI1=\"65536\"", "1610622736"),
        ("CapAI1=\"-1\"", "1610622736"),
        ("", "1073741824"),
        ("", "1342177280"),
        ("", "-1"),
        ("", "0"),
        ("", "future"),
        ("", "2147483648"),
    ]
    .into_iter()
    .map(|(capability, stored)| {
        (
            LEVEL_XML
                .replace("CapAI0=\"7\"", capability)
                .replace("Q2=\"1610622736\"", &format!("Q2=\"{stored}\"")),
            2,
            37,
        )
    })
    .collect();
    cases.extend([
        (LEVEL_XML.into(), 0, 37),
        (LEVEL_XML.into(), 19, 37),
        (LEVEL_XML.into(), 2, 101),
        (LEVEL_XML.into(), 2, u32::MAX),
        (LEVEL_XML.replace("Q2=\"1610622736\"", ""), 2, 37),
        (LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"1\""), 2, 37),
        (
            LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"bad\""),
            2,
            37,
        ),
        (
            LEVEL_XML.replace("Name=\"Second\"", "Name=\"First\""),
            2,
            37,
        ),
        (LEVEL_XML.replace("</C>", "<LightscenesC/></C>"), 2, 37),
        (LEVEL_XML.replace("LightController2", "Switch"), 2, 37),
        (LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"-1\""), 2, 37),
        (
            LEVEL_XML.replace("Outputs=\"18\"", "Outputs=\"2147483648\""),
            2,
            37,
        ),
        (LEVEL_XML.replace("Outputs=\"18\"", ""), 13, 37),
        (
            LEVEL_XML.replace("Name=\"First\"", "Name=\"Missing\""),
            2,
            37,
        ),
        (
            LEVEL_XML.replace("Name=\"Second\" SID=\"3\"", "Name=\"First\" SID=\"2\""),
            2,
            37,
        ),
    ]);
    for (xml, output, level) in cases {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview_error = editor
            .preview_lighting_mood_level("uuid:lighting", "First", output, level)
            .unwrap_err();
        assert_eq!(editor.to_bytes().unwrap(), before);
        let writer_error = editor
            .set_lighting_mood_level("uuid:lighting", "First", output, level)
            .unwrap_err();
        assert_eq!(
            preview_error.to_string(),
            writer_error.to_string(),
            "{xml}/{output}/{level}"
        );
        assert_eq!(
            preview_error.downcast_ref::<super::schema::Diagnostic>(),
            writer_error.downcast_ref::<super::schema::Diagnostic>()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    let editor = ConfigEditor::load(LEVEL_XML.as_bytes()).unwrap();
    for (selector, output, level, code) in [
        ("missing", 0, 101, "mood_level_out_of_range"),
        ("missing", 0, 37, "unsupported_mood_output_index"),
    ] {
        let error = editor
            .preview_lighting_mood_level(selector, "missing", output, level)
            .unwrap_err();
        assert_eq!(
            error
                .downcast_ref::<super::schema::Diagnostic>()
                .unwrap()
                .code,
            code
        );
    }
}

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

#[test]
fn lighting_raw_color_preview_preserves_first_recursive_match_without_new_classification() {
    let xml = r#"<ControlList><C Type="LightController2" U="lighting" Title="Lamp" CapAI0="opaque">
      <Unknown><!--retain--><LightsceneC UUID="nested" Name="A" SID="future" CID="opaque" Outputs="bad" Q99="future-value"><SeqConf Future="keep"/></LightsceneC></Unknown>
      <LightscenesC Num="bad"><LightsceneC UUID="later" Name="future" SID="A" Q99="-1"/></LightscenesC>
      <LightscenesC/><LSConfig OpM="opaque" ScMv="opaque"/><HCL T="2"/>
    </C></ControlList>"#;
    for (output, value) in [(99, -1), (99, i64::MIN), (99, i64::MAX), (usize::MAX, 42)] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_mood_color("uuid:lighting", "A", output, value)
            .unwrap();
        assert_eq!(editor.to_bytes().unwrap(), before);
        assert_eq!(preview.scope, "raw_packed_edit");
        assert_eq!(preview.controller_uuid.as_deref(), Some("lighting"));
        assert_eq!(preview.controller_title.as_deref(), Some("Lamp"));
        assert_eq!(preview.selected_mood.uuid.as_deref(), Some("nested"));
        assert_eq!(preview.selected_mood.sid.as_deref(), Some("future"));
        assert_eq!(preview.selected_mood.cid.as_deref(), Some("opaque"));
        assert_eq!(preview.selected_mood_name.as_deref(), Some("A"));
        assert_eq!(preview.selected_mood_path, vec![0, 1]);
        assert_eq!(preview.output, format!("Q{output}"));
        assert_eq!(
            preview.before.as_deref(),
            (output == 99).then_some("future-value")
        );
        assert_eq!(preview.after, value.to_string());
        assert!(preview.changed);
        assert_eq!(
            editor
                .set_mood_color("uuid:lighting", "A", output, value)
                .unwrap(),
            format!("Set Q{output}={value} on mood 'A' of LightController2 'Lamp'")
        );
        let expected = if output == 99 {
            xml.replace("Q99=\"future-value\"", &format!("Q99=\"{value}\""))
        } else {
            xml.replace(
                "Q99=\"future-value\"",
                &format!("Q99=\"future-value\" Q{output}=\"{value}\""),
            )
        };
        assert_eq!(
            editor.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }
}

#[test]
fn lighting_raw_color_preview_matches_raw_spelling_noop_and_missing_identity_rules() {
    for (attribute, value, changed) in [
        ("Q1=\"1610622736\"", 1_610_622_736, false),
        ("Q1=\"+01610622736\"", 1_610_622_736, true),
        ("Q1=\"4294967295\"", 4_294_967_295, false),
        ("Q1=\"-2147483648\"", -2_147_483_648, false),
        ("", 0, true),
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\"><Unknown><LightsceneC {attribute}/></Unknown><LightsceneC Name=\"\" SID=\"\" Q1=\"other\"/></C></ControlList>"
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_mood_color("uuid:lighting", "", 1, value)
            .unwrap();
        assert_eq!(preview.changed, changed);
        assert!(preview.selected_mood.uuid.is_none());
        assert!(preview.selected_mood.sid.is_none());
        assert!(preview.selected_mood_name.is_none());
        assert_eq!(preview.selected_mood_path, vec![0, 0]);
        assert_eq!(editor.to_bytes().unwrap(), before);
        editor
            .set_mood_color("uuid:lighting", "", 1, value)
            .unwrap();
        let expected = xml.replacen(
            &format!("<LightsceneC {attribute}/>"),
            &format!("<LightsceneC Q1=\"{value}\"/>"),
            1,
        );
        assert_eq!(
            editor.to_bytes().unwrap(),
            ConfigEditor::load(expected.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap()
        );
    }
}

#[test]
fn lighting_raw_color_preview_and_writer_share_existing_ordered_errors() {
    for (xml, selector, output, message) in [
        (LEVEL_XML.to_owned(), "uuid:missing", 0, "uuid:missing"),
        (
            LEVEL_XML.replace("LightController2", "Switch"),
            "uuid:lighting",
            0,
            "not a LightController2",
        ),
        (LEVEL_XML.to_owned(), "uuid:lighting", 0, "Q0"),
        (
            LEVEL_XML.to_owned(),
            "uuid:lighting",
            1,
            "mood 'missing' not found",
        ),
    ] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview_error = editor
            .preview_mood_color(selector, "missing", output, 42)
            .unwrap_err()
            .to_string();
        let writer_error = editor
            .set_mood_color(selector, "missing", output, 42)
            .unwrap_err()
            .to_string();
        assert_eq!(preview_error, writer_error);
        assert!(preview_error.contains(message), "{preview_error}");
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_mood_edit_preserves_nonzero_edits_of_sparse_moods() {
    let fixture = std::str::from_utf8(FIXTURE).unwrap();
    for (mood_ref, output, marker) in [
        ("First", 3, "FutureMood=\"opaque\""),
        ("3", 2, "Q18=\"future-encoding\""),
    ] {
        let mut editor = ConfigEditor::load(FIXTURE).unwrap();
        editor
            .set_mood_color("uuid:lighting", mood_ref, output, 42)
            .unwrap();
        let expected = fixture.replace(marker, &format!("{marker} Q{output}=\"42\""));
        let expected = ConfigEditor::load(expected.as_bytes()).unwrap();
        assert_eq!(editor.to_bytes().unwrap(), expected.to_bytes().unwrap());
    }
}

#[test]
fn lighting_mood_edit_rejects_zero_output_even_when_raw_storage_contains_q0() {
    let fixture = std::str::from_utf8(FIXTURE).unwrap();
    for xml in [
        fixture.to_string(),
        fixture.replace("Q2=\"37\"", "Q0=\"unknown\" Q2=\"37\""),
    ] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .set_mood_color("uuid:lighting", "First", 0, 42)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_mood_edit_preserves_everything_except_the_selected_stored_value() {
    let xml = std::str::from_utf8(FIXTURE).unwrap().replace(
        "FutureMood=\"opaque\"/>",
        "FutureMood=\"opaque\" Q01=\"opaque\" Q99=\"future-value\"><SeqConf Future=\"retain\"><Unknown Q2=\"nested\"/></SeqConf></LightsceneC>",
    );
    for (mood_ref, output, old_value) in [("First", 2, "37"), ("2", 99, "future-value")] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        editor
            .set_mood_color("uuid:lighting", mood_ref, output, 42)
            .unwrap();
        let expected = xml.replace(
            &format!("Q{output}=\"{old_value}\""),
            &format!("Q{output}=\"42\""),
        );
        let expected = ConfigEditor::load(expected.as_bytes()).unwrap();
        assert_eq!(editor.to_bytes().unwrap(), expected.to_bytes().unwrap());
        let moods = editor.inspect_lighting("uuid:lighting").unwrap().moods;
        assert!(!moods[0].attributes.contains_key("Q3"));
        assert_eq!(moods[0].attributes["Q01"], "opaque");
        assert_eq!(moods[1].attributes["Q1"], "0");
    }
}

const RECOUNT_XML: &str = r#"<ControlList Version="267">
  <C Type="LightController2" U="lighting" Title="Lamp">
    <Co K="AQ2" U="output-2"><In Input="source" FutureWire="retain"/></Co>
    <LightscenesC Num="99" Outputs="0002" FC="opaque" Future="retain">
      <LightsceneC UUID="first" Name="First" SID="2" CID="9" Outputs="3" Q1="00037" Q3="future-value">
        <SeqConf Future="keep"><Unknown Q2="nested"/></SeqConf>
      </LightsceneC>
      <Unknown><LightsceneC UUID="not-a-direct-mood" Num="71"/></Unknown>
      <LightsceneC UUID="second" Name="Second" SID="3" CID="10" Q2="-1" Q01="opaque"/>
    </LightscenesC>
    <FutureStorage Outputs="91" Num="84"/>
  </C>
</ControlList>"#;

#[test]
fn lighting_transaction_inspection_warns_on_explicit_zero_sid_without_allocating_ids() {
    let xml = r#"<ControlList><C Type="LightController2" U="lighting">
      <LightscenesC Num="2" Outputs="2">
        <LightsceneC UUID="first" SID="+0" Outputs="1" Q1="00037">
          <SeqConf Num="0" Future="retain"/>
        </LightsceneC>
        <Unknown><LightsceneC SID="0"/></Unknown>
        <LightsceneC UUID="second" SID="000" Q9="future"/>
      </LightscenesC>
    </C></ControlList>"#;
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
    let warnings: Vec<_> = inspection
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "native_zero_mood_sid_reconciled")
        .map(|diagnostic| diagnostic.path.as_str())
        .collect();
    assert_eq!(
        warnings,
        [
            "LightscenesC/LightsceneC[1]/@SID",
            "LightscenesC/LightsceneC[2]/@SID"
        ]
    );
    assert_eq!(inspection.moods[0].attributes["SID"], "+0");
    assert_eq!(inspection.moods[0].attributes["Q1"], "00037");
    assert!(!inspection.moods[0].attributes.contains_key("Q2"));
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_transaction_inspection_warns_only_for_explicit_empty_zero_count() {
    for (count, children, expected) in [
        ("0", "", true),
        ("+000", "<Unknown><LightsceneC SID=\"0\"/></Unknown>", true),
        ("", "", false),
        ("bad", "", false),
        ("-1", "", false),
        ("1", "", false),
        ("0", "<LightsceneC SID=\"2\"/>", false),
    ] {
        let num = if count.is_empty() {
            String::new()
        } else {
            format!("Num=\"{count}\"")
        };
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\"><LightscenesC {num} Outputs=\"2\">{children}</LightscenesC></C></ControlList>"
        );
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        assert_eq!(
            inspection
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "native_empty_mood_collection_reinitialized"),
            expected,
            "{num} {children}"
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_transaction_zero_sid_warning_requires_a_known_matching_native_count() {
    for (num, expected) in [
        ("", false),
        ("Num=\"bad\"", false),
        ("Num=\"4294967296\"", false),
        ("Num=\"-1\"", false),
        ("Num=\"0\"", false),
        ("Num=\"1\"", false),
        ("Num=\"3\"", false),
        ("Num=\"2\"", true),
        ("Num=\"+0002\"", true),
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\"><LightscenesC {num}><LightsceneC UUID=\"first\" SID=\"0\"/><LightsceneC UUID=\"second\" SID=\"0\"/></LightscenesC></C></ControlList>"
        );
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let warnings = inspection
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "native_zero_mood_sid_reconciled")
            .count();
        assert_eq!(warnings, if expected { 2 } else { 0 }, "{num}");
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_transaction_inspection_does_not_infer_zero_from_unknown_or_missing_sid() {
    for sid in [
        "",
        "SID=\"bad\"",
        "SID=\"4294967296\"",
        "SID=\"-1\"",
        "SID=\"2\"",
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"1\"><LightsceneC {sid}/></LightscenesC></C></ControlList>"
        );
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        assert!(
            !inspection
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "native_zero_mood_sid_reconciled"),
            "{sid}"
        );
    }
}

#[test]
fn lighting_native_identity_errors_leave_the_editor_unchanged() {
    let occupied = (1..=99)
        .map(|sid| format!(r#"<LightsceneC SID="{sid}" CID="0"/>"#))
        .collect::<String>();
    for (scenes, code) in [
        (occupied.as_str(), "mood_identity_exhausted"),
        (
            r#"<LightsceneC SID="future"/>"#,
            "unsupported_mood_identity_value",
        ),
        (
            r#"<LightsceneC SID="2" CID="future"/>"#,
            "unsupported_mood_identity_value",
        ),
    ] {
        let xml = format!(
            r#"<ControlList><C Type="LightController2" U="lighting"><LightscenesC Num="1">{scenes}</LightscenesC></C></ControlList>"#
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .add_mood("uuid:lighting", "New", 42, None, None)
            .unwrap_err();
        let diagnostic = error.downcast_ref::<super::schema::Diagnostic>().unwrap();
        assert_eq!(diagnostic.code, code);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_add_mood_recounts_direct_records_instead_of_incrementing_stale_num() {
    for replacement in [
        "",
        "Num=\"99\"",
        "Num=\"1\"",
        "Num=\"invalid\"",
        "Num=\"-1\"",
        "Num=\"9223372036854775807\"",
    ] {
        let xml = RECOUNT_XML
            .replace("Num=\"99\"", replacement)
            .replace("Outputs=\"3\"", "Outputs=\"2\"");
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        editor
            .add_mood("uuid:lighting", "Third", 42, None, None)
            .unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let collection = inspection
            .nested_storage
            .iter()
            .find(|storage| storage.element == "LightscenesC")
            .unwrap();
        assert_eq!(collection.attributes["Num"], "3", "{replacement}");
        assert_eq!(collection.attributes["Outputs"], "0002");
        assert!(!inspection.moods[0].attributes.contains_key("Q2"));
        assert_eq!(inspection.moods[0].attributes["Q3"], "future-value");
        assert!(
            !inspection
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "inconsistent_mood_count")
        );
    }
}

#[test]
fn lighting_native_append_uses_first_direct_mood_types_and_preserves_existing_storage() {
    let xml = r#"<ControlList><C Type="LightController2" U="lighting">
      <LightscenesC Num="99" Outputs="00020" Future="retain">
        <Unknown><LightsceneC Name="Nested" Q2="-1"/></Unknown>
        <LightsceneC UUID="first" Name="First" SID="2" CID="9" Outputs="3"
          Q1="opaque-selected-output" Q2="1610612778" Q3="-1" Q18="future">
          <SeqConf Future="retain"/>
        </LightsceneC>
        <LightsceneC UUID="second" Name="Second" SID="3" CID="10" Q2="0"/>
      </LightscenesC>
    </C></ControlList>"#;
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.inspect_lighting("uuid:lighting").unwrap();
    editor
        .add_mood("uuid:lighting", "New", 42, None, None)
        .unwrap();
    let after = editor.inspect_lighting("uuid:lighting").unwrap();
    for (old, unchanged) in before.moods.iter().zip(&after.moods) {
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(unchanged).unwrap()
        );
    }
    let new = after.moods.last().unwrap();
    assert_eq!(new.attributes["Outputs"], "18");
    assert_eq!(new.outputs.len(), 18);
    assert_eq!(new.attributes["Q1"], "42");
    assert_eq!(new.attributes["Q2"], "1610612736");
    assert_eq!(new.attributes["Q3"], "-268435456");
    assert_eq!(new.attributes["Q18"], "0");
    let collection = after
        .nested_storage
        .iter()
        .find(|storage| storage.element == "LightscenesC")
        .unwrap();
    assert_eq!(collection.attributes["Outputs"], "00020");
    assert_eq!(collection.attributes["Future"], "retain");
    assert_eq!(collection.attributes["Num"], "3");
}

#[test]
fn lighting_native_append_respects_source_read_bound_without_guessing_a_channel_maximum() {
    for (width, expected_q2, expected_q13) in [
        ("", "-2147483648", "0"),
        ("Outputs=\"0\"", "-2147483648", "0"),
        ("Outputs=\"20\"", "-2147483648", "1610612736"),
        ("Outputs=\"1\"", "0", "0"),
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\"><LightscenesC Outputs=\"future\" Num=\"1\"><LightsceneC Name=\"First\" {width} Q2=\"-2147483648\" Q13=\"1610612778\" Q19=\"opaque\"/></LightscenesC></C></ControlList>"
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        editor
            .add_mood("uuid:lighting", "New", 42, None, None)
            .unwrap();
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let new = inspection.moods.last().unwrap();
        assert_eq!(new.attributes["Outputs"], "18");
        assert_eq!(new.attributes["Q2"], expected_q2, "{width}");
        assert_eq!(new.attributes["Q13"], expected_q13, "{width}");
        assert!(!new.attributes.contains_key("Q19"));
        assert_eq!(inspection.moods[0].attributes["Q19"], "opaque");
    }
    let mut editor = ConfigEditor::load(
        br#"<ControlList><C Type="LightController2" U="lighting"><LightscenesC Outputs="future" Num="0"><Unknown/></LightscenesC></C></ControlList>"#,
    )
    .unwrap();
    editor
        .add_mood("uuid:lighting", "New", 42, None, None)
        .unwrap();
    let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
    let new = &inspection.moods[0];
    assert_eq!(new.attributes["Outputs"], "18");
    assert_eq!(new.attributes["Q2"], "0");
    assert_eq!(new.attributes["Q18"], "0");
}

#[test]
fn lighting_native_append_rejects_unsupported_source_values_and_shapes_atomically() {
    for fields in [
        "Outputs=\"future\"",
        "Outputs=\"-1\"",
        "Outputs=\"2147483648\"",
        "Outputs=\"2\" Q2=\"opaque\"",
        "Outputs=\"2\" Q2=\"2147483648\"",
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"99\"><LightsceneC Name=\"First\" {fields}/></LightscenesC></C></ControlList>"
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .add_mood("uuid:lighting", "New", 42, None, None)
            .unwrap_err();
        assert!(error.downcast_ref::<super::schema::Diagnostic>().is_some());
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    for collections in [
        "<Unknown><LightscenesC/></Unknown>",
        "<LightscenesC/><LightscenesC/>",
    ] {
        let xml = format!(
            "<ControlList><C Type=\"LightController2\" U=\"lighting\">{collections}</C></ControlList>"
        );
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .add_mood("uuid:lighting", "New", 42, None, None)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_recount_repairs_only_num_and_counts_direct_moods_not_outputs_or_nested_records() {
    let mut editor = ConfigEditor::load(RECOUNT_XML.as_bytes()).unwrap();
    assert!(
        editor
            .inspect_lighting("uuid:lighting")
            .unwrap()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inconsistent_mood_count")
    );
    let edit = editor.recount_lighting_moods("uuid:lighting").unwrap();
    assert_eq!(edit.before.as_deref(), Some("99"));
    assert_eq!(edit.after, "2");
    assert_eq!(edit.scenes, 2);
    assert!(edit.changed);
    let expected =
        ConfigEditor::load(RECOUNT_XML.replace("Num=\"99\"", "Num=\"2\"").as_bytes()).unwrap();
    assert_eq!(editor.to_bytes().unwrap(), expected.to_bytes().unwrap());
    assert!(
        !editor
            .inspect_lighting("uuid:lighting")
            .unwrap()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inconsistent_mood_count")
    );
}

#[test]
fn lighting_recount_handles_missing_invalid_and_negative_num_without_output_defaults() {
    for replacement in ["", "Num=\"invalid\"", "Num=\"-1\"", "Num=\"0\""] {
        let xml = RECOUNT_XML
            .replace("Num=\"99\"", replacement)
            .replace(" Outputs=\"0002\"", "");
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let edit = editor.recount_lighting_moods("uuid:lighting").unwrap();
        assert_eq!(edit.after, "2");
        assert!(edit.changed);
        let inspection = editor.inspect_lighting("uuid:lighting").unwrap();
        let collection = inspection
            .nested_storage
            .iter()
            .find(|storage| storage.element == "LightscenesC")
            .unwrap();
        assert!(!collection.attributes.contains_key("Outputs"));
        assert!(!inspection.moods[0].attributes.contains_key("Q2"));
        assert_eq!(inspection.moods[0].attributes["Q3"], "future-value");
    }
}

#[test]
fn lighting_recount_preserves_equivalent_raw_num_and_can_count_an_empty_collection() {
    for raw in ["2", "0002", "+2"] {
        let xml = RECOUNT_XML.replace("Num=\"99\"", &format!("Num=\"{raw}\""));
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let edit = editor.recount_lighting_moods("uuid:lighting").unwrap();
        assert!(!edit.changed);
        assert_eq!(edit.after, raw);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    let mut editor = ConfigEditor::load(
        br#"<ControlList><C Type="LightController2" U="empty"><LightscenesC Num="17" Outputs="future"><Unknown/></LightscenesC></C></ControlList>"#,
    )
    .unwrap();
    let edit = editor.recount_lighting_moods("uuid:empty").unwrap();
    assert_eq!(edit.scenes, 0);
    assert_eq!(edit.after, "0");
    assert!(edit.changed);
}

#[test]
fn lighting_recount_rejects_ambiguous_or_missing_direct_collection_before_mutating() {
    for xml in [
        r#"<ControlList><C Type="LightController2" U="lighting"><LightscenesC Num="99"/><LightscenesC Num="71"/></C></ControlList>"#,
        r#"<ControlList><C Type="LightController2" U="lighting"><Unknown><LightscenesC Num="99"/></Unknown></C></ControlList>"#,
        r#"<ControlList><C Type="And" U="lighting"><LightscenesC Num="99"/></C></ControlList>"#,
    ] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        assert!(editor.recount_lighting_moods("uuid:lighting").is_err());
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}
