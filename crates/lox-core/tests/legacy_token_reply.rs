use lox_core::legacy_token_reply::{ReplyErrorKind, parse};
use std::error::Error;

#[test]
fn legacy_codes_are_observed_not_validated() {
    let cases = [
        (r#""+200""#, 200),
        (r#""000200""#, 200),
        (r#"" 200""#, 0),
        (r#""200 ""#, 0),
        (r#""4294967296""#, 0),
        ("4294967496", 200),
        ("18446744073709551615", u32::MAX),
        ("-1", 0),
        ("200.0", 0),
        ("true", 0),
        ("null", 0),
        ("[]", 0),
        ("{}", 0),
    ];
    for (code, expected) in cases {
        let reply = parse(&format!(r#"{{"LL":{{"code":{code},"Code":401}}}}"#)).unwrap();
        assert_eq!(reply.code, expected, "{code}");
        assert!(reply.value.is_empty());
    }
    assert_eq!(parse(r#"{"LL":{"Code":200}}"#).unwrap().code, 0);
    assert_eq!(parse(r#"{"LL":{"code":"401"}}"#).unwrap().code, 401);
}

#[test]
fn present_envelopes_and_opaque_values_keep_legacy_projection() {
    for ll in ["null", "true", "42", "[]", r#""text""#] {
        let reply = parse(&format!(r#"{{"LL":{ll}}}"#)).unwrap();
        assert_eq!((reply.code, reply.value.as_str()), (0, ""));
    }
    for (value, expected) in [
        (r#""""#, ""),
        (r#"" \u0000Grüße雪\n ""#, " \0Grüße雪\n "),
        ("null", "null"),
        ("true", "true"),
        (r#"[1,"雪",null]"#, r#"[1,"雪",null]"#),
        ("1.25", "1.25"),
        ("-0.0", "-0.0"),
    ] {
        assert_eq!(
            parse(&format!(r#"{{"LL":{{"value":{value}}}}}"#))
                .unwrap()
                .value,
            expected
        );
    }
    let object = parse(r#"{"LL":{"value":{"z":0,"a":2,"z":1}}}"#)
        .unwrap()
        .value;
    assert!(
        object == r#"{"a":2,"z":1}"# || object == r#"{"z":1,"a":2}"#,
        "{object}"
    );
}

#[test]
fn errors_are_structured_redacted_and_original_source_is_opt_in() {
    for text in ["null", "[]", "42", r#""root""#, "{}", r#"{"ll":{}}"#] {
        let error = parse(text).unwrap_err();
        assert_eq!(error.kind(), ReplyErrorKind::MissingEnvelope);
        assert_eq!(error.location(), None);
        assert_eq!(error.to_string(), "response missing 'LL' envelope");
        assert!(error.source().is_none());
        assert!(error.into_json_error().is_none());
    }
    let text = "{\n\"LL\":{\"value\":\"PRIVATE_BODY\",\"code\":?}}";
    let original = serde_json::from_str::<serde_json::Value>(text).unwrap_err();
    let error = parse(text).unwrap_err();
    assert_eq!(error.kind(), ReplyErrorKind::InvalidJson);
    assert_eq!(error.location(), Some((original.line(), original.column())));
    assert!(!format!("{error:?} {error}").contains("PRIVATE_BODY"));
    assert!(error.source().is_none());
    let recovered = error.into_json_error().unwrap();
    assert_eq!(recovered.classify(), original.classify());
    assert_eq!(recovered.to_string(), original.to_string());
    assert_eq!(format!("{recovered:?}"), format!("{original:?}"));
    let reply = parse(r#"{"LL":{"value":"PRIVATE_BODY"}}"#).unwrap();
    assert!(!format!("{reply:?}").contains("PRIVATE_BODY"));
}
