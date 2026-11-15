//! `lox stats` — read the statistics the Miniserver logs to its SD card.
//!
//! The Miniserver serves an HTML index at `/stats/` with one entry per
//! statistic and month, and each month as a self-describing XML file
//! (`/stats/<uuid>[_<group>].<YYYYMM>.xml`). Everything here is read-only.

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lox_core::statistics::{
    StatSeries, StatisticIdentity, StatisticResolutionError, StatisticsError, parse_series,
    resolve_identity,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::client::LoxClient;
use crate::commands::RunContext;
use crate::config::Config;

#[derive(Subcommand)]
pub enum StatsCmd {
    /// List the statistics the Miniserver keeps: GET /stats/
    #[command(name = "ls", alias = "list")]
    List {
        /// Case-insensitive substring match on the label
        #[arg(long)]
        filter: Option<String>,
        /// Only this month (YYYY-MM)
        #[arg(long, value_parser = parse_month)]
        month: Option<String>,
        /// One row per statistic and group, newest month only
        #[arg(long)]
        latest: bool,
    },
    /// Print the logged values of one statistic across months
    Get {
        /// Statistic name (case-insensitive, exact match first, then substring) or UUID
        name: String,
        /// Statistic group, for grouped statistics (e.g. meters)
        #[arg(long)]
        group: Option<u32>,
        /// First month to include (YYYY-MM, inclusive; default: oldest available)
        #[arg(long, value_parser = parse_month)]
        from: Option<String>,
        /// Last month to include (YYYY-MM, inclusive; default: newest available)
        #[arg(long, value_parser = parse_month)]
        to: Option<String>,
    },
    /// Mirror the raw monthly XML files into a folder (incremental)
    Download {
        /// Target folder (created when missing)
        #[arg(long)]
        dir: PathBuf,
        /// Case-insensitive substring match on the label
        #[arg(long)]
        filter: Option<String>,
        /// First month to include (YYYY-MM, inclusive)
        #[arg(long, value_parser = parse_month)]
        from: Option<String>,
        /// Last month to include (YYYY-MM, inclusive)
        #[arg(long, value_parser = parse_month)]
        to: Option<String>,
        /// Refetch files that already exist locally
        #[arg(long)]
        force: bool,
    },
}

pub fn cmd_stats(ctx: &RunContext, action: StatsCmd) -> Result<()> {
    let cfg = Config::load()?;
    let lox = LoxClient::new(cfg)?;
    let mut out = std::io::stdout().lock();
    match action {
        StatsCmd::List {
            filter,
            month,
            latest,
        } => list(
            ctx,
            &lox,
            filter.as_deref(),
            month.as_deref(),
            latest,
            &mut out,
        ),
        StatsCmd::Get {
            name,
            group,
            from,
            to,
        } => get(ctx, &lox, &name, group, &MonthRange { from, to }, &mut out),
        StatsCmd::Download {
            dir,
            filter,
            from,
            to,
            force,
        } => {
            let opts = DownloadOpts {
                dir,
                filter,
                range: MonthRange { from, to },
                force,
            };
            download(ctx, &lox, &opts, chrono::Local::now(), &mut out)
        }
    }
}

struct DownloadOpts {
    dir: PathBuf,
    filter: Option<String>,
    range: MonthRange,
    force: bool,
}

/// Mirror the monthly XML files into `opts.dir`.
///
/// Existing files of months before the current one are skipped when they
/// were saved after that month ended (see [`is_final_copy`]); everything else
/// is fetched. `opts.force` refetches every file (P6).
fn download(
    ctx: &RunContext,
    lox: &LoxClient,
    opts: &DownloadOpts,
    now: chrono::DateTime<chrono::Local>,
    out: &mut dyn Write,
) -> Result<()> {
    opts.range.check()?;
    let current_month = now.format("%Y-%m").to_string();
    let filter = opts.filter.as_deref().map(str::to_lowercase);
    let entries: Vec<StatEntry> = fetch_index(lox)?
        .into_iter()
        .filter(|e| {
            filter
                .as_deref()
                .is_none_or(|f| e.label.to_lowercase().contains(f))
        })
        .filter(|e| opts.range.contains(&e.month))
        .collect();

    let mut fetch = Vec::new();
    let mut skipped = Vec::new();
    for e in entries {
        let path = opts.dir.join(&e.file);
        if !opts.force && e.month < current_month && is_final_copy(&path, &e.month) {
            skipped.push(e.file);
        } else {
            fetch.push(e.file);
        }
    }

    if ctx.dry_run {
        if ctx.json {
            let doc = serde_json::json!({
                "dry_run": true,
                "would_download": fetch,
                "skipped": skipped,
            });
            writeln!(out, "{}", serde_json::to_string_pretty(&doc)?)?;
        } else if ctx.csv {
            write_download_csv(ctx, &fetch, &skipped, true, out)?;
        } else {
            for f in &fetch {
                writeln!(out, "{f}")?;
            }
            if !ctx.quiet {
                writeln!(
                    out,
                    "{} would be downloaded, {} skipped (dry run)",
                    fetch.len(),
                    skipped.len()
                )?;
            }
        }
        return Ok(());
    }

    std::fs::create_dir_all(&opts.dir)
        .with_context(|| format!("creating {}", opts.dir.display()))?;
    for f in &fetch {
        let bytes = lox
            .get_bytes(&format!("/stats/{f}"))
            .with_context(|| format!("reading {f}"))?;
        // Write via a temp file so an interrupted run never leaves a truncated
        // file behind that a later run would skip as complete.
        let target = opts.dir.join(f);
        let part = opts.dir.join(format!("{f}.part"));
        std::fs::write(&part, &bytes).with_context(|| format!("writing {}", part.display()))?;
        std::fs::rename(&part, &target).with_context(|| format!("writing {}", target.display()))?;
    }

    if ctx.json {
        let doc = serde_json::json!({ "downloaded": fetch, "skipped": skipped });
        writeln!(out, "{}", serde_json::to_string_pretty(&doc)?)?;
    } else if ctx.csv {
        write_download_csv(ctx, &fetch, &skipped, false, out)?;
    } else if !ctx.quiet {
        writeln!(out, "{} downloaded, {} skipped", fetch.len(), skipped.len())?;
    }
    Ok(())
}

fn write_download_csv(
    ctx: &RunContext,
    fetch: &[String],
    skipped: &[String],
    dry_run: bool,
    out: &mut dyn Write,
) -> Result<()> {
    if !ctx.no_header {
        writeln!(out, "FILE,STATUS")?;
    }
    let status = if dry_run {
        "would_download"
    } else {
        "downloaded"
    };
    for file in fetch {
        writeln!(out, "{},{status}", csv_field(file))?;
    }
    for file in skipped {
        writeln!(out, "{},skipped", csv_field(file))?;
    }
    Ok(())
}

/// Whether the local file at `path` holds the complete data of `month`.
///
/// The Miniserver keeps appending to a month's file until the month is over,
/// and the file gets the next month's first row at 02:00. A copy counts as
/// final when it was saved at or after the first day of the following month
/// plus one day (local time), which also absorbs clock skew. A missing file or
/// unreadable modification time is never final.
fn is_final_copy(path: &Path, month: &str) -> bool {
    use chrono::TimeZone;
    let Ok(first) = chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d") else {
        return false;
    };
    let Some(cutoff) = first
        .checked_add_months(chrono::Months::new(1))
        .and_then(|d| d.succ_opt())
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .and_then(|d| chrono::Local.from_local_datetime(&d).earliest())
    else {
        return false;
    };
    let Ok(modified) = std::fs::metadata(path).and_then(|m| m.modified()) else {
        return false;
    };
    chrono::DateTime::<chrono::Local>::from(modified) >= cutoff
}

/// Validate a `YYYY-MM` argument.
fn parse_month(s: &str) -> std::result::Result<String, String> {
    chrono::NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d")
        .ok()
        .filter(|_| s.len() == 7)
        .map(|_| s.to_string())
        .ok_or_else(|| format!("expected YYYY-MM, got '{s}'"))
}

/// Inclusive month range; `None` bounds are open.
struct MonthRange {
    from: Option<String>,
    to: Option<String>,
}

impl MonthRange {
    /// Reject a range whose start lies after its end.
    fn check(&self) -> Result<()> {
        if let (Some(from), Some(to)) = (&self.from, &self.to)
            && from > to
        {
            bail!("--from {from} is later than --to {to}");
        }
        Ok(())
    }

    fn contains(&self, month: &str) -> bool {
        self.from.as_deref().is_none_or(|f| month >= f)
            && self.to.as_deref().is_none_or(|t| month <= t)
    }
}

/// One file listed in the `/stats/` index.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatEntry {
    /// File name as served, e.g. `0a1b2c3d-4e5f-6a7b-8c9d0e1f2a3b4c5d_2.202610.xml`
    pub file: String,
    pub uuid: String,
    pub group: Option<u32>,
    /// Month as `YYYY-MM`
    pub month: String,
    /// Statistic name (entity-decoded, group and room parts stripped)
    pub name: String,
    /// Full entity-decoded label without the trailing `YYYYMM`
    pub label: String,
}

/// Quote a CSV field when it contains a separator, quote or newline.
fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn group_label(group: Option<u32>) -> String {
    group.map_or_else(|| "-".to_string(), |g| g.to_string())
}

/// Fetch and parse the `/stats/` index.
fn fetch_index(lox: &LoxClient) -> Result<Vec<StatEntry>> {
    let html = lox
        .get_text("/stats/")
        .context("reading the statistics index (/stats/)")?;
    Ok(parse_index(&html))
}

fn fetch_series(lox: &LoxClient, entry: &StatEntry) -> Result<StatSeries> {
    let xml = lox
        .get_text(&format!("/stats/{}", entry.file))
        .with_context(|| format!("reading {}", entry.file))?;
    parse_series(&xml)
        .map_err(statistics_error)
        .with_context(|| format!("parsing {}", entry.file))
}

fn statistics_error(error: StatisticsError) -> anyhow::Error {
    let message = error.to_string();
    match error {
        StatisticsError::Xml { source } => anyhow::Error::new(source).context(message),
        StatisticsError::MalformedAttribute { source, .. } => {
            anyhow::Error::new(source).context(message)
        }
        StatisticsError::AttributeValue { source, .. } => source.into(),
        StatisticsError::InvalidStatsGroup { source, .. }
        | StatisticsError::InvalidNumOutputs { source, .. } => {
            anyhow::Error::new(source).context(message)
        }
        StatisticsError::InvalidValue { source, .. } => anyhow::Error::new(source).context(message),
        other => other.into(),
    }
}

fn list(
    ctx: &RunContext,
    lox: &LoxClient,
    filter: Option<&str>,
    month: Option<&str>,
    latest: bool,
    out: &mut dyn Write,
) -> Result<()> {
    let filter = filter.map(|f| f.to_lowercase());
    let mut entries: Vec<StatEntry> = fetch_index(lox)?
        .into_iter()
        .filter(|e| {
            filter
                .as_deref()
                .is_none_or(|f| e.label.to_lowercase().contains(f))
        })
        .filter(|e| month.is_none_or(|m| e.month == m))
        .collect();
    if latest {
        let mut newest: HashMap<(String, Option<u32>), String> = HashMap::new();
        for e in &entries {
            let m = newest.entry((e.uuid.clone(), e.group)).or_default();
            if e.month > *m {
                *m = e.month.clone();
            }
        }
        entries.retain(|e| newest[&(e.uuid.clone(), e.group)] == e.month);
    }

    if ctx.json {
        writeln!(out, "{}", serde_json::to_string_pretty(&entries)?)?;
    } else if ctx.csv {
        if !ctx.no_header {
            writeln!(out, "NAME,GROUP,MONTH,UUID,LABEL")?;
        }
        for e in &entries {
            writeln!(
                out,
                "{},{},{},{},{}",
                csv_field(&e.name),
                e.group.map(|g| g.to_string()).unwrap_or_default(),
                e.month,
                e.uuid,
                csv_field(&e.label)
            )?;
        }
    } else {
        let w = entries
            .iter()
            .map(|e| e.name.chars().count())
            .max()
            .unwrap_or(0)
            .max(4);
        if !ctx.no_header {
            writeln!(
                out,
                "{:<w$}  {:<5}  {:<7}  {:<35}  LABEL",
                "NAME", "GROUP", "MONTH", "UUID"
            )?;
        }
        for e in &entries {
            writeln!(
                out,
                "{:<w$}  {:<5}  {:<7}  {:<35}  {}",
                e.name,
                group_label(e.group),
                e.month,
                e.uuid,
                e.label
            )?;
        }
    }
    Ok(())
}

/// Resolve a name or uuid to the index entries of exactly one statistic uuid.
fn resolve<'a>(entries: &'a [StatEntry], query: &str) -> Result<Vec<&'a StatEntry>> {
    let identities: Vec<_> = entries
        .iter()
        .map(|e| StatisticIdentity {
            name: &e.name,
            group: e.group,
            uuid: &e.uuid,
        })
        .collect();
    let uuid = resolve_identity(&identities, query).map_err(|error| match error {
        StatisticResolutionError::NotFound { names } => {
            let names: Vec<String> = names.into_iter().map(str::to_owned).collect();
            crate::errors::not_found_error("Statistic", query, &names, "lox stats ls")
        }
        StatisticResolutionError::Ambiguous {
            distinct_uuids,
            candidates,
        } => {
            let lines: Vec<String> = candidates
                .iter()
                .map(|e| format!("  {}  group {}  {}", e.name, group_label(e.group), e.uuid))
                .collect();
            anyhow::anyhow!(
                "Ambiguous: '{query}' matches {} statistics; pass the uuid instead:\n{}",
                distinct_uuids,
                lines.join("\n")
            )
        }
    })?;
    Ok(entries.iter().filter(|e| e.uuid == uuid).collect())
}

fn get(
    ctx: &RunContext,
    lox: &LoxClient,
    query: &str,
    group: Option<u32>,
    range: &MonthRange,
    out: &mut dyn Write,
) -> Result<()> {
    range.check()?;
    let index = fetch_index(lox)?;
    let matches = resolve(&index, query)?;
    let name = matches[0].name.clone();
    let uuid = matches[0].uuid.clone();

    let mut groups: Vec<Option<u32>> = matches.iter().map(|e| e.group).collect();
    groups.sort();
    groups.dedup();
    let group = match group {
        Some(g) => {
            if !groups.contains(&Some(g)) {
                let available: Vec<String> = groups.iter().map(|g| group_label(*g)).collect();
                bail!(
                    "Statistic '{name}' ({uuid}) has no group {g}. Available groups: {}",
                    available.join(", ")
                );
            }
            Some(g)
        }
        None if groups.len() > 1 => {
            let mut lines = Vec::new();
            for g in &groups {
                let newest = matches
                    .iter()
                    .filter(|e| e.group == *g)
                    .max_by(|a, b| a.month.cmp(&b.month))
                    .expect("group taken from matches");
                let series = fetch_series(lox, newest)?;
                lines.push(format!(
                    "  group {}: {}",
                    group_label(*g),
                    series.outputs.join(", ")
                ));
            }
            bail!(
                "Statistic '{name}' ({uuid}) has {} groups; pass --group <N>:\n{}",
                groups.len(),
                lines.join("\n")
            );
        }
        None => groups[0],
    };

    let mut selected: Vec<&StatEntry> = matches.into_iter().filter(|e| e.group == group).collect();
    selected.sort_by(|a, b| a.month.cmp(&b.month));
    let first = selected
        .first()
        .map(|e| e.month.clone())
        .unwrap_or_default();
    let last = selected.last().map(|e| e.month.clone()).unwrap_or_default();
    selected.retain(|e| range.contains(&e.month));
    if selected.is_empty() {
        bail!("No months of '{name}' in the requested range (available: {first} to {last})");
    }

    let parts = selected
        .iter()
        .map(|e| Ok((e.month.clone(), fetch_series(lox, e)?)))
        .collect::<Result<Vec<_>>>()?;
    let series = merge_months(parts)?;
    write_series(ctx, &series, out)
}

fn write_series(ctx: &RunContext, series: &StatSeries, out: &mut dyn Write) -> Result<()> {
    if ctx.json {
        if series.outputs.iter().collect::<HashSet<_>>().len() != series.outputs.len() {
            bail!(
                "Statistic '{}' has duplicate output labels; use -o csv or -o table to preserve every value",
                series.name
            );
        }
        let rows: Vec<serde_json::Value> = series
            .rows
            .iter()
            .map(|(t, values)| {
                let values: serde_json::Map<String, serde_json::Value> = series
                    .outputs
                    .iter()
                    .cloned()
                    .zip(values.iter().map(|v| serde_json::json!(v)))
                    .collect();
                serde_json::json!({ "t": t, "values": values })
            })
            .collect();
        let doc = serde_json::json!({
            "name": series.name,
            "group": series.group,
            "outputs": series.outputs,
            "rows": rows,
        });
        writeln!(out, "{}", serde_json::to_string_pretty(&doc)?)?;
    } else if ctx.csv {
        if !ctx.no_header {
            let header: Vec<String> = std::iter::once("T".to_string())
                .chain(series.outputs.iter().map(|o| csv_field(o)))
                .collect();
            writeln!(out, "{}", header.join(","))?;
        }
        for (t, values) in &series.rows {
            let values: Vec<String> = values.iter().map(|v| v.to_string()).collect();
            writeln!(out, "{},{}", csv_field(t), values.join(","))?;
        }
    } else {
        let widths: Vec<usize> = series
            .outputs
            .iter()
            .enumerate()
            .map(|(i, o)| {
                series
                    .rows
                    .iter()
                    .map(|(_, v)| v[i].to_string().len())
                    .max()
                    .unwrap_or(0)
                    .max(o.chars().count())
            })
            .collect();
        if !ctx.no_header {
            let mut line = format!("{:<19}", "T");
            for (o, w) in series.outputs.iter().zip(&widths) {
                line += &format!("  {o:>w$}");
            }
            writeln!(out, "{}", line.trim_end())?;
        }
        for (t, values) in &series.rows {
            let mut line = format!("{t:<19}");
            for (v, w) in values.iter().zip(&widths) {
                line += &format!("  {:>w$}", v.to_string());
            }
            writeln!(out, "{line}")?;
        }
    }
    Ok(())
}

/// Map the shared index producer into the CLI's existing serialized record.
pub fn parse_index(html: &str) -> Vec<StatEntry> {
    lox_core::statistics::parse_index(html)
        .into_iter()
        .map(|entry| StatEntry {
            file: entry.file,
            uuid: entry.uuid,
            group: entry.group,
            month: entry.month,
            name: entry.name,
            label: entry.label,
        })
        .collect()
}

/// Preserve the CLI's single-message error chain around the typed merge.
pub fn merge_months(parts: Vec<(String, StatSeries)>) -> Result<StatSeries> {
    lox_core::statistics::merge_months(parts).map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_A: &str = "1a2b3c4d-0001-0002-ffff0123456789ab";
    const UUID_B: &str = "1a2b3c4d-0003-0004-ffffabcdef012345";

    fn identity_entry(uuid: &str, name: &str, group: Option<u32>, month: &str) -> StatEntry {
        StatEntry {
            file: format!("{uuid}.{month}.xml"),
            uuid: uuid.into(),
            group,
            month: month.into(),
            name: name.into(),
            label: name.into(),
        }
    }

    #[test]
    fn identity_characterization_priority_and_original_all_rows() {
        let entries = vec![
            identity_entry("winner", "Renamed", Some(8), "2026-12"),
            identity_entry("other", "winner", None, "2026-11"),
            identity_entry("winner", "Heat", None, "2026-10"),
            identity_entry("third", "Heat Pump", None, "2026-09"),
            identity_entry("winner", "Heat", Some(2), "2026-08"),
            identity_entry("winner", "Heat", Some(2), "2026-08"),
        ];
        for query in ["WINNER", "hEaT", "renam"] {
            let selected = resolve(&entries, query).unwrap();
            assert_eq!(selected.len(), 4);
            for (row, index) in selected.iter().zip([0, 2, 4, 5]) {
                assert!(std::ptr::eq(*row, &entries[index]));
            }
        }
    }

    #[test]
    fn identity_characterization_distinct_uuid_and_exact_ambiguity_chain() {
        let entries = vec![
            identity_entry("z", "Same", Some(2), "2026-12"),
            identity_entry("z", "Same", None, "2026-11"),
            identity_entry("a", "Same", Some(1), "2026-10"),
            identity_entry("z", "Same", Some(2), "2026-09"),
            identity_entry("a", "same", None, "2026-08"),
        ];
        let expected = "Ambiguous: 'sAmE' matches 2 statistics; pass the uuid instead:\n  Same  group -  z\n  Same  group 1  a\n  Same  group 2  z\n  same  group -  a";
        let error = resolve(&entries, "sAmE").unwrap_err();
        assert_eq!(error.to_string(), expected);
        assert_eq!(error.chain().count(), 1);
        let error = error.context("caller context");
        assert_eq!(format!("{error:#}"), format!("caller context: {expected}"));
        assert_eq!(error.chain().count(), 2);
        assert_eq!(resolve(&entries[..2], "").unwrap().len(), 2);
        assert!(resolve(&entries, "").is_err());
    }

    #[test]
    fn identity_characterization_raw_unicode_uuid_case_and_not_found_help() {
        let entries = vec![
            identity_entry("ABC", "Upper", None, "1"),
            identity_entry("lower", "İSTANBUL", None, "2"),
            identity_entry("accent", "Café", None, "3"),
        ];
        assert_eq!(resolve(&entries, "İstanbul").unwrap()[0].uuid, "lower");
        for query in ["ABC", " Cafe\u{301}", "Café ", "Istanbul", "cafe\u{301}"] {
            assert!(resolve(&entries, query).is_err(), "{query}");
        }
        let rows = vec![
            identity_entry("a", "Heat Pump", None, "1"),
            identity_entry("a", "Heat Pump", Some(1), "2"),
            identity_entry("b", "Kitchen", None, "1"),
        ];
        let expected = "Statistic 'Heat Pum' not found. Did you mean: Heat Pump?\n  Available: Heat Pump, Kitchen";
        let error = resolve(&rows, "Heat Pum ").unwrap_err();
        assert_eq!(
            error.to_string(),
            expected.replace("'Heat Pum'", "'Heat Pum '")
        );
        assert_eq!(error.chain().count(), 1);
        let expected = error.to_string();
        assert_eq!(
            format!("{:#}", error.context("reading selection")),
            format!("reading selection: {expected}")
        );
        let names: Vec<_> = (0..21)
            .map(|i| identity_entry(&format!("{i}"), &format!("Name{i:02}"), None, "1"))
            .collect();
        assert_eq!(
            resolve(&names, "zzzzzzzzzz").unwrap_err().to_string(),
            "Statistic 'zzzzzzzzzz' not found.\n  21 options available. Run: lox stats ls"
        );
        assert_eq!(
            resolve(&[], "Original QUERY").unwrap_err().to_string(),
            "Statistic 'Original QUERY' not found.\n  Available: "
        );
    }

    const INDEX: &str = r#"<html><head><title>Statistics</title><link href="styles.css" rel="stylesheet"></head><body><ul>
<li><a href="1a2b3c4d-0001-0002-ffff0123456789ab_2.202609.xml">Heat Pump (Statistic Group: 2) (Energy, Utility Room) 202609</a></li>
<li><a href="1a2b3c4d-0003-0004-ffffabcdef012345.202610.xml">Kitchen (Climate, Kitchen) 202610</a></li>
<li><a href="1a2b3c4d-0003-0004-ffffabcdef012345.202610.xml.bak">Kitchen backup 202610</a></li>
<li><a href="not-a-uuid.202610.xml">Broken (Climate) 202610</a></li>
<li><a href="1a2b3c4d-0003-0004-ffffabcdef012345.2026.xml">Short month 2026</a></li>
<li><a href="1a2b3c4d-0001-0002-ffff0123456789ab_x.202609.xml">Bad group 202609</a></li>
<li><a href="styles.css">styles.css</a></li>
<li><a href="1a2b3c4d-0001-0002-ffff0123456789ab_1.202610.xml">Caf&#xE9; &amp; Bar (Statistic Group: 1) (Energy, Terrace) 202610</a></li>
</ul></body></html>"#;

    // ── parse_index ──────────────────────────────────────────────────────────

    #[test]
    fn index_characterization_preserves_exact_markup_and_entry_boundaries() {
        let file = format!("{UUID_A}.202610.xml");
        let html = format!(
            "<LI><a href=\"{file}\">upper li</a></LI>\
             <li class=\"x\"><a href=\"{file}\">li attribute</a></li>\
             <li><A href=\"{file}\">upper a</A></li>\
             <li><a href='{file}'>single quote</a></li>\
             <li><a class=\"x\" href=\"{file}\">attribute order</a></li>\
             <li><a href=\"{file}\">unfinished\
             <li><a href=\"{file}\">first</a>\
             <li> \n<a href=\"{file}\" ignored=\"yes\">second</a>\
             <li><a href=\"{file}\">first</a>"
        );
        let entries = parse_index(&html);
        assert_eq!(
            entries.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["first", "second", "first"]
        );
        assert_eq!(entries[0], entries[2]);
    }

    #[test]
    fn index_characterization_preserves_filename_limits_without_calendar_validation() {
        let accepted = [
            (format!("{UUID_A}.000000.xml"), None, "0000-00"),
            (format!("{UUID_A}_000.202613.xml"), Some(0), "2026-13"),
            (
                format!("{UUID_A}_4294967295.999999.xml"),
                Some(u32::MAX),
                "9999-99",
            ),
        ];
        let mut html = String::new();
        for (file, _, _) in &accepted {
            html.push_str(&li(file, "accepted"));
        }
        for file in [
            format!("{}.202610.xml", UUID_A.to_uppercase()),
            "1a2b3c4d-0001-0002-ffff-0123456789ab.202610.xml".into(),
            format!("{UUID_A}_4294967296.202610.xml"),
            format!("{UUID_A}_+1.202610.xml"),
            format!("{UUID_A}_-1.202610.xml"),
            format!("{UUID_A}_ 1.202610.xml"),
            format!("{UUID_A}_1 .202610.xml"),
            format!("{UUID_A}_.202610.xml"),
            format!("{UUID_A}.20261.xml"),
            format!("{UUID_A}.2026100.xml"),
            format!("{UUID_A}.２０２６１０.xml"),
            format!("{UUID_A}.202610.XML"),
        ] {
            html.push_str(&li(&file, "rejected"));
        }
        let entries = parse_index(&html);
        assert_eq!(entries.len(), accepted.len());
        for (entry, (file, group, month)) in entries.iter().zip(accepted) {
            assert_eq!(entry.file, file);
            assert_eq!(entry.group, group);
            assert_eq!(entry.month, month);
        }
    }

    #[test]
    fn index_characterization_preserves_raw_entity_fallback_and_suffix_rules() {
        let file = format!("{UUID_A}_2.202610.xml");
        let cases = [
            (
                " \tCaf&#xE9; &amp; 気 (Grid) (Statistic Group: 2) (Room (North)) 202610 \n",
                "Café & 気 (Grid)",
                "Café & 気 (Grid) (Statistic Group: 2) (Room (North))",
            ),
            (
                " A &amp; B &unknown; (Room) 202610 ",
                "A &amp; B &unknown;",
                "A &amp; B &unknown; (Room)",
            ),
            ("Name(Room) 202610", "Name(Room)", "Name(Room)"),
            (
                "Name (unbalanced 202610",
                "Name (unbalanced",
                "Name (unbalanced",
            ),
            ("Name (Room)) 202610", "Name (Room))", "Name (Room))"),
            (
                "Name (Room) 202611",
                "Name (Room) 202611",
                "Name (Room) 202611",
            ),
            (
                "Name (Statistic Group: 3) (Room) 202610",
                "Name (Statistic Group: 3)",
                "Name (Statistic Group: 3) (Room)",
            ),
        ];
        for (raw, name, label) in cases {
            let entries = parse_index(&li(&file, raw));
            assert_eq!(entries[0].name, name, "{raw}");
            assert_eq!(entries[0].label, label, "{raw}");
        }
        let entry = parse_index(&li(&file, "Name (Room) 202610")).remove(0);
        assert_eq!(
            serde_json::to_value(entry).unwrap(),
            serde_json::json!({
                "file": file, "uuid": UUID_A, "group": 2, "month": "2026-10",
                "name": "Name", "label": "Name (Room)"
            })
        );
    }

    #[test]
    fn index_parses_grouped_ungrouped_and_encoded_labels() {
        let entries = parse_index(INDEX);
        assert_eq!(entries.len(), 3, "{entries:#?}");

        assert_eq!(
            entries[0],
            StatEntry {
                file: format!("{UUID_A}_2.202609.xml"),
                uuid: UUID_A.into(),
                group: Some(2),
                month: "2026-09".into(),
                name: "Heat Pump".into(),
                label: "Heat Pump (Statistic Group: 2) (Energy, Utility Room)".into(),
            }
        );
        assert_eq!(
            entries[1],
            StatEntry {
                file: format!("{UUID_B}.202610.xml"),
                uuid: UUID_B.into(),
                group: None,
                month: "2026-10".into(),
                name: "Kitchen".into(),
                label: "Kitchen (Climate, Kitchen)".into(),
            }
        );
        assert_eq!(entries[2].name, "Café & Bar");
        assert_eq!(
            entries[2].label,
            "Café & Bar (Statistic Group: 1) (Energy, Terrace)"
        );
        assert_eq!(entries[2].group, Some(1));
    }

    #[test]
    fn index_skips_stylesheet_and_malformed_hrefs() {
        let entries = parse_index(INDEX);
        assert!(!entries.is_empty());
        assert!(
            entries
                .iter()
                .all(|e| e.file.ends_with(".xml") && !e.file.contains("_x"))
        );
        assert!(!entries.iter().any(|e| e.file == "styles.css"));
        assert!(!entries.iter().any(|e| e.name == "Broken"));
    }

    #[test]
    fn index_preserves_parentheses_in_names_and_room_names() {
        let html = format!(
            "<ul>{}{}{}</ul>",
            li(
                &format!("{UUID_A}_2.202610.xml"),
                "Heat Pump (Grid) (Statistic Group: 2) (Energy, Utility (North)) 202610"
            ),
            li(
                &format!("{UUID_B}.202610.xml"),
                "Kitchen (Wall) (Climate, Kitchen) 202610"
            ),
            li(
                &format!("{UUID_A}_1.202610.xml"),
                "Heat Pump (Grid) (Statistic Group: 1) 202610"
            )
        );
        let names: Vec<_> = parse_index(&html).into_iter().map(|e| e.name).collect();
        assert_eq!(
            names,
            ["Heat Pump (Grid)", "Kitchen (Wall)", "Heat Pump (Grid)"]
        );
    }

    #[test]
    fn index_skips_malformed_entry_and_continues() {
        let html = format!(
            "<ul>\n\
             <li><a href=\"{UUID_B}.202609.xml\">Kitchen (Climate, Kitchen) 202609</a></li>\n\
             <li><a href=\"{UUID_B}.202610.xml\">Kitchen (Climate, Kitchen) 202610</li>\n\
             <li><a href=\"{UUID_B}.202611.xml>Kitchen (Climate, Kitchen) 202611</a></li>\n\
             <li><a href=\"{UUID_A}_1.202610.xml\">Heat Pump (Statistic Group: 1) 202610</a></li>\n\
             </ul>"
        );
        let entries = parse_index(&html);
        let files: Vec<&str> = entries.iter().map(|e| e.file.as_str()).collect();
        assert_eq!(
            files,
            vec![
                format!("{UUID_B}.202609.xml"),
                format!("{UUID_A}_1.202610.xml")
            ]
        );
        assert_eq!(entries[1].label, "Heat Pump (Statistic Group: 1)");
    }

    // ── parse_series ─────────────────────────────────────────────────────────

    #[test]
    fn monthly_decoder_preserves_defaults_entities_and_opaque_row_order() {
        let s = parse_series(
            "\u{feff}text<Wrapper><Statistics Name=\"Caf&#xE9; &amp; 気\" \
             StatsGroup=\" +2 \" NumOutputs=\"3\" Outputs=\" A &amp; B, , discarded, extra\">\
             <S T=\" later &amp; raw \" V3=\"3\" V2=\" 2e1 \" V=\"-0\"/>\
             </Statistics><S T=\"\" V=\"1\" V2=\"2\" V3=\"3\"/>\
             <S T=\"\" V=\"4\" V2=\"5\" V3=\"6\"/></Wrapper>trailing<Other/>",
        )
        .unwrap();
        assert_eq!(s.name, "Café & 気");
        assert_eq!(s.group, Some(2));
        assert_eq!(s.outputs, ["A & B", "V2", "discarded"]);
        assert_eq!(s.rows[0].0, " later & raw ");
        assert_eq!(s.rows[0].1, [-0.0, 20.0, 3.0]);
        assert!(s.rows[0].1[0].is_sign_negative());
        assert_eq!(s.rows[1].0, "");
        assert_eq!(s.rows[2].0, "");
        for (xml, labels, rows) in [
            ("<Statistics/>", vec!["V"], 0),
            ("<Statistics Outputs=\"\"/>", vec!["V"], 0),
            ("<Statistics Outputs=\" A, ,A \"/>", vec!["A", "V2", "A"], 0),
            (
                "<Statistics NumOutputs=\"0\"><S T=\"x\"/></Statistics>",
                vec![],
                1,
            ),
            (
                "<Statistics Name=\"n\" StatsGroup=\"0\" Outputs=\"A\" NumOutputs=\"1\" broken/>",
                vec!["A"],
                0,
            ),
        ] {
            let s = parse_series(xml).unwrap();
            assert_eq!(s.outputs, labels, "{xml}");
            assert_eq!(s.rows.len(), rows, "{xml}");
        }
    }

    #[test]
    fn monthly_decoder_preserves_validation_precedence_and_context() {
        for (xml, expected) in [
            ("", "no <Statistics> element in statistics file"),
            ("not XML", "no <Statistics> element in statistics file"),
            ("<S/>", "statistics row before <Statistics> element"),
            (
                "<Statistics/><Statistics StatsGroup=\"bad\"/>",
                "multiple <Statistics> elements in statistics file",
            ),
            ("<Statistics>", "truncated statistics XML: unclosed element"),
            ("<Statistics><S/>", "statistics row without T attribute"),
            ("<Statistics><S T=\"x\"/>", "row x: missing value V"),
            ("<Statistics StatsGroup=\"-1\"/>", "invalid StatsGroup '-1'"),
            ("<Statistics NumOutputs=\"-1\"/>", "invalid NumOutputs '-1'"),
            ("<Statistics broken/>", "malformed XML attribute"),
            (
                "<Statistics><S T=\"x\" V=\"oops\"/></Statistics>",
                "row x: V='oops' is not a number",
            ),
            (
                "<Statistics><S T=\"x\" V=\"1e9999\"/></Statistics>",
                "row x: V='1e9999' is not a finite number",
            ),
        ] {
            assert_eq!(
                parse_series(xml).unwrap_err().to_string(),
                expected,
                "{xml}"
            );
        }
        for xml in [
            "</Statistics>",
            "<Statistics></Other>",
            "<Statistics Name=\"&unknown;\"/>",
            "<Statistics Name=\"n\" Name=\"duplicate\"/>",
        ] {
            assert!(parse_series(xml).is_err(), "{xml}");
        }
    }

    #[test]
    fn series_with_one_output() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Statistics Name="Kitchen" NumOutputs="1" Outputs="Temperature">
<S T="2026-10-01 00:00:00" V="21.5"/>
<S T="2026-10-01 00:30:00" V="21.25"/>
</Statistics>"#;
        let s = parse_series(xml).unwrap();
        assert_eq!(s.name, "Kitchen");
        assert_eq!(s.group, None);
        assert_eq!(s.outputs, vec!["Temperature"]);
        assert_eq!(
            s.rows,
            vec![
                ("2026-10-01 00:00:00".to_string(), vec![21.5]),
                ("2026-10-01 00:30:00".to_string(), vec![21.25]),
            ]
        );
    }

    #[test]
    fn series_with_two_outputs_in_order() {
        let xml = r#"<Statistics Name="Heat Pump &amp; Boiler" StatsGroup="2" NumOutputs="2" Outputs="Consumed,Delivered">
<S T="2026-10-01 02:00:00" V="100.5" V2="42"/>
<S T="2026-10-01 03:00:00" V2="43.5" V="101"/>
</Statistics>"#;
        let s = parse_series(xml).unwrap();
        assert_eq!(s.name, "Heat Pump & Boiler");
        assert_eq!(s.group, Some(2));
        assert_eq!(s.outputs, vec!["Consumed", "Delivered"]);
        assert_eq!(s.rows[0].1, vec![100.5, 42.0]);
        assert_eq!(s.rows[1].1, vec![101.0, 43.5]);
    }

    #[test]
    fn series_row_missing_output_names_timestamp() {
        let xml = r#"<Statistics Name="Heat Pump" StatsGroup="1" NumOutputs="2" Outputs="Consumed,Delivered">
<S T="2026-10-01 02:00:00" V="1" V2="2"/>
<S T="2026-10-01 03:00:00" V="3"/>
</Statistics>"#;
        let err = parse_series(xml).unwrap_err().to_string();
        assert!(err.contains("2026-10-01 03:00:00"), "{err}");
        assert!(err.contains("V2"), "{err}");
    }

    #[test]
    fn series_non_numeric_value_is_error() {
        let xml = r#"<Statistics Name="Kitchen" NumOutputs="1" Outputs="Temperature">
<S T="2026-10-01 00:00:00" V="warm"/>
</Statistics>"#;
        let err = parse_series(xml).unwrap_err().to_string();
        assert!(err.contains("warm"), "{err}");
        assert!(err.contains("2026-10-01 00:00:00"), "{err}");
    }

    #[test]
    fn series_rejects_truncated_xml() {
        let xml = r#"<Statistics Name="Kitchen" NumOutputs="1" Outputs="Temperature">
<S T="2026-10-01 00:00:00" V="21"/>"#;
        assert!(parse_series(xml).is_err());
    }

    #[test]
    fn series_rejects_non_finite_values_instead_of_exporting_json_null() {
        for value in ["NaN", "inf", "-inf"] {
            let xml = format!(
                r#"<Statistics Name="Kitchen" NumOutputs="1" Outputs="Temperature">
<S T="2026-10-01 00:00:00" V="{value}"/></Statistics>"#
            );
            let err = parse_series(&xml).unwrap_err().to_string();
            assert!(err.contains(value), "{err}");
            assert!(err.contains("2026-10-01 00:00:00"), "{err}");
        }
    }

    #[test]
    fn json_rejects_duplicate_output_labels_without_losing_values() {
        let xml = r#"<Statistics Name="Kitchen" NumOutputs="2" Outputs="Temperature,Temperature">
<S T="2026-10-01 00:00:00" V="21" V2="22"/></Statistics>"#;
        let s = parse_series(xml).unwrap();
        let mut out = Vec::new();
        let err = write_series(&run_ctx(true, false), &s, &mut out)
            .unwrap_err()
            .to_string();
        assert!(err.contains("duplicate output labels"), "{err}");
        assert!(err.contains("-o csv"), "{err}");
        assert!(out.is_empty());
        write_series(&run_ctx(false, true), &s, &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "T,Temperature,Temperature\n2026-10-01 00:00:00,21,22\n"
        );
    }

    // ── merge_months ─────────────────────────────────────────────────────────

    #[test]
    fn merge_empty_and_first_metadata_keep_caller_order_without_validation() {
        let empty = merge_months(vec![]).unwrap();
        assert_eq!(
            empty,
            StatSeries {
                name: String::new(),
                group: None,
                outputs: vec![],
                rows: vec![],
            }
        );
        let mut first = series(&[("z", 1.0), ("a", 2.0)]);
        first.name = "first".into();
        first.group = Some(u32::MAX);
        first.outputs = vec![" raw ".into(), " raw ".into()];
        let mut later = series(&[("between", 3.0)]);
        later.name = "ignored".into();
        later.group = None;
        later.outputs = first.outputs.clone();
        let merged = merge_months(vec![
            ("not-a-month".into(), first.clone()),
            ("0000-00".into(), later.clone()),
            ("0000-00".into(), later),
        ])
        .unwrap();
        assert_eq!(merged.name, first.name);
        assert_eq!(merged.group, first.group);
        assert_eq!(merged.outputs, first.outputs);
        assert_eq!(
            merged.rows,
            vec![
                ("z".into(), vec![1.0]),
                ("a".into(), vec![2.0]),
                ("between".into(), vec![3.0]),
            ]
        );
    }

    #[test]
    fn merge_deduplicates_only_earlier_parts_and_preserves_value_bits() {
        let bits = [
            0x8000_0000_0000_0000,
            0x7ff8_0000_0000_0042,
            0x7ff0_0000_0000_0001,
            0xfff0_0000_0000_0000,
            1,
        ];
        let merged = merge_months(vec![
            (
                "z".into(),
                series(&[
                    ("same", f64::from_bits(bits[0])),
                    ("same", f64::from_bits(bits[1])),
                ]),
            ),
            (
                "a".into(),
                series(&[
                    ("same", 123.0),
                    ("new", f64::from_bits(bits[2])),
                    ("new", f64::from_bits(bits[3])),
                    (" same", f64::from_bits(bits[4])),
                ]),
            ),
            ("a".into(), series(&[("new", 456.0), ("same", 789.0)])),
        ])
        .unwrap();
        assert_eq!(
            merged
                .rows
                .iter()
                .map(|(t, _)| t.as_str())
                .collect::<Vec<_>>(),
            ["same", "same", "new", "new", " same"]
        );
        assert_eq!(
            merged
                .rows
                .iter()
                .map(|(_, v)| v[0].to_bits())
                .collect::<Vec<_>>(),
            bits
        );
    }

    #[test]
    fn merge_mismatch_keeps_exact_message_and_single_cause_after_empty_part() {
        let mut first = series(&[("row", 1.0)]);
        first.outputs = vec!["A".into(), "B".into()];
        let mut empty = first.clone();
        empty.name = "ignored".into();
        empty.rows.clear();
        let mut mismatch = series(&[("row", 2.0)]);
        mismatch.outputs = vec!["B".into(), "A".into()];
        let err = merge_months(vec![
            ("later".into(), first),
            ("empty".into(), empty),
            ("earlier".into(), mismatch),
        ])
        .unwrap_err();
        let expected = "Statistic 'Heat Pump' has different outputs in empty (A, B) and earlier (B, A); use --from/--to to select months with the same outputs";
        assert_eq!(err.to_string(), expected);
        assert_eq!(format!("{err:#}"), expected);
        assert_eq!(
            err.chain().map(ToString::to_string).collect::<Vec<_>>(),
            [expected]
        );
    }

    fn series(rows: &[(&str, f64)]) -> StatSeries {
        StatSeries {
            name: "Heat Pump".into(),
            group: Some(1),
            outputs: vec!["Consumed".into()],
            rows: rows
                .iter()
                .map(|(t, v)| (t.to_string(), vec![*v]))
                .collect(),
        }
    }

    #[test]
    fn merge_drops_boundary_duplicate_and_keeps_month_order() {
        let sep = series(&[("2026-09-30 23:00:00", 1.0), ("2026-10-01 02:00:00", 2.0)]);
        let oct = series(&[("2026-10-01 02:00:00", 2.0), ("2026-10-01 03:00:00", 3.0)]);
        let m = merge_months(vec![("2026-09".into(), sep), ("2026-10".into(), oct)]).unwrap();
        assert_eq!(m.name, "Heat Pump");
        assert_eq!(m.group, Some(1));
        assert_eq!(m.outputs, vec!["Consumed"]);
        let ts: Vec<&str> = m.rows.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(
            ts,
            vec![
                "2026-09-30 23:00:00",
                "2026-10-01 02:00:00",
                "2026-10-01 03:00:00"
            ]
        );
    }

    // ── list / get (httpmock) ────────────────────────────────────────────────

    use crate::client::HttpStatusError;
    use httpmock::prelude::*;

    const HP: &str = "0a0b0c0d-0001-0002-aaaa000000000001";
    const KITCHEN: &str = "0a0b0c0d-0003-0004-bbbb000000000002";
    const OTHER_1: &str = "0a0b0c0d-0005-0006-cccc000000000003";
    const OTHER_2: &str = "0a0b0c0d-0007-0008-dddd000000000004";

    fn li(file: &str, label: &str) -> String {
        format!("<li><a href=\"{file}\">{label}</a></li>\n")
    }

    /// Index: Heat Pump groups 1 (2026-10) and 2 (2026-08..2026-11),
    /// Kitchen ungrouped (2026-09, 2026-10), two different "Spare Circuit" statistics.
    fn mock_index() -> String {
        let mut html = String::from(
            "<html><head><link href=\"styles.css\" rel=\"stylesheet\"></head><body><ul>\n",
        );
        html += &li(
            &format!("{HP}_1.202610.xml"),
            "Heat Pump (Statistic Group: 1) (Energy, Utility Room) 202610",
        );
        for m in ["202608", "202609", "202610", "202611"] {
            html += &li(
                &format!("{HP}_2.{m}.xml"),
                &format!("Heat Pump (Statistic Group: 2) (Energy, Utility Room) {m}"),
            );
        }
        for m in ["202609", "202610"] {
            html += &li(
                &format!("{KITCHEN}.{m}.xml"),
                &format!("Kitchen (Climate, Kitchen) {m}"),
            );
        }
        html += &li(
            &format!("{OTHER_1}.202610.xml"),
            "Spare Circuit (Climate, Attic) 202610",
        );
        html += &li(
            &format!("{OTHER_2}.202610.xml"),
            "Spare Circuit (Energy, Garage) 202610",
        );
        html + "</ul></body></html>"
    }

    fn mock_client(server: &MockServer) -> LoxClient {
        LoxClient::new(Config {
            host: server.base_url(),
            user: "test".into(),
            pass: "test".into(),
            ..Default::default()
        })
        .unwrap()
    }

    fn run_ctx(json: bool, csv: bool) -> RunContext {
        RunContext {
            json,
            quiet: false,
            csv,
            dry_run: false,
            no_header: false,
            non_interactive: true,
            trace_id: None,
            telemetry: None,
        }
    }

    fn serve_index(server: &MockServer) -> httpmock::Mock<'_> {
        let body = mock_index();
        server.mock(move |when, then| {
            when.method(GET).path("/stats/");
            then.status(200).body(body);
        })
    }

    fn serve_file<'a>(server: &'a MockServer, file: &str, body: &str) -> httpmock::Mock<'a> {
        let path = format!("/stats/{file}");
        let body = body.to_string();
        server.mock(move |when, then| {
            when.method(GET).path(path);
            then.status(200).body(body);
        })
    }

    fn heat_pump_xml(group: u32, t: &str) -> String {
        format!(
            "<Statistics Name=\"Heat Pump\" StatsGroup=\"{group}\" NumOutputs=\"2\" \
             Outputs=\"Consumed,Delivered\">\n<S T=\"{t}\" V=\"1\" V2=\"2\"/>\n</Statistics>"
        )
    }

    fn run_get(
        server: &MockServer,
        ctx: &RunContext,
        query: &str,
        group: Option<u32>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<String> {
        let lox = mock_client(server);
        let mut out = Vec::new();
        let range = MonthRange {
            from: from.map(String::from),
            to: to.map(String::from),
        };
        get(ctx, &lox, query, group, &range, &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn get_merges_two_months_as_csv_with_output_header() {
        let server = MockServer::start();
        serve_index(&server);
        serve_file(
            &server,
            &format!("{KITCHEN}.202609.xml"),
            r#"<Statistics Name="Kitchen" NumOutputs="2" Outputs="Temperature,Humidity">
<S T="2026-09-30 23:30:00" V="21.5" V2="48"/>
<S T="2026-10-01 00:00:00" V="21.25" V2="49"/>
</Statistics>"#,
        );
        serve_file(
            &server,
            &format!("{KITCHEN}.202610.xml"),
            r#"<Statistics Name="Kitchen" NumOutputs="2" Outputs="Temperature,Humidity">
<S T="2026-10-01 00:00:00" V="21.25" V2="49"/>
<S T="2026-10-01 00:30:00" V="21" V2="50.5"/>
</Statistics>"#,
        );
        let out = run_get(&server, &run_ctx(false, true), "kitchen", None, None, None).unwrap();
        assert_eq!(
            out,
            "T,Temperature,Humidity\n\
             2026-09-30 23:30:00,21.5,48\n\
             2026-10-01 00:00:00,21.25,49\n\
             2026-10-01 00:30:00,21,50.5\n"
        );
    }

    #[test]
    fn get_monthly_decoder_failure_emits_no_partial_rows_and_keeps_error_chain() {
        let server = MockServer::start();
        let index = serve_index(&server);
        let sep = serve_file(
            &server,
            &format!("{KITCHEN}.202609.xml"),
            &kitchen_xml(&[("first", 21.5)]),
        );
        let oct = serve_file(
            &server,
            &format!("{KITCHEN}.202610.xml"),
            r#"<Statistics><S T="second" V="warm"/></Statistics>"#,
        );
        let mut out = Vec::new();
        let error = get(
            &run_ctx(false, true),
            &mock_client(&server),
            "Kitchen",
            None,
            &MonthRange {
                from: None,
                to: None,
            },
            &mut out,
        )
        .unwrap_err();
        assert!(out.is_empty());
        assert_eq!(
            error.chain().map(ToString::to_string).collect::<Vec<_>>(),
            [
                format!("parsing {KITCHEN}.202610.xml"),
                "row second: V='warm' is not a number".into(),
                "invalid float literal".into(),
            ]
        );
        index.assert_hits(1);
        sep.assert_hits(1);
        oct.assert_hits(1);
    }

    #[test]
    fn get_name_includes_history_before_the_statistic_was_renamed() {
        let server = MockServer::start();
        serve_body(
            &server,
            format!(
                "<ul>{}{}</ul>",
                li(
                    &format!("{KITCHEN}.202609.xml"),
                    "Old Kitchen (Climate, Kitchen) 202609"
                ),
                li(
                    &format!("{KITCHEN}.202610.xml"),
                    "Kitchen (Climate, Kitchen) 202610"
                )
            ),
        );
        serve_file(
            &server,
            &format!("{KITCHEN}.202609.xml"),
            &kitchen_xml(&[("2026-09-30 23:30:00", 21.5)]).replace("Kitchen", "Old Kitchen"),
        );
        serve_file(
            &server,
            &format!("{KITCHEN}.202610.xml"),
            &kitchen_xml(&[("2026-10-01 00:30:00", 21.0)]),
        );
        let by_name = run_get(&server, &run_ctx(false, true), "Kitchen", None, None, None).unwrap();
        let by_uuid = run_get(&server, &run_ctx(false, true), KITCHEN, None, None, None).unwrap();
        assert_eq!(by_name, by_uuid);
        assert_eq!(by_name.lines().count(), 3);
    }

    #[test]
    fn get_fetches_only_months_in_range() {
        let server = MockServer::start();
        serve_index(&server);
        let aug = serve_file(
            &server,
            &format!("{HP}_2.202608.xml"),
            &heat_pump_xml(2, "2026-08-01 02:00:00"),
        );
        let sep = serve_file(
            &server,
            &format!("{HP}_2.202609.xml"),
            &heat_pump_xml(2, "2026-09-01 02:00:00"),
        );
        let oct = serve_file(
            &server,
            &format!("{HP}_2.202610.xml"),
            &heat_pump_xml(2, "2026-10-01 02:00:00"),
        );
        let nov = serve_file(
            &server,
            &format!("{HP}_2.202611.xml"),
            &heat_pump_xml(2, "2026-11-01 02:00:00"),
        );
        let out = run_get(
            &server,
            &run_ctx(true, false),
            "Heat Pump",
            Some(2),
            Some("2026-09"),
            Some("2026-10"),
        )
        .unwrap();
        aug.assert_hits(0);
        sep.assert_hits(1);
        oct.assert_hits(1);
        nov.assert_hits(0);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["name"], "Heat Pump");
        assert_eq!(v["group"], 2);
        assert_eq!(v["outputs"], serde_json::json!(["Consumed", "Delivered"]));
        assert_eq!(v["rows"].as_array().unwrap().len(), 2);
        assert_eq!(v["rows"][0]["t"], "2026-09-01 02:00:00");
        assert_eq!(v["rows"][0]["values"]["Delivered"], 2.0);
    }

    #[test]
    fn get_ambiguous_name_lists_both_uuids() {
        let server = MockServer::start();
        serve_index(&server);
        let err = run_get(
            &server,
            &run_ctx(false, false),
            "spare circuit",
            None,
            None,
            None,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("Ambiguous: 'spare circuit'"), "{err}");
        assert!(err.contains(OTHER_1) && err.contains(OTHER_2), "{err}");
    }

    #[test]
    fn get_grouped_without_group_lists_groups_and_outputs() {
        let server = MockServer::start();
        serve_index(&server);
        serve_file(
            &server,
            &format!("{HP}_1.202610.xml"),
            &heat_pump_xml(1, "2026-10-01 02:00:00").replace("Consumed,Delivered", "Grid,Solar"),
        );
        serve_file(
            &server,
            &format!("{HP}_2.202611.xml"),
            &heat_pump_xml(2, "2026-11-01 02:00:00"),
        );
        let err = run_get(
            &server,
            &run_ctx(false, false),
            "Heat Pump",
            None,
            None,
            None,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("--group"), "{err}");
        assert!(err.contains("group 1: Grid, Solar"), "{err}");
        assert!(err.contains("group 2: Consumed, Delivered"), "{err}");
    }

    #[test]
    fn get_unknown_name_suggests_near_miss() {
        let server = MockServer::start();
        serve_index(&server);
        let err = run_get(&server, &run_ctx(false, false), "Kitchn", None, None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Statistic 'Kitchn' not found"), "{err}");
        assert!(err.contains("Did you mean: Kitchen?"), "{err}");
    }

    fn serve_body<'a>(server: &'a MockServer, body: String) -> httpmock::Mock<'a> {
        server.mock(move |when, then| {
            when.method(GET).path("/stats/");
            then.status(200).body(body);
        })
    }

    fn kitchen_xml(rows: &[(&str, f64)]) -> String {
        let mut xml = String::from(
            "<Statistics Name=\"Kitchen\" NumOutputs=\"1\" Outputs=\"Temperature\">\n",
        );
        for (t, v) in rows {
            xml += &format!("<S T=\"{t}\" V=\"{v}\"/>\n");
        }
        xml + "</Statistics>"
    }

    #[test]
    fn get_index_out_of_order_still_prints_chronologically() {
        let server = MockServer::start();
        serve_body(
            &server,
            format!(
                "<html><body><ul>\n{}{}</ul></body></html>",
                li(
                    &format!("{KITCHEN}.202610.xml"),
                    "Kitchen (Climate, Kitchen) 202610"
                ),
                li(
                    &format!("{KITCHEN}.202609.xml"),
                    "Kitchen (Climate, Kitchen) 202609"
                ),
            ),
        );
        serve_file(
            &server,
            &format!("{KITCHEN}.202609.xml"),
            &kitchen_xml(&[("2026-09-30 23:30:00", 21.5), ("2026-10-01 00:00:00", 21.0)]),
        );
        serve_file(
            &server,
            &format!("{KITCHEN}.202610.xml"),
            &kitchen_xml(&[("2026-10-01 00:00:00", 21.0), ("2026-10-01 00:30:00", 20.5)]),
        );
        let out = run_get(&server, &run_ctx(false, true), "Kitchen", None, None, None).unwrap();
        assert_eq!(
            out,
            "T,Temperature\n\
             2026-09-30 23:30:00,21.5\n\
             2026-10-01 00:00:00,21\n\
             2026-10-01 00:30:00,20.5\n"
        );
    }

    #[test]
    fn get_months_with_different_outputs_is_error() {
        let server = MockServer::start();
        serve_index(&server);
        serve_file(
            &server,
            &format!("{KITCHEN}.202609.xml"),
            &kitchen_xml(&[("2026-09-30 23:30:00", 21.5)]),
        );
        serve_file(
            &server,
            &format!("{KITCHEN}.202610.xml"),
            r#"<Statistics Name="Kitchen" NumOutputs="2" Outputs="Temperature,Humidity">
<S T="2026-10-01 00:30:00" V="21" V2="50.5"/>
</Statistics>"#,
        );
        let err = run_get(&server, &run_ctx(false, true), "Kitchen", None, None, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("'Kitchen'"), "{err}");
        assert!(err.contains("2026-09 (Temperature)"), "{err}");
        assert!(err.contains("2026-10 (Temperature, Humidity)"), "{err}");
        assert!(err.contains("--from/--to"), "{err}");
    }

    #[test]
    fn get_csv_no_header_omits_header() {
        let server = MockServer::start();
        serve_index(&server);
        serve_file(
            &server,
            &format!("{KITCHEN}.202610.xml"),
            &kitchen_xml(&[("2026-10-01 00:00:00", 21.0)]),
        );
        let mut ctx = run_ctx(false, true);
        ctx.no_header = true;
        let out = run_get(
            &server,
            &ctx,
            "Kitchen",
            None,
            Some("2026-10"),
            Some("2026-10"),
        )
        .unwrap();
        assert_eq!(out, "2026-10-01 00:00:00,21\n");
    }

    #[test]
    fn get_from_after_to_is_error_without_requests() {
        let server = MockServer::start();
        let index = serve_index(&server);
        let err = run_get(
            &server,
            &run_ctx(false, false),
            "Kitchen",
            None,
            Some("2026-10"),
            Some("2026-09"),
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("--from 2026-10 is later than --to 2026-09"),
            "{err}"
        );
        index.assert_hits(0);
    }

    #[test]
    fn get_http_401_surfaces_as_http_status_error() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/stats/");
            then.status(401);
        });
        let err =
            run_get(&server, &run_ctx(false, false), "Kitchen", None, None, None).unwrap_err();
        let http = err
            .downcast_ref::<HttpStatusError>()
            .expect("HttpStatusError");
        assert_eq!(http.status, 401);
    }

    fn run_list(
        server: &MockServer,
        filter: Option<&str>,
        month: Option<&str>,
        latest: bool,
    ) -> Vec<StatEntry> {
        let lox = mock_client(server);
        let mut out = Vec::new();
        list(&run_ctx(true, false), &lox, filter, month, latest, &mut out).unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_slice(&out).unwrap();
        v.into_iter()
            .map(|e| StatEntry {
                file: e["file"].as_str().unwrap().into(),
                uuid: e["uuid"].as_str().unwrap().into(),
                group: e["group"].as_u64().map(|g| g as u32),
                month: e["month"].as_str().unwrap().into(),
                name: e["name"].as_str().unwrap().into(),
                label: e["label"].as_str().unwrap().into(),
            })
            .collect()
    }

    #[test]
    fn list_filter_is_case_insensitive_on_label() {
        let server = MockServer::start();
        serve_index(&server);
        let all = run_list(&server, None, None, false);
        assert_eq!(all.len(), 9);
        let garage = run_list(&server, Some("GARAGE"), None, false);
        assert_eq!(garage.len(), 1);
        assert_eq!(garage[0].uuid, OTHER_2);
        let oct = run_list(&server, Some("kitchen"), Some("2026-10"), false);
        assert_eq!(oct.len(), 1);
        assert_eq!(oct[0].file, format!("{KITCHEN}.202610.xml"));
    }

    #[test]
    fn list_table_no_header_omits_header() {
        let server = MockServer::start();
        serve_index(&server);
        let lox = mock_client(&server);
        let mut ctx = run_ctx(false, false);
        let mut with_header = Vec::new();
        list(&ctx, &lox, None, None, false, &mut with_header).unwrap();
        let with_header = String::from_utf8(with_header).unwrap();
        assert!(with_header.starts_with("NAME"), "{with_header}");
        ctx.no_header = true;
        let mut out = Vec::new();
        list(&ctx, &lox, None, None, false, &mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(!out.contains("NAME"), "{out}");
        assert_eq!(out.lines().count(), 9, "{out}");
    }

    #[test]
    fn list_latest_keeps_newest_month_per_uuid_and_group() {
        let server = MockServer::start();
        serve_index(&server);
        let latest = run_list(&server, None, None, true);
        let files: Vec<&str> = latest.iter().map(|e| e.file.as_str()).collect();
        assert_eq!(
            files,
            vec![
                format!("{HP}_1.202610.xml"),
                format!("{HP}_2.202611.xml"),
                format!("{KITCHEN}.202610.xml"),
                format!("{OTHER_1}.202610.xml"),
                format!("{OTHER_2}.202610.xml"),
            ]
        );
    }

    // ── download (httpmock + tempdir) ────────────────────────────────────────

    /// The injected "now" for download tests: mid-October 2026, local time.
    fn now() -> chrono::DateTime<chrono::Local> {
        local(2026, 10, 15, 12)
    }

    fn local(y: i32, m: u32, d: u32, h: u32) -> chrono::DateTime<chrono::Local> {
        use chrono::TimeZone;
        chrono::Local
            .with_ymd_and_hms(y, m, d, h, 0, 0)
            .earliest()
            .unwrap()
    }

    fn set_mtime(path: &std::path::Path, t: chrono::DateTime<chrono::Local>) {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(t.into())
            .unwrap();
    }

    /// Kitchen 2026-09 (past) and 2026-10 (current), Heat Pump group 1 2026-10.
    fn download_index() -> String {
        format!(
            "<html><body><ul>\n{}{}{}</ul></body></html>",
            li(
                &format!("{KITCHEN}.202609.xml"),
                "Kitchen (Climate, Kitchen) 202609"
            ),
            li(
                &format!("{KITCHEN}.202610.xml"),
                "Kitchen (Climate, Kitchen) 202610"
            ),
            li(
                &format!("{HP}_1.202610.xml"),
                "Heat Pump (Statistic Group: 1) (Energy, Utility Room) 202610"
            ),
        )
    }

    struct DownloadMocks<'a> {
        kitchen_sep: httpmock::Mock<'a>,
        kitchen_oct: httpmock::Mock<'a>,
        heat_pump_oct: httpmock::Mock<'a>,
    }

    fn serve_download(server: &MockServer) -> DownloadMocks<'_> {
        let index = download_index();
        server.mock(move |when, then| {
            when.method(GET).path("/stats/");
            then.status(200).body(index);
        });
        DownloadMocks {
            kitchen_sep: serve_file(server, &format!("{KITCHEN}.202609.xml"), "<sep/>\r\n"),
            kitchen_oct: serve_file(server, &format!("{KITCHEN}.202610.xml"), "<oct/>"),
            heat_pump_oct: serve_file(
                server,
                &format!("{HP}_1.202610.xml"),
                &heat_pump_xml(1, "2026-10-01 02:00:00"),
            ),
        }
    }

    fn run_download(
        server: &MockServer,
        dir: &std::path::Path,
        force: bool,
        dry_run: bool,
    ) -> String {
        let lox = mock_client(server);
        let mut ctx = run_ctx(false, false);
        ctx.dry_run = dry_run;
        let opts = DownloadOpts {
            dir: dir.to_path_buf(),
            filter: None,
            range: MonthRange {
                from: None,
                to: None,
            },
            force,
        };
        let mut out = Vec::new();
        download(&ctx, &lox, &opts, now(), &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn download_first_run_writes_all_files() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("stats");
        let out = run_download(&server, &dir, false, false);
        assert!(out.contains("3 downloaded, 0 skipped"), "{out}");
        mocks.kitchen_sep.assert_hits(1);
        mocks.kitchen_oct.assert_hits(1);
        mocks.heat_pump_oct.assert_hits(1);
        assert_eq!(
            std::fs::read(dir.join(format!("{KITCHEN}.202609.xml"))).unwrap(),
            b"<sep/>\r\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join(format!("{HP}_1.202610.xml"))).unwrap(),
            heat_pump_xml(1, "2026-10-01 02:00:00")
        );
    }

    #[test]
    fn download_second_run_refetches_only_current_month() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        run_download(&server, tmp.path(), false, false);
        let out = run_download(&server, tmp.path(), false, false);
        assert!(out.contains("2 downloaded, 1 skipped"), "{out}");
        mocks.kitchen_sep.assert_hits(1);
        mocks.kitchen_oct.assert_hits(2);
        mocks.heat_pump_oct.assert_hits(2);
    }

    #[test]
    fn download_refetches_past_month_saved_before_month_end() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        run_download(&server, tmp.path(), false, false);
        set_mtime(
            &tmp.path().join(format!("{KITCHEN}.202609.xml")),
            local(2026, 9, 30, 12),
        );
        let out = run_download(&server, tmp.path(), false, false);
        assert!(out.contains("3 downloaded, 0 skipped"), "{out}");
        mocks.kitchen_sep.assert_hits(2);
    }

    #[test]
    fn download_refetches_past_month_saved_on_first_day_of_next_month() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        run_download(&server, tmp.path(), false, false);
        set_mtime(
            &tmp.path().join(format!("{KITCHEN}.202609.xml")),
            local(2026, 10, 1, 23),
        );
        let out = run_download(&server, tmp.path(), false, false);
        assert!(out.contains("3 downloaded, 0 skipped"), "{out}");
        mocks.kitchen_sep.assert_hits(2);
    }

    #[test]
    fn download_skips_past_month_saved_after_month_end_plus_one_day() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        run_download(&server, tmp.path(), false, false);
        set_mtime(
            &tmp.path().join(format!("{KITCHEN}.202609.xml")),
            local(2026, 10, 2, 0),
        );
        let out = run_download(&server, tmp.path(), false, false);
        assert!(out.contains("2 downloaded, 1 skipped"), "{out}");
        mocks.kitchen_sep.assert_hits(1);
    }

    #[test]
    fn download_from_after_to_is_error_without_requests() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        let lox = mock_client(&server);
        let opts = DownloadOpts {
            dir: tmp.path().to_path_buf(),
            filter: None,
            range: MonthRange {
                from: Some("2026-10".into()),
                to: Some("2026-09".into()),
            },
            force: false,
        };
        let err = download(&run_ctx(false, false), &lox, &opts, now(), &mut Vec::new())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("--from 2026-10 is later than --to 2026-09"),
            "{err}"
        );
        mocks.kitchen_sep.assert_hits(0);
    }

    #[test]
    fn download_force_refetches_everything() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        run_download(&server, tmp.path(), false, false);
        let out = run_download(&server, tmp.path(), true, false);
        assert!(out.contains("3 downloaded, 0 skipped"), "{out}");
        mocks.kitchen_sep.assert_hits(2);
        mocks.kitchen_oct.assert_hits(2);
        mocks.heat_pump_oct.assert_hits(2);
    }

    #[test]
    fn download_dry_run_makes_no_file_requests() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("stats");
        let out = run_download(&server, &dir, false, true);
        mocks.kitchen_sep.assert_hits(0);
        mocks.kitchen_oct.assert_hits(0);
        mocks.heat_pump_oct.assert_hits(0);
        assert!(!dir.exists(), "dry run must not create the folder");
        assert!(out.contains(&format!("{KITCHEN}.202609.xml")), "{out}");
        assert!(out.contains("3 would be downloaded, 0 skipped"), "{out}");
    }

    #[test]
    fn download_dry_run_keeps_missing_named_context_data_cold() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let unexpected = server.mock(|when, then| {
            when.any_request();
            then.status(400);
        });
        let root = tempfile::Builder::new()
            .prefix(".stats-cold-context-")
            .tempdir_in(env!("CARGO_MANIFEST_DIR"))
            .unwrap();
        let data_dir = root.path().join("contexts").join("cold");
        let host = server.base_url().replace("127.0.0.1", "2130706433");
        assert!(!host.contains("127.0.0.1") && !host.contains("localhost"));
        let lox = LoxClient::new(Config {
            host,
            user: "synthetic".into(),
            pass: "synthetic".into(),
            data_dir: data_dir.clone(),
            context_name: Some("cold".into()),
            ..Default::default()
        })
        .unwrap();
        let mut ctx = run_ctx(true, false);
        ctx.dry_run = true;
        let opts = DownloadOpts {
            dir: root.path().join("stats"),
            filter: Some("Kitchen".into()),
            range: MonthRange {
                from: Some("2026-10".into()),
                to: Some("2026-10".into()),
            },
            force: true,
        };
        let mut out = Vec::new();
        download(&ctx, &lox, &opts, now(), &mut out).unwrap();
        let plan: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            plan["would_download"],
            serde_json::json!([format!("{KITCHEN}.202610.xml")])
        );
        assert!(!data_dir.exists());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        mocks.kitchen_sep.assert_hits(0);
        mocks.kitchen_oct.assert_hits(0);
        mocks.heat_pump_oct.assert_hits(0);
        unexpected.assert_hits(0);
    }

    #[test]
    fn download_csv_reports_files_and_skips_without_human_summary() {
        let server = MockServer::start();
        let mocks = serve_download(&server);
        let tmp = tempfile::tempdir().unwrap();
        run_download(&server, tmp.path(), false, false);
        let lox = mock_client(&server);
        let mut ctx = run_ctx(false, true);
        ctx.dry_run = true;
        let opts = DownloadOpts {
            dir: tmp.path().to_path_buf(),
            filter: Some("Kitchen".into()),
            range: MonthRange {
                from: Some("2026-09".into()),
                to: Some("2026-10".into()),
            },
            force: false,
        };
        let mut out = Vec::new();
        download(&ctx, &lox, &opts, now(), &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!(
                "FILE,STATUS\n{KITCHEN}.202610.xml,would_download\n{KITCHEN}.202609.xml,skipped\n"
            )
        );
        mocks.kitchen_oct.assert_hits(1);
        ctx.dry_run = false;
        ctx.no_header = true;
        ctx.quiet = true;
        let mut out = Vec::new();
        download(&ctx, &lox, &opts, now(), &mut out).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!("{KITCHEN}.202610.xml,downloaded\n{KITCHEN}.202609.xml,skipped\n")
        );
        mocks.kitchen_sep.assert_hits(1);
        mocks.kitchen_oct.assert_hits(2);
    }
}
