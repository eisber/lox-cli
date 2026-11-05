use super::ConfigEditor;
use super::energy::*;

fn fixture() -> Vec<u8> {
    let mut xml = String::from(
        "<ControlList><C Type=\"EnergyManager2\" Title=\"Pilot\" U=\"a0000000-0000-0000-0000-000000000001\" 12hTF=\"true\" Future=\"keep\">\
         <Co K=\"O\" U=\"offset-wire\"/><Co K=\"MinSoc\" Def=\"0\" U=\"soc-wire\"/>\
         <Co K=\"MaxSpwr\" U=\"storage-wire\"/><Co K=\"OL1\" U=\"load-wire\"/>",
    );
    for index in 0..12 {
        let fields = match index {
            0 => "U=\"true\" MaxPwr=\"2.5\" ico=\"keep-icon\" Future=\"opaque\"",
            1 => "U=\"1\" SwPwr=\"2.6\" MaxPwr=\"2.5\" RT=\"30\" tpR=\"27\"",
            _ => "",
        };
        xml.push_str(&format!(
            "<Load Id=\"b0000000-0000-0000-0000-{index:012x}\" OIdx=\"{index}\" {fields}><Unknown key=\"{index}\"><Inner value=\"retain\"/></Unknown></Load>"
        ));
        if index == 0 {
            xml.push_str("<Interleaved keep=\"position\"/>");
        }
    }
    xml.push_str("</C><C Type=\"StateV\" U=\"actor\"><Co K=\"I\"><In Input=\"load-wire\"/></Co></C></ControlList>");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(xml.replace("><", ">\r\n<").as_bytes());
    bytes
}

fn editor() -> ConfigEditor {
    ConfigEditor::load(&fixture()).unwrap()
}

fn first_id() -> &'static str {
    "b0000000-0000-0000-0000-000000000000"
}

#[test]
fn energy_builtin_icon_selection_uses_exact_membership_and_canonical_native_id() {
    let mut editor = editor();
    let path = editor.require_one("Pilot").unwrap();
    for selection in [
        "electric-fan.svg",
        "builtin:electric-fan.svg",
        "00000094-00FF-0000-0000000000000000",
        "00000094-00FF-0000-0000-000000000000",
    ] {
        let mut expected = editor.root.clone();
        let block = expected.children[path[0]].as_mut_element().unwrap();
        let position = load_positions(block)[0];
        block.children[position]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("ico".into(), "00000094-00ff-0000-0000000000000000".into());
        editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    icon: EnergyIconChange::SetBuiltin(selection.into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(editor.root, expected);
        let inspection = editor.inspect_energy("Pilot").unwrap();
        assert_eq!(inspection.loads[0].icon_resolution.status, "known_builtin");
        assert_eq!(
            inspection.loads[0].icon_resolution.filename.as_deref(),
            Some("electric-fan.svg")
        );
        assert_eq!(inspection.builtin_icon_catalog.version, "20261009");
        let saved = editor.to_bytes().unwrap();
        let reloaded = ConfigEditor::load(&saved).unwrap();
        assert_eq!(
            reloaded.inspect_energy("Pilot").unwrap().loads[0].icon,
            inspection.loads[0].icon
        );
    }
}

#[test]
fn energy_icon_invalid_selections_fail_transactionally_with_actionable_diagnostics() {
    let mut editor = editor();
    for (selection, code) in [
        (
            "0000001d-00ff-0000-0000000000000000",
            "unknown_builtin_icon",
        ),
        ("awning-close.svg", "unknown_builtin_icon"),
        (
            "000000a0-00ff-0000-0000000000000000",
            "unknown_builtin_icon",
        ),
        (
            "0000094-000ff-0000-0000000000000000",
            "unknown_builtin_icon",
        ),
        ("Electric-fan.svg", "unknown_builtin_icon"),
        ("electric-fan", "unknown_builtin_icon"),
        (
            "project:b0000000-0000-0000-0000000000000000",
            "unsupported_custom_icon",
        ),
        (
            "b0000000-0000-0000-0000000000000000",
            "unknown_builtin_icon",
        ),
        ("", "invalid_icon_selection"),
        ("builtin:", "invalid_icon_selection"),
        (
            "00000000-0000-0000-0000000000000000",
            "explicit_icon_clear_required",
        ),
        (
            "00000000-0000-0000-0000-000000000000",
            "explicit_icon_clear_required",
        ),
    ] {
        let before = editor.to_bytes().unwrap();
        let error = editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    name: Some("must not persist".into()),
                    mode: Some(EnergyLoadMode::Analog),
                    rated_power: Some("8".into()),
                    icon: EnergyIconChange::SetBuiltin(selection.into()),
                    ..Default::default()
                },
            )
            .unwrap_err();
        let diagnostic = error.downcast_ref::<super::schema::Diagnostic>().unwrap();
        assert_eq!(diagnostic.code, code, "{selection}");
        assert!(diagnostic.message.contains("config energy icons"));
        assert!(diagnostic.message.contains("--no-icon"));
        assert_eq!(editor.to_bytes().unwrap(), before, "{selection}");
    }
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    preparation_seconds: Some(29),
                    icon: EnergyIconChange::Clear,
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_icon_clear_only_removes_matched_reference_and_preserves_definitions() {
    let xml = String::from_utf8(fixture()).unwrap().replace("</ControlList>",
        "<C Type=\"LoxCaption\" CaptionType=\"5\" SubType=\"5\" U=\"icon-caption\"><C Type=\"Icon\" U=\"f0000000-0000-0000-0000000000000001\" User=\"1\" Ix=\"4\" Icon=\"000102\" Future=\"opaque\"><Unknown keep=\"all\"/></C></C><C Type=\"StateV\" U=\"other-consumer\" ico=\"f0000000-0000-0000-0000000000000001\"/></ControlList>");
    let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
    let path = editor.require_one("Pilot").unwrap();
    let block = editor.get_element_mut(&path);
    for position in load_positions(block).into_iter().take(2) {
        block.children[position]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("ico".into(), "f0000000-0000-0000-0000000000000001".into());
    }
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().loads[0]
            .icon_resolution
            .status,
        "unresolved"
    );
    let mut expected = editor.root.clone();
    let block = expected.children[path[0]].as_mut_element().unwrap();
    let position = load_positions(block)[0];
    block.children[position]
        .as_mut_element()
        .unwrap()
        .attributes
        .shift_remove("ico");
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                icon: EnergyIconChange::Clear,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(editor.root, expected);
    assert!(
        editor.inspect_energy("Pilot").unwrap().loads[0]
            .icon
            .is_none()
    );
    let saved = editor.to_bytes().unwrap();
    let text = String::from_utf8(saved.clone()).unwrap();
    assert!(saved.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(!text.replace("\r\n", "").contains('\n'));
    assert!(text.contains("12hTF=\"true\""));
    assert!(text.contains("Input=\"load-wire\""));
    let before = editor.to_bytes().unwrap();
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                icon: EnergyIconChange::Clear,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_unknown_icon_spelling_survives_unrelated_edits_and_inspection() {
    for (reference, status) in [
        ("000000A0-00FF-0000-0000000000000000", "unresolved"),
        ("f0000000-0000-0000-0000000000000001", "unresolved"),
        ("keep-icon", "malformed_id"),
    ] {
        let mut editor = editor();
        let path = editor.require_one("Pilot").unwrap();
        let block = editor.get_element_mut(&path);
        let position = load_positions(block)[0];
        block.children[position]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("ico".into(), reference.into());
        let before = editor.to_bytes().unwrap();
        assert_eq!(
            editor.inspect_energy("Pilot").unwrap().loads[0]
                .icon_resolution
                .status,
            status
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
        editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    name: Some("unrelated edit".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            editor.inspect_energy("Pilot").unwrap().loads[0]
                .icon
                .as_deref(),
            Some(reference)
        );
    }
}

#[test]
fn energy_icon_catalog_corruption_is_an_explicit_error() {
    use super::icons::BuiltinIconCatalog;
    let source = serde_json::json!({
        "schema_version": 1,
        "catalog": {"version": "20400103", "archive_sha256": "a".repeat(64)},
        "icons": [
            {"native_id": "abcdef01-00ff-0000-0123456789abcdef", "filename": "independent.svg"},
            {"native_id": "abcdef02-00ff-0000-0123456789abcdef", "filename": "another.svg"}
        ],
    });
    for mutation in 0..7 {
        let mut invalid = source.clone();
        match mutation {
            0 => invalid["schema_version"] = 2.into(),
            1 => invalid["icons"][1]["native_id"] = "ABCDEF01-00FF-0000-0123456789ABCDEF".into(),
            2 => invalid["icons"][1]["filename"] = "independent.svg".into(),
            3 => invalid["icons"][0]["native_id"] = "abcdef01-00ff-0000-0123-456789abcdef".into(),
            4 => invalid["catalog"]["archive_sha256"] = "not-a-hash".into(),
            5 => invalid["icons"][0]["filename"] = "../asset.svg".into(),
            _ => invalid["catalog"]["version"] = "".into(),
        }
        let error = BuiltinIconCatalog::parse(&invalid.to_string()).unwrap_err();
        let diagnostic = error.downcast_ref::<super::schema::Diagnostic>().unwrap();
        assert_eq!(diagnostic.code, "invalid_icon_catalog");
        assert!(diagnostic.message.contains("regenerate"));
    }
    assert!(BuiltinIconCatalog::parse("{not JSON}").is_err());
}

fn load_positions(element: &xmltree::Element) -> Vec<usize> {
    element
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.as_element()
                .filter(|child| child.name == "Load")
                .map(|_| index)
        })
        .collect()
}

#[test]
fn energy_inspection_distinguishes_omission_and_unknown_binding() {
    let mut editor = editor();
    let inspection = editor.inspect_energy("Pilot").unwrap();
    assert!(inspection.load_edit_supported);
    assert_eq!(inspection.scalar_parameters[0].value.stored, None);
    assert_eq!(inspection.scalar_parameters[0].value.effective, Some(0.0));
    assert_eq!(
        inspection.scalar_parameters[1].value.stored.as_deref(),
        Some("0")
    );
    assert_eq!(inspection.loads[0].preparation_seconds.effective, Some(60));
    assert_eq!(inspection.loads[0].min_runtime_seconds.stored, None);
    assert_eq!(inspection.loads[2].used.effective, Some(false));
    assert_eq!(inspection.loads[2].unused_feedback, Some(true));
    assert_eq!(inspection.loads[1].deadline_label.as_deref(), Some("24:00"));
    assert!(inspection.loads[1].deadline_gui_verified);
    let path = editor.require_one("Pilot").unwrap();
    editor.get_element_mut(&path).children[0]
        .as_mut_element()
        .unwrap()
        .attributes
        .insert("K".into(), "Unknown".into());
    let inspection = editor.inspect_energy("Pilot").unwrap();
    assert!(!inspection.scalar_parameters[0].connector_present);
    assert_eq!(inspection.scalar_parameters[0].value.effective, None);
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "ambiguous_scalar_binding")
    );
}

#[test]
fn energy_signed_offset_is_parameter_and_bounds_are_atomic() {
    let mut editor = editor();
    assert_eq!(
        ConfigEditor::connector_io_type("EnergyManager2", "O").as_deref(),
        Some("P")
    );
    for raw in ["-1.25", "0", "2.75"] {
        editor
            .set_energy_parameter("Pilot", EnergyParameter::Offset, raw)
            .unwrap();
        assert_eq!(
            editor.inspect_energy("Pilot").unwrap().scalar_parameters[0]
                .value
                .effective,
            Some(raw.parse().unwrap())
        );
    }
    editor
        .set_energy_parameter("Pilot", EnergyParameter::Offset, "0")
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().scalar_parameters[0]
            .value
            .stored,
        None
    );
    for (parameter, raw) in [
        (EnergyParameter::MinimumStateOfCharge, "-1"),
        (EnergyParameter::MinimumStateOfCharge, "101"),
        (EnergyParameter::MaximumStoragePower, "-1"),
        (EnergyParameter::Offset, "NaN"),
        (EnergyParameter::Offset, "inf"),
        (EnergyParameter::Offset, "1e3"),
        (EnergyParameter::Offset, "1,25"),
    ] {
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .set_energy_parameter("Pilot", parameter, raw)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    for raw in ["0", "100"] {
        editor
            .set_energy_parameter("Pilot", EnergyParameter::MinimumStateOfCharge, raw)
            .unwrap();
    }
    editor
        .set_energy_parameter("Pilot", EnergyParameter::MaximumStoragePower, "5.5")
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().scalar_parameters[2]
            .value
            .effective,
        Some(5.5)
    );
    editor
        .set_energy_parameter("Pilot", EnergyParameter::MaximumStoragePower, "0")
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().scalar_parameters[2]
            .value
            .stored,
        None
    );
    let inspection = editor.inspect_energy("Pilot").unwrap();
    assert_eq!(inspection.loads[0].switch_on_power.stored, None);
    assert_eq!(inspection.loads[0].name.stored, None);
    assert_eq!(inspection.loads[0].attributes["Future"], "opaque");
}

#[test]
fn energy_all_deadlines_and_invalid_hours_are_atomic() {
    let mut editor = editor();
    let labels: Vec<_> = ["sunset", "dusk", "sunrise", "dawn"]
        .into_iter()
        .map(String::from)
        .chain((1..=24).map(|hour| format!("{hour:02}:00")))
        .collect();
    for (value, label) in labels.iter().enumerate() {
        editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    deadline: Some(label.parse().unwrap()),
                    ..Default::default()
                },
            )
            .unwrap();
        let inspection = editor.inspect_energy("Pilot").unwrap();
        assert_eq!(inspection.loads[0].deadline.effective, Some(value as i32));
        assert_eq!(inspection.loads[0].deadline_label.as_ref(), Some(label));
        assert!(inspection.loads[0].deadline_gui_verified);
        if value == 0 {
            assert!(inspection.loads[0].deadline.stored.is_none());
        }
    }
    for raw in ["00:00", "25:00", "01:30", "1:00", "01:00extra", "sunrisex"] {
        assert!(raw.parse::<EnergyDeadline>().is_err(), "{raw}");
    }
    for hour in [0, 25, u8::MAX] {
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .configure_energy_load(
                    "Pilot",
                    first_id(),
                    &EnergyLoadPatch {
                        name: Some("must not persist".into()),
                        deadline: Some(EnergyDeadline::Hour(hour)),
                        ..Default::default()
                    },
                )
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn energy_minutes_seconds_negative_runtime_and_overflow() {
    let mut editor = editor();
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                preparation_seconds: Some(86400),
                min_runtime_minutes: Some(-1),
                daily_minutes: Some(1440),
                deadline: Some(EnergyDeadline::Dusk),
                ..Default::default()
            },
        )
        .unwrap();
    let inspection = editor.inspect_energy("Pilot").unwrap();
    assert_eq!(inspection.loads[0].min_runtime_seconds.effective, Some(-60));
    assert_eq!(inspection.loads[0].daily_seconds.effective, Some(86400));
    assert_eq!(
        inspection.loads[0].preparation_seconds.effective,
        Some(86400)
    );
    assert_eq!(inspection.loads[0].deadline.effective, Some(1));
    for patch in [
        EnergyLoadPatch {
            name: Some("must not persist".into()),
            preparation_seconds: Some(29),
            ..Default::default()
        },
        EnergyLoadPatch {
            preparation_seconds: Some(86401),
            ..Default::default()
        },
        EnergyLoadPatch {
            min_runtime_minutes: Some(i64::MAX),
            ..Default::default()
        },
        EnergyLoadPatch {
            min_runtime_minutes: Some(i64::MIN),
            ..Default::default()
        },
        EnergyLoadPatch {
            min_runtime_minutes: Some(35_791_395),
            ..Default::default()
        },
        EnergyLoadPatch {
            daily_minutes: Some(-1),
            ..Default::default()
        },
        EnergyLoadPatch {
            daily_minutes: Some(1441),
            ..Default::default()
        },
    ] {
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .configure_energy_load("Pilot", first_id(), &patch)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                min_runtime_minutes: Some(35_791_394),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().loads[0]
            .min_runtime_seconds
            .effective,
        Some(2_147_483_640)
    );
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                preparation_seconds: Some(30),
                min_runtime_minutes: Some(0),
                daily_minutes: Some(0),
                deadline: Some(EnergyDeadline::Sunset),
                ..Default::default()
            },
        )
        .unwrap();
    let inspection = editor.inspect_energy("Pilot").unwrap();
    assert_eq!(inspection.loads[0].preparation_seconds.effective, Some(30));
    assert_eq!(inspection.loads[0].min_runtime_seconds.stored, None);
    assert_eq!(inspection.loads[0].daily_seconds.stored, None);
    assert_eq!(inspection.loads[0].deadline.stored, None);
}

#[test]
fn energy_power_zero_substitution_and_above_rated_are_allowed() {
    let mut editor = editor();
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                rated_power: Some("4".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let first = editor.inspect_energy("Pilot").unwrap().loads.remove(0);
    assert_eq!(first.switch_on_power.effective, Some(4.0));
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                rated_power: Some("2.5".into()),
                switch_on_power: Some("0".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().loads[0]
            .switch_on_power
            .effective,
        Some(2.5)
    );
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                switch_on_power: Some("2.6".into()),
                ..Default::default()
            },
        )
        .unwrap();
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                rated_power: Some("0".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let first = editor.inspect_energy("Pilot").unwrap().loads.remove(0);
    assert_eq!(first.rated_power.stored, None);
    assert_eq!(first.used.stored.as_deref(), Some("true"));
    assert_eq!(first.unused_feedback, Some(true));
    assert_eq!(first.switch_on_power.effective, Some(2.6));
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    name: Some("discard".into()),
                    rated_power: Some("5".into()),
                    switch_on_power: Some("-1".into()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_reorder_preserves_whole_records_unknown_children_and_wire_spellings() {
    let mut editor = editor();
    let before = editor.to_bytes().unwrap();
    let path = editor.require_one("Pilot").unwrap();
    let block = editor.get_element(&path);
    let positions = load_positions(block);
    let records: Vec<_> = positions
        .iter()
        .map(|&position| block.children[position].clone())
        .collect();
    let untouched: Vec<_> = block
        .children
        .iter()
        .enumerate()
        .filter(|(index, _)| !positions.contains(index))
        .map(|(index, node)| (index, node.clone()))
        .collect();
    editor.move_energy_load("Pilot", first_id(), 12).unwrap();
    let block = editor.get_element(&path);
    assert_eq!(block.children[positions[11]], records[0]);
    for (position, record) in positions[..11].iter().zip(&records[1..]) {
        assert_eq!(&block.children[*position], record);
    }
    for (position, node) in untouched {
        assert_eq!(block.children[position], node);
    }
    let output = editor.to_bytes().unwrap();
    assert!(output.starts_with(&[0xEF, 0xBB, 0xBF]));
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("12hTF=\"true\""));
    assert!(!text.contains("_12hTF="));
    assert!(text.contains("Input=\"load-wire\""));
    assert!(text.contains("\r\n"));
    assert!(!text.replace("\r\n", "").contains('\n'));
    editor.move_energy_load("Pilot", first_id(), 1).unwrap();
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_unsupported_shapes_and_ambiguous_ids_do_not_mutate() {
    for mutation in 0..8 {
        let mut editor = editor();
        let path = editor.require_one("Pilot").unwrap();
        let block = editor.get_element_mut(&path);
        let positions = load_positions(block);
        match mutation {
            0 => {
                block.children.remove(positions[11]);
            }
            1 => {
                block.children.push(block.children[positions[0]].clone());
            }
            2 => {
                block.children[positions[1]]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("Id".into(), first_id().into());
            }
            3 => {
                block.children[positions[1]]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("OIdx".into(), "0".into());
            }
            4 => {
                block.children[positions[0]]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("U".into(), first_id().into());
            }
            5 => {
                block.children[positions[0]]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .shift_remove("Id");
            }
            6 => {
                block.children[positions[0]]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .shift_remove("OIdx");
            }
            _ => {
                block.children[positions[0]]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("OIdx".into(), "12".into());
            }
        }
        let before = editor.to_bytes().unwrap();
        let inspection = editor.inspect_energy("Pilot").unwrap();
        assert!(!inspection.load_edit_supported);
        assert!(!inspection.diagnostics.is_empty());
        assert!(editor.move_energy_load("Pilot", first_id(), 2).is_err());
        assert!(
            editor
                .configure_energy_load(
                    "Pilot",
                    first_id(),
                    &EnergyLoadPatch {
                        name: Some("do not mutate".into()),
                        ..Default::default()
                    }
                )
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}

#[test]
fn energy_native_name_mode_markers_preserve_other_storage() {
    let mut editor = editor();
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                name: Some("Native CLI & <name>".into()),
                mode: Some(EnergyLoadMode::Analog),
                ..Default::default()
            },
        )
        .unwrap();
    let first = editor.inspect_energy("Pilot").unwrap().loads.remove(0);
    assert_eq!(first.name.effective.as_deref(), Some("Native CLI & <name>"));
    assert_eq!(first.name_customized.effective, Some(true));
    assert_eq!(first.mode_customized.effective, Some(true));
    assert_eq!(first.analog.effective, Some(true));
    assert_eq!(first.icon.as_deref(), Some("keep-icon"));
    assert_eq!(first.switch_on_power.stored, None);
    assert_eq!(first.child_elements, vec!["Unknown"]);
    let reloaded = ConfigEditor::load(&editor.to_bytes().unwrap()).unwrap();
    assert_eq!(
        reloaded.inspect_energy("Pilot").unwrap().loads[0]
            .name
            .effective,
        first.name.effective
    );
    editor
        .configure_energy_load(
            "Pilot",
            first_id(),
            &EnergyLoadPatch {
                mode: Some(EnergyLoadMode::Digital),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().loads[0]
            .analog
            .stored,
        None
    );
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .configure_energy_load(
                "Pilot",
                first_id(),
                &EnergyLoadPatch {
                    name: Some("invalid\0name".into()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_wrong_type_missing_id_and_invalid_priorities_fail_atomically() {
    let mut editor = editor();
    let before = editor.to_bytes().unwrap();
    assert!(editor.inspect_energy("uuid:actor").is_err());
    assert!(
        editor
            .set_energy_parameter("uuid:actor", EnergyParameter::Offset, "-1")
            .is_err()
    );
    for priority in [0, 13] {
        assert!(
            editor
                .move_energy_load("Pilot", first_id(), priority)
                .is_err()
        );
    }
    assert!(editor.move_energy_load("Pilot", "0", 2).is_err());
    assert!(
        editor
            .move_energy_load("Pilot", "c0000000-0000-0000-0000-000000000000", 2)
            .is_err()
    );
    assert!(
        editor
            .configure_energy_load("Pilot", first_id(), &EnergyLoadPatch::default())
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_scalar_edits_preserve_entire_other_records_and_fail_ambiguous_bindings() {
    let mut editor = editor();
    let path = editor.require_one("Pilot").unwrap();
    let mut expected = editor.root.clone();
    let expected_block = expected.children[path[0]].as_mut_element().unwrap();
    expected_block.children[0]
        .as_mut_element()
        .unwrap()
        .attributes
        .insert("Def".into(), "-1.25".into());
    editor
        .set_energy_parameter("Pilot", EnergyParameter::Offset, "-1.25")
        .unwrap();
    assert_eq!(editor.root, expected);
    let block = editor.get_element_mut(&path);
    block.children.push(block.children[0].clone());
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .set_energy_parameter("Pilot", EnergyParameter::Offset, "4")
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
    let mut other = editor.get_element(&path).clone();
    other.attributes.insert("U".into(), "other-identity".into());
    editor.root.children.push(xmltree::XMLNode::Element(other));
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .set_energy_parameter("Pilot", EnergyParameter::Offset, "4")
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_native_ids_and_aliases_preserve_records() {
    let bytes = include_bytes!("../../tests/ux/energy-manager2/synthetic.Loxone");
    let mut editor = ConfigEditor::load(bytes).unwrap();
    let native = "30000000-0000-0000-0000000000000001";
    let rfc = "30000000-0000-0000-0000-000000000001";
    assert!(
        editor
            .inspect_energy("Synthetic Energy")
            .unwrap()
            .load_edit_supported
    );
    editor
        .configure_energy_load(
            "Synthetic Energy",
            native,
            &EnergyLoadPatch {
                name: Some("Native edited".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let before = editor.inspect_energy("Synthetic Energy").unwrap();
    assert_eq!(before.loads[0].id.as_deref(), Some(native));
    assert_eq!(
        before.loads[0].name.effective.as_deref(),
        Some("Native edited")
    );
    assert_eq!(before.loads[0].attributes["PilotUnknown"], "retain");
    let bytes_before_move = editor.to_bytes().unwrap();
    editor.move_energy_load("Synthetic Energy", rfc, 2).unwrap();
    let after = editor.inspect_energy("Synthetic Energy").unwrap();
    assert_eq!(after.loads[1].id.as_deref(), Some(native));
    for prior in &before.loads {
        let moved = after.loads.iter().find(|load| load.id == prior.id).unwrap();
        assert_eq!(moved.attributes, prior.attributes);
    }
    editor
        .move_energy_load("Synthetic Energy", native, 1)
        .unwrap();
    assert_eq!(editor.to_bytes().unwrap(), bytes_before_move);
    editor
        .configure_energy_load(
            "Synthetic Energy",
            "30000000-0000-0000-000000000000000A",
            &EnergyLoadPatch {
                rated_power: Some("1".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Synthetic Energy").unwrap().loads[9]
            .id
            .as_deref(),
        Some("30000000-0000-0000-000000000000000a")
    );
    let before = editor.to_bytes().unwrap();
    assert!(
        editor
            .move_energy_load("Synthetic Energy", "3000000-00000-0000-0000000000000001", 2)
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
    let path = editor.require_one("Synthetic Energy").unwrap();
    let block = editor.get_element_mut(&path);
    let positions = load_positions(block);
    block.children[positions[1]]
        .as_mut_element()
        .unwrap()
        .attributes
        .insert("Id".into(), rfc.into());
    let before = editor.to_bytes().unwrap();
    let inspection = editor.inspect_energy("Synthetic Energy").unwrap();
    assert!(!inspection.load_edit_supported);
    assert!(
        inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "ambiguous_load_id")
    );
    assert!(
        editor
            .move_energy_load("Synthetic Energy", native, 2)
            .is_err()
    );
    assert_eq!(editor.to_bytes().unwrap(), before);
}

#[test]
fn energy_nonzero_decimal_underflow_fails_atomically() {
    let mut editor = editor();
    editor
        .set_energy_parameter("Pilot", EnergyParameter::Offset, "2.75")
        .unwrap();
    let tiny = format!("0.{}1", "0".repeat(399));
    for raw in [&tiny, &format!("-{tiny}")] {
        for parameter in [
            EnergyParameter::Offset,
            EnergyParameter::MinimumStateOfCharge,
            EnergyParameter::MaximumStoragePower,
        ] {
            let before = editor.to_bytes().unwrap();
            let error = editor
                .set_energy_parameter("Pilot", parameter, raw)
                .unwrap_err();
            assert_eq!(
                error
                    .downcast_ref::<super::schema::Diagnostic>()
                    .unwrap()
                    .code,
                "numeric_underflow"
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
        for patch in [
            EnergyLoadPatch {
                name: Some("discard".into()),
                switch_on_power: Some(raw.clone()),
                ..Default::default()
            },
            EnergyLoadPatch {
                rated_power: Some(raw.clone()),
                ..Default::default()
            },
        ] {
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .configure_energy_load("Pilot", first_id(), &patch)
                    .is_err()
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }
    editor
        .set_energy_parameter("Pilot", EnergyParameter::Offset, "-0.000")
        .unwrap();
    assert_eq!(
        editor.inspect_energy("Pilot").unwrap().scalar_parameters[0]
            .value
            .stored,
        None
    );
}
