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
    /// Inspect stored Qn editor kinds, AQn references and unknown nested data
    Inspect {
        #[command(flatten)]
        target: LightingTarget,
    },
    /// Repair only Num from direct mood records; preserve Outputs and sparse Qn
    RecountMoods {
        #[command(flatten)]
        target: LightingTarget,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set a numeric mood level, preserving its type tag; reject other editor kinds
    SetMoodLevel {
        #[command(flatten)]
        target: LightingTarget,
        /// Exact stored SID or Name of one direct mood
        #[arg(long)]
        mood: String,
        /// 1-based Q index in the recovered 18-output editor path
        #[arg(long, default_value_t = 1)]
        output_index: usize,
        /// Integer level in 0..100; never clamped or packed as RGB
        #[arg(long)]
        level: u32,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Edit an existing CID with collision swaps and source-backed SID/ref remapping
    SetMoodCid {
        #[command(flatten)]
        target: LightingTarget,
        /// Exact stored SID or Name of one existing direct mood
        #[arg(long)]
        mood: String,
        /// Nonnegative signed-32-bit CID; 22/24 invoke native SID role handling
        #[arg(long)]
        cid: u32,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
}

pub(crate) fn run(ctx: &RunContext, action: LightingCmd) -> Result<()> {
    let target = match action {
        LightingCmd::Inspect { target } => target,
        LightingCmd::RecountMoods { target, save_as } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = editor.recount_lighting_moods(&selector)?;
            if edit.changed || save_as.is_some() {
                super::save_edited_with_quiet(&mut editor, &target.file, save_as.as_deref(), true)?;
            }
            if ctx.json {
                let mut value = serde_json::to_value(&edit)?;
                if let Some(trace_id) = &ctx.trace_id {
                    value["trace_id"] = trace_id.clone().into();
                }
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else if ctx.csv {
                if !ctx.no_header {
                    println!("BEFORE,AFTER,SCENES,CHANGED");
                }
                println!(
                    "{},{},{},{}",
                    csv(edit.before.as_deref().unwrap_or("")),
                    csv(&edit.after),
                    edit.scenes,
                    edit.changed
                );
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: Num={} ({} direct moods; Outputs and Qn unchanged)",
                    edit.after, edit.scenes
                );
            }
            return Ok(());
        }
        LightingCmd::SetMoodLevel {
            target,
            mood,
            output_index,
            level,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = editor.set_lighting_mood_level(&selector, &mood, output_index, level)?;
            if edit.changed || save_as.is_some() {
                super::save_edited_with_quiet(&mut editor, &target.file, save_as.as_deref(), true)?;
            }
            if ctx.json {
                let mut value = serde_json::to_value(&edit)?;
                if let Some(trace_id) = &ctx.trace_id {
                    value["trace_id"] = trace_id.clone().into();
                }
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else if ctx.csv {
                if !ctx.no_header {
                    println!("MOOD,OUTPUT,BEFORE,AFTER,LEVEL,TYPE_TAG,CHANGED");
                }
                println!(
                    "{},{},{},{},{},{},{}",
                    csv(&edit.mood),
                    edit.output,
                    csv(&edit.before),
                    csv(&edit.after),
                    edit.level,
                    edit.type_tag,
                    edit.changed
                );
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{}' {}={} (numeric level {}; type tag {} preserved)",
                    edit.mood, edit.output, edit.after, edit.level, edit.type_tag
                );
            }
            return Ok(());
        }
        LightingCmd::SetMoodCid {
            target,
            mood,
            cid,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = editor.set_lighting_mood_cid(&selector, &mood, cid)?;
            if edit.changed || save_as.is_some() {
                super::save_edited_with_quiet(&mut editor, &target.file, save_as.as_deref(), true)?;
            }
            if ctx.json {
                let mut value = serde_json::to_value(&edit)?;
                if let Some(trace_id) = &ctx.trace_id {
                    value["trace_id"] = trace_id.clone().into();
                }
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else if ctx.csv {
                if !ctx.no_header {
                    println!(
                        "RECORD,UUID,SID_BEFORE,SID_AFTER,CID_BEFORE,CID_AFTER,SCMV_BEFORE,SCMV_AFTER"
                    );
                }
                for change in &edit.identities {
                    println!(
                        "mood,{},{},{},{},{},,",
                        csv(change.uuid.as_deref().unwrap_or("")),
                        csv(&change.before_sid),
                        csv(&change.after_sid),
                        csv(change.before_cid.as_deref().unwrap_or("")),
                        csv(change.after_cid.as_deref().unwrap_or(""))
                    );
                }
                if let Some(selection) = &edit.selection_refs {
                    println!(
                        "selection,,,,,,{},{}",
                        csv(&selection.before),
                        csv(&selection.after)
                    );
                }
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{mood}' CID={} ({} identity records changed; {} ordered SID remaps)",
                    edit.requested_cid,
                    edit.identities.len(),
                    edit.selection_remaps.len()
                );
                if !ctx.no_header && !edit.identities.is_empty() {
                    println!("RECORD  UUID  SID BEFORE→AFTER  CID BEFORE→AFTER");
                }
                for change in &edit.identities {
                    println!(
                        "{}  {}  {}→{}  {}→{}",
                        change.record,
                        change.uuid.as_deref().unwrap_or("<missing>"),
                        change.before_sid,
                        change.after_sid,
                        change.before_cid.as_deref().unwrap_or("<absent>"),
                        change.after_cid.as_deref().unwrap_or("<absent>")
                    );
                }
                if let Some(selection) = &edit.selection_refs
                    && selection.changed
                {
                    println!("LSConfig ScMv: {} → {}", selection.before, selection.after);
                }
            }
            return Ok(());
        }
    };
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
                println!(
                    "RECORD,NAME,UUID,OUTPUT,STORED,EDITOR,TYPE_TAG,CAPABILITY_STORED,CAPABILITY_EFFECTIVE,ISSUE"
                );
            }
            for mood in &inspection.moods {
                for output in &mood.outputs {
                    println!(
                        "mood,{},{},{},{},{},{},{},{},{}",
                        csv(&mood.summary.name),
                        csv(&mood.summary.uuid),
                        output.key,
                        csv(&output.stored),
                        output.editor.kind.as_str(),
                        output
                            .editor
                            .type_tag
                            .map(|tag| tag.to_string())
                            .unwrap_or_default(),
                        csv(output
                            .editor
                            .capability
                            .as_ref()
                            .and_then(|cap| cap.stored.as_deref())
                            .unwrap_or("")),
                        output
                            .editor
                            .capability
                            .as_ref()
                            .and_then(|cap| cap.effective)
                            .map(|cap| cap.to_string())
                            .unwrap_or_default(),
                        output
                            .editor
                            .issue
                            .as_ref()
                            .map(|issue| issue.code.as_str())
                            .unwrap_or("")
                    );
                }
            }
            for channel in &inspection.channels {
                println!(
                    "channel,,{},{},,,,,,",
                    csv(channel.uuid.as_deref().unwrap_or("")),
                    channel.key
                );
            }
        } else {
            if !ctx.no_header {
                println!(
                    "{:<24} {:<10} {:<24} EDITOR",
                    "MOOD", "OUTPUT", "STORED (UNIT UNKNOWN)"
                );
                println!("{}", "─".repeat(76));
            }
            for mood in &inspection.moods {
                for output in &mood.outputs {
                    println!(
                        "{:<24} {:<10} {:<24} {}",
                        mood.summary.name,
                        output.key,
                        output.stored,
                        output.editor.kind.as_str()
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
        let crate::Cmd::Config { action } = cli.cmd else {
            panic!("config inspection was not dispatched");
        };
        let crate::ConfigCmd::Lighting { action } = *action else {
            panic!("lighting inspection was not dispatched");
        };
        let LightingCmd::Inspect { target } = action else {
            panic!("lighting inspection was not dispatched");
        };
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
