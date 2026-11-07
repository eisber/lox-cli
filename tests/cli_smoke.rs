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
subcmd_help!(help_feedback, "feedback");
subcmd_help!(help_feedback_report, "feedback", "report");
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

mod feedback {
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
            .args(["schema", "feedback"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let schema: Value = serde_json::from_slice(&output).unwrap();
        let args = schema["subcommands"][0]["args"].as_array().unwrap();
        let body_file = args.iter().find(|arg| arg["name"] == "body_file").unwrap();
        assert_eq!(body_file["long"], "--body-file");
    }

    #[test]
    fn dry_run_previews_inline_body_without_github_or_telemetry() {
        let dir = tempfile::tempdir().unwrap();
        let output = command()
            .env("PATH", dir.path())
            .args([
                "feedback",
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
                "feedback",
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
                "feedback",
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
                "feedback",
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
                "feedback",
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
                .args([
                    "feedback",
                    "report",
                    "--title",
                    "Synthetic issue",
                    "--dry-run",
                ])
                .args(args)
                .assert()
                .failure();
        }
        command()
            .args([
                "feedback",
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
