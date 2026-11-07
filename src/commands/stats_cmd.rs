//! `lox stats` — read the statistics the Miniserver logs to its SD card.
//!
//! The Miniserver serves an HTML index at `/stats/` with one entry per
//! statistic and month, and each month as a self-describing XML file
//! (`/stats/<uuid>[_<group>].<YYYYMM>.xml`). Everything here is read-only.

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
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

/// The logged values of one statistic file (or several merged months).
#[derive(Debug, Clone, PartialEq)]
pub struct StatSeries {
    pub name: String,
    pub group: Option<u32>,
    /// Output labels, from `Outputs="a,b"`
    pub outputs: Vec<String>,
    /// `(timestamp, values in output order)`
    pub rows: Vec<(String, Vec<f64>)>,
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
    parse_series(&xml).with_context(|| format!("parsing {}", entry.file))
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
    let q = query.to_lowercase();
    let mut matches: Vec<&StatEntry> = entries.iter().filter(|e| e.uuid == q).collect();
    if matches.is_empty() {
        matches = entries
            .iter()
            .filter(|e| e.name.to_lowercase() == q)
            .collect();
    }
    if matches.is_empty() {
        matches = entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&q))
            .collect();
    }
    if matches.is_empty() {
        let mut names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
        names.sort();
        names.dedup();
        return Err(crate::errors::not_found_error(
            "Statistic",
            query,
            &names,
            "lox stats ls",
        ));
    }
    let uuids: HashSet<&str> = matches.iter().map(|e| e.uuid.as_str()).collect();
    if uuids.len() > 1 {
        let mut candidates: Vec<(&str, Option<u32>, &str)> = matches
            .iter()
            .map(|e| (e.name.as_str(), e.group, e.uuid.as_str()))
            .collect();
        candidates.sort();
        candidates.dedup();
        let lines: Vec<String> = candidates
            .iter()
            .map(|(name, group, uuid)| format!("  {name}  group {}  {uuid}", group_label(*group)))
            .collect();
        bail!(
            "Ambiguous: '{query}' matches {} statistics; pass the uuid instead:\n{}",
            uuids.len(),
            lines.join("\n")
        );
    }
    let uuid = matches[0].uuid.as_str();
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

/// Split a stats file name into `(uuid, group, "YYYY-MM")`.
///
/// Accepts `<uuid>.<YYYYMM>.xml` and `<uuid>_<group>.<YYYYMM>.xml`, where the
/// uuid is the Loxone form `8-4-4-16` lowercase hex. Parsed by hand so no
/// regex dependency is needed.
fn parse_file_name(file: &str) -> Option<(String, Option<u32>, String)> {
    let stem = file.strip_suffix(".xml")?;
    let (stem, month) = stem.rsplit_once('.')?;
    if month.len() != 6 || !month.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (uuid, group) = match stem.split_once('_') {
        Some((uuid, group)) => {
            if group.is_empty() || !group.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            (uuid, Some(group.parse::<u32>().ok()?))
        }
        None => (stem, None),
    };
    let parts: Vec<&str> = uuid.split('-').collect();
    let lens: Vec<usize> = parts.iter().map(|p| p.len()).collect();
    if lens != [8, 4, 4, 16]
        || !parts
            .iter()
            .all(|p| p.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
    {
        return None;
    }
    Some((
        uuid.to_string(),
        group,
        format!("{}-{}", &month[..4], &month[4..]),
    ))
}

/// Parse the HTML index served at `/stats/`.
///
/// Only `<li><a href="...">label</a></li>` entries whose href is a stats file
/// name are returned; anything else (stylesheet, foreign files) is skipped.
pub fn parse_index(html: &str) -> Vec<StatEntry> {
    let mut entries = Vec::new();
    let mut rest = html;
    while let Some(pos) = rest.find("<li>") {
        rest = &rest[pos + 4..];
        // Parse within this entry only, so a malformed entry is skipped
        // without consuming the next one.
        let item = rest.find("<li>").map_or(rest, |end| &rest[..end]);
        let Some(after) = item.trim_start().strip_prefix("<a href=\"") else {
            continue;
        };
        let Some((href, after)) = after.split_once('"') else {
            continue;
        };
        let Some(gt) = after.find('>') else {
            continue;
        };
        let after = &after[gt + 1..];
        let Some((raw_label, _)) = after.split_once("</a>") else {
            continue;
        };
        let Some((uuid, group, month)) = parse_file_name(href) else {
            continue;
        };
        let decoded = unescape(raw_label)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| raw_label.to_string());
        let yyyymm = month.replace('-', "");
        let label = decoded
            .trim()
            .strip_suffix(yyyymm.as_str())
            .unwrap_or(decoded.trim())
            .trim_end()
            .to_string();
        let name = strip_label_suffix(&label);
        let name = group
            .and_then(|g| name.strip_suffix(&format!(" (Statistic Group: {g})")))
            .unwrap_or(name)
            .trim()
            .to_string();
        entries.push(StatEntry {
            file: href.to_string(),
            uuid,
            group,
            month,
            name,
            label,
        });
    }
    entries
}

fn strip_label_suffix(label: &str) -> &str {
    if !label.ends_with(')') {
        return label;
    }
    let mut depth = 0usize;
    for (i, c) in label.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' => {
                let Some(next) = depth.checked_sub(1) else {
                    return label;
                };
                depth = next;
                if depth == 0 {
                    return label[..i].strip_suffix(' ').unwrap_or(label);
                }
            }
            _ => {}
        }
    }
    label
}

/// Read an attribute as an entity-decoded string.
fn attr(e: &BytesStart, name: &[u8]) -> Result<Option<String>> {
    for a in e.attributes() {
        let a = a.context("malformed XML attribute")?;
        if a.key.as_ref() == name {
            return Ok(Some(a.unescape_value()?.into_owned()));
        }
    }
    Ok(None)
}

/// Parse one statistics XML file (`<Statistics ...><S T=".." V=".." V2=".."/>`).
pub fn parse_series(xml: &str) -> Result<StatSeries> {
    let mut reader = Reader::from_str(xml);
    let mut series: Option<StatSeries> = None;
    let mut depth = 0usize;
    // Attribute names holding the values, in output order: V, V2, V3, ...
    let mut value_attrs: Vec<String> = Vec::new();
    loop {
        let event = reader.read_event().context("parsing statistics XML")?;
        match &event {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth = depth
                    .checked_sub(1)
                    .context("unexpected closing element in statistics XML")?;
            }
            Event::Eof if depth != 0 => bail!("truncated statistics XML: unclosed element"),
            _ => {}
        }
        match event {
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                b"Statistics" => {
                    if series.is_some() {
                        bail!("multiple <Statistics> elements in statistics file");
                    }
                    let name = attr(&e, b"Name")?.unwrap_or_default();
                    let group = match attr(&e, b"StatsGroup")? {
                        Some(g) => Some(
                            g.trim()
                                .parse::<u32>()
                                .with_context(|| format!("invalid StatsGroup '{g}'"))?,
                        ),
                        None => None,
                    };
                    let mut outputs: Vec<String> = attr(&e, b"Outputs")?
                        .map(|o| o.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_default();
                    let num = match attr(&e, b"NumOutputs")? {
                        Some(n) => n
                            .trim()
                            .parse::<usize>()
                            .with_context(|| format!("invalid NumOutputs '{n}'"))?,
                        None => outputs.len().max(1),
                    };
                    value_attrs = (1..=num)
                        .map(|i| if i == 1 { "V".into() } else { format!("V{i}") })
                        .collect();
                    // Fall back to the attribute name for unlabelled outputs.
                    outputs.resize(num, String::new());
                    for (o, a) in outputs.iter_mut().zip(&value_attrs) {
                        if o.is_empty() {
                            *o = a.clone();
                        }
                    }
                    series = Some(StatSeries {
                        name,
                        group,
                        outputs,
                        rows: Vec::new(),
                    });
                }
                b"S" => {
                    let Some(s) = series.as_mut() else {
                        bail!("statistics row before <Statistics> element");
                    };
                    let t = attr(&e, b"T")?.context("statistics row without T attribute")?;
                    let mut values = Vec::with_capacity(value_attrs.len());
                    for a in &value_attrs {
                        let raw = attr(&e, a.as_bytes())?
                            .with_context(|| format!("row {t}: missing value {a}"))?;
                        let v = raw
                            .trim()
                            .parse::<f64>()
                            .with_context(|| format!("row {t}: {a}='{raw}' is not a number"))?;
                        if !v.is_finite() {
                            bail!("row {t}: {a}='{raw}' is not a finite number");
                        }
                        values.push(v);
                    }
                    s.rows.push((t, values));
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    series.context("no <Statistics> element in statistics file")
}

/// Concatenate month files (in month order) into one series.
///
/// Each part is `(month, series)`. A month file can end with the first
/// timestamp of the next month; rows whose timestamp was already emitted by an
/// earlier month are dropped (P5). Months whose outputs differ cannot be
/// combined into one table and are an error.
pub fn merge_months(parts: Vec<(String, StatSeries)>) -> Result<StatSeries> {
    let mut parts = parts.into_iter();
    let Some((mut prev_month, mut merged)) = parts.next() else {
        return Ok(StatSeries {
            name: String::new(),
            group: None,
            outputs: Vec::new(),
            rows: Vec::new(),
        });
    };
    let mut seen: HashSet<String> = merged.rows.iter().map(|(t, _)| t.clone()).collect();
    for (month, part) in parts {
        if part.outputs != merged.outputs {
            bail!(
                "Statistic '{}' has different outputs in {prev_month} ({}) and {month} ({}); \
                 use --from/--to to select months with the same outputs",
                merged.name,
                merged.outputs.join(", "),
                part.outputs.join(", ")
            );
        }
        prev_month = month;
        let fresh: Vec<_> = part
            .rows
            .into_iter()
            .filter(|(t, _)| !seen.contains(t))
            .collect();
        seen.extend(fresh.iter().map(|(t, _)| t.clone()));
        merged.rows.extend(fresh);
    }
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_A: &str = "1a2b3c4d-0001-0002-ffff0123456789ab";
    const UUID_B: &str = "1a2b3c4d-0003-0004-ffffabcdef012345";

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
