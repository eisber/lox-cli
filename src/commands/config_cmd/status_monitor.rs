use anyhow::{Result, bail};
use clap::{Args, Subcommand, ValueEnum};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::status_monitor::StatusPriorityDirection;

#[derive(Args)]
pub(crate) struct StatusMonitorTarget {
    /// Local .Loxone file
    file: String,
    /// StatusMonitor title, uuid:... or room-qualified selector
    block: String,
    #[arg(short, long)]
    room: Option<String>,
}

impl StatusMonitorTarget {
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
pub(crate) enum StatusMonitorCmd {
    /// Show eleven existing State records in native priority order (standalone files only)
    Inspect {
        #[command(flatten)]
        target: StatusMonitorTarget,
    },
    /// Move one configured state one priority up/down; preserve Id, OIdx and wires
    Move {
        #[command(flatten)]
        target: StatusMonitorTarget,
        /// Persistent State Id from inspect, NOT OIdx, name or priority
        id: String,
        #[arg(value_enum)]
        direction: DirectionArg,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
}

pub(crate) fn run(ctx: &RunContext, action: StatusMonitorCmd) -> Result<()> {
    let (target, edit, save_as) = match action {
        StatusMonitorCmd::Inspect { target } => (target, None, None),
        StatusMonitorCmd::Move {
            target,
            id,
            direction,
            save_as,
        } => (target, Some((id, direction)), save_as),
    };
    let selector = target.selector()?;
    let original = std::fs::read(&target.file)?;
    let mut editor = ConfigEditor::load(&original)?;
    let movement = edit
        .map(|(id, direction)| {
            editor.move_status_monitor_state(
                &selector,
                &id,
                match direction {
                    DirectionArg::Up => StatusPriorityDirection::Up,
                    DirectionArg::Down => StatusPriorityDirection::Down,
                },
            )
        })
        .transpose()?;
    let inspection = editor.inspect_status_monitor(&selector)?;
    let quiet = ctx.quiet || ctx.json || ctx.csv;
    let saved_to = if let Some(movement) = &movement {
        if movement.changed {
            super::save_edited_with_quiet(&mut editor, &target.file, save_as.as_deref(), quiet)?;
            Some(save_as.as_deref().unwrap_or(&target.file))
        } else if let Some(output) = save_as.as_deref() {
            super::write_edited_bytes(&original, &target.file, Some(output), quiet)?;
            Some(output)
        } else {
            None
        }
    } else {
        None
    };
    if ctx.json {
        let mut result = serde_json::json!({
            "ok": true,
            "changed": movement.as_ref().is_some_and(|movement| movement.changed),
            "saved_to": saved_to,
            "movement": movement,
            "status_monitor": inspection,
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
                    "PRIORITY,ID,OUTPUT_INDEX,NAME"
                } else {
                    "PRIORITY  ID                                   OUTPUT_INDEX  NAME"
                }
            );
        }
        for state in &inspection.states {
            let name = state.attributes.get("Na").map_or("", String::as_str);
            if ctx.csv {
                println!(
                    "{},\"{}\",{},\"{}\"",
                    state.priority,
                    state.id,
                    state.output_index,
                    name.replace('"', "\"\"")
                );
            } else {
                println!(
                    "{:<10}{:<37}{:<14}{}",
                    state.priority, state.id, state.output_index, name
                );
            }
        }
        if !ctx.csv && !ctx.quiet {
            println!("\n{} states", inspection.states.len());
        }
    }
    Ok(())
}
