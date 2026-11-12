use std::fmt;

use anyhow::Result;

use crate::commands::color_cmd::ColorCmd;
use crate::commands::config_cmd::energy::{EnergyCmd, EnergyLoadCmd};
use crate::commands::config_cmd::lighting::LightingCmd;
use crate::commands::live_cmd::LiveCmd;
use crate::commands::sim_cmd::SimCmd;
use crate::commands::stats_cmd::StatsCmd;
use crate::{
    BlocksCmd, CacheCmd, Cli, Cmd, ConfigCmd, ControlCmd, CtxCmd, MqttConfigCmd, RoomCmd, SetupCmd,
    TelemetryCmd, TokenCmd,
};

#[derive(Debug)]
pub(crate) struct Unsupported;

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "dry-run not supported for this command: no safe non-mutating plan is available. \
             Nothing was executed and inputs were not fully validated. \
             Remove --dry-run only when you intend to execute the command.",
        )
    }
}

impl std::error::Error for Unsupported {}

/// Keep unsupported operations outside dispatch, including startup side effects.
pub(crate) fn dispatch(cli: Cli, execute: impl FnOnce(Cli) -> Result<()>) -> Result<()> {
    if cli.dry_run && !has_non_mutating_path(&cli.cmd) {
        return Err(Unsupported.into());
    }
    execute(cli)
}

fn has_non_mutating_path(cmd: &Cmd) -> bool {
    // New commands fail closed until their read-only path or planner is audited.
    match cmd {
        Cmd::Config { action } => matches!(
            action.as_ref(),
            ConfigCmd::Energy {
                action: EnergyCmd::Icons | EnergyCmd::Inspect { .. } | EnergyCmd::Set { .. }
            } | ConfigCmd::Energy {
                action: EnergyCmd::Load {
                    action: EnergyLoadCmd::Set { .. } | EnergyLoadCmd::Move { .. }
                }
            } | ConfigCmd::Lighting {
                action: LightingCmd::Inspect { .. }
                    | LightingCmd::RecountMoods { .. }
                    | LightingCmd::SetMoodCid { .. }
                    | LightingCmd::SetMoodSid { .. }
                    | LightingCmd::SetMoodMix { .. }
                    | LightingCmd::SetMoodSkip { .. }
                    | LightingCmd::RenameMood { .. }
                    | LightingCmd::SetMoodDisplayName { .. }
                    | LightingCmd::SetMoodLevel { .. }
            } | ConfigCmd::Ls
                | ConfigCmd::Users { .. }
                | ConfigCmd::Devices { .. }
                | ConfigCmd::Diff { .. }
                | ConfigCmd::Log { .. }
                | ConfigCmd::Rooms { .. }
                | ConfigCmd::Controls { .. }
                | ConfigCmd::Validate { .. }
                | ConfigCmd::Check { .. }
                | ConfigCmd::Scan { .. }
                | ConfigCmd::UserPerms { .. }
                | ConfigCmd::AutopilotList { .. }
                | ConfigCmd::ModeList { .. }
                | ConfigCmd::Moods { .. }
                | ConfigCmd::SetMoodColor { .. }
                | ConfigCmd::AddMood { .. }
                | ConfigCmd::SpliceActor { .. }
                | ConfigCmd::Room(RoomCmd::Rename { .. })
                | ConfigCmd::GetParams { .. }
                | ConfigCmd::Connectors { .. }
                | ConfigCmd::GetProgram { .. }
                | ConfigCmd::Describe { .. }
                | ConfigCmd::Surfaces { .. }
                | ConfigCmd::Wires { .. }
                | ConfigCmd::Stats { .. }
                | ConfigCmd::Report {
                    open_issue: false,
                    ..
                }
                | ConfigCmd::Control(ControlCmd::Describe { .. } | ControlCmd::Wires { .. })
                | ConfigCmd::Mqtt(MqttConfigCmd::List { .. } | MqttConfigCmd::Topics { .. })
        ),
        _ => matches!(
            cmd,
            Cmd::Setup {
                action: SetupCmd::Show
            } | Cmd::Cache {
                action: CacheCmd::Info | CacheCmd::Check
            } | Cmd::Token {
                action: TokenCmd::Info | TokenCmd::Check
            } | Cmd::Telemetry {
                action: TelemetryCmd::Status
            } | Cmd::Ctx {
                action: CtxCmd::List | CtxCmd::Current
            } | Cmd::Completions { install: false, .. }
                | Cmd::Report(_)
                | Cmd::Feedback {
                    action: crate::commands::report_cmd::LegacyFeedbackCmd::Report(_)
                }
                | Cmd::Schema { .. }
                | Cmd::Sim {
                    action: SimCmd::Check { .. }
                        | SimCmd::Run { .. }
                        | SimCmd::Step { .. }
                        | SimCmd::Dump { .. }
                }
                | Cmd::Blocks {
                    action: BlocksCmd::Search { .. }
                        | BlocksCmd::Info { .. }
                        | BlocksCmd::List { .. }
                }
                | Cmd::Docs { .. }
                | Cmd::Color {
                    action: ColorCmd::Encode { .. } | ColorCmd::Decode { .. }
                }
                | Cmd::Live {
                    action: LiveCmd::Time
                }
                | Cmd::Stats {
                    action: StatsCmd::List { .. }
                        | StatsCmd::Get { .. }
                        | StatsCmd::Download { .. }
                }
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use clap::Parser;

    use super::*;

    fn parse_cli(args: impl IntoIterator<Item = impl AsRef<str>>) -> Cli {
        let args: Vec<String> = args
            .into_iter()
            .map(|arg| arg.as_ref().to_owned())
            .collect();
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || Cli::try_parse_from(args).unwrap())
            .unwrap()
            .join()
            .unwrap()
    }

    #[test]
    fn dry_run_unplanned_mutations_never_reach_dispatch() {
        let cases: &[&[&str]] = &[
            &["config", "push", "input.Loxone", "--force", "--reboot"],
            &["config", "upload", "input.zip", "--force"],
            &[
                "config",
                "patch",
                "--replace",
                "OLD",
                "NEW",
                "--force",
                "--reboot",
            ],
            &["config", "push-http", "input.zip", "--force"],
            &["config", "restore", "HEAD", "--force"],
            &["live", "set", "target", "1", "--write"],
            &["setup", "set", "--host", "example.invalid"],
            &["cache", "clear"],
            &["cache", "refresh"],
            &["token", "fetch"],
            &["token", "clear"],
            &["token", "refresh"],
            &["token", "revoke"],
            &["telemetry", "enable"],
            &["telemetry", "disable"],
            &[
                "ctx",
                "add",
                "new",
                "--host",
                "example.invalid",
                "--user",
                "test",
                "--pass",
                "test",
            ],
            &["ctx", "use", "new"],
            &["ctx", "remove", "old"],
            &["ctx", "rename", "old", "new"],
            &["ctx", "init"],
            &["ctx", "migrate"],
            &["completions", "bash", "--install"],
            &["config", "download", "--extract"],
            &[
                "config",
                "extract",
                "input.zip",
                "--save-as",
                "output.Loxone",
            ],
            &["config", "compress", "input.Loxone"],
            &["config", "init", "repo"],
            &["config", "pull"],
            &["config", "set-param", "input.Loxone", "block", "I1", "1"],
            &["config", "wire", "input.Loxone", "Lamp.AI", "uuid:source"],
            &[
                "config",
                "wire-connector",
                "input.Loxone",
                "Lamp.AI",
                "uuid:source",
            ],
            &["config", "room", "add", "input.Loxone", "new"],
            &[
                "config",
                "control",
                "move",
                "input.Loxone",
                "--to-room",
                "new",
                "--title",
                "Lamp",
            ],
            &["config", "user-remove", "input.Loxone", "user"],
            &[
                "config",
                "xml",
                "set-attr",
                "input.Loxone",
                "block",
                "Title",
                "new",
            ],
            &["config", "snapshot", "input.Loxone", "--before"],
            &["config", "report", "input.Loxone", "--open-issue"],
            // Even a read can populate the structure cache while resolving a name.
            &["live", "get", "target"],
        ];
        for args in cases {
            let cli = parse_cli(["lox", "--dry-run"].into_iter().chain(args.iter().copied()));
            let calls = Cell::new(0);
            let result = dispatch(cli, |_| {
                calls.set(calls.get() + 1);
                Ok(())
            });
            assert!(result.unwrap_err().is::<Unsupported>(), "{args:?}");
            assert_eq!(calls.get(), 0, "{args:?} reached a mutation backend");
        }
    }

    #[test]
    fn dry_run_preserves_existing_planners_and_read_only_dispatch() {
        for args in [
            vec!["config", "room", "rename", "input.Loxone", "old", "new"],
            vec![
                "config",
                "splice-actor",
                "input.Loxone",
                "Lamp",
                "--source",
                "uuid:source",
            ],
            vec![
                "config",
                "energy",
                "set",
                "input.Loxone",
                "Energy",
                "MinSoc",
                "50",
            ],
            vec!["stats", "download", "--dir", "stats", "--force"],
            vec!["config", "validate", "input.Loxone"],
            vec![
                "config",
                "add-mood",
                "input.Loxone",
                "Lamp",
                "--name",
                "A",
                "--color",
                "hsv(0,100,100)",
            ],
            vec![
                "config",
                "set-mood-color",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--color",
                "hsv(120,100,10)",
            ],
            vec![
                "config",
                "lighting",
                "set-mood-level",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--level",
                "37",
            ],
            vec![
                "config",
                "lighting",
                "rename-mood",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--name",
                "New",
            ],
            vec![
                "config",
                "lighting",
                "set-mood-display-name",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--name",
                "New",
                "--normalize-ascii",
            ],
            vec![
                "config",
                "lighting",
                "set-mood-display-name",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--clear",
            ],
            vec![
                "config",
                "lighting",
                "set-mood-cid",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--cid",
                "22",
            ],
            vec![
                "config",
                "lighting",
                "set-mood-sid",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--sid",
                "32",
                "--allow-cid-clear",
            ],
            vec![
                "config",
                "lighting",
                "recount-moods",
                "input.Loxone",
                "Lamp",
            ],
            vec!["config", "report", "input.Loxone"],
            vec![
                "config",
                "lighting",
                "set-mood-skip",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--skip",
            ],
            vec![
                "config",
                "lighting",
                "set-mood-mix",
                "input.Loxone",
                "Lamp",
                "--mood",
                "A",
                "--mix",
            ],
            vec!["color", "encode", "--rgb", "1,2,3"],
            vec!["completions", "bash"],
        ] {
            let cli = parse_cli(["lox", "--dry-run"].into_iter().chain(args));
            let calls = Cell::new(0);
            dispatch(cli, |cli| {
                assert!(cli.dry_run);
                calls.set(calls.get() + 1);
                Ok(())
            })
            .unwrap();
            assert_eq!(calls.get(), 1);
        }
    }

    #[test]
    fn normal_mutations_still_dispatch() {
        let cli = parse_cli(["lox", "config", "push", "input.Loxone", "--force"]);
        let calls = Cell::new(0);
        dispatch(cli, |_| {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .unwrap();
        assert_eq!(calls.get(), 1);
    }
}
