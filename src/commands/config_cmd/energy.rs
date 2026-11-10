use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::energy::{
    EnergyDeadline, EnergyIconChange, EnergyInspection, EnergyLoadMode, EnergyLoadPatch,
    EnergyParameter,
};
use crate::config_edit::icons::BuiltinIconCatalog;
use crate::config_edit::schema::{Diagnostic, NumericBinding, minutes_to_seconds};

#[derive(Args)]
pub(crate) struct EnergyTarget {
    /// Local .Loxone file (no server connection)
    file: String,
    /// EnergyManager2 title, uuid:... or room-qualified selector
    block: String,
    /// Room disambiguation
    #[arg(short, long)]
    room: Option<String>,
}

impl EnergyTarget {
    fn selector(&self) -> Result<String> {
        match &self.room {
            Some(room) if self.block.ends_with(']') && self.block.contains('[') => {
                Err(Diagnostic::new(
                    "conflicting_room_selector",
                    &self.block,
                    format!("Use either the bracket room qualifier or --room {room}, not both."),
                )
                .into())
            }
            Some(room) => Ok(format!("{} [{room}]", self.block)),
            None => Ok(self.block.clone()),
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum EnergyCmd {
    /// List exact loaded built-in icon filenames/IDs and archive provenance (no file required)
    Icons,
    /// Inspect exact storage, effective defaults and limitations (read-only)
    Inspect {
        #[command(flatten)]
        target: EnergyTarget,
    },
    /// Set a validated scalar: O (signed kW), MinSoc (0..100%), MaxSpwr (kW>=0)
    Set {
        #[command(flatten)]
        target: EnergyTarget,
        #[arg(value_parser = ["O", "MinSoc", "MaxSpwr"])]
        parameter: String,
        /// Invariant finite decimal; scientific notation is unsupported
        #[arg(allow_hyphen_values = true)]
        value: String,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Configure/reorder existing twelve slots; no add/delete or runtime parity
    Load {
        #[command(subcommand)]
        action: EnergyLoadCmd,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ModeArg {
    Digital,
    Analog,
}

#[derive(Subcommand)]
pub(crate) enum EnergyLoadCmd {
    /// Patch an existing persistent Load Id; untouched storage is preserved
    Set {
        #[command(flatten)]
        target: EnergyTarget,
        /// Persistent UUID from inspect, NOT U, name, priority or OIdx
        id: String,
        /// Mark the name as customized, including an unchanged value
        #[arg(long, conflicts_with = "no_name")]
        name: Option<String>,
        /// Store an empty customized name (Ncu=true), not inherited naming
        #[arg(long)]
        no_name: bool,
        /// Mark the mode as customized, including an unchanged value
        #[arg(long, value_enum)]
        mode: Option<ModeArg>,
        /// Preparation seconds (30..86400), not minimum runtime
        #[arg(long, allow_hyphen_values = true)]
        preparation_seconds: Option<i64>,
        /// Minutes rounded half away from zero, then checked signed 32-bit seconds; negatives allowed
        #[arg(long, allow_hyphen_values = true, value_parser = parse_minimum_runtime_minutes)]
        min_runtime_minutes: Option<i64>,
        /// Integer minutes/day (0..1440), stored in seconds
        #[arg(long, allow_hyphen_values = true)]
        daily_minutes: Option<i64>,
        /// sunset, dusk, sunrise, dawn, or an hourly time 01:00..24:00
        #[arg(long, value_name = "CHOICE")]
        deadline: Option<EnergyDeadline>,
        /// Rated kW >=0; zero does not disconnect U
        #[arg(long, allow_hyphen_values = true)]
        rated_power: Option<String>,
        /// kW >=0; zero substitutes rated power; greater than rated is allowed
        #[arg(long, allow_hyphen_values = true)]
        switch_on_power: Option<String>,
        /// Exact built-in canonical filename or native/RFC ID; see `config energy icons`
        #[arg(long, value_name = "BUILTIN", conflicts_with = "no_icon")]
        icon: Option<String>,
        /// Remove only this Load's icon reference; retain all definitions and other references
        #[arg(long)]
        no_icon: bool,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Move a complete existing record to priority 1..12; retain Id/OIdx/wires
    Move {
        #[command(flatten)]
        target: EnergyTarget,
        id: String,
        priority: usize,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
}

fn parse_minimum_runtime_minutes(raw: &str) -> Result<i64> {
    if let Ok(minutes) = raw.parse::<i64>() {
        return Ok(minutes);
    }
    let value = NumericBinding {
        key: "MinRt",
        unit: "min",
        default: 0.0,
        min: None,
        max: None,
    }
    .parse(raw, "Load/@MinRt")?;
    // Qt 6.10.2's Double-to-Int helper uses std::round, not qRound.
    let minutes = value.round() as i64;
    // Checked seconds also reject saturation of an out-of-range float-to-integer cast.
    minutes_to_seconds(minutes, "Load/@MinRt")?;
    Ok(minutes)
}

pub(crate) fn run(ctx: &RunContext, action: EnergyCmd) -> Result<()> {
    let (target, save_as, edit) = match action {
        EnergyCmd::Icons => return print_icons(ctx),
        EnergyCmd::Inspect { target } => (target, None, Edit::Inspect),
        EnergyCmd::Set {
            target,
            parameter,
            value,
            save_as,
        } => (
            target,
            save_as,
            Edit::Scalar(EnergyParameter::try_from(parameter.as_str())?, value),
        ),
        EnergyCmd::Load { action } => match action {
            EnergyLoadCmd::Set {
                target,
                id,
                name,
                no_name,
                mode,
                preparation_seconds,
                min_runtime_minutes,
                daily_minutes,
                deadline,
                rated_power,
                switch_on_power,
                icon,
                no_icon,
                save_as,
            } => (
                target,
                save_as,
                Edit::Load(
                    id,
                    EnergyLoadPatch {
                        name,
                        clear_name: no_name,
                        mode: mode.map(|mode| match mode {
                            ModeArg::Digital => EnergyLoadMode::Digital,
                            ModeArg::Analog => EnergyLoadMode::Analog,
                        }),
                        preparation_seconds,
                        min_runtime_minutes,
                        daily_minutes,
                        deadline,
                        rated_power,
                        switch_on_power,
                        icon: match icon {
                            Some(selection) => EnergyIconChange::SetBuiltin(selection),
                            None if no_icon => EnergyIconChange::Clear,
                            None => EnergyIconChange::Preserve,
                        },
                    },
                ),
            ),
            EnergyLoadCmd::Move {
                target,
                id,
                priority,
                save_as,
            } => (target, save_as, Edit::Move(id, priority)),
        },
    };
    let selector = target.selector()?;
    let bytes = std::fs::read(&target.file)
        .map_err(|error| anyhow::anyhow!("Cannot read {}: {error}", target.file))?;
    let mut editor = ConfigEditor::load(&bytes)?;
    let changed = !matches!(edit, Edit::Inspect);
    match edit {
        Edit::Inspect => {}
        Edit::Scalar(parameter, value) => {
            editor.set_energy_parameter(&selector, parameter, &value)?
        }
        Edit::Load(id, patch) => editor.configure_energy_load(&selector, &id, &patch)?,
        Edit::Move(id, priority) => editor.move_energy_load(&selector, &id, priority)?,
    }
    let inspection = editor.inspect_energy(&selector)?;
    let saved_to = if changed && !ctx.dry_run {
        super::save_edited_with_quiet(
            &mut editor,
            &target.file,
            save_as.as_deref(),
            ctx.quiet || ctx.json || ctx.csv,
        )?;
        Some(save_as.as_deref().unwrap_or(&target.file))
    } else {
        None
    };
    if ctx.json {
        let mut result = serde_json::json!({
            "ok": true,
            "dry_run": ctx.dry_run,
            "saved_to": saved_to,
            "energy": inspection,
        });
        if let Some(trace_id) = &ctx.trace_id {
            result["trace_id"] = serde_json::json!(trace_id);
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print_inspection(ctx, &inspection);
        if changed && ctx.dry_run && !ctx.quiet {
            eprintln!("✓ Validated EnergyManager2 edit; dry-run wrote no files.");
        }
    }
    Ok(())
}

enum Edit {
    Inspect,
    Scalar(EnergyParameter, String),
    Load(String, EnergyLoadPatch),
    Move(String, usize),
}

fn print_icons(ctx: &RunContext) -> Result<()> {
    let catalog = BuiltinIconCatalog::embedded()?;
    if ctx.json {
        let mut result = serde_json::json!({
            "ok": true,
            "source": "builtin",
            "catalog": catalog.catalog,
            "count": catalog.icons.len(),
            "icons": catalog.icons,
            "limitations": [
                "Exact filled loaded-catalog membership only; not a Qt SVG rendering or full GUI parity oracle.",
                "Archive version/hash is independent of executable version; a newer user archive may override an installed archive.",
                "No custom/project icon selection, import, replacement, deletion or decoding. Explicit GUI clear/save/reopen proof is still missing."
            ],
        });
        if let Some(trace_id) = &ctx.trace_id {
            result["trace_id"] = serde_json::json!(trace_id);
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if ctx.csv {
        if !ctx.no_header {
            println!("SOURCE,NAME,NATIVE_ID,CATALOG_VERSION,ARCHIVE_SHA256");
        }
        for icon in &catalog.icons {
            csv_row(&[
                "builtin".into(),
                icon.filename.clone(),
                icon.native_id.clone(),
                catalog.catalog.version.clone(),
                catalog.catalog.archive_sha256.clone(),
            ]);
        }
    } else {
        if !ctx.no_header {
            println!("NAME                                     NATIVE_ID");
            println!(
                "─────────────────────────────────────────────────────────────────────────────"
            );
        }
        for icon in &catalog.icons {
            println!("{:<40} {}", icon.filename, icon.native_id);
        }
        println!(
            "\n{} built-in icons\nCatalog {}; archive SHA256 {}",
            catalog.icons.len(),
            catalog.catalog.version,
            catalog.catalog.archive_sha256
        );
    }
    Ok(())
}

fn print_inspection(ctx: &RunContext, inspection: &EnergyInspection) {
    if ctx.csv {
        if !ctx.no_header {
            println!("KIND,PRIORITY,ID,FIELD,STORED,EFFECTIVE,UNIT");
        }
        for parameter in &inspection.scalar_parameters {
            csv_row(&[
                "parameter".into(),
                String::new(),
                inspection.uuid.clone(),
                parameter.key.clone(),
                parameter.value.stored.clone().unwrap_or_default(),
                parameter
                    .value
                    .effective
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                parameter.unit.into(),
            ]);
        }
        for load in &inspection.loads {
            for (key, raw) in &load.attributes {
                csv_row(&[
                    "load".into(),
                    load.priority.to_string(),
                    load.id.clone().unwrap_or_default(),
                    key.clone(),
                    raw.clone(),
                    String::new(),
                    String::new(),
                ]);
            }
        }
    } else {
        if !ctx.no_header {
            println!("PARAMETER  STORED       EFFECTIVE  UNIT");
            println!("──────────────────────────────────────");
        }
        for parameter in &inspection.scalar_parameters {
            println!(
                "{:<10} {:<12} {:<10} {}",
                parameter.key,
                parameter.value.stored.as_deref().unwrap_or("(omitted)"),
                parameter
                    .value
                    .effective
                    .map_or("?".into(), |value| value.to_string()),
                parameter.unit
            );
        }
        if !ctx.no_header {
            println!(
                "\nPRIORITY OUTPUT USED  NAME                 ID                                   ICON"
            );
            println!("────────────────────────────────────────────────────────────");
        }
        for load in &inspection.loads {
            println!(
                "{:<8} {:<6} {:<5} {:<20} {} {}",
                load.priority,
                load.output.as_deref().unwrap_or("?"),
                load.used
                    .effective
                    .map_or("?".into(), |used| used.to_string()),
                load.name.effective.as_deref().unwrap_or(""),
                load.id.as_deref().unwrap_or("?"),
                load.icon_resolution
                    .filename
                    .as_deref()
                    .or(load.icon.as_deref())
                    .unwrap_or("(none)")
            );
        }
        println!(
            "\n{} parameters, {} loads",
            inspection.scalar_parameters.len(),
            inspection.loads.len()
        );
    }
    if !ctx.quiet {
        eprintln!(
            "Verified storage subset only; no complete GUI/runtime parity. See --output json for storage defaults and limitations."
        );
        eprintln!(
            "Built-in icon catalog {}; archive SHA256 {} (independent of Config executable version).",
            inspection.builtin_icon_catalog.version, inspection.builtin_icon_catalog.archive_sha256
        );
    }
    for diagnostic in &inspection.diagnostics {
        eprintln!("{diagnostic}");
    }
}

fn csv_row(values: &[String]) {
    println!(
        "{}",
        values
            .iter()
            .map(|value| format!("\"{}\"", value.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(",")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct EnergyCli {
        #[command(subcommand)]
        action: EnergyCmd,
    }

    #[test]
    fn energy_explicit_name_clear_needs_no_empty_shell_argument() {
        let args = [
            "energy",
            "load",
            "set",
            "synthetic.Loxone",
            "Synthetic Energy",
            "30000000-0000-0000-0000000000000002",
            "--no-name",
        ];
        let parsed = EnergyCli::try_parse_from(args).unwrap();
        let EnergyCmd::Load {
            action: EnergyLoadCmd::Set { name, no_name, .. },
        } = parsed.action
        else {
            panic!("expected load set");
        };
        assert!(no_name);
        assert!(name.is_none());
        let error = EnergyCli::try_parse_from(args.into_iter().chain(["--name", "conflicting"]))
            .err()
            .unwrap();
        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn energy_fractional_minimum_runtime_uses_native_minute_conversion() {
        let source = include_bytes!("../../../tests/ux/energy-manager2/synthetic.Loxone");
        let id = "30000000-0000-0000-0000000000000002";
        for (raw, expected) in [
            ("1.5", 2),
            ("-1.5", -2),
            ("-0.5", -1),
            ("0.5", 1),
            ("2", 2),
            ("-1", -1),
            ("0", 0),
        ] {
            let parsed = EnergyCli::try_parse_from([
                "energy",
                "load",
                "set",
                "synthetic.Loxone",
                "Synthetic Energy",
                id,
                "--min-runtime-minutes",
                raw,
            ])
            .unwrap_or_else(|error| panic!("{raw}: {error}"));
            let EnergyCmd::Load {
                action:
                    EnergyLoadCmd::Set {
                        min_runtime_minutes,
                        ..
                    },
            } = parsed.action
            else {
                panic!("expected existing load set command");
            };
            assert_eq!(min_runtime_minutes, Some(expected));
            let mut editor = ConfigEditor::load(source).unwrap();
            editor
                .configure_energy_load(
                    "Synthetic Energy",
                    id,
                    &EnergyLoadPatch {
                        min_runtime_minutes,
                        ..Default::default()
                    },
                )
                .unwrap();
            let replacement = if expected == 0 {
                String::new()
            } else {
                format!(" MinRt=\"{}\"", expected * 60)
            };
            let expected_xml = std::str::from_utf8(source)
                .unwrap()
                .replace(" MinRt=\"600\"", &replacement);
            let expected_bytes = ConfigEditor::load(expected_xml.as_bytes())
                .unwrap()
                .to_bytes()
                .unwrap();
            let saved = editor.to_bytes().unwrap();
            assert_eq!(saved, expected_bytes, "{raw}: preserve all other storage");
            let reloaded = ConfigEditor::load(&saved).unwrap();
            assert_eq!(
                reloaded.inspect_energy("Synthetic Energy").unwrap().loads[1]
                    .min_runtime_seconds
                    .effective,
                Some(expected as i32 * 60)
            );
        }
    }

    #[test]
    fn energy_fractional_minimum_runtime_retains_checked_cli_policy() {
        for raw in [
            "1,5",
            "1e1",
            "NaN",
            "inf",
            "35791394.5",
            "-35791394.5",
            "35791394.50000001",
            "-35791394.50000001",
        ] {
            assert!(
                EnergyCli::try_parse_from([
                    "energy",
                    "load",
                    "set",
                    "synthetic.Loxone",
                    "Synthetic Energy",
                    "30000000-0000-0000-0000000000000001",
                    "--min-runtime-minutes",
                    raw,
                ])
                .is_err(),
                "{raw}"
            );
        }
        for (raw, expected) in [
            ("35791394.49999999", 35_791_394),
            ("-35791394.49999999", -35_791_394),
        ] {
            let minutes = parse_minimum_runtime_minutes(raw).unwrap();
            assert_eq!(minutes, expected);
            assert!(minutes_to_seconds(minutes, "Load/@MinRt").is_ok());
        }
        for minutes in [i64::MIN, i64::MAX] {
            assert_eq!(
                parse_minimum_runtime_minutes(&minutes.to_string()).unwrap(),
                minutes
            );
            assert!(minutes_to_seconds(minutes, "Load/@MinRt").is_err());
        }
    }
}
