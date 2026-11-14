use std::fs;

use predicates::prelude::*;
use serde_json::Value;
use xmltree::Element;

const MODE_A: &str = "11111111-1111-1111-1111-111111111111";
const MODE_B: &str = "22222222-2222-2222-2222-222222222222";
const NATIVE_MODE_A: &str = "00000000-0000-0004-1500000000000000";
const NATIVE_MODE_B: &str = "00000000-0000-0005-1500000000000000";
const RFC_MODE_A: &str = "00000000-0000-0004-1500-000000000000";

fn move_command(input: &std::path::Path, from: &str, to: &str, shift: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-move"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            RFC_MODE_A,
            "--from",
            from,
            "--to",
            to,
            "--shift-minutes",
            shift,
        ]);
    command
}

#[test]
fn daytimer_move_real_project_snaps_both_boundaries_and_preserves_every_other_byte() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = include_str!("eval/golden/configs/Schedule-Based AC Control.Loxone");
    fs::write(&input, source).unwrap();
    fs::write(&output, b"old destination").unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-move"])
        .arg(&input)
        .args([
            "uuid:1ecf58a7-03a3-70b0-ffff48a8846ceda9",
            "--mode-uuid",
            RFC_MODE_A,
            "--from",
            "06:00",
            "--to",
            "21:00",
            "--shift-minutes",
            "8",
            "-o",
            "json",
        ])
        .arg("--save-as")
        .arg(&output)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["move_change"]["new_from_minutes"], 375);
    assert_eq!(result["move_change"]["new_to_minutes"], 1275);
    assert_eq!(result["move_change"]["stored_mode_uuid"], NATIVE_MODE_A);
    assert_eq!(result["saved_to"], output.to_str().unwrap());
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(
        without_document_stamp(&fs::read_to_string(&output).unwrap()),
        without_document_stamp(&source.replacen(
            "<Entry From=\"360\" To=\"1260\" Ix=\"1\" V=\"1\"/>",
            "<Entry From=\"375\" To=\"1275\" Ix=\"1\" V=\"1\"/>",
            1,
        ))
    );
}

#[test]
fn daytimer_move_snaps_start_not_delta_and_saves_once_without_rewriting_metadata() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture()
        .replace("DateS=\"1\"", "DateS=\"2147483645\" 12hTF=\"true\"")
        .replace(
            "From=\"120\" To=\"180\" V=\"5\"",
            "From = '+0122' To='+0182' V=\"5\"",
        )
        .replace(
            "<Unknown Preserve=\"yes\"/>",
            "<Unknown Preserve=\"yes\" Preserve=\"yes\" TF=\"12h\"/>",
        );
    for (shift, from, to) in [("5", 120, 180), ("6", 135, 195), ("-10", 105, 165)] {
        fs::write(&input, &source).unwrap();
        move_command(&input, "02:02", "03:02", shift)
            .args(["-q", "-o", "csv", "--no-header"])
            .assert()
            .success()
            .stdout(format!(
                "\"timer\",{RFC_MODE_A},{NATIVE_MODE_A},122,182,{shift},{from},{to},true\n"
            ));
        let saved = fs::read_to_string(&input).unwrap();
        assert!(saved.contains("DateS=\"2147483646\""));
        assert_eq!(
            without_document_stamp(&saved),
            without_document_stamp(&source.replace(
                "From = '+0122' To='+0182'",
                &format!("From = '{from}' To='{to}'")
            ))
        );
    }
}

#[test]
fn daytimer_move_grid_and_clamp_noops_keep_exact_bytes_and_actual_saved_to() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (source, from, to, shift) in [
        (end_fixture(), "02:00", "03:00", "7"),
        (end_fixture(), "02:00", "03:00", "-7"),
        (
            end_fixture().replace(
                "From=\"120\" To=\"180\" V=\"5\"",
                "From=\"+0122\" To=\"+0182\" V=\"5\"",
            ),
            "02:02",
            "03:02",
            "0",
        ),
        (
            end_fixture()
                .replace("N=\"3\"", "N=\"2\"")
                .replace(
                    "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
                    "",
                )
                .replace(
                    "From=\"120\" To=\"180\" V=\"5\"",
                    "From=\"1432\" To=\"1440\" V=\"5\"",
                ),
            "23:52",
            "24:00",
            "8",
        ),
    ] {
        fs::write(&input, &source).unwrap();
        let result = move_command(&input, from, to, shift)
            .args(["-q", "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["saved_to"], Value::Null);
        assert_eq!(result["move_change"]["changed"], false);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        move_command(&input, from, to, shift)
            .arg("--save-as")
            .arg(&output)
            .args(["-q"])
            .assert()
            .success()
            .stdout("");
        assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
}

#[test]
fn daytimer_move_linked_clamp_and_midnight_omission_insertion_follow_native_grid() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for (source, old_from, old_to, shift, previous, replacement) in [
        (
            end_fixture(),
            "02:00",
            "03:00",
            "-121",
            "From=\"120\" To=\"180\"",
            " To=\"60\"",
        ),
        (
            end_fixture().replacen("From=\"120\" To=\"180\"", "To=\"180\"", 1),
            "00:00",
            "03:00",
            "8",
            "To=\"180\" V=\"5\" Marker=\"selected\"/>",
            "To=\"195\" V=\"5\" Marker=\"selected\" From=\"15\"/>",
        ),
        (
            end_fixture()
                .replace("N=\"3\"", "N=\"2\"")
                .replace(
                    "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
                    "",
                )
                .replace("To=\"180\" V=\"5\"", "To=\"128\" V=\"5\""),
            "02:00",
            "02:08",
            "1320",
            "From=\"120\" To=\"128\"",
            "From=\"1425\" To=\"1440\"",
        ),
    ] {
        fs::write(&input, &source).unwrap();
        move_command(&input, old_from, old_to, shift)
            .args(["-q"])
            .assert()
            .success()
            .stdout("");
        assert_eq!(
            without_document_stamp(&fs::read_to_string(&input).unwrap()),
            without_document_stamp(&source.replacen(previous, replacement, 1))
        );
    }
}

#[test]
fn daytimer_move_refuses_unmodeled_grid_overflow_overlap_reorder_and_unreachable_delta() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let missing = dir.path().join("missing.Loxone");
    let source = end_fixture();
    let overflow = source
        .replace("N=\"3\"", "N=\"2\"")
        .replace(
            "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
            "",
        )
        .replace("To=\"180\" V=\"5\"", "To=\"127\" V=\"5\"");
    for (source, to, shift) in [
        (&overflow, "02:07", "1320"),
        (&source, "03:00", "420"),
        (&source, "03:00", "480"),
        (&source, "03:00", "600"),
        (&source, "03:00", "-122"),
        (&source, "03:00", "1321"),
        (&source, "03:00", "2147483647"),
    ] {
        for destination in [&existing, &missing] {
            fs::write(&input, source).unwrap();
            fs::write(&existing, b"preserve destination").unwrap();
            move_command(&input, "02:00", to, shift)
                .arg("--save-as")
                .arg(destination)
                .args(["-o", "json"])
                .assert()
                .failure();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"preserve destination");
            assert!(!missing.exists());
        }
    }
}

#[test]
fn daytimer_move_admission_including_processing_instructions_precedes_noop_and_output() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = end_fixture();
    let mut refused = vec![
        source.replace("Type=\"DayTimer\"", "Type=\"daytimer\""),
        source.replace("Analog=\"false\"", "Analog=\"true\""),
        source.replace("Analog=\"false\"", "Analog=\"false\" M=\"4\""),
        source.replace("WF=\"16384\"", "WF=\"536887296\""),
        source.replace("WF=\"16384\"", "WF=\"16384\" uuidParent=\"parent\""),
        source.replace("WF=\"16384\"", "WF=\"16384\" IfT=\"false\""),
        source.replace("DateS=\"1\">", "DateS=\"1\"><Template IfT=\"false\"/>"),
        source.replace("<Loxone>", "<Loxone xmlns=\"urn:test\">"),
        source.replace("<Loxone>", "<!DOCTYPE Loxone><Loxone>"),
        source.replace("Marker=\"selected\"", "Marker=\"Wake 7am=ok\""),
        source.replace("From=\"120\"", "From=\"120\" From=\"120\""),
        source.replace(
            "<Unknown Preserve=\"yes\"/>",
            &format!("<Co U=\"{RFC_MODE_A}\"/>"),
        ),
        source.replace(
            "<Unknown Preserve=\"yes\"/>",
            "<Ref U=\"foreign\" U=\"foreign\"/>",
        ),
        format!("{source}<C Type=\"Mode\" U=\"foreign\"/>"),
    ];
    for instruction in ["<??>", "<?1bad?>", "<?a?b?>", "<?XmL ignored?>"] {
        refused.push(source.replace("<Loxone>", &format!("{instruction}<Loxone>")));
    }
    for stamp in ["", "+1", "-1", "2147483647", "2147483648", "１２"] {
        refused.push(source.replace("DateS=\"1\"", &format!("DateS=\"{stamp}\"")));
    }
    for source in refused {
        for shift in ["0", "8"] {
            for existing in [false, true] {
                fs::write(&input, &source).unwrap();
                if existing {
                    fs::write(&output, b"preserve destination").unwrap();
                }
                move_command(&input, "02:00", "03:00", shift)
                    .arg("--save-as")
                    .arg(&output)
                    .args(["-o", "json"])
                    .assert()
                    .failure();
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                if existing {
                    assert_eq!(fs::read(&output).unwrap(), b"preserve destination");
                    fs::remove_file(&output).unwrap();
                } else {
                    assert!(!output.exists());
                }
            }
        }
    }
}

#[test]
fn daytimer_move_valid_prolog_pi_missing_stamps_root_document_and_dry_run() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture()
        .replace("<Loxone>", "<?outside data?><!--keep-->")
        .replace("</Loxone>", "<?after data?>")
        .replace(" Date=\"2020-01-01 00:00:00\" DateS=\"1\"", "")
        .replace(
            "<Unknown Preserve=\"yes\"/>",
            "<?inside data?><Unknown Preserve=\"yes\"/>",
        );
    fs::write(&input, &source).unwrap();
    move_command(&input, "02:00", "03:00", "8")
        .args(["-q"])
        .assert()
        .success()
        .stdout("");
    let saved = fs::read_to_string(&input).unwrap();
    assert!(saved.contains(" DateS=\""));
    assert_eq!(
        without_document_stamp(&saved),
        source.replacen("From=\"120\" To=\"180\"", "From=\"135\" To=\"195\"", 1)
    );
    move_command(&dir.path().join("missing.Loxone"), "02:00", "03:00", "8")
        .args(["--dry-run", "-o", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("dry_run_not_supported"))
        .stdout(predicate::str::contains("Cannot read").not());
}

#[test]
fn daytimer_move_exclusive_staging_and_input_alias_refusals_preserve_all_files() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let staging = dir.path().join("output.Loxone.tmp");
    let source = end_fixture();
    fs::write(&input, &source).unwrap();
    for shift in ["0", "8"] {
        for existing in [false, true] {
            if existing {
                fs::write(&output, b"preserve destination").unwrap();
            }
            fs::hard_link(&input, &staging).unwrap();
            move_command(&input, "02:00", "03:00", shift)
                .arg("--save-as")
                .arg(&output)
                .args(["-o", "json"])
                .assert()
                .failure()
                .stdout(predicate::str::contains("staging"));
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&staging).unwrap(), source.as_bytes());
            fs::remove_file(&staging).unwrap();
            if existing {
                assert_eq!(fs::read(&output).unwrap(), b"preserve destination");
                fs::remove_file(&output).unwrap();
            } else {
                assert!(!output.exists());
            }
        }
        move_command(&input, "02:00", "03:00", shift)
            .arg("--save-as")
            .arg(dir.path().join(".").join("input.Loxone"))
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
}

fn start_command(input: &std::path::Path, from: &str, start: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-set-start"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            RFC_MODE_A,
            "--from",
            from,
            "--to",
            "03:00",
            "--new-from",
            start,
        ]);
    command
}

fn assert_start_staging_refusal(input: &std::path::Path, destination: &std::path::Path) {
    let source = fs::read(input).unwrap();
    let previous_destination = fs::read(destination).ok();
    for start in ["02:01", "02:00"] {
        start_command(input, "02:00", start)
            .arg("--save-as")
            .arg(destination)
            .args(["-o", "json"])
            .assert()
            .failure()
            .stdout(predicate::str::contains("staging"));
        assert_eq!(fs::read(input).unwrap(), source);
        assert_eq!(fs::read(destination).ok(), previous_destination);
    }
}

#[test]
fn daytimer_start_staging_input_collision_refuses_changed_and_noop_save_as() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("output.Loxone.tmp");
    let destination = dir.path().join("output.Loxone");
    fs::write(&input, end_fixture()).unwrap();
    assert_start_staging_refusal(&input, &destination);
    fs::write(&destination, b"preserve existing destination").unwrap();
    assert_start_staging_refusal(&input, &destination);
}

#[test]
fn daytimer_start_staging_foreign_file_and_hardlink_are_never_overwritten() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let destination = dir.path().join("output.Loxone");
    let staging = dir.path().join("output.Loxone.tmp");
    let source = end_fixture();
    fs::write(&input, &source).unwrap();
    fs::write(&destination, b"preserve existing destination").unwrap();
    fs::write(&staging, b"foreign staging sentinel").unwrap();
    assert_start_staging_refusal(&input, &destination);
    assert_eq!(fs::read(&staging).unwrap(), b"foreign staging sentinel");
    fs::remove_file(&staging).unwrap();
    fs::hard_link(&input, &staging).unwrap();
    assert_start_staging_refusal(&input, &destination);
    assert_eq!(fs::read(&staging).unwrap(), source.as_bytes());
}

#[test]
fn daytimer_start_staging_relative_and_case_aliases_refuse_before_output() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let subdirectory = dir.path().join("subdirectory");
    fs::create_dir(&subdirectory).unwrap();
    let input = dir.path().join("output.Loxone.tmp");
    let destination = dir.path().join(".").join("output.Loxone");
    fs::write(&input, end_fixture()).unwrap();
    assert_start_staging_refusal(
        &subdirectory.join("..").join("output.Loxone.tmp"),
        &destination,
    );
    #[cfg(windows)]
    assert_start_staging_refusal(&dir.path().join("OUTPUT.LOXONE.TMP"), &destination);
}

#[cfg(unix)]
#[test]
fn daytimer_start_staging_symlink_never_follows_or_removes_foreign_storage() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let destination = dir.path().join("output.Loxone");
    let staging = dir.path().join("output.Loxone.tmp");
    fs::write(&input, end_fixture()).unwrap();
    symlink(&input, &staging).unwrap();
    assert_start_staging_refusal(&input, &destination);
    assert_eq!(fs::read_link(&staging).unwrap(), input);
    fs::remove_file(&staging).unwrap();
    let missing = dir.path().join("foreign-missing");
    symlink(&missing, &staging).unwrap();
    assert_start_staging_refusal(&input, &destination);
    assert_eq!(fs::read_link(&staging).unwrap(), missing);
    assert!(!missing.exists());
}

#[test]
fn daytimer_start_save_as_input_path_alias_refuses_but_hardlink_destination_is_safe() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture();
    fs::write(&input, &source).unwrap();
    for start in ["02:01", "02:00"] {
        start_command(&input, "02:00", start)
            .arg("--save-as")
            .arg(dir.path().join(".").join("input.Loxone"))
            .args(["-o", "json"])
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    }
    let destination = dir.path().join("output.Loxone");
    fs::hard_link(&input, &destination).unwrap();
    start_command(&input, "02:00", "02:01")
        .arg("--save-as")
        .arg(&destination)
        .args(["-q"])
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(
        without_document_stamp(&fs::read_to_string(&destination).unwrap()),
        without_document_stamp(&source.replace(
            "From=\"120\" To=\"180\" V=\"5\"",
            "From=\"121\" To=\"180\" V=\"5\""
        ))
    );
}

#[test]
fn daytimer_start_real_project_preserves_nonzero_wf_and_every_unselected_byte() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = include_str!("eval/golden/configs/Schedule-Based AC Control.Loxone");
    fs::write(&input, source).unwrap();
    let result = assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-set-start"])
        .arg(&input)
        .args([
            "uuid:1ecf58a7-03a3-70b0-ffff48a8846ceda9",
            "--mode-uuid",
            NATIVE_MODE_A,
            "--from",
            "06:00",
            "--to",
            "21:00",
            "--new-from",
            "05:59",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["time_change"]["new_from_minutes"], 359);
    assert_eq!(result["time_change"]["entry"]["to_minutes"], 1260);
    assert_eq!(result["saved_to"], input.to_str().unwrap());
    assert_eq!(
        without_document_stamp(&fs::read_to_string(&input).unwrap()),
        without_document_stamp(&source.replacen(
            "<Entry From=\"360\" To=\"1260\" Ix=\"1\" V=\"1\"/>",
            "<Entry From=\"359\" To=\"1260\" Ix=\"1\" V=\"1\"/>",
            1,
        ))
    );
}

#[test]
fn daytimer_start_minute_edits_and_midnight_removal_preserve_raw_metadata() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture()
        .replace("DateS=\"1\"", "DateS=\"1\" 12hTF=\"true\"")
        .replace(
            "From=\"120\" To=\"180\" V=\"5\"",
            "From = '+0120' To=\"+0180\" V=\"5\"",
        )
        .replace(
            "<Unknown Preserve=\"yes\"/>",
            "<Unknown Preserve=\"yes\" Preserve=\"yes\" TF=\"12h\" Text=\"Wake 7am=ok\"/>",
        );
    for (start, replacement) in [
        ("01:59", "From = '119'"),
        ("02:01", "From = '121'"),
        ("00:00", ""),
    ] {
        fs::write(&input, &source).unwrap();
        start_command(&input, "02:00", start)
            .args(["-q"])
            .assert()
            .success()
            .stdout("");
        let saved = fs::read_to_string(&input).unwrap();
        assert_eq!(
            without_document_stamp(&saved),
            without_document_stamp(&source.replace("From = '+0120'", replacement))
        );
    }
}

#[test]
fn daytimer_start_omitted_zero_inserts_positive_from_and_noops_are_exact() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    for zero in ["", "From=\"+0000\" "] {
        let source = end_fixture()
            .replace(
                "From=\"120\" To=\"180\" V=\"5\"",
                &format!("{zero}To=\"180\" V=\"5\""),
            )
            .replace("DateS=\"1\"", "DateS=\"0000000001\"");
        fs::write(&input, &source).unwrap();
        let result = start_command(&input, "00:00", "00:00")
            .args(["-q", "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["saved_to"], Value::Null);
        assert_eq!(result["time_change"]["changed"], false);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let result = start_command(&input, "00:00", "00:00")
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["saved_to"], output.to_str().unwrap());
        assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
        start_command(&input, "00:00", "00:01")
            .arg("--save-as")
            .arg(&output)
            .args(["-q", "-o", "csv", "--no-header"])
            .assert()
            .success()
            .stdout(format!(
                "\"timer\",{RFC_MODE_A},{NATIVE_MODE_A},0,180,1,true\n"
            ));
        let expected = if zero.is_empty() {
            source.replace("Marker=\"selected\"/>", "Marker=\"selected\" From=\"1\"/>")
        } else {
            source.replace("From=\"+0000\"", "From=\"1\"")
        };
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(
            without_document_stamp(&fs::read_to_string(&output).unwrap()),
            without_document_stamp(&expected)
        );
    }
}

#[test]
fn daytimer_start_rejects_coupled_to_and_strict_previous_neighbor_bounds_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let missing = dir.path().join("missing.Loxone");
    let source = end_fixture().replace("N=\"3\"", "N=\"4\"").replace(
        "<Entry From=\"120\" To=\"180\" V=\"5\"",
        "<Entry From=\"30\" To=\"60\" V=\"1\"/><Entry From=\"120\" To=\"180\" V=\"5\"",
    );
    for start in [
        "00:00", "00:59", "01:00", "03:00", "03:01", "24:00", "24:01", "2:01", "02:01:00",
    ] {
        for destination in [&existing, &missing] {
            fs::write(&input, &source).unwrap();
            fs::write(&existing, b"preserve destination").unwrap();
            start_command(&input, "02:00", start)
                .arg("--save-as")
                .arg(destination)
                .args(["-o", "json"])
                .assert()
                .failure();
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"preserve destination");
            assert!(!missing.exists());
        }
    }
    start_command(&input, "02:00", "01:01")
        .args(["-q"])
        .assert()
        .success()
        .stdout("");
    assert_eq!(
        without_document_stamp(&fs::read_to_string(&input).unwrap()),
        without_document_stamp(&source.replace(
            "From=\"120\" To=\"180\" V=\"5\"",
            "From=\"61\" To=\"180\" V=\"5\""
        ))
    );
}

#[test]
fn daytimer_start_admission_precedes_noop_and_any_destination_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let missing = dir.path().join("missing.Loxone");
    let source = end_fixture();
    let mut refused = vec![
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A),
        source.replace("Type=\"Document\"", "Type=\"document\""),
        source.replace(
            "Type=\"DayTimer\" U=\"timer\"",
            "Type=\"daytimer\" U=\"timer\"",
        ),
        source.replace("<Loxone>", "<Loxone xmlns=\"urn:test\">"),
        source.replace("<Loxone>", "<!DOCTYPE Loxone><Loxone>"),
        format!("{source}<C Type=\"Mode\" U=\"{RFC_MODE_A}\"/>"),
        source.replace("DateS=\"1\">", "DateS=\"1\"><Template IfT=\"false\"/>"),
        source.replace("DateS=\"1\">", "DateS=\"1\"><Template IfT=\"true\"/>"),
        source.replace("WF=\"16384\"", "WF=\"536887296\""),
        source.replace("WF=\"16384\"", "WF=\"16384\" uuidParent=\"parent\""),
        source.replace(
            "From=\"120\" To=\"180\" V=\"5\"",
            "From=\"120\" From=\"120\" To=\"180\" V=\"5\"",
        ),
        source.replace("Marker=\"selected\"", "Marker=\"Wake 7am=ok\""),
        source.replace(
            &format!("U=\"{NATIVE_MODE_A}\""),
            &format!("U=\"{NATIVE_MODE_A}\" U=\"{NATIVE_MODE_A}\""),
        ),
        source.replace(
            "</Loxone>",
            &format!("<C Type=\"And\" U=\"{RFC_MODE_A}\"/></Loxone>"),
        ),
        source.replace(
            "</Loxone>",
            &format!(
                "<C Type=\"And\" U=\"{}\"/></Loxone>",
                NATIVE_MODE_A.to_uppercase()
            ),
        ),
        source.replace("Analog=\"false\"", "Analog=\"true\""),
        source.replace("Analog=\"false\"", "Analog=\"false\" M=\"4\""),
    ];
    for stamp in [
        "",
        "+1",
        "-1",
        "2147483647",
        "2147483648",
        "invalid",
        "１２",
    ] {
        refused.push(source.replace("DateS=\"1\"", &format!("DateS=\"{stamp}\"")));
    }
    for source in refused {
        for start in ["02:00", "02:01"] {
            for destination in [&existing, &missing] {
                fs::write(&input, &source).unwrap();
                fs::write(&existing, b"preserve destination").unwrap();
                start_command(&input, "02:00", start)
                    .arg("--save-as")
                    .arg(destination)
                    .args(["-o", "json"])
                    .assert()
                    .failure();
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"preserve destination");
                assert!(!missing.exists());
            }
        }
    }
}

#[test]
fn daytimer_start_missing_stamps_root_document_midnight_end_and_dry_run() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture()
        .replace("<Loxone>", "")
        .replace("</Loxone>", "")
        .replace(" Date=\"2020-01-01 00:00:00\" DateS=\"1\"", "")
        .replace("N=\"3\"", "N=\"2\"")
        .replace(
            "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
            "",
        )
        .replace("To=\"180\" V=\"5\"", "To=\"1440\" V=\"5\"");
    fs::write(&input, &source).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-set-start"])
        .arg(&input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            NATIVE_MODE_A,
            "--from",
            "02:00",
            "--to",
            "24:00",
            "--new-from",
            "00:00",
            "-q",
        ])
        .assert()
        .success()
        .stdout("");
    let saved = fs::read_to_string(&input).unwrap();
    assert!(saved.contains(" DateS=\""));
    assert_eq!(
        without_document_stamp(&saved),
        source.replace("From=\"120\" To=\"1440\" V=\"5\"", " To=\"1440\" V=\"5\"")
    );
    start_command(&dir.path().join("missing.Loxone"), "02:00", "02:01")
        .args(["--dry-run", "-o", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("dry_run_not_supported"))
        .stdout(predicate::str::contains("Cannot read").not());
}

#[test]
fn daytimer_start_rejects_global_owner_native_case_and_rfc_aliases_before_noop() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    let native = "abcdefab-cdef-abcd-efabcdefabcdefab";
    let rfc = "abcdefab-cdef-abcd-efab-cdefabcdefab";
    for duplicate in [native.to_owned(), native.to_uppercase(), rfc.to_uppercase()] {
        let source = end_fixture()
            .replace("U=\"timer\"", &format!("U=\"{native}\""))
            .replace(
                "</Loxone>",
                &format!("<C Type=\"And\" U=\"{duplicate}\"/></Loxone>"),
            );
        for start in ["02:00", "02:01"] {
            fs::write(&input, &source).unwrap();
            fs::write(&output, b"preserve destination").unwrap();
            assert_cmd::cargo::cargo_bin_cmd!("lox")
                .env("LOX_TELEMETRY", "0")
                .args(["config", "timer-entry-set-start"])
                .arg(&input)
                .args([
                    &format!("uuid:{native}"),
                    "--mode-uuid",
                    RFC_MODE_A,
                    "--from",
                    "02:00",
                    "--to",
                    "03:00",
                    "--new-from",
                    start,
                ])
                .arg("--save-as")
                .arg(&output)
                .args(["-o", "json"])
                .assert()
                .failure()
                .stdout(predicate::str::contains(
                    "Ambiguous DayTimer owner identity",
                ));
            assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
            assert_eq!(fs::read(&output).unwrap(), b"preserve destination");
        }
    }
}

fn end_command(input: &std::path::Path, end: &str) -> assert_cmd::Command {
    end_command_from(input, "02:00", end)
}

fn end_command_from(input: &std::path::Path, from: &str, end: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-set-end"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            RFC_MODE_A,
            "--from",
            from,
            "--to",
            "03:00",
            "--new-to",
            end,
        ]);
    command
}

fn end_fixture() -> String {
    let source = native_fixture(NATIVE_MODE_A, NATIVE_MODE_A);
    let rows = [
        "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
        "<Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/>",
        "<Entry From=\"120\" To=\"180\" V=\"5\" Marker=\"selected\"/>",
    ];
    let mut sorted = source;
    for row in rows {
        sorted = sorted.replace(row, "");
    }
    sorted.replace(
        "<Unknown Preserve=\"yes\"/>",
        &format!(
            "{}{}{}<Unknown Preserve=\"yes\"/>",
            rows[2], rows[0], rows[1]
        ),
    )
}

fn without_document_stamp(source: &str) -> String {
    let document = source.find("Type=\"Document\"").unwrap();
    let start = source[..document].rfind("<C").unwrap();
    let end = document + source[document..].find('>').unwrap();
    let mut tag = source[start..end].to_owned();
    for key in ["Date", "DateS"] {
        let prefix = format!(" {key}=\"");
        if let Some(position) = tag.find(&prefix) {
            let value = position + prefix.len();
            let end = value + tag[value..].find('"').unwrap() + 1;
            tag.replace_range(position..end, "");
        }
    }
    format!("{}{tag}{}", &source[..start], &source[end..])
}

#[test]
fn daytimer_end_real_native_fixture_keeps_wf_metadata_and_all_other_bytes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = include_str!("eval/golden/configs/Schedule-Based AC Control.Loxone");
    fs::write(&input, source).unwrap();
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    let result = command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-set-end"])
        .arg(&input)
        .args([
            "uuid:1ecf58a7-03a3-70b0-ffff48a8846ceda9",
            "--mode-uuid",
            NATIVE_MODE_A,
            "--from",
            "06:00",
            "--to",
            "21:00",
            "--new-to",
            "20:59",
            "-o",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["time_change"]["new_to_minutes"], 1259);
    assert_eq!(result["time_change"]["changed"], true);
    let saved = fs::read_to_string(&input).unwrap();
    assert_eq!(
        without_document_stamp(&saved),
        without_document_stamp(&source.replacen(
            "<Entry From=\"360\" To=\"1260\" Ix=\"1\" V=\"1\"/>",
            "<Entry From=\"360\" To=\"1259\" Ix=\"1\" V=\"1\"/>",
            1,
        ))
    );
    assert_ne!(saved, source);
}

#[test]
fn daytimer_end_changes_only_to_and_stamp_and_keeps_raw_noop_and_duplicates() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = end_fixture()
        .replace("To=\"180\" V=\"5\"", "To=\"+0180\" V=\"5\"")
        .replace(
            "<Unknown Preserve=\"yes\"/>",
            "<Unknown Preserve=\"yes\" Preserve=\"yes\" Text=\"Wake 7am=ok\"/>",
        )
        .replace("Title=\"Other\"", "Title=\"Other\" Title=\"Other\"");
    fs::write(&input, &source).unwrap();
    end_command(&input, "03:00")
        .arg("--save-as")
        .arg(&output)
        .args(["-q", "-o", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"changed\":false"));
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    end_command(&input, "03:01")
        .args(["-q", "-o", "csv", "--no-header"])
        .assert()
        .success()
        .stdout(format!(
            "\"timer\",{RFC_MODE_A},{NATIVE_MODE_A},120,180,181,true\n"
        ));
    let saved = fs::read_to_string(&input).unwrap();
    assert_eq!(
        without_document_stamp(&saved),
        without_document_stamp(&source.replace("To=\"+0180\"", "To=\"181\""))
    );
    end_command(&output, "02:01")
        .args(["-q"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn daytimer_end_refuses_invalid_or_collateral_ranges_without_any_output_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    for end in [
        "00:00", "02:00", "01:59", "24:01", "3:00", "03:00:00", "10:00", "10:30", "11:30",
    ] {
        let source = end_fixture();
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        end_command(&input, end)
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
        let missing = dir.path().join("missing.Loxone");
        end_command(&input, end)
            .arg("--save-as")
            .arg(&missing)
            .assert()
            .failure();
        assert!(!missing.exists());
    }
}

#[test]
fn daytimer_end_admission_precedes_noop_for_unsorted_template_and_raw_conflicts() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    for source in [
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A),
        end_fixture().replace("Type=\"Document\"", "Type=\"document\""),
        end_fixture()
            .replace("<Loxone>", "<Loxone><Nested>")
            .replace("</Loxone>", "</Nested></Loxone>"),
        end_fixture().replace("DateS=\"1\">", "DateS=\"1\"><Template IfT=\"true\"/>"),
        end_fixture().replace("DateS=\"1\">", "DateS=\"1\"><Template IfT=\"false\"/>"),
        end_fixture().replace("WF=\"16384\"", "WF=\"536887296\""),
        end_fixture().replace("WF=\"16384\"", "WF=\"16384\" uuidParent=\"parent\""),
        end_fixture().replace("To=\"180\" V=\"5\"", "To=\"180\" To=\"180\" V=\"5\""),
        end_fixture().replace("Marker=\"selected\"", "Marker=\"Wake 7am=ok\""),
        end_fixture().replace("DateS=\"1\"", "DateS=\"1\" DateS=\"1\""),
        end_fixture().replace(
            "<Unknown Preserve=\"yes\"/>",
            "<C Type=\"And\" U=\"timer\"/><Unknown Preserve=\"yes\"/>",
        ),
        end_fixture().replace("<Loxone>", "<Loxone xmlns=\"urn:test\">"),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        end_command(&input, "03:00")
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
}

#[test]
fn daytimer_end_dates_guard_rejects_bad_stamps_before_noop_or_any_destination_write() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let missing = dir.path().join("missing.Loxone");
    for stamp in ["2147483647", "-1", "2147483648", "invalid"] {
        let source = end_fixture().replace("DateS=\"1\"", &format!("DateS=\"{stamp}\""));
        for end in ["03:01", "03:00"] {
            for destination in [None, Some(&existing), Some(&missing)] {
                fs::write(&input, &source).unwrap();
                fs::write(&existing, b"existing destination").unwrap();
                let mut command = end_command(&input, end);
                if let Some(destination) = destination {
                    command.arg("--save-as").arg(destination);
                }
                command
                    .args(["-o", "json"])
                    .assert()
                    .failure()
                    .stdout(predicate::str::contains(
                        "incrementable nonnegative signed32 Document DateS",
                    ));
                assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"existing destination");
                assert!(!missing.exists());
            }
        }
    }
}

#[test]
fn daytimer_end_dates_guard_keeps_valid_raw_noops_and_allows_last_safe_increment() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for stamp in ["0", "0000000001", "2147483646"] {
        let source = end_fixture()
            .replace("DateS=\"1\"", &format!("DateS=\"{stamp}\""))
            .replace("To=\"180\" V=\"5\"", "To=\"+0180\" V=\"5\"");
        fs::write(&input, &source).unwrap();
        end_command(&input, "03:00")
            .arg("--save-as")
            .arg(&output)
            .args(["-q", "-o", "json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"changed\":false"));
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
        end_command(&input, "03:01")
            .arg("--save-as")
            .arg(&output)
            .args(["-q"])
            .assert()
            .success()
            .stdout("");
        let saved = fs::read_to_string(&output).unwrap();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(
            without_document_stamp(&saved),
            without_document_stamp(&source.replace("To=\"+0180\"", "To=\"181\""))
        );
        let parsed = parse(saved.as_bytes());
        let seconds = parsed.get_child("C").unwrap().attributes["DateS"]
            .parse::<i32>()
            .unwrap();
        assert!(seconds >= 0);
        if stamp == "2147483646" {
            assert_eq!(seconds, i32::MAX);
        }
    }
}

#[test]
fn daytimer_end_root_document_adds_missing_stamps_and_dry_run_never_reads_input() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture()
        .replace("<Loxone>", "")
        .replace("</Loxone>", "")
        .replace(" Date=\"2020-01-01 00:00:00\" DateS=\"1\"", "");
    fs::write(&input, &source).unwrap();
    end_command(&input, "03:01")
        .args(["-q"])
        .assert()
        .success()
        .stdout("");
    let saved = fs::read_to_string(&input).unwrap();
    assert!(saved.contains(" DateS=\""));
    assert_eq!(
        without_document_stamp(&saved),
        source.replace("To=\"180\" V=\"5\"", "To=\"181\" V=\"5\"")
    );
    end_command(&dir.path().join("missing.Loxone"), "03:01")
        .args(["--dry-run", "-o", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("dry_run_not_supported"))
        .stdout(predicate::str::contains("Cannot read").not());
}

#[test]
fn daytimer_end_supports_final_midnight_bound_without_recounting_or_new_start() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = end_fixture().replace("N=\"3\"", "N=\"2\"").replace(
        "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
        "",
    );
    fs::write(&input, &source).unwrap();
    end_command(&input, "24:00")
        .args(["-o", "json"])
        .assert()
        .success();
    assert_eq!(
        without_document_stamp(&fs::read_to_string(&input).unwrap()),
        without_document_stamp(&source.replace("To=\"180\" V=\"5\"", "To=\"1440\" V=\"5\""))
    );
    end_command(&input, "03:00")
        .args(["--new-from", "01:00"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--new-from"));
    let zero = source.replace("From=\"120\" To=\"180\" V=\"5\"", "To=\"180\" V=\"5\"");
    fs::write(&input, &zero).unwrap();
    end_command_from(&input, "00:00", "00:01")
        .args(["-q"])
        .assert()
        .success()
        .stdout("");
    assert_eq!(
        without_document_stamp(&fs::read_to_string(&input).unwrap()),
        without_document_stamp(&zero.replace("To=\"180\" V=\"5\"", "To=\"1\" V=\"5\""))
    );
}

fn fixture() -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"Document\" Date=\"2020-01-01 00:00:00\" DateS=\"1\"><C Type=\"DayTimer\" U=\"timer\" Title=\"Schedule\" Analog=\"false\" DefValue=\"0\" N=\"3\" Modes=\"{MODE_A},{MODE_B}\" WF=\"16384\" Unknown=\"keep\"><Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/><Co K=\"AQ\" U=\"output\"><In Input=\"wire\"/></Co><Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/><Entry From=\"120\" To=\"180\" V=\"5\" Marker=\"selected\"/><Unknown Preserve=\"yes\"/></C><C Type=\"DayTimer\" U=\"other\" Title=\"Other\" N=\"1\"><Entry To=\"300\" V=\"9\"/></C><C Type=\"Mode\" U=\"{MODE_A}\" Title=\"Mode A\"/><C Type=\"Mode\" U=\"{MODE_B}\" Title=\"Mode B\"/></C></Loxone>\r\n"
    )
}

fn command(input: &std::path::Path, from: &str, to: &str) -> assert_cmd::Command {
    command_for_mode(input, MODE_A, from, to)
}

fn command_for_mode(
    input: &std::path::Path,
    mode: &str,
    from: &str,
    to: &str,
) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-remove"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            mode,
            "--from",
            from,
            "--to",
            to,
        ]);
    command
}

fn parse(bytes: &[u8]) -> Element {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    Element::parse(bytes).unwrap()
}

#[test]
fn daytimer_delete_selects_mode_and_interval_not_xml_order_and_preserves_survivors() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let original = fixture();
    fs::write(&input, &original).unwrap();
    let result = command(&input, "02:00", "03:00")
        .arg("--save-as")
        .arg(&output)
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(result["removed"]["remaining_entries"], 2);
    assert_eq!(result["removed"]["entry"]["mode_uuid"], MODE_A);
    assert_eq!(
        result["removed"]["removed_attributes"]["Marker"],
        "selected"
    );
    assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    let saved = fs::read(&output).unwrap();
    assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(
        saved
            .iter()
            .enumerate()
            .all(|(index, byte)| { *byte != b'\n' || (index > 0 && saved[index - 1] == b'\r') })
    );
    let mut expected = parse(original.as_bytes());
    let document = expected.get_mut_child("C").unwrap();
    let timer = document.get_mut_child("C").unwrap();
    timer.children.retain(|node| {
        node.as_element().is_none_or(|element| {
            element.attributes.get("Marker").map(String::as_str) != Some("selected")
        })
    });
    timer.attributes.insert("N".into(), "2".into());
    let mut actual = parse(&saved);
    for root in [&mut actual, &mut expected] {
        let doc = root.get_mut_child("C").unwrap();
        doc.attributes.shift_remove("Date");
        doc.attributes.shift_remove("DateS");
    }
    assert_eq!(actual, expected);
    assert!(!String::from_utf8(saved).unwrap().contains("DateS=\"1\""));
}

#[test]
fn daytimer_delete_last_entry_omits_zero_count_and_keeps_default_and_mode_references() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = fixture()
        .replace("N=\"3\"", "N=\"1\"")
        .replace(
            "<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>",
            "",
        )
        .replace(
            "<Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/>",
            "",
        );
    fs::write(&input, &source).unwrap();
    command(&input, "02:00", "03:00")
        .args(["-o", "csv", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(format!("\"timer\",{MODE_A},{MODE_A},120,180,0\n"));
    let saved = parse(&fs::read(&input).unwrap());
    let timer = saved.get_child("C").unwrap().get_child("C").unwrap();
    assert!(!timer.attributes.contains_key("N"));
    assert_eq!(timer.attributes["DefValue"], "0");
    assert_eq!(timer.attributes["Modes"], format!("{MODE_A},{MODE_B}"));
    assert!(timer.get_child("Entry").is_none());
    let before = fs::read(&input).unwrap();
    command(&input, "02:00", "03:00").assert().failure();
    assert_eq!(fs::read(&input).unwrap(), before);
}

fn native_fixture(stored_mode: &str, object_mode: &str) -> String {
    fixture()
        .replace(
            &format!("Modes=\"{MODE_A},{MODE_B}\""),
            &format!("Modes=\"{stored_mode},{NATIVE_MODE_B}\""),
        )
        .replace(
            &format!("Type=\"Mode\" U=\"{MODE_A}\""),
            &format!("Type=\"Mode\" U=\"{object_mode}\""),
        )
        .replace(
            &format!("Type=\"Mode\" U=\"{MODE_B}\""),
            &format!("Type=\"Mode\" U=\"{NATIVE_MODE_B}\""),
        )
}

#[test]
fn daytimer_native_mode_ids_match_aliases_and_case_without_rewriting_spelling() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (stored, object, request) in [
        (NATIVE_MODE_A, NATIVE_MODE_A, NATIVE_MODE_A),
        (NATIVE_MODE_A, RFC_MODE_A, RFC_MODE_A),
        (
            "ABCDEF01-ABCD-0004-15000000ABCDEF00",
            "abcdef01-abcd-0004-1500-0000abcdef00",
            "aBcDeF01-AbCd-0004-15000000aBcDeF00",
        ),
    ] {
        let source = native_fixture(stored, object);
        fs::write(&input, &source).unwrap();
        let result = command_for_mode(&input, request, "02:00", "03:00")
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["removed"]["entry"]["mode_uuid"], request);
        assert_eq!(result["removed"]["stored_mode_uuid"], stored);
        assert_eq!(
            result["removed"]["removed_attributes"]["Marker"],
            "selected"
        );
        assert_eq!(result["removed"]["remaining_entries"], 2);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let mut expected = parse(source.as_bytes());
        let timer = expected
            .get_mut_child("C")
            .unwrap()
            .get_mut_child("C")
            .unwrap();
        timer.children.retain(|node| {
            node.as_element().is_none_or(|element| {
                element.attributes.get("Marker").map(String::as_str) != Some("selected")
            })
        });
        timer.attributes.insert("N".into(), "2".into());
        let mut actual = parse(&fs::read(&output).unwrap());
        for root in [&mut actual, &mut expected] {
            let document = root.get_mut_child("C").unwrap();
            document.attributes.shift_remove("Date");
            document.attributes.shift_remove("DateS");
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn daytimer_native_last_entry_and_mixed_alias_duplicates_keep_atomic_boundaries() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    let source = native_fixture(NATIVE_MODE_A, NATIVE_MODE_A)
        .replace("N=\"3\"", "N=\"1\"")
        .replace("<Entry From=\"600\" To=\"660\" V=\"3\" Marker=\"late\"/>", "")
        .replace("<Entry From=\"120\" To=\"180\" Ix=\"1\" V=\"4\" NeedActivate=\"true\" Marker=\"other-mode\"/>", "");
    fs::write(&input, &source).unwrap();
    command_for_mode(&input, RFC_MODE_A, "02:00", "03:00")
        .args(["-o", "csv", "--no-header", "-q"])
        .assert()
        .success()
        .stdout(format!(
            "\"timer\",{RFC_MODE_A},{NATIVE_MODE_A},120,180,0\n"
        ));
    let saved = parse(&fs::read(&input).unwrap());
    let timer = saved.get_child("C").unwrap().get_child("C").unwrap();
    assert!(!timer.attributes.contains_key("N"));
    assert!(timer.get_child("Entry").is_none());
    assert_eq!(timer.attributes["DefValue"], "0");
    assert_eq!(
        timer.attributes["Modes"],
        format!("{NATIVE_MODE_A},{NATIVE_MODE_B}")
    );
    for invalid in [
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A).replace(
            &format!("Modes=\"{NATIVE_MODE_A},{NATIVE_MODE_B}\""),
            &format!("Modes=\"{NATIVE_MODE_A},{RFC_MODE_A}\""),
        ),
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A).replace(
            &format!("<C Type=\"Mode\" U=\"{NATIVE_MODE_A}\" Title=\"Mode A\"/>"),
            &format!("<C Type=\"Mode\" U=\"{NATIVE_MODE_A}\" Title=\"Mode A\"/><C Type=\"Mode\" U=\"{RFC_MODE_A}\"/>"),
        ),
        native_fixture(NATIVE_MODE_A, NATIVE_MODE_A)
            .replace(NATIVE_MODE_B, "00000000-0000-0000-0000000000000000"),
    ] {
        fs::write(&input, &invalid).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        command_for_mode(&input, NATIVE_MODE_A, "02:00", "03:00")
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), invalid.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
    let missing = dir.path().join("missing.Loxone");
    for malformed in [
        "00000000000000041500000000000000",
        "urn:uuid:00000000-0000-0004-1500-000000000000",
        "{00000000-0000-0004-1500-000000000000}",
        "00000000-0000-0004-1500000000000000 ",
        "0000000-00000-0004-1500000000000000",
        "00000000-0000-0004-150000000000000g",
        "00000000-0000-0000-0000000000000000",
    ] {
        command_for_mode(&missing, malformed, "02:00", "03:00")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Cannot read").not());
    }
}

#[test]
fn daytimer_delete_rejects_unsafe_storage_without_touching_input_or_existing_destination() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("existing.Loxone");
    for source in [
        fixture().replace("N=\"3\"", "N=\"2\""),
        fixture().replace("Type=\"DayTimer\" U=\"timer\"", "Type=\"And\" U=\"timer\""),
        fixture().replace("N=\"3\"", "M=\"4\" N=\"3\""),
        fixture().replace("From=\"120\" To=\"180\"", "From=\"600\" To=\"660\""),
        fixture().replace("From=\"120\" To=\"180\"", "From=\"660\" To=\"720\""),
        fixture().replace(
            "V=\"5\" Marker=\"selected\"",
            "Ix=\"2\" V=\"5\" Marker=\"selected\"",
        ),
        fixture().replace(
            &format!("Type=\"Mode\" U=\"{MODE_B}\""),
            &format!("Type=\"And\" U=\"{MODE_B}\""),
        ),
        fixture().replace("Analog=\"false\"", "Analog=\"true\""),
        fixture().replace(" N=\"3\"", ""),
        fixture().replace(
            &format!("Modes=\"{MODE_A},{MODE_B}\""),
            &format!("Modes=\"{MODE_A},{MODE_A}\""),
        ),
        fixture().replace(
            &format!("Modes=\"{MODE_A},{MODE_B}\""),
            &format!("Modes=\"urn:uuid:{MODE_A},{MODE_B}\""),
        ),
        fixture().replace(
            &format!("Type=\"Mode\" U=\"{MODE_A}\""),
            &format!("Type=\"Mode\" U=\"urn:uuid:{MODE_A}\""),
        ),
        fixture().replace(MODE_B, "00000000-0000-0000-0000-000000000000"),
        fixture().replace(" Modes=", " UnsupportedModes="),
    ] {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        command(&input, "02:00", "03:00")
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
}

#[test]
fn daytimer_delete_uses_omitted_zero_identity_and_dry_run_fails_closed_before_io() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for source in [
        fixture(),
        fixture().replace(" Analog=\"false\" DefValue=\"0\"", ""),
    ] {
        let source = source.replace("From=\"120\" To=\"180\" V=\"5\"", "To=\"180\" V=\"5\"");
        fs::write(&input, source).unwrap();
        command(&input, "00:00", "03:00")
            .arg("-q")
            .assert()
            .success()
            .stdout("");
    }
    let missing = dir.path().join("missing.Loxone");
    command(&missing, "00:00", "03:00")
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("dry-run not supported"))
        .stderr(predicate::str::contains("Cannot read").not());
    for (from, to) in [("2:00", "03:00"), ("23:00", "01:00"), ("24:01", "24:02")] {
        command(&missing, from, to)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Cannot read").not());
    }
}

fn activation_command(input: &std::path::Path, mode: &str, from: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "timer-entry-set-activation"])
        .arg(input)
        .args([
            "uuid:timer",
            "--mode-uuid",
            mode,
            "--from",
            from,
            "--to",
            "03:00",
        ]);
    command
}

#[test]
fn daytimer_activation_changes_only_selected_flag_without_recounting_or_uuid_rewrite() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (attribute, flag, expected_required) in [
        ("", "--required", true),
        (" NeedActivate=\"FaLsE\"", "--required", true),
        (" NeedActivate=\"TrUe\"", "--no-required", false),
    ] {
        let source = native_fixture(NATIVE_MODE_A, RFC_MODE_A)
            .replace("N=\"3\"", "N=\"+03\"")
            .replace(
                "V=\"5\" Marker=\"selected\"",
                &format!("V=\"5\"{attribute} Marker=\"selected\""),
            );
        fs::write(&input, &source).unwrap();
        let result = activation_command(&input, RFC_MODE_A, "02:00")
            .arg(flag)
            .arg("--save-as")
            .arg(&output)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(result["activation"]["changed"], true);
        assert_eq!(
            result["activation"]["activation_required"],
            expected_required
        );
        assert_eq!(result["activation"]["entry"]["mode_uuid"], RFC_MODE_A);
        assert_eq!(result["activation"]["stored_mode_uuid"], NATIVE_MODE_A);
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        let saved = fs::read(&output).unwrap();
        assert!(saved.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(
            saved
                .iter()
                .enumerate()
                .all(|(i, b)| *b != b'\n' || saved[i - 1] == b'\r')
        );
        let mut expected = parse(source.as_bytes());
        let timer = expected
            .get_mut_child("C")
            .unwrap()
            .get_mut_child("C")
            .unwrap();
        let entry = timer
            .children
            .iter_mut()
            .filter_map(|n| n.as_mut_element())
            .find(|e| e.attributes.get("Marker").map(String::as_str) == Some("selected"))
            .unwrap();
        if expected_required {
            entry
                .attributes
                .insert("NeedActivate".into(), "true".into());
        } else {
            entry.attributes.shift_remove("NeedActivate");
        }
        let mut actual = parse(&saved);
        for root in [&mut expected, &mut actual] {
            let doc = root.get_mut_child("C").unwrap();
            doc.attributes.shift_remove("Date");
            doc.attributes.shift_remove("DateS");
        }
        assert_eq!(actual, expected);
        assert!(!String::from_utf8(saved).unwrap().contains("DateS=\"1\""));
    }
}

#[test]
fn daytimer_activation_semantic_noops_preserve_exact_bytes_spelling_and_stamp() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    for (attribute, flag) in [
        ("", "--no-required"),
        (" NeedActivate=\"FALSE\"", "--no-required"),
        (" NeedActivate=\"tRuE\"", "--required"),
    ] {
        let source = fixture()
            .replace(" Analog=\"false\" DefValue=\"0\"", "")
            .replace(
                "From=\"120\" To=\"180\" V=\"5\"",
                &format!("To=\"180\" V=\"5\"{attribute}"),
            );
        fs::write(&input, &source).unwrap();
        activation_command(&input, MODE_A, "00:00")
            .arg(flag)
            .args(["-o", "csv", "--no-header", "-q"])
            .assert()
            .success()
            .stdout(format!(
                "\"timer\",{MODE_A},{MODE_A},0,180,{},{},false\n",
                flag == "--required",
                flag == "--required",
            ));
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        fs::write(&output, b"old destination").unwrap();
        activation_command(&input, MODE_A, "00:00")
            .arg(flag)
            .arg("--save-as")
            .arg(&output)
            .arg("-q")
            .assert()
            .success()
            .stdout("");
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    }
}

#[test]
fn daytimer_activation_rejects_malformed_boolean_and_shared_unsafe_domains_atomically() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = native_fixture(NATIVE_MODE_A, NATIVE_MODE_A);
    let mut invalid = ["", "1", " true", "false ", "junk"]
        .map(|value| {
            source.replace(
                "V=\"5\" Marker=\"selected\"",
                &format!("V=\"5\" NeedActivate=\"{value}\" Marker=\"selected\""),
            )
        })
        .to_vec();
    invalid.extend([
        source.replace("N=\"3\"", "N=\"2\""),
        source.replace("N=\"3\"", "M=\"4\" N=\"3\""),
        source.replace("Analog=\"false\"", "Analog=\"true\""),
        source.replace("From=\"120\" To=\"180\"", "From=\"660\" To=\"720\""),
        source.replace(
            "V=\"5\" Marker=\"selected\"",
            "Ix=\"2\" V=\"5\" Marker=\"selected\"",
        ),
        source.replace(
            &format!("Modes=\"{NATIVE_MODE_A},{NATIVE_MODE_B}\""),
            &format!("Modes=\"{NATIVE_MODE_A},{RFC_MODE_A}\""),
        ),
        source.replace(
            &format!("<C Type=\"Mode\" U=\"{NATIVE_MODE_A}\" Title=\"Mode A\"/>"),
            &format!(
                "<C Type=\"Mode\" U=\"{NATIVE_MODE_A}\"/><C Type=\"Mode\" U=\"{RFC_MODE_A}\"/>"
            ),
        ),
        source.replace(
            &format!("Type=\"Mode\" U=\"{NATIVE_MODE_B}\""),
            &format!("Type=\"And\" U=\"{NATIVE_MODE_B}\""),
        ),
        source.replace(
            "From=\"120\" To=\"180\" V=\"5\"",
            "From=\"121\" To=\"180\" V=\"5\"",
        ),
    ]);
    for source in invalid {
        fs::write(&input, &source).unwrap();
        fs::write(&output, b"existing destination").unwrap();
        activation_command(&input, RFC_MODE_A, "02:00")
            .arg("--required")
            .arg("--save-as")
            .arg(&output)
            .assert()
            .failure();
        assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
        assert_eq!(fs::read(&output).unwrap(), b"existing destination");
    }
}

#[test]
fn daytimer_activation_requires_one_flag_and_dry_run_rejects_before_io() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let missing = dir.path().join("missing.Loxone");
    activation_command(&missing, MODE_A, "02:00")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot read").not());
    activation_command(&missing, MODE_A, "02:00")
        .args(["--required", "--no-required"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot read").not());
    activation_command(&missing, MODE_A, "02:00")
        .args(["--required", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("dry-run not supported"))
        .stderr(predicate::str::contains("Cannot read").not());
}
