//! Offline codecs for the CLI's existing color profiles, not native GUI modes.
//!
//! Actor composites use byte channels in base 1000. LightsceneC mood values
//! instead use percent channels plus a prefix. Legacy observations deliberately
//! accept more than validated encode inputs; they are not device-validity claims.

use std::fmt;

const RGB_MAX: i64 = 255 + 255 * 1_000 + 255 * 1_000_000;
const MOOD_PREFIX: i64 = 0x6000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    /// Parse floating channels, validate all three, then round to bytes.
    pub fn parse(spec: &str) -> Result<Self, ColorError> {
        let (r, g, b) = parse_triple(spec, Notation::Rgb)?;
        Self::from_channels(r, g, b)
    }

    /// Validate floating channels, then round to bytes, just like [`Self::parse`].
    pub fn from_channels(r: f64, g: f64, b: f64) -> Result<Self, ColorError> {
        for (channel, value) in [(Channel::Red, r), (Channel::Green, g), (Channel::Blue, b)] {
            if !(0.0..=255.0).contains(&value) {
                return Err(ColorError::RgbChannel { channel, value });
            }
        }
        Ok(Self {
            red: r.round() as u8,
            green: g.round() as u8,
            blue: b.round() as u8,
        })
    }

    pub fn actor_composite(self) -> ActorComposite {
        ActorComposite {
            value: self.red as i64 + self.green as i64 * 1_000 + self.blue as i64 * 1_000_000,
            rgb: self,
        }
    }

    pub fn mood_value(self) -> MoodValue {
        let percent = PercentRgb {
            red: chan_to_pct(self.red),
            green: chan_to_pct(self.green),
            blue: chan_to_pct(self.blue),
        };
        MoodValue {
            value: MOOD_PREFIX
                + percent.red as i64
                + percent.green as i64 * 1_000
                + percent.blue as i64 * 1_000_000,
            percent,
        }
    }

    /// Hue in degrees, saturation/value in percent.
    pub fn hsv(self) -> Hsv {
        let (hue, saturation, value) = rgb_to_hsv(self.red, self.green, self.blue);
        Hsv {
            hue,
            saturation: saturation * 100.0,
            value: value * 100.0,
        }
    }

    pub fn hsv_command(self) -> String {
        let hsv = self.hsv();
        format!(
            "hsv({},{},{})",
            hsv.hue.round() as i64,
            hsv.saturation.round() as i64,
            hsv.value.round() as i64
        )
    }
}

/// An input or observation, not an always-validated HSV value.
///
/// [`encode`] validates ranges; legacy decode/mood parsing intentionally does not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub hue: f64,
    pub saturation: f64,
    pub value: f64,
}

impl Hsv {
    /// Parse the encode profile's `H,S,V` notation without range validation.
    pub fn parse(spec: &str) -> Result<Self, ColorError> {
        parse_hsv(spec, Notation::HsvInput)
    }

    fn legacy_rgb(self) -> Rgb {
        let (red, green, blue) = hsv_to_rgb(self.hue, self.saturation / 100.0, self.value / 100.0);
        Rgb { red, green, blue }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorComposite {
    value: i64,
    rgb: Rgb,
}

impl ActorComposite {
    /// Observe the original modulo-channel profile, without a global range cap.
    ///
    /// Unlike numeric [`decode_legacy`], this admits high (and some negative)
    /// integers when all three modulo channels are byte-sized.
    pub fn from_legacy_raw(value: i64) -> Result<Self, ColorError> {
        let (r, g, b) = channels(value);
        for (channel, value_channel) in [(Channel::Red, r), (Channel::Green, g), (Channel::Blue, b)]
        {
            if !(0..=255).contains(&value_channel) {
                return Err(ColorError::ActorChannel {
                    value,
                    channel,
                    channel_value: value_channel,
                });
            }
        }
        Ok(Self {
            value,
            rgb: Rgb {
                red: r as u8,
                green: g as u8,
                blue: b as u8,
            },
        })
    }

    pub fn value(self) -> i64 {
        self.value
    }

    pub fn rgb(self) -> Rgb {
        self.rgb
    }
}

/// Percent-channel observation returned by a mood codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PercentRgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoodValue {
    value: i64,
    percent: PercentRgb,
}

impl MoodValue {
    pub fn value(self) -> i64 {
        self.value
    }

    pub fn percent(self) -> PercentRgb {
        self.percent
    }

    /// Reconstruct the existing approximate byte-color projection.
    pub fn approximate_rgb(self) -> Rgb {
        Rgb {
            red: ((self.percent.red as f64 / 100.0) * 255.0).round() as u8,
            green: ((self.percent.green as f64 / 100.0) * 255.0).round() as u8,
            blue: ((self.percent.blue as f64 / 100.0) * 255.0).round() as u8,
        }
    }

    fn observe(value: i64) -> Result<Self, ColorError> {
        // Callers classify the prefix before subtracting, avoiding i64 underflow.
        let (r, g, b) = channels(value - MOOD_PREFIX);
        for (channel, channel_value) in [(Channel::Red, r), (Channel::Green, g), (Channel::Blue, b)]
        {
            if !(0..=100).contains(&channel_value) {
                return Err(ColorError::MoodChannel {
                    value,
                    channel,
                    channel_value,
                });
            }
        }
        Ok(Self {
            value,
            percent: PercentRgb {
                red: r as u8,
                green: g as u8,
                blue: b as u8,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodeInput {
    Rgb(Rgb),
    Hsv(Hsv),
    Temperature { kelvin: u32 },
}

/// Validated temperature producer. There is deliberately no integer packing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    kelvin: u32,
    brightness: f64,
}

impl Temperature {
    pub fn kelvin(self) -> u32 {
        self.kelvin
    }

    pub fn brightness(self) -> f64 {
        self.brightness
    }

    pub fn command(self) -> String {
        format!("temp({},{})", format_number(self.brightness), self.kelvin)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodedColor {
    Rgb(Rgb),
    Temperature(Temperature),
}

/// Encode the existing producer profile; optional brightness scales rounded bytes.
pub fn encode(input: EncodeInput, brightness: Option<f64>) -> Result<EncodedColor, ColorError> {
    let rgb = match input {
        EncodeInput::Temperature { kelvin } => {
            let brightness = brightness.unwrap_or(100.0);
            validate_brightness(brightness)?;
            if !(1000..=15000).contains(&kelvin) {
                return Err(ColorError::Kelvin { value: kelvin });
            }
            return Ok(EncodedColor::Temperature(Temperature {
                kelvin,
                brightness,
            }));
        }
        EncodeInput::Rgb(rgb) => rgb,
        EncodeInput::Hsv(hsv) => {
            if !(0.0..=360.0).contains(&hsv.hue)
                || !(0.0..=100.0).contains(&hsv.saturation)
                || !(0.0..=100.0).contains(&hsv.value)
            {
                return Err(ColorError::HsvRange { value: hsv });
            }
            hsv.legacy_rgb()
        }
    };
    let rgb = if let Some(brightness) = brightness {
        validate_brightness(brightness)?;
        let scale = brightness / 100.0;
        Rgb {
            red: (rgb.red as f64 * scale).round() as u8,
            green: (rgb.green as f64 * scale).round() as u8,
            blue: (rgb.blue as f64 * scale).round() as u8,
        }
    } else {
        rgb
    };
    Ok(EncodedColor::Rgb(rgb))
}

/// Legacy observations can contain nonfinite or out-of-encode-range floats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecodedColor {
    Hsv { hsv: Hsv, rgb: Rgb },
    Temperature { brightness: f64, kelvin: f64 },
    Rgb(ActorComposite),
    Mood(MoodValue),
}

/// Decode the actual existing command profile, including its asymmetries.
pub fn decode_legacy(spec: &str) -> Result<DecodedColor, ColorError> {
    let value = spec.trim();
    if let Some(args) = strip_call(value, "hsv") {
        let hsv = parse_hsv(args, Notation::HsvCommand)?;
        return Ok(DecodedColor::Hsv {
            hsv,
            rgb: hsv.legacy_rgb(),
        });
    }
    if let Some(args) = strip_call(value, "temp").or_else(|| strip_call(value, "lumitech")) {
        let (brightness, kelvin) = parse_pair(args, Notation::Temperature)?;
        return Ok(DecodedColor::Temperature { brightness, kelvin });
    }
    let n: i64 = value.parse().map_err(|_| ColorError::InvalidValue {
        value: value.into(),
        profile: IntegerProfile::Decode,
    })?;
    if n >= MOOD_PREFIX {
        return MoodValue::observe(n).map(DecodedColor::Mood);
    }
    if (0..=RGB_MAX).contains(&n) {
        ActorComposite::from_legacy_raw(n).map(DecodedColor::Rgb)
    } else {
        Err(ColorError::ActorRange { value: n })
    }
}

/// Existing raw `set-mood-color`/`add-mood` parser, not mode-aware eligibility.
///
/// Raw moods preserve their full original i64, including ignored high digits.
/// Raw actor integers have modulo-channel validation but no decode range cap.
pub fn parse_mood_legacy(spec: &str) -> Result<MoodValue, ColorError> {
    let value = spec.trim();
    if let Some(args) = strip_call(value, "hsv") {
        return Ok(parse_hsv(args, Notation::HsvCommand)?
            .legacy_rgb()
            .mood_value());
    }
    if let Some(args) = strip_call(value, "rgb") {
        return Ok(Rgb::parse(args)?.mood_value());
    }
    let n: i64 = value.parse().map_err(|_| ColorError::InvalidValue {
        value: value.into(),
        profile: IntegerProfile::Mood,
    })?;
    if n >= MOOD_PREFIX {
        MoodValue::observe(n)
    } else {
        Ok(ActorComposite::from_legacy_raw(n)?.rgb().mood_value())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Red,
    Green,
    Blue,
}

impl Channel {
    pub fn rgb_label(self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Green => "G",
            Self::Blue => "B",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::Blue => "blue",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Rgb,
    HsvInput,
    HsvCommand,
    Temperature,
}

impl fmt::Display for Notation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rgb => "R,G,B",
            Self::HsvInput => "H,S,V",
            Self::HsvCommand => "hsv(H,S,V)",
            Self::Temperature => "temp(brightness,kelvin)",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerProfile {
    Decode,
    Mood,
}

/// Structured errors; diagnostics may contain caller-provided color text.
///
/// Float parsing intentionally discards its source, as the original head did.
/// No terminal, clap or output-format policy is embedded here.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorError {
    Arity {
        value: String,
        notation: Notation,
        expected: usize,
    },
    Number {
        value: String,
        notation: Notation,
    },
    RgbChannel {
        channel: Channel,
        value: f64,
    },
    HsvRange {
        value: Hsv,
    },
    Brightness {
        value: f64,
    },
    Kelvin {
        value: u32,
    },
    ActorChannel {
        value: i64,
        channel: Channel,
        channel_value: i64,
    },
    MoodChannel {
        value: i64,
        channel: Channel,
        channel_value: i64,
    },
    InvalidValue {
        value: String,
        profile: IntegerProfile,
    },
    ActorRange {
        value: i64,
    },
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arity {
                value,
                notation,
                expected,
            } => {
                let count = if *expected == 3 { "three" } else { "two" };
                write!(
                    f,
                    "expected {count} comma-separated values ({notation}), got '{value}'"
                )
            }
            Self::Number { value, notation } => write!(f, "'{value}' is not a number ({notation})"),
            Self::RgbChannel { channel, value } => {
                write!(f, "RGB channel {channel}={value} out of range (0..255)")
            }
            Self::HsvRange { .. } => f.write_str("HSV expects H in 0..360, S and V in 0..100"),
            Self::Brightness { value } => write!(f, "brightness {value} out of range (0..100)"),
            Self::Kelvin { value } => write!(f, "kelvin {value} out of range (1000..15000)"),
            Self::ActorChannel {
                value,
                channel,
                channel_value,
            } => write!(
                f,
                "composite {value} has {channel} channel {channel_value} outside 0..255 — not a valid RGB value"
            ),
            Self::MoodChannel {
                value,
                channel,
                channel_value,
            } => write!(
                f,
                "mood value {value} has {channel} channel {channel_value}% outside 0..100 — not a valid mood color"
            ),
            Self::InvalidValue {
                value,
                profile: IntegerProfile::Decode,
            } => write!(
                f,
                "'{value}' is not a number or a hsv(...)/temp(...) command"
            ),
            Self::InvalidValue {
                value,
                profile: IntegerProfile::Mood,
            } => write!(
                f,
                "'{value}' is not hsv(...), rgb(...), or an integer mood value"
            ),
            Self::ActorRange { value } => write!(
                f,
                "value {value} is outside the RGB composite range (0..{RGB_MAX}); color-temperature values use the temp(brightness,kelvin) command string, not an integer"
            ),
        }
    }
}

impl std::error::Error for ColorError {}

fn channels(n: i64) -> (i64, i64, i64) {
    (n % 1_000, (n / 1_000) % 1_000, (n / 1_000_000) % 1_000)
}

fn chan_to_pct(c: u8) -> u8 {
    ((c as f64 / 255.0) * 100.0).round() as u8
}

fn parse_hsv(spec: &str, notation: Notation) -> Result<Hsv, ColorError> {
    let (hue, saturation, value) = parse_triple(spec, notation)?;
    Ok(Hsv {
        hue,
        saturation,
        value,
    })
}

fn parse_triple(s: &str, notation: Notation) -> Result<(f64, f64, f64), ColorError> {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.len() != 3 {
        return Err(ColorError::Arity {
            value: s.into(),
            notation,
            expected: 3,
        });
    }
    Ok((
        parse_number(parts[0], notation)?,
        parse_number(parts[1], notation)?,
        parse_number(parts[2], notation)?,
    ))
}

fn parse_pair(s: &str, notation: Notation) -> Result<(f64, f64), ColorError> {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.len() != 2 {
        return Err(ColorError::Arity {
            value: s.into(),
            notation,
            expected: 2,
        });
    }
    Ok((
        parse_number(parts[0], notation)?,
        parse_number(parts[1], notation)?,
    ))
}

fn parse_number(s: &str, notation: Notation) -> Result<f64, ColorError> {
    s.parse().map_err(|_| ColorError::Number {
        value: s.into(),
        notation,
    })
}

fn strip_call<'a>(v: &'a str, name: &str) -> Option<&'a str> {
    let v = v.trim();
    let lower = v.to_ascii_lowercase();
    let prefix = format!("{name}(");
    if lower.starts_with(&prefix) && v.ends_with(')') {
        Some(&v[prefix.len()..v.len() - 1])
    } else {
        None
    }
}

fn validate_brightness(value: f64) -> Result<(), ColorError> {
    if !(0.0..=100.0).contains(&value) {
        return Err(ColorError::Brightness { value });
    }
    Ok(())
}

/// Existing command-number spelling, also used by legacy temperature displays.
pub fn format_number(n: f64) -> String {
    if (n.fract()).abs() < 1e-9 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h {
        _ if h < 60.0 => (c, x, 0.0),
        _ if h < 120.0 => (x, c, 0.0),
        _ if h < 180.0 => (0.0, c, x),
        _ if h < 240.0 => (0.0, x, c),
        _ if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let h = if delta < 1e-9 {
        0.0
    } else if (max - r).abs() < 1e-9 {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if (max - g).abs() < 1e-9 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let s = if max < 1e-9 { 0.0 } else { delta / max };
    (h, s, max)
}
