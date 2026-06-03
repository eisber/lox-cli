#[allow(dead_code)]
mod client;
mod commands;
#[allow(dead_code)]
mod config;
mod config_edit;
mod errors;
mod ftp;
mod gitops;
mod loxcc;
mod loxone_xml;
#[allow(dead_code)]
mod rc6;
mod telemetry;
#[allow(dead_code)]
mod token;
#[allow(dead_code)]
mod ws;
#[allow(dead_code)]
mod wsx;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use dirs::home_dir;
use serde_json::Value;
use std::fs;

/// Load config XML from a .Loxone file or extract from a .zip backup.
pub(crate) fn load_config_xml(path: &str) -> Result<Vec<u8>> {
    let data = fs::read(path).with_context(|| format!("Cannot read {}", path))?;
    if path.ends_with(".zip") {
        loxcc::extract_and_decompress(&data)
    } else {
        Ok(data)
    }
}

/// Extract a JSON value as a display string, handling both string and numeric types.
pub(crate) fn json_val_str(v: &Value) -> Option<String> {
    v.as_str()
        .map(|s| s.to_string())
        .or_else(|| v.as_i64().map(|n| n.to_string()))
        .or_else(|| v.as_f64().map(|n| n.to_string()))
}

pub(crate) fn detect_shell() -> Option<Shell> {
    if let Some(shell) = std::env::var("SHELL").ok().and_then(|s| {
        let name = s.rsplit('/').next().unwrap_or(&s);
        match name {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            "elvish" => Some(Shell::Elvish),
            "pwsh" | "powershell" => Some(Shell::PowerShell),
            _ => None,
        }
    }) {
        return Some(shell);
    }
    if std::env::var("PSModulePath").is_ok() {
        return Some(Shell::PowerShell);
    }
    None
}

pub(crate) fn install_completions(shell: Shell, cmd: &mut clap::Command) -> Result<()> {
    let home = std::env::var("LOX_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(home_dir)
        .unwrap_or_default();
    let (path, label) = match shell {
        Shell::Bash => {
            let dir = home.join(".local/share/bash-completion/completions");
            fs::create_dir_all(&dir)?;
            (dir.join("lox"), "bash")
        }
        Shell::Zsh => {
            let dir = home.join(".zfunc");
            fs::create_dir_all(&dir)?;
            (dir.join("_lox"), "zsh")
        }
        Shell::Fish => {
            let dir = home.join(".config/fish/completions");
            fs::create_dir_all(&dir)?;
            (dir.join("lox.fish"), "fish")
        }
        Shell::PowerShell => {
            let documents = if cfg!(windows) {
                home.join("Documents")
            } else {
                home.join(".config")
            };
            let dir = documents.join("PowerShell");
            fs::create_dir_all(&dir)?;
            (dir.join("lox_completions.ps1"), "powershell")
        }
        Shell::Elvish => {
            bail!("Elvish completions must be installed manually — run: lox completions elvish");
        }
        _ => bail!("Unsupported shell"),
    };
    let mut buf = Vec::new();
    generate(shell, cmd, "lox", &mut buf);
    fs::write(&path, &buf)?;

    eprintln!("Installed {} completions to {}", label, path.display());
    if shell == Shell::Zsh {
        eprintln!("Ensure ~/.zfunc is in your fpath. Add to ~/.zshrc:");
        eprintln!("  fpath=(~/.zfunc $fpath)");
        eprintln!("  autoload -Uz compinit && compinit");
    }
    if shell == Shell::PowerShell {
        eprintln!("Add this line to your PowerShell $PROFILE to load completions:");
        eprintln!("  . \"{}\"", path.display());
    }
    eprintln!("Restart your shell or open a new tab to activate.");
    Ok(())
}

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "lox",
    about = "Loxone Config-as-Code CLI",
    version,
    infer_subcommands = true,
    disable_help_subcommand = true,
    help_template = "\
{about-with-newline}
{usage-heading} {usage}

Configuration:
  setup                          Connection settings
  cache                          Structure cache management
  token                          Auth token management
  ctx                            Manage multiple Miniserver contexts
  config                         Loxone Config files (download/inspect/edit/push)
  telemetry                      Manage anonymous usage analytics
  completions                    Generate shell completions
  schema                         Command schema for AI agent discovery

{options}{after-help}"
)]
pub(crate) struct Cli {
    /// Output format: json, csv, or table (default)
    #[arg(long, short = 'o', global = true, value_enum, default_value = "table")]
    output: OutputFormat,
    /// Shorthand for --output json
    #[arg(long = "json", global = true)]
    json_flag: bool,
    /// Suppress non-essential output
    #[arg(long, short = 'q', global = true)]
    quiet: bool,
    /// Disable colored output (also respects NO_COLOR env var)
    #[arg(long, global = true)]
    no_color: bool,
    /// Suppress table headers (useful for piping)
    #[arg(long, global = true)]
    no_header: bool,
    /// Verbose output (-v for requests, -vv for request+response bodies)
    #[arg(short = 'v', long, global = true, action = ArgAction::Count)]
    verbose: u8,
    /// Validate and resolve inputs without executing commands
    #[arg(long, global = true)]
    dry_run: bool,
    /// Fail instead of prompting for confirmation (implied by --output json)
    #[arg(long, global = true)]
    non_interactive: bool,
    /// Trace ID for correlating agent actions in logs
    #[arg(long, global = true)]
    trace_id: Option<String>,
    /// Use a specific context (overrides active context)
    #[arg(long = "ctx", global = true)]
    context: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    /// Human-readable table (default)
    Table,
    /// JSON output
    Json,
    /// CSV output (where applicable)
    Csv,
}

#[derive(Subcommand)]
pub(crate) enum Cmd {
    // ── Configuration ────────────────────────────────────────────────────────
    /// Configure connection settings
    Setup {
        #[command(subcommand)]
        action: SetupCmd,
    },
    /// Manage local cache
    Cache {
        #[command(subcommand)]
        action: CacheCmd,
    },
    /// Manage auth token (more secure than Basic Auth)
    Token {
        #[command(subcommand)]
        action: TokenCmd,
    },
    /// Manage anonymous usage telemetry
    Telemetry {
        #[command(subcommand)]
        action: TelemetryCmd,
    },
    /// Download, inspect, and manage Loxone Config files
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Manage multiple Miniserver contexts
    Ctx {
        #[command(subcommand)]
        action: CtxCmd,
    },
    /// Generate or install shell completions
    Completions {
        /// Shell to generate completions for (auto-detected if omitted)
        shell: Option<Shell>,
        /// Install completions to the standard location for your shell
        #[arg(long)]
        install: bool,
    },
    /// Show command schema for AI agent discovery
    Schema {
        /// Show schema for a specific command (e.g. "config", "setup")
        command: Option<String>,
    },
    /// Run Loxone SPS simulator against config files
    Sim {
        #[command(subcommand)]
        action: commands::sim_cmd::SimCmd,
    },
    /// Search, inspect, and list Loxone automation block types
    Blocks {
        #[command(subcommand)]
        action: BlocksCmd,
    },
    /// Browse Loxone KB articles, datasheets, and technical documentation
    Docs {
        /// Search term or article slug (e.g. "lighting-controller", "mqtt", "nano")
        query: Option<String>,
        /// Show datasheets instead of KB articles
        #[arg(long)]
        datasheet: bool,
        /// Show element schema (inputs, outputs, parameters, properties)
        #[arg(long)]
        schema: bool,
        /// List all available articles
        #[arg(long)]
        list: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum BlocksCmd {
    /// Search for block types by intent or keyword (e.g. "timed light", "blind wind")
    Search {
        /// Search query (natural language or block name)
        query: String,
    },
    /// Show full details for a block type (inputs, outputs, parameters)
    Info {
        /// Block xml_type (e.g. StairwayLS, And, JalousieUpDown2)
        block_type: String,
    },
    /// List block types, optionally filtered by category
    #[command(alias = "ls")]
    List {
        /// Filter by category (logic, math, compare, timer, schedule, state, lighting, shading, hvac, security, energy, io, button, misc)
        #[arg(long)]
        category: Option<String>,
        /// Maximum number of results to display
        #[arg(long, default_value = "100")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub(crate) enum TokenCmd {
    /// Fetch and save a new token (valid 20 days)
    Fetch,
    /// Show current token status
    Info,
    /// Delete saved token (revert to Basic Auth)
    Clear,
    /// Check if token is still valid on the Miniserver
    Check,
    /// Refresh token to extend validity
    Refresh,
    /// Revoke token on the Miniserver
    Revoke,
}

#[derive(Subcommand)]
pub(crate) enum TelemetryCmd {
    /// Opt in to anonymous usage analytics
    Enable,
    /// Opt out of anonymous usage analytics
    Disable,
    /// Show current telemetry setting
    Status,
}

#[derive(Subcommand)]
pub(crate) enum CacheCmd {
    /// Show cache info
    Info,
    /// Clear structure cache (forces fresh fetch)
    Clear,
    /// Refresh structure cache now
    Refresh,
    /// Check if cache is current (without full download)
    Check,
}

#[derive(Subcommand)]
pub(crate) enum CtxCmd {
    /// Add a new context
    Add {
        /// Context name
        name: String,
        /// Miniserver host URL
        #[arg(long)]
        host: String,
        /// Username
        #[arg(long)]
        user: String,
        /// Password
        #[arg(long)]
        pass: String,
        /// Miniserver serial number (for DynDNS TLS)
        #[arg(long)]
        serial: Option<String>,
    },
    /// Switch the active context
    Use {
        /// Context name to activate
        name: String,
    },
    /// List all contexts (* = active)
    #[command(alias = "ls")]
    List,
    /// Show the current active context
    Current,
    /// Remove a context
    Remove {
        /// Context name to remove
        name: String,
    },
    /// Rename a context
    Rename {
        /// Current name
        old: String,
        /// New name
        new: String,
    },
    /// Initialize a project-local .lox/ directory
    Init {
        /// Miniserver host URL
        #[arg(long)]
        host: Option<String>,
        /// Username
        #[arg(long)]
        user: Option<String>,
        /// Password
        #[arg(long)]
        pass: Option<String>,
        /// Miniserver serial number
        #[arg(long)]
        serial: Option<String>,
    },
    /// Migrate flat config to a 'default' context
    Migrate,
    /// Shortcut: `lox ctx <name>` = `lox ctx use <name>`
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand)]
pub(crate) enum SetupCmd {
    /// Set one or more config fields (omitted fields are preserved)
    Set {
        #[arg(long, env = "LOX_HOST")]
        host: Option<String>,
        #[arg(long, env = "LOX_USER")]
        user: Option<String>,
        /// Password (or set LOX_PASS env var to avoid it appearing in the process table)
        #[arg(long, env = "LOX_PASS")]
        pass: Option<String>,
        #[arg(long, env = "LOX_SERIAL")]
        serial: Option<String>,
        /// Enable SSL certificate verification (for valid certs)
        #[arg(long)]
        verify_ssl: bool,
        /// Disable SSL certificate verification (default, for self-signed Miniserver certs)
        #[arg(long, conflicts_with = "verify_ssl")]
        no_verify_ssl: bool,
    },
    /// Show current config (password redacted)
    Show,
}

#[derive(Subcommand)]
pub(crate) enum ConfigCmd {
    /// Download the latest Loxone Config from the Miniserver via FTP
    Download {
        /// Custom output filename
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
        /// Also decompress LoxCC to XML
        #[arg(long)]
        extract: bool,
    },
    /// List available configs on the Miniserver
    #[command(alias = "list")]
    Ls,
    /// Decompress a local config ZIP to .Loxone XML
    Extract {
        /// Path to a config ZIP file
        file: String,
        /// Custom output filename
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// Upload a config to the Miniserver via FTP (dangerous — requires --force)
    Upload {
        /// Path to a config ZIP file
        file: String,
        /// Confirm the upload
        #[arg(long)]
        force: bool,
    },
    /// List user accounts from a .Loxone config file
    Users {
        /// Path to a .Loxone XML file (from `lox config extract`)
        file: String,
        /// Limit number of results displayed
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List hardware devices from a .Loxone config file
    Devices {
        /// Path to a .Loxone XML file (from `lox config extract`)
        file: String,
        /// Show all I/O ports with used/free status
        #[arg(long)]
        ports: bool,
        /// Optional: emit devices whose snapshot room label matches this room only
        #[arg(short, long)]
        room: Option<String>,
        /// Print device counts to stderr instead of emitting device records
        #[arg(long)]
        summary: bool,
        /// Limit number of results displayed
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Compare two config files (ZIP or .Loxone)
    Diff {
        /// First config file (older)
        file1: String,
        /// Second config file (newer)
        file2: String,
    },
    /// Initialize a git repository for config version tracking
    Init {
        /// Path for the config git repository
        path: String,
    },
    /// Download the latest config from the Miniserver and commit to git
    Pull {
        /// Suppress output (for cron usage)
        #[arg(long)]
        quiet: bool,
    },
    /// Show config change history from the git repository
    Log {
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },
    /// Restore a previous config version from git and upload to Miniserver
    Restore {
        /// Git commit hash to restore from
        commit: String,
        /// Confirm the upload (required — dangerous operation)
        #[arg(long)]
        force: bool,
    },
    /// Compress a .Loxone XML file into LoxCC format (with correct CRC32)
    Compress {
        /// Path to a .Loxone XML file
        file: String,
        /// Custom output filename
        #[arg(long, value_name = "PATH")]
        save_as: Option<String>,
    },
    /// List rooms and item counts from a .Loxone config file
    Rooms {
        /// Path to a .Loxone XML file (from `lox config extract`)
        file: String,
        /// Limit number of results displayed
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List controls with type, title, room, and category from a .Loxone config
    Controls {
        /// Path to a .Loxone XML file (from `lox config extract`)
        file: String,
        /// Filter by control type (e.g. WeatherData, SysVar)
        #[arg(short = 't', long)]
        r#type: Option<String>,
        /// Filter by room name
        #[arg(short, long)]
        room: Option<String>,
        /// Maximum number of results to display
        #[arg(long, default_value = "100")]
        limit: usize,
    },
    /// Download, patch, recompress, and upload a config in one step
    Patch {
        /// Replacement pairs: --replace OLD NEW (can repeat)
        #[arg(long, num_args = 2, action = clap::ArgAction::Append)]
        replace: Vec<String>,
        /// Reboot the Miniserver after upload
        #[arg(long)]
        reboot: bool,
        /// Confirm the operation (required — modifies live config)
        #[arg(long)]
        force: bool,
    },
    /// Upload a pre-edited .Loxone XML file to the Miniserver (recompress + upload)
    Push {
        /// Path to a .Loxone XML file
        file: String,
        /// Reboot the Miniserver after upload
        #[arg(long)]
        reboot: bool,
        /// Confirm the operation (required — modifies live config)
        #[arg(long)]
        force: bool,
    },
    /// Upload a config ZIP to the Miniserver via HTTP POST (fsput)
    PushHttp {
        /// Path to a config ZIP file
        file: String,
        /// Confirm the upload (required — dangerous operation)
        #[arg(long)]
        force: bool,
    },
    /// Manage rooms in a .Loxone config file
    #[command(subcommand)]
    Room(RoomCmd),
    /// Manage controls in a .Loxone config file
    #[command(subcommand)]
    Control(ControlCmd),
    /// Manage MQTT plugin configuration
    #[command(subcommand)]
    Mqtt(MqttConfigCmd),
    /// Add a control element with type-aware defaults
    Add {
        file: String,
        /// Control type: light, switch, presence, alarm-clock, memory, timer, mqtt-sub, mqtt-pub, calendar, autopilot
        #[arg(long = "type")]
        control_type: String,
        /// Element title
        #[arg(long)]
        title: String,
        /// Room name
        #[arg(long)]
        room: Option<String>,
        /// Category name
        #[arg(long)]
        category: Option<String>,
        /// Parent selector (required for mqtt-sub, mqtt-pub)
        #[arg(long)]
        parent: Option<String>,
        /// Page name to place block on (for visual layout in UX)
        #[arg(long)]
        page: Option<String>,
        /// MQTT topic (for mqtt-sub, mqtt-pub)
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Validate a .Loxone config file for common issues
    Validate { file: String },
    /// Check automation blocks for completeness (unset params, dead-end wiring)
    Check {
        /// Path to a .Loxone XML file
        file: String,
        /// Only check blocks matching this selector
        #[arg(long)]
        selector: Option<String>,
    },
    /// Scan a .Loxone config file for PII, secrets, and credentials
    Scan {
        /// Path to a .Loxone XML file
        file: String,
        /// Exit with error code if PII found (for CI)
        #[arg(long)]
        strict: bool,
    },
    /// Auto-arrange blocks on a Page using ELK layout engine
    #[command(name = "layout")]
    Layout {
        file: String,
        /// Page selector (default: first Page)
        #[arg(long)]
        page: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Add a user account to a .Loxone config file
    UserAdd {
        file: String,
        name: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Remove a user account from a .Loxone config file
    UserRemove {
        file: String,
        name: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Wire a physical device/sensor to a logical control
    #[command(visible_alias = "bind")]
    DeviceBind {
        file: String,
        /// Control selector (e.g. "Kitchen Light")
        control: String,
        /// Control output connector (e.g. "AQ1")
        output_conn: String,
        /// Device selector (e.g. "Smartaktor RGBW")
        #[arg(long)]
        device: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// List automation/autopilot rules from a .Loxone config file
    AutopilotList { file: String },
    /// Add an autopilot rule to a .Loxone config file
    AutopilotAdd {
        file: String,
        /// Rule name
        name: String,
        /// Room name
        #[arg(long)]
        room: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Add a calendar/schedule element to a .Loxone config file
    CalendarAdd {
        file: String,
        name: String,
        #[arg(long)]
        room: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Add a schedule entry to a DayTimer block (e.g. 20:00-24:00)
    #[command(name = "timer-schedule")]
    TimerSchedule {
        /// Path to a .Loxone XML file
        file: String,
        /// Block selector (title or "uuid:...")
        selector: String,
        /// Time range as "HH:MM-HH:MM" (e.g. "20:00-24:00")
        range: String,
        /// Output value during active period
        #[arg(long, default_value = "1")]
        value: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// List operating modes from a .Loxone config file
    ModeList { file: String },
    /// Create a VirtualIn element in a config file (returns connector UUID)
    #[command(visible_alias = "add-vi")]
    AddVirtualIn {
        /// Path to a .Loxone XML file
        file: String,
        /// Title for the VirtualIn
        title: String,
        /// Use analog mode (default: digital)
        #[arg(long)]
        analog: bool,
        /// Parent element selector (default: VirtualInCaption)
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Wire a connector: add <In Input="source-uuid"/> to target connector
    #[command(visible_alias = "wire")]
    WireConnector {
        /// Path to a .Loxone XML file
        file: String,
        /// Target: "BlockTitle.ConnectorKey" (e.g. "CatA_And.I1")
        target: String,
        /// Source connector UUID to wire from
        source_uuid: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Set a parameter on a block (sets Def= on the connector)
    #[command(name = "set-param", visible_alias = "set")]
    SetParam {
        /// Path to a .Loxone XML file
        file: String,
        /// Block selector (title or "uuid:...")
        selector: String,
        /// Parameter name (connector K value, e.g. "FadingTime")
        param: String,
        /// Value to set
        value: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Show all parameters of a block with current and default values
    #[command(name = "get-params", visible_alias = "get")]
    GetParams {
        /// Path to a .Loxone XML file
        file: String,
        /// Block selector (title or "uuid:...")
        selector: String,
    },
    /// Read the program text from a SequenceController block
    #[command(name = "get-program")]
    GetProgram {
        /// Path to a .Loxone XML file
        file: String,
        /// Block selector (title or "uuid:...")
        selector: String,
    },
    /// Write program text to a SequenceController block (validates syntax first)
    #[command(name = "set-program")]
    SetProgram {
        /// Path to a .Loxone XML file
        file: String,
        /// Block selector (title or "uuid:...")
        selector: String,
        /// Program text (inline). Mutually exclusive with --file.
        program: Option<String>,
        /// Read program text from a file instead of inline argument
        #[arg(long = "file", name = "program_file")]
        program_file: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Apply a room template (standard presets for common room types)
    #[command(name = "template")]
    Template {
        /// Path to a .Loxone XML file
        file: String,
        /// Template name: standard, bathroom, hallway, bedroom, kitchen, outdoor
        template: String,
        /// Target room name (must exist)
        #[arg(long)]
        room: String,
        /// Save to a different file (default: overwrite)
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Describe the full configuration in human-readable form
    #[command(name = "describe")]
    Describe {
        file: String,
        /// Optional: describe a specific room only
        #[arg(long)]
        room: Option<String>,
    },
    /// Emit the full resolved wire graph as source/target connector edges
    #[command(name = "wires")]
    Wires {
        /// Path to a .Loxone XML file
        #[arg(value_name = "FILE")]
        file: Option<String>,
        /// Path to a .Loxone XML file
        #[arg(short, long = "file", value_name = "FILE")]
        file_opt: Option<String>,
        /// Optional: emit wires whose target block is in this room only
        #[arg(short, long)]
        room: Option<String>,
    },
    /// Show comprehensive config statistics
    Stats {
        /// Path to a .Loxone XML file
        file: String,
    },
    /// Generate an anonymised diagnostic report (scan + stats + check)
    Report {
        /// Path to a .Loxone XML file
        file: String,
        /// Description of the issue being reported
        #[arg(long)]
        issue: Option<String>,
        /// Open a GitHub issue via `gh` CLI (if available)
        #[arg(long)]
        open_issue: bool,
    },
    /// Save a config snapshot for evaluation (before/after comparison)
    Snapshot {
        /// Path to a .Loxone XML file
        file: String,
        /// Save as the "before" baseline
        #[arg(long, conflicts_with = "after")]
        before: bool,
        /// Save as the "after" result and generate an eval case
        #[arg(long, conflicts_with = "before")]
        after: bool,
        /// Natural-language utterance that describes the intended change
        #[arg(long)]
        utterance: Option<String>,
        /// Directory to store snapshots (default: .lox/snapshots/)
        #[arg(long)]
        dir: Option<String>,
    },
    /// Low-level XML editing operations (power users)
    #[command(subcommand)]
    Xml(XmlEditCmd),
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum RoomCmd {
    /// Add a new room
    Add {
        /// Path to a .Loxone XML file
        file: String,
        /// Room name
        name: String,
        /// Save to a different file (default: overwrite)
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Rename a room
    Rename {
        /// Path to a .Loxone XML file
        file: String,
        /// Current room name
        old_name: String,
        /// New room name
        new_name: String,
        #[arg(long)]
        save_as: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum ControlCmd {
    /// Move controls to a different room
    Move {
        /// Path to a .Loxone XML file
        file: String,
        /// Target room name
        #[arg(long)]
        to_room: String,
        /// Filter by control type (e.g. WeatherData, SysVar)
        #[arg(short = 't', long = "type")]
        type_filter: Option<String>,
        /// Match controls by title (substring)
        #[arg(long)]
        title: Option<String>,
        /// Exclude control types
        #[arg(long)]
        exclude: Vec<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Rename a control
    Rename {
        file: String,
        /// Current name or selector (uuid:X, gid:X)
        selector: String,
        /// New title
        new_name: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Show detailed information about a control
    Describe {
        file: String,
        /// Element selector (title, uuid:X, gid:X, Type:X)
        selector: String,
    },
    /// Connect two element connectors
    Wire {
        file: String,
        /// Source: "ElementSelector.ConnectorName"
        source: String,
        /// Target: "ElementSelector.ConnectorName"
        target: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Disconnect a connector
    Unwire {
        file: String,
        /// "ElementSelector.ConnectorName"
        connector: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Show all connections for an element
    Wires {
        file: String,
        /// Element selector
        selector: String,
    },
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum MqttConfigCmd {
    /// Configure MQTT broker connection
    Setup {
        file: String,
        /// Broker address
        #[arg(long)]
        broker: String,
        /// Broker port
        #[arg(long, default_value = "1883")]
        port: String,
        /// Username
        #[arg(long)]
        user: Option<String>,
        /// Password (stored as plaintext t="11")
        #[arg(long)]
        password: Option<String>,
        /// Client ID
        #[arg(long)]
        client_id: Option<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// List MQTT subscriptions and publishes
    List { file: String },
    /// List all MQTT topic bindings (GenTSensor/GenTActor)
    Topics { file: String },
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum XmlEditCmd {
    /// Set a property in an element's SET block
    SetProperty {
        file: String,
        /// Element selector
        selector: String,
        /// Property name
        property: String,
        /// Property value
        value: String,
        /// Property type code (11=string, 7=number, 8=int, 1=bool, 15=encrypted)
        #[arg(long, default_value = "11")]
        r#type: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Set an attribute on an element
    SetAttr {
        file: String,
        selector: String,
        attr: String,
        value: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Move elements to a room by type
    Move {
        file: String,
        /// Control type to move (e.g. WeatherData)
        #[arg(long = "type")]
        type_filter: String,
        /// Target room name
        #[arg(long)]
        to_room: String,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Add a child element under a parent
    Add {
        file: String,
        /// Parent selector (e.g. gid:Mqtt)
        #[arg(long)]
        parent: String,
        /// Element type (e.g. GenTSensor, Switch)
        #[arg(long = "type")]
        element_type: String,
        /// Element title
        #[arg(long)]
        title: String,
        /// Optional gid
        #[arg(long)]
        gid: Option<String>,
        /// Room name
        #[arg(long)]
        room: Option<String>,
        /// Category name
        #[arg(long)]
        category: Option<String>,
        /// Properties as NAME:TYPE=VALUE (can repeat)
        #[arg(long, action = clap::ArgAction::Append)]
        property: Vec<String>,
        #[arg(long)]
        save_as: Option<String>,
    },
    /// Remove an element by UUID
    Remove {
        file: String,
        /// UUID of the element to remove
        #[arg(long)]
        uuid: String,
        #[arg(long)]
        save_as: Option<String>,
    },
}

// ── Error envelope ────────────────────────────────────────────────────────────

fn categorize_error(e: &anyhow::Error) -> &'static str {
    let msg = format!("{:#}", e);
    let lower = msg.to_lowercase();
    if lower.contains("no control matching") {
        "control_not_found"
    } else if lower.contains("ambiguous") {
        "ambiguous_control"
    } else if lower.contains("config not found") {
        "config_not_found"
    } else if lower.contains("requires --yes") {
        "confirmation_required"
    } else if let Some(http_err) = e.downcast_ref::<client::HttpStatusError>() {
        match http_err.status {
            401 => "unauthorized",
            403 => "forbidden",
            404 => "not_found",
            _ => "http_error",
        }
    } else if lower.contains("connection") || lower.contains("timeout") {
        "connection_error"
    } else {
        "error"
    }
}

/// Extract a privacy-safe command name from the CLI invocation (no arguments).
fn command_name(cmd: &Cmd) -> String {
    match cmd {
        Cmd::Setup { .. } => "setup".into(),
        Cmd::Cache { .. } => "cache".into(),
        Cmd::Token { .. } => "token".into(),
        Cmd::Telemetry { .. } => "telemetry".into(),
        Cmd::Config { .. } => "config".into(),
        Cmd::Ctx { .. } => "ctx".into(),
        Cmd::Completions { .. } => "completions".into(),
        Cmd::Schema { .. } => "schema".into(),
        Cmd::Sim { .. } => "sim".into(),
        Cmd::Blocks { .. } => "blocks".into(),
        Cmd::Docs { .. } => "docs".into(),
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let json = cli.output == OutputFormat::Json;
    if let Err(e) = run(cli) {
        if json {
            let envelope = serde_json::json!({
                "ok": false,
                "error": categorize_error(&e),
                "message": format!("{:#}", e),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| {
                    r#"{"ok":false,"error":"serialization_failed"}"#.to_string()
                })
            );
        } else {
            eprintln!("Error: {:#}", e);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    // Set context override before any Config::load() calls
    config::set_ctx_override(cli.context.clone());

    client::set_verbose(cli.verbose);
    let ctx = commands::RunContext {
        json: cli.output == OutputFormat::Json || cli.json_flag,
        quiet: cli.quiet,
        csv: cli.output == OutputFormat::Csv,
        dry_run: cli.dry_run,
        no_header: cli.no_header,
        non_interactive: cli.non_interactive,
        trace_id: cli.trace_id.clone(),
    };

    if let Some(tid) = &ctx.trace_id
        && cli.verbose > 0
    {
        eprintln!("trace-id: {}", tid);
    }

    // Respect NO_COLOR env var (clig.dev standard) and --no-color flag
    if cli.no_color || std::env::var("NO_COLOR").is_ok() {
        // SAFETY: Called during single-threaded startup before any concurrent work.
        unsafe { std::env::set_var("NO_COLOR", "1") };
    }

    // Show first-run telemetry notice (once only)
    if !ctx.quiet {
        telemetry::maybe_show_first_run_notice();
    }

    // Track the command name (fire-and-forget, no-op when disabled)
    let telem =
        telemetry::Telemetry::from_config(config::Config::load().ok().and_then(|c| c.telemetry));
    telem.track_command(&command_name(&cli.cmd));

    match cli.cmd {
        // ── Configuration commands ────────────────────────────────────
        Cmd::Setup { action } => commands::config_cmd::cmd_setup(&ctx, action),
        Cmd::Cache { action } => commands::config_cmd::cmd_cache(&ctx, action),
        Cmd::Token { action } => commands::config_cmd::cmd_token(&ctx, action),
        Cmd::Telemetry { action } => match action {
            TelemetryCmd::Enable => telemetry::cmd_telemetry_enable(),
            TelemetryCmd::Disable => telemetry::cmd_telemetry_disable(),
            TelemetryCmd::Status => telemetry::cmd_telemetry_status(),
        },
        Cmd::Config { action } => commands::config_cmd::cmd_config(&ctx, action),
        Cmd::Ctx { action } => commands::ctx::cmd_ctx(&ctx, action),
        Cmd::Completions { shell, install } => {
            commands::config_cmd::cmd_completions(&ctx, shell, install)
        }
        Cmd::Schema { command } => commands::config_cmd::cmd_schema(&ctx, command),
        Cmd::Sim { action } => commands::sim_cmd::cmd_sim(&ctx, action),
        Cmd::Blocks { action } => commands::blocks_cmd::cmd_blocks(&ctx, action),
        Cmd::Docs {
            query,
            datasheet,
            schema,
            list,
        } => cmd_docs(&ctx, query, datasheet, schema, list),
    }
}

// ── Schema introspection ─────────────────────────────────────────────────────

pub(crate) fn build_schema(filter: Option<&str>) -> Result<Value> {
    let cmd = Cli::command();

    if let Some(name) = filter {
        let sub = cmd.get_subcommands().find(|s| {
            s.get_name().eq_ignore_ascii_case(name)
                || s.get_all_aliases().any(|a| a.eq_ignore_ascii_case(name))
        });
        let Some(sub) = sub else {
            bail!(
                "Unknown command '{}'. Run `lox schema` for all commands.",
                name
            );
        };
        return Ok(describe_command(sub, true));
    }

    let commands: Vec<Value> = cmd
        .get_subcommands()
        .filter(|s| !s.is_hide_set())
        .map(|s| describe_command(s, false))
        .collect();

    Ok(serde_json::json!({
        "name": "lox",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Loxone Config-as-Code CLI",
        "global_flags": [
            {"name": "--output", "short": "-o", "type": "enum", "values": ["table", "json", "csv"], "default": "table"},
            {"name": "--quiet", "short": "-q", "type": "bool", "description": "Suppress non-essential output"},
            {"name": "--dry-run", "type": "bool", "description": "Validate and resolve inputs without executing commands"},
            {"name": "--non-interactive", "type": "bool", "description": "Fail instead of prompting for confirmation"},
            {"name": "--trace-id", "type": "string", "description": "Trace ID for correlating agent actions"},
            {"name": "--verbose", "short": "-v", "type": "count", "description": "Verbose output (-v requests, -vv bodies)"},
        ],
        "commands": commands,
    }))
}

pub(crate) fn describe_command(cmd: &clap::Command, _detailed: bool) -> Value {
    let mut obj = serde_json::json!({
        "name": cmd.get_name(),
        "description": cmd.get_about().map(|s| s.to_string()).unwrap_or_default(),
    });

    let args: Vec<Value> = cmd
        .get_arguments()
        .filter(|a| a.get_id() != "help" && a.get_id() != "version")
        .map(|a| {
            let mut arg = serde_json::json!({
                "name": a.get_id().to_string(),
            });
            if let Some(help) = a.get_help() {
                arg["description"] = Value::String(help.to_string());
            }
            if let Some(short) = a.get_short() {
                arg["short"] = Value::String(format!("-{}", short));
            }
            if a.get_long().is_some() {
                arg["long"] = Value::String(format!("--{}", a.get_id()));
            }
            if a.get_num_args().is_some_and(|r| r.max_values() == 0) {
                arg["type"] = Value::String("bool".to_string());
            }
            if let Some(vals) = a.get_default_values().first() {
                arg["default"] = Value::String(vals.to_string_lossy().to_string());
            }
            arg
        })
        .collect();

    if !args.is_empty() {
        obj["args"] = Value::Array(args);
    }

    let subs: Vec<Value> = cmd
        .get_subcommands()
        .filter(|s| !s.is_hide_set())
        .map(|s| describe_command(s, false))
        .collect();

    if !subs.is_empty() {
        obj["subcommands"] = Value::Array(subs);
    }

    obj
}

// ── Documentation command ────────────────────────────────────────────────────

fn cmd_docs(
    ctx: &commands::RunContext,
    query: Option<String>,
    datasheet: bool,
    schema: bool,
    list: bool,
) -> Result<()> {
    let docs_dir = find_docs_dir()?;

    if schema {
        return cmd_docs_schema(ctx, &docs_dir, query, list);
    }

    if datasheet {
        let index_path = docs_dir.join("docs").join("datasheets").join("index.json");
        let index: std::collections::HashMap<String, serde_json::Value> = if index_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&index_path)?)?
        } else {
            bail!("Datasheet index not found. Run: python3 scripts/fetch-datasheets.py");
        };

        if list || query.is_none() {
            let mut entries: Vec<_> = index
                .iter()
                .filter_map(|(k, v)| {
                    let name = v.get("name")?.as_str()?;
                    Some((k.as_str(), name))
                })
                .collect();
            entries.sort_by_key(|(_, n)| n.to_lowercase());

            if ctx.json {
                let arr: Vec<_> = entries
                    .iter()
                    .map(|(k, n)| serde_json::json!({"filename": k, "name": n}))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr)?);
            } else {
                for (fname, name) in &entries {
                    println!("  {:<50} {}", name, fname);
                }
                println!("\n{} datasheets", entries.len());
            }
            return Ok(());
        }

        let q = query.unwrap();
        let q_lower = q.to_lowercase();
        let matches: Vec<_> = index
            .iter()
            .filter(|(k, v)| {
                let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("");
                k.to_lowercase().contains(&q_lower) || name.to_lowercase().contains(&q_lower)
            })
            .collect();

        if matches.is_empty() {
            bail!(
                "No datasheet matching '{}'. Run: lox docs --datasheet --list",
                q
            );
        }

        for (_, v) in &matches {
            if let Some(path) = v.get("extracted_path").and_then(|p| p.as_str()) {
                let full = docs_dir.join("docs").join(path);
                if full.exists() {
                    let content = std::fs::read_to_string(&full)?;
                    println!("{}", content);
                    return Ok(());
                }
            }
        }
        bail!("Datasheet found but extracted content not available.");
    }

    // KB articles
    let index_path = docs_dir.join("docs").join("kb").join("index.json");
    let index: std::collections::HashMap<String, serde_json::Value> = if index_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&index_path)?)?
    } else {
        bail!("KB index not found. Run: python3 scripts/scrape-docs.py");
    };

    if list {
        let mut entries: Vec<_> = index
            .iter()
            .filter_map(|(k, v)| {
                let title = v.get("title")?.as_str()?;
                Some((k.as_str(), title))
            })
            .collect();
        entries.sort_by_key(|(_, t)| t.to_lowercase());

        if ctx.json {
            let arr: Vec<_> = entries
                .iter()
                .map(|(k, t)| serde_json::json!({"slug": k, "title": t}))
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr)?);
        } else {
            for (slug, title) in &entries {
                println!("  {:<50} {}", title, slug);
            }
            println!("\n{} articles", entries.len());
        }
        return Ok(());
    }

    let q = query.unwrap_or_default();
    if q.is_empty() {
        println!("Usage: lox docs <search-term>");
        println!("       lox docs --list              # list all articles");
        println!("       lox docs --datasheet <term>  # search datasheets");
        println!("\nExamples:");
        println!("  lox docs lighting-controller");
        println!("  lox docs mqtt");
        println!("  lox docs switch");
        println!("  lox docs --datasheet nano");
        return Ok(());
    }

    let q_lower = q.to_lowercase();

    if let Some(entry) = index.get(&q) {
        let path = entry.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let full = docs_dir.join("docs").join(path);
        if full.exists() {
            println!("{}", std::fs::read_to_string(&full)?);
            return Ok(());
        }
    }

    let matches: Vec<_> = index
        .iter()
        .filter(|(k, v)| {
            let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("");
            k.to_lowercase().contains(&q_lower) || title.to_lowercase().contains(&q_lower)
        })
        .collect();

    match matches.len() {
        0 => {
            let slugs: Vec<String> = index.keys().cloned().collect();
            let suggestion = errors::suggest(&q, &slugs);
            let mut msg = format!("No article matching '{}'.", q);
            if let Some(s) = suggestion {
                msg.push_str(&format!("\n  Did you mean '{}'?", s));
            }
            msg.push_str("\n  Run: lox docs --list");
            bail!(msg);
        }
        1 => {
            let (_, entry) = matches[0];
            let path = entry.get("path").and_then(|p| p.as_str()).unwrap_or("");
            let full = docs_dir.join("docs").join(path);
            if full.exists() {
                println!("{}", std::fs::read_to_string(&full)?);
            } else {
                bail!("Article file not found: {}", path);
            }
        }
        n if n <= 20 => {
            println!("Found {} articles matching '{}':\n", n, q);
            for (slug, v) in &matches {
                let title = v.get("title").and_then(|t| t.as_str()).unwrap_or(slug);
                println!("  lox docs {:<40} # {}", slug, title);
            }
        }
        n => {
            println!("{} articles match '{}'. Showing first 20:\n", n, q);
            for (slug, v) in matches.iter().take(20) {
                let title = v.get("title").and_then(|t| t.as_str()).unwrap_or(slug);
                println!("  lox docs {:<40} # {}", slug, title);
            }
            println!("\n  Narrow your search or run: lox docs --list");
        }
    }

    Ok(())
}

fn cmd_docs_schema(
    ctx: &commands::RunContext,
    docs_dir: &std::path::Path,
    query: Option<String>,
    list: bool,
) -> Result<()> {
    let schema_dir = docs_dir.join("docs").join("schemas");
    let index_path = schema_dir.join("index.json");

    if !index_path.exists() {
        bail!("Schema index not found. Run: python3 scripts/build-schemas.py");
    }

    let index: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(&index_path)?)?;

    if list || query.is_none() {
        let mut entries: Vec<_> = index
            .iter()
            .filter_map(|(k, v)| {
                let title = v.get("title")?.as_str()?;
                let inputs = v.get("inputs")?.as_u64()?;
                let outputs = v.get("outputs")?.as_u64()?;
                Some((k.as_str(), title, inputs, outputs))
            })
            .collect();
        entries.sort_by_key(|(_, t, _, _)| t.to_lowercase());

        if ctx.json {
            println!("{}", serde_json::to_string_pretty(&index)?);
        } else {
            println!(
                "  {:<40} {:>4} {:>4} {:>4} {:>4}",
                "Type", "In", "Out", "Par", "Prop"
            );
            println!("  {}", "─".repeat(64));
            for (slug, title, inp, out) in &entries {
                let params = index[*slug]
                    .get("parameters")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0);
                let props = index[*slug]
                    .get("properties")
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0);
                println!(
                    "  {:<40} {:>4} {:>4} {:>4} {:>4}",
                    title, inp, out, params, props
                );
            }
            println!("\n{} element schemas", entries.len());
        }
        return Ok(());
    }

    let q = query.unwrap();
    let q_lower = q.to_lowercase();

    if index.contains_key(&q) {
        let schema_path = schema_dir.join(format!("{}.json", q));
        if schema_path.exists() {
            return print_schema(&schema_path, ctx.json);
        }
    }

    let matches: Vec<_> = index
        .iter()
        .filter(|(k, v)| {
            let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("");
            let xml_type = v.get("xml_type").and_then(|t| t.as_str()).unwrap_or("");
            k.to_lowercase().contains(&q_lower)
                || title.to_lowercase().contains(&q_lower)
                || xml_type.to_lowercase().contains(&q_lower)
        })
        .collect();

    match matches.len() {
        0 => {
            let slugs: Vec<String> = index.keys().cloned().collect();
            let suggestion = errors::suggest(&q, &slugs);
            let mut msg = format!("No schema matching '{}'.", q);
            if let Some(s) = suggestion {
                msg.push_str(&format!("\n  Did you mean '{}'?", s));
            }
            msg.push_str("\n  Run: lox docs --schema --list");
            bail!(msg);
        }
        1 => {
            let (slug, _) = matches[0];
            let schema_path = schema_dir.join(format!("{}.json", slug));
            return print_schema(&schema_path, ctx.json);
        }
        _ => {
            println!("Found {} schemas matching '{}':\n", matches.len(), q);
            for (slug, v) in &matches {
                let title = v.get("title").and_then(|t| t.as_str()).unwrap_or(slug);
                println!("  lox docs --schema {:<35} # {}", slug, title);
            }
        }
    }

    Ok(())
}

fn print_schema(path: &std::path::Path, json_output: bool) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let schema: serde_json::Value = serde_json::from_str(&content)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(());
    }

    let title = schema.get("title").and_then(|t| t.as_str()).unwrap_or("?");
    let xml_type = schema
        .get("xml_type")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let desc = schema
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let url = schema
        .get("source_url")
        .and_then(|u| u.as_str())
        .unwrap_or("");

    println!("# {}", title);
    if !xml_type.is_empty() {
        println!("XML Type: {}", xml_type);
    }
    println!();
    if !desc.is_empty() {
        println!("{}\n", desc);
    }

    if let Some(inputs) = schema.get("inputs").and_then(|i| i.as_array())
        && !inputs.is_empty()
    {
        println!("## Inputs ({})", inputs.len());
        println!("  {:<12} {:<35} {:<8} Range", "Name", "Summary", "Unit");
        println!("  {}", "─".repeat(70));
        for inp in inputs {
            let name = inp
                .get("abbreviation")
                .or_else(|| inp.get("summary"))
                .and_then(|n| n.as_str())
                .unwrap_or("?");
            let summary = inp.get("summary").and_then(|s| s.as_str()).unwrap_or("");
            let unit = inp.get("unit").and_then(|u| u.as_str()).unwrap_or("-");
            let range = inp
                .get("value_range")
                .and_then(|r| r.as_str())
                .unwrap_or("");
            println!("  {:<12} {:<35} {:<8} {}", name, summary, unit, range);
        }
        println!();
    }

    if let Some(outputs) = schema.get("outputs").and_then(|o| o.as_array())
        && !outputs.is_empty()
    {
        println!("## Outputs ({})", outputs.len());
        println!("  {:<12} {:<35} {:<8} Range", "Name", "Summary", "Unit");
        println!("  {}", "─".repeat(70));
        for out in outputs {
            let name = out
                .get("abbreviation")
                .or_else(|| out.get("summary"))
                .and_then(|n| n.as_str())
                .unwrap_or("?");
            let summary = out.get("summary").and_then(|s| s.as_str()).unwrap_or("");
            let unit = out.get("unit").and_then(|u| u.as_str()).unwrap_or("-");
            let range = out
                .get("value_range")
                .and_then(|r| r.as_str())
                .unwrap_or("");
            println!("  {:<12} {:<35} {:<8} {}", name, summary, unit, range);
        }
        println!();
    }

    if let Some(params) = schema.get("parameters").and_then(|p| p.as_array())
        && !params.is_empty()
    {
        println!("## Parameters ({})", params.len());
        println!("  {:<12} {:<35} {:<8} Range", "Name", "Summary", "Default");
        println!("  {}", "─".repeat(70));
        for par in params {
            let name = par
                .get("abbreviation")
                .or_else(|| par.get("summary"))
                .and_then(|n| n.as_str())
                .unwrap_or("?");
            let summary = par.get("summary").and_then(|s| s.as_str()).unwrap_or("");
            let default = par
                .get("default_value")
                .and_then(|d| d.as_str())
                .unwrap_or("-");
            let range = par
                .get("value_range")
                .and_then(|r| r.as_str())
                .unwrap_or("");
            println!("  {:<12} {:<35} {:<8} {}", name, summary, default, range);
        }
        println!();
    }

    if let Some(props) = schema.get("properties").and_then(|p| p.as_array())
        && !props.is_empty()
    {
        println!("## Properties ({})", props.len());
        for prop in props {
            let summary = prop.get("summary").and_then(|s| s.as_str()).unwrap_or("?");
            let default = prop
                .get("default_value")
                .and_then(|d| d.as_str())
                .unwrap_or("-");
            println!("  {:<40} [{}]", summary, default);
        }
        println!();
    }

    if !url.is_empty() {
        println!("Source: {}", url);
    }

    Ok(())
}

/// Find the base directory containing `docs/` — try cwd, binary dir, then ~/.lox/
fn find_docs_dir() -> Result<std::path::PathBuf> {
    let cwd = std::path::PathBuf::from(".");
    if cwd.join("docs").join("kb").exists() {
        return Ok(cwd);
    }

    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .filter(|d| d.join("docs").join("kb").exists())
    {
        return Ok(dir);
    }

    if let Some(home) = dirs::home_dir() {
        let lox_dir = home.join(".lox");
        if lox_dir.join("docs").join("kb").exists() {
            return Ok(lox_dir);
        }
    }

    bail!(
        "Documentation not found. Install docs to ~/.lox/docs/ or run from the repo root.\n\
         To install: cp -r docs ~/.lox/"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::is_uuid;

    fn run_with_large_stack<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn test_json_val_str_string() {
        let v = serde_json::json!("200");
        assert_eq!(json_val_str(&v), Some("200".to_string()));
    }

    #[test]
    fn test_json_val_str_integer() {
        let v = serde_json::json!(200);
        assert_eq!(json_val_str(&v), Some("200".to_string()));
    }

    #[test]
    fn test_json_val_str_float() {
        let v = serde_json::json!(21.5);
        assert_eq!(json_val_str(&v), Some("21.5".to_string()));
    }

    #[test]
    fn test_json_val_str_null() {
        let v = serde_json::json!(null);
        assert_eq!(json_val_str(&v), None);
    }

    #[test]
    fn test_is_uuid() {
        assert!(is_uuid("1fbc668c-005c-7471-ffffed57184a04d2"));
        assert!(!is_uuid("Licht Wohnzimmer"));
        assert!(!is_uuid("short-str"));
    }

    #[test]
    fn test_cli_debug_assert() {
        use clap::CommandFactory;
        run_with_large_stack(|| {
            Cli::command().debug_assert();
        });
    }

    #[test]
    fn test_completions_bash_generates_output() {
        run_with_large_stack(|| {
            let mut cmd = Cli::command();
            let mut buf = Vec::new();
            generate(Shell::Bash, &mut cmd, "lox", &mut buf);
            let output = String::from_utf8(buf).unwrap();
            assert!(
                output.contains("_lox"),
                "bash completions should define _lox function"
            );
        });
    }

    #[test]
    fn test_detect_shell_from_env() {
        let result = detect_shell();
        let _ = result;
    }

    #[test]
    fn test_install_completions_creates_file() {
        run_with_large_stack(|| {
            let tmp = std::env::temp_dir().join("lox_test_completions");
            let _ = fs::remove_dir_all(&tmp);
            fs::create_dir_all(&tmp).unwrap();

            unsafe { std::env::set_var("LOX_HOME", tmp.to_str().unwrap()) };

            let mut cmd = Cli::command();

            if cfg!(windows) {
                let result = install_completions(Shell::PowerShell, &mut cmd);
                assert!(result.is_ok());
            } else {
                let result = install_completions(Shell::Bash, &mut cmd);
                assert!(result.is_ok());
                let bash_file = tmp.join(".local/share/bash-completion/completions/lox");
                assert!(bash_file.exists());
                let content = fs::read_to_string(&bash_file).unwrap();
                assert!(content.contains("_lox"));
            }

            unsafe { std::env::remove_var("LOX_HOME") };
            let _ = fs::remove_dir_all(&tmp);
        });
    }
}
