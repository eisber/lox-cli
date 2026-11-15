//! Statistics HTML index/XML decoding and caller-ordered series concatenation.
//!
//! This is not a decoder for binary statistics or an epoch/timezone converter.

use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use std::collections::HashSet;
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

/// One file listed in the `/stats/` index, in encounter order.
#[derive(Debug, Clone, PartialEq)]
pub struct StatEntry {
    /// File name as served, e.g. `0a1b2c3d-4e5f-6a7b-8c9d0e1f2a3b4c5d_2.202610.xml`.
    pub file: String,
    /// Lowercase Loxone `8-4-4-16` UUID.
    pub uuid: String,
    pub group: Option<u32>,
    /// Six filename digits with a hyphen inserted: `YYYY-MM`, not calendar-validated.
    pub month: String,
    /// Label with the trailing room and matching statistic group parts stripped.
    pub name: String,
    /// Trimmed label without the matching trailing `YYYYMM`.
    pub label: String,
}

/// Parse the Miniserver's existing `/stats/` HTML index grammar without I/O.
///
/// Matching is case-sensitive: exact `<li>` followed by optional whitespace and
/// `<a href="`. Other anchor attributes after the href and absent `</li>` are
/// accepted. A malformed entry is skipped, resynchronizing at the next `<li>`.
/// Only lowercase native UUID filenames, optional decimal `u32` groups and six
/// ASCII month digits are accepted; no calendar validation is performed.
///
/// Labels are XML-entity-decoded, but any unescape error intentionally retains
/// the ORIGINAL raw label (including otherwise valid entities). This permissive
/// fallback is not an XML/HTML validation policy. Label trimming and balanced
/// parenthesis suffix stripping preserve the existing CLI behavior.
/// Duplicates and encounter order are retained; malformed entries are omitted.
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

/// One statistics series, with row and output ordering preserved.
#[derive(Debug, Clone, PartialEq)]
pub struct StatSeries {
    pub name: String,
    pub group: Option<u32>,
    /// Ordered labels; the XML parser decodes/trims them and fills empty labels.
    pub outputs: Vec<String>,
    /// Opaque timestamps and values in output order; only XML parsing requires finite values.
    pub rows: Vec<(String, Vec<f64>)>,
}

/// Exact output-label mismatch between the first series and a later part.
///
/// `previous_month` identifies the immediately preceding part, even if empty;
/// `name` and `previous_outputs` still come from the first series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeMonthsError {
    pub name: String,
    pub previous_month: String,
    pub month: String,
    pub previous_outputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl fmt::Display for MergeMonthsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Statistic '{}' has different outputs in {} ({}) and {} ({}); \
             use --from/--to to select months with the same outputs",
            self.name,
            self.previous_month,
            self.previous_outputs.join(", "),
            self.month,
            self.outputs.join(", ")
        )
    }
}

impl std::error::Error for MergeMonthsError {}

/// Concatenate `(month, series)` parts in caller order, without sorting.
///
/// Empty input returns an empty series. The first part's metadata and rows are
/// retained. Exact output-vector equality is checked before each later batch.
/// Timestamps are opaque: only rows matching timestamps from earlier parts are
/// dropped. Duplicates within the first part or any later batch all survive,
/// because the seen set is extended only after collecting that entire batch.
///
/// Months, names/groups, row widths and values are not validated or converted;
/// repeated/unsorted months, arbitrary timestamps and every f64 bit pattern are
/// preserved. This compatibility operation does not enforce the XML parser's
/// finite-value policy.
pub fn merge_months(
    parts: Vec<(String, StatSeries)>,
) -> std::result::Result<StatSeries, MergeMonthsError> {
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
            return Err(MergeMonthsError {
                name: merged.name,
                previous_month: prev_month,
                month,
                previous_outputs: merged.outputs,
                outputs: part.outputs,
            });
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

#[derive(Debug)]
pub enum StatisticsError {
    Xml {
        source: quick_xml::Error,
    },
    MalformedAttribute {
        attribute: String,
        source: AttrError,
    },
    AttributeValue {
        attribute: String,
        source: quick_xml::Error,
    },
    UnexpectedClosingElement,
    TruncatedXml,
    MultipleStatistics,
    MissingStatistics,
    RowBeforeStatistics,
    InvalidStatsGroup {
        value: String,
        source: ParseIntError,
    },
    InvalidNumOutputs {
        value: String,
        source: ParseIntError,
    },
    MissingTimestamp,
    MissingValue {
        timestamp: String,
        attribute: String,
    },
    InvalidValue {
        timestamp: String,
        attribute: String,
        value: String,
        source: ParseFloatError,
    },
    NonFiniteValue {
        timestamp: String,
        attribute: String,
        value: String,
    },
}

impl fmt::Display for StatisticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { .. } => f.write_str("parsing statistics XML"),
            Self::MalformedAttribute { .. } => f.write_str("malformed XML attribute"),
            Self::AttributeValue { source, .. } => source.fmt(f),
            Self::UnexpectedClosingElement => {
                f.write_str("unexpected closing element in statistics XML")
            }
            Self::TruncatedXml => f.write_str("truncated statistics XML: unclosed element"),
            Self::MultipleStatistics => {
                f.write_str("multiple <Statistics> elements in statistics file")
            }
            Self::MissingStatistics => f.write_str("no <Statistics> element in statistics file"),
            Self::RowBeforeStatistics => f.write_str("statistics row before <Statistics> element"),
            Self::InvalidStatsGroup { value, .. } => write!(f, "invalid StatsGroup '{value}'"),
            Self::InvalidNumOutputs { value, .. } => write!(f, "invalid NumOutputs '{value}'"),
            Self::MissingTimestamp => f.write_str("statistics row without T attribute"),
            Self::MissingValue {
                timestamp,
                attribute,
            } => write!(f, "row {timestamp}: missing value {attribute}"),
            Self::InvalidValue {
                timestamp,
                attribute,
                value,
                ..
            } => write!(f, "row {timestamp}: {attribute}='{value}' is not a number"),
            Self::NonFiniteValue {
                timestamp,
                attribute,
                value,
            } => write!(
                f,
                "row {timestamp}: {attribute}='{value}' is not a finite number"
            ),
        }
    }
}

impl std::error::Error for StatisticsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source } | Self::AttributeValue { source, .. } => Some(source),
            Self::MalformedAttribute { source, .. } => Some(source),
            Self::InvalidStatsGroup { source, .. } | Self::InvalidNumOutputs { source, .. } => {
                Some(source)
            }
            Self::InvalidValue { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, StatisticsError>;

fn attr(e: &BytesStart<'_>, name: &str) -> Result<Option<String>> {
    for a in e.attributes() {
        let a = a.map_err(|source| StatisticsError::MalformedAttribute {
            attribute: name.into(),
            source,
        })?;
        if a.key.as_ref() == name.as_bytes() {
            return a
                .unescape_value()
                .map(|value| Some(value.into_owned()))
                .map_err(|source| StatisticsError::AttributeValue {
                    attribute: name.into(),
                    source,
                });
        }
    }
    Ok(None)
}

/// Parse one monthly statistics XML response without I/O or partial results.
///
/// The borrowed UTF-8 string preserves the existing HTTP-text caller's policy;
/// no byte decoding, calendar validation, timestamp conversion or XML repair is
/// performed. Attributes are entity-decoded, unlike configuration room inventory.
/// Unclosed/mismatched elements, duplicate statistics headers and missing/invalid
/// row values are errors. Unknown elements/text are ignored, and rows after a
/// closed header element remain accepted, matching the original route decoder.
///
/// `NumOutputs` is a `usize` without an added resource cap; zero is accepted.
/// Missing count defaults to the label count (at least one). Labels are resized
/// to that count. Values use `V`, `V2`, ... regardless of attribute order.
/// Timestamps, row order and duplicate timestamps/labels remain unchanged.
pub fn parse_series(xml: &str) -> Result<StatSeries> {
    let mut reader = Reader::from_str(xml);
    let mut series: Option<StatSeries> = None;
    let mut depth = 0usize;
    let mut value_attrs: Vec<String> = Vec::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|source| StatisticsError::Xml { source })?;
        match &event {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(StatisticsError::UnexpectedClosingElement)?;
            }
            Event::Eof if depth != 0 => return Err(StatisticsError::TruncatedXml),
            _ => {}
        }
        match event {
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                b"Statistics" => {
                    if series.is_some() {
                        return Err(StatisticsError::MultipleStatistics);
                    }
                    let name = attr(&e, "Name")?.unwrap_or_default();
                    let group = match attr(&e, "StatsGroup")? {
                        Some(g) => Some(g.trim().parse::<u32>().map_err(|source| {
                            StatisticsError::InvalidStatsGroup { value: g, source }
                        })?),
                        None => None,
                    };
                    let mut outputs: Vec<String> = attr(&e, "Outputs")?
                        .map(|o| o.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_default();
                    let num = match attr(&e, "NumOutputs")? {
                        Some(n) => n.trim().parse::<usize>().map_err(|source| {
                            StatisticsError::InvalidNumOutputs { value: n, source }
                        })?,
                        None => outputs.len().max(1),
                    };
                    value_attrs = (1..=num)
                        .map(|i| if i == 1 { "V".into() } else { format!("V{i}") })
                        .collect();
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
                        return Err(StatisticsError::RowBeforeStatistics);
                    };
                    let t = attr(&e, "T")?.ok_or(StatisticsError::MissingTimestamp)?;
                    let mut values = Vec::with_capacity(value_attrs.len());
                    for a in &value_attrs {
                        let raw = attr(&e, a)?.ok_or_else(|| StatisticsError::MissingValue {
                            timestamp: t.clone(),
                            attribute: a.clone(),
                        })?;
                        let v = raw.trim().parse::<f64>().map_err(|source| {
                            StatisticsError::InvalidValue {
                                timestamp: t.clone(),
                                attribute: a.clone(),
                                value: raw.clone(),
                                source,
                            }
                        })?;
                        if !v.is_finite() {
                            return Err(StatisticsError::NonFiniteValue {
                                timestamp: t,
                                attribute: a.clone(),
                                value: raw,
                            });
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
    series.ok_or(StatisticsError::MissingStatistics)
}
