//! `lox live` — thin, read-only-by-default helpers over the Miniserver's
//! `jdev/sps/io` HTTP API.
//!
//! These exist so live verification (does the Virtual Input echo? what is the
//! actor showing? what is the Miniserver clock?) does not require hand-rolled
//! `curl`/`Invoke-WebRequest` with basic-auth and `--SkipCertificateCheck`.
//! Reads are unrestricted; writes are gated behind `--write`.
//!
//! Note: color/composite (`<v.col>`) states return the literal template string
//! over HTTP polling, not a number — perceived color cannot be read back here
//! (it needs the websocket/visu channel), so color verification stays a
//! human-in-the-loop step.

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use serde_json::{Value, json};

use crate::client::LoxClient;
use crate::commands::RunContext;
use crate::config::Config;

#[derive(Subcommand)]
pub enum LiveCmd {
    /// Read a control/connector state: GET /jdev/sps/io/<uuid>
    Get {
        /// Control UUID (or name resolvable via the structure cache)
        uuid: String,
    },
    /// Write a value/command to a control: GET /jdev/sps/io/<uuid>/<value>
    Set {
        /// Control UUID (or name resolvable via the structure cache)
        uuid: String,
        /// Value or command string (e.g. "1", "0", "on", "hsv(120,100,30)")
        value: String,
        /// Required to perform the write (live, mutating operation)
        #[arg(long)]
        write: bool,
    },
    /// Show the Miniserver date and time
    Time,
}

pub fn cmd_live(ctx: &RunContext, action: LiveCmd) -> Result<()> {
    let cfg = Config::load()?;
    let mut lox = LoxClient::new(cfg)?;
    match action {
        LiveCmd::Get { uuid } => get(ctx, &mut lox, &uuid),
        LiveCmd::Set { uuid, value, write } => set(ctx, &mut lox, &uuid, &value, write),
        LiveCmd::Time => time(ctx, &lox),
    }
}

/// Extract `(value, code)` from a Loxone `{"LL": {...}}` envelope.
fn parse_ll(v: &Value) -> (String, Option<String>) {
    let ll = &v["LL"];
    let value = ll["value"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| ll["value"].as_f64().map(|f| f.to_string()))
        .unwrap_or_default();
    let code = ll["Code"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| ll["code"].as_str().map(|s| s.to_string()))
        .or_else(|| ll["Code"].as_i64().map(|n| n.to_string()));
    (value, code)
}

fn get(ctx: &RunContext, lox: &mut LoxClient, name_or_uuid: &str) -> Result<()> {
    let uuid = lox.resolve(name_or_uuid)?;
    let resp = lox
        .get_json(&format!("/jdev/sps/io/{}", uuid))
        .with_context(|| format!("reading state of {uuid}"))?;
    let (value, code) = parse_ll(&resp);
    if ctx.json {
        println!("{}", json!({ "uuid": uuid, "value": value, "code": code }));
    } else {
        println!("{value}");
        if value.contains("<v.col>") || value.contains("<v>") {
            eprintln!(
                "note: this is a color/template state — HTTP polling returns the literal \
                 template, not a number. Perceived color is not readable here."
            );
        }
    }
    Ok(())
}

fn set(
    ctx: &RunContext,
    lox: &mut LoxClient,
    name_or_uuid: &str,
    value: &str,
    write: bool,
) -> Result<()> {
    if !write {
        bail!(
            "refusing to write to the live Miniserver without --write \
             (would send '{value}' to '{name_or_uuid}')"
        );
    }
    let uuid = lox.resolve(name_or_uuid)?;
    let resp = lox
        .send_cmd(&uuid, value)
        .with_context(|| format!("writing '{value}' to {uuid}"))?;
    let (ret, code) = parse_ll(&resp);
    if ctx.json {
        println!(
            "{}",
            json!({ "uuid": uuid, "sent": value, "value": ret, "code": code })
        );
    } else if !ctx.quiet {
        println!(
            "✓ {uuid} ← {value}  (Code {})",
            code.as_deref().unwrap_or("?")
        );
        if !ret.is_empty() {
            println!("  value: {ret}");
        }
    }
    Ok(())
}

fn time(ctx: &RunContext, lox: &LoxClient) -> Result<()> {
    let date = parse_ll(&lox.get_json("/jdev/sys/date")?).0;
    let time = parse_ll(&lox.get_json("/jdev/sys/time")?).0;
    if ctx.json {
        println!("{}", json!({ "date": date, "time": time }));
    } else {
        println!("{date} {time}");
    }
    Ok(())
}
