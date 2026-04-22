use anyhow::{Context, Result, bail};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::CtxCmd;
use crate::commands::RunContext;
use crate::config::{Config, ContextEntry, GlobalConfig, validate_context_name};

pub fn cmd_ctx(ctx: &RunContext, action: CtxCmd) -> Result<()> {
    match action {
        CtxCmd::Add {
            name,
            host,
            user,
            pass,
            serial,
        } => ctx_add(ctx, name, host, user, pass, serial),
        CtxCmd::Use { name } => ctx_use(ctx, name),
        CtxCmd::List => ctx_list(ctx),
        CtxCmd::Current => ctx_current(ctx),
        CtxCmd::Remove { name } => ctx_remove(ctx, name),
        CtxCmd::Rename { old, new } => ctx_rename(ctx, old, new),
        CtxCmd::Init {
            host,
            user,
            pass,
            serial,
        } => ctx_init(ctx, host, user, pass, serial),
        CtxCmd::Migrate => ctx_migrate(ctx),
        CtxCmd::External(args) => {
            if args.len() == 1 {
                // `lox ctx <name>` is a shortcut for `lox ctx use <name>`
                ctx_use(ctx, args[0].clone())
            } else {
                bail!(
                    "Unknown ctx subcommand '{}'. Run: lox ctx --help",
                    args.join(" ")
                );
            }
        }
    }
}

fn ctx_add(
    ctx: &RunContext,
    name: String,
    host: String,
    user: String,
    pass: String,
    serial: Option<String>,
) -> Result<()> {
    validate_context_name(&name)?;
    let mut global = GlobalConfig::load_or_default();

    if global.contexts.contains_key(&name) {
        bail!(
            "Context '{}' already exists. Remove it first: lox ctx remove {}",
            name,
            name
        );
    }

    let mut host_normalized = host;
    if !host_normalized.starts_with("http://") && !host_normalized.starts_with("https://") {
        host_normalized = format!("https://{}", host_normalized);
    }

    let entry = ContextEntry {
        host: host_normalized.clone(),
        user: user.clone(),
        pass,
        serial: serial.unwrap_or_default(),
        ..Default::default()
    };

    global.contexts.insert(name.clone(), entry);

    // If no active context is set, make this one active
    if global.active_context.is_none() {
        global.active_context = Some(name.clone());
    }

    // Create the context data directory
    let data_dir = Config::context_data_dir(&name);
    fs::create_dir_all(data_dir.join("cache"))?;

    global.save()?;

    if ctx.json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "context": name,
                "host": host_normalized,
                "user": user,
            })
        );
    } else if !ctx.quiet {
        println!("✓  Added context '{}'", name);
        println!("   host: {}", host_normalized);
        println!("   user: {}", user);
        if global.active_context.as_deref() == Some(&name) {
            println!("   (set as active context)");
        }
    }
    Ok(())
}

fn ctx_use(ctx: &RunContext, name: String) -> Result<()> {
    let mut global = GlobalConfig::load_or_default();

    if !global.contexts.contains_key(&name) {
        bail!(
            "Context '{}' not found. Available: {}",
            name,
            global
                .contexts
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    global.active_context = Some(name.clone());
    global.save()?;

    if ctx.json {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "active_context": name })
        );
    } else if !ctx.quiet {
        println!("✓  Switched to context '{}'", name);
    }
    Ok(())
}

fn ctx_list(ctx: &RunContext) -> Result<()> {
    let global = GlobalConfig::load_or_default();

    if global.contexts.is_empty() {
        if ctx.json {
            println!("{}", serde_json::json!({ "contexts": [] }));
        } else {
            println!(
                "No contexts configured. Add one: lox ctx add <name> --host ... --user ... --pass ..."
            );
        }
        return Ok(());
    }

    if ctx.json {
        let contexts: Vec<serde_json::Value> = global
            .contexts
            .iter()
            .map(|(name, entry)| {
                serde_json::json!({
                    "name": name,
                    "host": entry.host,
                    "user": entry.user,
                    "active": global.active_context.as_deref() == Some(name.as_str()),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({ "contexts": contexts }))?
        );
    } else {
        let mut names: Vec<_> = global.contexts.keys().collect();
        names.sort();
        for name in names {
            let entry = &global.contexts[name];
            let marker = if global.active_context.as_deref() == Some(name.as_str()) {
                "* "
            } else {
                "  "
            };
            println!("{}{:<16} {:<40} {}", marker, name, entry.host, entry.user);
        }
    }
    Ok(())
}

fn ctx_current(ctx: &RunContext) -> Result<()> {
    // Check for project-local first
    if let Some(local_dir) = crate::config::find_local_lox_dir() {
        if ctx.json {
            println!(
                "{}",
                serde_json::json!({
                    "context": null,
                    "local": true,
                    "path": local_dir.to_string_lossy(),
                })
            );
        } else {
            println!("(local) {}", local_dir.display());
        }
        return Ok(());
    }

    let global = GlobalConfig::load_or_default();
    if let Some(ref active) = global.active_context {
        if ctx.json {
            let entry = global.contexts.get(active);
            println!(
                "{}",
                serde_json::json!({
                    "context": active,
                    "local": false,
                    "host": entry.map(|e| e.host.as_str()),
                })
            );
        } else {
            println!("{}", active);
        }
    } else if ctx.json {
        println!("{}", serde_json::json!({ "context": null, "local": false }));
    } else {
        println!("No active context. Run: lox ctx use <name>");
    }
    Ok(())
}

fn ctx_remove(ctx: &RunContext, name: String) -> Result<()> {
    let mut global = GlobalConfig::load_or_default();

    if global.contexts.remove(&name).is_none() {
        bail!("Context '{}' not found", name);
    }

    // Clear active if it was the removed context
    if global.active_context.as_deref() == Some(&name) {
        global.active_context = None;
    }

    global.save()?;

    // Optionally remove context data directory
    let data_dir = Config::context_data_dir(&name);
    if data_dir.exists() {
        let _ = fs::remove_dir_all(&data_dir);
    }

    if ctx.json {
        println!("{}", serde_json::json!({ "ok": true, "removed": name }));
    } else if !ctx.quiet {
        println!("✓  Removed context '{}'", name);
    }
    Ok(())
}

fn ctx_rename(ctx: &RunContext, old: String, new: String) -> Result<()> {
    validate_context_name(&new)?;
    let mut global = GlobalConfig::load_or_default();

    let entry = global
        .contexts
        .remove(&old)
        .with_context(|| format!("Context '{}' not found", old))?;

    if global.contexts.contains_key(&new) {
        // Put it back
        global.contexts.insert(old, entry);
        bail!("Context '{}' already exists", new);
    }

    global.contexts.insert(new.clone(), entry);

    // Update active context reference
    if global.active_context.as_deref() == Some(&old) {
        global.active_context = Some(new.clone());
    }

    // Rename data directory
    let old_dir = Config::context_data_dir(&old);
    let new_dir = Config::context_data_dir(&new);
    if old_dir.exists() {
        fs::create_dir_all(new_dir.parent().unwrap_or(&Config::dir()))?;
        fs::rename(&old_dir, &new_dir)?;
    }

    global.save()?;

    if ctx.json {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "old": old, "new": new })
        );
    } else if !ctx.quiet {
        println!("✓  Renamed context '{}' → '{}'", old, new);
    }
    Ok(())
}

fn ctx_init(
    ctx: &RunContext,
    host: Option<String>,
    user: Option<String>,
    pass: Option<String>,
    serial: Option<String>,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let lox_dir = cwd.join(".lox");

    if lox_dir.join("config.yaml").exists() {
        bail!("Project-local .lox/ already exists in {}", cwd.display());
    }

    // Create directory structure
    fs::create_dir_all(lox_dir.join("cache"))?;
    fs::create_dir_all(lox_dir.join("scenes"))?;

    // Create config.yaml (with provided values or empty placeholders)
    let cfg = Config {
        host: host.unwrap_or_default(),
        user: user.unwrap_or_default(),
        pass: pass.unwrap_or_default(),
        serial: serial.unwrap_or_default(),
        ..Default::default()
    };
    let config_path = lox_dir.join("config.yaml");
    fs::write(&config_path, serde_yaml::to_string(&cfg)?)?;
    #[cfg(unix)]
    let _ = fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600));

    // Create .gitignore
    let gitignore = "# Secrets — never commit\nconfig.yaml\ntoken.json\n\n# Cache — regenerated automatically\ncache/\n";
    fs::write(lox_dir.join(".gitignore"), gitignore)?;

    if ctx.json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "path": lox_dir.to_string_lossy(),
            })
        );
    } else if !ctx.quiet {
        println!(
            "✓  Initialized project-local config at {}",
            lox_dir.display()
        );
        println!("   Created:");
        println!("     .lox/config.yaml    — connection settings");
        println!("     .lox/.gitignore     — excludes secrets and cache");
        println!("     .lox/cache/         — structure cache");
        println!("     .lox/scenes/        — scene definitions");
        if cfg.host.is_empty() {
            println!("\n   Next: edit .lox/config.yaml or run:");
            println!("     lox setup set --host <host> --user <user> --pass <pass>");
        }
    }
    Ok(())
}

fn ctx_migrate(ctx: &RunContext) -> Result<()> {
    if GlobalConfig::is_multi_context() {
        bail!("Config is already in multi-context format");
    }

    let flat = GlobalConfig::load_flat_config()?;
    let entry = ContextEntry::from(&flat);

    let mut global = GlobalConfig {
        active_context: Some("default".to_string()),
        ..Default::default()
    };
    global.contexts.insert("default".to_string(), entry);

    // Create context data directory and move existing cache/token
    let data_dir = Config::context_data_dir("default");
    fs::create_dir_all(data_dir.join("cache"))?;

    // Move existing cache
    let old_cache = Config::dir().join("cache").join("structure.json");
    let new_cache = data_dir.join("cache").join("structure.json");
    if old_cache.exists() {
        fs::rename(&old_cache, &new_cache)?;
    }

    // Move existing token
    let old_token = Config::dir().join("token.json");
    let new_token = data_dir.join("token.json");
    if old_token.exists() {
        fs::rename(&old_token, &new_token)?;
    }

    global.save()?;

    if ctx.json {
        println!(
            "{}",
            serde_json::json!({ "ok": true, "context": "default" })
        );
    } else if !ctx.quiet {
        println!("✓  Migrated flat config to context 'default'");
        println!("   Active context: default");
    }
    Ok(())
}
