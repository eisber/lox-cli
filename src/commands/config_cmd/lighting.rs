use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::lighting_cid::{LightingIdentityChange, LightingSelectionEdit};
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
    /// Inspect stored mood flags, display text, icon ID and bounded checkbox projections
    InspectMoodOptions {
        #[command(flatten)]
        target: LightingTarget,
        #[arg(long)]
        mood: String,
    },
    /// Inspect selected stored SID references in the OpM-bounded ScMv prefix; never save
    InspectMoodReferences {
        #[command(flatten)]
        target: LightingTarget,
        /// Exact stored SID spelling or case-sensitive Name of one direct mood
        #[arg(long)]
        mood: String,
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
    /// Reassign an ordinary SID1..90 with explicit exchange/CID-clear acceptance
    SetMoodSid {
        #[command(flatten)]
        target: LightingTarget,
        #[arg(long)]
        mood: String,
        #[arg(long)]
        sid: u32,
        /// Accept exchanging occupied ordinary SIDs and ordered selection references
        #[arg(long)]
        swap: bool,
        /// Accept native CID clearing for a free SID32..90 when the selected CID is positive
        #[arg(long)]
        allow_cid_clear: bool,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Rename only an existing direct mood's Name; bounded ASCII input
    RenameMood {
        #[command(flatten)]
        target: LightingTarget,
        /// Exact stored SID or case-sensitive Name; ambiguous matches are rejected
        #[arg(long)]
        mood: String,
        /// ASCII letters/digits and internal spaces, with a letter/digit at each end
        #[arg(long)]
        name: String,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set MixMod for role-independent SID1..31 moods; CID12..19 is unsupported
    SetMoodMix {
        #[command(flatten)]
        target: LightingTarget,
        #[arg(long)]
        mood: String,
        /// Enable the existing per-mood option; not role/cohort membership
        #[arg(long, required_unless_present = "no_mix", conflicts_with = "no_mix")]
        mix: bool,
        /// Disable the existing per-mood option; not role/cohort membership
        #[arg(long, required_unless_present = "mix", conflicts_with = "mix")]
        no_mix: bool,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set SkipNext only; the native cycling checkbox displays its inverse
    SetMoodSkip {
        #[command(flatten)]
        target: LightingTarget,
        #[arg(long)]
        mood: String,
        /// Exclude this mood when cycling (SkipNext=true)
        #[arg(long, required_unless_present = "no_skip", conflicts_with = "no_skip")]
        skip: bool,
        /// Include this mood when cycling (SkipNext=false)
        #[arg(long, required_unless_present = "skip", conflicts_with = "skip")]
        no_skip: bool,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Set only SName (display name), using uppercase ASCII; or explicitly clear it
    SetMoodDisplayName {
        #[command(flatten)]
        target: LightingTarget,
        #[arg(long)]
        mood: String,
        /// Uppercase ASCII letters/digits and internal spaces, with alphanumeric edges
        #[arg(long, required_unless_present = "clear", conflicts_with = "clear",
            value_parser = clap::builder::NonEmptyStringValueParser::new())]
        name: Option<String>,
        /// Clear SName, not Name; no inherited-label or hardware-display claim
        #[arg(long, required_unless_present = "name", conflicts_with = "name")]
        clear: bool,
        /// Explicitly apply native ASCII lowercase-to-uppercase conversion; no transliteration
        #[arg(long, requires = "name", conflicts_with = "clear")]
        normalize_ascii: bool,
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
}

fn print_identity_changes(
    ctx: &RunContext,
    identities: &[LightingIdentityChange],
    selection: Option<&LightingSelectionEdit>,
) {
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "RECORD,UUID,SID_BEFORE,SID_AFTER,CID_BEFORE,CID_AFTER,SCMV_BEFORE,SCMV_AFTER"
            );
        }
        for change in identities {
            println!(
                "mood,{},{},{},{},{},,",
                csv(change.uuid.as_deref().unwrap_or("")),
                csv(&change.before_sid),
                csv(&change.after_sid),
                csv(change.before_cid.as_deref().unwrap_or("")),
                csv(change.after_cid.as_deref().unwrap_or(""))
            );
        }
        if let Some(selection) = selection {
            println!(
                "selection,,,,,,{},{}",
                csv(&selection.before),
                csv(&selection.after)
            );
        }
    } else {
        if !ctx.no_header && !identities.is_empty() {
            println!("RECORD  UUID  SID BEFORE→AFTER  CID BEFORE→AFTER");
        }
        for change in identities {
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
        if let Some(selection) = selection
            && selection.changed
        {
            println!("LSConfig ScMv: {} → {}", selection.before, selection.after);
        }
    }
}

pub(crate) fn run(ctx: &RunContext, action: LightingCmd) -> Result<()> {
    let target = match action {
        LightingCmd::Inspect { target } => target,
        LightingCmd::InspectMoodReferences { target, mood } => {
            let selector = target.selector()?;
            let editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let inspection = editor.inspect_lighting_mood_references(&selector, &mood)?;
            if ctx.json {
                let mut value = serde_json::to_value(&inspection)?;
                if let Some(trace_id) = &ctx.trace_id {
                    value["trace_id"] = trace_id.clone().into();
                }
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                if !ctx.no_header {
                    println!(
                        "{}",
                        if ctx.csv {
                            "RECORD,INDEX,RAW_PRESENT,RAW,VALUE,MATCHES_SELECTED_SID"
                        } else {
                            "RECORD  INDEX  RAW  VALUE  MATCHES_SELECTED_SID"
                        }
                    );
                }
                let row = |record: &str,
                           index: Option<usize>,
                           raw: Option<&str>,
                           value: Option<String>,
                           matches: Option<bool>| {
                    let index = index.map(|i| i.to_string()).unwrap_or_default();
                    let value = value.unwrap_or_default();
                    let matches = matches.map(|v| v.to_string()).unwrap_or_default();
                    if ctx.csv {
                        println!(
                            "{},{},{},{},{},{}",
                            csv(record),
                            index,
                            raw.is_some(),
                            csv(raw.unwrap_or("")),
                            csv(&value),
                            matches
                        );
                    } else {
                        println!(
                            "{record}  {index}  {}  {value}  {matches}",
                            raw.unwrap_or("<absent>")
                        );
                    }
                };
                row("status", None, None, Some(inspection.status.into()), None);
                row(
                    "selected_sid",
                    None,
                    inspection.mood.sid.as_deref(),
                    Some(inspection.selected_sid.to_string()),
                    None,
                );
                row("Num", None, inspection.stored_num.as_deref(), None, None);
                row(
                    "direct_moods",
                    None,
                    None,
                    Some(inspection.direct_moods.to_string()),
                    None,
                );
                row(
                    "lsconfig_present",
                    None,
                    None,
                    Some(inspection.lsconfig_present.to_string()),
                    None,
                );
                row(
                    "OpM",
                    None,
                    inspection.opm.as_deref(),
                    inspection.opm_count.map(|n| n.to_string()),
                    None,
                );
                row("ScMv", None, inspection.scmv.as_deref(), None, None);
                if let Some(visited) = &inspection.visited {
                    for reference in visited {
                        row(
                            "visited",
                            Some(reference.index),
                            Some(&reference.raw),
                            Some(reference.value.to_string()),
                            Some(reference.matches_selected_sid),
                        );
                    }
                }
                if let Some(tail) = &inspection.unvisited_tail {
                    row(
                        "unvisited_tail",
                        Some(tail.start_index),
                        Some(&tail.raw),
                        Some(tail.token_count.to_string()),
                        None,
                    );
                }
                if !ctx.csv && !ctx.quiet {
                    match &inspection.visited {
                        Some(visited) => println!("\n{} visited references", visited.len()),
                        None => println!("\nvisited references unresolved"),
                    }
                }
                for d in &inspection.diagnostics {
                    eprintln!("{} at {}: {}", d.code, d.path, d.message);
                }
            }
            return Ok(());
        }
        LightingCmd::InspectMoodOptions { target, mood } => {
            let selector = target.selector()?;
            let editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let inspection = editor.inspect_lighting_mood_options(&selector, &mood)?;
            if ctx.json {
                let mut value = serde_json::to_value(&inspection)?;
                if let Some(trace_id) = &ctx.trace_id {
                    value["trace_id"] = trace_id.clone().into();
                }
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                let bool_text =
                    |value: Option<bool>| value.map(|v| v.to_string()).unwrap_or_default();
                if !ctx.no_header {
                    println!(
                        "{}",
                        if ctx.csv {
                            "OPTION,STORED_PRESENT,STORED,EFFECTIVE,DEFAULT,CHECKBOX_CHECKED,PROJECTION"
                        } else {
                            "OPTION  STORED  EFFECTIVE  DEFAULT  CHECKBOX  PROJECTION"
                        }
                    );
                }
                for flag in &inspection.flags {
                    let stored = flag.value.stored.as_deref();
                    let effective = bool_text(flag.value.effective);
                    let default = bool_text(flag.value.default);
                    let checked = bool_text(flag.checkbox_checked);
                    if ctx.csv {
                        println!(
                            "{},{},{},{},{},{},{}",
                            csv(flag.key),
                            stored.is_some(),
                            csv(stored.unwrap_or("")),
                            csv(&effective),
                            csv(&default),
                            csv(&checked),
                            csv(flag.projection)
                        );
                    } else {
                        println!(
                            "{}  {}  {}  {}  {}  {}",
                            flag.key,
                            stored.unwrap_or("<absent>"),
                            if effective.is_empty() {
                                "<unresolved>"
                            } else {
                                &effective
                            },
                            default,
                            if checked.is_empty() {
                                "<unknown>"
                            } else {
                                &checked
                            },
                            flag.projection
                        );
                    }
                }
                let name = &inspection.display_name;
                if ctx.csv {
                    println!(
                        "\"SName\",{},{},{},{},,\"stored_text\"",
                        name.stored.is_some(),
                        csv(name.stored.as_deref().unwrap_or("")),
                        csv(name.effective.as_deref().unwrap_or("")),
                        csv(name.default.as_deref().unwrap_or(""))
                    );
                } else {
                    println!(
                        "SName  {}  {}  \"\"  <unknown>  stored_text",
                        name.stored.as_deref().unwrap_or("<absent>"),
                        name.effective.as_deref().unwrap_or("")
                    );
                }
                let icon = &inspection.icon_id;
                let icon_value = icon.effective.map(|v| v.to_string()).unwrap_or_default();
                if ctx.csv {
                    println!(
                        "\"IID\",{},{},{},\"0\",,\"icon_reference_unresolved\"",
                        icon.stored.is_some(),
                        csv(icon.stored.as_deref().unwrap_or("")),
                        csv(&icon_value)
                    );
                } else {
                    println!(
                        "IID  {}  {}  0  <unknown>  icon_reference_unresolved",
                        icon.stored.as_deref().unwrap_or("<absent>"),
                        if icon_value.is_empty() {
                            "<unresolved>"
                        } else {
                            &icon_value
                        }
                    );
                }
                if !ctx.csv && !ctx.quiet {
                    println!("\n{} options", inspection.flags.len() + 2);
                }
                for d in &inspection.diagnostics {
                    eprintln!("{} at {}: {}", d.code, d.path, d.message);
                }
            }
            return Ok(());
        }
        LightingCmd::SetMoodDisplayName {
            target,
            mood,
            name,
            clear: _,
            normalize_ascii,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let name = name.as_deref().unwrap_or("");
            let edit = if normalize_ascii {
                editor.set_lighting_mood_display_name_ascii(&selector, &mood, name)?
            } else {
                editor.set_lighting_mood_display_name(&selector, &mood, name)?
            };
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
                    println!("UUID,SID,CID,BEFORE,AFTER,CHANGED");
                }
                println!(
                    "{},{},{},{},{},{}",
                    csv(edit.mood.uuid.as_deref().unwrap_or("")),
                    csv(edit.mood.sid.as_deref().unwrap_or("")),
                    csv(edit.mood.cid.as_deref().unwrap_or("")),
                    csv(edit.before.as_deref().unwrap_or("")),
                    csv(&edit.after),
                    edit.changed
                );
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{mood}' SName='{}' (Name and other storage unchanged)",
                    edit.after
                );
            }
            return Ok(());
        }
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
        LightingCmd::SetMoodSid {
            target,
            mood,
            sid,
            swap,
            allow_cid_clear,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = if allow_cid_clear {
                editor.set_lighting_mood_sid_with_cid_clear(&selector, &mood, sid, swap, true)?
            } else {
                editor.set_lighting_mood_sid(&selector, &mood, sid, swap)?
            };
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
                print_identity_changes(ctx, &edit.identities, edit.selection_refs.as_ref());
            } else if !ctx.quiet {
                let cid_status = if edit.cid_cleared {
                    "selected CID cleared to0"
                } else {
                    "CIDs unchanged"
                };
                println!(
                    "✓  {selector}: mood '{mood}' SID={} ({} identity records changed; {} ordered SID remaps; {cid_status})",
                    edit.requested_sid,
                    edit.identities.len(),
                    edit.selection_remaps.len()
                );
                print_identity_changes(ctx, &edit.identities, edit.selection_refs.as_ref());
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
                print_identity_changes(ctx, &edit.identities, edit.selection_refs.as_ref());
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{mood}' CID={} ({} identity records changed; {} ordered SID remaps)",
                    edit.requested_cid,
                    edit.identities.len(),
                    edit.selection_remaps.len()
                );
                print_identity_changes(ctx, &edit.identities, edit.selection_refs.as_ref());
            }
            return Ok(());
        }
        LightingCmd::RenameMood {
            target,
            mood,
            name,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = editor.rename_lighting_mood(&selector, &mood, &name)?;
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
                    println!("MOOD_UUID,SID,CID,BEFORE,AFTER,CHANGED");
                }
                println!(
                    "{},{},{},{},{},{}",
                    csv(edit.mood.uuid.as_deref().unwrap_or("")),
                    csv(edit.mood.sid.as_deref().unwrap_or("")),
                    csv(edit.mood.cid.as_deref().unwrap_or("")),
                    csv(edit.before.as_deref().unwrap_or("")),
                    csv(&edit.after),
                    edit.changed
                );
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{mood}' Name={} (selected Name only; bounded ASCII editor)",
                    edit.after
                );
            }
            return Ok(());
        }
        LightingCmd::SetMoodMix {
            target,
            mood,
            mix,
            no_mix: _,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = editor.set_lighting_mood_mix(&selector, &mood, mix)?;
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
                    println!("MOOD_UUID,SID,CID,BEFORE,AFTER,ENABLED,CHANGED,SCOPE");
                }
                println!(
                    "{},{},{},{},{},{},{},{}",
                    csv(edit.mood.uuid.as_deref().unwrap_or("")),
                    csv(edit.mood.sid.as_deref().unwrap_or("")),
                    csv(edit.mood.cid.as_deref().unwrap_or("")),
                    csv(edit.before.as_deref().unwrap_or("")),
                    csv(edit.after.as_deref().unwrap_or("")),
                    edit.enabled,
                    edit.changed,
                    edit.scope
                );
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{mood}' MixMod={} (selected role-independent record only)",
                    edit.after.as_deref().unwrap_or("<absent; effective false>")
                );
            }
            return Ok(());
        }
        LightingCmd::SetMoodSkip {
            target,
            mood,
            skip,
            no_skip: _,
            save_as,
        } => {
            let selector = target.selector()?;
            let mut editor = ConfigEditor::load(&crate::load_config_xml(&target.file)?)?;
            let edit = editor.set_lighting_mood_skip(&selector, &mood, skip)?;
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
                        "MOOD_UUID,BEFORE_STORED,BEFORE_EFFECTIVE,AFTER_STORED,SKIP,CHECKBOX_CHECKED,CHANGED"
                    );
                }
                println!(
                    "{},{},{},{},{},{},{}",
                    csv(edit.mood.uuid.as_deref().unwrap_or("")),
                    csv(edit.before.stored.as_deref().unwrap_or("")),
                    edit.before.effective.unwrap(),
                    csv(edit.after.as_deref().unwrap_or("")),
                    edit.skip,
                    edit.checkbox_checked,
                    edit.changed
                );
            } else if !ctx.quiet {
                println!(
                    "✓  {selector}: mood '{mood}' SkipNext={} (native cycling checkbox checked={})",
                    edit.skip, edit.checkbox_checked
                );
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
