//! Owned color values only; validation, profiles and rendering belong to core.

use lox_core::color as core;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ColorRgb {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
}

impl From<core::Rgb> for ColorRgb {
    fn from(value: core::Rgb) -> Self {
        Self {
            red: value.red.into(),
            green: value.green.into(),
            blue: value.blue.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct ColorHsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64,
}

impl From<core::Hsv> for ColorHsv {
    fn from(value: core::Hsv) -> Self {
        Self {
            hue: value.hue,
            saturation: value.saturation,
            value: value.value,
        }
    }
}

impl From<ColorHsv> for core::Hsv {
    fn from(value: ColorHsv) -> Self {
        Self {
            hue: value.hue,
            saturation: value.saturation,
            value: value.value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ColorPercentRgb {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ColorActor {
    pub value: i64,
    pub rgb: ColorRgb,
}

impl From<core::ActorComposite> for ColorActor {
    fn from(value: core::ActorComposite) -> Self {
        Self {
            value: value.value(),
            rgb: value.rgb().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ColorMood {
    pub value: i64,
    pub percent: ColorPercentRgb,
    pub approximate_rgb: ColorRgb,
}

impl From<core::MoodValue> for ColorMood {
    fn from(value: core::MoodValue) -> Self {
        let percent = value.percent();
        Self {
            value: value.value(),
            percent: ColorPercentRgb {
                red: percent.red.into(),
                green: percent.green.into(),
                blue: percent.blue.into(),
            },
            approximate_rgb: value.approximate_rgb().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, uniffi::Enum)]
pub enum ColorEncodeInput {
    Rgb { red: f64, green: f64, blue: f64 },
    Hsv { hsv: ColorHsv },
    Temperature { kelvin: i64 },
}

#[derive(Clone, Debug, PartialEq, uniffi::Enum)]
pub enum ColorEncoded {
    Rgb {
        rgb: ColorRgb,
        actor: ColorActor,
        mood: ColorMood,
        hsv: ColorHsv,
        hsv_command: String,
    },
    Temperature {
        kelvin: i64,
        brightness: f64,
        command: String,
    },
}

#[derive(Clone, Debug, PartialEq, uniffi::Enum)]
pub enum ColorDecoded {
    Hsv { hsv: ColorHsv, rgb: ColorRgb },
    Temperature { brightness: f64, kelvin: f64 },
    Rgb { actor: ColorActor },
    Mood { mood: ColorMood },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
}

impl From<core::Channel> for ColorChannel {
    fn from(value: core::Channel) -> Self {
        match value {
            core::Channel::Red => Self::Red,
            core::Channel::Green => Self::Green,
            core::Channel::Blue => Self::Blue,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ColorNotation {
    Rgb,
    HsvInput,
    HsvCommand,
    Temperature,
}

impl From<core::Notation> for ColorNotation {
    fn from(value: core::Notation) -> Self {
        match value {
            core::Notation::Rgb => Self::Rgb,
            core::Notation::HsvInput => Self::HsvInput,
            core::Notation::HsvCommand => Self::HsvCommand,
            core::Notation::Temperature => Self::Temperature,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ColorIntegerProfile {
    Decode,
    Mood,
}

impl From<core::IntegerProfile> for ColorIntegerProfile {
    fn from(value: core::IntegerProfile) -> Self {
        match value {
            core::IntegerProfile::Decode => Self::Decode,
            core::IntegerProfile::Mood => Self::Mood,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ColorError {
    #[error("Expected {expected} values ({notation:?}): {value}")]
    Arity {
        value: String,
        notation: ColorNotation,
        expected: i64,
    },
    #[error("Invalid number ({notation:?}): {value}")]
    Number {
        value: String,
        notation: ColorNotation,
    },
    #[error("RGB channel {channel:?} out of range: {value}")]
    RgbChannel { channel: ColorChannel, value: f64 },
    #[error("HSV out of range: {value:?}")]
    HsvRange { value: ColorHsv },
    #[error("Brightness out of range: {value}")]
    Brightness { value: f64 },
    #[error("Kelvin out of range: {value}")]
    Kelvin { value: i64 },
    #[error("Kelvin cannot be represented by the core u32 input: {value}")]
    KelvinRepresentation { value: i64 },
    #[error("Actor {value} has invalid {channel:?} channel: {channel_value}")]
    ActorChannel {
        value: i64,
        channel: ColorChannel,
        channel_value: i64,
    },
    #[error("Mood {value} has invalid {channel:?} channel: {channel_value}")]
    MoodChannel {
        value: i64,
        channel: ColorChannel,
        channel_value: i64,
    },
    #[error("Invalid {profile:?} value: {value}")]
    InvalidValue {
        value: String,
        profile: ColorIntegerProfile,
    },
    #[error("Actor outside decode range: {value}")]
    ActorRange { value: i64 },
}

impl From<core::ColorError> for ColorError {
    fn from(value: core::ColorError) -> Self {
        match value {
            core::ColorError::Arity {
                value,
                notation,
                expected,
            } => Self::Arity {
                value,
                notation: notation.into(),
                expected: i64::try_from(expected).expect("core arity fits i64"),
            },
            core::ColorError::Number { value, notation } => Self::Number {
                value,
                notation: notation.into(),
            },
            core::ColorError::RgbChannel { channel, value } => Self::RgbChannel {
                channel: channel.into(),
                value,
            },
            core::ColorError::HsvRange { value } => Self::HsvRange {
                value: value.into(),
            },
            core::ColorError::Brightness { value } => Self::Brightness { value },
            core::ColorError::Kelvin { value } => Self::Kelvin {
                value: value.into(),
            },
            core::ColorError::ActorChannel {
                value,
                channel,
                channel_value,
            } => Self::ActorChannel {
                value,
                channel: channel.into(),
                channel_value,
            },
            core::ColorError::MoodChannel {
                value,
                channel,
                channel_value,
            } => Self::MoodChannel {
                value,
                channel: channel.into(),
                channel_value,
            },
            core::ColorError::InvalidValue { value, profile } => Self::InvalidValue {
                value,
                profile: profile.into(),
            },
            core::ColorError::ActorRange { value } => Self::ActorRange { value },
        }
    }
}

#[uniffi::export]
pub fn encode_color(
    input: ColorEncodeInput,
    brightness: Option<f64>,
) -> Result<ColorEncoded, ColorError> {
    let input = match input {
        ColorEncodeInput::Rgb { red, green, blue } => {
            core::EncodeInput::Rgb(core::Rgb::from_channels(red, green, blue)?)
        }
        ColorEncodeInput::Hsv { hsv } => core::EncodeInput::Hsv(hsv.into()),
        ColorEncodeInput::Temperature { kelvin } => core::EncodeInput::Temperature {
            kelvin: u32::try_from(kelvin)
                .map_err(|_| ColorError::KelvinRepresentation { value: kelvin })?,
        },
    };
    Ok(match core::encode(input, brightness)? {
        core::EncodedColor::Rgb(rgb) => ColorEncoded::Rgb {
            rgb: rgb.into(),
            actor: rgb.actor_composite().into(),
            mood: rgb.mood_value().into(),
            hsv: rgb.hsv().into(),
            hsv_command: rgb.hsv_command(),
        },
        core::EncodedColor::Temperature(temp) => ColorEncoded::Temperature {
            kelvin: temp.kelvin().into(),
            brightness: temp.brightness(),
            command: temp.command(),
        },
    })
}

#[uniffi::export]
pub fn decode_color_legacy(spec: String) -> Result<ColorDecoded, ColorError> {
    Ok(match core::decode_legacy(&spec)? {
        core::DecodedColor::Hsv { hsv, rgb } => ColorDecoded::Hsv {
            hsv: hsv.into(),
            rgb: rgb.into(),
        },
        core::DecodedColor::Temperature { brightness, kelvin } => {
            ColorDecoded::Temperature { brightness, kelvin }
        }
        core::DecodedColor::Rgb(actor) => ColorDecoded::Rgb {
            actor: actor.into(),
        },
        core::DecodedColor::Mood(mood) => ColorDecoded::Mood { mood: mood.into() },
    })
}

#[uniffi::export]
pub fn parse_mood_color_legacy(spec: String) -> Result<ColorMood, ColorError> {
    core::parse_mood_legacy(&spec)
        .map(Into::into)
        .map_err(Into::into)
}

#[uniffi::export]
pub fn observe_actor_color_legacy(value: i64) -> Result<ColorActor, ColorError> {
    core::ActorComposite::from_legacy_raw(value)
        .map(Into::into)
        .map_err(Into::into)
}

#[uniffi::export]
pub fn color_format_number(value: f64) -> String {
    core::format_number(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_rgb_factory_preserves_parser_validation_and_double_rounding() {
        for (r, g, b) in [(0.5, 127.5, 255.0), (-0.0, 0.0, 0.0)] {
            assert_eq!(
                core::Rgb::from_channels(r, g, b).unwrap(),
                core::Rgb::parse(&format!("{r},{g},{b}")).unwrap()
            );
        }
        let ColorEncoded::Rgb { rgb, .. } = encode_color(
            ColorEncodeInput::Rgb {
                red: 0.5,
                green: 0.0,
                blue: 0.0,
            },
            Some(50.0),
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(rgb.red, 1);
        assert!(matches!(
            encode_color(
                ColorEncodeInput::Rgb {
                    red: 256.0,
                    green: 0.0,
                    blue: 0.0
                },
                None
            ),
            Err(ColorError::RgbChannel {
                channel: ColorChannel::Red,
                value: 256.0
            })
        ));
        assert!(matches!(
            encode_color(ColorEncodeInput::Temperature { kelvin: i64::MAX }, None),
            Err(ColorError::KelvinRepresentation { value: i64::MAX })
        ));
    }

    #[test]
    fn raw_profiles_retain_full_width_and_nonfinite_observations() {
        for line in include_str!("../tests/fixtures/color_profiles.tsv").lines() {
            let fields: Vec<_> = line.split('\t').collect();
            let mood = parse_mood_color_legacy(fields[0].into()).unwrap();
            assert_eq!(mood.value, fields[1].parse::<i64>().unwrap());
            assert_eq!(
                [mood.percent.red, mood.percent.green, mood.percent.blue],
                [fields[2], fields[3], fields[4]].map(|v| v.parse::<i32>().unwrap())
            );
            assert_eq!(
                [
                    mood.approximate_rgb.red,
                    mood.approximate_rgb.green,
                    mood.approximate_rgb.blue
                ],
                [fields[5], fields[6], fields[7]].map(|v| v.parse::<i32>().unwrap())
            );
        }
        let value = 6_710_612_736_i64;
        assert_eq!(
            parse_mood_color_legacy(value.to_string()).unwrap().value,
            value
        );
        assert_eq!(
            observe_actor_color_legacy(-1_000_000_000).unwrap().value,
            -1_000_000_000
        );
        assert!(matches!(
            decode_color_legacy("-1000000000".into()),
            Err(ColorError::ActorRange {
                value: -1_000_000_000
            })
        ));
        let ColorDecoded::Temperature { brightness, kelvin } =
            decode_color_legacy("temp(NaN,-inf)".into()).unwrap()
        else {
            panic!()
        };
        assert!(brightness.is_nan());
        assert_eq!(kelvin, f64::NEG_INFINITY);
    }
}
