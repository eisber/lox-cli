use assert_cmd::Command;
use predicates::prelude::*;

fn lox() -> Command {
    let mut command = Command::cargo_bin("lox").unwrap();
    command.env("LOX_TELEMETRY", "0");
    command
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

#[test]
fn operating_mode_add_json_quiet_and_unsupported_options() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("modes.Loxone");
    let destination = dir.path().join("destination.Loxone");
    let mut modes: String = (1..=6).map(|id| format!("<C Type=\"Mode\" U=\"00000000-0000-{id:04x}-ffff112233445566\" Title=\"Builtin{id}\" Mode=\"-{id}\"/>")).collect();
    for id in 0..=9 {
        let identity = id + 1;
        modes.push_str(&format!("<C Type=\"Mode\" U=\"00000000-0000-{identity:04x}-1500000000000000\" Title=\"Day{id}\" Mode=\"{id}\"/>"));
    }
    let input = format!(
        "<ControlList><C Type=\"Document\" U=\"11111111-2222-3333-ffff112233445566\" NumO=\"18\"><C Type=\"ModeCaption\" U=\"22222222-2222-3333-ffff112233445566\">{modes}</C></C></ControlList>"
    );
    std::fs::write(&file, input).unwrap();
    let result = lox()
        .args([
            "config",
            "add",
            "--type",
            "operating-mode",
            "--title",
            "Evening",
            "-o",
            "json",
        ])
        .arg(&file)
        .assert()
        .success();
    let json: serde_json::Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(json["added"]["mode"], -7);
    lox()
        .args(["-q", "config", "add", "--type", "Mode", "--title", "Night"])
        .arg(&file)
        .assert()
        .success()
        .stdout("");
    let before = std::fs::read(&file).unwrap();
    std::fs::write(&destination, b"untouched").unwrap();
    lox()
        .args([
            "config",
            "add",
            "--type",
            "Mode",
            "--title",
            "Other",
            "--room",
            "Room",
            "--save-as",
        ])
        .arg(&destination)
        .arg(&file)
        .assert()
        .failure();
    assert_eq!(std::fs::read(&file).unwrap(), before);
    assert_eq!(std::fs::read(&destination).unwrap(), b"untouched");
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
subcmd_help!(help_report, "report");
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

mod reporting {
    use super::*;
    use serde_json::Value;

    fn command() -> Command {
        let mut command = lox();
        command
            .env("LOX_TELEMETRY", "0")
            .args(["--quiet", "--json"]);
        command
    }

    #[test]
    fn agent_schema_advertises_the_actual_body_file_flag() {
        let output = command()
            .args(["schema", "report"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let schema: Value = serde_json::from_slice(&output).unwrap();
        let args = schema["args"].as_array().unwrap();
        let body_file = args.iter().find(|arg| arg["name"] == "body_file").unwrap();
        assert_eq!(body_file["long"], "--body-file");
    }

    #[test]
    fn dry_run_previews_inline_body_without_github_or_telemetry() {
        let dir = tempfile::tempdir().unwrap();
        let output = command()
            .env("PATH", dir.path())
            .args([
                "report",
                "--title",
                "Synthetic issue",
                "--body",
                "Repro steps",
                "--dry-run",
                "--trace-id",
                "local-trace",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["repository"], "eisber/lox-cli");
        assert_eq!(result["title"], "Synthetic issue");
        assert!(result["body"].as_str().unwrap().starts_with("Repro steps"));
        assert_eq!(result["telemetry_fallback_enabled"], false);
        assert_eq!(result["trace_id"], "local-trace");
    }

    #[test]
    fn report_accepts_file_or_stdin_body() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("report.txt");
        let body = "Synthetic reproduction\nExpected 1, got 0";
        std::fs::write(&path, body).unwrap();
        for from_stdin in [false, true] {
            let mut command = command();
            command.args([
                "report",
                "--title",
                "Synthetic issue",
                "--dry-run",
                "--body-file",
            ]);
            if from_stdin {
                command.arg("-").write_stdin(body);
            } else {
                command.arg(&path);
            }
            let output = command.assert().success().get_output().stdout.clone();
            let result: Value = serde_json::from_slice(&output).unwrap();
            assert!(result["body"].as_str().unwrap().starts_with(body));
        }
    }

    #[test]
    fn missing_github_and_opt_out_returns_json_error() {
        let dir = tempfile::tempdir().unwrap();
        let output = command()
            .env("PATH", dir.path())
            .args([
                "report",
                "--title",
                "Synthetic issue",
                "--body",
                "Synthetic details",
            ])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["ok"], false);
        let message = result["message"].as_str().unwrap();
        assert!(message.contains("Cannot run GitHub CLI"));
        assert!(message.contains("Telemetry is disabled"));
        assert!(message.contains("No delivery was confirmed"));
    }

    #[test]
    fn config_opt_out_does_not_require_connection_settings_or_active_context() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.yaml");
        std::fs::write(&config, "telemetry: false\ncontexts: {}\n").unwrap();
        let output = command()
            .env_remove("LOX_TELEMETRY")
            .env_remove("DO_NOT_TRACK")
            .env("LOX_CONFIG", &config)
            .env("PATH", dir.path())
            .args([
                "report",
                "--title",
                "Synthetic issue",
                "--body",
                "Synthetic details",
            ])
            .assert()
            .failure()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert!(
            result["message"]
                .as_str()
                .unwrap()
                .contains("Telemetry is disabled")
        );
        lox()
            .env_remove("LOX_TELEMETRY")
            .env_remove("DO_NOT_TRACK")
            .env("LOX_CONFIG", &config)
            .args(["telemetry", "status", "--quiet"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Effective:        disabled"));
    }

    #[test]
    fn malformed_telemetry_settings_warn_and_disable_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.yaml");
        std::fs::write(&config, "telemetry: [not a boolean]\n").unwrap();
        command()
            .env_remove("LOX_TELEMETRY")
            .env_remove("DO_NOT_TRACK")
            .env("LOX_CONFIG", &config)
            .env("PATH", dir.path())
            .args([
                "report",
                "--title",
                "Synthetic issue",
                "--body",
                "Synthetic details",
            ])
            .assert()
            .failure()
            .stdout(predicate::str::contains("Telemetry is disabled"))
            .stderr(predicate::str::contains(
                "Cannot parse telemetry preference",
            ));
    }

    #[test]
    fn report_rejects_missing_conflicting_empty_or_unreadable_body() {
        let dir = tempfile::tempdir().unwrap();
        for args in [
            vec![],
            vec!["--body", "details", "--body-file", "-"],
            vec!["--body", " \n"],
        ] {
            command()
                .args(["report", "--title", "Synthetic issue", "--dry-run"])
                .args(args)
                .assert()
                .failure();
        }
        command()
            .args([
                "report",
                "--title",
                "Synthetic issue",
                "--dry-run",
                "--body-file",
            ])
            .arg(dir.path().join("missing.txt"))
            .assert()
            .failure()
            .stdout(predicate::str::contains("Cannot read issue report"));
    }

    #[test]
    fn report_is_discoverable_without_advertising_legacy_nesting() {
        lox()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("  report "))
            .stdout(predicate::str::contains("feedback").not());
        lox()
            .args(["report", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("--body-file"))
            .stdout(predicate::str::contains("GitHub issues are public"));
        let output = command()
            .arg("schema")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let schema: Value = serde_json::from_slice(&output).unwrap();
        let names = schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|command| command["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(names.contains(&"report"));
        assert!(!names.contains(&"feedback"));
        assert!(
            schema["report_hint"]
                .as_str()
                .unwrap()
                .contains("lox report --help")
        );
        for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
            lox()
                .env("LOX_TELEMETRY", "0")
                .args(["completions", shell, "--quiet"])
                .assert()
                .success()
                .stdout(predicate::str::contains("report"))
                .stdout(predicate::str::contains("feedback").not());
        }
    }

    #[test]
    fn legacy_feedback_report_still_uses_the_canonical_handler() {
        let output = command()
            .args([
                "feedback",
                "report",
                "--title",
                "Synthetic issue",
                "--body",
                "Synthetic details",
                "--dry-run",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert!(
            result["body"]
                .as_str()
                .unwrap()
                .contains("Reported via `lox report`")
        );
    }

    #[test]
    fn parameter_errors_return_json_and_reporting_guidance_for_each_output_spelling() {
        for flags in [
            vec!["--json"],
            vec!["--output", "json"],
            vec!["--output=json"],
            vec!["-o", "json"],
            vec!["-ojson"],
            vec!["-qojson"],
        ] {
            let output = lox()
                .env("LOX_TELEMETRY", "0")
                .args(["color", "encode", "--brightness", "not-a-number"])
                .args(flags)
                .assert()
                .code(2)
                .stderr("")
                .get_output()
                .stdout
                .clone();
            let result: Value = serde_json::from_slice(&output).unwrap();
            assert_eq!(result["error"], "invalid_arguments");
            assert_eq!(result["ok"], false);
            assert!(result["message"].as_str().unwrap().contains("not-a-number"));
            assert!(
                result["report_hint"]
                    .as_str()
                    .unwrap()
                    .contains("Correct input/setup errors first")
            );
        }
    }

    #[test]
    fn human_argument_errors_preserve_usage_and_hint_even_when_quiet() {
        lox()
            .args(["--quiet", "report", "--unknown-flag"])
            .assert()
            .code(2)
            .stdout("")
            .stderr(predicate::str::contains("unexpected argument"))
            .stderr(predicate::str::contains("Usage:"))
            .stderr(predicate::str::contains("lox report --help"));
    }

    #[test]
    fn runtime_errors_include_guidance_in_json_and_human_output() {
        let output = command()
            .args(["color", "decode", "invalid"])
            .assert()
            .code(1)
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["ok"], false);
        assert!(
            result["report_hint"]
                .as_str()
                .unwrap()
                .contains("lox report --help")
        );
        lox()
            .env("LOX_TELEMETRY", "0")
            .args(["--quiet", "color", "decode", "invalid"])
            .assert()
            .code(1)
            .stdout("")
            .stderr(predicate::str::contains("lox report --help"));
    }

    #[test]
    fn help_and_version_are_not_json_errors() {
        for flag in ["--help", "--version"] {
            command()
                .arg(flag)
                .assert()
                .success()
                .stderr("")
                .stdout(predicate::str::contains("\"ok\":false").not());
        }
    }
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
