//! Git-based versioning of Loxone Miniserver configurations.
//!
//! Provides a workflow for tracking config changes over time:
//! - `init` — create a git repo to store config snapshots
//! - `pull` — download the current config, diff against the last version, commit if changed
//! - `log`  — view commit history across miniservers
//! - `restore` — check out an earlier config and optionally push it back

use crate::config::Config;
use crate::loxcc::decompress_loxcc;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Metadata parsed from the decompressed XML
// ---------------------------------------------------------------------------

/// Summary statistics extracted from a Loxone config XML.
#[derive(Debug, Clone)]
struct ConfigMeta {
    serial: String,
    firmware_version: String,
    rooms: usize,
    blocks: usize,
    wires: usize,
}

impl ConfigMeta {
    /// Quick-parse a decompressed Loxone XML to extract counts and version info.
    ///
    /// We avoid pulling in a full XML parser here — the structure file is
    /// well-formed and the elements we need (`<C Type="Room" …>`,
    /// `<C …>` blocks, `<Wire …>`) are easy to count with line scanning.
    fn from_xml(xml: &str, serial: &str) -> Self {
        let mut rooms: usize = 0;
        let mut blocks: usize = 0;
        let mut wires: usize = 0;
        let mut firmware_version = String::new();

        for line in xml.lines() {
            let trimmed = line.trim();

            // Count rooms: <C Type="Room" …>
            if trimmed.starts_with("<C ") && trimmed.contains("Type=\"Room\"") {
                rooms += 1;
            }
            // Count all control blocks: any <C …> that is NOT a Room
            // (rooms were already counted above)
            else if trimmed.starts_with("<C ") && trimmed.contains("Type=\"") {
                blocks += 1;
            }

            // Count wires
            if trimmed.starts_with("<Wire ") || trimmed.starts_with("<Wire>") {
                wires += 1;
            }

            // Firmware version from the root element, e.g. Version="14.5.12.18"
            if (trimmed.starts_with("<LoxoneConfig")
                || trimmed.starts_with("<LoxLIVE")
                || trimmed.starts_with("<MiniserverConfig"))
                && let Some(v) = extract_attr(trimmed, "Version")
            {
                firmware_version = v;
            }
        }

        ConfigMeta {
            serial: serial.to_string(),
            firmware_version,
            rooms,
            blocks,
            wires,
        }
    }

    /// Render as a YAML string for `metadata.yaml`.
    fn to_yaml(&self) -> String {
        let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z");
        format!(
            "serial: \"{}\"\nfirmware_version: \"{}\"\ntimestamp: \"{}\"\nrooms: {}\nblocks: {}\nwires: {}\n",
            self.serial, self.firmware_version, ts, self.rooms, self.blocks, self.wires,
        )
    }

    /// One-line summary for commit messages.
    fn summary(&self) -> String {
        format!(
            "config: {} — {}r {}b {}w",
            self.serial, self.rooms, self.blocks, self.wires,
        )
    }
}

/// Extract the value of an XML attribute from a tag line.
/// e.g. `extract_attr(r#"<Foo Bar="baz">"#, "Bar")` → `Some("baz")`.
fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Run a git command inside `repo`, returning stdout on success.
fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("--no-pager")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .context("failed to run git")?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Run a git command, ignoring the exit code (used for diff which returns 1
/// when there are differences).
fn git_status_code(repo: &Path, args: &[&str]) -> Result<(i32, String)> {
    let out = Command::new("git")
        .arg("--no-pager")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .context("failed to run git")?;

    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    Ok((code, stdout))
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

/// Build a blocking HTTP client that accepts self-signed certs (Miniservers
/// use self-signed TLS).
fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("lox-cli")
        .build()
        .context("failed to build HTTP client")
}

/// Download the raw LoxCC config archive from the Miniserver.
fn download_loxcc(cfg: &Config) -> Result<Vec<u8>> {
    let url = format!(
        "{}/dev/fsget/prog/{}.LoxCC",
        cfg.host.trim_end_matches('/'),
        cfg.serial,
    );
    let client = http_client()?;
    let resp = client
        .get(&url)
        .basic_auth(&cfg.user, Some(&cfg.pass))
        .send()
        .with_context(|| format!("HTTP request to {url} failed"))?;

    if !resp.status().is_success() {
        bail!("download failed: HTTP {} from {}", resp.status(), url,);
    }

    resp.bytes()
        .map(|b| b.to_vec())
        .context("failed to read response body")
}

/// Upload a compressed LoxCC config to the Miniserver.
fn upload_config(cfg: &Config, data: &[u8]) -> Result<()> {
    let url = format!(
        "{}/dev/fsput/prog/{}.LoxCC",
        cfg.host.trim_end_matches('/'),
        cfg.serial,
    );
    let client = http_client()?;
    let resp = client
        .post(&url)
        .basic_auth(&cfg.user, Some(&cfg.pass))
        .body(data.to_vec())
        .send()
        .with_context(|| format!("upload to {url} failed"))?;

    if !resp.status().is_success() {
        bail!("upload failed: HTTP {} from {}", resp.status(), url);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Initialize a git repository for config versioning.
///
/// Creates `<path>/<serial>/config.Loxone` and `metadata.yaml`, along with a
/// `.gitignore` that excludes raw backup files.  Returns the path to the
/// per-miniserver directory.
pub fn init(path: &Path, cfg: &Config) -> Result<PathBuf> {
    let repo = path.to_path_buf();
    let ms_dir = repo.join(&cfg.serial);

    // Create directory tree
    fs::create_dir_all(&ms_dir).with_context(|| format!("cannot create {}", ms_dir.display()))?;

    // Initialize git repo (idempotent — `git init` is safe to re-run)
    let out = Command::new("git")
        .arg("init")
        .arg(&repo)
        .output()
        .context("failed to run git init")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("git init failed: {}", stderr.trim());
    }

    // .gitignore — exclude raw backup files
    let gitignore_path = repo.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(
            &gitignore_path,
            "# Exclude raw backup archives\n*.LoxCC\n*.zip\n",
        )
        .context("failed to write .gitignore")?;
    }

    // Placeholder config and metadata so the first commit has structure
    let config_path = ms_dir.join("config.Loxone");
    if !config_path.exists() {
        fs::write(&config_path, "").context("failed to write placeholder config.Loxone")?;
    }

    let meta_path = ms_dir.join("metadata.yaml");
    if !meta_path.exists() {
        let meta = ConfigMeta {
            serial: cfg.serial.clone(),
            firmware_version: String::new(),
            rooms: 0,
            blocks: 0,
            wires: 0,
        };
        fs::write(&meta_path, meta.to_yaml()).context("failed to write metadata.yaml")?;
    }

    // Stage and create initial commit
    git(&repo, &["add", "-A"])?;
    // Only commit if there is something to commit (might already be init'd)
    let (code, _) = git_status_code(&repo, &["diff", "--cached", "--quiet"])?;
    if code != 0 {
        git(
            &repo,
            &["commit", "-m", "init: config versioning repository"],
        )?;
    }

    Ok(ms_dir)
}

/// Pull the current config from the Miniserver, decompress it, and commit
/// if it differs from the last saved version.
///
/// Returns `true` when a new version was committed.
pub fn pull(repo: &Path, cfg: &Config, quiet: bool) -> Result<bool> {
    let ms_dir = repo.join(&cfg.serial);
    if !ms_dir.exists() {
        bail!(
            "no tracked directory for serial {} — run `init` first",
            cfg.serial,
        );
    }

    // 1. Download and decompress
    if !quiet {
        eprintln!("Downloading config from {} …", cfg.host);
    }
    let raw = download_loxcc(cfg)?;
    let xml_bytes = decompress_loxcc(&raw).context("failed to decompress LoxCC")?;
    let xml = String::from_utf8(xml_bytes).context("config XML is not valid UTF-8")?;

    // 2. Compare with the current version on disk
    let config_path = ms_dir.join("config.Loxone");
    let existing = fs::read_to_string(&config_path).unwrap_or_default();
    if xml == existing {
        if !quiet {
            eprintln!("No changes detected.");
        }
        return Ok(false);
    }

    // 3. Parse metadata from the new XML
    let meta = ConfigMeta::from_xml(&xml, &cfg.serial);

    // 4. Write files
    fs::write(&config_path, &xml).context("failed to write config.Loxone")?;
    fs::write(ms_dir.join("metadata.yaml"), meta.to_yaml())
        .context("failed to write metadata.yaml")?;

    // 5. Stage and commit
    git(repo, &["add", "-A"])?;

    let message = meta.summary();
    git(repo, &["commit", "-m", &message])?;

    if !quiet {
        eprintln!("Committed: {message}");
    }

    Ok(true)
}

/// Show the git log of config versions.
///
/// If `ms_filter` is provided, only commits touching `<serial>/` are shown.
/// Returns the formatted log output.
pub fn log(repo: &Path, ms_filter: Option<&str>, count: usize) -> Result<String> {
    let count_str = format!("-{count}");
    let format_str = "%C(yellow)%h%Creset %s %C(dim)(%cr)%Creset";
    let pretty_arg = format!("--pretty=format:{format_str}");

    let mut args = vec!["log", "--oneline", "--no-decorate", &count_str, &pretty_arg];

    // Scope to a specific miniserver's subdirectory
    let path_filter;
    if let Some(serial) = ms_filter {
        path_filter = format!("{serial}/");
        args.push("--");
        args.push(&path_filter);
    }

    let output = git(repo, &args)?;
    Ok(output)
}

/// Restore a config from a specific commit.
///
/// Checks out `<serial>/config.Loxone` at the given commit, and — when
/// `force` is true — uploads it back to the Miniserver.
pub fn restore(repo: &Path, cfg: &Config, commit: &str, force: bool) -> Result<()> {
    let ms_dir = repo.join(&cfg.serial);
    let config_rel = format!("{}/config.Loxone", cfg.serial);

    // Verify the commit exists and contains our config file
    let (code, _) = git_status_code(repo, &["cat-file", "-e", commit])?;
    if code != 0 {
        bail!("commit {commit} does not exist in this repository");
    }

    // Show what we're restoring
    let show_out = git(repo, &["log", "--oneline", "-1", commit])?;
    eprintln!("Restoring: {}", show_out.trim());

    // Checkout the file at that revision
    git(repo, &["checkout", commit, "--", &config_rel])?;

    let config_path = ms_dir.join("config.Loxone");
    let xml = fs::read_to_string(&config_path).context("failed to read restored config")?;

    // Regenerate metadata from the restored XML
    let meta = ConfigMeta::from_xml(&xml, &cfg.serial);
    fs::write(ms_dir.join("metadata.yaml"), meta.to_yaml())
        .context("failed to write metadata.yaml")?;

    // Commit the restore
    git(repo, &["add", "-A"])?;
    let msg = format!(
        "restore: {} from {}",
        cfg.serial,
        &commit[..7.min(commit.len())]
    );
    // Only commit if there are staged changes (might be same as HEAD)
    let (diff_code, _) = git_status_code(repo, &["diff", "--cached", "--quiet"])?;
    if diff_code != 0 {
        git(repo, &["commit", "-m", &msg])?;
    }

    // Upload to Miniserver if --force
    if force {
        eprintln!("Uploading restored config to {} …", cfg.host);
        let xml_bytes = fs::read(&config_path)?;
        let compressed = crate::loxcc::compress_loxcc(&xml_bytes);
        upload_config(cfg, &compressed)?;
        eprintln!("Upload complete. Reboot the Miniserver to apply.");
    } else {
        eprintln!("Restored locally. Use --force to upload to the Miniserver.");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<LoxoneConfig Version="14.5.12.18" xmlns="http://www.loxone.com">
  <C Type="Room" Title="Living Room" U="0a1b2c3d-0001-0000-ffff-aabbccddeeff"/>
  <C Type="Room" Title="Bedroom" U="0a1b2c3d-0002-0000-ffff-aabbccddeeff"/>
  <C Type="Room" Title="Kitchen" U="0a1b2c3d-0003-0000-ffff-aabbccddeeff"/>
  <C Type="LightController2" Title="Living Light" U="0a1b2c3d-1001-0000-ffff-aabbccddeeff"/>
  <C Type="JalousieUpDown2" Title="Blinds" U="0a1b2c3d-1002-0000-ffff-aabbccddeeff"/>
  <C Type="Thermostat" Title="Heating" U="0a1b2c3d-1003-0000-ffff-aabbccddeeff"/>
  <C Type="And" Title="Logic Gate" U="0a1b2c3d-1004-0000-ffff-aabbccddeeff"/>
  <Wire U="0a1b2c3d-2001-0000-ffff-aabbccddeeff"/>
  <Wire U="0a1b2c3d-2002-0000-ffff-aabbccddeeff"/>
</LoxoneConfig>
"#;

    #[test]
    fn parse_metadata_from_xml() {
        let meta = ConfigMeta::from_xml(SAMPLE_XML, "504F94AABBCC");
        assert_eq!(meta.serial, "504F94AABBCC");
        assert_eq!(meta.firmware_version, "14.5.12.18");
        assert_eq!(meta.rooms, 3);
        assert_eq!(meta.blocks, 4);
        assert_eq!(meta.wires, 2);
    }

    #[test]
    fn metadata_summary_format() {
        let meta = ConfigMeta {
            serial: "504F94AABBCC".to_string(),
            firmware_version: "14.5.12.18".to_string(),
            rooms: 3,
            blocks: 4,
            wires: 2,
        };
        assert_eq!(meta.summary(), "config: 504F94AABBCC — 3r 4b 2w");
    }

    #[test]
    fn metadata_yaml_contains_required_fields() {
        let meta = ConfigMeta {
            serial: "504F94AABBCC".to_string(),
            firmware_version: "14.5.12.18".to_string(),
            rooms: 3,
            blocks: 4,
            wires: 2,
        };
        let yaml = meta.to_yaml();
        assert!(yaml.contains("serial: \"504F94AABBCC\""));
        assert!(yaml.contains("firmware_version: \"14.5.12.18\""));
        assert!(yaml.contains("rooms: 3"));
        assert!(yaml.contains("blocks: 4"));
        assert!(yaml.contains("wires: 2"));
        assert!(yaml.contains("timestamp:"));
    }

    #[test]
    fn extract_attr_basic() {
        assert_eq!(
            extract_attr(r#"<Foo Bar="hello" Baz="world">"#, "Bar"),
            Some("hello".to_string()),
        );
        assert_eq!(
            extract_attr(r#"<Foo Bar="hello" Baz="world">"#, "Baz"),
            Some("world".to_string()),
        );
        assert_eq!(extract_attr(r#"<Foo Bar="hello">"#, "Missing"), None,);
    }

    #[test]
    fn parse_empty_xml() {
        let meta = ConfigMeta::from_xml("", "EMPTY");
        assert_eq!(meta.rooms, 0);
        assert_eq!(meta.blocks, 0);
        assert_eq!(meta.wires, 0);
        assert!(meta.firmware_version.is_empty());
    }

    #[test]
    fn parse_xml_with_nested_controls() {
        // Controls can be nested inside rooms in some config variants
        let xml = r#"<LoxoneConfig Version="15.0.1.2">
  <C Type="Room" Title="Office">
    <C Type="VirtualIn" Title="Sensor"/>
  </C>
  <Wire U="abc"/>
</LoxoneConfig>"#;
        let meta = ConfigMeta::from_xml(xml, "TEST");
        assert_eq!(meta.firmware_version, "15.0.1.2");
        assert_eq!(meta.rooms, 1);
        // VirtualIn is inside the Room element but on its own line
        assert_eq!(meta.blocks, 1);
        assert_eq!(meta.wires, 1);
    }
}
