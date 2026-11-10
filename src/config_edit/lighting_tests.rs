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
