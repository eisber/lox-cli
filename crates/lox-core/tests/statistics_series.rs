use lox_core::statistics::{StatEntry, StatSeries, StatisticsError, parse_index, parse_series};
use std::error::Error;

#[test]
fn index_consumer_preserves_owned_order_duplicates_and_permissive_raw_fallback() {
    let html = String::from(
        "<li><a href=\"1a2b3c4d-0001-0002-ffff0123456789ab_4294967295.202613.xml\">\
         A &amp; B &unknown; (Room (North)) 202613</a>\
         <li><a href=\"broken.xml\">ignored</a>\
         <li><a href=\"1a2b3c4d-0001-0002-ffff0123456789ab_4294967295.202613.xml\">\
         A &amp; B &unknown; (Room (North)) 202613</a>",
    );
    let entries: Vec<StatEntry> = parse_index(&html);
    drop(html);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], entries[1]);
    assert_eq!(entries[0].group, Some(u32::MAX));
    assert_eq!(entries[0].month, "2026-13");
    assert_eq!(entries[0].name, "A &amp; B &unknown;");
    assert_eq!(entries[0].label, "A &amp; B &unknown; (Room (North))");
    assert!(parse_index("<LI>not a matching entry</LI>").is_empty());
}

#[test]
fn monthly_consumer_gets_decoded_labels_and_ordered_finite_values_without_time_conversion() {
    let xml = "\u{feff}<?xml version=\"1.0\"?><Envelope>\
        <Statistics Name=\"Caf&#xE9; &amp; 気\" StatsGroup=\" 4294967295 \" \
        NumOutputs=\"3\" Outputs=\" Energy &amp; power, ,third,discarded\">\
        <S T=\" arbitrary &amp; timestamp \" V3=\"3\" V2=\" 2e1 \" V=\"-0\"/>\
        <S T=\"earlier\" V=\"1\" V2=\"2\" V3=\"3\"/>\
        <S T=\"earlier\" V=\"4\" V2=\"5\" V3=\"6\"/>\
        </Statistics></Envelope>trailing<Other/>";
    let StatSeries {
        name,
        group,
        outputs,
        rows,
    } = parse_series(xml).unwrap();
    assert_eq!(name, "Café & 気");
    assert_eq!(group, Some(u32::MAX));
    assert_eq!(outputs, ["Energy & power", "V2", "third"]);
    assert_eq!(rows[0].0, " arbitrary & timestamp ");
    assert_eq!(rows[0].1, [-0.0, 20.0, 3.0]);
    assert!(rows[0].1[0].is_sign_negative());
    assert_eq!(rows[1], ("earlier".into(), vec![1.0, 2.0, 3.0]));
    assert_eq!(rows[2], ("earlier".into(), vec![4.0, 5.0, 6.0]));
}

#[test]
fn monthly_consumer_retains_count_defaults_zero_outputs_and_lazy_attribute_validation() {
    for (xml, labels, rows) in [
        ("<Statistics/>", vec!["V"], vec![]),
        ("<Statistics Outputs=\"\"/>", vec!["V"], vec![]),
        (
            "<Statistics Outputs=\"A,,A\"><S T=\"\" V3=\"3\" V=\"1\" V2=\"2\"/></Statistics>",
            vec!["A", "V2", "A"],
            vec![("".into(), vec![1.0, 2.0, 3.0])],
        ),
        (
            "<Statistics NumOutputs=\"0\" Outputs=\"ignored\"><S T=\"x\"/></Statistics>",
            vec![],
            vec![("x".into(), vec![])],
        ),
        (
            "<Statistics Name=\"n\" StatsGroup=\"0\" Outputs=\"A\" NumOutputs=\"1\" broken/>\
             <S T=\"outside\" V=\"2\" broken/>",
            vec!["A"],
            vec![("outside".into(), vec![2.0])],
        ),
        (
            "<Statistics Other=\"&unknown;\"/><Ignored broken/>",
            vec!["V"],
            vec![],
        ),
    ] {
        let series = parse_series(xml).unwrap();
        assert_eq!(series.outputs, labels, "{xml}");
        assert_eq!(series.rows, rows, "{xml}");
    }
}

#[test]
fn monthly_errors_expose_fields_and_original_sources_without_partial_series() {
    let error = parse_series(
        "<Statistics Outputs=\"A,B\"><S T=\"first\" V=\"1\" V2=\"2\"/>\
         <S T=\"later &amp; raw\" V=\"3\" V2=\" warm \"/></Statistics>",
    )
    .unwrap_err();
    assert_eq!(error.source().unwrap().to_string(), "invalid float literal");
    match error {
        StatisticsError::InvalidValue {
            timestamp,
            attribute,
            value,
            ..
        } => {
            assert_eq!(timestamp, "later & raw");
            assert_eq!(attribute, "V2");
            assert_eq!(value, " warm ");
        }
        other => panic!("expected numeric error, got {other:?}"),
    }
    let overflow = (usize::MAX as u128 + 1).to_string();
    for xml in [
        "<Statistics NumOutputs=\"-1\"/>".to_string(),
        format!("<Statistics NumOutputs=\"{overflow}\"/>"),
    ] {
        let error = parse_series(&xml).unwrap_err();
        assert!(error.source().is_some());
        assert!(matches!(error, StatisticsError::InvalidNumOutputs { .. }));
    }
    let error = parse_series("<Statistics StatsGroup=\"4294967296\"/>").unwrap_err();
    assert!(error.source().is_some());
    assert!(matches!(error, StatisticsError::InvalidStatsGroup { .. }));
    let error = parse_series("<Statistics broken/>").unwrap_err();
    assert!(error.source().is_some());
    assert!(
        matches!(error, StatisticsError::MalformedAttribute { attribute, .. } if attribute == "Name")
    );
    let error = parse_series("<Statistics Name=\"&unknown;\"/>").unwrap_err();
    assert!(error.source().is_some());
    assert!(
        matches!(error, StatisticsError::AttributeValue { attribute, .. } if attribute == "Name")
    );
    for value in ["NaN", "inf", "-inf", "1e9999"] {
        let error = parse_series(&format!(
            "<Statistics><S T=\"first\" V=\"1\"/><S T=\"later\" V=\"{value}\"/></Statistics>"
        ))
        .unwrap_err();
        assert!(matches!(
            error,
            StatisticsError::NonFiniteValue { timestamp, attribute, value: raw }
                if timestamp == "later" && attribute == "V" && raw == value
        ));
    }
}

#[test]
fn monthly_framing_and_required_field_errors_keep_original_precedence() {
    for xml in ["", "text", "<Other/>"] {
        assert!(matches!(
            parse_series(xml),
            Err(StatisticsError::MissingStatistics)
        ));
    }
    assert!(matches!(
        parse_series("<S/>"),
        Err(StatisticsError::RowBeforeStatistics)
    ));
    assert!(matches!(
        parse_series("<Statistics/><Statistics StatsGroup=\"bad\"/>"),
        Err(StatisticsError::MultipleStatistics)
    ));
    assert!(matches!(
        parse_series("<Statistics><S T=\"valid\" V=\"1\"/>"),
        Err(StatisticsError::TruncatedXml)
    ));
    assert!(matches!(
        parse_series("<Statistics><S/>"),
        Err(StatisticsError::MissingTimestamp)
    ));
    assert!(matches!(
        parse_series("<Statistics NumOutputs=\"2\"><S T=\"x\" V=\"1\"/>"),
        Err(StatisticsError::MissingValue { timestamp, attribute })
            if timestamp == "x" && attribute == "V2"
    ));
    for xml in [
        "</Statistics>",
        "<Statistics></Other>",
        "<Statistics/><Other>",
    ] {
        assert!(parse_series(xml).is_err(), "{xml}");
    }
}
