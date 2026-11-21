//! Owned statistics series; XML parsing and caller-ordered merging remain in Core.

use lox_core::statistics as core;

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct StatRow {
    /// Opaque Core text, without epoch, timezone or calendar interpretation.
    pub timestamp: String,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct StatSeries {
    pub name: String,
    pub group: Option<u32>,
    pub outputs: Vec<String>,
    pub rows: Vec<StatRow>,
}

impl From<core::StatSeries> for StatSeries {
    fn from(value: core::StatSeries) -> Self {
        Self {
            name: value.name,
            group: value.group,
            outputs: value.outputs,
            rows: value
                .rows
                .into_iter()
                .map(|(timestamp, values)| StatRow { timestamp, values })
                .collect(),
        }
    }
}

impl From<StatSeries> for core::StatSeries {
    fn from(value: StatSeries) -> Self {
        Self {
            name: value.name,
            group: value.group,
            outputs: value.outputs,
            rows: value
                .rows
                .into_iter()
                .map(|StatRow { timestamp, values }| (timestamp, values))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct StatMonthPart {
    pub month: String,
    pub series: StatSeries,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum StatisticsMergeError {
    #[error(
        "Statistic '{name}' has different outputs in {previous_month} ({previous}) and {month} ({current}); \
         use --from/--to to select months with the same outputs",
        previous = .previous_outputs.join(", "),
        current = .outputs.join(", ")
    )]
    OutputMismatch {
        name: String,
        previous_month: String,
        month: String,
        previous_outputs: Vec<String>,
        outputs: Vec<String>,
    },
}

impl From<core::MergeMonthsError> for StatisticsMergeError {
    fn from(error: core::MergeMonthsError) -> Self {
        let core::MergeMonthsError {
            name,
            previous_month,
            month,
            previous_outputs,
            outputs,
        } = error;
        Self::OutputMismatch {
            name,
            previous_month,
            month,
            previous_outputs,
            outputs,
        }
    }
}

/// Merge already supplied series in caller order, without parsing or choosing months.
#[uniffi::export]
pub fn merge_statistics_months(
    parts: Vec<StatMonthPart>,
) -> Result<StatSeries, StatisticsMergeError> {
    core::merge_months(
        parts
            .into_iter()
            .map(|part| (part.month, part.series.into()))
            .collect(),
    )
    .map(Into::into)
    .map_err(Into::into)
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum StatisticsError {
    #[error("parsing statistics XML")]
    Xml { source_message: String },
    #[error("malformed XML attribute")]
    MalformedAttribute {
        attribute: String,
        source_message: String,
    },
    #[error("{source_message}")]
    AttributeValue {
        attribute: String,
        source_message: String,
    },
    #[error("unexpected closing element in statistics XML")]
    UnexpectedClosingElement,
    #[error("truncated statistics XML: unclosed element")]
    TruncatedXml,
    #[error("multiple <Statistics> elements in statistics file")]
    MultipleStatistics,
    #[error("no <Statistics> element in statistics file")]
    MissingStatistics,
    #[error("statistics row before <Statistics> element")]
    RowBeforeStatistics,
    #[error("invalid StatsGroup '{value}'")]
    InvalidStatsGroup {
        value: String,
        source_message: String,
    },
    #[error("invalid NumOutputs '{value}'")]
    InvalidNumOutputs {
        value: String,
        source_message: String,
    },
    #[error("statistics row without T attribute")]
    MissingTimestamp,
    #[error("row {timestamp}: missing value {attribute}")]
    MissingValue {
        timestamp: String,
        attribute: String,
    },
    #[error("row {timestamp}: {attribute}='{value}' is not a number")]
    InvalidValue {
        timestamp: String,
        attribute: String,
        value: String,
        source_message: String,
    },
    #[error("row {timestamp}: {attribute}='{value}' is not a finite number")]
    NonFiniteValue {
        timestamp: String,
        attribute: String,
        value: String,
    },
}

impl From<core::StatisticsError> for StatisticsError {
    fn from(error: core::StatisticsError) -> Self {
        match error {
            core::StatisticsError::Xml { source } => Self::Xml {
                source_message: source.to_string(),
            },
            core::StatisticsError::MalformedAttribute { attribute, source } => {
                Self::MalformedAttribute {
                    attribute,
                    source_message: source.to_string(),
                }
            }
            core::StatisticsError::AttributeValue { attribute, source } => Self::AttributeValue {
                attribute,
                source_message: source.to_string(),
            },
            core::StatisticsError::UnexpectedClosingElement => Self::UnexpectedClosingElement,
            core::StatisticsError::TruncatedXml => Self::TruncatedXml,
            core::StatisticsError::MultipleStatistics => Self::MultipleStatistics,
            core::StatisticsError::MissingStatistics => Self::MissingStatistics,
            core::StatisticsError::RowBeforeStatistics => Self::RowBeforeStatistics,
            core::StatisticsError::InvalidStatsGroup { value, source } => Self::InvalidStatsGroup {
                value,
                source_message: source.to_string(),
            },
            core::StatisticsError::InvalidNumOutputs { value, source } => Self::InvalidNumOutputs {
                value,
                source_message: source.to_string(),
            },
            core::StatisticsError::MissingTimestamp => Self::MissingTimestamp,
            core::StatisticsError::MissingValue {
                timestamp,
                attribute,
            } => Self::MissingValue {
                timestamp,
                attribute,
            },
            core::StatisticsError::InvalidValue {
                timestamp,
                attribute,
                value,
                source,
            } => Self::InvalidValue {
                timestamp,
                attribute,
                value,
                source_message: source.to_string(),
            },
            core::StatisticsError::NonFiniteValue {
                timestamp,
                attribute,
                value,
            } => Self::NonFiniteValue {
                timestamp,
                attribute,
                value,
            },
        }
    }
}

/// Parse one acquired monthly XML string atomically, without I/O or time conversion.
#[uniffi::export]
pub fn parse_statistics_series(xml: String) -> Result<StatSeries, StatisticsError> {
    core::parse_series(&xml).map(Into::into).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn month_part(
        month: &str,
        name: &str,
        group: Option<u32>,
        outputs: &[&str],
        rows: &[(&str, &[f64])],
    ) -> StatMonthPart {
        StatMonthPart {
            month: month.into(),
            series: StatSeries {
                name: name.into(),
                group,
                outputs: outputs.iter().map(|value| (*value).into()).collect(),
                rows: rows
                    .iter()
                    .map(|(timestamp, values)| StatRow {
                        timestamp: (*timestamp).into(),
                        values: values.to_vec(),
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn merge_retains_caller_order_batch_duplicates_and_opaque_float_bits() {
        let bits = [
            0x8000_0000_0000_0000,
            1,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_1234,
            0xfff0_0000_0000_5678,
        ];
        let values = bits.map(f64::from_bits);
        let labels = [" watts ", "", " watts "];
        let opaque = " \0e\u{301}\r\n ";
        let merged = merge_statistics_months(vec![
            month_part(
                "z\0",
                "first \0",
                Some(u32::MAX),
                &labels,
                &[(opaque, &values), (opaque, &[])],
            ),
            month_part(
                "",
                "ignored",
                Some(0),
                &labels,
                &[(opaque, &[99.0]), ("", &[2.0]), ("", &[3.0])],
            ),
            month_part(
                "z\0",
                "",
                None,
                &labels,
                &[("", &[99.0]), ("a", &[4.0, 5.0])],
            ),
        ])
        .unwrap();
        assert_eq!(merged.name, "first \0");
        assert_eq!(merged.group, Some(u32::MAX));
        assert_eq!(merged.outputs, labels);
        assert_eq!(
            merged
                .rows
                .iter()
                .map(|row| row.timestamp.as_str())
                .collect::<Vec<_>>(),
            [opaque, opaque, "", "", "a"]
        );
        assert_eq!(
            merged.rows[0]
                .values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            bits
        );
        assert!(merged.rows[1].values.is_empty());
        assert_eq!(merged.rows[2].values, [2.0]);
        assert_eq!(merged.rows[3].values, [3.0]);
        assert_eq!(merged.rows[4].values, [4.0, 5.0]);
        assert_eq!(
            merge_statistics_months(vec![]).unwrap(),
            month_part("", "", None, &[], &[]).series
        );
    }

    #[test]
    fn merge_reports_first_schema_and_immediately_preceding_empty_month() {
        let first = [" B ", "A", "A", ""];
        let different = ["A", " B ", "A", ""];
        let error = merge_statistics_months(vec![
            month_part("first", "raw \0name", Some(0), &first, &[("x", &[])]),
            month_part(" empty \0", "ignored", None, &first, &[]),
            month_part("bad\0 ", "also ignored", None, &different, &[("x", &[])]),
        ])
        .unwrap_err();
        let expected = core::MergeMonthsError {
            name: "raw \0name".into(),
            previous_month: " empty \0".into(),
            month: "bad\0 ".into(),
            previous_outputs: first.map(String::from).to_vec(),
            outputs: different.map(String::from).to_vec(),
        };
        assert_eq!(error.to_string(), expected.to_string());
        assert_eq!(
            error,
            StatisticsMergeError::OutputMismatch {
                name: expected.name,
                previous_month: expected.previous_month,
                month: expected.month,
                previous_outputs: expected.previous_outputs,
                outputs: expected.outputs,
            }
        );
    }

    #[test]
    fn owned_rows_preserve_core_labels_order_and_finite_float_bits() {
        let input = include_str!("../tests/fixtures/statistics_series.xml");
        let expected = core::parse_series(input).unwrap();
        let mut actual = parse_statistics_series(input.to_owned()).unwrap();
        assert_eq!(actual.name, "Café & 気");
        assert_eq!(actual.group, Some(u32::MAX));
        assert_eq!(actual.outputs, ["Energy & power", "V2", "A", "A"]);
        assert_eq!(actual.rows[0].timestamp, " opaque & timestamp ");
        assert_eq!(actual.rows[1].timestamp, actual.rows[2].timestamp);
        for (row, (timestamp, values)) in actual.rows.iter().zip(&expected.rows) {
            assert_eq!(&row.timestamp, timestamp);
            assert_eq!(
                row.values.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                values.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
            );
        }
        assert_eq!(
            actual.rows[0]
                .values
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            [
                0x8000_0000_0000_0000,
                1,
                0x8000_0000_0000_0001,
                0x7fef_ffff_ffff_ffff
            ]
        );
        actual.rows[0].values[0] = 9.0;
        actual.outputs[0].push_str("changed");
        let fresh = parse_statistics_series(input.to_owned()).unwrap();
        assert!(fresh.rows[0].values[0].is_sign_negative());
        assert_eq!(fresh.outputs[0], "Energy & power");
    }

    #[test]
    fn core_defaults_zero_width_and_lazy_attributes_are_not_revalidated() {
        for input in [
            "<Statistics/>",
            "<Statistics Outputs=\"\"/>",
            "<Statistics Outputs=\"A,,A\"><S T=\"\" V=\"1\" V2=\"2\" V3=\"3\"/></Statistics>",
            "<Statistics NumOutputs=\"0\"><S T=\"x\"/></Statistics>",
            "<Statistics Name=\"n\" StatsGroup=\"0\" Outputs=\"A\" NumOutputs=\"1\" broken/>\
             <S T=\"outside\" V=\"2\" broken/>",
            "<Statistics Other=\"&unknown;\"/><Ignored broken/>",
        ] {
            assert_eq!(
                parse_statistics_series(input.into()).unwrap(),
                StatSeries::from(core::parse_series(input).unwrap())
            );
        }
    }

    #[test]
    fn atomic_errors_preserve_variants_fields_display_and_original_causes() {
        for input in [
            "<Statistics></Other>",
            "<Statistics broken/>",
            "<Statistics Name=\"&unknown;\"/>",
            "<Statistics><S T=\"valid\" V=\"1\"/>",
            "<Statistics/><Statistics StatsGroup=\"bad\"/>",
            "",
            "<S/>",
            "<Statistics StatsGroup=\"4294967296\"/>",
            "<Statistics NumOutputs=\"-1\"/>",
            "<Statistics><S/>",
            "<Statistics NumOutputs=\"2\"><S T=\"first\" V=\"1\" V2=\"2\"/><S T=\"later &amp; raw\" V=\"3\"/></Statistics>",
            "<Statistics><S T=\"first\" V=\"1\"/><S T=\"later\" V=\" warm \"/></Statistics>",
            "<Statistics><S T=\"first\" V=\"1\"/><S T=\"later\" V=\"NaN\"/></Statistics>",
        ] {
            let core_error = core::parse_series(input).unwrap_err();
            let message = core_error.to_string();
            let source = core_error.source().map(ToString::to_string);
            let error = parse_statistics_series(input.into()).unwrap_err();
            assert_eq!(error.to_string(), message, "{input}");
            match &error {
                StatisticsError::Xml { source_message }
                | StatisticsError::MalformedAttribute { source_message, .. }
                | StatisticsError::AttributeValue { source_message, .. }
                | StatisticsError::InvalidStatsGroup { source_message, .. }
                | StatisticsError::InvalidNumOutputs { source_message, .. }
                | StatisticsError::InvalidValue { source_message, .. } => {
                    assert_eq!(Some(source_message), source.as_ref());
                }
                _ => assert!(source.is_none()),
            }
        }
        assert!(matches!(
            parse_statistics_series("<Statistics><S T=\"later\" V=\" warm \"/></Statistics>".into()),
            Err(StatisticsError::InvalidValue { timestamp, attribute, value, source_message })
                if timestamp == "later" && attribute == "V" && value == " warm "
                    && source_message == "invalid float literal"
        ));
        assert_eq!(
            StatisticsError::from(core::StatisticsError::UnexpectedClosingElement),
            StatisticsError::UnexpectedClosingElement
        );
        for value in ["NaN", "inf", "-inf", "1e9999"] {
            assert!(matches!(
                parse_statistics_series(format!("<Statistics><S T=\"x\" V=\"{value}\"/></Statistics>")),
                Err(StatisticsError::NonFiniteValue { timestamp, attribute, value: raw })
                    if timestamp == "x" && attribute == "V" && raw == value
            ));
        }
    }
}
