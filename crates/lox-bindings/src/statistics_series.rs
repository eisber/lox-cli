//! Owned monthly XML series; parsing and rejection policy remain in Core.

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
