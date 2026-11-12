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
}

pub(crate) fn run(ctx: &RunContext, action: LoadManagerCmd) -> Result<()> {
    let (target, edit, save_as) = match action {
        LoadManagerCmd::Inspect { target } => (target, None, None),
        LoadManagerCmd::Move {
            target,
            io_index,
            direction,
            save_as,
        } => (target, Some((io_index, direction)), save_as),
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
    let inspection = editor.inspect_load_manager(&selector)?;
    let saved_to = if edit.is_some() {
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
