//! Owned monthly XML projections; parsing and admission remain in Core.

use lox_core::statistics as core;

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct StatRow {
    pub timestamp: String,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct StatSeries {
    pub name: String,
    /// Core u32 widened losslessly so Java Long is always nonnegative.
    pub group: Option<u64>,
    pub outputs: Vec<String>,
    pub rows: Vec<StatRow>,
}

impl From<core::StatSeries> for StatSeries {
    fn from(value: core::StatSeries) -> Self {
        Self {
            name: value.name,
            group: value.group.map(u64::from),
            outputs: value.outputs,
            rows: value
                .rows
                .into_iter()
                .map(|(timestamp, values)| StatRow { timestamp, values })
                .collect(),
        }
    }
}

/// Payload-free variants deliberately omit XML, timestamps and numeric input.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum StatisticsError {
    #[error("statistics XML reader error")]
    Xml,
    #[error("malformed statistics XML attribute")]
    MalformedAttribute,
    #[error("invalid statistics XML attribute encoding")]
    AttributeValue,
    #[error("unexpected statistics closing element")]
    UnexpectedClosingElement,
    #[error("truncated statistics XML")]
    TruncatedXml,
    #[error("multiple statistics headers")]
    MultipleStatistics,
    #[error("missing statistics header")]
    MissingStatistics,
    #[error("row before statistics header")]
    RowBeforeStatistics,
    #[error("invalid statistics group")]
    InvalidStatsGroup,
    #[error("invalid statistics output count")]
    InvalidNumOutputs,
    #[error("missing statistics timestamp")]
    MissingTimestamp,
    #[error("missing statistics row value")]
    MissingValue,
    #[error("invalid statistics row value")]
    InvalidValue,
    #[error("nonfinite statistics row value")]
    NonFiniteValue,
}

impl From<core::StatisticsError> for StatisticsError {
    fn from(value: core::StatisticsError) -> Self {
        match value {
            core::StatisticsError::Xml { .. } => Self::Xml,
            core::StatisticsError::MalformedAttribute { .. } => Self::MalformedAttribute,
            core::StatisticsError::AttributeValue { .. } => Self::AttributeValue,
            core::StatisticsError::UnexpectedClosingElement => Self::UnexpectedClosingElement,
            core::StatisticsError::TruncatedXml => Self::TruncatedXml,
            core::StatisticsError::MultipleStatistics => Self::MultipleStatistics,
            core::StatisticsError::MissingStatistics => Self::MissingStatistics,
            core::StatisticsError::RowBeforeStatistics => Self::RowBeforeStatistics,
            core::StatisticsError::InvalidStatsGroup { .. } => Self::InvalidStatsGroup,
            core::StatisticsError::InvalidNumOutputs { .. } => Self::InvalidNumOutputs,
            core::StatisticsError::MissingTimestamp => Self::MissingTimestamp,
            core::StatisticsError::MissingValue { .. } => Self::MissingValue,
            core::StatisticsError::InvalidValue { .. } => Self::InvalidValue,
            core::StatisticsError::NonFiniteValue { .. } => Self::NonFiniteValue,
        }
    }
}

/// Decode monthly XML text; timestamps remain opaque, with no transport or cap.
#[uniffi::export]
pub fn parse_statistics_series(xml: String) -> Result<StatSeries, StatisticsError> {
    core::parse_series(&xml).map(Into::into).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_nested_projection_preserves_core_data_and_negative_zero() {
        let xml = include_str!("../tests/fixtures/statistics.xml");
        let expected = core::parse_series(xml).unwrap();
        let result = parse_statistics_series(xml.into()).unwrap();
        assert_eq!(result.name, expected.name);
        assert_eq!(result.group, Some(4_294_967_295));
        assert_eq!(result.outputs, expected.outputs);
        for (row, (timestamp, values)) in result.rows.iter().zip(expected.rows) {
            assert_eq!(row.timestamp, timestamp);
            assert_eq!(row.values, values);
            assert_eq!(row.values[0].to_bits(), values[0].to_bits());
        }
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0].values[0].to_bits(), (-0.0_f64).to_bits());
        let empty = parse_statistics_series("<Statistics NumOutputs=\"0\"/>".into()).unwrap();
        assert_eq!(empty.group, None);
        assert!(empty.outputs.is_empty() && empty.rows.is_empty());
    }

    #[test]
    fn core_errors_are_atomic_payload_free_and_recoverable() {
        for (xml, expected) in [
            ("<Statistics></secret>", StatisticsError::Xml),
            ("<Statistics broken/>", StatisticsError::MalformedAttribute),
            (
                "<Statistics Name=\"&secret;\"/>",
                StatisticsError::AttributeValue,
            ),
            ("<Statistics>", StatisticsError::TruncatedXml),
            (
                "<Statistics/><Statistics/>",
                StatisticsError::MultipleStatistics,
            ),
            ("", StatisticsError::MissingStatistics),
            ("<S/>", StatisticsError::RowBeforeStatistics),
            (
                "<Statistics StatsGroup=\"secret\"/>",
                StatisticsError::InvalidStatsGroup,
            ),
            (
                "<Statistics NumOutputs=\"secret\"/>",
                StatisticsError::InvalidNumOutputs,
            ),
            ("<Statistics><S/>", StatisticsError::MissingTimestamp),
            (
                "<Statistics><S T=\"secret\"/>",
                StatisticsError::MissingValue,
            ),
            (
                "<Statistics><S T=\"secret\" V=\"secret\"/>",
                StatisticsError::InvalidValue,
            ),
            (
                "<Statistics><S T=\"secret\" V=\"NaN\"/>",
                StatisticsError::NonFiniteValue,
            ),
        ] {
            let error = parse_statistics_series(xml.into()).unwrap_err();
            assert_eq!(error, expected);
            assert!(!format!("{error:?} {error}").contains("secret"));
            assert!(parse_statistics_series("<Statistics/>".into()).is_ok());
        }
        assert_eq!(
            StatisticsError::from(core::StatisticsError::UnexpectedClosingElement),
            StatisticsError::UnexpectedClosingElement
        );
    }
}
