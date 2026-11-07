use assert_cmd::Command;
use predicates::prelude::*;

fn lox() -> Command {
    Command::cargo_bin("lox").unwrap()
}

// ── Top-level --help and --version ─────────────────────────────────────────

#[test]
fn help_exits_0() {
    lox().arg("--help").assert().success();
}

#[test]
fn version_exits_0() {
    lox().arg("--version").assert().success();
}

// ── Every subcommand --help exits 0 ────────────────────────────────────────
// This catches clap definition bugs (duplicate short flags, missing values,
// conflicting attributes) at the integration-test level.

macro_rules! subcmd_help {
    ($name:ident, $($arg:expr_2021),+) => {
        #[test]
        fn $name() {
            lox()$(.arg($arg))+ .arg("--help").assert().success();
        }
    };
}

// Configuration commands
subcmd_help!(help_setup, "setup");
subcmd_help!(help_setup_set, "setup", "set");
subcmd_help!(help_setup_show, "setup", "show");
subcmd_help!(help_cache, "cache");
subcmd_help!(help_cache_info, "cache", "info");
subcmd_help!(help_cache_clear, "cache", "clear");
subcmd_help!(help_cache_refresh, "cache", "refresh");
subcmd_help!(help_cache_check, "cache", "check");
subcmd_help!(help_token, "token");
subcmd_help!(help_token_fetch, "token", "fetch");
subcmd_help!(help_token_info, "token", "info");
subcmd_help!(help_token_clear, "token", "clear");
subcmd_help!(help_token_check, "token", "check");
subcmd_help!(help_token_refresh, "token", "refresh");
subcmd_help!(help_token_revoke, "token", "revoke");
subcmd_help!(help_config, "config");
subcmd_help!(help_config_download, "config", "download");
subcmd_help!(help_config_ls, "config", "ls");
subcmd_help!(help_config_extract, "config", "extract");
subcmd_help!(help_config_upload, "config", "upload");
subcmd_help!(help_config_users, "config", "users");
subcmd_help!(help_config_user_grant, "config", "user-grant");
subcmd_help!(help_config_user_revoke, "config", "user-revoke");
subcmd_help!(help_config_user_perms, "config", "user-perms");
subcmd_help!(help_config_devices, "config", "devices");
subcmd_help!(help_config_wires, "config", "wires");
subcmd_help!(help_config_diff, "config", "diff");

#[test]
fn config_add_help_describes_block_type_discovery() {
    lox()
        .args(["config", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Friendly aliases"))
        .stdout(predicate::str::contains("lox blocks list"))
        .stdout(predicate::str::contains("And/Or/Not/Xor"))
        .stdout(predicate::str::contains("lox-cli blocks list").not());
}

subcmd_help!(help_ctx, "ctx");
subcmd_help!(help_ctx_add, "ctx", "add");
subcmd_help!(help_ctx_use, "ctx", "use");
subcmd_help!(help_ctx_list, "ctx", "list");
subcmd_help!(help_ctx_current, "ctx", "current");
subcmd_help!(help_ctx_remove, "ctx", "remove");
subcmd_help!(help_ctx_rename, "ctx", "rename");
subcmd_help!(help_ctx_init, "ctx", "init");
subcmd_help!(help_ctx_migrate, "ctx", "migrate");
subcmd_help!(help_completions, "completions");
subcmd_help!(help_docs, "docs");
subcmd_help!(help_schema, "schema");
subcmd_help!(help_telemetry, "telemetry");
subcmd_help!(help_color, "color");
subcmd_help!(help_color_encode, "color", "encode");
subcmd_help!(help_color_decode, "color", "decode");
subcmd_help!(help_stats, "stats");
subcmd_help!(help_stats_list, "stats", "list");
subcmd_help!(help_stats_ls, "stats", "ls");
subcmd_help!(help_stats_get, "stats", "get");
subcmd_help!(help_stats_download, "stats", "download");

#[test]
fn help_mentions_stats() {
    lox()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Miniserver statistics"));
    lox()
        .args(["stats", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("  ls "));
}

#[test]
fn blocks_info_connector_inputs_from_embedded_schema_forms() {
    for (block_type, expected_inputs, expected_output, expected_parameter) in [
        (
            "LightController2",
            &["I1", "Move", "Presence", "EnMove", "InputDisable", "Sel8"][..],
            "Scene",
            Some("FadingTime"),
        ),
        ("And", &["I1", "I2"][..], "Q", None),
        ("Monoflop", &["InputTrigger"][..], "Q", Some("Time")),
    ] {
        let output = lox()
            .args(["blocks", "info", block_type, "-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let connector_names = |kind: &str| -> Vec<&str> {
            result[kind]
                .as_array()
                .unwrap()
                .iter()
                .map(|connector| connector["name"].as_str().unwrap())
                .collect()
        };
        let inputs = connector_names("inputs");
        for expected in expected_inputs {
            assert!(
                inputs.contains(expected),
                "{block_type} is missing input {expected}: {result}"
            );
        }
        assert!(connector_names("outputs").contains(&expected_output));
        if let Some(expected) = expected_parameter {
            assert!(connector_names("parameters").contains(&expected));
            assert!(!inputs.contains(&expected));
        }
        assert!(!inputs.contains(&expected_output));
    }
}

// ── color encode/decode functional ─────────────────────────────────────────

#[test]
fn color_encode_rgb_composite() {
    // value = R + G*1000 + B*1000000  → 255 for pure red
    lox()
        .args(["color", "encode", "--rgb", "255,0,0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("255"));
}

#[test]
fn color_encode_rgb_base1000_packing() {
    // 100,40,0 → 40100 (regression: NOT 24-bit 0xRRGGBB)
    lox()
        .args(["color", "encode", "--rgb", "100,40,0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("40100"));
}

#[test]
fn color_decode_composite_to_rgb() {
    lox()
        .args(["color", "decode", "40100"])
        .assert()
        .success()
        .stdout(predicate::str::contains("100,40,0"));
}

#[test]
fn color_encode_kelvin_emits_temp_command() {
    lox()
        .args(["color", "encode", "--kelvin", "2700", "--brightness", "15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("temp(15,2700)"));
}

// ── Global flags accepted with subcommands ─────────────────────────────────

#[test]
fn global_json_flag_accepted() {
    lox()
        .args(["--output", "json", "--help"])
        .assert()
        .success();
}

#[test]
fn global_quiet_flag_accepted() {
    lox().args(["-q", "--help"]).assert().success();
}

#[test]
fn global_no_header_flag_accepted() {
    lox().args(["--no-header", "--help"]).assert().success();
}

#[test]
fn help_mentions_loxone() {
    lox()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Loxone"));
}
