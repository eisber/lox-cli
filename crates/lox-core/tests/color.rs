use lox_core::color::{
    ActorComposite, Channel, ColorError, DecodedColor, EncodeInput, EncodedColor, Hsv,
    IntegerProfile, Notation, PercentRgb, Rgb, decode_legacy, encode, format_number,
    parse_mood_legacy,
};

fn encoded_rgb(input: EncodeInput, brightness: Option<f64>) -> Rgb {
    match encode(input, brightness).unwrap() {
        EncodedColor::Rgb(rgb) => rgb,
        other => panic!("expected RGB, got {other:?}"),
    }
}

#[test]
fn actor_roundtrip_and_mood_quantization_are_distinct() {
    for (spec, composite, percent) in [
        ("255,0,0", 255, (100, 0, 0)),
        ("0,255,0", 255_000, (0, 100, 0)),
        ("0,0,255", 255_000_000, (0, 0, 100)),
        ("255,255,255", 255_255_255, (100, 100, 100)),
        ("100,40,0", 40_100, (39, 16, 0)),
        ("0,0,0", 0, (0, 0, 0)),
    ] {
        let rgb = Rgb::parse(spec).unwrap();
        let actor = rgb.actor_composite();
        assert_eq!(actor.value(), composite);
        assert_eq!(
            ActorComposite::from_legacy_raw(composite).unwrap().rgb(),
            rgb
        );
        let mood = rgb.mood_value();
        assert_eq!(
            mood.percent(),
            PercentRgb {
                red: percent.0,
                green: percent.1,
                blue: percent.2
            }
        );
        assert_eq!(parse_mood_legacy(&mood.value().to_string()).unwrap(), mood);
    }
    let dim = encoded_rgb(
        EncodeInput::Rgb(Rgb::parse("1.49,127.5,254.5").unwrap()),
        Some(50.0),
    );
    assert_eq!(
        dim,
        Rgb {
            red: 1,
            green: 64,
            blue: 128
        }
    );
    assert_eq!(dim.hsv_command(), "hsv(210,99,50)");
}

#[test]
fn encode_hsv_validates_but_legacy_observers_wrap_and_saturate() {
    for (spec, expected) in [
        (
            "0,100,100",
            Rgb {
                red: 255,
                green: 0,
                blue: 0,
            },
        ),
        (
            "120,100,100",
            Rgb {
                red: 0,
                green: 255,
                blue: 0,
            },
        ),
        (
            "240,100,100",
            Rgb {
                red: 0,
                green: 0,
                blue: 255,
            },
        ),
        (
            "360,100,100",
            Rgb {
                red: 255,
                green: 0,
                blue: 0,
            },
        ),
    ] {
        assert_eq!(
            encoded_rgb(EncodeInput::Hsv(Hsv::parse(spec).unwrap()), None),
            expected
        );
    }
    for spec in [
        "720,100,100",
        "-120,100,100",
        "NaN,100,100",
        "inf,100,100",
        "0,100,200",
    ] {
        let hsv = Hsv::parse(spec).unwrap();
        assert!(matches!(
            encode(EncodeInput::Hsv(hsv), Some(f64::NAN)),
            Err(ColorError::HsvRange { .. })
        ));
        let DecodedColor::Hsv { rgb, .. } = decode_legacy(&format!(" HsV({spec}) ")).unwrap()
        else {
            panic!("expected HSV")
        };
        assert_eq!(
            parse_mood_legacy(&format!("hsv({spec})")).unwrap(),
            rgb.mood_value()
        );
    }
}

#[test]
fn integer_profiles_preserve_modulo_high_bits_and_rejection_order() {
    assert!(matches!(
        decode_legacy("1000040100"),
        Err(ColorError::ActorRange {
            value: 1_000_040_100
        })
    ));
    assert_eq!(
        parse_mood_legacy("1000040100").unwrap().value(),
        1_610_628_775
    );
    assert_eq!(
        parse_mood_legacy("-1000000000").unwrap().value(),
        1_610_612_736
    );
    assert!(matches!(
        decode_legacy("-1000000000"),
        Err(ColorError::ActorRange { .. })
    ));
    let DecodedColor::Mood(mood) = decode_legacy("9610622736").unwrap() else {
        panic!("expected mood")
    };
    assert_eq!(mood.value(), 9_610_622_736);
    assert_eq!(mood.approximate_rgb().hsv_command(), "hsv(120,100,10)");
    assert_eq!(parse_mood_legacy(" +009610622736 ").unwrap(), mood);
    assert!(matches!(
        decode_legacy("1610612735"),
        Err(ColorError::ActorRange { .. })
    ));
    assert!(matches!(
        parse_mood_legacy("1610612735"),
        Err(ColorError::ActorChannel {
            channel: Channel::Red,
            channel_value: 735,
            ..
        })
    ));
    assert!(matches!(
        decode_legacy("1610812736"),
        Err(ColorError::MoodChannel {
            channel: Channel::Green,
            channel_value: 200,
            ..
        })
    ));
    assert!(matches!(
        decode_legacy("999"),
        Err(ColorError::ActorChannel {
            channel: Channel::Red,
            channel_value: 999,
            ..
        })
    ));
    assert!(matches!(
        parse_mood_legacy(&i64::MIN.to_string()),
        Err(ColorError::ActorChannel { .. })
    ));
    assert!(matches!(
        decode_legacy(&i64::MAX.to_string()),
        Err(ColorError::MoodChannel { .. })
    ));
}

#[test]
fn temperature_encode_bounds_and_legacy_float_observations_stay_separate() {
    for kelvin in [1000, 15000] {
        let EncodedColor::Temperature(temp) =
            encode(EncodeInput::Temperature { kelvin }, Some(-0.0)).unwrap()
        else {
            panic!("expected temperature")
        };
        assert!(temp.brightness().is_sign_negative());
        assert_eq!(temp.command(), format!("temp(0,{kelvin})"));
    }
    assert!(matches!(
        encode(
            EncodeInput::Temperature { kelvin: 999 },
            Some(f64::INFINITY)
        ),
        Err(ColorError::Brightness { .. })
    ));
    assert_eq!(
        encode(EncodeInput::Temperature { kelvin: 15001 }, None).unwrap_err(),
        ColorError::Kelvin { value: 15001 }
    );
    let DecodedColor::Temperature { brightness, kelvin } =
        decode_legacy(" LuMiTeCh(-0,NaN) ").unwrap()
    else {
        panic!("expected temperature observation")
    };
    assert!(brightness.is_sign_negative());
    assert!(kelvin.is_nan());
    assert_eq!(format_number(-0.0), "0");
    assert_eq!(format_number(f64::INFINITY), "inf");
    assert_eq!(format_number(f64::NAN), "NaN");
    assert_eq!(format_number(1e-10), "0");
}

#[test]
fn parsing_and_validation_errors_retain_typed_input_context_and_order() {
    assert_eq!(
        Rgb::parse("0,256,no").unwrap_err(),
        ColorError::Number {
            value: "no".into(),
            notation: Notation::Rgb
        }
    );
    assert_eq!(
        Hsv::parse("1,2").unwrap_err(),
        ColorError::Arity {
            value: "1,2".into(),
            notation: Notation::HsvInput,
            expected: 3
        }
    );
    assert!(
        matches!(Rgb::parse("NaN,0,0"), Err(ColorError::RgbChannel { channel: Channel::Red, value }) if value.is_nan())
    );
    assert!(
        matches!(encode(EncodeInput::Rgb(Rgb::parse("0,0,0").unwrap()), Some(f64::NAN)), Err(ColorError::Brightness { value }) if value.is_nan())
    );
    for (spec, notation) in [
        ("hsv(1,2)", Notation::HsvCommand),
        ("temp(1)", Notation::Temperature),
    ] {
        assert!(
            matches!(decode_legacy(spec), Err(ColorError::Arity { notation: actual, .. }) if actual == notation)
        );
    }
    assert_eq!(
        parse_mood_legacy("temp(10,2700)").unwrap_err(),
        ColorError::InvalidValue {
            value: "temp(10,2700)".into(),
            profile: IntegerProfile::Mood
        }
    );
    assert!(matches!(
        decode_legacy("hsv (1,2,3)"),
        Err(ColorError::InvalidValue {
            profile: IntegerProfile::Decode,
            ..
        })
    ));
}
