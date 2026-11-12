use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::RunContext;
use crate::config_edit::ConfigEditor;
use crate::config_edit::lighting::{
    LightingMoodColorPreview, LightingMoodCountPreview, LightingMoodCreationPreview,
    LightingMoodLevelPreview,
};
use crate::config_edit::lighting_cid::{
    LightingIdentityChange, LightingSelectionEdit, LightingSidRemap,
};
use crate::config_edit::lighting_metadata::{
    LightingMoodDisplayNamePreview, LightingMoodMixEdit, LightingMoodRenameEdit,
    LightingMoodSkipEdit,
};
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
    /// Set a numeric mood level, preserving its type tag; --dry-run previews without saving
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
    /// Edit an existing CID with collision swaps and SID/ref remapping; --dry-run plans without saving
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
    /// Reassign an ordinary SID1..90 with explicit exchange/CID-clear acceptance; --dry-run plans without saving
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
    /// Rename only an existing direct mood's Name; bounded ASCII input; --dry-run plans without saving
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

fn print_identity_preview(
    ctx: &RunContext,
    selected: &LightingIdentityChange,
    changes: &[LightingIdentityChange],
    remaps: &[LightingSidRemap],
    selection: Option<&LightingSelectionEdit>,
    effective: (&str, i32, i32),
    changed: bool,
) {
    if !ctx.no_header {
        println!(
            "{}",
            if ctx.csv {
                "RECORD,INDEX,UUID,FIELD,BEFORE_PRESENT,BEFORE,AFTER_PRESENT,AFTER"
            } else {
                "RECORD  INDEX  UUID  FIELD  BEFORE→AFTER"
            }
        );
    }
    let row = |record: &str,
               index: Option<usize>,
               uuid: Option<&str>,
               field: &str,
               before: Option<&str>,
               after: Option<&str>| {
        let index = index.map(|i| i.to_string()).unwrap_or_default();
        if ctx.csv {
            println!(
                "{},{},{},{},{},{},{},{}",
                csv(record),
                index,
                csv(uuid.unwrap_or("")),
                csv(field),
                before.is_some(),
                csv(before.unwrap_or("")),
                after.is_some(),
                csv(after.unwrap_or(""))
            );
        } else {
            println!(
                "{record}  {index}  {}  {field}  {}→{}",
                uuid.unwrap_or("<absent>"),
                before.unwrap_or("<absent>"),
                after.unwrap_or("<absent>")
            );
        }
    };
    let identities = std::iter::once(("selected", selected))
        .chain(changes.iter().map(|identity| ("edit", identity)));
    for (record, identity) in identities {
        for (field, before, after) in [
            (
                "SID",
                Some(identity.before_sid.as_str()),
                Some(identity.after_sid.as_str()),
            ),
            (
                "CID",
                identity.before_cid.as_deref(),
                identity.after_cid.as_deref(),
            ),
        ] {
            row(
                record,
                Some(identity.record),
                identity.uuid.as_deref(),
                field,
                before,
                after,
            );
        }
    }
    let (field, current, proposed) = effective;
    row(
        "selected_effective",
        Some(selected.record),
        selected.uuid.as_deref(),
        field,
        Some(&current.to_string()),
        Some(&proposed.to_string()),
    );
    for (index, remap) in remaps.iter().enumerate() {
        row(
            "ordered_remap",
            Some(index + 1),
            None,
            "ScMv SID",
            Some(&remap.before.to_string()),
            Some(&remap.after.to_string()),
        );
    }
    if let Some(selection) = selection {
        row(
            "selection",
            None,
            None,
            "ScMv",
            Some(&selection.before),
            Some(&selection.after),
        );
    }
    if !ctx.csv && !ctx.quiet {
        println!(
            "\n{} identity edits; selected effective {field} {current}→{proposed}; changed={changed}; dry-run wrote no files",
            changes.len(),
        );
    }
}

fn print_rename_preview(ctx: &RunContext, edit: &LightingMoodRenameEdit) {
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "MOOD_UUID_PRESENT,MOOD_UUID,SID_PRESENT,SID,CID_PRESENT,CID,BEFORE_PRESENT,BEFORE,AFTER,CHANGED"
            );
        }
        println!(
            "{},{},{},{},{},{},{},{},{},{}",
            edit.mood.uuid.is_some(),
            csv(edit.mood.uuid.as_deref().unwrap_or("")),
            edit.mood.sid.is_some(),
            csv(edit.mood.sid.as_deref().unwrap_or("")),
            edit.mood.cid.is_some(),
            csv(edit.mood.cid.as_deref().unwrap_or("")),
            edit.before.is_some(),
            csv(edit.before.as_deref().unwrap_or("")),
            csv(&edit.after),
            edit.changed
        );
    } else {
        if !ctx.no_header {
            println!("MOOD_UUID  SID  CID (RAW)  NAME BEFORE→AFTER  CHANGED");
        }
        println!(
            "{}  {}  {}  {}→{}  {}",
            edit.mood.uuid.as_deref().unwrap_or("<absent>"),
            edit.mood.sid.as_deref().unwrap_or("<absent>"),
            edit.mood.cid.as_deref().unwrap_or("<absent>"),
            edit.before.as_deref().unwrap_or("<absent>"),
            edit.after,
            edit.changed
        );
        if !ctx.quiet {
            println!("\n1 mood; selected Name only; dry-run wrote no files");
        }
    }
}

fn print_display_name_preview(ctx: &RunContext, preview: &LightingMoodDisplayNamePreview) {
    let edit = &preview.edit;
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "MOOD_UUID_PRESENT,MOOD_UUID,SID_PRESENT,SID,CID_PRESENT,CID,BEFORE_PRESENT,BEFORE,AFTER_PRESENT,AFTER_STORED,BEFORE_EFFECTIVE,AFTER_EFFECTIVE,NORMALIZED_FROM_PRESENT,NORMALIZED_FROM,CHANGED"
            );
        }
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            edit.mood.uuid.is_some(),
            csv(edit.mood.uuid.as_deref().unwrap_or("")),
            edit.mood.sid.is_some(),
            csv(edit.mood.sid.as_deref().unwrap_or("")),
            edit.mood.cid.is_some(),
            csv(edit.mood.cid.as_deref().unwrap_or("")),
            edit.before.is_some(),
            csv(edit.before.as_deref().unwrap_or("")),
            preview.after_stored.is_some(),
            csv(preview.after_stored.as_deref().unwrap_or("")),
            csv(&preview.before_effective),
            csv(&edit.after),
            edit.normalized_from.is_some(),
            csv(edit.normalized_from.as_deref().unwrap_or("")),
            edit.changed
        );
    } else {
        if !ctx.no_header {
            println!(
                "MOOD_UUID  SID  CID (RAW)  SNAME STORED BEFORE→AFTER  EFFECTIVE BEFORE→AFTER  NORMALIZED FROM  CHANGED"
            );
        }
        let stored = |value: Option<&str>| {
            value.map_or_else(|| "<absent>".to_owned(), |value| format!("{value:?}"))
        };
        println!(
            "{}  {}  {}  {}→{}  {:?}→{:?}  {}  {}",
            stored(edit.mood.uuid.as_deref()),
            stored(edit.mood.sid.as_deref()),
            stored(edit.mood.cid.as_deref()),
            stored(edit.before.as_deref()),
            stored(preview.after_stored.as_deref()),
            preview.before_effective,
            edit.after,
            stored(edit.normalized_from.as_deref()),
            edit.changed
        );
        if !ctx.quiet {
            println!("\n1 mood; selected SName only; dry-run wrote no files");
        }
    }
}

fn print_count_preview(ctx: &RunContext, preview: &LightingMoodCountPreview) {
    let path = preview
        .collection_path
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".");
    let before_effective = preview.before_effective.map(|value| value.to_string());
    let fields = [
        ("CONTROLLER_UUID", preview.controller_uuid.as_deref()),
        ("CONTROLLER_TITLE", preview.controller_title.as_deref()),
        ("COLLECTION_PATH", Some(path.as_str())),
        ("BEFORE", preview.edit.before.as_deref()),
        ("AFTER", Some(preview.edit.after.as_str())),
        ("BEFORE_EFFECTIVE", before_effective.as_deref()),
    ];
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "{},AFTER_EFFECTIVE,SCENES,CHANGED,SCOPE",
                fields
                    .iter()
                    .map(|(key, _)| format!("{key}_PRESENT,{key}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        println!(
            "{},{},{},{},{}",
            fields
                .iter()
                .map(|(_, value)| format!("{},{}", value.is_some(), csv(value.unwrap_or(""))))
                .collect::<Vec<_>>()
                .join(","),
            preview.after_effective,
            preview.edit.scenes,
            preview.edit.changed,
            preview.scope
        );
    } else {
        if !ctx.no_header {
            println!("COUNT REPAIR PREVIEW  FIELD  STORED/PLANNED VALUE");
        }
        for (key, value) in fields {
            println!(
                "{key}  {}",
                value.map_or_else(|| "<absent>".to_owned(), |value| format!("{value:?}"))
            );
        }
        println!(
            "AFTER_EFFECTIVE  {}\nSCENES  {}\nCHANGED  {}\nSCOPE  {}",
            preview.after_effective, preview.edit.scenes, preview.edit.changed, preview.scope
        );
        if !ctx.quiet {
            println!("\nNum only; Outputs and Qn unchanged; dry-run wrote no files");
        }
    }
}

pub(super) fn print_creation_preview(ctx: &RunContext, preview: &LightingMoodCreationPreview) {
    let path = preview
        .collection_path
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".");
    let mut fields = vec![
        (
            "CONTROLLER_UUID".to_owned(),
            preview.controller_uuid.as_deref(),
        ),
        (
            "CONTROLLER_TITLE".to_owned(),
            preview.controller_title.as_deref(),
        ),
        ("COLLECTION_PATH".to_owned(), Some(path.as_str())),
        ("BEFORE_NUM".to_owned(), preview.before_num.as_deref()),
        ("AFTER_NUM".to_owned(), Some(preview.after_num.as_str())),
    ];
    fields.extend(
        preview
            .proposed_mood
            .iter()
            .map(|(key, value)| (format!("PROPOSED_{key}"), Some(value.as_str()))),
    );
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "{},CONTAINER_CREATED,SCOPE",
                fields
                    .iter()
                    .map(|(key, _)| format!("{key}_PRESENT,{key}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        println!(
            "{},{},{}",
            fields
                .iter()
                .map(|(_, value)| format!("{},{}", value.is_some(), csv(value.unwrap_or(""))))
                .collect::<Vec<_>>()
                .join(","),
            preview.container_created,
            preview.scope
        );
    } else {
        if !ctx.no_header {
            println!("CREATION PREVIEW  FIELD  STORED/PROPOSED VALUE");
        }
        for (key, value) in fields {
            println!(
                "{key}  {}",
                value.map_or_else(|| "<absent>".to_owned(), |value| format!("{value:?}"))
            );
        }
        println!(
            "CONTAINER_CREATED  {}\nSCOPE  {}",
            preview.container_created, preview.scope
        );
        if !ctx.quiet {
            println!(
                "\n1 proposed LightsceneC; UUID is ephemeral across invocations; dry-run wrote no files"
            );
        }
    }
}

pub(super) fn print_color_preview(ctx: &RunContext, preview: &LightingMoodColorPreview) {
    let path = preview
        .selected_mood_path
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".");
    let fields = [
        ("CONTROLLER_UUID", preview.controller_uuid.as_deref()),
        ("CONTROLLER_TITLE", preview.controller_title.as_deref()),
        ("MOOD", Some(preview.mood.as_str())),
        ("MOOD_UUID", preview.selected_mood.uuid.as_deref()),
        ("SID", preview.selected_mood.sid.as_deref()),
        ("CID", preview.selected_mood.cid.as_deref()),
        ("NAME", preview.selected_mood_name.as_deref()),
        ("MOOD_PATH", Some(path.as_str())),
        ("OUTPUT", Some(preview.output.as_str())),
        ("BEFORE", preview.before.as_deref()),
        ("AFTER", Some(preview.after.as_str())),
    ];
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "{},OUTPUT_INDEX,VALUE,CHANGED,SCOPE",
                fields
                    .iter()
                    .map(|(key, _)| format!("{key}_PRESENT,{key}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        println!(
            "{},{},{},{},{}",
            fields
                .iter()
                .map(|(_, value)| format!("{},{}", value.is_some(), csv(value.unwrap_or(""))))
                .collect::<Vec<_>>()
                .join(","),
            preview.output_index,
            preview.value,
            preview.changed,
            preview.scope
        );
    } else {
        if !ctx.no_header {
            println!("RAW PACKED EDIT  FIELD  STORED VALUE");
        }
        for (key, value) in fields {
            println!(
                "{key}  {}",
                value.map_or_else(|| "<absent>".to_owned(), |value| format!("{value:?}"))
            );
        }
        println!(
            "OUTPUT_INDEX  {}\nVALUE  {}\nCHANGED  {}\nSCOPE  {}",
            preview.output_index, preview.value, preview.changed, preview.scope
        );
        if !ctx.quiet {
            println!(
                "\n1 mood; raw packed edit only, not a mode-aware color-picker operation; dry-run wrote no files"
            );
        }
    }
}

fn print_level_preview(ctx: &RunContext, preview: &LightingMoodLevelPreview) {
    let edit = &preview.edit;
    let capability = edit.capability.as_ref();
    let raw = capability.and_then(|value| value.stored.as_deref());
    let effective = capability.and_then(|value| value.effective);
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "MOOD,MOOD_UUID_PRESENT,MOOD_UUID,SID_PRESENT,SID,CID_PRESENT,CID,OUTPUT,OUTPUT_INDEX,EDITOR_KIND,BEFORE,AFTER,BEFORE_EFFECTIVE,AFTER_EFFECTIVE,LEVEL,TYPE_TAG,CAPABILITY_CONSULTED,CAPABILITY_PRESENT,CAPABILITY,CAPABILITY_EFFECTIVE,CHANGED"
            );
        }
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv(&edit.mood),
            preview.selected_mood.uuid.is_some(),
            csv(preview.selected_mood.uuid.as_deref().unwrap_or("")),
            preview.selected_mood.sid.is_some(),
            csv(preview.selected_mood.sid.as_deref().unwrap_or("")),
            preview.selected_mood.cid.is_some(),
            csv(preview.selected_mood.cid.as_deref().unwrap_or("")),
            edit.output,
            preview.output_index,
            preview.editor_kind.as_str(),
            csv(&edit.before),
            csv(&edit.after),
            preview.before_effective,
            preview.after_effective,
            edit.level,
            edit.type_tag,
            capability.is_some(),
            raw.is_some(),
            csv(raw.unwrap_or("")),
            effective.map(|value| value.to_string()).unwrap_or_default(),
            edit.changed
        );
    } else {
        if !ctx.no_header {
            println!(
                "MOOD_UUID  SID  CID (RAW)  OUTPUT/INDEX  EDITOR_KIND  STORED BEFORE→AFTER  SCALAR BEFORE→AFTER  TYPE_TAG  CAPABILITY RAW/EFFECTIVE  CHANGED"
            );
        }
        let stored = |value: Option<&str>| {
            value.map_or_else(|| "<absent>".to_owned(), |value| format!("{value:?}"))
        };
        let capability_text = if capability.is_some() {
            format!(
                "{}/{}",
                stored(raw),
                effective.map(|value| value.to_string()).unwrap_or_default()
            )
        } else {
            "<not consulted>".to_owned()
        };
        println!(
            "{}  {}  {}  {}/{}  {}  {:?}→{:?}  {}→{}  {}  {}  {}",
            stored(preview.selected_mood.uuid.as_deref()),
            stored(preview.selected_mood.sid.as_deref()),
            stored(preview.selected_mood.cid.as_deref()),
            edit.output,
            preview.output_index,
            preview.editor_kind.as_str(),
            edit.before,
            edit.after,
            preview.before_effective,
            preview.after_effective,
            edit.type_tag,
            capability_text,
            edit.changed
        );
        if !ctx.quiet {
            println!("\n1 mood; selected numeric Qn only; dry-run wrote no files");
        }
    }
}

fn print_mix_preview(ctx: &RunContext, edit: &LightingMoodMixEdit) {
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "MOOD_UUID,SID,CID_PRESENT,CID,CID_EFFECTIVE,BEFORE_PRESENT,BEFORE,AFTER_PRESENT,AFTER,BEFORE_EFFECTIVE,AFTER_EFFECTIVE,DEFAULT,CHANGED,SCOPE"
            );
        }
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv(edit.mood.uuid.as_deref().unwrap_or("")),
            csv(edit.mood.sid.as_deref().unwrap_or("")),
            edit.mood.cid.is_some(),
            csv(edit.mood.cid.as_deref().unwrap_or("")),
            edit.cid_effective,
            edit.before.is_some(),
            csv(edit.before.as_deref().unwrap_or("")),
            edit.after.is_some(),
            csv(edit.after.as_deref().unwrap_or("")),
            edit.before_effective,
            edit.enabled,
            edit.default,
            edit.changed,
            edit.scope
        );
    } else {
        if !ctx.no_header {
            println!(
                "MOOD_UUID  SID  CID (RAW/EFFECTIVE)  MIXMOD BEFORE→AFTER  EFFECTIVE BEFORE→AFTER  DEFAULT  CHANGED"
            );
        }
        println!(
            "{}  {}  {}/{}  {}→{}  {}→{}  {}  {}",
            edit.mood.uuid.as_deref().unwrap_or("<absent>"),
            edit.mood.sid.as_deref().unwrap_or("<absent>"),
            edit.mood.cid.as_deref().unwrap_or("<absent>"),
            edit.cid_effective,
            edit.before.as_deref().unwrap_or("<absent>"),
            edit.after.as_deref().unwrap_or("<absent>"),
            edit.before_effective,
            edit.enabled,
            edit.default,
            edit.changed
        );
        if !ctx.quiet {
            println!("\n1 mood; scope={}; dry-run wrote no files", edit.scope);
        }
    }
}

fn print_skip_preview(ctx: &RunContext, edit: &LightingMoodSkipEdit) {
    if ctx.csv {
        if !ctx.no_header {
            println!(
                "MOOD_UUID,SID,CID_PRESENT,CID,BEFORE_PRESENT,BEFORE,AFTER_PRESENT,AFTER,BEFORE_EFFECTIVE,AFTER_EFFECTIVE,DEFAULT,CHECKBOX_CHECKED,CHANGED"
            );
        }
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv(edit.mood.uuid.as_deref().unwrap_or("")),
            csv(edit.mood.sid.as_deref().unwrap_or("")),
            edit.mood.cid.is_some(),
            csv(edit.mood.cid.as_deref().unwrap_or("")),
            edit.before.stored.is_some(),
            csv(edit.before.stored.as_deref().unwrap_or("")),
            edit.after.is_some(),
            csv(edit.after.as_deref().unwrap_or("")),
            edit.before.effective.unwrap(),
            edit.skip,
            edit.before.default.unwrap(),
            edit.checkbox_checked,
            edit.changed
        );
    } else {
        if !ctx.no_header {
            println!(
                "MOOD_UUID  SID  CID (RAW)  SKIPNEXT BEFORE→AFTER  EFFECTIVE BEFORE→AFTER  DEFAULT  CHECKBOX_CHECKED  CHANGED"
            );
        }
        println!(
            "{}  {}  {}  {}→{}  {}→{}  {}  {}  {}",
            edit.mood.uuid.as_deref().unwrap_or("<absent>"),
            edit.mood.sid.as_deref().unwrap_or("<absent>"),
            edit.mood.cid.as_deref().unwrap_or("<absent>"),
            edit.before.stored.as_deref().unwrap_or("<absent>"),
            edit.after.as_deref().unwrap_or("<absent>"),
            edit.before.effective.unwrap(),
            edit.skip,
            edit.before.default.unwrap(),
            edit.checkbox_checked,
            edit.changed
        );
        if !ctx.quiet {
            println!("\n1 mood; selected SkipNext only; dry-run wrote no files");
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_display_name(
                    &selector,
                    &mood,
                    name,
                    normalize_ascii,
                )?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_display_name_preview(ctx, &preview);
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_count(&selector)?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    super::emit_json(ctx, value);
                } else {
                    print_count_preview(ctx, &preview);
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview =
                    editor.preview_lighting_mood_level(&selector, &mood, output_index, level)?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_level_preview(ctx, &preview);
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_sid(
                    &selector,
                    &mood,
                    sid,
                    swap,
                    allow_cid_clear,
                )?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_identity_preview(
                        ctx,
                        &preview.selected,
                        &preview.edit.identities,
                        &preview.edit.selection_remaps,
                        preview.edit.selection_refs.as_ref(),
                        ("SID", preview.current_sid, preview.edit.requested_sid),
                        preview.edit.changed,
                    );
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_cid(&selector, &mood, cid)?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_identity_preview(
                        ctx,
                        &preview.selected,
                        &preview.edit.identities,
                        &preview.edit.selection_remaps,
                        preview.edit.selection_refs.as_ref(),
                        ("CID", preview.current_cid, preview.edit.requested_cid),
                        preview.edit.changed,
                    );
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_rename(&selector, &mood, &name)?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_rename_preview(ctx, &preview);
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_mix(&selector, &mood, mix)?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_mix_preview(ctx, &preview);
                }
                return Ok(());
            }
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
            if ctx.dry_run {
                let preview = editor.preview_lighting_mood_skip(&selector, &mood, skip)?;
                if ctx.json {
                    let mut value = serde_json::to_value(&preview)?;
                    value["dry_run"] = true.into();
                    if let Some(trace_id) = &ctx.trace_id {
                        value["trace_id"] = trace_id.clone().into();
                    }
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    print_skip_preview(ctx, &preview);
                }
                return Ok(());
            }
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
