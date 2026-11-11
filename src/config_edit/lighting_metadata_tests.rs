use super::ConfigEditor;
use super::schema::Diagnostic;

const METADATA_XML: &str = r#"<ControlList><C Type="LightController2" U="lighting" Title="Lamp" CapAI0="opaque">
  <LightscenesC Num="009" Outputs="bad" Future="retain">
    <LightsceneC Name="First" SID="2" CID="+012" IID="7" UUID="first" Outputs="18" Q1="opaque" Q2="+01610612736" Q19="future" MixMod="true" Move="false" ReadOnly="true" Sequencing="true" SkipNext="false" SName="alias" Future="opaque">
      <SeqConf Future="keep"><Unknown Name="First" Q2="nested"/></SeqConf>
    </LightsceneC>
    <Unknown><LightsceneC Name="First" SID="2" UUID="nested"/></Unknown>
    <LightsceneC Name="Second" SID="3" CID="0" UUID="second" MixMod="false"/>
  </LightscenesC>
  <LSConfig OpM="future"><Unknown SID="2" Name="First"/></LSConfig>
  <HCL T="2" TMIN="2000" FROM="0" TO="0"/>
</C></ControlList>"#;

fn assert_xml(editor: &ConfigEditor, expected: &str) {
    assert_eq!(
        editor.to_bytes().unwrap(),
        ConfigEditor::load(expected.as_bytes())
            .unwrap()
            .to_bytes()
            .unwrap()
    );
}

fn assert_error_unchanged(editor: &ConfigEditor, before: &[u8], error: anyhow::Error, code: &str) {
    assert_eq!(error.downcast_ref::<Diagnostic>().unwrap().code, code);
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_rename_changes_only_selected_name_and_preserves_noop() {
    let mut editor = ConfigEditor::load(METADATA_XML.as_bytes()).unwrap();
    let edit = editor
        .rename_lighting_mood("uuid:lighting", "2", "Evening Scene 2")
        .unwrap();
    assert_eq!(edit.before.as_deref(), Some("First"));
    assert_eq!(edit.after, "Evening Scene 2");
    assert_eq!(edit.mood.uuid.as_deref(), Some("first"));
    assert_eq!(edit.mood.sid.as_deref(), Some("2"));
    assert_eq!(edit.mood.cid.as_deref(), Some("+012"));
    assert!(edit.changed);
    assert_xml(
        &editor,
        &METADATA_XML.replacen("Name=\"First\"", "Name=\"Evening Scene 2\"", 1),
    );
    let before = editor.to_bytes().unwrap();
    let edit = editor
        .rename_lighting_mood("uuid:lighting", "Evening Scene 2", "Evening Scene 2")
        .unwrap();
    assert!(!edit.changed);
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_rename_permits_duplicate_new_names_but_rejects_ambiguous_selection() {
    let mut editor = ConfigEditor::load(METADATA_XML.as_bytes()).unwrap();
    editor
        .rename_lighting_mood("uuid:lighting", "First", "Second")
        .unwrap();
    assert_xml(
        &editor,
        &METADATA_XML.replacen("Name=\"First\"", "Name=\"Second\"", 1),
    );
    let before = editor.to_bytes().unwrap();
    let error = editor
        .rename_lighting_mood("uuid:lighting", "Second", "Third")
        .unwrap_err();
    assert_error_unchanged(&editor, &before, error, "ambiguous_or_missing_mood");
    editor
        .rename_lighting_mood("uuid:lighting", "2", "Third")
        .unwrap();
}

#[test]
fn lighting_metadata_rename_can_fill_missing_name_without_synthesizing_identity() {
    let xml = METADATA_XML.replacen("Name=\"First\" ", "", 1);
    let xml = xml.replacen("CID=\"+012\" IID=\"7\" UUID=\"first\" ", "", 1);
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let edit = editor
        .rename_lighting_mood("uuid:lighting", "2", "New Name")
        .unwrap();
    assert_eq!(edit.before, None);
    assert_eq!(edit.mood.uuid, None);
    assert_eq!(edit.mood.cid, None);
    assert_xml(
        &editor,
        &xml.replacen(
            "Future=\"opaque\">",
            "Future=\"opaque\" Name=\"New Name\">",
            1,
        ),
    );
}

#[test]
fn lighting_metadata_rename_rejects_unmodeled_name_normalization_atomically() {
    for name in [
        "",
        " ",
        " First",
        "First ",
        "First\tSecond",
        "First\nSecond",
        "Evening_2",
        "50%",
        "A{B}",
        "A/B",
        "$S_thing",
        "Grün",
        "\u{0}",
    ] {
        let mut editor = ConfigEditor::load(METADATA_XML.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .rename_lighting_mood("uuid:lighting", "2", name)
            .unwrap_err();
        assert_error_unchanged(&editor, &before, error, "unsupported_mood_name");
    }
}

#[test]
fn lighting_metadata_rename_requires_one_direct_exact_unambiguous_mood() {
    for (xml, reference, code) in [
        (
            METADATA_XML.to_owned(),
            "first",
            "ambiguous_or_missing_mood",
        ),
        (METADATA_XML.to_owned(), "Fir", "ambiguous_or_missing_mood"),
        (
            METADATA_XML.replace("Name=\"Second\"", "Name=\"2\""),
            "2",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.replace("SID=\"3\"", "SID=\"2\""),
            "2",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.replace("</C>", "<LightscenesC/></C>"),
            "2",
            "unsupported_mood_container_shape",
        ),
        (
            METADATA_XML.replace("LightscenesC", "UnknownContainer"),
            "2",
            "unsupported_mood_container_shape",
        ),
        (
            METADATA_XML.replace("LightController2", "LightController"),
            "2",
            "wrong_block_type",
        ),
    ] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .rename_lighting_mood("uuid:lighting", reference, "New Name")
            .unwrap_err();
        assert_error_unchanged(&editor, &before, error, code);
    }
}

fn mix_xml() -> String {
    METADATA_XML
        .replacen("CID=\"+012\"", "CID=\"+000\"", 1)
        .replacen("SID=\"2\"", "SID=\"+002\"", 1)
}

#[test]
fn lighting_metadata_mix_toggles_only_selected_record_without_normalizing_others() {
    let xml = mix_xml();
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "First", false)
        .unwrap();
    assert_eq!(edit.before.as_deref(), Some("true"));
    assert_eq!(edit.after.as_deref(), Some("false"));
    assert!(!edit.enabled);
    assert!(edit.changed);
    assert_eq!(edit.scope, "selected_record_role_independent");
    assert_eq!(edit.mood.sid.as_deref(), Some("+002"));
    assert_eq!(edit.mood.cid.as_deref(), Some("+000"));
    assert_xml(
        &editor,
        &xml.replacen("MixMod=\"true\"", "MixMod=\"false\"", 1),
    );
    let before = editor.to_bytes().unwrap();
    assert!(
        !editor
            .set_lighting_mood_mix("uuid:lighting", "+002", false)
            .unwrap()
            .changed
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
    editor
        .set_lighting_mood_mix("uuid:lighting", "+002", true)
        .unwrap();
    assert_xml(&editor, &xml);
}

#[test]
fn lighting_metadata_mix_enables_existing_false_without_touching_other_true_records() {
    let xml = mix_xml();
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "3", true)
        .unwrap();
    assert_eq!(edit.before.as_deref(), Some("false"));
    assert_eq!(edit.after.as_deref(), Some("true"));
    assert_eq!(edit.mood.uuid.as_deref(), Some("second"));
    assert!(edit.changed);
    assert_xml(&editor, &xml.replace("MixMod=\"false\"", "MixMod=\"true\""));
    for (sid, cid) in [(1, 11), (31, 20)] {
        let xml = mix_xml()
            .replacen("SID=\"+002\"", &format!("SID=\"{sid}\""), 1)
            .replacen("CID=\"+000\"", &format!("CID=\"{cid}\""), 1);
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        editor
            .set_lighting_mood_mix("uuid:lighting", "First", false)
            .unwrap();
        assert_xml(
            &editor,
            &xml.replacen("MixMod=\"true\"", "MixMod=\"false\"", 1),
        );
    }
}

#[test]
fn lighting_metadata_mix_rejects_role_dependent_and_non_checkbox_rows_even_on_noop() {
    for cid in 12..=19 {
        let xml = mix_xml().replacen("CID=\"+000\"", &format!("CID=\"{cid}\""), 1);
        for enabled in [false, true] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let error = editor
                .set_lighting_mood_mix("uuid:lighting", "First", enabled)
                .unwrap_err();
            assert_error_unchanged(&editor, &before, error, "role_dependent_mood_mix");
        }
    }
    for sid in [-1, 0, 32, 90, 776, 777, 778] {
        let xml = mix_xml().replacen("SID=\"+002\"", &format!("SID=\"{sid}\""), 1);
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .set_lighting_mood_mix("uuid:lighting", "First", true)
            .unwrap_err();
        assert_error_unchanged(&editor, &before, error, "unsupported_mood_mix_scope");
    }
}

#[test]
fn lighting_metadata_mix_rejects_opaque_missing_or_ambiguous_identity_and_flags_atomically() {
    for (xml, code) in [
        (
            mix_xml().replace("SID=\"3\"", "SID=\"0002\""),
            "ambiguous_mood_sid",
        ),
        (
            mix_xml().replace("SID=\"3\"", "SID=\"opaque\""),
            "unsupported_mood_identity",
        ),
        (
            mix_xml().replacen("SID=\"+002\"", "", 1),
            "unsupported_mood_identity",
        ),
        (
            mix_xml().replacen("CID=\"+000\"", "CID=\"-1\"", 1),
            "unsupported_mood_identity",
        ),
        (
            mix_xml().replacen("SID=\"+002\"", "SID=\"2147483648\"", 1),
            "unsupported_mood_identity",
        ),
        (
            mix_xml().replacen("CID=\"+000\"", "CID=\"2147483648\"", 1),
            "unsupported_mood_identity",
        ),
        (
            mix_xml().replacen("MixMod=\"true\"", "MixMod=\"1\"", 1),
            "unsupported_mood_mix_value",
        ),
        (
            mix_xml().replacen("MixMod=\"true\"", "MixMod=\"TRUE\"", 1),
            "unsupported_mood_mix_value",
        ),
        (
            mix_xml().replacen("MixMod=\"true\"", "MixMod=\"opaque\"", 1),
            "unsupported_mood_mix_value",
        ),
        (
            mix_xml().replace("Name=\"Second\"", "Name=\"First\""),
            "ambiguous_or_missing_mood",
        ),
        (
            mix_xml().replace("</C>", "<LightscenesC/></C>"),
            "unsupported_mood_container_shape",
        ),
    ] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .set_lighting_mood_mix("uuid:lighting", "First", false)
            .unwrap_err();
        assert_error_unchanged(&editor, &before, error, code);
    }
}

#[test]
fn lighting_metadata_mix_omitted_cid_is_zero_without_identity_insertion() {
    let xml = mix_xml().replacen(" CID=\"+000\"", "", 1);
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "First", false)
        .unwrap();
    assert_eq!(edit.mood.cid, None);
    assert_eq!(edit.cid_effective, 0);
    assert_xml(
        &editor,
        &xml.replacen("MixMod=\"true\"", "MixMod=\"false\"", 1),
    );
    let before = editor.to_bytes().unwrap();
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "First", false)
        .unwrap();
    assert!(!edit.changed);
    assert_eq!(editor.to_bytes().unwrap(), before);
    let xml = xml.replacen(" MixMod=\"true\"", "", 1);
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "First", false)
        .unwrap();
    assert_eq!(edit.mood.cid, None);
    assert_eq!(edit.before, None);
    assert!(!edit.changed);
    assert_eq!(editor.to_bytes().unwrap(), before);
    let xml = xml.replacen("Name=\"First\"", "Name=\"First\" CID=\"future\"", 1);
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let error = editor
        .set_lighting_mood_mix("uuid:lighting", "First", true)
        .unwrap_err();
    assert_error_unchanged(&editor, &before, error, "unsupported_mood_identity");
}

#[test]
fn lighting_metadata_mix_missing_false_preserves_omission_then_edits_only_selected_flag() {
    let xml = mix_xml().replacen(" MixMod=\"true\"", "", 1);
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "First", false)
        .unwrap();
    assert_eq!(edit.before, None);
    assert!(!edit.before_effective && !edit.default && !edit.changed);
    assert_eq!(edit.after, None);
    assert_eq!(editor.to_bytes().unwrap(), before);
    let edit = editor
        .set_lighting_mood_mix("uuid:lighting", "First", true)
        .unwrap();
    assert!(edit.changed && edit.enabled);
    assert_eq!(edit.after.as_deref(), Some("true"));
    let expected = xml.replacen(
        "Future=\"opaque\">",
        "Future=\"opaque\" MixMod=\"true\">",
        1,
    );
    assert_xml(&editor, &expected);
    let xml = xml.replacen("CID=\"+000\"", "CID=\"12\"", 1);
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let error = editor
        .set_lighting_mood_mix("uuid:lighting", "First", false)
        .unwrap_err();
    assert_error_unchanged(&editor, &before, error, "role_dependent_mood_mix");
}
const SKIP_XML: &str = r#"<ControlList><C Type="LightController2" U="lighting">
  <LightscenesC Num="2" Outputs="18" Future="keep">
    <LightsceneC Name="A" UUID="a" SID="777" CID="opaque" MixMod="true" Q1="future"><SeqConf Future="keep"/></LightsceneC>
    <LightsceneC Name="B" SID="+0002" SkipNext="true" Q2="-1"/>
  </LightscenesC><LSConfig ScMv="opaque"/><HCL T="2" FROM="0"/>
</C></ControlList>"#;

#[test]
fn lighting_metadata_option_inspection_preserves_raw_defaults_and_special_row_projection() {
    let editor = ConfigEditor::load(SKIP_XML.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let options = editor
        .inspect_lighting_mood_options("uuid:lighting", "A")
        .unwrap();
    assert!(options.read_only);
    let skip = options.flags.iter().find(|f| f.key == "SkipNext").unwrap();
    assert_eq!(skip.value.stored, None);
    assert_eq!(skip.value.effective, Some(false));
    assert_eq!(skip.checkbox_checked, Some(true));
    let mix = options.flags.iter().find(|f| f.key == "MixMod").unwrap();
    assert_eq!(mix.value.stored.as_deref(), Some("true"));
    assert_eq!(mix.projection, "not_checkbox_row");
    assert_eq!(mix.checkbox_checked, None);
    assert_eq!(options.display_name.stored, None);
    assert_eq!(options.display_name.effective.as_deref(), Some(""));
    assert_eq!(options.icon_id.stored, None);
    assert_eq!(options.icon_id.effective, Some(0));
    assert_eq!(options.icon_id.default, Some(0));
    assert_eq!(options.icon_resolution, "unresolved_catalog_mapping");
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_option_inspection_keeps_role_and_unsupported_value_unresolved() {
    let xml = METADATA_XML.replace("SkipNext=\"false\"", "SkipNext=\"future\"");
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let options = editor
        .inspect_lighting_mood_options("uuid:lighting", "First")
        .unwrap();
    let mix = options.flags.iter().find(|f| f.key == "MixMod").unwrap();
    assert_eq!(mix.projection, "role_dependent");
    assert_eq!(mix.checkbox_checked, None);
    assert_eq!(options.icon_id.stored.as_deref(), Some("7"));
    assert_eq!(options.icon_id.effective, Some(7));
    let skip = options.flags.iter().find(|f| f.key == "SkipNext").unwrap();
    assert_eq!(skip.value.stored.as_deref(), Some("future"));
    assert_eq!(skip.value.effective, None);
    assert_eq!(skip.value.default, Some(false));
    assert_eq!(skip.checkbox_checked, None);
    assert_eq!(skip.projection, "unresolved_value");
    assert!(
        options
            .diagnostics
            .iter()
            .any(|d| d.code == "unsupported_mood_option_value")
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_option_icon_retains_unsupported_storage_without_input_inference() {
    let xml = METADATA_XML.replace("IID=\"7\"", "IID=\"future\"");
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let options = editor
        .inspect_lighting_mood_options("uuid:lighting", "First")
        .unwrap();
    assert_eq!(options.icon_id.stored.as_deref(), Some("future"));
    assert_eq!(options.icon_id.effective, None);
    assert_eq!(options.icon_id.default, Some(0));
    assert!(
        options
            .diagnostics
            .iter()
            .any(|d| d.code == "unsupported_mood_icon_id")
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_option_mix_projection_distinguishes_omitted_and_unsupported_identity() {
    for (extra, projection, checked) in [
        ("", "direct", Some(false)),
        ("CID=\"future\"", "unresolved_identity", None),
        ("CID=\"-1\"", "unresolved_identity", None),
        ("CID=\"12\"", "role_dependent", None),
    ] {
        let xml = SKIP_XML.replace("SID=\"+0002\"", &format!("SID=\"+0002\" {extra}"));
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let result = editor
            .inspect_lighting_mood_options("uuid:lighting", "B")
            .unwrap();
        let mix = result.flags.iter().find(|f| f.key == "MixMod").unwrap();
        assert_eq!(mix.projection, projection);
        assert_eq!(mix.checkbox_checked, checked);
        assert_eq!(mix.value.stored, None);
        assert_eq!(mix.value.effective, Some(false));
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_metadata_skip_uses_stored_default_and_inverse_checkbox_without_other_writes() {
    let mut editor = super::ConfigEditor::load(SKIP_XML.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let edit = editor
        .set_lighting_mood_skip("uuid:lighting", "A", false)
        .unwrap();
    assert_eq!(edit.before.stored, None);
    assert_eq!(edit.before.effective, Some(false));
    assert_eq!(edit.after, None);
    assert!(edit.checkbox_checked);
    assert!(!edit.changed);
    assert_eq!(editor.to_bytes().unwrap(), before);
    let edit = editor
        .set_lighting_mood_skip("uuid:lighting", "A", true)
        .unwrap();
    assert_eq!(edit.after.as_deref(), Some("true"));
    assert!(!edit.checkbox_checked);
    let expected = SKIP_XML.replace("Q1=\"future\">", "Q1=\"future\" SkipNext=\"true\">");
    assert_eq!(
        editor.to_bytes().unwrap(),
        super::ConfigEditor::load(expected.as_bytes())
            .unwrap()
            .to_bytes()
            .unwrap()
    );
    editor
        .set_lighting_mood_skip("uuid:lighting", "B", false)
        .unwrap();
    let expected = expected.replace(
        "SID=\"+0002\" SkipNext=\"true\"",
        "SID=\"+0002\" SkipNext=\"false\"",
    );
    assert_eq!(
        editor.to_bytes().unwrap(),
        super::ConfigEditor::load(expected.as_bytes())
            .unwrap()
            .to_bytes()
            .unwrap()
    );
}

#[test]
fn lighting_metadata_skip_rejects_unmodeled_flags_and_ambiguous_identity_atomically() {
    for xml in [
        SKIP_XML.replace("Name=\"A\"", "Name=\"A\" SkipNext=\"0\""),
        SKIP_XML.replace("Name=\"A\"", "Name=\"A\" SkipNext=\"opaque\""),
        SKIP_XML.replace("SID=\"777\"", "SID=\"0\""),
        SKIP_XML.replace("SID=\"+0002\"", "SID=\"+00777\""),
        SKIP_XML.replace("SID=\"+0002\"", "SID=\"future\""),
        SKIP_XML.replace("Name=\"B\"", "Name=\"A\""),
        SKIP_XML.replace("</C>", "<LightscenesC/></C>"),
    ] {
        let mut editor = super::ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .set_lighting_mood_skip("uuid:lighting", "A", true)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}
