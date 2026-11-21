//! Decoder for the monthly XML files served at `/stats/<file>.xml`.
//!
//! This is not a decoder for binary statistics or an epoch/timezone converter.

use quick_xml::Reader;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use std::fmt;
use std::num::{ParseFloatError, ParseIntError};

/// One monthly file, with row and output ordering preserved.
#[derive(Debug, Clone, PartialEq)]
pub struct StatSeries {
    pub name: String,
    pub group: Option<u32>,
    /// Entity-decoded, trimmed labels, with `V`, `V2`, ... for empty labels.
    pub outputs: Vec<String>,
    /// Entity-decoded, otherwise opaque timestamp and finite values in output order.
    pub rows: Vec<(String, Vec<f64>)>,
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
