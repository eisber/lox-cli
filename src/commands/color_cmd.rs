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
use serde_json::json;

use crate::commands::RunContext;

/// Maximum valid RGB composite (`255 + 255*1000 + 255*1000000`).
const RGB_MAX: i64 = 255 + 255 * 1_000 + 255 * 1_000_000;

/// Mood-type prefix for a `LightController2` `LightsceneC` per-output value
/// (`Q1..Qn`). A mood value packs **percent** channels (`0..=100`) plus this
/// prefix — distinct from the actor `<v.col>` composite which uses `0..=255`.
const MOOD_PREFIX: i64 = 0x6000_0000;

/// Pack an RGB triple (each channel `0..=255`) into the Loxone analog colour
/// value: `red + green * 1000 + blue * 1000000`.
fn rgb_to_composite(r: u8, g: u8, b: u8) -> i64 {
    r as i64 + (g as i64) * 1_000 + (b as i64) * 1_000_000
}

/// Channel value `0..=255` → percent `0..=100`.
fn chan_to_pct(c: u8) -> i64 {
    ((c as f64 / 255.0) * 100.0).round() as i64
}

/// Pack an RGB triple (each channel `0..=255`) into a `LightsceneC` mood value:
/// `0x60000000 + (R% + G%*1000 + B%*1000000)` with each channel in **percent**.
fn rgb_to_mood(r: u8, g: u8, b: u8) -> i64 {
    MOOD_PREFIX + chan_to_pct(r) + chan_to_pct(g) * 1_000 + chan_to_pct(b) * 1_000_000
}

/// Unpack a mood value into percent channels `(r%, g%, b%)`.
fn mood_to_pct(n: i64) -> Result<(i64, i64, i64)> {
    let v = n - MOOD_PREFIX;
    let r = v % 1_000;
    let g = (v / 1_000) % 1_000;
    let b = (v / 1_000_000) % 1_000;
    for (name, c) in [("red", r), ("green", g), ("blue", b)] {
        if !(0..=100).contains(&c) {
            bail!("mood value {n} has {name} channel {c}% outside 0..100 — not a valid mood color");
        }
    }
    Ok((r, g, b))
}

/// Parse a colour spec (`hsv(...)`, `rgb(...)`, or a raw mood/composite integer)
/// into a `LightsceneC` mood value. Used by `config set-mood-color`.
pub(crate) fn parse_color_to_mood(spec: &str) -> Result<i64> {
    let s = spec.trim();
    if let Some(args) = strip_call(s, "hsv") {
        let (h, sat, val) = parse_triple(&args, "hsv(H,S,V)")?;
        let (r, g, b) = hsv_to_rgb(h, sat / 100.0, val / 100.0);
        return Ok(rgb_to_mood(r, g, b));
    }
    if let Some(args) = strip_call(s, "rgb") {
        let (r, g, b) = parse_rgb(&args)?;
        return Ok(rgb_to_mood(r, g, b));
    }
    let n: i64 = s.parse().map_err(|_| {
        anyhow::anyhow!("'{s}' is not hsv(...), rgb(...), or an integer mood value")
    })?;
    if n >= MOOD_PREFIX {
        // Already a mood value — validate and pass through.
        mood_to_pct(n)?;
        Ok(n)
    } else {
        // Treat as an actor composite (0..255 channels) and convert to mood.
        let (r, g, b) = composite_to_rgb(n)?;
        Ok(rgb_to_mood(r, g, b))
    }
}

/// Unpack a Loxone analog colour value into `(red, green, blue)` channels.
/// Returns an error if any channel is outside `0..=255`.
fn composite_to_rgb(n: i64) -> Result<(u8, u8, u8)> {
    let r = n % 1_000;
    let g = (n / 1_000) % 1_000;
    let b = (n / 1_000_000) % 1_000;
    for (name, c) in [("red", r), ("green", g), ("blue", b)] {
        if !(0..=255).contains(&c) {
            bail!("composite {n} has {name} channel {c} outside 0..255 — not a valid RGB value");
        }
    }
    Ok((r as u8, g as u8, b as u8))
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

    if let Some(kelvin) = kelvin {
        let brightness = brightness.unwrap_or(100.0);
        validate_brightness(brightness)?;
        if !(1000..=15000).contains(&kelvin) {
            bail!("--kelvin {kelvin} out of range (expected ~2700..6500)");
        }
        let cmd = format!("temp({},{})", fmt_num(brightness), kelvin);
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

    // RGB or HSV → a concrete (r,g,b) triple in 0..=255.
    let (mut r, mut g, mut b) = if let Some(rgb) = rgb {
        parse_rgb(&rgb)?
    } else {
        let (h, s, v) = parse_triple(hsv.as_deref().unwrap(), "H,S,V")?;
        if !(0.0..=360.0).contains(&h) || !(0.0..=100.0).contains(&s) || !(0.0..=100.0).contains(&v)
        {
            bail!("--hsv expects H in 0..360, S and V in 0..100");
        }
        hsv_to_rgb(h, s / 100.0, v / 100.0)
    };

    if let Some(brightness) = brightness {
        validate_brightness(brightness)?;
        let scale = brightness / 100.0;
        r = (r as f64 * scale).round() as u8;
        g = (g as f64 * scale).round() as u8;
        b = (b as f64 * scale).round() as u8;
    }

    let composite = rgb_to_composite(r, g, b);
    let (h, s, v) = rgb_to_hsv(r, g, b);
    let hsv_cmd = format!(
        "hsv({},{},{})",
        h.round() as i64,
        (s * 100.0).round() as i64,
        (v * 100.0).round() as i64
    );

    if mood {
        let mood_value = rgb_to_mood(r, g, b);
        let (rp, gp, bp) = mood_to_pct(mood_value)?;
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

    // String command forms.
    if let Some(args) = strip_call(v, "hsv") {
        let (h, s, val) = parse_triple(&args, "hsv(H,S,V)")?;
        let (r, g, b) = hsv_to_rgb(h, s / 100.0, val / 100.0);
        let composite = rgb_to_composite(r, g, b);
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
        return Ok(());
    }
    if let Some(args) = strip_call(v, "temp").or_else(|| strip_call(v, "lumitech")) {
        let (brightness, kelvin) = parse_pair(&args, "temp(brightness,kelvin)")?;
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
        return Ok(());
    }

    // Numeric composite.
    let n: i64 = v
        .parse()
        .map_err(|_| anyhow::anyhow!("'{v}' is not a number or a hsv(...)/temp(...) command"))?;
    // Mood value (LightsceneC Qn): percent channels + 0x60000000 prefix.
    if n >= MOOD_PREFIX {
        let (rp, gp, bp) = mood_to_pct(n)?;
        // Reconstruct an approximate hsv from the percent channels.
        let (r, g, b) = (
            ((rp as f64 / 100.0) * 255.0).round() as u8,
            ((gp as f64 / 100.0) * 255.0).round() as u8,
            ((bp as f64 / 100.0) * 255.0).round() as u8,
        );
        let (h, s, val) = rgb_to_hsv(r, g, b);
        let hsv_cmd = format!(
            "hsv({},{},{})",
            h.round() as i64,
            (s * 100.0).round() as i64,
            (val * 100.0).round() as i64
        );
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
        return Ok(());
    }
    if (0..=RGB_MAX).contains(&n) {
        let (r, g, b) = composite_to_rgb(n)?;
        let (h, s, val) = rgb_to_hsv(r, g, b);
        let hsv_cmd = format!(
            "hsv({},{},{})",
            h.round() as i64,
            (s * 100.0).round() as i64,
            (val * 100.0).round() as i64
        );
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
    } else {
        bail!(
            "value {n} is outside the RGB composite range (0..{RGB_MAX}); color-temperature \
             values use the temp(brightness,kelvin) command string, not an integer"
        )
    }
}

// ── parsing helpers ──────────────────────────────────────────────────────────

fn parse_rgb(s: &str) -> Result<(u8, u8, u8)> {
    let (r, g, b) = parse_triple(s, "R,G,B")?;
    for (name, c) in [("R", r), ("G", g), ("B", b)] {
        if !(0.0..=255.0).contains(&c) {
            bail!("--rgb channel {name}={c} out of range (0..255)");
        }
    }
    Ok((r.round() as u8, g.round() as u8, b.round() as u8))
}

fn parse_triple(s: &str, what: &str) -> Result<(f64, f64, f64)> {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.len() != 3 {
        bail!("expected three comma-separated values ({what}), got '{s}'");
    }
    let a = parts[0].parse::<f64>().map_err(|_| bad(parts[0], what))?;
    let b = parts[1].parse::<f64>().map_err(|_| bad(parts[1], what))?;
    let c = parts[2].parse::<f64>().map_err(|_| bad(parts[2], what))?;
    Ok((a, b, c))
}

fn parse_pair(s: &str, what: &str) -> Result<(f64, f64)> {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.len() != 2 {
        bail!("expected two comma-separated values ({what}), got '{s}'");
    }
    let a = parts[0].parse::<f64>().map_err(|_| bad(parts[0], what))?;
    let b = parts[1].parse::<f64>().map_err(|_| bad(parts[1], what))?;
    Ok((a, b))
}

fn bad(part: &str, what: &str) -> anyhow::Error {
    anyhow::anyhow!("'{part}' is not a number ({what})")
}

fn strip_call(v: &str, name: &str) -> Option<String> {
    let v = v.trim();
    let lower = v.to_ascii_lowercase();
    let prefix = format!("{name}(");
    if lower.starts_with(&prefix) && v.ends_with(')') {
        Some(v[prefix.len()..v.len() - 1].to_string())
    } else {
        None
    }
}

fn validate_brightness(b: f64) -> Result<()> {
    if !(0.0..=100.0).contains(&b) {
        bail!("--brightness {b} out of range (0..100)");
    }
    Ok(())
}

fn fmt_num(n: f64) -> String {
    if (n.fract()).abs() < 1e-9 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

// ── color space conversions ──────────────────────────────────────────────────

/// HSV (h in 0..360, s/v in 0..1) → RGB (0..=255).
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

/// RGB (0..=255) → HSV (h in 0..360, s/v in 0..1).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_composite_roundtrip() {
        // value = R + G*1000 + B*1000000 (each channel 0..255)
        for (rgb, comp) in [
            ((255u8, 0u8, 0u8), 255i64),
            ((0, 255, 0), 255_000),
            ((0, 0, 255), 255_000_000),
            ((255, 255, 255), 255_255_255),
            ((0, 0, 0), 0),
            ((100, 40, 0), 40_100),
        ] {
            let (r, g, b) = rgb;
            assert_eq!(rgb_to_composite(r, g, b), comp, "rgb {rgb:?}");
            assert_eq!(composite_to_rgb(comp).unwrap(), rgb, "composite {comp}");
        }
    }

    #[test]
    fn composite_rejects_out_of_range_channel() {
        // 999 → red channel 999, which is > 255
        assert!(composite_to_rgb(999).is_err());
    }

    #[test]
    fn hsv_red_is_full_red() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), (255, 0, 0));
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), (0, 255, 0));
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), (0, 0, 255));
    }

    #[test]
    fn rgb_to_hsv_red() {
        let (h, s, v) = rgb_to_hsv(255, 0, 0);
        assert!((h - 0.0).abs() < 1e-6);
        assert!((s - 1.0).abs() < 1e-6);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn brightness_scaling_dims_channels() {
        // amber 255,100,0 scaled to 50%
        let scale = 50.0_f64 / 100.0;
        let r = (255.0 * scale).round() as u8;
        assert_eq!(r, 128);
    }

    #[test]
    fn mood_packing_matches_documented_values() {
        // green @ 10% = 0x60000000 + 10000 = 1610622736 (hsv(120,100,10))
        let (r, g, b) = hsv_to_rgb(120.0, 1.0, 0.10);
        assert_eq!(rgb_to_mood(r, g, b), 1_610_622_736);
        // round-trip percent channels
        assert_eq!(mood_to_pct(1_610_622_736).unwrap(), (0, 10, 0));
        // orange-ish R5 G2 B1 = 0x60000000 + 1002005
        assert_eq!(mood_to_pct(1_611_614_741).unwrap(), (5, 2, 1));
    }

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
    fn mood_value_rejects_out_of_range_channel() {
        // green channel 200% is invalid
        assert!(mood_to_pct(MOOD_PREFIX + 200_000).is_err());
    }
}
