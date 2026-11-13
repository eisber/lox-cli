use anyhow::{Result, bail};
use clap::{Args, Subcommand, ValueEnum};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::load_manager::LoadMoveDirection;

#[derive(Args)]
pub(crate) struct LoadManagerTarget {
    /// Local .Loxone file
    file: String,
    /// LoadShed title, uuid:... or room-qualified selector
    block: String,
    #[arg(short, long)]
    room: Option<String>,
}

impl LoadManagerTarget {
    fn selector(&self) -> Result<String> {
        match &self.room {
            Some(_) if self.block.ends_with(']') && self.block.contains('[') => {
                bail!("Use either a bracket room qualifier or --room, not both.")
            }
            Some(room) => Ok(format!("{} [{room}]", self.block)),
            None => Ok(self.block.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum DirectionArg {
    Up,
    Down,
}

#[derive(Subcommand)]
pub(crate) enum LoadManagerCmd {
    /// Show ordered priorities and exact stored settings (complete twelve-load shape only)
    Inspect {
        #[command(flatten)]
        target: LoadManagerTarget,
    },
    /// Move one load one place in Configure loads/XML order; preserve ioIdx/settings
    Move {
        #[command(flatten)]
        target: LoadManagerTarget,
        /// Connector association ioIdx (0..11), NOT the 1-based priority
        #[arg(value_parser = clap::value_parser!(u8).range(0..=11))]
        io_index: u8,
        #[arg(value_enum)]
        direction: DirectionArg,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set one configured consumer's Name cell by stable ioIdx, not priority
    SetName {
        #[command(flatten)]
        target: LoadManagerTarget,
        /// Connector association ioIdx (0..11), NOT the 1-based priority
        #[arg(value_parser = clap::value_parser!(u8).range(0..=11))]
        io_index: u8,
        /// Exact single-line Unicode name; empty clears it, spaces are preserved
        name: String,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set one configured consumer's Power [kW] cell by stable ioIdx, not priority
    SetPower {
        #[command(flatten)]
        target: LoadManagerTarget,
        /// Connector association ioIdx (0..11), NOT the 1-based priority
        #[arg(value_parser = clap::value_parser!(u8).range(0..=11))]
        io_index: u8,
        /// Nonnegative kW decimal, at most 3 fractional digits; never clamped or rounded
        power_kw: String,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set only Maximum off time [s] by stable ioIdx, not priority (positive subset)
    SetMaxOff {
        #[command(flatten)]
        target: LoadManagerTarget,
        /// Connector association ioIdx (0..11), NOT the 1-based priority
        #[arg(value_parser = clap::value_parser!(u8).range(0..=11))]
        io_index: u8,
        /// Whole seconds in 10..2147483647; zero/rounding/clamping are unsupported
        max_off_seconds: String,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
}

pub(crate) fn run(ctx: &RunContext, action: LoadManagerCmd) -> Result<()> {
    let (target, edit, set_name, set_power, set_max_off, save_as) = match action {
        LoadManagerCmd::Inspect { target } => (target, None, None, None, None, None),
        LoadManagerCmd::Move {
            target,
            io_index,
            direction,
            save_as,
        } => (
            target,
            Some((io_index, direction)),
            None,
            None,
            None,
            save_as,
        ),
        LoadManagerCmd::SetName {
            target,
            io_index,
            name,
            save_as,
        } => (target, None, Some((io_index, name)), None, None, save_as),
        LoadManagerCmd::SetPower {
            target,
            io_index,
            power_kw,
            save_as,
        } => (
            target,
            None,
            None,
            Some((io_index, power_kw)),
            None,
            save_as,
        ),
        LoadManagerCmd::SetMaxOff {
            target,
            io_index,
            max_off_seconds,
            save_as,
        } => (
            target,
            None,
            None,
            None,
            Some((io_index, max_off_seconds)),
            save_as,
        ),
    };
    let selector = target.selector()?;
    let mut editor = ConfigEditor::load(&std::fs::read(&target.file)?)?;
    if let Some((io_index, direction)) = edit {
        editor.move_load_manager_load(
            &selector,
            io_index,
            match direction {
                DirectionArg::Up => LoadMoveDirection::Up,
                DirectionArg::Down => LoadMoveDirection::Down,
            },
        )?;
    }
    let name_changed = set_name
        .as_ref()
        .map(|(io_index, name)| editor.set_load_manager_load_name(&selector, *io_index, name))
        .transpose()?;
    let power_changed = set_power
        .as_ref()
        .map(|(io_index, power)| editor.set_load_manager_load_power(&selector, *io_index, power))
        .transpose()?;
    let max_off_changed = set_max_off
        .as_ref()
        .map(|(io_index, seconds)| {
            editor.set_load_manager_load_max_off(&selector, *io_index, seconds)
        })
        .transpose()?;
    let changed = name_changed.or(power_changed).or(max_off_changed);
    let inspection = editor.inspect_load_manager(&selector)?;
    let saved_to =
        if edit.is_some() || changed == Some(true) || (changed.is_some() && save_as.is_some()) {
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
            "saved_to": saved_to,
            "load_manager": inspection,
        });
        if let Some(changed) = changed {
            result["changed"] = serde_json::json!(changed);
        }
        if let Some(trace_id) = &ctx.trace_id {
            result["trace_id"] = serde_json::json!(trace_id);
        }
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if !ctx.no_header {
            println!(
                "{}",
                if ctx.csv {
                    "PRIORITY,IO_INDEX,NAME"
                } else {
                    "PRIORITY  IO_INDEX  NAME"
                }
            );
        }
        for load in &inspection.loads {
            let name = load.attributes.get("nm").map_or("", String::as_str);
            if ctx.csv {
                println!(
                    "{},{},\"{}\"",
                    load.priority,
                    load.io_index,
                    name.replace('"', "\"\"")
                );
            } else {
                println!("{:<10}{:<10}{}", load.priority, load.io_index, name);
            }
        }
        if !ctx.csv && !ctx.quiet {
            println!("\n{} loads", inspection.loads.len());
        }
    }
    Ok(())
}
