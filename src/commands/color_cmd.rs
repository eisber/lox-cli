//! `lox color` — encode/decode Loxone composite color values.
//!
//! Loxone represents colours in two ways:
//!
//! * **RGB composite integer** (the `<v.col>` analog colour value used by RGBW
//!   actor colour inputs): `value = red + green * 1000 + blue * 1000000` with each
//!   channel in `0..=255`. This is a base-1000 packing (NOT 24-bit `0xRRGGBB`):
//!   e.g. `100,40,0` → `40100`. This is the integer you wire/`set-param` into a
//!   numeric colour input, and the value the Miniserver expects for that input.
//! * **Tunable-white / colour-temperature**: the official ColorPickerV2 API uses
//!   the *string* command `temp(<brightness>,<kelvin>)` (brightness `0..=100`,
//!   kelvin e.g. `2700..=6500`). There is no portable composite integer for
//!   colour temperature, so this helper emits the `temp(...)` command string.
//!
//! HSV is also accepted/emitted as the `hsv(<hue>,<sat>,<val>)` string command
//! (hue `0..=360`, saturation/value `0..=100`).

use anyhow::{Result, bail};
use clap::Subcommand;
use lox_core::color::{
    ColorError, DecodedColor, EncodeInput, EncodedColor, Hsv, PercentRgb, Rgb, decode_legacy,
    encode as encode_color, format_number as fmt_num, parse_mood_legacy,
};
use serde_json::json;

use crate::commands::RunContext;

/// Parse a colour spec (`hsv(...)`, `rgb(...)`, or a raw mood/composite integer)
/// into a `LightsceneC` mood value. Used by `config set-mood-color`.
pub(crate) fn parse_color_to_mood(spec: &str) -> Result<i64> {
    parse_mood_legacy(spec)
        .map(|value| value.value())
        .map_err(color_error)
}

fn color_error(error: ColorError) -> anyhow::Error {
    match error {
        ColorError::RgbChannel { channel, value } => anyhow::anyhow!(
            "--rgb channel {}={value} out of range (0..255)",
            channel.rgb_label()
        ),
        ColorError::HsvRange { .. } => {
            anyhow::anyhow!("--hsv expects H in 0..360, S and V in 0..100")
        }
        ColorError::Brightness { value } => {
            anyhow::anyhow!("--brightness {value} out of range (0..100)")
        }
        ColorError::Kelvin { value } => {
            anyhow::anyhow!("--kelvin {value} out of range (expected ~2700..6500)")
        }
        other => other.into(),
    }
}

#[derive(Subcommand)]
pub enum ColorCmd {
    /// Encode a color to the Loxone composite value / command string
    Encode {
        /// RGB channels "R,G,B" each 0..255 (e.g. "255,100,0")
        #[arg(long)]
        rgb: Option<String>,
        /// HSV "H,S,V" — H 0..360, S/V 0..100
        #[arg(long)]
        hsv: Option<String>,
        /// Color temperature in Kelvin (tunable white, e.g. 2700)
        #[arg(long)]
        kelvin: Option<u32>,
        /// Brightness 0..100 — scales RGB, or sets brightness for --kelvin
        #[arg(long)]
        brightness: Option<f64>,
        /// Emit the LightController2 mood (LightsceneC Qn) value instead of the
        /// actor `<v.col>` composite — percent channels + 0x60000000 prefix
        #[arg(long)]
        mood: bool,
    },
    /// Decode a Loxone color value: integer composite, hsv(...) or temp(...) string
    Decode {
        /// The value to decode (e.g. "40100", "hsv(0,100,100)", "temp(15,2700)")
        value: String,
    },
}

pub fn cmd_color(ctx: &RunContext, action: ColorCmd) -> Result<()> {
    match action {
        ColorCmd::Encode {
            rgb,
            hsv,
            kelvin,
            brightness,
            mood,
        } => encode(ctx, rgb, hsv, kelvin, brightness, mood),
        ColorCmd::Decode { value } => decode(ctx, &value),
    }
}

// ── encode ───────────────────────────────────────────────────────────────────

fn encode(
    ctx: &RunContext,
    rgb: Option<String>,
    hsv: Option<String>,
    kelvin: Option<u32>,
    brightness: Option<f64>,
    mood: bool,
) -> Result<()> {
    let modes = rgb.is_some() as u8 + hsv.is_some() as u8 + kelvin.is_some() as u8;
    if modes == 0 {
        bail!("specify one of --rgb R,G,B | --hsv H,S,V | --kelvin K");
    }
    if modes > 1 {
        bail!("--rgb, --hsv and --kelvin are mutually exclusive");
    }
    if mood && kelvin.is_some() {
        bail!("--mood applies to RGB/HSV colors, not --kelvin (tunable white)");
    }

    let input = if let Some(kelvin) = kelvin {
        EncodeInput::Temperature { kelvin }
    } else if let Some(rgb) = rgb {
        EncodeInput::Rgb(Rgb::parse(&rgb).map_err(color_error)?)
    } else {
        EncodeInput::Hsv(Hsv::parse(hsv.as_deref().unwrap()).map_err(color_error)?)
    };
    let rgb = match encode_color(input, brightness).map_err(color_error)? {
        EncodedColor::Rgb(rgb) => rgb,
        EncodedColor::Temperature(temperature) => {
            let kelvin = temperature.kelvin();
            let brightness = temperature.brightness();
            let cmd = temperature.command();
            if ctx.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "mode": "temperature",
                        "kelvin": kelvin,
                        "brightness": brightness,
                        "command": cmd,
                    }))?
                );
            } else {
                println!("Tunable white — color temperature");
                println!("  kelvin:     {kelvin} K");
                println!("  brightness: {}%", fmt_num(brightness));
                println!("  command:    {cmd}");
                println!(
                    "\nNote: color temperature uses the ColorPickerV2 string command (no\n\
                 composite integer). Send it to the control's UUID, e.g.\n\
                 /jdev/sps/io/<uuid>/{cmd}"
                );
            }
            return Ok(());
        }
    };
    let Rgb {
        red: r,
        green: g,
        blue: b,
    } = rgb;
    let composite = rgb.actor_composite().value();
    let hsv_cmd = rgb.hsv_command();

    if mood {
        let mood = rgb.mood_value();
        let mood_value = mood.value();
        let PercentRgb {
            red: rp,
            green: gp,
            blue: bp,
        } = mood.percent();
        if ctx.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "mode": "mood",
                    "rgb": [r, g, b],
                    "percent": [rp, gp, bp],
                    "mood_value": mood_value,
                    "command": hsv_cmd,
                }))?
            );
        } else {
            println!("LightController2 mood value (LightsceneC Qn)");
            println!("  rgb:        {r},{g},{b}");
            println!("  percent:    {rp},{gp},{bp}");
            println!("  mood_value: {mood_value}   ← set-mood-color writes this into Qn");
            println!("  command:    {hsv_cmd}");
            println!(
                "\nNote: this is the mood packing (percent + 0x60000000), NOT the\n\
                 actor <v.col> composite ({composite}). Use `config set-mood-color`."
            );
        }
        return Ok(());
    }

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "mode": "rgb",
                "rgb": [r, g, b],
                "hex": format!("#{r:02X}{g:02X}{b:02X}"),
                "composite": composite,
                "command": hsv_cmd,
            }))?
        );
    } else {
        println!("RGB color");
        println!("  rgb:       {r},{g},{b}");
        println!("  hex:       #{r:02X}{g:02X}{b:02X}");
        println!("  composite: {composite}   ← wire/set-param this integer (<v.col>)");
        println!("  command:   {hsv_cmd}");
    }
    Ok(())
}

// ── decode ───────────────────────────────────────────────────────────────────

fn decode(ctx: &RunContext, value: &str) -> Result<()> {
    let v = value.trim();

    match decode_legacy(value).map_err(color_error)? {
        DecodedColor::Hsv { hsv, rgb } => {
            let Hsv {
                hue: h,
                saturation: s,
                value: val,
            } = hsv;
            let Rgb {
                red: r,
                green: g,
                blue: b,
            } = rgb;
            let composite = rgb.actor_composite().value();
            if ctx.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "mode": "hsv",
                        "hsv": [h, s, val],
                        "rgb": [r, g, b],
                        "hex": format!("#{r:02X}{g:02X}{b:02X}"),
                        "composite": composite,
                    }))?
                );
            } else {
                println!("HSV color");
                println!("  hsv:       {}", v);
                println!("  rgb:       {r},{g},{b}");
                println!("  hex:       #{r:02X}{g:02X}{b:02X}");
                println!("  composite: {composite}");
            }
            Ok(())
        }
        DecodedColor::Temperature { brightness, kelvin } => {
            if ctx.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "mode": "temperature",
                        "brightness": brightness,
                        "kelvin": kelvin,
                    }))?
                );
            } else {
                println!("Tunable white — color temperature");
                println!("  brightness: {}%", fmt_num(brightness));
                println!("  kelvin:     {} K", fmt_num(kelvin));
            }
            Ok(())
        }
        DecodedColor::Mood(mood) => {
            let n = mood.value();
            let PercentRgb {
                red: rp,
                green: gp,
                blue: bp,
            } = mood.percent();
            let hsv_cmd = mood.approximate_rgb().hsv_command();
            if ctx.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "mode": "mood",
                        "mood_value": n,
                        "percent": [rp, gp, bp],
                        "command": hsv_cmd,
                    }))?
                );
            } else {
                println!("LightController2 mood value {n}");
                println!("  percent: {rp},{gp},{bp}");
                println!("  command: {hsv_cmd}");
            }
            Ok(())
        }
        DecodedColor::Rgb(actor) => {
            let n = actor.value();
            let Rgb {
                red: r,
                green: g,
                blue: b,
            } = actor.rgb();
            let hsv_cmd = actor.rgb().hsv_command();
            if ctx.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "mode": "rgb",
                        "composite": n,
                        "rgb": [r, g, b],
                        "hex": format!("#{r:02X}{g:02X}{b:02X}"),
                        "command": hsv_cmd,
                    }))?
                );
            } else {
                println!("RGB composite {n}");
                println!("  rgb:     {r},{g},{b}");
                println!("  hex:     #{r:02X}{g:02X}{b:02X}");
                println!("  command: {hsv_cmd}");
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_to_mood_accepts_forms() {
        assert_eq!(
            parse_color_to_mood("hsv(120,100,10)").unwrap(),
            1_610_622_736
        );
        assert_eq!(parse_color_to_mood("rgb(0,26,0)").unwrap(), 1_610_622_736);
        // raw mood value passes through
        assert_eq!(parse_color_to_mood("1610622736").unwrap(), 1_610_622_736);
    }

    #[test]
    fn raw_mood_adapter_preserves_error_text_without_sources() {
        for (spec, expected) in [
            ("rgb(256,0,0)", "--rgb channel R=256 out of range (0..255)"),
            (
                "temp(1,2700)",
                "'temp(1,2700)' is not hsv(...), rgb(...), or an integer mood value",
            ),
            ("hsv(no,0,0)", "'no' is not a number (hsv(H,S,V))"),
        ] {
            let error = parse_color_to_mood(spec).unwrap_err();
            assert_eq!(error.to_string(), expected);
            assert_eq!(error.chain().count(), 1);
        }
    }
}
