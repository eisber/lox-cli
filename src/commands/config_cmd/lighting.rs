use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::schema::Diagnostic;

#[derive(Args)]
pub(crate) struct LightingTarget {
    /// Local .Loxone XML file; never connects to a Miniserver
    file: String,
    /// LightController2 title, uuid:... or room-qualified selector
    block: String,
    /// Room disambiguation
    #[arg(short, long)]
    room: Option<String>,
}

impl LightingTarget {
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
pub(crate) enum LightingCmd {
    /// Inspect all stored mood Qn, AQn connectors, references and unknown nested data
    Inspect {
        #[command(flatten)]
        target: LightingTarget,
    },
}

pub(crate) fn run(ctx: &RunContext, action: LightingCmd) -> Result<()> {
    let LightingCmd::Inspect { target } = action;
    let selector = target.selector()?;
    let editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
    let inspection = editor.inspect_lighting(&selector)?;
    if ctx.json {
        let mut value = serde_json::to_value(&inspection)?;
        if let Some(trace_id) = &ctx.trace_id {
            value["trace_id"] = trace_id.clone().into();
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        if ctx.csv {
            if !ctx.no_header {
                println!("RECORD,NAME,UUID,OUTPUT,STORED");
            }
            for mood in &inspection.moods {
                for output in &mood.outputs {
                    println!(
                        "mood,{},{},{},{}",
                        csv(&mood.summary.name),
                        csv(&mood.summary.uuid),
                        output.key,
                        csv(&output.stored)
                    );
                }
            }
            for channel in &inspection.channels {
                println!(
                    "channel,,{},{},",
                    csv(channel.uuid.as_deref().unwrap_or("")),
                    channel.key
                );
            }
        } else {
            if !ctx.no_header {
                println!("{:<24} {:<10} STORED (UNIT UNKNOWN)", "MOOD", "OUTPUT");
                println!("{}", "─".repeat(60));
            }
            for mood in &inspection.moods {
                for output in &mood.outputs {
                    println!(
                        "{:<24} {:<10} {}",
                        mood.summary.name, output.key, output.stored
                    );
                }
            }
            println!(
                "\n{} moods, {} stored output connectors",
                inspection.moods.len(),
                inspection.channels.len()
            );
            for channel in &inspection.channels {
                println!(
                    "{}  UUID={}  {} downstream references; role unknown",
                    channel.key,
                    channel.uuid.as_deref().unwrap_or("<missing>"),
                    channel.downstream.len()
                );
            }
        }
        // Limitations must remain visible even with --quiet.
        for diagnostic in &inspection.diagnostics {
            eprintln!("Warning: {diagnostic}");
        }
    }
    Ok(())
}

fn csv(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn lighting_cli_preserves_dispatch_and_room_guards() {
        let cli = crate::Cli::try_parse_from([
            "lox",
            "config",
            "lighting",
            "inspect",
            "fixture.Loxone",
            "Lamp",
            "--room",
            "Study",
            "-o",
            "json",
        ])
        .unwrap();
        let crate::Cmd::Config {
            action: crate::ConfigCmd::Lighting { action },
        } = cli.cmd
        else {
            panic!("lighting inspection was not dispatched");
        };
        let LightingCmd::Inspect { target } = action;
        assert_eq!(target.selector().unwrap(), "Lamp [Study]");
        let conflicting = LightingTarget {
            file: target.file,
            block: "Lamp [Study]".into(),
            room: Some("Study".into()),
        };
        assert!(conflicting.selector().is_err());
        assert!(
            crate::Cli::try_parse_from(["lox", "config", "moods", "fixture.Loxone", "Lamp"])
                .is_ok()
        );
        assert!(
            crate::Cli::try_parse_from([
                "lox",
                "config",
                "lighting",
                "set",
                "fixture.Loxone",
                "Lamp"
            ])
            .is_err()
        );
    }
}
