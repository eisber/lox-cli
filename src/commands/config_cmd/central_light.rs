use anyhow::{Result, bail};
use clap::{Args, Subcommand, ValueEnum};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::central_light_links::CentralLightLinkDirection;

#[derive(Args)]
pub(crate) struct CentralLightTarget {
    /// Local standalone .Loxone file
    file: String,
    /// CentralLight title, uuid:... or room-qualified selector
    owner: String,
    #[arg(short, long)]
    room: Option<String>,
}

impl CentralLightTarget {
    fn selector(&self) -> Result<String> {
        match &self.room {
            Some(_) if self.owner.ends_with(']') && self.owner.contains('[') => {
                bail!("Use either a bracket room qualifier or --room, not both.")
            }
            Some(room) => Ok(format!("{} [{room}]", self.owner)),
            None => Ok(self.owner.clone()),
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum DirectionArg {
    Up,
    Down,
}

#[derive(Subcommand)]
pub(crate) enum CentralLightCmd {
    /// Show existing linked functions in persisted linkC order, not controlled lights
    Inspect {
        #[command(flatten)]
        target: CentralLightTarget,
    },
    /// Move an existing linked-function UUID one place up/down (standalone files only)
    MoveLink {
        #[command(flatten)]
        target: CentralLightTarget,
        /// Linked control U from inspect, NOT its position or display name
        uuid: String,
        #[arg(value_enum)]
        direction: DirectionArg,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
}

pub(crate) fn run(ctx: &RunContext, action: CentralLightCmd) -> Result<()> {
    let (target, edit, save_as) = match action {
        CentralLightCmd::Inspect { target } => (target, None, None),
        CentralLightCmd::MoveLink {
            target,
            uuid,
            direction,
            save_as,
        } => (target, Some((uuid, direction)), save_as),
    };
    let selector = target.selector()?;
    let original = std::fs::read(&target.file)?;
    let mut editor = ConfigEditor::load(&original)?;
    let movement = edit
        .map(|(uuid, direction)| {
            editor.move_central_light_link(
                &selector,
                &uuid,
                match direction {
                    DirectionArg::Up => CentralLightLinkDirection::Up,
                    DirectionArg::Down => CentralLightLinkDirection::Down,
                },
            )
        })
        .transpose()?;
    let inspection = editor.inspect_central_light_links(&selector)?;
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
        super::emit_json(
            ctx,
            serde_json::json!({
                "ok": true,
                "changed": movement.as_ref().is_some_and(|movement| movement.changed),
                "saved_to": saved_to,
                "movement": movement,
                "central_light": inspection,
            }),
        );
    } else {
        if !ctx.no_header {
            println!(
                "{}",
                if ctx.csv {
                    "POSITION,UUID,TYPE,TITLE"
                } else {
                    "POSITION  UUID                                 TYPE  TITLE"
                }
            );
        }
        for link in &inspection.links {
            if ctx.csv {
                println!(
                    "{},\"{}\",\"{}\",\"{}\"",
                    link.position,
                    link.uuid,
                    link.control_type.replace('"', "\"\""),
                    link.title.replace('"', "\"\"")
                );
            } else {
                println!(
                    "{:<10}{:<37}{}  {}",
                    link.position, link.uuid, link.control_type, link.title
                );
            }
        }
        if !ctx.csv && !ctx.quiet {
            println!("\n{} linked functions", inspection.links.len());
        }
    }
    Ok(())
}
