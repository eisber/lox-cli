use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::energy::{
    EnergyDeadline, EnergyIconChange, EnergyInspection, EnergyLoadMode, EnergyLoadPatch,
    EnergyParameter,
};
use crate::config_edit::icons::BuiltinIconCatalog;
use crate::config_edit::schema::Diagnostic;

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
        #[arg(long)]
        name: Option<String>,
        /// Mark the mode as customized, including an unchanged value
        #[arg(long, value_enum)]
        mode: Option<ModeArg>,
        /// Preparation seconds (30..86400), not minimum runtime
        #[arg(long, allow_hyphen_values = true)]
        preparation_seconds: Option<i64>,
        /// Integer minutes converted to signed 32-bit seconds; negatives allowed
        #[arg(long, allow_hyphen_values = true)]
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
            &editor,
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
