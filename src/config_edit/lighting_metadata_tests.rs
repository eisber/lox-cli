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

#[test]
fn lighting_metadata_references_visit_exact_prefix_and_leave_opaque_tail_uninterpreted() {
    let xml = METADATA_XML
        .replace("SID=\"2\" CID=\"+012\"", "SID=\"+0002\" CID=\"opaque\"")
        .replace(
            "OpM=\"future\"",
            "OpM=\" +00 ,1,-02\" ScMv=\" &#9;+02 , -1 ,+2,opaque,,future\"",
        );
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    let result = editor
        .inspect_lighting_mood_references("uuid:lighting", "First")
        .unwrap();
    assert_eq!(result.mood.sid.as_deref(), Some("+0002"));
    assert_eq!(result.mood.cid.as_deref(), Some("opaque"));
    assert_eq!(result.selected_sid, 2);
    assert_eq!(result.stored_num.as_deref(), Some("009"));
    assert_eq!(result.direct_moods, 2);
    assert_eq!(result.opm.as_deref(), Some(" +00 ,1,-02"));
    assert_eq!(result.opm_count, Some(3));
    assert_eq!(result.status, "resolved");
    let visited = result.visited.unwrap();
    assert_eq!(
        visited
            .iter()
            .map(|r| (r.index, r.raw.as_str(), r.value, r.matches_selected_sid))
            .collect::<Vec<_>>(),
        [
            (0, " \t+02 ", 2, true),
            (1, " -1 ", -1, false),
            (2, "+2", 2, true)
        ]
    );
    let tail = result.unvisited_tail.unwrap();
    assert_eq!(tail.start_index, 3);
    assert_eq!(tail.token_count, 3);
    assert_eq!(tail.raw, "opaque,,future");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "stored_mood_count_not_reconciled")
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_references_missing_vectors_do_not_parse_unneeded_opaque_values() {
    for (attributes, opm, scmv) in [
        ("", None, None),
        ("OpM=\"\"", Some(""), None),
        ("OpM=\"future\"", Some("future"), None),
        ("OpM=\"future\" ScMv=\"\"", Some("future"), Some("")),
        ("ScMv=\"opaque,,tail\"", None, Some("opaque,,tail")),
        ("OpM=\"\" ScMv=\"opaque\"", Some(""), Some("opaque")),
    ] {
        let xml = METADATA_XML.replace("OpM=\"future\"", attributes);
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let result = editor
            .inspect_lighting_mood_references("uuid:lighting", "First")
            .unwrap();
        assert_eq!(result.status, "no_visited_references");
        assert!(result.visited.unwrap().is_empty());
        assert!(result.opm_count.is_none());
        assert!(result.unvisited_tail.is_none());
        assert_eq!(result.opm.as_deref(), opm);
        assert_eq!(result.scmv.as_deref(), scmv);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    let xml = METADATA_XML.replace(
        "<LSConfig OpM=\"future\"><Unknown SID=\"2\" Name=\"First\"/></LSConfig>",
        "",
    );
    let result = ConfigEditor::load(xml.as_bytes())
        .unwrap()
        .inspect_lighting_mood_references("uuid:lighting", "First")
        .unwrap();
    assert!(!result.lsconfig_present);
    assert_eq!(result.status, "no_visited_references");
}

#[test]
fn lighting_metadata_references_needed_unsupported_lists_are_unresolved_not_empty() {
    for (attributes, path) in [
        ("OpM=\"future\" ScMv=\"2\"", "LSConfig/@OpM"),
        ("OpM=\"0,future\" ScMv=\"2\"", "LSConfig/@OpM"),
        ("OpM=\"2147483648\" ScMv=\"2\"", "LSConfig/@OpM"),
        ("OpM=\" \" ScMv=\"2\"", "LSConfig/@OpM"),
        ("OpM=\"0,1\" ScMv=\"2,\"", "LSConfig/@ScMv"),
        ("OpM=\"0\" ScMv=\"future,2\"", "LSConfig/@ScMv"),
        ("OpM=\"0\" ScMv=\"2147483648\"", "LSConfig/@ScMv"),
    ] {
        let xml = METADATA_XML.replace("OpM=\"future\"", attributes);
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let result = editor
            .inspect_lighting_mood_references("uuid:lighting", "First")
            .unwrap();
        assert_eq!(result.status, "unresolved", "{attributes}");
        assert!(result.visited.is_none());
        assert!(result.opm_count.is_none());
        assert!(result.unvisited_tail.is_none());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| { d.code == "unsupported_lighting_selection_value" && d.path == path })
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    let xml = METADATA_XML.replace("OpM=\"future\"", "OpM=\"0,1,2\" ScMv=\"+02\"");
    let result = ConfigEditor::load(xml.as_bytes())
        .unwrap()
        .inspect_lighting_mood_references("uuid:lighting", "First")
        .unwrap();
    assert_eq!(result.opm_count, Some(3));
    assert_eq!(result.visited.unwrap().len(), 1);
    assert!(result.unvisited_tail.is_none());
}

#[test]
fn lighting_metadata_references_reject_ambiguous_containers_selection_and_numeric_identity() {
    for (xml, mood_ref, code) in [
        (
            METADATA_XML.replace("</C>", "<LSConfig/></C>"),
            "First",
            "unsupported_lighting_selection_shape",
        ),
        (
            METADATA_XML.replace("</C>", "<LightscenesC/></C>"),
            "First",
            "unsupported_mood_container_shape",
        ),
        (
            METADATA_XML.replace("Name=\"Second\"", "Name=\"First\""),
            "First",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.replace("Name=\"Second\"", "Name=\"2\""),
            "2",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.replace("SID=\"3\"", "SID=\"+02\""),
            "First",
            "ambiguous_mood_sid",
        ),
        (
            METADATA_XML.replace("SID=\"2\" CID", "CID"),
            "First",
            "unsupported_mood_identity",
        ),
        (
            METADATA_XML.replace("SID=\"2\" CID", "SID=\"0\" CID"),
            "First",
            "unsupported_mood_identity",
        ),
        (
            METADATA_XML.replace("SID=\"3\"", "SID=\"opaque\""),
            "First",
            "unsupported_mood_identity",
        ),
    ] {
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .inspect_lighting_mood_references("uuid:lighting", mood_ref)
            .unwrap_err();
        assert_eq!(error.downcast_ref::<Diagnostic>().unwrap().code, code);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_metadata_boolean_case_preserves_native_spelling_on_noop_and_inspection() {
    for (raw, effective) in [
        ("TRUE", true),
        ("tRuE", true),
        ("FALSE", false),
        ("FaLsE", false),
    ] {
        let xml = mix_xml()
            .replacen("MixMod=\"true\"", &format!("MixMod=\"{raw}\""), 1)
            .replacen("SkipNext=\"false\"", &format!("SkipNext=\"{raw}\""), 1)
            .replacen("ReadOnly=\"true\"", &format!("ReadOnly=\"{raw}\""), 1)
            .replacen("Sequencing=\"true\"", &format!("Sequencing=\"{raw}\""), 1)
            .replacen("Move=\"false\"", &format!("Move=\"{raw}\""), 1);
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let options = editor
            .inspect_lighting_mood_options("uuid:lighting", "First")
            .unwrap();
        for flag in &options.flags {
            assert_eq!(flag.value.stored.as_deref(), Some(raw));
            assert_eq!(flag.value.effective, Some(effective));
        }
        let skip = editor
            .set_lighting_mood_skip("uuid:lighting", "First", effective)
            .unwrap();
        assert!(!skip.changed);
        assert_eq!(skip.after.as_deref(), Some(raw));
        let mix = editor
            .set_lighting_mood_mix("uuid:lighting", "First", effective)
            .unwrap();
        assert!(!mix.changed);
        assert_eq!(mix.after.as_deref(), Some(raw));
        assert_eq!(editor.to_bytes().unwrap(), before);
        editor
            .set_lighting_mood_mix("uuid:lighting", "First", !effective)
            .unwrap();
        assert_xml(
            &editor,
            &xml.replacen(
                &format!("MixMod=\"{raw}\""),
                &format!("MixMod=\"{}\"", !effective),
                1,
            ),
        );
    }
}

#[test]
fn lighting_metadata_boolean_case_does_not_coerce_numeric_space_or_opaque_flags() {
    for raw in ["1", "0", " true", "false ", "yes", "future", "Trüe"] {
        let xml = mix_xml()
            .replacen("MixMod=\"true\"", &format!("MixMod=\"{raw}\""), 1)
            .replacen("SkipNext=\"false\"", &format!("SkipNext=\"{raw}\""), 1);
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let options = editor
            .inspect_lighting_mood_options("uuid:lighting", "First")
            .unwrap();
        for key in ["MixMod", "SkipNext"] {
            assert_eq!(
                options
                    .flags
                    .iter()
                    .find(|f| f.key == key)
                    .unwrap()
                    .value
                    .effective,
                None
            );
        }
        assert!(
            editor
                .set_lighting_mood_mix("uuid:lighting", "First", false)
                .is_err()
        );
        assert!(
            editor
                .set_lighting_mood_skip("uuid:lighting", "First", false)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

fn assert_error_unchanged(editor: &ConfigEditor, before: &[u8], error: anyhow::Error, code: &str) {
    assert_eq!(error.downcast_ref::<Diagnostic>().unwrap().code, code);
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn lighting_metadata_display_name_ascii_is_explicit_and_does_not_trim_or_transliterate() {
    let mut editor = ConfigEditor::load(METADATA_XML.as_bytes()).unwrap();
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .set_lighting_mood_display_name("uuid:lighting", "First", "Late Evening 2")
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
    let edit = editor
        .set_lighting_mood_display_name_ascii("uuid:lighting", "First", "Late Evening 2")
        .unwrap();
    assert_eq!(edit.after, "LATE EVENING 2");
    assert_eq!(edit.normalized_from.as_deref(), Some("Late Evening 2"));
    assert_xml(
        &editor,
        &METADATA_XML.replacen("SName=\"alias\"", "SName=\"LATE EVENING 2\"", 1),
    );
    let before = editor.to_bytes().unwrap();
    assert!(
        !editor
            .set_lighting_mood_display_name_ascii("uuid:lighting", "First", "late evening 2")
            .unwrap()
            .changed
    );
    for name in [" Late Evening", "Late Evening ", "Late_Evening", "Grün"] {
        assert!(
            editor
                .set_lighting_mood_display_name_ascii("uuid:lighting", "First", name)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn lighting_metadata_display_name_preview_is_immutable_and_matches_both_writers() {
    for (attribute, before_name, name, normalize, after, changed) in [
        ("", None, "", false, "", false),
        (" SName=\"\"", Some(""), "", false, "", false),
        (" SName=\"123\"", Some("123"), "123", false, "123", false),
        (" SName=\"alias\"", Some("alias"), "", false, "", true),
        ("", None, "NEW  NAME 2", false, "NEW  NAME 2", true),
        (
            " SName=\"Grün &amp; &quot;A,B&quot;\"",
            Some("Grün & \"A,B\""),
            "Late Evening 2",
            true,
            "LATE EVENING 2",
            true,
        ),
        (
            " SName=\"LATE EVENING 2\"",
            Some("LATE EVENING 2"),
            "late evening 2",
            true,
            "LATE EVENING 2",
            false,
        ),
        (" SName=\"ABC\"", Some("ABC"), "ABC", true, "ABC", false),
    ] {
        for identity in [
            "IID=\"7\"",
            "SID=\"+03\" CID=\"opaque\" IID=\"7\" UUID=\"\"",
        ] {
            let xml = METADATA_XML
                .replace(" SName=\"alias\"", attribute)
                .replacen(
                    "SID=\"2\" CID=\"+012\" IID=\"7\" UUID=\"first\"",
                    identity,
                    1,
                );
            let input = format!("\u{feff}{}", xml.replace('\n', "\r\n"));
            let editor = ConfigEditor::load(input.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_display_name("uuid:lighting", "First", name, normalize)
                .unwrap();
            assert_eq!(editor.to_bytes().unwrap(), before);
            assert_eq!(preview.edit.before.as_deref(), before_name);
            assert_eq!(preview.before_effective, before_name.unwrap_or(""));
            assert_eq!(preview.edit.after, after);
            assert_eq!(preview.edit.changed, changed);
            assert_eq!(
                preview.after_stored.as_deref(),
                if changed { Some(after) } else { before_name }
            );
            assert_eq!(
                preview.edit.normalized_from.as_deref(),
                (normalize && name != after).then_some(name)
            );
            let mut writer = ConfigEditor::load(input.as_bytes()).unwrap();
            let edit = if normalize {
                writer.set_lighting_mood_display_name_ascii("uuid:lighting", "First", name)
            } else {
                writer.set_lighting_mood_display_name("uuid:lighting", "First", name)
            }
            .unwrap();
            assert_eq!(
                serde_json::to_value(&preview.edit).unwrap(),
                serde_json::to_value(&edit).unwrap()
            );
            let mut expected = ConfigEditor::load(input.as_bytes()).unwrap();
            if changed {
                let path = expected
                    .lighting_metadata_mood_path("uuid:lighting", "First")
                    .unwrap();
                expected
                    .get_element_mut(&path)
                    .attributes
                    .insert("SName".into(), after.into());
            }
            assert_eq!(writer.to_bytes().unwrap(), expected.to_bytes().unwrap());
            if !changed {
                assert_eq!(writer.to_bytes().unwrap(), before);
            }
        }
    }
}

#[test]
fn lighting_metadata_display_name_preview_preserves_ordered_errors_and_unsupported_noops() {
    for normalize in [false, true] {
        for (xml, selector, reference, name, code) in [
            (
                METADATA_XML.to_owned(),
                "missing",
                "missing",
                "Bad!",
                "unsupported_mood_display_name",
            ),
            (
                METADATA_XML.replace("SName=\"alias\"", "SName=\"Grün\""),
                "uuid:lighting",
                "First",
                "Grün",
                "unsupported_mood_display_name",
            ),
            (
                METADATA_XML.replace("LightController2", "And"),
                "uuid:lighting",
                "missing",
                "ABC",
                "wrong_block_type",
            ),
            (
                METADATA_XML.replace("</C>", "<LightscenesC/></C>"),
                "uuid:lighting",
                "First",
                "",
                "unsupported_mood_container_shape",
            ),
            (
                METADATA_XML.replace("Name=\"Second\"", "Name=\"First\""),
                "uuid:lighting",
                "First",
                "",
                "ambiguous_or_missing_mood",
            ),
            (
                METADATA_XML.replace("Name=\"Second\"", "Name=\"2\""),
                "uuid:lighting",
                "2",
                "ABC",
                "ambiguous_or_missing_mood",
            ),
            (
                METADATA_XML.to_owned(),
                "uuid:lighting",
                "first",
                "ABC",
                "ambiguous_or_missing_mood",
            ),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_display_name(selector, reference, name, normalize)
                .unwrap_err();
            let writer = if normalize {
                editor.set_lighting_mood_display_name_ascii(selector, reference, name)
            } else {
                editor.set_lighting_mood_display_name(selector, reference, name)
            }
            .unwrap_err();
            assert_eq!(preview.to_string(), writer.to_string());
            assert_eq!(preview.downcast_ref::<Diagnostic>().unwrap().code, code);
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
        for name in [" ABC", "ABC ", "A_B", "A\tB", "A\nB", "50%", "\0"] {
            let mut editor = ConfigEditor::load(METADATA_XML.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_display_name("missing", "missing", name, normalize)
                .unwrap_err();
            let writer = if normalize {
                editor.set_lighting_mood_display_name_ascii("missing", "missing", name)
            } else {
                editor.set_lighting_mood_display_name("missing", "missing", name)
            }
            .unwrap_err();
            assert_eq!(preview.to_string(), writer.to_string());
            assert_eq!(
                preview.downcast_ref::<Diagnostic>().unwrap().code,
                "unsupported_mood_display_name"
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
    let xml = METADATA_XML.replace("SName=\"alias\"", "SName=\"lower\"");
    let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    assert!(
        editor
            .preview_lighting_mood_display_name("uuid:lighting", "First", "lower", false)
            .is_err()
    );
}

#[test]
fn lighting_metadata_rename_preview_is_immutable_and_matches_writer_without_identity_guards() {
    for (attributes, reference, name) in [
        (
            "Name=\"First\" SID=\"2\" CID=\"+012\" UUID=\"first\"",
            "First",
            "First",
        ),
        (
            "Name=\"Grün &amp; &quot;A,B&quot;\" SID=\"opaque\" CID=\"future\" UUID=\"a&amp;b\"",
            "opaque",
            "Second",
        ),
        ("SID=\"0\"", "0", "New  Name 2"),
        ("Name=\"First\"", "First", "New Name"),
        ("Name=\"\" SID=\"2\" CID=\"\" UUID=\"\"", "2", "New Name"),
        ("Name=\"First\" SID=\"+03\"", "First", "First"),
    ] {
        let xml = METADATA_XML.replacen(
            "Name=\"First\" SID=\"2\" CID=\"+012\" IID=\"7\" UUID=\"first\"",
            attributes,
            1,
        );
        let input = format!("\u{feff}{}", xml.replace('\n', "\r\n"));
        let editor = ConfigEditor::load(input.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_lighting_mood_rename("uuid:lighting", reference, name)
            .unwrap();
        assert_eq!(editor.to_bytes().unwrap(), before);
        let mut writer = ConfigEditor::load(input.as_bytes()).unwrap();
        let edit = writer
            .rename_lighting_mood("uuid:lighting", reference, name)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&preview).unwrap(),
            serde_json::to_value(&edit).unwrap()
        );
        let mut expected = ConfigEditor::load(input.as_bytes()).unwrap();
        let path = expected
            .lighting_metadata_mood_path("uuid:lighting", reference)
            .unwrap();
        expected
            .get_element_mut(&path)
            .attributes
            .insert("Name".into(), name.into());
        assert_eq!(writer.to_bytes().unwrap(), expected.to_bytes().unwrap());
        if !preview.changed {
            assert_eq!(writer.to_bytes().unwrap(), before);
        }
    }
}

#[test]
fn lighting_metadata_rename_preview_and_writer_share_ordered_errors_including_noops() {
    for (xml, selector, reference, name, code) in [
        (
            METADATA_XML.to_owned(),
            "missing",
            "missing",
            "Bad!",
            "unsupported_mood_name",
        ),
        (
            METADATA_XML.replace("Name=\"First\"", "Name=\"Grün\""),
            "uuid:lighting",
            "Grün",
            "Grün",
            "unsupported_mood_name",
        ),
        (
            METADATA_XML.replace("LightController2", "And"),
            "uuid:lighting",
            "missing",
            "New",
            "wrong_block_type",
        ),
        (
            METADATA_XML.replace("</C>", "<LightscenesC/></C>"),
            "uuid:lighting",
            "First",
            "First",
            "unsupported_mood_container_shape",
        ),
        (
            METADATA_XML.replace("LightscenesC", "UnknownContainer"),
            "uuid:lighting",
            "2",
            "New",
            "unsupported_mood_container_shape",
        ),
        (
            METADATA_XML.replace("Name=\"Second\"", "Name=\"First\""),
            "uuid:lighting",
            "First",
            "First",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.replace("Name=\"Second\"", "Name=\"2\""),
            "uuid:lighting",
            "2",
            "New",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.to_owned(),
            "uuid:lighting",
            "first",
            "First",
            "ambiguous_or_missing_mood",
        ),
        (
            METADATA_XML.to_owned(),
            "uuid:lighting",
            "Fir",
            "New",
            "ambiguous_or_missing_mood",
        ),
    ] {
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_lighting_mood_rename(selector, reference, name)
            .unwrap_err();
        let writer = editor
            .rename_lighting_mood(selector, reference, name)
            .unwrap_err();
        assert_eq!(preview.to_string(), writer.to_string());
        assert_eq!(preview.downcast_ref::<Diagnostic>().unwrap().code, code);
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    for name in [
        "",
        " ",
        " First",
        "First ",
        "First\tSecond",
        "A\nB",
        "A/B",
        "A_B",
        "50%",
        "\u{0}",
    ] {
        let mut editor = ConfigEditor::load(METADATA_XML.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let preview = editor
            .preview_lighting_mood_rename("missing", "missing", name)
            .unwrap_err();
        let writer = editor
            .rename_lighting_mood("missing", "missing", name)
            .unwrap_err();
        assert_eq!(preview.to_string(), writer.to_string());
        assert_eq!(
            preview.downcast_ref::<Diagnostic>().unwrap().code,
            "unsupported_mood_name"
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
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
fn lighting_metadata_mix_preview_is_immutable_and_matches_writer_without_normalization() {
    for (raw, effective) in [
        (None, false),
        (Some("false"), false),
        (Some("FaLsE"), false),
        (Some("true"), true),
        (Some("tRuE"), true),
    ] {
        let attribute = raw
            .map(|raw| format!("MixMod=\"{raw}\""))
            .unwrap_or_default();
        let mut xml = mix_xml().replacen("MixMod=\"true\"", &attribute, 1);
        if raw.is_none() {
            xml = xml.replacen(" CID=\"+000\"", "", 1);
        }
        for enabled in [false, true] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_mix("uuid:lighting", "First", enabled)
                .unwrap();
            assert_eq!(editor.to_bytes().unwrap(), before);
            assert_eq!(preview.before.as_deref(), raw);
            assert_eq!(preview.before_effective, effective);
            assert_eq!(preview.mood.sid.as_deref(), Some("+002"));
            assert_eq!(preview.mood.cid.as_deref(), raw.map(|_| "+000"));
            assert_eq!(preview.cid_effective, 0);
            let after = if enabled != effective {
                Some(enabled.to_string())
            } else {
                raw.map(str::to_string)
            };
            assert_eq!(preview.after, after);
            assert_eq!(preview.changed, enabled != effective);
            let written = editor
                .set_lighting_mood_mix("uuid:lighting", "First", enabled)
                .unwrap();
            assert_eq!(
                serde_json::to_value(preview).unwrap(),
                serde_json::to_value(written).unwrap()
            );
            let expected = if enabled == effective {
                xml.clone()
            } else if raw.is_none() {
                xml.replacen(
                    "Future=\"opaque\">",
                    "Future=\"opaque\" MixMod=\"true\">",
                    1,
                )
            } else {
                xml.replacen(&attribute, &format!("MixMod=\"{enabled}\""), 1)
            };
            assert_xml(&editor, &expected);
        }
    }
}

#[test]
fn lighting_metadata_mix_preview_and_writer_share_exact_failures_and_are_atomic() {
    let mut cases = vec![
        (mix_xml(), "missing".to_string()),
        (
            mix_xml().replace("Name=\"Second\"", "Name=\"First\""),
            "First".into(),
        ),
        (
            mix_xml().replace("Type=\"LightController2\"", "Type=\"And\""),
            "First".into(),
        ),
        (
            mix_xml().replace("</LightscenesC>", "</LightscenesC><LightscenesC/>"),
            "First".into(),
        ),
        (
            mix_xml().replace("SID=\"3\"", "SID=\"0002\""),
            "First".into(),
        ),
        (
            mix_xml().replace("SID=\"3\"", "SID=\"opaque\""),
            "First".into(),
        ),
    ];
    for sid in ["", "opaque", "2147483648", "-1", "0", "32", "778"] {
        let replacement = if sid.is_empty() {
            String::new()
        } else {
            format!("SID=\"{sid}\"")
        };
        cases.push((
            mix_xml().replacen("SID=\"+002\"", &replacement, 1),
            "First".into(),
        ));
    }
    for cid in ["opaque", "-1", "2147483648", "+012", "19"] {
        cases.push((
            mix_xml().replacen("CID=\"+000\"", &format!("CID=\"{cid}\""), 1),
            "First".into(),
        ));
    }
    for flag in ["1", "0", "", " true ", "future", "ＴＲＵＥ"] {
        cases.push((
            mix_xml().replacen("MixMod=\"true\"", &format!("MixMod=\"{flag}\""), 1),
            "First".into(),
        ));
    }
    for (xml, mood) in cases {
        for enabled in [false, true] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_mix("uuid:lighting", &mood, enabled)
                .unwrap_err();
            assert_eq!(editor.to_bytes().unwrap(), before);
            let writer = editor
                .set_lighting_mood_mix("uuid:lighting", &mood, enabled)
                .unwrap_err();
            assert_eq!(preview.to_string(), writer.to_string(), "{xml}/{enabled}");
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
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
fn lighting_metadata_skip_preview_is_immutable_and_matches_writer_without_normalization() {
    for (raw, effective, sid, cid) in [
        (None, false, "+000777", None),
        (Some("false"), false, "32", Some("12")),
        (Some("FaLsE"), false, "+0002", Some("opaque")),
        (Some("true"), true, "2147483647", Some("-1")),
        (Some("tRuE"), true, "778", Some("19")),
    ] {
        let attribute = raw
            .map(|raw| format!("SkipNext=\"{raw}\""))
            .unwrap_or_default();
        let cid_attribute = cid.map(|cid| format!(" CID=\"{cid}\"")).unwrap_or_default();
        let xml = METADATA_XML
            .replacen(
                "Name=\"First\" SID=\"2\" CID=\"+012\"",
                &format!("Name=\"First\" SID=\"{sid}\"{cid_attribute}"),
                1,
            )
            .replacen("SkipNext=\"false\"", &attribute, 1);
        for skip in [false, true] {
            let source = format!("\u{feff}{}", xml.replace('\n', "\r\n"));
            let mut editor = ConfigEditor::load(source.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_skip("uuid:lighting", "First", skip)
                .unwrap();
            assert_eq!(editor.to_bytes().unwrap(), before);
            assert_eq!(preview.mood.sid.as_deref(), Some(sid));
            assert_eq!(preview.mood.cid.as_deref(), cid);
            assert_eq!(preview.before.stored.as_deref(), raw);
            assert_eq!(preview.before.effective, Some(effective));
            assert_eq!(preview.checkbox_checked, !skip);
            assert_eq!(preview.changed, skip != effective);
            assert_eq!(
                preview.after,
                if skip != effective {
                    Some(skip.to_string())
                } else {
                    raw.map(str::to_string)
                }
            );
            let written = editor
                .set_lighting_mood_skip("uuid:lighting", "First", skip)
                .unwrap();
            assert_eq!(
                serde_json::to_value(preview).unwrap(),
                serde_json::to_value(written).unwrap()
            );
            let expected = if skip == effective {
                xml.clone()
            } else if raw.is_none() {
                xml.replacen(
                    "Future=\"opaque\">",
                    "Future=\"opaque\" SkipNext=\"true\">",
                    1,
                )
            } else {
                xml.replacen(&attribute, &format!("SkipNext=\"{skip}\""), 1)
            };
            assert_xml(
                &editor,
                &format!("\u{feff}{}", expected.replace('\n', "\r\n")),
            );
        }
    }
}

#[test]
fn lighting_metadata_skip_preview_and_writer_share_ordered_failures_and_are_atomic() {
    let mut cases = vec![
        (METADATA_XML.to_string(), "missing"),
        (
            METADATA_XML.replace("Name=\"Second\"", "Name=\"First\""),
            "First",
        ),
        (
            METADATA_XML.replace("Type=\"LightController2\"", "Type=\"And\""),
            "First",
        ),
        (
            METADATA_XML.replace("</LightscenesC>", "</LightscenesC><LightscenesC/>"),
            "First",
        ),
        (METADATA_XML.replace("SID=\"3\"", "SID=\"0002\""), "First"),
        (METADATA_XML.replace("SID=\"3\"", "SID=\"opaque\""), "First"),
        // Earlier identity guards must still win over an unsupported flag.
        (
            METADATA_XML
                .replace("SID=\"2\"", "SID=\"0\"")
                .replace("SkipNext=\"false\"", "SkipNext=\"0\""),
            "First",
        ),
        (
            METADATA_XML
                .replace("SID=\"3\"", "SID=\"0002\"")
                .replace("SkipNext=\"false\"", "SkipNext=\"0\""),
            "First",
        ),
    ];
    for sid in ["", "opaque", "2147483648", "-1", "0"] {
        let replacement = if sid.is_empty() {
            String::new()
        } else {
            format!("SID=\"{sid}\"")
        };
        cases.push((METADATA_XML.replacen("SID=\"2\"", &replacement, 1), "First"));
    }
    for raw in ["1", "0", "", " true ", "future", "ＴＲＵＥ"] {
        cases.push((
            METADATA_XML.replacen("SkipNext=\"false\"", &format!("SkipNext=\"{raw}\""), 1),
            "First",
        ));
    }
    for (xml, mood) in cases {
        for skip in [false, true] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            let preview = editor
                .preview_lighting_mood_skip("uuid:lighting", mood, skip)
                .unwrap_err();
            assert_eq!(editor.to_bytes().unwrap(), before);
            let writer = editor
                .set_lighting_mood_skip("uuid:lighting", mood, skip)
                .unwrap_err();
            assert_eq!(preview.to_string(), writer.to_string(), "{xml}/{skip}");
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
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
