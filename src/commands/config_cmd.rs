use anyhow::{Context, Result, bail};
use clap::CommandFactory;
use clap_complete::generate;
use reqwest::blocking::Client;
use std::fs;
use std::time::Duration;

use crate::client::{LOXONE_EPOCH_SECS, LoxClient, USER_AGENT};
use crate::commands::RunContext;
use crate::config::Config;
use crate::config_edit::ConfigEditor;
use crate::token;
use crate::{
    CacheCmd, Cli, ConfigCmd, ControlCmd, MqttConfigCmd, RoomCmd, SetupCmd, TokenCmd, XmlEditCmd,
    build_schema, detect_shell, ftp, gitops, install_completions, json_val_str, load_config_xml,
    loxcc, loxone_xml,
};

/// Emit a JSON object, injecting `trace_id` if set in context.
fn emit_json(ctx: &RunContext, mut obj: serde_json::Value) {
    if let Some(ref tid) = ctx.trace_id {
        obj["trace_id"] = serde_json::Value::String(tid.clone());
    }
    println!("{}", obj);
}

pub fn cmd_setup(ctx: &RunContext, action: SetupCmd) -> Result<()> {
    match action {
        SetupCmd::Set {
            host,
            user,
            pass,
            serial,
            verify_ssl,
            no_verify_ssl,
        } => {
            let mut cfg = Config::load().unwrap_or_default();
            if let Some(h) = host {
                cfg.host = if h.starts_with("http://") || h.starts_with("https://") {
                    h
                } else {
                    format!("https://{}", h)
                };
            }
            if let Some(u) = user {
                cfg.user = u;
            }
            if let Some(p) = pass {
                cfg.pass = p;
            }
            if let Some(s) = serial {
                cfg.serial = s;
            }
            if verify_ssl {
                cfg.verify_ssl = Some(true);
            } else if no_verify_ssl {
                cfg.verify_ssl = Some(false);
            }
            let path = cfg.save()?;
            if !ctx.quiet {
                eprintln!("✓  Config saved to {:?}", path);
            }
        }
        SetupCmd::Show => {
            let cfg = Config::load()?;
            if ctx.json {
                let aliases: std::collections::BTreeMap<&str, &str> = cfg
                    .aliases
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "host": cfg.host,
                        "user": cfg.user,
                        "serial": if cfg.serial.is_empty() { None } else { Some(&cfg.serial) },
                        "aliases": aliases,
                    })
                );
            } else {
                println!("host:   {}", cfg.host);
                println!("user:   {}", cfg.user);
                println!("pass:   {}", "*".repeat(cfg.pass.len()));
                if !cfg.serial.is_empty() {
                    println!("serial: {}", cfg.serial);
                }
                if !cfg.aliases.is_empty() {
                    println!("aliases:");
                    let mut aliases: Vec<_> = cfg.aliases.iter().collect();
                    aliases.sort_by_key(|(k, _)| k.as_str());
                    for (name, uuid) in aliases {
                        println!("  {}: {}", name, uuid);
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn cmd_cache(ctx: &RunContext, action: CacheCmd) -> Result<()> {
    let cfg = Config::load()?;
    let cache = LoxClient::cache_path(&cfg);
    match action {
        CacheCmd::Info => {
            if cache.exists() {
                let meta = cache.metadata()?;
                let age = std::time::SystemTime::now()
                    .duration_since(meta.modified()?)
                    .unwrap_or_default();
                let size = meta.len();
                let stale = age.as_secs() >= 86400;
                if ctx.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "path": cache.to_string_lossy(),
                            "size_bytes": size,
                            "age_seconds": age.as_secs(),
                            "stale": stale,
                        })
                    );
                } else {
                    println!("Cache: {:?}", cache);
                    println!("Size:  {:.1} KB", size as f64 / 1024.0);
                    println!("Age:   {}m {}s", age.as_secs() / 60, age.as_secs() % 60);
                    if !stale {
                        println!("Status: ✓ valid ({} until refresh)", {
                            let remaining = 86400u64.saturating_sub(age.as_secs());
                            format!("{}h {}m", remaining / 3600, (remaining % 3600) / 60)
                        });
                    } else {
                        println!("Status: ⚠ stale (will refresh on next command)");
                    }
                }
            } else if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "path": cache.to_string_lossy(),
                        "size_bytes": null,
                        "age_seconds": null,
                        "stale": null,
                    })
                );
            } else {
                println!("No cache. Will be created on first command.");
            }
        }
        CacheCmd::Clear => {
            if cache.exists() {
                fs::remove_file(&cache)?;
                eprintln!("✓ Cache cleared");
            } else {
                eprintln!("No cache to clear");
            }
        }
        CacheCmd::Check => {
            let lox = LoxClient::new(cfg)?;
            let resp = lox.get_json("/jdev/sps/LoxAPPversion3")?;
            let remote_ver = resp
                .pointer("/LL/value")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "remote_version": remote_ver,
                        "cache_exists": cache.exists(),
                    })
                );
            } else {
                println!("Remote structure version: {}", remote_ver);
                if cache.exists() {
                    println!("Cache: exists at {:?}", cache);
                } else {
                    println!("Cache: not present");
                }
            }
        }
        CacheCmd::Refresh => {
            let client = Client::builder()
                .user_agent(USER_AGENT)
                .danger_accept_invalid_certs(!cfg.verify_ssl.unwrap_or(false))
                .redirect(LoxClient::same_origin_redirect_policy(&cfg.host))
                .timeout(Duration::from_secs(15))
                .build()?;
            if cache.exists() {
                let _ = fs::remove_file(&cache);
            }
            LoxClient::load_or_fetch_structure(&cfg, &client)?;
            println!("✓ Structure cache refreshed ({:?})", cache);
        }
    }
    Ok(())
}

pub fn cmd_token(ctx: &RunContext, action: TokenCmd) -> Result<()> {
    let cfg = Config::load()?;
    match action {
        TokenCmd::Fetch => {
            println!("Fetching token from Miniserver...");
            let rt = tokio::runtime::Runtime::new()?;
            match rt.block_on(token::acquire_token(&cfg)) {
                Ok(ts) => {
                    let _exp =
                        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.valid_until);
                    let days = ts.valid_until.saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ) / 86400;
                    println!("✓ Token saved to {:?}", token::TokenStore::path_for(&cfg));
                    println!("  Valid for: {} days", days);
                }
                Err(e) => bail!("Token fetch failed: {}", e),
            }
        }
        TokenCmd::Info => match token::TokenStore::load_for(&cfg) {
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let days_left = ts.valid_until.saturating_sub(now) / 86400;
                if ts.token.len() >= 12 {
                    println!(
                        "Token: {}...{}",
                        &ts.token[..8],
                        &ts.token[ts.token.len() - 4..]
                    );
                } else {
                    println!("Token: {}", ts.token);
                }
                if ts.is_valid() {
                    println!("Status: ✓ valid ({} days remaining)", days_left);
                } else {
                    println!("Status: ⚠ expired — run: lox token fetch");
                }
            }
            None => println!("No token saved. Using Basic Auth. Run: lox token fetch"),
        },
        TokenCmd::Clear => {
            let path = token::TokenStore::path_for(&cfg);
            if path.exists() {
                fs::remove_file(&path)?;
                println!("✓ Token cleared (reverting to Basic Auth)");
            } else {
                println!("No token to clear");
            }
        }
        TokenCmd::Check => {
            let ts = token::TokenStore::load_for(&cfg)
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run: lox token fetch"))?;
            let lox = LoxClient::new(cfg.clone())?;
            // Hash the token with the key for the check endpoint
            let hash = token::hash_token(&ts.token, &ts.key);
            let resp = lox.get_json(&format!("/jdev/sys/checktoken/{}/{}", hash, cfg.user))?;
            let code = resp
                .pointer("/LL/Code")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "valid": code == "200",
                        "code": code,
                    })
                );
            } else if code == "200" {
                println!("✓ Token is valid on the Miniserver");
            } else {
                println!("✗ Token is not valid (code {})", code);
            }
        }
        TokenCmd::Refresh => {
            let ts = token::TokenStore::load_for(&cfg)
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run: lox token fetch"))?;
            let lox = LoxClient::new(cfg.clone())?;
            let hash = token::hash_token(&ts.token, &ts.key);
            let resp = lox.get_json(&format!("/jdev/sys/refreshtoken/{}/{}", hash, cfg.user))?;
            let code = resp
                .pointer("/LL/Code")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if code == "200" {
                // Update the valid_until in our local store
                let valid_until = resp
                    .pointer("/LL/value")
                    .and_then(|v| v.get("validUntil"))
                    .and_then(|v| v.as_u64());
                if let Some(vu) = valid_until {
                    let unix_until = if vu > 1_500_000_000 {
                        vu
                    } else {
                        LOXONE_EPOCH_SECS.saturating_add(vu)
                    };
                    let new_ts = token::TokenStore {
                        token: ts.token,
                        key: ts.key,
                        valid_until: unix_until,
                    };
                    new_ts.save_for(&cfg)?;
                    let days = unix_until.saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ) / 86400;
                    println!("✓ Token refreshed ({} days remaining)", days);
                } else {
                    println!("✓ Token refreshed");
                }
            } else {
                bail!("Token refresh failed (code {})", code);
            }
        }
        TokenCmd::Revoke => {
            let ts = token::TokenStore::load_for(&cfg)
                .ok_or_else(|| anyhow::anyhow!("No token saved. Run: lox token fetch"))?;
            let lox = LoxClient::new(cfg.clone())?;
            let hash = token::hash_token(&ts.token, &ts.key);
            let resp = lox.get_json(&format!("/jdev/sys/killtoken/{}/{}", hash, cfg.user))?;
            let code = resp
                .pointer("/LL/Code")
                .and_then(json_val_str)
                .unwrap_or_else(|| "?".to_string());
            if code == "200" {
                // Remove local token
                let path = token::TokenStore::path_for(&cfg);
                if path.exists() {
                    fs::remove_file(&path)?;
                }
                println!("✓ Token revoked and cleared");
            } else {
                bail!("Token revoke failed (code {})", code);
            }
        }
    }
    Ok(())
}

pub fn cmd_config(ctx: &RunContext, action: ConfigCmd) -> Result<()> {
    match action {
        ConfigCmd::Ls => {
            let cfg = Config::load()?;
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                println!("No configs found on the Miniserver.");
            } else if ctx.json {
                let arr: Vec<serde_json::Value> = backups
                    .iter()
                    .map(|b| {
                        serde_json::json!({
                            "filename": b.filename,
                            "version": b.version,
                            "date": b.formatted_date(),
                            "size": b.size,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr)?);
            } else {
                println!("  {:<4} {:<8} {:<22} Size", "#", "Version", "Date");
                for (i, b) in backups.iter().enumerate() {
                    println!(
                        "  {:<4} {:<8} {:<22} {} KB{}",
                        i + 1,
                        b.version,
                        b.formatted_date(),
                        b.size / 1024,
                        if i == 0 { "  (latest)" } else { "" }
                    );
                }
            }
        }
        ConfigCmd::Download { save_as, extract } => {
            let cfg = Config::load()?;
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                bail!("No configs found on the Miniserver.");
            }
            let newest = &backups[0];
            eprintln!(
                "Downloading {} ({} KB)...",
                newest.filename,
                newest.size / 1024
            );
            let data = ftp::download_backup(&cfg, &newest.filename)?;
            let out_path = save_as.unwrap_or_else(|| newest.filename.clone());
            fs::write(&out_path, &data)?;
            println!("Saved to {}", out_path);

            if extract {
                eprintln!("Extracting sps0.LoxCC...");
                let xml = loxcc::extract_and_decompress(&data)?;
                let xml_path = out_path
                    .strip_suffix(".zip")
                    .unwrap_or(&out_path)
                    .to_string()
                    + ".Loxone";
                fs::write(&xml_path, &xml)?;
                println!(
                    "Decompressed {} KB → {} KB → {}",
                    data.len() / 1024,
                    xml.len() / 1024,
                    xml_path
                );
            }
        }
        ConfigCmd::Extract { file, save_as } => {
            let zip_data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            eprintln!("Extracting sps0.LoxCC...");
            let xml = loxcc::extract_and_decompress(&zip_data)?;
            let xml_path = save_as.unwrap_or_else(|| {
                file.strip_suffix(".zip").unwrap_or(&file).to_string() + ".Loxone"
            });
            fs::write(&xml_path, &xml)?;
            println!(
                "Decompressed {} KB → {} KB → {}",
                zip_data.len() / 1024,
                xml.len() / 1024,
                xml_path
            );
        }
        ConfigCmd::Upload { file, force } => {
            let cfg = Config::load()?;
            if !force {
                if ctx.non_interactive {
                    bail!("Destructive operation requires --force flag");
                }
                eprintln!(
                    "⚠  WARNING: Uploading a config will replace the current Miniserver\n\
                     \x20  programming. A bad configuration can require physical SD card\n\
                     \x20  access to recover.\n\
                     \n\
                     \x20  Config file: {}\n\
                     \n\
                     \x20  Use --force to proceed.",
                    file
                );
                bail!("Destructive operation requires --force flag");
            }
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let filename = std::path::Path::new(&file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file);
            eprintln!("Uploading {} ({} KB)...", filename, data.len() / 1024);
            ftp::upload_backup(&cfg, filename, &data)?;
            println!("Upload complete.");
            println!("Reboot the Miniserver to apply: lox reboot");
        }
        ConfigCmd::Users { file, limit } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let users = loxone_xml::parse_users(&xml)?;
            let total = users.len();
            let truncated = limit.is_some_and(|l| total > l);
            let display_limit = limit.unwrap_or(total);
            if ctx.json {
                let shown = total.min(display_limit);
                let items: Vec<_> = users.iter().take(display_limit).collect();
                let mut obj = serde_json::to_value(&items)?;
                if truncated {
                    let wrapper = serde_json::json!({
                        "users": obj,
                        "truncated": true,
                        "total": total,
                        "shown": shown,
                    });
                    obj = wrapper;
                }
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                let nfc_count = users.iter().filter(|u| u.nfc).count();
                println!("  {:<26} {:<6} Description", "Name", "NFC");
                for u in users.iter().take(display_limit) {
                    println!(
                        "  {:<26} {:<6} {}",
                        u.name,
                        if u.nfc { "yes" } else { "-" },
                        u.description,
                    );
                }
                println!("\n{} users ({} with NFC)", total, nfc_count);
            }
            if truncated {
                eprintln!(
                    "Showing {} of {}. Use --limit to see more.",
                    display_limit, total
                );
            }
        }
        ConfigCmd::Devices {
            file,
            ports,
            room,
            summary,
            limit,
        } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;

            if ports {
                let editor = ConfigEditor::load(&xml)?;
                print!("{}", editor.list_device_ports());
            } else if summary || ctx.json || room.is_some() {
                let editor = ConfigEditor::load(&xml)?;
                let devices = editor.config_devices(room.as_deref());
                let total = devices.len();
                let with_bus_address = devices
                    .iter()
                    .filter(|d| d.bus_address.as_ref().is_some_and(|a| !a.is_empty()))
                    .count();
                let standalone = devices
                    .iter()
                    .filter(|d| d.bus_type == "standalone")
                    .count();
                let low_confidence = devices.iter().filter(|d| d.low_confidence_identity).count();
                if summary {
                    eprintln!(
                        "{} devices, {} with bus_address, {} standalone, {} low_confidence",
                        total, with_bus_address, standalone, low_confidence
                    );
                } else if ctx.json {
                    println!("{}", serde_json::to_string_pretty(&devices)?);
                } else if devices.is_empty() {
                    println!("No devices found.");
                } else {
                    for device in devices.iter().take(limit.unwrap_or(devices.len())) {
                        println!(
                            "{} ({}) [{} {}]",
                            device.derived_label,
                            device.device_type,
                            device.bus_type,
                            device.bus_address.as_deref().unwrap_or("-")
                        );
                    }
                    println!("\n{} devices total", total);
                }
            } else {
                let devices = loxone_xml::parse_devices(&xml)?;
                let total = devices.len();
                let truncated = limit.is_some_and(|l| total > l);
                let display_limit = limit.unwrap_or(total);
                if ctx.json {
                    let shown = total.min(display_limit);
                    let items: Vec<_> = devices.iter().take(display_limit).collect();
                    let mut obj = serde_json::to_value(&items)?;
                    if truncated {
                        let wrapper = serde_json::json!({
                            "devices": obj,
                            "truncated": true,
                            "total": total,
                            "shown": shown,
                        });
                        obj = wrapper;
                    }
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                } else {
                    let tree: Vec<_> = devices
                        .iter()
                        .filter(|d| d.bus == loxone_xml::DeviceBus::Tree)
                        .take(display_limit)
                        .collect();
                    let remaining = display_limit.saturating_sub(tree.len());
                    let air: Vec<_> = devices
                        .iter()
                        .filter(|d| d.bus == loxone_xml::DeviceBus::Air)
                        .take(remaining)
                        .collect();
                    let remaining = remaining.saturating_sub(air.len());
                    let net: Vec<_> = devices
                        .iter()
                        .filter(|d| d.bus == loxone_xml::DeviceBus::Network)
                        .take(remaining)
                        .collect();

                    if !tree.is_empty() {
                        println!("  Tree devices ({})", tree.len());
                        println!("  {:<30} {:<12} Type", "Name", "Serial");
                        for d in &tree {
                            println!(
                                "  {:<30} {:<12} {}",
                                d.name,
                                d.serial.as_deref().unwrap_or("-"),
                                d.type_label
                            );
                        }
                    }
                    if !air.is_empty() {
                        if !tree.is_empty() {
                            println!();
                        }
                        println!("  LoxAIR devices ({})", air.len());
                        println!("  {:<30} Type", "Name");
                        for d in &air {
                            println!("  {:<30} {}", d.name, d.type_label);
                        }
                    }
                    if !net.is_empty() {
                        if !tree.is_empty() || !air.is_empty() {
                            println!();
                        }
                        println!("  Network devices ({})", net.len());
                        println!("  {:<30} {:<18} MAC", "Name", "Address");
                        for d in &net {
                            println!(
                                "  {:<30} {:<18} {}",
                                d.name,
                                d.address.as_deref().unwrap_or("-"),
                                d.mac.as_deref().unwrap_or("-")
                            );
                        }
                    }
                    println!("\n{} devices total", total);
                }
                if truncated {
                    eprintln!(
                        "Showing {} of {}. Use --limit to see more.",
                        display_limit, total
                    );
                }
            }
        }
        ConfigCmd::Diff { file1, file2 } => {
            let xml1 = load_config_xml(&file1)?;
            let xml2 = load_config_xml(&file2)?;
            let s1 = loxone_xml::parse_config_summary(&xml1)?;
            let s2 = loxone_xml::parse_config_summary(&xml2)?;
            let diff = loxone_xml::diff_configs(&s1, &s2);

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&diff)?);
            } else {
                println!(
                    "Config version: {} → {}",
                    diff.version_old, diff.version_new
                );
                println!("Modified: {} → {}", diff.date_old, diff.date_new);

                if !diff.controls_added.is_empty()
                    || !diff.controls_removed.is_empty()
                    || !diff.controls_changed.is_empty()
                {
                    println!("\nControls:");
                    for c in &diff.controls_added {
                        println!("  + Added: \"{}\" ({})", c.name, c.control_type);
                    }
                    for c in &diff.controls_changed {
                        println!(
                            "  ~ Changed: \"{}\" — {} \"{}\" → \"{}\"",
                            c.name, c.field, c.old_value, c.new_value
                        );
                    }
                    for c in &diff.controls_removed {
                        println!("  - Removed: \"{}\" ({})", c.name, c.control_type);
                    }
                }

                if !diff.rooms_added.is_empty()
                    || !diff.rooms_removed.is_empty()
                    || !diff.rooms_renamed.is_empty()
                {
                    println!("\nRooms:");
                    for r in &diff.rooms_added {
                        println!("  + Added: \"{}\"", r);
                    }
                    for r in &diff.rooms_renamed {
                        println!("  ~ Renamed: \"{}\" → \"{}\"", r.old, r.new);
                    }
                    for r in &diff.rooms_removed {
                        println!("  - Removed: \"{}\"", r);
                    }
                }

                if !diff.categories_added.is_empty()
                    || !diff.categories_removed.is_empty()
                    || !diff.categories_renamed.is_empty()
                {
                    println!("\nCategories:");
                    for c in &diff.categories_added {
                        println!("  + Added: \"{}\"", c);
                    }
                    for c in &diff.categories_renamed {
                        println!("  ~ Renamed: \"{}\" → \"{}\"", c.old, c.new);
                    }
                    for c in &diff.categories_removed {
                        println!("  - Removed: \"{}\"", c);
                    }
                }

                if !diff.users_added.is_empty() || !diff.users_removed.is_empty() {
                    println!("\nUsers:");
                    for u in &diff.users_added {
                        println!("  + Added: \"{}\"", u);
                    }
                    for u in &diff.users_removed {
                        println!("  - Removed: \"{}\"", u);
                    }
                }

                let total = diff.controls_added.len()
                    + diff.controls_removed.len()
                    + diff.controls_changed.len()
                    + diff.rooms_added.len()
                    + diff.rooms_removed.len()
                    + diff.rooms_renamed.len()
                    + diff.categories_added.len()
                    + diff.categories_removed.len()
                    + diff.categories_renamed.len()
                    + diff.users_added.len()
                    + diff.users_removed.len();

                if !diff.has_changes() {
                    println!("\nNo structural changes.");
                } else {
                    println!("\n{} changes total", total);
                }
            }
        }
        ConfigCmd::Init { path } => {
            let cfg = Config::load()?;
            let abs_path = if path.starts_with('/') || path.starts_with('~') {
                let expanded = if path.starts_with('~') {
                    path.replacen(
                        '~',
                        &dirs::home_dir().unwrap_or_default().to_string_lossy(),
                        1,
                    )
                } else {
                    path.clone()
                };
                std::path::PathBuf::from(expanded)
            } else {
                std::env::current_dir()?.join(&path)
            };
            let repo = gitops::init(&abs_path, &cfg)?;
            // Save the repo path in config
            let mut cfg = Config::load().unwrap_or_default();
            cfg.config_repo = Some(repo.to_string_lossy().to_string());
            let saved = cfg.save()?;
            println!("Config repo initialized at {}", repo.display());
            println!("Saved repo path to {}", saved.display());
            println!("\nNext: run `lox config pull` to download and commit the current config.");
        }
        ConfigCmd::Pull { quiet: pull_quiet } => {
            let cfg = Config::load()?;
            let repo_path = cfg
                .config_repo
                .as_deref()
                .context("No config repo configured. Run `lox config init <path>` first.")?;
            let q = pull_quiet || ctx.quiet;
            let committed = gitops::pull(std::path::Path::new(repo_path), &cfg, q)?;
            if q && committed {
                // In quiet/cron mode, just indicate a commit was made
                println!("committed");
            }
        }
        ConfigCmd::Log { count } => {
            let cfg = Config::load()?;
            let repo_path = cfg
                .config_repo
                .as_deref()
                .context("No config repo configured. Run `lox config init <path>` first.")?;
            let ms = if !cfg.serial.is_empty() {
                Some(cfg.serial.as_str())
            } else {
                None
            };
            let output = gitops::log(std::path::Path::new(repo_path), ms, count)?;
            if output.is_empty() {
                println!("No config history yet. Run `lox config pull` first.");
            } else {
                println!("{}", output);
            }
        }
        ConfigCmd::Restore { commit, force } => {
            let cfg = Config::load()?;
            let repo_path = cfg
                .config_repo
                .as_deref()
                .context("No config repo configured. Run `lox config init <path>` first.")?;
            gitops::restore(std::path::Path::new(repo_path), &cfg, &commit, force)?;
        }
        ConfigCmd::Compress { file, save_as } => {
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let loxcc_data = loxcc::compress_loxcc(&xml);
            let out_path = save_as.unwrap_or_else(|| {
                file.strip_suffix(".Loxone")
                    .or_else(|| file.strip_suffix(".loxone"))
                    .unwrap_or(&file)
                    .to_string()
                    + ".LoxCC"
            });
            fs::write(&out_path, &loxcc_data)?;
            println!(
                "Compressed {} KB → {} KB → {}",
                xml.len() / 1024,
                loxcc_data.len() / 1024,
                out_path
            );
        }
        ConfigCmd::Rooms { file, limit } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let rooms = loxone_xml::parse_rooms(&xml)?;
            let total = rooms.len();
            let truncated = limit.is_some_and(|l| total > l);
            let display_limit = limit.unwrap_or(total);
            if ctx.json {
                let shown = total.min(display_limit);
                let items: Vec<_> = rooms.iter().take(display_limit).collect();
                let mut obj = serde_json::to_value(&items)?;
                if truncated {
                    let wrapper = serde_json::json!({
                        "rooms": obj,
                        "truncated": true,
                        "total": total,
                        "shown": shown,
                    });
                    obj = wrapper;
                }
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                println!("  {:<30} {:<6} UUID", "Room", "Items");
                println!(
                    "  {:<30} {:<6} {}",
                    "─".repeat(30),
                    "─".repeat(6),
                    "─".repeat(36)
                );
                for r in rooms.iter().take(display_limit) {
                    println!("  {:<30} {:<6} {}", r.name, r.item_count, r.uuid);
                }
                let total_items: usize = rooms.iter().map(|r| r.item_count).sum();
                println!("\n{} rooms, {} items total", total, total_items);
            }
            if truncated {
                eprintln!(
                    "Showing {} of {}. Use --limit to see more.",
                    display_limit, total
                );
            }
        }
        ConfigCmd::Controls {
            file,
            r#type,
            room,
            limit,
        } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let controls = loxone_xml::parse_controls(&xml, r#type.as_deref(), room.as_deref())?;
            let total = controls.len();
            let truncated = total > limit;
            if ctx.json {
                let shown = total.min(limit);
                let items: Vec<_> = controls.iter().take(limit).collect();
                let mut obj = serde_json::to_value(&items)?;
                if truncated {
                    let wrapper = serde_json::json!({
                        "controls": obj,
                        "truncated": true,
                        "total": total,
                        "shown": shown,
                    });
                    obj = wrapper;
                }
                println!("{}", serde_json::to_string_pretty(&obj)?);
            } else {
                println!(
                    "  {:<20} {:<30} {:<20} {:<20} UUID",
                    "Type", "Title", "Room", "Category"
                );
                println!(
                    "  {:<20} {:<30} {:<20} {:<20} {}",
                    "─".repeat(20),
                    "─".repeat(30),
                    "─".repeat(20),
                    "─".repeat(20),
                    "─".repeat(36)
                );
                for c in controls.iter().take(limit) {
                    println!(
                        "  {:<20} {:<30} {:<20} {:<20} {}",
                        c.control_type, c.title, c.room, c.category, c.uuid
                    );
                }
                println!("\n{} controls", total);
            }
            if truncated {
                eprintln!(
                    "Showing {} of {}. Use --limit to see more or --type/--room to filter.",
                    limit, total
                );
            }
        }
        ConfigCmd::Patch {
            replace,
            reboot,
            force,
        } => {
            if !force {
                if ctx.non_interactive {
                    bail!("Destructive operation requires --force flag");
                }
                eprintln!(
                    "⚠  WARNING: This will modify the live Miniserver configuration.\n\
                     \n\
                     \x20  Use --force to proceed."
                );
                bail!("Destructive operation requires --force flag");
            }
            if replace.len() % 2 != 0 {
                bail!("--replace requires pairs of OLD NEW values");
            }
            let cfg = Config::load()?;

            // Download current config
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                bail!("No configs found on the Miniserver.");
            }
            let newest = &backups[0];
            eprintln!("Downloading {}...", newest.filename);
            let zip_data = ftp::download_backup(&cfg, &newest.filename)?;

            // Extract XML
            let xml = loxcc::extract_and_decompress(&zip_data)?;
            let mut patched = xml.clone();

            // Apply replacements
            for pair in replace.chunks(2) {
                if pair[0].is_empty() {
                    bail!("Replacement pattern must not be empty");
                }
                let old = pair[0].as_bytes();
                let new = pair[1].as_bytes();
                let count = patched.windows(old.len()).filter(|w| *w == old).count();
                if count == 0 {
                    eprintln!("  ⚠ Pattern '{}' not found in config", pair[0]);
                } else {
                    eprintln!(
                        "  ✓ Replacing '{}' → '{}' ({} occurrences)",
                        pair[0], pair[1], count
                    );
                    // Byte-level replacement
                    let mut result = Vec::with_capacity(patched.len());
                    let mut pos = 0;
                    while pos < patched.len() {
                        if pos + old.len() <= patched.len() && &patched[pos..pos + old.len()] == old
                        {
                            result.extend_from_slice(new);
                            pos += old.len();
                        } else {
                            result.push(patched[pos]);
                            pos += 1;
                        }
                    }
                    patched = result;
                }
            }

            if patched == xml {
                println!("No changes made.");
                return Ok(());
            }

            // Repack and upload
            let new_zip = loxcc::repack_zip(&zip_data, &patched)?;
            let upload_name = &newest.filename;
            eprintln!("Uploading patched config as {}...", upload_name);
            ftp::upload_backup(&cfg, upload_name, &new_zip)?;
            println!("✓ Patched config uploaded.");

            if reboot {
                eprintln!("Rebooting Miniserver...");
                let lox = crate::client::LoxClient::new(cfg)?;
                lox.get_text("/dev/sys/reboot")?;
                println!("✓ Reboot initiated.");
            } else {
                println!("Reboot the Miniserver to apply: lox reboot --yes");
            }
        }
        ConfigCmd::Push {
            file,
            reboot,
            force,
        } => {
            if !force {
                if ctx.non_interactive {
                    bail!("Destructive operation requires --force flag");
                }
                eprintln!(
                    "⚠  WARNING: This will upload a config to the live Miniserver.\n\
                     \n\
                     \x20  Use --force to proceed."
                );
                bail!("Destructive operation requires --force flag");
            }
            // Read the .Loxone XML
            let xml = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let cfg = Config::load()?;

            // Validate config before upload
            let editor = ConfigEditor::load(&xml)?;
            let issues = editor.validate_config();
            let errors: Vec<_> = issues.iter().filter(|i| i.starts_with('✗')).collect();
            let warnings: Vec<_> = issues.iter().filter(|i| i.starts_with('⚠')).collect();

            if !errors.is_empty() {
                eprintln!("Config validation failed:");
                for issue in &issues {
                    eprintln!("  {}", issue);
                }
                bail!("{} error(s) found. Fix before uploading.", errors.len());
            }
            if !warnings.is_empty() {
                for w in &warnings {
                    eprintln!("  {}", w);
                }
            }

            // Download current backup ZIP as a template (for LoxAPP3.json, permissions.bin, etc.)
            let backups = ftp::list_backups(&cfg)?;
            if backups.is_empty() {
                bail!("No configs found on the Miniserver to use as template.");
            }
            let newest = &backups[0];
            eprintln!("Downloading {} as template...", newest.filename);
            let template_zip = ftp::download_backup(&cfg, &newest.filename)?;

            // Resolve stale InputRef wiring: UX regenerates connector UUIDs on save,
            // so our wiring may point to old UUIDs. Remap to current ones from template.
            let live_xml = loxcc::extract_and_decompress(&template_zip)?;
            let mut editor_for_wiring = ConfigEditor::load(&xml)?;
            let resolved = editor_for_wiring.resolve_wiring(&live_xml)?;
            let xml = if resolved > 0 {
                eprintln!("✓ Resolved {} stale wiring reference(s)", resolved);
                editor_for_wiring.to_bytes()?
            } else {
                xml
            };

            // Repack with our edited XML
            let new_zip = loxcc::repack_zip(&template_zip, &xml)?;

            // Upload as sps_new.zip (consumed by Miniserver on reload)
            eprintln!("Uploading patched config ({} KB)...", new_zip.len() / 1024);
            ftp::upload_backup(&cfg, "sps_new.zip", &new_zip)?;

            if reboot {
                // Fast reload via /wsx 0x3A → 0x05 (~4s)
                eprintln!("Triggering fast SPS reload via /wsx...");
                match crate::ws::trigger_fast_reload(&cfg) {
                    Ok(()) => {
                        if ctx.json {
                            emit_json(
                                ctx,
                                serde_json::json!({
                                    "ok": true,
                                    "file": file,
                                    "uploaded_as": "sps_new.zip",
                                    "reboot": "fast_reload",
                                }),
                            );
                        } else {
                            println!("✓ Config uploaded as sps_new.zip");
                            println!("✓ SPS reloading (~4s)");
                        }
                    }
                    Err(e) => {
                        eprintln!("Fast reload failed ({}), falling back to reboot...", e);
                        let lox = crate::client::LoxClient::new(cfg)?;
                        lox.get_text("/dev/sys/reboot")?;
                        if ctx.json {
                            emit_json(
                                ctx,
                                serde_json::json!({
                                    "ok": true,
                                    "file": file,
                                    "uploaded_as": "sps_new.zip",
                                    "reboot": "full",
                                }),
                            );
                        } else {
                            println!("✓ Config uploaded as sps_new.zip");
                            println!("✓ Reboot initiated (~60s).");
                        }
                    }
                }
            } else if ctx.json {
                emit_json(
                    ctx,
                    serde_json::json!({
                        "ok": true,
                        "file": file,
                        "uploaded_as": "sps_new.zip",
                        "reboot": false,
                    }),
                );
            } else {
                println!("✓ Config uploaded as sps_new.zip");
                println!(
                    "Apply with: lox config push --file {} --reboot --force",
                    file
                );
            }
        }
        ConfigCmd::PushHttp { file, force } => {
            if !force {
                if ctx.non_interactive {
                    bail!("Destructive operation requires --force flag");
                }
                eprintln!(
                    "⚠  WARNING: This will upload a config ZIP to the live Miniserver via HTTP.\n\
                     \n\
                     \x20  The Miniserver will auto-restart after receiving the file.\n\
                     \x20  Use --force to proceed."
                );
                bail!("Destructive operation requires --force flag");
            }

            let zip_data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            eprintln!("Read {} ({} KB)", file, zip_data.len() / 1024);

            let cfg = Config::load()?;
            let verify_ssl = cfg.verify_ssl.unwrap_or(false);

            // Build a client with cookie store for session persistence across requests.
            let client = Client::builder()
                .user_agent(USER_AGENT)
                .danger_accept_invalid_certs(!verify_ssl)
                .cookie_store(true)
                .timeout(Duration::from_secs(120))
                .build()
                .context("failed to build HTTP client")?;

            // Use HTTPS for all requests (fsput requires it)
            let https_host = cfg.host.replace("http://", "https://");
            eprintln!("Authenticating via HTTPS...");
            let _pub_key: serde_json::Value = client
                .get(format!("{}/jdev/sys/getPublicKey", https_host))
                .send()
                .context("getPublicKey request failed")?
                .json()
                .context("getPublicKey parse failed")?;

            // Step 2: Get key2 (HMAC key + salt + hash algorithm)
            let key2_resp: serde_json::Value = client
                .get(format!("{}/jdev/sys/getkey2/{}", https_host, cfg.user))
                .send()
                .context("getkey2 request failed")?
                .json()
                .context("getkey2 parse failed")?;
            let key2_val_raw = key2_resp
                .pointer("/LL/value")
                .ok_or_else(|| anyhow::anyhow!("getkey2: no value in response"))?;
            // value can be a JSON string (needs parsing) or already an object
            let key2_val: serde_json::Value = if let Some(s) = key2_val_raw.as_str() {
                serde_json::from_str(s).context("getkey2: failed to parse value JSON")?
            } else {
                key2_val_raw.clone()
            };
            let key_hex = key2_val
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("getkey2: missing 'key'"))?;
            let salt_hex = key2_val
                .get("salt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("getkey2: missing 'salt'"))?;

            // Step 3: Compute pwHash = SHA256("{pass}:{salt}").toUpperCase()
            // Note: salt is used as the raw hex string, NOT hex-decoded
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            let pw_hash = format!(
                "{:X}",
                <Sha256 as sha2::Digest>::digest(format!("{}:{}", cfg.pass, salt_hex).as_bytes())
            );

            // Step 4: sig = HMAC-SHA256(hex_decode(key), "{user}:{pwHash}") → lowercase hex
            let key_bytes = hex::decode(key_hex).context("getkey2: failed to hex-decode key")?;
            let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).context("HMAC key error")?;
            mac.update(format!("{}:{}", cfg.user, pw_hash).as_bytes());
            let sig = hex::encode(mac.finalize().into_bytes());

            // Step 5: Get JWT token with permission 8 (CONFIG)
            let client_uuid = uuid::Uuid::new_v4().to_string();
            let jwt_resp: serde_json::Value = client
                .get(format!(
                    "{}/jdev/sys/getjwt/{}/{}/8/{}/lox-cli",
                    https_host, sig, cfg.user, client_uuid
                ))
                .send()
                .context("getjwt request failed")?
                .json()
                .context("getjwt parse failed")?;
            let jwt_code = jwt_resp
                .pointer("/LL/Code")
                .or_else(|| jwt_resp.pointer("/LL/code"))
                .and_then(crate::json_val_str)
                .unwrap_or_default();
            if jwt_code != "200" {
                bail!(
                    "getjwt failed (code {}): {}",
                    jwt_code,
                    serde_json::to_string_pretty(&jwt_resp)?
                );
            }
            eprintln!("✓ JWT acquired (permission=CONFIG)");

            // Extract token and key from getjwt response
            // The key from getjwt can be used like a getkey result for subsequent commands.
            let jwt_val = jwt_resp
                .pointer("/LL/value")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        serde_json::from_str::<serde_json::Value>(s).ok()
                    } else {
                        Some(v.clone())
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("getjwt: missing value"))?;
            let jwt_token = jwt_val
                .get("token")
                .and_then(|t| t.as_str())
                .ok_or_else(|| anyhow::anyhow!("getjwt: missing 'token' in response"))?
                .to_string();
            let jwt_key_hex = jwt_val
                .get("key")
                .and_then(|k| k.as_str())
                .ok_or_else(|| anyhow::anyhow!("getjwt: missing 'key' in response"))?
                .to_string();

            // Step 6: autht = HMAC-SHA1(hex_decode_to_ascii(key), token) — no case change
            // Per official protocol: key from getjwt is hex-decoded to ASCII bytes,
            // then used as HMAC-SHA1 key with the JWT token as message.
            use sha1::Sha1;
            type HmacSha1 = Hmac<Sha1>;
            let key_ascii_bytes =
                hex::decode(&jwt_key_hex).context("getjwt: failed to hex-decode key")?;
            let mut mac_autht =
                HmacSha1::new_from_slice(&key_ascii_bytes).context("HMAC-SHA1 key error")?;
            mac_autht.update(jwt_token.as_bytes());
            let autht = hex::encode(mac_autht.finalize().into_bytes());

            // Step 7: Establish /wsx WebSocket session (required before fsput)
            eprintln!("Establishing WebSocket session...");
            let wsx_url = format!(
                "{}/wsx?autht={}&user={}",
                https_host
                    .replace("https://", "wss://")
                    .replace("http://", "ws://"),
                autht,
                cfg.user
            );
            let rt = tokio::runtime::Runtime::new()?;
            let wsx_result = rt.block_on(async {
                use tokio_tungstenite::Connector;
                use tokio_tungstenite::connect_async_tls_with_config;
                let tls_cfg = crate::ws::make_tls_config_pub();
                let req = tokio_tungstenite::tungstenite::http::Request::builder()
                    .uri(&wsx_url)
                    .header(
                        "Host",
                        https_host.replace("https://", "").replace("http://", ""),
                    )
                    .header("Connection", "Upgrade")
                    .header("Upgrade", "websocket")
                    .header("Sec-WebSocket-Version", "13")
                    .header("Sec-WebSocket-Key", {
                        let mut bytes = [0u8; 16];
                        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
                        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
                    })
                    .body(())?;
                let (ws, _) = connect_async_tls_with_config(
                    req,
                    None,
                    false,
                    Some(Connector::Rustls(tls_cfg)),
                )
                .await?;
                Ok::<_, anyhow::Error>(ws)
            });
            let _wsx = match wsx_result {
                Ok(ws) => {
                    eprintln!("✓ WebSocket session established");
                    Some(ws)
                }
                Err(e) => {
                    eprintln!("⚠ WebSocket /wsx failed ({}), trying fsput anyway...", e);
                    None
                }
            };

            // Step 8: POST the ZIP via fsput
            eprintln!(
                "Uploading {} ({} KB) via HTTP POST...",
                file,
                zip_data.len() / 1024
            );
            let upload_url = format!(
                "{}/dev/fsput/lx/prog/sps_new.zip?autht={}&user={}",
                https_host, autht, cfg.user
            );
            let resp = client
                .post(&upload_url)
                .header("Content-Type", "application/octet-stream")
                .body(zip_data)
                .send()
                .context("fsput POST failed")?;
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            if !status.is_success() {
                bail!("fsput failed (HTTP {}): {}", status.as_u16(), body);
            }
            println!("✓ Config uploaded via HTTP. Miniserver will auto-restart.");
        }
        ConfigCmd::Add {
            file,
            control_type,
            title,
            room,
            category,
            parent,
            page,
            topic,
            save_as,
        } => {
            let (xml_type_owned, default_parent) = resolve_block_type(&control_type)?;
            let xml_type = xml_type_owned.as_str();

            let parent_sel = parent.as_deref().or(default_parent);
            let needs_parent = matches!(control_type.as_str(), "mqtt-sub" | "mqtt-pub");
            if needs_parent && parent_sel.is_none() {
                bail!("--parent is required for {}", control_type);
            }

            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            let cat_uuid = if let Some(ref c) = category {
                Some(editor.find_category_uuid(c)?)
            } else {
                None
            };

            let mut props: Vec<(&str, &str, &str)> = Vec::new();
            if let Some(ref t) = topic {
                props.push(("mqtt_topic", t.as_str(), "11"));
            }

            // Resolve target page: --page flag takes priority, then --room auto-detect
            let target_page_path = if let Some(ref p) = page {
                Some(editor.find_page_path(p)?)
            } else if let Some(ref r) = room {
                editor.find_page_path_for_room(r)
            } else {
                None
            };

            // Idempotency: check if a block with the same Title AND Type already exists
            let type_selector = format!("Type:{}", xml_type);
            let existing = editor.find_elements(&type_selector);
            let existing_match = existing.iter().find(|path| {
                let elem = editor.get_element(path);
                elem.attributes
                    .get("Title")
                    .map(|t| t == &title)
                    .unwrap_or(false)
            });
            if let Some(found_path) = existing_match {
                let elem = editor.get_element(found_path);
                let existing_uuid = elem.attributes.get("U").cloned().unwrap_or_default();
                if ctx.json {
                    let connectors = collect_block_connectors(&editor, &existing_uuid);
                    emit_json(
                        ctx,
                        serde_json::json!({
                            "ok": true,
                            "existing": true,
                            "uuid": existing_uuid,
                            "type": xml_type,
                            "title": title,
                            "connectors": connectors,
                        }),
                    );
                } else {
                    println!(
                        "✓ Already exists: {} '{}' (UUID: {})",
                        xml_type, title, existing_uuid
                    );
                }
                return Ok(());
            }

            if let Some(actual_parent) = parent_sel {
                // --parent explicitly provided — use it (unless --page overrides for page-bound types)
                if target_page_path.is_some() {
                    // --page with --parent: --parent wins for the element insertion,
                    // but warn that --page is ignored when --parent is explicit
                    eprintln!("note: --parent overrides --page for element placement");
                }
                let uuid = editor.add_element(
                    actual_parent,
                    xml_type,
                    &title,
                    None,
                    room_uuid.as_deref(),
                    cat_uuid.as_deref(),
                    &props,
                )?;
                if ctx.json {
                    let connectors = collect_block_connectors(&editor, &uuid);
                    emit_json(
                        ctx,
                        serde_json::json!({
                            "ok": true,
                            "uuid": uuid,
                            "type": xml_type,
                            "title": title,
                            "connectors": connectors,
                        }),
                    );
                } else {
                    println!("✓ Added {} '{}' (UUID: {})", xml_type, title, uuid);
                }
            } else {
                // Auto-detect parent: logic/math blocks go under the first Page,
                // other types go to root
                let default_parent = match xml_type {
                    "And"
                    | "Or"
                    | "Not"
                    | "Xor"
                    | "Add"
                    | "Add4"
                    | "Sub"
                    | "Mult"
                    | "Div"
                    | "Mod"
                    | "AnalogComparator"
                    | "FlipFlop"
                    | "Counter"
                    | "AMemory"
                    | "Monoflop"
                    | "Formula"
                    | "Average"
                    | "Int"
                    | "Avg"
                    | "State"
                    | "AnalogStepper"
                    | "EdgeDetection"
                    | "PulseGen"
                    | "OnPulseDelay"
                    | "PWM"
                    | "Shift"
                    | "UpDownCounter"
                    | "StatusMonitor"
                    | "AnalogThresholdTrigger"
                    | "AnalogDiffTrigger"
                    | "Minmax"
                    | "TimeMinmax"
                    | "AMinmax"
                    | "AnalogWatchdog"
                    | "Validator"
                    | "AnalogScaler"
                    | "BinDecoder"
                    | "BinEncoder"
                    | "AnalogMultiplexer2"
                    | "AnalogMultiplexer"
                    | "OnDelay"
                    | "OnOffDelay"
                    | "OffDelay"
                    | "RetOnDelay"
                    | "EdgeWipingRelay"
                    | "Rand"
                    | "RandomGen"
                    | "PulseAt"
                    | "PulseBy"
                    | "Equal"
                    | "Greater"
                    | "Less"
                    | "GreaterEqual"
                    | "LessEqual"
                    | "NotEqual"
                    | "RSFlipFlop"
                    | "SRFlipFlop"
                    | "StairwayLS"
                    | "Ramp"
                    | "LongClick"
                    | "MultiClick"
                    | "PButtonT"
                    | "Radio"
                    | "Radio2"
                    | "StepSel"
                    | "MultiFuncSW"
                    | "HourCounter"
                    | "TextGenerator"
                    | "PI"
                    | "PID"
                    | "2Point"
                    | "3Point"
                    | "AutopilotRule"
                    | "BrightnessControl"
                    | "DaylightController"
                    | "Wind"
                    | "SequenceController"
                    | "AalEmergency"
                    | "HvacAC"
                    | "WBEM"
                    | "Code1"
                    | "Code4"
                    | "Code8"
                    | "Code16"
                    | "IRcontroller"
                    | "CmdRecognition"
                    | "CallGen"
                    | "MailGen"
                    | "MailBox"
                    | "Nevo"
                    | "AcControl"
                    | "InputRef"
                    | "OutputRef" => Some("Type:Page"),
                    _ => None,
                };

                // If --page or --room matched a page, use that; otherwise fall back to first Page
                let is_page_type = default_parent.is_some();

                if let Some(ref page_path) = target_page_path {
                    // --page or --room matched a specific page — place block there
                    let serial = editor.find_miniserver_serial().unwrap_or_default();
                    let uuid = crate::config_edit::ConfigEditor::loxone_uuid(&serial);
                    let mut elem = xmltree::Element::new("C");
                    elem.attributes
                        .insert("Type".to_string(), xml_type.to_string());
                    elem.attributes.insert("V".to_string(), "175".to_string());
                    elem.attributes.insert("U".to_string(), uuid.clone());
                    elem.attributes.insert("Title".to_string(), title.clone());
                    elem.attributes
                        .insert("WF".to_string(), "16384".to_string());
                    if let Some(prefix) = crate::config_edit::ConfigEditor::iname_prefix(xml_type) {
                        elem.attributes
                            .insert("IName".to_string(), editor.next_iname(prefix));
                    }

                    // Position on the page grid
                    let max_py = editor.max_py_on_page(page_path);
                    elem.attributes.insert("Px".to_string(), "7392".to_string());
                    elem.attributes
                        .insert("Py".to_string(), (max_py + 48).to_string());

                    // Create connectors from embedded connector map
                    let cmap = crate::config_edit::ConfigEditor::connector_map();
                    let (connectors, defaults, _types) =
                        cmap.get(xml_type).cloned().unwrap_or_default();
                    let nio = connectors.len();
                    elem.attributes.insert("Nio".to_string(), nio.to_string());

                    for conn_key in &connectors {
                        let mut co = xmltree::Element::new("Co");
                        co.attributes.insert("K".to_string(), conn_key.clone());
                        co.attributes.insert(
                            "U".to_string(),
                            crate::config_edit::ConfigEditor::loxone_uuid(&serial),
                        );
                        if let Some(def_val) = defaults.get(conn_key) {
                            co.attributes.insert("Def".to_string(), def_val.clone());
                        }
                        elem.children.push(xmltree::XMLNode::Element(co));
                    }

                    // Room/category IoData
                    if room_uuid.is_some() || cat_uuid.is_some() {
                        let mut iodata = xmltree::Element::new("IoData");
                        if let Some(ref r) = room_uuid {
                            iodata.attributes.insert("Pr".to_string(), r.to_string());
                        }
                        if let Some(ref c) = cat_uuid {
                            iodata.attributes.insert("Cr".to_string(), c.to_string());
                        }
                        elem.children.push(xmltree::XMLNode::Element(iodata));
                    }

                    let page_title = editor
                        .get_element(page_path)
                        .attributes
                        .get("Title")
                        .cloned()
                        .unwrap_or_default();
                    let parent_elem = editor.get_element_mut(page_path);
                    parent_elem.children.push(xmltree::XMLNode::Element(elem));
                    if ctx.json {
                        let connectors = collect_block_connectors(&editor, &uuid);
                        emit_json(
                            ctx,
                            serde_json::json!({
                                "ok": true,
                                "uuid": uuid,
                                "type": xml_type,
                                "title": title,
                                "page": page_title,
                                "connectors": connectors,
                            }),
                        );
                    } else {
                        println!(
                            "✓ Added {} '{}' on page '{}' (UUID: {})",
                            xml_type, title, page_title, uuid
                        );
                    }
                } else if is_page_type {
                    // Find first matching Page and insert directly via path
                    let matches = editor.find_elements("Type:Page");
                    if let Some(first_path) = matches.into_iter().next() {
                        let serial = editor.find_miniserver_serial().unwrap_or_default();
                        let uuid = crate::config_edit::ConfigEditor::loxone_uuid(&serial);
                        let mut elem = xmltree::Element::new("C");
                        elem.attributes
                            .insert("Type".to_string(), xml_type.to_string());
                        elem.attributes.insert("V".to_string(), "175".to_string());
                        elem.attributes.insert("U".to_string(), uuid.clone());
                        elem.attributes.insert("Title".to_string(), title.clone());
                        elem.attributes
                            .insert("WF".to_string(), "16384".to_string());
                        if let Some(prefix) =
                            crate::config_edit::ConfigEditor::iname_prefix(xml_type)
                        {
                            elem.attributes
                                .insert("IName".to_string(), editor.next_iname(prefix));
                        }

                        // Create connectors from embedded connector map
                        let cmap = crate::config_edit::ConfigEditor::connector_map();
                        let (connectors, defaults, _types) =
                            cmap.get(xml_type).cloned().unwrap_or_default();
                        let nio = connectors.len();
                        elem.attributes.insert("Nio".to_string(), nio.to_string());

                        for conn_key in &connectors {
                            let mut co = xmltree::Element::new("Co");
                            co.attributes.insert("K".to_string(), conn_key.clone());
                            co.attributes.insert(
                                "U".to_string(),
                                crate::config_edit::ConfigEditor::loxone_uuid(&serial),
                            );
                            if let Some(def_val) = defaults.get(conn_key) {
                                co.attributes.insert("Def".to_string(), def_val.clone());
                            }
                            elem.children.push(xmltree::XMLNode::Element(co));
                        }

                        let parent_elem = editor.get_element_mut(&first_path);
                        parent_elem.children.push(xmltree::XMLNode::Element(elem));
                        if ctx.json {
                            let connectors = collect_block_connectors(&editor, &uuid);
                            emit_json(
                                ctx,
                                serde_json::json!({
                                    "ok": true,
                                    "uuid": uuid,
                                    "type": xml_type,
                                    "title": title,
                                    "connectors": connectors,
                                }),
                            );
                        } else {
                            println!("✓ Added {} '{}' (UUID: {})", xml_type, title, uuid);
                        }
                    } else {
                        let uuid = editor.add_element_to_root(
                            xml_type,
                            &title,
                            room_uuid.as_deref(),
                            cat_uuid.as_deref(),
                            &props,
                        )?;
                        if ctx.json {
                            let connectors = collect_block_connectors(&editor, &uuid);
                            emit_json(
                                ctx,
                                serde_json::json!({
                                    "ok": true,
                                    "uuid": uuid,
                                    "type": xml_type,
                                    "title": title,
                                    "connectors": connectors,
                                }),
                            );
                        } else {
                            println!("✓ Added {} '{}' (UUID: {})", xml_type, title, uuid);
                        }
                    }
                } else {
                    let uuid = editor.add_element_to_root(
                        xml_type,
                        &title,
                        room_uuid.as_deref(),
                        cat_uuid.as_deref(),
                        &props,
                    )?;
                    if ctx.json {
                        let connectors = collect_block_connectors(&editor, &uuid);
                        println!(
                            "{}",
                            serde_json::json!({
                                "ok": true,
                                "uuid": uuid,
                                "type": xml_type,
                                "title": title,
                                "connectors": connectors,
                            })
                        );
                    } else {
                        println!("✓ Added {} '{}' (UUID: {})", xml_type, title, uuid);
                    }
                }
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::Validate { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let results = editor.validate_config();

            let ok = results.iter().filter(|r| r.starts_with('✓')).count();
            let warn = results.iter().filter(|r| r.starts_with('⚠')).count();
            let err = results.iter().filter(|r| r.starts_with('✗')).count();
            if ctx.json {
                let items: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        let level = if r.starts_with('✓') {
                            "ok"
                        } else if r.starts_with('⚠') {
                            "warning"
                        } else if r.starts_with('✗') {
                            "error"
                        } else {
                            "info"
                        };
                        // Strip leading emoji + space
                        let msg = r
                            .chars()
                            .skip_while(|c| !c.is_alphanumeric())
                            .collect::<String>();
                        serde_json::json!({ "level": level, "message": msg })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": err == 0,
                        "passed": ok,
                        "warnings": warn,
                        "errors": err,
                        "results": items,
                    })
                );
            } else {
                for r in &results {
                    println!("{}", r);
                }
                println!("\n{} passed, {} warnings, {} errors", ok, warn, err);
            }
        }
        ConfigCmd::Check { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let results = editor.check_blocks(selector.as_deref());

            let mut ok = 0;
            let mut warn = 0;
            let mut err = 0;
            for r in &results {
                if r.starts_with('✓') {
                    ok += 1;
                } else if r.starts_with('⚠') {
                    warn += 1;
                } else if r.starts_with('✗') {
                    err += 1;
                }
            }
            if ctx.json {
                let items: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        let level = if r.starts_with('✓') {
                            "ok"
                        } else if r.starts_with('⚠') {
                            "warning"
                        } else if r.starts_with('✗') {
                            "error"
                        } else {
                            "info"
                        };
                        let msg = r
                            .chars()
                            .skip_while(|c| !c.is_alphanumeric())
                            .collect::<String>();
                        serde_json::json!({ "level": level, "message": msg })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": err == 0,
                        "passed": ok,
                        "warnings": warn,
                        "errors": err,
                        "results": items,
                    })
                );
            } else {
                for r in &results {
                    println!("{}", r);
                }
                println!("\n{} ok, {} warnings, {} errors", ok, warn, err);
            }
            if err > 0 {
                anyhow::bail!("{} error(s) found in config check", err);
            }
        }
        ConfigCmd::Scan { file, strict } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let content = String::from_utf8_lossy(&data);
            let findings = scan_for_pii(&content, &file);

            for f in &findings {
                println!("{}", f);
            }

            let issues = findings.iter().filter(|f| f.starts_with('✗')).count();
            let warnings = findings.iter().filter(|f| f.starts_with('⚠')).count();

            if issues == 0 && warnings == 0 {
                println!("\n✓ No PII or secrets found — safe to commit");
            } else {
                println!("\n{} issues, {} warnings", issues, warnings);
            }

            if strict && issues > 0 {
                anyhow::bail!("{} PII/secret issues found", issues);
            }
        }
        ConfigCmd::Layout {
            file,
            page,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let page_sel = page.as_deref().unwrap_or("Type:Page");

            let count = editor.grid_layout(page_sel)?;
            println!("✓ Laid out {} elements", count);

            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::UserAdd {
            file,
            name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let uuid = editor.add_user(&name)?;
            println!("✓ Added user '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::UserRemove {
            file,
            name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let uuid = editor.remove_user(&name)?;
            println!("✓ Removed user '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::DeviceBind {
            file,
            control,
            output_conn,
            device,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.create_output_ref(&control, &output_conn, &device)?;
            println!("{}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::AutopilotList { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let controls = loxone_xml::parse_controls(&data, Some("AutoPilot"), None)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&controls)?);
            } else {
                println!("  {:<30} {:<20} UUID", "Title", "Room");
                println!(
                    "  {:<30} {:<20} {}",
                    "─".repeat(30),
                    "─".repeat(20),
                    "─".repeat(36)
                );
                for c in &controls {
                    println!("  {:<30} {:<20} {}", c.title, c.room, c.uuid);
                }
                println!("\n{} autopilot rules", controls.len());
            }
        }
        ConfigCmd::AutopilotAdd {
            file,
            name,
            room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            let uuid =
                editor.add_element_to_root("AutoPilot", &name, room_uuid.as_deref(), None, &[])?;
            println!("✓ Added AutoPilot '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::CalendarAdd {
            file,
            name,
            room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            let uuid =
                editor.add_element_to_root("Calendar", &name, room_uuid.as_deref(), None, &[])?;
            println!("✓ Added Calendar '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::TimerSchedule {
            file,
            selector,
            range,
            value,
            save_as,
        } => {
            let (start_str, end_str) = range.split_once('-').ok_or_else(|| {
                anyhow::anyhow!("Range must be 'HH:MM-HH:MM' (e.g. '20:00-24:00')")
            })?;
            let parse_time = |s: &str| -> Result<u32> {
                let (h, m) = s
                    .split_once(':')
                    .ok_or_else(|| anyhow::anyhow!("Invalid time '{}', expected HH:MM", s))?;
                let hours: u32 = h
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid hour '{}'", h))?;
                let mins: u32 = m
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid minute '{}'", m))?;
                Ok(hours * 60 + mins)
            };
            let start_minutes = parse_time(start_str)?;
            let end_minutes = parse_time(end_str)?;

            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            editor.add_daytimer_entries(&selector, start_minutes, end_minutes, &value)?;

            println!(
                "✓ Added schedule {}-{} (value={}) to '{}'",
                start_str, end_str, value, selector
            );
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::ModeList { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let controls = loxone_xml::parse_controls(&data, Some("Mode"), None)?;
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&controls)?);
            } else {
                println!("  {:<30} UUID", "Title");
                println!("  {:<30} {}", "─".repeat(30), "─".repeat(36));
                for c in &controls {
                    println!("  {:<30} {}", c.title, c.uuid);
                }
                println!("\n{} operating modes", controls.len());
            }
        }
        ConfigCmd::AddVirtualIn {
            file,
            title,
            analog,
            parent,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let parent_sel = parent.as_deref().unwrap_or("Type:VirtualInCaption");
            let aq_uuid = editor.add_virtual_in(&title, analog, parent_sel)?;
            if ctx.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "uuid": aq_uuid,
                        "connector_key": "AQ",
                        "connector_uuid": aq_uuid,
                        "title": title,
                    })
                );
            } else {
                println!("✓ Created VirtualIn \"{}\" (UUID: {})", title, aq_uuid);
                println!("  AQ connector: {}", aq_uuid);
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::WireConnector {
            file,
            target,
            source_uuid,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            // Parse target as "BlockTitle.ConnectorKey"
            let (block_title, conn_key) = target
                .rsplit_once('.')
                .ok_or_else(|| anyhow::anyhow!("Target must be 'BlockTitle.ConnectorKey'"))?;
            editor.wire_connector(block_title, conn_key, &source_uuid)?;
            if ctx.json {
                emit_json(
                    ctx,
                    serde_json::json!({
                        "ok": true,
                        "target": format!("{}.{}", block_title, conn_key),
                        "source": source_uuid,
                    }),
                );
            } else if !ctx.quiet {
                println!("✓ Wired {}.{} ← {}", block_title, conn_key, source_uuid);
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::SetParam {
            file,
            selector,
            param,
            value,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let path = editor.require_one(&selector)?;
            let elem = editor.get_element_mut(&path);
            let title = elem.attributes.get("Title").cloned().unwrap_or_default();
            let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();

            // Find the connector with matching K value
            let mut found = false;
            for child in &mut elem.children {
                if let Some(co) = child.as_mut_element()
                    && co.name == "Co"
                    && co.attributes.get("K").map(|k| k == &param).unwrap_or(false)
                {
                    let old = co
                        .attributes
                        .get("Def")
                        .cloned()
                        .unwrap_or_else(|| "(none)".to_string());
                    co.attributes.insert("Def".to_string(), value.clone());
                    // Validate it's a Parameter connector
                    if let Some(io) = ConfigEditor::connector_io_type(&block_type, &param)
                        && io != "P"
                    {
                        eprintln!(
                            "⚠ {}.{} is an {} connector, not a Parameter",
                            title,
                            param,
                            match io.as_str() {
                                "I" => "Input",
                                "O" => "Output",
                                _ => &io,
                            }
                        );
                    }
                    if ctx.json {
                        emit_json(
                            ctx,
                            serde_json::json!({
                                "ok": true,
                                "block": title,
                                "type": block_type,
                                "param": param,
                                "old": old,
                                "new": value,
                            }),
                        );
                    } else {
                        println!("✓ Set {}.{}: {} → {}", title, param, old, value);
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                bail!(
                    "Parameter '{}' not found on '{}' ({})",
                    param,
                    title,
                    block_type
                );
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::GetParams { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;

            let path = editor.require_one(&selector)?;
            let elem = editor.get_element(&path);
            let title = elem.attributes.get("Title").cloned().unwrap_or_default();
            let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
            let uuid = elem.attributes.get("U").cloned().unwrap_or_default();

            let cmap = ConfigEditor::connector_map();
            let types = cmap
                .get(&block_type)
                .map(|(_, _, t)| t.clone())
                .unwrap_or_default();

            if ctx.json {
                let connectors: Vec<serde_json::Value> = elem
                    .children
                    .iter()
                    .filter_map(|c| c.as_element())
                    .filter(|e| e.name == "Co")
                    .map(|co| {
                        let k = co.attributes.get("K").cloned().unwrap_or_default();
                        let co_uuid = co.attributes.get("U").cloned().unwrap_or_default();
                        let io = types.get(&k).cloned().unwrap_or_else(|| "?".to_string());
                        let def = co.attributes.get("Def").cloned();
                        let inv = co.attributes.contains_key("Inv");
                        let wired = co
                            .children
                            .iter()
                            .any(|c| c.as_element().is_some_and(|e| e.name == "In"));
                        serde_json::json!({
                            "key": k,
                            "uuid": co_uuid,
                            "direction": io,
                            "default": def,
                            "inverted": inv,
                            "wired": wired,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "title": title,
                        "type": block_type,
                        "uuid": uuid,
                        "connectors": connectors,
                    })
                );
            } else {
                println!("{} ({}):", title, block_type);
                for child in &elem.children {
                    if let Some(co) = child.as_element()
                        && co.name == "Co"
                    {
                        let k = co.attributes.get("K").cloned().unwrap_or_default();
                        let io = types.get(&k).map(|s| s.as_str()).unwrap_or("?");
                        let def = co.attributes.get("Def");
                        let inv = co.attributes.get("Inv");
                        if io == "P" || def.is_some() {
                            let def_str = def.map(|d| d.as_str()).unwrap_or("(none)");
                            let inv_str = if inv.is_some() { " [inverted]" } else { "" };
                            println!("  {} {}: {}{}", io, k, def_str, inv_str);
                        }
                    }
                }
            }
        }
        ConfigCmd::GetProgram { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let text = editor.get_program_text(&selector)?;

            if ctx.json {
                let lines = lox_sim::blocks::sequence::count_program_lines(&text);
                let seqs = lox_sim::blocks::sequence::count_sequences(&text);
                emit_json(
                    ctx,
                    serde_json::json!({
                        "program": text,
                        "lines": lines,
                        "sequences": seqs,
                    }),
                );
            } else if text.is_empty() {
                println!("(no program text)");
            } else {
                for (i, line) in text.lines().enumerate() {
                    println!("  {:>3} | {}", i + 1, line);
                }
            }
        }
        ConfigCmd::SetProgram {
            file,
            selector,
            program,
            program_file,
            save_as,
        } => {
            let program_text = match (program, program_file) {
                (Some(text), None) => text,
                (None, Some(path)) => fs::read_to_string(&path)
                    .with_context(|| format!("Cannot read program file '{}'", path))?,
                (Some(_), Some(_)) => {
                    bail!("Specify either inline program text or --file, not both");
                }
                (None, None) => {
                    bail!("Provide program text as argument or use --file to read from a file");
                }
            };

            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let summary = editor.set_program_text(&selector, &program_text)?;

            if ctx.json {
                let lines = lox_sim::blocks::sequence::count_program_lines(&program_text);
                let seqs = lox_sim::blocks::sequence::count_sequences(&program_text);
                let path = editor.require_one(&selector)?;
                let elem = editor.get_element(&path);
                let title = elem.attributes.get("Title").cloned().unwrap_or_default();
                emit_json(
                    ctx,
                    serde_json::json!({
                        "ok": true,
                        "block": title,
                        "lines": lines,
                        "sequences": seqs,
                    }),
                );
            } else {
                println!("{}", summary);
            }
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ConfigCmd::Describe { file, room } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            if ctx.json {
                let structured = editor.describe_config_structured(room.as_deref());
                println!("{}", serde_json::to_string_pretty(&structured)?);
            } else {
                let description = editor.describe_config(room.as_deref());
                println!("{}", description);
            }
        }
        ConfigCmd::Wires {
            file,
            file_opt,
            room,
        } => {
            if file.is_some() && file_opt.is_some() {
                bail!("Pass the config file either positionally or with --file, not both.");
            }
            let file = file
                .or(file_opt)
                .ok_or_else(|| anyhow::anyhow!("Missing config file path."))?;
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let wires = editor.config_wires(room.as_deref());
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&wires)?);
            } else if wires.is_empty() {
                println!("No wires found.");
            } else {
                for wire in wires {
                    println!(
                        "{}.{} ({}) → {}.{} ({})",
                        wire.source.block_title,
                        wire.source.connector_key,
                        wire.source.connector_uuid,
                        wire.target.block_title,
                        wire.target.connector_key,
                        wire.target.connector_uuid
                    );
                }
            }
        }
        ConfigCmd::Stats { file } => {
            if file.ends_with(".zip") {
                bail!(
                    "Expected a .Loxone XML file. Run `lox config extract {}` first.",
                    file
                );
            }
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let stats = editor.config_stats();
            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                print!("{}", stats.format_text());
            }
        }
        ConfigCmd::Template {
            file,
            template,
            room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let plan = editor.apply_template(&template, &room)?;
            save_edited(&editor, &file, save_as.as_deref())?;

            if ctx.json {
                let blocks: Vec<_> = plan
                    .iter()
                    .map(|(block_type, title, params)| {
                        let params_map: serde_json::Map<String, serde_json::Value> = params
                            .iter()
                            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                            .collect();
                        serde_json::json!({
                            "type": block_type,
                            "title": title,
                            "params": params_map,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "template": template,
                        "room": room,
                        "blocks": blocks,
                    })
                );
            } else {
                println!("Template '{}' applied to room '{}':", template, room);
                for (block_type, title, params) in &plan {
                    let param_str = if params.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " [{}]",
                            params
                                .iter()
                                .map(|(k, v)| format!("{}={}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    println!("  ✓ {} ({}){}", title, block_type, param_str);
                }
                println!("\n{} controls created.", plan.len());
                println!(
                    "Next: wire inputs with `lox config wire-connector` and outputs with `lox config device-bind`."
                );
            }
        }
        ConfigCmd::Room(action) => cmd_room(ctx, action)?,
        ConfigCmd::Control(action) => cmd_control(ctx, action)?,
        ConfigCmd::Mqtt(action) => cmd_mqtt_config(ctx, action)?,
        ConfigCmd::Report {
            file,
            issue,
            open_issue,
        } => cmd_config_report(ctx, &file, issue.as_deref(), open_issue)?,
        ConfigCmd::Snapshot {
            file,
            before,
            after,
            utterance,
            dir,
        } => cmd_config_snapshot(
            ctx,
            &file,
            before,
            after,
            utterance.as_deref(),
            dir.as_deref(),
        )?,
        ConfigCmd::Xml(action) => cmd_xml_edit(ctx, action)?,
    }
    Ok(())
}

// ── config report ───────────────────────────────────────────────────────────

/// Generate an anonymised diagnostic report (scan + stats + check).
fn cmd_config_report(
    ctx: &RunContext,
    file: &str,
    issue: Option<&str>,
    open_issue: bool,
) -> Result<()> {
    let data = fs::read(file).with_context(|| format!("Cannot read {}", file))?;
    let content = String::from_utf8_lossy(&data);
    let editor = ConfigEditor::load(&data)?;

    // 1. PII scan
    let findings = scan_for_pii(&content, file);
    let scan_issues = findings.iter().filter(|f| f.starts_with('✗')).count();
    let scan_warnings = findings.iter().filter(|f| f.starts_with('⚠')).count();
    let scan_clean = scan_issues == 0 && scan_warnings == 0;

    // 2. Config stats (anonymised)
    let stats = editor.config_stats();

    // 3. Block check
    let check_results = editor.check_blocks(None);
    let check_ok = check_results.iter().filter(|r| r.starts_with('✓')).count();
    let check_warn = check_results.iter().filter(|r| r.starts_with('⚠')).count();
    let check_err = check_results.iter().filter(|r| r.starts_with('✗')).count();

    // Collect error *types* only (no details) for the report
    let error_types: Vec<String> = check_results
        .iter()
        .filter(|r| r.starts_with('✗') || r.starts_with('⚠'))
        .filter_map(|r| {
            // Extract the error category (first word after the emoji + space)
            r.chars()
                .skip(2) // skip emoji + space
                .collect::<String>()
                .split(':')
                .next()
                .map(|s| s.trim().to_string())
        })
        .collect();

    // Build anonymised report
    let report = serde_json::json!({
        "lox_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "issue_description": issue.unwrap_or("(none)"),
        "scan": {
            "clean": scan_clean,
            "issues": scan_issues,
            "warnings": scan_warnings,
        },
        "stats": {
            "rooms": stats.room_count,
            "pages": stats.page_count,
            "categories": stats.category_count,
            "total_items": stats.total_items,
            "wiring_total": stats.wiring_total,
            "wiring_cross_page": stats.wiring_cross_page,
            "block_types": stats.block_types,
            "device_buses": stats.devices.iter()
                .map(|d| serde_json::json!({"bus": &d.bus_type, "count": d.count}))
                .collect::<Vec<_>>(),
        },
        "check": {
            "ok": check_ok,
            "warnings": check_warn,
            "errors": check_err,
            "error_types": error_types,
        },
    });

    if ctx.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("═══ Diagnostic Report ═══\n");
        if let Some(desc) = issue {
            println!("Issue: {}\n", desc);
        }
        println!(
            "PII scan: {}",
            if scan_clean {
                "✓ clean".to_string()
            } else {
                format!("{} issues, {} warnings", scan_issues, scan_warnings)
            }
        );
        println!(
            "Config:   {} rooms, {} items, {} wires",
            stats.room_count, stats.total_items, stats.wiring_total
        );
        println!(
            "Check:    {} ok, {} warnings, {} errors",
            check_ok, check_warn, check_err
        );
        if !error_types.is_empty() {
            println!("Issues:   {}", error_types.join(", "));
        }
        println!("\nFull JSON: lox config report {} -o json", file);
    }

    // Optionally open a GitHub issue
    if open_issue {
        let title = issue.unwrap_or("Bug report from lox config report");
        let body = format!(
            "## Diagnostic Report\n\n```json\n{}\n```\n\n\
             _Generated by `lox config report`_",
            serde_json::to_string_pretty(&report)?
        );

        // Try gh CLI first
        let gh_result = std::process::Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                "eisber/lox-cli",
                "--title",
                title,
                "--body",
                &body,
            ])
            .output();

        match gh_result {
            Ok(output) if output.status.success() => {
                let url = String::from_utf8_lossy(&output.stdout);
                println!("✓ GitHub issue created: {}", url.trim());
            }
            _ => {
                eprintln!(
                    "⚠ Could not create GitHub issue (install `gh` CLI and run `gh auth login`)."
                );
                eprintln!("  Copy the JSON report above and file manually at:");
                eprintln!("  https://github.com/eisber/lox-cli/issues/new");
            }
        }
    }

    Ok(())
}

// ── config snapshot ─────────────────────────────────────────────────────────

/// Save a config snapshot for before/after evaluation cases.
fn cmd_config_snapshot(
    _ctx: &RunContext,
    file: &str,
    before: bool,
    after: bool,
    utterance: Option<&str>,
    snap_dir: Option<&str>,
) -> Result<()> {
    if !before && !after {
        bail!("Specify --before or --after");
    }

    let data = fs::read(file).with_context(|| format!("Cannot read {}", file))?;
    let editor = ConfigEditor::load(&data)?;
    let stats = editor.config_stats();

    let dir = snap_dir.map(std::path::PathBuf::from).unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".lox")
            .join("snapshots")
    });
    fs::create_dir_all(&dir)?;

    if before {
        let dest = dir.join("before.Loxone");
        fs::copy(file, &dest)?;
        println!(
            "✓ Saved baseline: {} ({} rooms, {} items)",
            dest.display(),
            stats.room_count,
            stats.total_items
        );
        return Ok(());
    }

    // --after: save the file and generate an eval case from the diff
    let dest = dir.join("after.Loxone");
    fs::copy(file, &dest)?;

    let before_path = dir.join("before.Loxone");
    if !before_path.exists() {
        println!("✓ Saved after: {}", dest.display());
        println!("⚠ No before.Loxone found — run with --before first to enable diff.");
        return Ok(());
    }

    // Load before stats for comparison
    let before_data = fs::read(&before_path)?;
    let before_editor = ConfigEditor::load(&before_data)?;
    let before_stats = before_editor.config_stats();

    // Build the eval case
    let eval_case = serde_json::json!({
        "utterance": utterance.unwrap_or(""),
        "before": {
            "rooms": before_stats.room_count,
            "items": before_stats.total_items,
            "wires": before_stats.wiring_total,
            "block_types": before_stats.block_types,
        },
        "after": {
            "rooms": stats.room_count,
            "items": stats.total_items,
            "wires": stats.wiring_total,
            "block_types": stats.block_types,
        },
        "diff": {
            "rooms_delta": stats.room_count as i64 - before_stats.room_count as i64,
            "items_delta": stats.total_items as i64 - before_stats.total_items as i64,
            "wires_delta": stats.wiring_total as i64 - before_stats.wiring_total as i64,
        },
    });

    // Append to cases.json
    let cases_path = dir.join("cases.json");
    let mut cases: Vec<serde_json::Value> = if cases_path.exists() {
        let content = fs::read_to_string(&cases_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    cases.push(eval_case);
    fs::write(&cases_path, serde_json::to_string_pretty(&cases)?)?;

    println!("✓ Saved after:  {}", dest.display());
    println!(
        "✓ Eval case #{} added to {}",
        cases.len(),
        cases_path.display()
    );
    if let Some(utt) = utterance {
        println!("  Utterance: \"{}\"", utt);
    }
    println!(
        "  Delta: {} rooms, {} items, {} wires",
        stats.room_count as i64 - before_stats.room_count as i64,
        stats.total_items as i64 - before_stats.total_items as i64,
        stats.wiring_total as i64 - before_stats.wiring_total as i64,
    );
    Ok(())
}

/// Scan config content for PII, secrets, and credentials.
fn scan_for_pii(content: &str, filename: &str) -> Vec<String> {
    let mut findings = Vec::new();

    // ── Email addresses (simple pattern) ──
    for line in content.lines() {
        if let Some(at_pos) = line.find('@') {
            let before = &line[..at_pos];
            let after = &line[at_pos + 1..];
            // Check it looks like user@domain.tld
            if before.len() >= 2
                && after.contains('.')
                && after.len() >= 4
                && !after.contains("example.com")
                && !after.contains("loxone.com")
                && !after.contains("localhost")
                && !before.contains("test")
            {
                // Extract the email-like substring
                let start = before
                    .rfind(|c: char| {
                        !c.is_alphanumeric()
                            && c != '.'
                            && c != '_'
                            && c != '-'
                            && c != '+'
                            && c != '%'
                    })
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let end_part = after
                    .find(|c: char| !c.is_alphanumeric() && c != '.' && c != '-')
                    .unwrap_or(after.len());
                let email = format!("{}@{}", &before[start..], &after[..end_part]);
                if email.contains('.') && email.len() > 5 {
                    findings.push(format!("✗ Email address found: {}", email));
                }
            }
        }
    }

    // ── Phone numbers ──
    for line in content.lines() {
        if line.contains('+') {
            let mut i = 0;
            let chars: Vec<char> = line.chars().collect();
            while i < chars.len() {
                if chars[i] == '+' && i + 4 < chars.len() && chars[i + 1].is_ascii_digit() {
                    let start = i;
                    i += 1;
                    let mut digits = 0;
                    while i < chars.len()
                        && (chars[i].is_ascii_digit() || chars[i] == ' ' || chars[i] == '-')
                    {
                        if chars[i].is_ascii_digit() {
                            digits += 1;
                        }
                        i += 1;
                    }
                    if digits >= 7 {
                        let phone: String = chars[start..i].iter().collect();
                        findings.push(format!("✗ Phone number found: {}", phone.trim()));
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    // ── Plaintext passwords (t="11") ──
    if content.contains(r#"t="11""#) {
        let count = content.matches(r#"t="11""#).count();
        findings.push(format!(
            "✗ {} plaintext password field(s) found (t=\"11\")",
            count
        ));
    }

    // ── Encrypted passwords (t="15") ──
    if content.contains(r#"t="15""#) {
        let count = content.matches(r#"t="15""#).count();
        findings.push(format!(
            "⚠ {} encrypted password field(s) found (t=\"15\")",
            count
        ));
    }

    // ── User elements with real names ──
    let generic_names = [
        "admin",
        "administrator",
        "user",
        "guest",
        "benutzer",
        "gast",
        "eval fixture",
    ];
    for line in content.lines() {
        if line.contains("Type=\"User\"")
            && line.contains("Title=\"")
            && let Some(start) = line.find("Title=\"")
        {
            let rest = &line[start + 7..];
            if let Some(end) = rest.find('"') {
                let name = &rest[..end];
                if !generic_names.contains(&name.to_lowercase().as_str()) && !name.is_empty() {
                    findings.push(format!("✗ User account with real name: \"{}\"", name));
                }
            }
        }
    }

    // ── Real Miniserver serial in UUIDs ──
    let mut has_real_serial = false;
    for line in content.lines() {
        if line.contains("U=\"") {
            // UUIDs from real Miniservers DON'T end with ffff000000000001
            if let Some(start) = line.find("U=\"") {
                let rest = &line[start + 3..];
                if let Some(end) = rest.find('"') {
                    let uuid = &rest[..end];
                    if uuid.len() >= 36 {
                        let suffix = &uuid[20..];
                        if !suffix.starts_with("ffff") && suffix != "0000000000000000" {
                            has_real_serial = true;
                        }
                    }
                }
            }
        }
    }
    if has_real_serial {
        findings.push(
            "⚠ Real Miniserver serial detected in UUIDs — config is from a real device".to_string(),
        );
    }

    // ── GPS coordinates ──
    for attr in ["Latitude", "Longitude"] {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = content.find(&pattern) {
            let rest = &content[start + pattern.len()..];
            if let Some(end) = rest.find('"') {
                let val = &rest[..end];
                if val.parse::<f64>().is_ok() && val.len() > 2 {
                    findings.push(format!("⚠ GPS {} found: {}", attr, val));
                }
            }
        }
    }

    // ── Internal IP addresses ──
    let mut ips = std::collections::BTreeSet::new();
    for line in content.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("192.168.")
            || line_lower.contains("10.0.")
            || line_lower.contains("172.16.")
        {
            // Extract IP-like patterns
            for word in line.split(|c: char| !c.is_ascii_digit() && c != '.') {
                let parts: Vec<&str> = word.split('.').collect();
                if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
                    ips.insert(word.to_string());
                }
            }
        }
    }
    if !ips.is_empty() {
        findings.push(format!(
            "⚠ Internal IP addresses: {}",
            ips.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    // ── DynDNS hostnames ──
    if content.contains("dns.loxonecloud.com") {
        findings.push("✗ DynDNS hostname found (dns.loxonecloud.com)".to_string());
    }

    // ── NFC tokens ──
    if content.contains("NfcCodeTouch") || content.contains("NFC") {
        // Check for hex token values
        for line in content.lines() {
            if (line.contains("NFC") || line.contains("Badge") || line.contains("NfcCode"))
                && line.contains("V=\"")
            {
                findings.push("✗ NFC/access token data found".to_string());
                break;
            }
        }
    }

    // ── Street addresses ──
    for keyword in ["straße", "strasse", "gasse", "Straße", "Strasse", "Gasse"] {
        if content.contains(keyword) {
            findings.push(format!(
                "⚠ Possible street address found (contains '{}')",
                keyword
            ));
            break;
        }
    }

    if findings.is_empty() {
        findings.push(format!("✓ {} — no PII or secrets detected", filename));
    }

    findings
}

/// Collect connector info from a newly-added element for JSON output.
fn collect_block_connectors(editor: &ConfigEditor, uuid: &str) -> Vec<serde_json::Value> {
    let selector = format!("uuid:{}", uuid);
    if let Ok(path) = editor.require_one(&selector) {
        let elem = editor.get_element(&path);
        let block_type = elem.attributes.get("Type").cloned().unwrap_or_default();
        let cmap = ConfigEditor::connector_map();
        let types = cmap
            .get(&block_type)
            .map(|(_, _, t)| t.clone())
            .unwrap_or_default();

        elem.children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|e| e.name == "Co")
            .map(|co| {
                let k = co.attributes.get("K").cloned().unwrap_or_default();
                let co_uuid = co.attributes.get("U").cloned().unwrap_or_default();
                let io = types.get(&k).cloned().unwrap_or_else(|| "?".to_string());
                let required = io == "I";
                serde_json::json!({
                    "key": k,
                    "uuid": co_uuid,
                    "direction": io,
                    "required": required,
                })
            })
            .collect()
    } else {
        vec![]
    }
}

fn save_edited(editor: &ConfigEditor, original_path: &str, save_as: Option<&str>) -> Result<()> {
    let output = editor.to_bytes()?;
    let out_path = save_as.unwrap_or(original_path);

    // Resolve to canonical parent dir to prevent path traversal via ".." in out_path
    let out = std::path::Path::new(out_path);
    let parent = out.parent().unwrap_or(std::path::Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        parent
    };
    let canon_parent = parent
        .canonicalize()
        .with_context(|| format!("Cannot resolve output directory: {}", parent.display()))?;
    let file_name = out
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid output path: {}", out_path))?;

    let tmp_name = format!("{}.tmp", file_name.to_string_lossy());
    let tmp_path = canon_parent.join(&tmp_name);
    let final_path = canon_parent.join(file_name);

    fs::write(&tmp_path, &output)?;
    fs::rename(&tmp_path, &final_path).or_else(|_| {
        // rename can fail across filesystems — fall back to copy+delete
        fs::copy(&tmp_path, &final_path)?;
        fs::remove_file(&tmp_path)
            .with_context(|| format!("Failed to clean up temp file: {}", tmp_path.display()))?;
        Ok::<(), anyhow::Error>(())
    })?;
    eprintln!("✓ Saved to {}", out_path);
    Ok(())
}

fn cmd_room(_ctx: &RunContext, action: RoomCmd) -> Result<()> {
    match action {
        RoomCmd::Add {
            file,
            name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let uuid = editor.add_room(&name)?;
            println!("✓ Added room '{}' (UUID: {})", name, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        RoomCmd::Rename {
            file,
            old_name,
            new_name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            // Find the room by title and rename
            let _path = editor.require_one(&old_name)?;
            let msg = editor.set_attribute(
                &format!("uuid:{}", {
                    let elem = editor.find_elements(&old_name);
                    if elem.is_empty() {
                        bail!("Room '{}' not found", old_name);
                    }
                    // Get UUID from the element
                    let p = &elem[0];
                    let mut current = &editor.root;
                    for &idx in p {
                        current = current.children[idx].as_element().unwrap();
                    }
                    current.attributes.get("U").cloned().unwrap_or_default()
                }),
                "Title",
                &new_name,
            )?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
    }
    Ok(())
}

fn cmd_control(ctx: &RunContext, action: ControlCmd) -> Result<()> {
    match action {
        ControlCmd::Move {
            file,
            to_room,
            type_filter,
            title,
            exclude,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let exclude_refs: Vec<&str> = exclude.iter().map(|s| s.as_str()).collect();

            if let Some(ref tf) = type_filter {
                let (count, room_uuid) = editor.move_to_room(tf, &to_room, &exclude_refs)?;
                println!(
                    "✓ Moved {} {} items to '{}' ({})",
                    count, tf, to_room, room_uuid
                );
            } else if let Some(ref t) = title {
                // Move single element by title
                let path = editor.require_one(t)?;
                let room_uuid = editor.find_room_uuid(&to_room)?;
                let elem = editor.get_element_mut(&path);
                for child in &mut elem.children {
                    if let Some(iodata) = child.as_mut_element()
                        && iodata.name == "IoData"
                    {
                        iodata
                            .attributes
                            .insert("Pr".to_string(), room_uuid.clone());
                    }
                }
                println!("✓ Moved '{}' to '{}' ({})", t, to_room, room_uuid);
            } else {
                bail!("Specify --type or --title to select controls to move");
            }

            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Rename {
            file,
            selector,
            new_name,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.set_attribute(&selector, "Title", &new_name)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Describe { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let desc = editor.describe(&selector)?;

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&desc)?);
            } else {
                println!("  Type:     {}", desc.element_type);
                println!("  Title:    {}", desc.title);
                println!("  UUID:     {}", desc.uuid);
                if !desc.gid.is_empty() {
                    println!("  gid:      {}", desc.gid);
                }
                if !desc.room_uuid.is_empty() {
                    println!("  Room:     {}", desc.room_uuid);
                }
                if !desc.category_uuid.is_empty() {
                    println!("  Category: {}", desc.category_uuid);
                }
                if !desc.properties.is_empty() {
                    println!("  Properties:");
                    for (k, v) in &desc.properties {
                        println!("    {} = '{}' (t={})", k, v.value, v.type_code);
                    }
                }
                if !desc.connectors.is_empty() {
                    println!("  Connectors:");
                    for c in &desc.connectors {
                        println!("    {} → {}", c.kind, c.target);
                    }
                }
                if !desc.children.is_empty() {
                    println!("  Children:");
                    for c in &desc.children {
                        println!("    {}", c);
                    }
                }
            }
        }
        ControlCmd::Wire {
            file,
            source,
            target,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            // Parse "Selector.Connector" format
            let (src_sel, src_co) = source.rsplit_once('.').ok_or_else(|| {
                anyhow::anyhow!(
                    "Source must be 'ElementSelector.ConnectorName', got '{}'",
                    source
                )
            })?;
            let (tgt_sel, tgt_co) = target.rsplit_once('.').ok_or_else(|| {
                anyhow::anyhow!(
                    "Target must be 'ElementSelector.ConnectorName', got '{}'",
                    target
                )
            })?;

            let msg = editor.wire(src_sel, src_co, tgt_sel, tgt_co)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Unwire {
            file,
            connector,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            let (sel, co_name) = connector.rsplit_once('.').ok_or_else(|| {
                anyhow::anyhow!(
                    "Must be 'ElementSelector.ConnectorName', got '{}'",
                    connector
                )
            })?;

            let msg = editor.unwire(sel, co_name)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        ControlCmd::Wires { file, selector } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let wires = editor.list_wires(&selector)?;

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&wires)?);
            } else {
                let inputs: Vec<_> = wires.iter().filter(|w| w.direction == "input").collect();
                let outputs: Vec<_> = wires.iter().filter(|w| w.direction == "output").collect();
                let params: Vec<_> = wires
                    .iter()
                    .filter(|w| w.direction == "parameter")
                    .collect();

                if !inputs.is_empty() {
                    println!("  Inputs:");
                    for w in &inputs {
                        let status = if w.connected {
                            &w.target_uuid
                        } else {
                            "(unconnected)"
                        };
                        println!("    {:<20} ← {}", w.connector, status);
                    }
                }
                if !outputs.is_empty() {
                    println!("  Outputs:");
                    for w in &outputs {
                        let status = if w.connected {
                            &w.target_uuid
                        } else {
                            "(unconnected)"
                        };
                        println!("    {:<20} → {}", w.connector, status);
                    }
                }
                if !params.is_empty() {
                    println!("  Parameters:");
                    for w in &params {
                        let status = if w.connected {
                            &w.target_uuid
                        } else {
                            "(unconnected)"
                        };
                        println!("    {:<20}   {}", w.connector, status);
                    }
                }
                println!(
                    "\n{} connectors ({} connected, {} unconnected)",
                    wires.len(),
                    wires.iter().filter(|w| w.connected).count(),
                    wires.iter().filter(|w| !w.connected).count(),
                );
            }
        }
    }
    Ok(())
}

fn cmd_mqtt_config(ctx: &RunContext, action: MqttConfigCmd) -> Result<()> {
    match action {
        MqttConfigCmd::Setup {
            file,
            broker,
            port,
            user,
            password,
            client_id,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            editor.set_property("gid:Mqtt", "mqtt_broker_address", &broker, "11")?;
            eprintln!("  Broker: {}", broker);
            editor.set_property("gid:Mqtt", "mqtt_broker_port", &port, "7")?;
            eprintln!("  Port: {}", port);
            if let Some(u) = &user {
                editor.set_property("gid:Mqtt", "mqtt_auth_user", u, "11")?;
                eprintln!("  User: {}", u);
            }
            if let Some(p) = &password {
                editor.set_property("gid:Mqtt", "mqtt_auth_pwd", p, "11")?;
                eprintln!("  Password: (set, plaintext t=11)");
            }
            if let Some(c) = &client_id {
                editor.set_property("gid:Mqtt", "mqtt_client_id", c, "11")?;
                eprintln!("  Client ID: {}", c);
            }

            println!("✓ MQTT configured");
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        MqttConfigCmd::List { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let desc = editor.describe("gid:Mqtt")?;

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&desc)?);
            } else {
                println!("MQTT Plugin: {}", desc.title);
                if !desc.properties.is_empty() {
                    println!("  Configuration:");
                    for (k, v) in &desc.properties {
                        if k.contains("pwd") || k.contains("password") {
                            println!("    {} = *** (t={})", k, v.type_code);
                        } else {
                            println!("    {} = '{}' (t={})", k, v.value, v.type_code);
                        }
                    }
                }
                if !desc.children.is_empty() {
                    println!("  Topics ({}):", desc.children.len());
                    for c in &desc.children {
                        println!("    {}", c);
                    }
                }
            }
        }
        MqttConfigCmd::Topics { file } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let editor = ConfigEditor::load(&data)?;
            let topics = editor.list_mqtt_topics();

            if ctx.json {
                println!("{}", serde_json::to_string_pretty(&topics)?);
            } else {
                println!("  {:<12} {:<30} {:<40} QoS", "Direction", "Title", "Topic");
                println!(
                    "  {:<12} {:<30} {:<40} {}",
                    "─".repeat(12),
                    "─".repeat(30),
                    "─".repeat(40),
                    "─".repeat(4)
                );
                for t in &topics {
                    println!(
                        "  {:<12} {:<30} {:<40} {}",
                        t.direction,
                        t.title,
                        if t.topic.is_empty() {
                            "(not set)"
                        } else {
                            &t.topic
                        },
                        if t.qos.is_empty() { "-" } else { &t.qos }
                    );
                }
                println!(
                    "\n{} topics ({} subscribe, {} publish)",
                    topics.len(),
                    topics.iter().filter(|t| t.direction == "subscribe").count(),
                    topics.iter().filter(|t| t.direction == "publish").count()
                );
            }
        }
    }
    Ok(())
}

fn cmd_xml_edit(_ctx: &RunContext, action: XmlEditCmd) -> Result<()> {
    match action {
        XmlEditCmd::SetProperty {
            file,
            selector,
            property,
            value,
            r#type,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.set_property(&selector, &property, &value, &r#type)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::SetAttr {
            file,
            selector,
            attr,
            value,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let msg = editor.set_attribute(&selector, &attr, &value)?;
            println!("✓ {}", msg);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::Move {
            file,
            type_filter,
            to_room,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let (count, uuid) = editor.move_to_room(&type_filter, &to_room, &[])?;
            println!(
                "✓ Moved {} {} items to '{}' ({})",
                count, type_filter, to_room, uuid
            );
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::Add {
            file,
            parent,
            element_type,
            title,
            gid,
            room,
            category,
            property,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;

            // Resolve room/category names to UUIDs if provided
            let room_uuid = if let Some(ref r) = room {
                Some(editor.find_room_uuid(r)?)
            } else {
                None
            };
            // Category UUID lookup would be similar, but for now pass as-is
            let cat_uuid = category.as_deref();

            // Parse properties: "name:type=value"
            let props: Vec<(&str, &str, &str)> = property
                .iter()
                .filter_map(|p| {
                    let (name_type, value) = p.split_once('=')?;
                    let (name, type_code) = name_type.split_once(':')?;
                    Some((name, value, type_code))
                })
                .collect();

            let uuid = editor.add_element(
                &parent,
                &element_type,
                &title,
                gid.as_deref(),
                room_uuid.as_deref(),
                cat_uuid,
                &props,
            )?;
            println!("✓ Added {} '{}' (UUID: {})", element_type, title, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
        XmlEditCmd::Remove {
            file,
            uuid,
            save_as,
        } => {
            let data = fs::read(&file).with_context(|| format!("Cannot read {}", file))?;
            let mut editor = ConfigEditor::load(&data)?;
            let title = editor.remove_element(&uuid)?;
            println!("✓ Removed '{}' (UUID: {})", title, uuid);
            save_edited(&editor, &file, save_as.as_deref())?;
        }
    }
    Ok(())
}

pub fn cmd_completions(
    _ctx: &RunContext,
    shell: Option<clap_complete::Shell>,
    install: bool,
) -> Result<()> {
    let detected = shell.or_else(detect_shell);
    let Some(sh) = detected else {
        bail!("Could not detect shell. Specify one: lox completions bash|zsh|fish");
    };
    let mut cmd = Cli::command();
    if install {
        install_completions(sh, &mut cmd)?;
    } else {
        generate(sh, &mut cmd, "lox", &mut std::io::stdout());
    }
    Ok(())
}

pub fn cmd_schema(_ctx: &RunContext, command: Option<String>) -> Result<()> {
    let schema = build_schema(command.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}

/// All 163 known Loxone block types (PascalCase, from crosswalk.json).
const KNOWN_BLOCK_TYPES: &[&str] = &[
    // 221 block types from Loxone Config UX TechDoc (tdc_ENG.LxRes)
    "2Point",
    "3Point",
    "AMemory",
    "AMinmax",
    "AalEmergency",
    "AalSmartAlarm",
    "AcControl",
    "Access",
    "Add",
    "Add4",
    "Alarm",
    "AlarmChain",
    "AlarmClock",
    "AnalogComparator",
    "AnalogDiffTrigger",
    "AnalogMultiplexer",
    "AnalogMultiplexer2",
    "AnalogScaler",
    "AnalogStepper",
    "AnalogThresholdTrigger",
    "AnalogWatchdog",
    "And",
    "Application",
    "AudioServer",
    "AutoJalousie",
    "AutomaticScene",
    "AutopilotRule",
    "Average",
    "Avg",
    "BinDecoder",
    "BinEncoder",
    "BrightnessControl",
    "CallGen",
    "CarCharger",
    "CarChargerDevice",
    "CentralAlarm",
    "CentralFancoil",
    "CentralGate",
    "CentralLight",
    "CentralMusic",
    "CentralPresence",
    "CentralRoofwindow",
    "CentralShade",
    "ClimateControllerUS",
    "CmdRecognition",
    "Code1",
    "Code16",
    "Code4",
    "Code8",
    "Comm1wire",
    "Comm232",
    "Comm485",
    "CommDMX",
    "Counter",
    "DayTimer",
    "DaylightController",
    "DbConE",
    "DbConS",
    "DbConT",
    "Devicemonitor",
    "DewPoint",
    "Div",
    "Door",
    "Doorcontroller",
    "EFM",
    "EIBJalousie",
    "EIBPush",
    "EIBsensor",
    "EIBactor",
    "EIBextactor",
    "EdgeDetection",
    "EdgeWipingRelay",
    "EibDimmer",
    "Energy",
    "EnergyManager",
    "EnergyManager2",
    "Equal",
    "Fan",
    "Fancoil",
    "FancoilFreshAir",
    "FlipFlop",
    "Formula",
    "Fronius",
    "GlobalStates",
    "Greater",
    "GreaterEqual",
    "HVACController",
    "HeatCentral",
    "HeatIRoomController2",
    "Heatcurve",
    "Heatmixer",
    "Heatmixer2",
    "HourCounter",
    "HvacAC",
    "IRcontroller",
    "IRoomcontrol",
    "InputRef",
    "Int",
    "Intercom",
    "IntercomDevice",
    "Irrigation",
    "JalousieUpDown2",
    "Jalousiemotor",
    "JoinWindowSensor",
    "Leaf",
    "Less",
    "LessEqual",
    "LightController",
    "LightController2",
    "LightControllerH",
    "Lightscene",
    "LightsceneLearn",
    "LightsceneRGB",
    "LoadShed",
    "LongClick",
    "LoxAin",
    "LoxAout",
    "LoxDIMM",
    "LoxDigin",
    "LoxKnx",
    "LoxLIVE",
    "LoxMORE",
    "LoxOCEAN",
    "MBusExtension",
    "MPGroup",
    "MailBox",
    "MailGen",
    "Media",
    "MediaClient",
    "MessageCenter",
    "MeterAbsBi",
    "MeterAbsSt",
    "MeterAbsUni",
    "MeterDig",
    "MeterPBi",
    "MeterPSt",
    "MeterPUni",
    "Minmax",
    "Mod",
    "ModbusServer",
    "Monoflop",
    "MsShortcut",
    "Mult",
    "MultiClick",
    "MultiFuncSW",
    "MusicPlayer",
    "Nevo",
    "NfcCodeTouch",
    "Not",
    "NotEqual",
    "OffDelay",
    "OnDelay",
    "OnOffDelay",
    "OnPulseDelay",
    "Or",
    "OutputRef",
    "PButtonT",
    "PI",
    "PID",
    "PVProductionForecast",
    "PWM",
    "Ping",
    "Plugin",
    "PoolController",
    "Power",
    "PowerUnit",
    "Presence",
    "PresenceController",
    "PresenceDetector",
    "PulseAt",
    "PulseBy",
    "PulseGen",
    "PushButton",
    "PushButton2",
    "PushButton2Sel",
    "PushButtonSel",
    "PushDimmer",
    "RSFlipFlop",
    "Radio",
    "Radio2",
    "Ramp",
    "Rand",
    "RandomGen",
    "RetOnDelay",
    "RoofWindow",
    "Roomcontrol",
    "SRFlipFlop",
    "Sauna",
    "SaunaVapor",
    "SequenceController",
    "Sequencer",
    "ShadeRoof",
    "Shift",
    "SmokeAlarm",
    "Solarpumpcontrol",
    "SonnenBatteryDevice",
    "SpotOpt",
    "StairwayLS",
    "State",
    "StateV",
    "StatusMonitor",
    "SteakThermo",
    "StepSel",
    "Sub",
    "SystemScheme",
    "Tablet",
    "Text",
    "TextGenerator",
    "TimeMinmax",
    "ToiletFan",
    "TpfController",
    "Tracker",
    "UpDownCounter",
    "Validator",
    "VentInternorm",
    "Ventilation",
    "WBEM",
    "Wallbox",
    "WeatherServer",
    "Weed",
    "Wind",
    "WindowsMonitor",
    "Xor",
    // Additional types used by CLI aliases
    "Calendar",
    "GenTActor",
    "GenTSensor",
    "VirtualIn",
    "VirtualState",
];

/// Resolve a user-provided block type name to PascalCase XML type.
/// Accepts: PascalCase ("And"), kebab-case ("flip-flop-rs"), lowercase ("and"),
/// or friendly aliases ("light" → "LightController2").
fn resolve_block_type(input: &str) -> Result<(String, Option<&'static str>)> {
    // Friendly aliases (short names → XML types)
    let (xml_type, parent) = match input {
        "light" => {
            eprintln!("ℹ Alias 'light' → LightController2");
            return Ok(("LightController2".into(), None));
        }
        "switch" => {
            eprintln!("ℹ Alias 'switch' → PushButton");
            return Ok(("PushButton".into(), None));
        }
        "presence" => {
            eprintln!("ℹ Alias 'presence' → PresenceDetector");
            return Ok(("PresenceDetector".into(), None));
        }
        "alarm-clock" => return Ok(("AlarmClock".into(), None)),
        "memory" => return Ok(("AMemory".into(), None)),
        "timer" => return Ok(("DayTimer".into(), None)),
        "mqtt-sub" => return Ok(("GenTSensor".into(), Some("gid:Mqtt"))),
        "mqtt-pub" => return Ok(("GenTActor".into(), Some("gid:Mqtt"))),
        "calendar" => return Ok(("Calendar".into(), None)),
        "autopilot" => return Ok(("AutoPilot".into(), None)),
        "virtual-state" => return Ok(("VirtualState".into(), Some("Type:VirtualOutCaption"))),
        "state-v" | "status" => return Ok(("StateV".into(), Some("Type:Page"))),
        _ => (input, None),
    };

    // Exact PascalCase match
    if KNOWN_BLOCK_TYPES.contains(&xml_type) {
        return Ok((xml_type.to_string(), parent));
    }

    // Case-insensitive match
    let lower = xml_type.to_lowercase();
    if let Some(t) = KNOWN_BLOCK_TYPES.iter().find(|t| t.to_lowercase() == lower) {
        if *t != xml_type {
            eprintln!("⚠ Type '{}' resolved to '{}' (case corrected)", input, t);
        }
        return Ok((t.to_string(), parent));
    }

    // Kebab-case → PascalCase conversion
    let pascal: String = xml_type
        .split('-')
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        })
        .collect();
    if KNOWN_BLOCK_TYPES.contains(&pascal.as_str()) {
        if pascal != input {
            eprintln!(
                "⚠ Type '{}' resolved to '{}' (kebab-case converted)",
                input, pascal
            );
        }
        return Ok((pascal, parent));
    }
    // Case-insensitive on pascal conversion
    if let Some(t) = KNOWN_BLOCK_TYPES
        .iter()
        .find(|t| t.to_lowercase() == pascal.to_lowercase())
    {
        eprintln!("⚠ Type '{}' resolved to '{}' (case corrected)", input, t);
        return Ok((t.to_string(), parent));
    }

    // No match — use Levenshtein fuzzy matching for better suggestions
    let candidates: Vec<String> = KNOWN_BLOCK_TYPES.iter().map(|s| s.to_string()).collect();
    Err(crate::errors::not_found_error(
        "Block type",
        input,
        &candidates,
        "lox config add --help",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    const FIXTURE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="r-0001-0001-0001-ffff000000000001" Title="Room1" WF="16384"/>
  <C Type="Place" V="175" U="r-0002-0002-0002-ffff000000000002" Title="Room2" WF="16384"/>
  <C Type="Category" V="175" U="c-0001-0001-0001-ffff000000000001" Title="Beleuchtung" WF="16384"/>
  <C Type="Page" V="175" U="p-0001-0001-0001-ffff000000000001" Title="Page1" WF="16384">
    <C Type="And" V="175" U="a-0001-0001-0001-ffff000000000001" Title="TestAnd" WF="16384" Nc="2" Nio="3">
      <Co K="I1" U="i-0001-0001-0001-ffff000000000001"/>
      <Co K="I2" U="i-0002-0002-0002-ffff000000000002"/>
      <Co K="Q" U="q-0001-0001-0001-ffff000000000001"/>
      <IoData Cr="c-0001-0001-0001-ffff000000000001" Pr="r-0001-0001-0001-ffff000000000001"/>
    </C>
  </C>
  <C Type="TreeDevice" V="175" U="td-001-0001-0001-ffff000000000001" Title="Touch Tree" WF="16384" Serial="B056A424" SubType="32771">
    <IoData Pr="r-0001-0001-0001-ffff000000000001"/>
    <C Type="TreeAsensor" IName="AI1" V="175" U="ts-001-0001-0001-ffff000000000001" Title="Temperatur" Nio="1" WF="20480">
      <Co K="AQ" U="tq-001-0001-0001-ffff000000000001"/>
      <IoData Cr="c-0001-0001-0001-ffff000000000001" Pr="r-0001-0001-0001-ffff000000000001"/>
    </C>
    <C Type="TreeAactor" IName="AQ1" V="175" U="ta-001-0001-0001-ffff000000000001" Title="Dimmer" Nio="1" WF="20480">
      <Co K="I" U="ti-001-0001-0001-ffff000000000001"/>
      <IoData Cr="c-0001-0001-0001-ffff000000000001" Pr="r-0001-0001-0001-ffff000000000001"/>
    </C>
  </C>
</ControlList>"#;

    const WIRES_FIXTURE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-1" Title="Room1" WF="16384"/>
  <C Type="Place" V="175" U="room-2" Title="Room2" WF="16384"/>
  <C Type="Category" V="175" U="cat-1" Title="Beleuchtung" WF="16384"/>
  <C Type="Page" V="175" U="page-1" Title="Page1" WF="16384">
    <C Type="WeatherData" V="175" U="source-weather" Title="Helligkeitssensor BWM 1" WF="16384">
      <Co K="AQ" U="source-weather-aq"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="AlarmClock" V="175" U="source-clock" Title="Wecker" WF="16384">
      <Co K="Q" U="source-clock-q"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="LightController2" V="175" U="target-light" Title="LightController2" WF="16384">
      <Co K="AI1" U="target-light-ai1"><In Input="source-weather-aq"/></Co>
      <Co K="I1" U="target-light-i1"><In Input="source-clock-q"/></Co>
      <Co K="I2" U="00000000-0000-0000-0000000000000000"/>
      <Co K="I3" U="target-light-i3"/>
      <Co K="I4" U="target-light-i4"><In Input="target-light-aq1"/></Co>
      <Co K="AQ1" U="target-light-aq1"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="WeatherData" V="175" U="legacy-source" Title="Legacy Sensor" WF="16384">
      <Co K="AQ" U="legacy-source-aq"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="WeatherData" V="175" U="legacy-source-2" Title="Legacy Sensor 2" WF="16384">
      <Co K="AQ" U="legacy-source-2-aq"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="AlarmClock" V="175" U="legacy-source-3" Title="Legacy Source 3" WF="16384">
      <Co K="Q" U="legacy-source-3-q"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="LightController2" V="175" U="legacy-target" Title="Legacy Light" WF="16384">
      <Co K="AI1" U="legacy-source-aq"/>
      <Co K="AI2" U="legacy-source-2-aq"/>
      <Co K="I1" U="legacy-source-3-q"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="LightController2" V="175" U="legacy-target-2" Title="Legacy Light 2" WF="16384">
      <Co K="AI1" U="legacy-source-aq"/>
      <Co K="AI2" U="legacy-source-2-aq"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
  </C>
  <C Type="Page" V="175" U="page-2" Title="Page2" WF="16384">
    <C Type="LightController2" V="175" U="target-room2" Title="Room2 Light" WF="16384">
      <Co K="AI1" U="target-room2-ai1"><In Input="source-weather-aq"/></Co>
      <IoData Cr="cat-1" Pr="room-2"/>
    </C>
  </C>
</ControlList>"#;

    const DEVICES_FIXTURE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-kitchen" Title="Küche" WF="16384"/>
  <C Type="Place" V="175" U="room-office" Title="Office" WF="16384"/>
  <C Type="Category" V="175" U="cat-hardware" Title="Hardware" WF="16384"/>
  <C Type="TreeExtension" V="175" U="ext-tree" Serial="TREE-EXT-01">
    <C Type="TreeDevice" V="175" U="tree-touch" Title="Tree Touch Küche" Serial="TREE-TOUCH-01" DeviceType="Loxone Tree Touch">
      <Co K="P1" U="tree-touch-p1"/>
      <Co K="P2" U="tree-touch-p2"/>
      <IoData Cr="cat-hardware" Pr="room-kitchen"/>
      <C Type="TreeAsensor" V="175" U="tree-touch-temp" Title="Tree Touch Temp">
        <Co K="AQ" U="tree-touch-temp-aq"/>
        <IoData Cr="cat-hardware" Pr="room-kitchen"/>
      </C>
    </C>
  </C>
  <C Type="DALIextension" V="175" U="ext-dali" Serial="DALI-EXT-01">
    <C Type="DALIDriver" V="175" U="dali-driver" Title="DALI Driver" BusAddress="192" DeviceType="DALI Driver 8ch">
      <Co K="CH1" U="dali-driver-ch1"/>
      <Co K="CH2" U="dali-driver-ch2"/>
      <Co K="CH3" U="dali-driver-ch3"/>
      <IoData Cr="cat-hardware" Pr="room-kitchen"/>
    </C>
  </C>
  <C Type="KNXExtension" V="175" U="ext-knx">
    <C Type="EIBSensor" V="175" U="knx-sensor" Title="KNX Sensor" GroupAddress="1/2/3">
      <Co K="Q" U="knx-sensor-q"/>
      <IoData Cr="cat-hardware" Pr="room-kitchen"/>
    </C>
  </C>
  <C Type="AirExtension" V="175" U="ext-air" Serial="AIR-EXT-01">
    <C Type="AirDevice" V="175" U="air-switch" Title="Air Switch" BusSerial="AIR-SWITCH-01">
      <Co K="I1" U="air-switch-i1"/>
      <IoData Cr="cat-hardware" Pr="room-kitchen"/>
    </C>
  </C>
  <C Type="1WireExtension" V="175" U="ext-onewire" Serial="ONEWIRE-EXT-01">
    <C Type="OneWireSensor" V="175" U="onewire-temp" Title="1-Wire Temp" Address="28.FF.01">
      <Co K="AQ" U="onewire-temp-aq"/>
      <IoData Cr="cat-hardware" Pr="room-kitchen"/>
    </C>
  </C>
  <C Type="Page" V="175" U="page-office" Title="Office Page" WF="16384">
    <C Type="NetworkDevice" V="175" U="standalone-weather" Title="Standalone Weather" WF="16384">
      <Co K="AQ" U="standalone-weather-aq"/>
      <IoData Cr="cat-hardware" Pr="room-office"/>
    </C>
    <C Type="Calculator" V="175" U="calc-virtual" Title="Virtual Calculator" WF="16384">
      <Co K="Q" U="calc-virtual-q"/>
      <IoData Cr="cat-hardware" Pr="room-office"/>
    </C>
    <C Type="DayTimer" V="175" U="timer-virtual" Title="Virtual Timer" WF="16384">
      <Co K="Q" U="timer-virtual-q"/>
      <IoData Cr="cat-hardware" Pr="room-office"/>
    </C>
  </C>
</ControlList>"#;

    const EIB_CAPTION_FIXTURE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-1" Title="Room1" WF="16384"/>
  <C Type="EIBline" V="175" U="eib-line" Title="KNX/EIB Linie">
    <C Type="EIBsensorCaption" V="175" U="eib-sensor-caption" Title="Sensors">
      <C Type="EIBsensor" IName="KGI1.2.3" V="175" U="eib-sensor-1" Title="Taster 1 links" Nio="1" WF="16384" EibAddr="1/2/3">
        <Co K="Q" U="eib-sensor-1-q"/>
        <IoData Pr="room-1"/>
      </C>
      <C Type="EIBsensor" IName="KGI1.2.4" V="175" U="eib-sensor-2" Title="Taster 1 rechts" Nio="1" WF="16384" EibAddr="1/2/4">
        <Co K="Q" U="eib-sensor-2-q"/>
        <IoData Pr="room-1"/>
      </C>
      <C Type="EIBsensor" IName="KGI1.2.5" V="175" U="eib-sensor-3" Title="Taster 2 links" Nio="1" WF="16384" EibAddr="1/2/5">
        <Co K="Q" U="eib-sensor-3-q"/>
        <IoData Pr="room-1"/>
      </C>
    </C>
    <C Type="EIBactorCaption" V="175" U="eib-actor-caption" Title="Actuators">
      <C Type="EIBactor" IName="KGQ2.0.1" V="175" U="eib-actor-1" Title="Relay 1 light" Nio="1" WF="16384" EibAddr="2/0/1">
        <Co K="I" U="eib-actor-1-i"/>
        <IoData Pr="room-1"/>
      </C>
      <C Type="EIBactor" IName="KGQ2.0.2" V="175" U="eib-actor-2" Title="Relay 1 fan" Nio="1" WF="16384" EibAddr="2/0/2">
        <Co K="I" U="eib-actor-2-i"/>
        <IoData Pr="room-1"/>
      </C>
    </C>
  </C>
</ControlList>"#;

    fn fixture_file() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.Loxone");
        fs::write(&path, FIXTURE_XML).unwrap();
        (dir, path.to_str().unwrap().to_string())
    }

    fn wires_fixture_file() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("wires.Loxone");
        fs::write(&path, WIRES_FIXTURE_XML).unwrap();
        (dir, path.to_str().unwrap().to_string())
    }

    fn devices_fixture_file() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("devices.Loxone");
        fs::write(&path, DEVICES_FIXTURE_XML).unwrap();
        (dir, path.to_str().unwrap().to_string())
    }

    fn ctx() -> RunContext {
        RunContext {
            json: false,
            quiet: false,
            csv: false,
            dry_run: false,
            no_header: false,
            non_interactive: false,
            trace_id: None,
        }
    }

    #[test]
    fn test_cmd_rooms() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::Rooms { file, limit: None });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_controls() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Controls {
                file,
                r#type: None,
                room: None,
                limit: 100,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_controls_filter_type() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Controls {
                file,
                r#type: Some("And".to_string()),
                room: None,
                limit: 100,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_controls_filter_room() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Controls {
                file,
                r#type: None,
                room: Some("Room1".to_string()),
                limit: 100,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_devices() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Devices {
                file,
                ports: false,
                room: None,
                summary: false,
                limit: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_devices_ports() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Devices {
                file,
                ports: true,
                room: None,
                summary: false,
                limit: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_validate() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::Validate { file });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_describe() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::Describe { file, room: None });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::Stats { file });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_json() {
        let (_dir, file) = fixture_file();
        let json_ctx = RunContext {
            json: true,
            quiet: false,
            csv: false,
            dry_run: false,
            no_header: false,
            non_interactive: false,
            trace_id: None,
        };
        let result = cmd_config(&json_ctx, ConfigCmd::Stats { file });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_stats_rejects_zip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.zip");
        fs::write(&path, b"dummy").unwrap();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Stats {
                file: path.to_str().unwrap().to_string(),
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_describe_room() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Describe {
                file,
                room: Some("Room1".to_string()),
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_wires_basic() {
        let (_dir, file) = wires_fixture_file();
        let json_ctx = RunContext {
            json: true,
            quiet: false,
            csv: false,
            dry_run: false,
            no_header: false,
            non_interactive: false,
            trace_id: None,
        };
        let result = cmd_config(
            &json_ctx,
            ConfigCmd::Wires {
                file: Some(file.clone()),
                file_opt: None,
                room: None,
            },
        );
        assert!(result.is_ok());

        let data = fs::read(&file).unwrap();
        let editor = ConfigEditor::load(&data).unwrap();
        let wires = editor.config_wires(None);
        assert_eq!(wires.len(), 9);
        let light_ai1 = wires
            .iter()
            .find(|w| w.target.block_uuid == "target-light" && w.target.connector_key == "AI1")
            .unwrap();
        assert_eq!(light_ai1.source.block_uuid, "source-weather");
        assert_eq!(light_ai1.source.block_title, "Helligkeitssensor BWM 1");
        assert_eq!(light_ai1.source.block_type, "WeatherData");
        assert_eq!(light_ai1.source.connector_uuid, "source-weather-aq");
        assert_eq!(light_ai1.source.connector_key, "AQ");
        assert_eq!(light_ai1.target.block_title, "LightController2");
        assert_eq!(light_ai1.target.block_type, "LightController2");
        assert_eq!(light_ai1.target.connector_uuid, "target-light-ai1");
        let same_block = wires
            .iter()
            .find(|w| w.target.block_uuid == "target-light" && w.target.connector_key == "I4")
            .unwrap();
        assert_eq!(same_block.source.block_uuid, "target-light");
        assert_eq!(same_block.source.connector_uuid, "target-light-aq1");
        assert_eq!(same_block.target.connector_uuid, "target-light-i4");
    }

    #[test]
    fn test_cmd_wires_room_filter() {
        let data = WIRES_FIXTURE_XML.as_bytes();
        let editor = ConfigEditor::load(data).unwrap();
        let wires = editor.config_wires(Some("Room2"));
        assert_eq!(wires.len(), 1);
        assert_eq!(wires[0].source.block_uuid, "source-weather");
        assert_eq!(wires[0].target.block_uuid, "target-room2");
        assert_eq!(wires[0].target.connector_key, "AI1");
    }

    #[test]
    fn test_cmd_wires_unwired_skipped() {
        let editor = ConfigEditor::load(WIRES_FIXTURE_XML.as_bytes()).unwrap();
        let wires = editor.config_wires(None);
        assert!(!wires.iter().any(|w| {
            w.target.connector_uuid == "00000000-0000-0000-0000000000000000"
                || w.target.connector_key == "I2"
                || w.target.connector_key == "I3"
        }));
        assert!(
            !wires
                .iter()
                .any(|w| { w.source.connector_uuid == "00000000-0000-0000-0000000000000000" })
        );
    }

    #[test]
    fn test_cmd_devices_basic() {
        let editor = ConfigEditor::load(DEVICES_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(None);
        assert_eq!(devices.len(), 6);
        let bus_types: HashSet<_> = devices.iter().map(|d| d.bus_type.as_str()).collect();
        assert!(bus_types.contains("tree"));
        assert!(bus_types.contains("dali"));
        assert!(bus_types.contains("knx"));
        assert!(bus_types.contains("loxone-air"));
        assert!(bus_types.contains("1-wire"));
        assert!(bus_types.contains("standalone"));

        let tree = devices.iter().find(|d| d.bus_type == "tree").unwrap();
        assert_eq!(tree.bus_serial.as_deref(), Some("TREE-EXT-01"));
        assert_eq!(tree.bus_address.as_deref(), Some("TREE-TOUCH-01"));
        assert_eq!(tree.device_type, "Loxone Tree Touch");
        assert_eq!(tree.primary_block_uuid, "tree-touch");
        assert!(
            tree.secondary_block_uuids
                .contains(&"tree-touch-temp".to_string())
        );
        assert!(tree.connectors.iter().any(|c| {
            c.uuid == "tree-touch-p2"
                && c.role == "P2"
                && c.channel_index == Some(2)
                && c.connector_type == "parameter"
        }));
        assert_eq!(tree.snapshot_room_label.as_deref(), Some("Küche"));
        assert!(!tree.low_confidence_identity);
        assert_eq!(tree.identity_components.bus_type, "tree");
    }

    #[test]
    fn test_cmd_devices_room_filter() {
        let editor = ConfigEditor::load(DEVICES_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(Some("Office"));
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].primary_block_uuid, "standalone-weather");
        assert_eq!(devices[0].snapshot_room_label.as_deref(), Some("Office"));
    }

    #[test]
    fn test_cmd_devices_excluded_types_skipped() {
        let editor = ConfigEditor::load(DEVICES_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(None);
        assert!(
            !devices
                .iter()
                .any(|d| d.primary_block_uuid == "calc-virtual")
        );
        assert!(
            !devices
                .iter()
                .any(|d| d.primary_block_uuid == "timer-virtual")
        );
        assert!(
            !devices
                .iter()
                .any(|d| d.derived_label.contains("Virtual Calculator"))
        );
    }

    #[test]
    fn test_cmd_devices_dali_grouping() {
        let editor = ConfigEditor::load(DEVICES_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(None);
        let dali = devices.iter().find(|d| d.bus_type == "dali").unwrap();
        assert_eq!(dali.primary_block_uuid, "dali-driver");
        assert_eq!(dali.bus_address.as_deref(), Some("192"));
        let channels: Vec<_> = dali
            .connectors
            .iter()
            .filter(|c| c.connector_type == "channel")
            .collect();
        assert_eq!(channels.len(), 3);
        assert!(
            channels
                .iter()
                .any(|c| c.role == "CH3" && c.channel_index == Some(3))
        );
    }

    #[test]
    fn test_cmd_devices_eib_caption_grouping() {
        let editor = ConfigEditor::load(EIB_CAPTION_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(None);
        assert_eq!(devices.len(), 3);

        let taster_1 = devices
            .iter()
            .find(|d| d.bus_address.as_deref() == Some("1.2.3,1.2.4"))
            .unwrap();
        assert_eq!(taster_1.bus_type, "knx");
        assert_eq!(taster_1.bus_serial, None);
        assert_eq!(taster_1.primary_block_uuid, "eib-sensor-caption");
        assert_eq!(taster_1.secondary_block_uuids.len(), 2);
        assert_eq!(taster_1.device_type, "EIB Sensor (2ch)");
        assert!(
            taster_1
                .connectors
                .iter()
                .any(|c| c.uuid == "eib-sensor-2-q" && c.channel_index == Some(2))
        );

        let relay_1 = devices
            .iter()
            .find(|d| d.bus_address.as_deref() == Some("2.0.1,2.0.2"))
            .unwrap();
        assert_eq!(relay_1.primary_block_uuid, "eib-actor-caption");
        assert_eq!(relay_1.secondary_block_uuids.len(), 2);
        assert_eq!(relay_1.device_type, "EIB Actor (2ch)");
    }

    #[test]
    fn test_cmd_devices_summary_flag() {
        let (_dir, file) = devices_fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Devices {
                file,
                ports: false,
                room: None,
                summary: true,
                limit: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_devices_sort_order() {
        let editor = ConfigEditor::load(DEVICES_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(None);
        let keys: Vec<_> = devices
            .iter()
            .map(|d| {
                (
                    d.bus_type.clone(),
                    d.bus_serial.clone(),
                    d.bus_address.clone(),
                    d.identity_components.channel_role.clone(),
                    d.primary_block_uuid.clone(),
                )
            })
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn test_cmd_devices_consistency_with_describe() {
        let editor = ConfigEditor::load(DEVICES_FIXTURE_XML.as_bytes()).unwrap();
        let devices = editor.config_devices(None);
        let described_uuids: HashSet<_> = editor
            .describe_config_structured(None)
            .into_iter()
            .flat_map(|room| room.blocks.into_iter().map(|block| block.uuid))
            .collect();

        for device in devices {
            assert!(
                described_uuids.contains(&device.primary_block_uuid),
                "{} missing from describe output",
                device.primary_block_uuid
            );
            for secondary_uuid in device.secondary_block_uuids {
                assert!(
                    described_uuids.contains(&secondary_uuid),
                    "{} missing from describe output",
                    secondary_uuid
                );
            }
        }
    }

    #[test]
    fn test_cmd_wires_legacy_co_u_wiring() {
        let editor = ConfigEditor::load(WIRES_FIXTURE_XML.as_bytes()).unwrap();
        let legacy: Vec<_> = editor
            .config_wires(None)
            .into_iter()
            .filter(|w| w.target.block_uuid.starts_with("legacy-target"))
            .collect();
        assert_eq!(legacy.len(), 5);
        assert!(legacy.iter().any(|wire| {
            wire.source.block_uuid == "legacy-source"
                && wire.source.connector_uuid == "legacy-source-aq"
                && wire.target.block_uuid == "legacy-target"
                && wire.target.connector_key == "AI1"
        }));
        assert!(legacy.iter().any(|wire| {
            wire.source.block_uuid == "legacy-source-2"
                && wire.source.connector_uuid == "legacy-source-2-aq"
                && wire.target.block_uuid == "legacy-target"
                && wire.target.connector_key == "AI2"
        }));
        assert!(legacy.iter().any(|wire| {
            wire.source.block_uuid == "legacy-source-3"
                && wire.source.connector_uuid == "legacy-source-3-q"
                && wire.target.block_uuid == "legacy-target"
                && wire.target.connector_key == "I1"
        }));
        assert!(legacy.iter().any(|wire| {
            wire.source.block_uuid == "legacy-source"
                && wire.target.block_uuid == "legacy-target-2"
                && wire.target.connector_key == "AI1"
        }));
        assert!(legacy.iter().any(|wire| {
            wire.source.block_uuid == "legacy-source-2"
                && wire.target.block_uuid == "legacy-target-2"
                && wire.target.connector_key == "AI2"
        }));
    }

    #[test]
    fn test_cmd_wires_includes_ref_blocks_counted_by_stats() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="room-1" Title="Room1"/>
  <C Type="Category" V="175" U="cat-1" Title="Beleuchtung"/>
  <C Type="Page" V="175" U="page-1" Title="Page1">
    <C Type="InputRef" V="175" U="inputref-1" Title="Input Ref">
      <Co K="AQ" U="inputref-1-aq"/>
      <Co K="I" U="inputref-1-i"><In Input="virtual-in-aq"/></Co>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="OutputRef" V="175" U="outputref-1" Title="Output Ref">
      <Co K="AQ" U="outputref-1-aq"><In Input="inputref-1-aq"/></Co>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="VirtualIn" V="175" U="virtual-in" Title="Virtual In">
      <Co K="AQ" U="virtual-in-aq"/>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
    <C Type="EIBactor" V="175" U="actor-1" Title="Actor">
      <Co K="I" U="actor-1-i"><In Input="outputref-1-aq"/></Co>
      <IoData Cr="cat-1" Pr="room-1"/>
    </C>
  </C>
</ControlList>"#;
        let editor = ConfigEditor::load(xml).unwrap();
        let wires = editor.config_wires(None);
        assert_eq!(editor.config_stats().wiring_total, 3);
        assert_eq!(wires.len(), 3);
        assert!(wires.iter().any(|wire| {
            wire.source.block_uuid == "inputref-1" && wire.target.block_uuid == "outputref-1"
        }));
        assert!(wires.iter().any(|wire| {
            wire.source.block_uuid == "outputref-1" && wire.target.block_uuid == "actor-1"
        }));
        assert!(wires.iter().any(|wire| {
            wire.source.block_uuid == "virtual-in" && wire.target.block_uuid == "inputref-1"
        }));
    }

    #[test]
    fn test_cmd_wires_describe_consistency() {
        let editor = ConfigEditor::load(WIRES_FIXTURE_XML.as_bytes()).unwrap();
        let describe = editor.describe_config_structured(None);
        let connector_uuids: HashSet<String> = describe
            .into_iter()
            .flat_map(|room| room.blocks)
            .flat_map(|block| block.connectors)
            .map(|connector| connector.uuid)
            .collect();

        for wire in editor.config_wires(None) {
            assert!(connector_uuids.contains(&wire.source.connector_uuid));
            assert!(connector_uuids.contains(&wire.target.connector_uuid));
        }
    }

    #[test]
    fn test_cmd_users() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::Users { file, limit: None });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_add_and() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Add {
                file: file.clone(),
                control_type: "And".to_string(),
                title: "NewAnd".to_string(),
                room: Some("Room1".to_string()),
                category: Some("Beleuchtung".to_string()),
                parent: None,
                page: None,
                topic: None,
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read(&file).unwrap();
        assert!(String::from_utf8_lossy(&data).contains("NewAnd"));
    }

    #[test]
    fn test_cmd_add_with_save_as() {
        let (dir, file) = fixture_file();
        let out = dir.path().join("output.Loxone");
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Add {
                file: file.clone(),
                control_type: "Or".to_string(),
                title: "NewOr".to_string(),
                room: None,
                category: None,
                parent: None,
                page: None,
                topic: None,
                save_as: Some(out.to_str().unwrap().to_string()),
            },
        );
        assert!(result.is_ok());
        // Original unchanged
        let orig = fs::read_to_string(&file).unwrap();
        assert!(!orig.contains("NewOr"));
        // Output has it
        let output = fs::read_to_string(&out).unwrap();
        assert!(output.contains("NewOr"));
    }

    #[test]
    fn test_cmd_set_param() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::SetParam {
                file: file.clone(),
                selector: "TestAnd".to_string(),
                param: "I1".to_string(),
                value: "42".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read_to_string(&file).unwrap();
        assert!(data.contains(r#"Def="42""#));
    }

    #[test]
    fn test_cmd_get_params() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::GetParams {
                file,
                selector: "TestAnd".to_string(),
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_set_and_get_program() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seq.Loxone");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-0001" Title="Page1" WF="16384">
    <C Type="SequenceController" V="175" U="seq-0001" Title="TestSeq" WF="16384">
      <Co K="S1" U="seq-s1"/>
      <Co K="AQ1" U="seq-aq1"/>
    </C>
  </C>
</ControlList>"#;
        fs::write(&path, xml).unwrap();
        let file = path.to_str().unwrap().to_string();

        // Set a valid program
        let result = cmd_config(
            &ctx(),
            ConfigCmd::SetProgram {
                file: file.clone(),
                selector: "TestSeq".to_string(),
                program: Some("sequence 1\nset AQ1 = 1\nsleep 5 s\nset AQ1 = 0".to_string()),
                program_file: None,
                save_as: None,
            },
        );
        assert!(result.is_ok());

        // Get it back
        let result = cmd_config(
            &ctx(),
            ConfigCmd::GetProgram {
                file: file.clone(),
                selector: "TestSeq".to_string(),
            },
        );
        assert!(result.is_ok());

        // Verify the XML has the program text
        let data = fs::read_to_string(&file).unwrap();
        assert!(data.contains("set AQ1 = 1"));
        assert!(data.contains("sleep 5 s"));
    }

    #[test]
    fn test_cmd_set_program_invalid_syntax() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seq2.Loxone");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-0001" Title="Page1" WF="16384">
    <C Type="SequenceController" V="175" U="seq-0002" Title="BadSeq" WF="16384">
      <Co K="S1" U="seq2-s1"/>
    </C>
  </C>
</ControlList>"#;
        fs::write(&path, xml).unwrap();
        let file = path.to_str().unwrap().to_string();

        // Try to set an invalid program
        let result = cmd_config(
            &ctx(),
            ConfigCmd::SetProgram {
                file,
                selector: "BadSeq".to_string(),
                program: Some("seet AQ1 = 5\nsleep".to_string()),
                program_file: None,
                save_as: None,
            },
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("syntax error"), "error: {}", err);
    }

    #[test]
    fn test_cmd_set_program_wrong_type() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::SetProgram {
                file,
                selector: "TestAnd".to_string(),
                program: Some("set AQ1 = 1".to_string()),
                program_file: None,
                save_as: None,
            },
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("only for SequenceController"),
            "error: {}",
            err
        );
    }

    #[test]
    fn test_cmd_get_program_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seq3.Loxone");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-0001" Title="Page1" WF="16384">
    <C Type="SequenceController" V="175" U="seq-0003" Title="EmptySeq" WF="16384"/>
  </C>
</ControlList>"#;
        fs::write(&path, xml).unwrap();
        let file = path.to_str().unwrap().to_string();

        let result = cmd_config(
            &ctx(),
            ConfigCmd::GetProgram {
                file,
                selector: "EmptySeq".to_string(),
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_sequence_controller_no_program() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seq4.Loxone");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-0001" Title="Page1" WF="16384">
    <C Type="SequenceController" V="175" U="seq-0004" Title="NoProgSeq" WF="16384">
      <Co K="S1" U="seq4-s1"/>
    </C>
  </C>
</ControlList>"#;
        fs::write(&path, xml).unwrap();
        let file = path.to_str().unwrap().to_string();

        let result = cmd_config(
            &ctx(),
            ConfigCmd::Check {
                file,
                selector: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_sequence_controller_with_valid_program() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seq5.Loxone");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-0001" Title="Page1" WF="16384">
    <C Type="SequenceController" V="175" U="seq-0005" Title="GoodSeq" WF="16384">
      <Co K="S1" U="seq5-s1"/>
      <Field Name="Configuration">sequence 1
set AQ1 = 1
sleep 5 s
set AQ1 = 0</Field>
    </C>
  </C>
</ControlList>"#;
        fs::write(&path, xml).unwrap();
        let file = path.to_str().unwrap().to_string();

        let result = cmd_config(
            &ctx(),
            ConfigCmd::Check {
                file,
                selector: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_sequence_controller_with_bad_program() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seq6.Loxone");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Page" V="175" U="p-0001" Title="Page1" WF="16384">
    <C Type="SequenceController" V="175" U="seq-0006" Title="BadProgSeq" WF="16384">
      <Co K="S1" U="seq6-s1"/>
      <Field Name="Configuration">seet AQ1 = 5
sleep</Field>
    </C>
  </C>
</ControlList>"#;
        fs::write(&path, xml).unwrap();
        let file = path.to_str().unwrap().to_string();

        // Check should report errors (and bail because errors > 0)
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Check {
                file,
                selector: None,
            },
        );
        assert!(result.is_err(), "check should fail with bad program");
    }

    #[test]
    fn test_cmd_template_bedroom() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Template {
                file: file.clone(),
                template: "bedroom".to_string(),
                room: "Room1".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read_to_string(&file).unwrap();
        assert!(data.contains("Room1 Licht"));
        assert!(data.contains("LightController2"));
    }

    #[test]
    fn test_cmd_template_bad_name() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Template {
                file,
                template: "nonexistent".to_string(),
                room: "Room1".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_template_bad_room() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Template {
                file,
                template: "standard".to_string(),
                room: "NoSuchRoom".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_room_add() {
        let (_dir, file) = fixture_file();
        let result = cmd_room(
            &ctx(),
            RoomCmd::Add {
                file: file.clone(),
                name: "NewRoom".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read_to_string(&file).unwrap();
        assert!(data.contains("NewRoom"));
    }

    #[test]
    fn test_cmd_room_add_duplicate() {
        let (_dir, file) = fixture_file();
        let result = cmd_room(
            &ctx(),
            RoomCmd::Add {
                file,
                name: "Room1".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_diff_identical() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Diff {
                file1: file.clone(),
                file2: file,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_diff_different() {
        let (dir, file1) = fixture_file();
        let file2 = dir.path().join("modified.Loxone");
        let mut data = fs::read_to_string(&file1).unwrap();
        data = data.replace("Room1", "RenamedRoom");
        fs::write(&file2, &data).unwrap();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Diff {
                file1,
                file2: file2.to_str().unwrap().to_string(),
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_add_virtual_in() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::AddVirtualIn {
                file: file.clone(),
                title: "TestVIn".to_string(),
                analog: false,
                parent: Some("Type:Page".to_string()),
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read_to_string(&file).unwrap();
        assert!(data.contains("TestVIn"));
        assert!(data.contains("VirtualIn"));
    }

    #[test]
    fn test_cmd_wire_connector() {
        let (_dir, file) = fixture_file();
        // Wire I2 → I1 (just testing the command path, not logical correctness)
        let result = cmd_config(
            &ctx(),
            ConfigCmd::WireConnector {
                file: file.clone(),
                target: "TestAnd.I2".to_string(),
                source_uuid: "i-0001-0001-0001-ffff000000000001".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_user_add_and_remove() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::UserAdd {
                file: file.clone(),
                name: "testuser".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read_to_string(&file).unwrap();
        assert!(data.contains("testuser"));

        let result = cmd_config(
            &ctx(),
            ConfigCmd::UserRemove {
                file: file.clone(),
                name: "testuser".to_string(),
                save_as: None,
            },
        );
        assert!(result.is_ok());
        let data = fs::read_to_string(&file).unwrap();
        assert!(!data.contains("testuser"));
    }

    #[test]
    fn test_cmd_autopilot_list() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::AutopilotList { file });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cmd_mode_list() {
        let (_dir, file) = fixture_file();
        let result = cmd_config(&ctx(), ConfigCmd::ModeList { file });
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_block_type_valid() {
        let (name, _parent) = resolve_block_type("And").unwrap();
        assert_eq!(name, "And");
    }

    #[test]
    fn test_resolve_block_type_alias() {
        let (name, _parent) = resolve_block_type("light").unwrap();
        assert_eq!(name, "LightController2");
    }

    #[test]
    fn test_resolve_block_type_fuzzy() {
        let result = resolve_block_type("Andd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Did you mean"));
    }

    #[test]
    fn test_resolve_block_type_unknown() {
        let result = resolve_block_type("TotallyFakeType");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_edited_atomic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.Loxone");
        fs::write(&path, FIXTURE_XML).unwrap();
        let editor = ConfigEditor::load(FIXTURE_XML.as_bytes()).unwrap();
        save_edited(&editor, path.to_str().unwrap(), None).unwrap();
        // Verify no .tmp file left behind
        assert!(!dir.path().join("test.Loxone.tmp").exists());
        // Verify content is valid
        let data = fs::read(&path).unwrap();
        assert!(String::from_utf8_lossy(&data).contains("ControlList"));
    }

    #[test]
    fn test_cmd_rejects_zip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.zip");
        fs::write(&path, "fake").unwrap();
        let result = cmd_config(
            &ctx(),
            ConfigCmd::Rooms {
                file: path.to_str().unwrap().to_string(),
                limit: None,
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("extract"));
    }

    // ── HTTP mock tests (httpmock) ───────────────────────────────────────

    /// Build a Config pointing at a mock server URL with a temp data_dir.
    fn mock_cfg(base_url: &str, dir: &std::path::Path) -> Config {
        Config {
            host: base_url.to_string(),
            user: "admin".into(),
            pass: "test".into(),
            serial: "00000000".into(),
            data_dir: dir.to_path_buf(),
            ..Default::default()
        }
    }

    #[test]
    fn test_cache_check_via_mock() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/jdev/sps/LoxAPPversion3");
            then.status(200).json_body(serde_json::json!({
                "LL": { "value": "17.0.3.31", "Code": "200" }
            }));
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let lox = LoxClient::new(cfg).unwrap();
        let resp = lox.get_json("/jdev/sps/LoxAPPversion3").unwrap();

        let ver = resp.pointer("/LL/value").and_then(|v| v.as_str()).unwrap();
        assert_eq!(ver, "17.0.3.31");
        mock.assert();
    }

    #[test]
    fn test_cache_check_json_output() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/jdev/sps/LoxAPPversion3");
            then.status(200).json_body(serde_json::json!({
                "LL": { "value": "17.0.3.31", "Code": "200" }
            }));
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let lox = LoxClient::new(cfg).unwrap();
        let resp = lox.get_json("/jdev/sps/LoxAPPversion3").unwrap();

        let remote_ver = resp
            .pointer("/LL/value")
            .and_then(json_val_str)
            .unwrap_or_else(|| "?".to_string());
        let cache = LoxClient::cache_path(&mock_cfg(&server.base_url(), dir.path()));
        let output = serde_json::json!({
            "remote_version": remote_ver,
            "cache_exists": cache.exists(),
        });
        assert_eq!(output["remote_version"], "17.0.3.31");
        assert_eq!(output["cache_exists"], false);
    }

    #[test]
    fn test_cache_check_server_error() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/jdev/sps/LoxAPPversion3");
            then.status(401);
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let lox = LoxClient::new(cfg).unwrap();
        let result = lox.get_json("/jdev/sps/LoxAPPversion3");
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_refresh_via_mock() {
        use crate::client::{LoxClient, USER_AGENT};
        use httpmock::prelude::*;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "msInfo": { "serialNr": "00:00:00:00:00:00" },
                "controls": {},
                "rooms": {},
                "cats": {}
            }));
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .danger_accept_invalid_certs(true)
            .redirect(LoxClient::same_origin_redirect_policy(&cfg.host))
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let structure = LoxClient::load_or_fetch_structure(&cfg, &client).unwrap();
        assert!(structure.get("controls").is_some());
        assert!(structure.get("rooms").is_some());
        mock.assert();
    }

    #[test]
    fn test_cache_refresh_no_disk_cache_for_localhost() {
        use crate::client::{LoxClient, USER_AGENT};
        use httpmock::prelude::*;

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "controls": {}
            }));
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        // Fetch should succeed but NOT write a disk cache for localhost
        LoxClient::load_or_fetch_structure(&cfg, &client).unwrap();
        let cache_path = LoxClient::cache_path(&cfg);
        assert!(
            !cache_path.exists(),
            "disk cache should be skipped for localhost"
        );
    }

    #[test]
    fn test_token_check_valid_via_mock() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());

        // Write a minimal token.json so TokenStore::load_for works
        let ts = token::TokenStore {
            token: "abc123".into(),
            key: "deadbeef".into(),
            valid_until: u64::MAX,
        };
        ts.save_for(&cfg).unwrap();

        let hash = token::hash_token(&ts.token, &ts.key);

        let check_path = format!("/jdev/sys/checktoken/{}/{}", hash, cfg.user);
        let mock = server.mock(|when, then| {
            when.method(GET).path(check_path);
            then.status(200).json_body(serde_json::json!({
                "LL": { "Code": "200", "value": "ok" }
            }));
        });

        let loaded_ts = token::TokenStore::load_for(&cfg).unwrap();
        let lox = LoxClient::new(cfg.clone()).unwrap();
        let loaded_hash = token::hash_token(&loaded_ts.token, &loaded_ts.key);
        let resp = lox
            .get_json(&format!(
                "/jdev/sys/checktoken/{}/{}",
                loaded_hash, cfg.user
            ))
            .unwrap();

        let code = resp
            .pointer("/LL/Code")
            .and_then(json_val_str)
            .unwrap_or_else(|| "?".to_string());
        assert_eq!(code, "200");
        mock.assert();
    }

    #[test]
    fn test_token_check_invalid_via_mock() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());

        let ts = token::TokenStore {
            token: "expired_token".into(),
            key: "deadbeef".into(),
            valid_until: 1,
        };
        ts.save_for(&cfg).unwrap();

        let hash = token::hash_token(&ts.token, &ts.key);
        let check_path = format!("/jdev/sys/checktoken/{}/{}", hash, cfg.user);
        server.mock(|when, then| {
            when.method(GET).path(check_path);
            then.status(200).json_body(serde_json::json!({
                "LL": { "Code": "401", "value": "invalid" }
            }));
        });

        let loaded_ts = token::TokenStore::load_for(&cfg).unwrap();
        let lox = LoxClient::new(cfg.clone()).unwrap();
        let loaded_hash = token::hash_token(&loaded_ts.token, &loaded_ts.key);
        let resp = lox
            .get_json(&format!(
                "/jdev/sys/checktoken/{}/{}",
                loaded_hash, cfg.user
            ))
            .unwrap();

        let code = resp
            .pointer("/LL/Code")
            .and_then(json_val_str)
            .unwrap_or_else(|| "?".to_string());
        assert_eq!(code, "401");
    }

    #[test]
    fn test_token_refresh_via_mock() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());

        let ts = token::TokenStore {
            token: "mytoken".into(),
            key: "deadbeef".into(),
            valid_until: u64::MAX,
        };
        ts.save_for(&cfg).unwrap();

        let hash = token::hash_token(&ts.token, &ts.key);
        let refresh_path = format!("/jdev/sys/refreshtoken/{}/{}", hash, cfg.user);
        let new_valid = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 86400 * 30;
        server.mock(|when, then| {
            when.method(GET).path(refresh_path);
            then.status(200).json_body(serde_json::json!({
                "LL": {
                    "Code": "200",
                    "value": { "validUntil": new_valid }
                }
            }));
        });

        let loaded_ts = token::TokenStore::load_for(&cfg).unwrap();
        let lox = LoxClient::new(cfg.clone()).unwrap();
        let loaded_hash = token::hash_token(&loaded_ts.token, &loaded_ts.key);
        let resp = lox
            .get_json(&format!(
                "/jdev/sys/refreshtoken/{}/{}",
                loaded_hash, cfg.user
            ))
            .unwrap();

        let code = resp
            .pointer("/LL/Code")
            .and_then(json_val_str)
            .unwrap_or_else(|| "?".to_string());
        assert_eq!(code, "200");

        let valid_until = resp
            .pointer("/LL/value")
            .and_then(|v| v.get("validUntil"))
            .and_then(|v| v.as_u64())
            .unwrap();
        assert_eq!(valid_until, new_valid);
    }

    // ── LoxCC extract/compress pipeline tests ────────────────────────────

    #[test]
    fn test_extract_and_compress_roundtrip() {
        use std::io::Write;
        let xml = FIXTURE_XML.as_bytes();

        // Build a ZIP containing sps0.LoxCC
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut zip_writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip_writer.start_file("sps0.LoxCC", options).unwrap();
        zip_writer.write_all(&loxcc::compress_loxcc(xml)).unwrap();
        let cursor = zip_writer.finish().unwrap();
        let zip_data = cursor.into_inner();

        // extract_and_decompress should recover the original XML
        let recovered = loxcc::extract_and_decompress(&zip_data).unwrap();
        assert_eq!(recovered, xml);
    }

    #[test]
    fn test_config_compress_then_extract() {
        let dir = TempDir::new().unwrap();
        let xml_path = dir.path().join("test.Loxone");
        fs::write(&xml_path, FIXTURE_XML).unwrap();

        // Compress (the same logic as ConfigCmd::Compress)
        let xml = fs::read(&xml_path).unwrap();
        let loxcc_data = loxcc::compress_loxcc(&xml);
        let loxcc_path = dir.path().join("test.LoxCC");
        fs::write(&loxcc_path, &loxcc_data).unwrap();

        // Decompress back and verify round-trip
        let decompressed = loxcc::decompress_loxcc(&loxcc_data).unwrap();
        assert_eq!(decompressed, xml);
    }

    #[test]
    fn test_config_repack_preserves_other_entries() {
        use std::io::{Read, Write};
        let xml = FIXTURE_XML.as_bytes();

        // Build source ZIP with sps0.LoxCC + extra files
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut zip_writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip_writer.start_file("sps0.LoxCC", options).unwrap();
        zip_writer.write_all(&loxcc::compress_loxcc(xml)).unwrap();
        zip_writer.start_file("LoxAPP3.json", options).unwrap();
        zip_writer
            .write_all(b"{\"controls\":{},\"rooms\":{}}")
            .unwrap();
        zip_writer.start_file("permissions.bin", options).unwrap();
        zip_writer.write_all(b"\x00\x01\x02\x03").unwrap();

        let cursor = zip_writer.finish().unwrap();
        let src_zip = cursor.into_inner();

        // Repack with modified XML
        let modified_xml = FIXTURE_XML.replace("Room1", "ModifiedRoom");
        let repacked = loxcc::repack_zip(&src_zip, modified_xml.as_bytes()).unwrap();

        // Verify new XML
        let extracted = loxcc::extract_and_decompress(&repacked).unwrap();
        assert!(String::from_utf8_lossy(&extracted).contains("ModifiedRoom"));

        // Verify other entries preserved
        let cursor = std::io::Cursor::new(&repacked);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut json_data = Vec::new();
        archive
            .by_name("LoxAPP3.json")
            .unwrap()
            .read_to_end(&mut json_data)
            .unwrap();
        assert_eq!(json_data, b"{\"controls\":{},\"rooms\":{}}");

        let mut perm_data = Vec::new();
        archive
            .by_name("permissions.bin")
            .unwrap()
            .read_to_end(&mut perm_data)
            .unwrap();
        assert_eq!(perm_data, b"\x00\x01\x02\x03");
    }

    #[test]
    fn test_list_controls_via_mock() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "msInfo": { "serialNr": "00:00:00:00:00:00" },
                "rooms": {
                    "r-uuid-1": { "name": "Living Room", "uuid": "r-uuid-1" }
                },
                "cats": {
                    "c-uuid-1": { "name": "Lighting", "uuid": "c-uuid-1" }
                },
                "controls": {
                    "ctrl-uuid-1": {
                        "name": "Main Light",
                        "type": "Switch",
                        "room": "r-uuid-1",
                        "cat": "c-uuid-1",
                        "isFavorite": true,
                        "isSecured": false
                    },
                    "ctrl-uuid-2": {
                        "name": "Dimmer",
                        "type": "Dimmer",
                        "room": "r-uuid-1",
                        "cat": "c-uuid-1",
                        "isFavorite": false,
                        "isSecured": false
                    }
                }
            }));
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let mut lox = LoxClient::new(cfg).unwrap();

        let controls = lox.list_controls(None, None).unwrap();
        assert_eq!(controls.len(), 2);
        let names: Vec<&str> = controls.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"Main Light"));
        assert!(names.contains(&"Dimmer"));

        // Filter by type
        let switches = lox.list_controls(Some("Switch"), None).unwrap();
        assert_eq!(switches.len(), 1);
        assert_eq!(switches[0].name, "Main Light");

        // Filter by room
        let room_ctrls = lox.list_controls(None, Some("Living Room")).unwrap();
        assert_eq!(room_ctrls.len(), 2);
    }

    #[test]
    fn test_send_cmd_via_mock() {
        use crate::client::LoxClient;
        use httpmock::prelude::*;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/jdev/sps/io/0b4e0ea5-0191-11b5-ffff504f94000000/on");
            then.status(200).json_body(serde_json::json!({
                "LL": { "Code": "200", "value": "1" }
            }));
        });

        let dir = TempDir::new().unwrap();
        let cfg = mock_cfg(&server.base_url(), dir.path());
        let lox = LoxClient::new(cfg).unwrap();

        let resp = lox
            .send_cmd("0b4e0ea5-0191-11b5-ffff504f94000000", "on")
            .unwrap();
        let code = resp.pointer("/LL/Code").and_then(|v| v.as_str()).unwrap();
        assert_eq!(code, "200");
        mock.assert();
    }
}
