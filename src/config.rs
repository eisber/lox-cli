use anyhow::{Context, Result, bail};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

// ── Global context override (set from --ctx flag) ────────────────────────────

static CTX_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);

/// Set the global context override from the `--ctx` CLI flag.
/// Called early in main, before any `Config::load()`.
pub fn set_ctx_override(ctx: Option<String>) {
    *CTX_OVERRIDE.write().unwrap() = ctx;
}

fn ctx_override() -> Option<String> {
    CTX_OVERRIDE.read().unwrap().clone()
}

/// Validate that a context name is safe for use as a directory name.
/// Rejects names containing path separators, `..`, or control characters.
pub fn validate_context_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Context name cannot be empty");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!(
            "Context name '{}' contains invalid characters (/, \\, or ..)",
            name
        );
    }
    if name.chars().any(|c| c.is_control()) {
        bail!("Context name '{}' contains control characters", name);
    }
    Ok(())
}

// ── Per-connection config (one Miniserver) ───────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub host: String,
    pub user: String,
    pub pass: String,
    #[serde(default)]
    pub serial: String,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    /// Enable SSL certificate verification (default: false for Miniserver self-signed certs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_ssl: Option<bool>,
    /// Path to a git repository for config version tracking (`lox config init`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_repo: Option<String>,
    /// Opt-in anonymous usage telemetry (`lox telemetry enable`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<bool>,

    // ── Runtime-only fields (not serialized) ─────────────────────────────
    /// Resolved base directory for this config's data (cache, token, scenes).
    /// For project-local: the `.lox/` directory.
    /// For global flat: `~/.lox/`.
    /// For global context: `~/.lox/contexts/<name>/`.
    #[serde(skip)]
    pub data_dir: PathBuf,

    /// Context name, if loaded from a multi-context config.
    #[serde(skip)]
    pub context_name: Option<String>,

    /// Whether this config was loaded from a project-local `.lox/` directory.
    #[serde(skip)]
    pub is_local: bool,
}

// ── Multi-context config file format ─────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GlobalConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_context: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub contexts: HashMap<String, ContextEntry>,
    /// Opt-in anonymous usage telemetry (global, not per-context)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<bool>,
}

/// A single context entry within the global config file.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ContextEntry {
    pub host: String,
    pub user: String,
    pub pass: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub serial: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub aliases: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_ssl: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_repo: Option<String>,
}

impl From<&Config> for ContextEntry {
    fn from(cfg: &Config) -> Self {
        Self {
            host: cfg.host.clone(),
            user: cfg.user.clone(),
            pass: cfg.pass.clone(),
            serial: cfg.serial.clone(),
            aliases: cfg.aliases.clone(),
            verify_ssl: cfg.verify_ssl,
            config_repo: cfg.config_repo.clone(),
        }
    }
}

impl ContextEntry {
    pub fn into_config(self, name: &str, data_dir: PathBuf) -> Config {
        Config {
            host: self.host,
            user: self.user,
            pass: self.pass,
            serial: self.serial,
            aliases: self.aliases,
            verify_ssl: self.verify_ssl,
            config_repo: self.config_repo,
            telemetry: None,
            data_dir,
            context_name: Some(name.to_string()),
            is_local: false,
        }
    }
}

// ── Config implementation ────────────────────────────────────────────────────

impl Config {
    /// Global config directory: `~/.lox/`
    pub fn dir() -> PathBuf {
        home_dir().unwrap_or_default().join(".lox")
    }

    /// Global config file path: `~/.lox/config.yaml`
    pub fn path() -> PathBuf {
        Self::dir().join("config.yaml")
    }

    /// Data directory for a named global context: `~/.lox/contexts/<name>/`
    pub fn context_data_dir(name: &str) -> PathBuf {
        Self::dir().join("contexts").join(name)
    }

    /// Cache directory for this config's structure cache.
    pub fn cache_dir(&self) -> PathBuf {
        self.data_dir.join("cache")
    }

    /// Token file path for this config.
    pub fn token_path(&self) -> PathBuf {
        self.data_dir.join("token.json")
    }

    /// Scenes directory for this config.
    pub fn scenes_dir(&self) -> PathBuf {
        self.data_dir.join("scenes")
    }

    /// Load the effective config, considering context resolution:
    /// 1. `LOX_CONFIG` env var (flat format, bypass everything)
    /// 2. Project-local `.lox/` (walk up from cwd) — flat format
    /// 3. Global `~/.lox/config.yaml` — flat or multi-context
    /// 4. `--ctx` flag overrides context selection within global config
    pub fn load() -> Result<Self> {
        // 1. LOX_CONFIG env var takes absolute priority
        if let Ok(env_path) = std::env::var("LOX_CONFIG") {
            let path = PathBuf::from(&env_path);
            let data_dir = path.parent().unwrap_or(&Self::dir()).to_path_buf();
            return Self::load_flat(&path, data_dir, false);
        }

        let ctx_flag = ctx_override();

        // 2. Project-local .lox/ (only if --ctx is not set)
        if ctx_flag.is_none()
            && let Some(local_dir) = find_local_lox_dir()
        {
            let path = local_dir.join("config.yaml");
            return Self::load_flat(&path, local_dir, true);
        }

        // 3. Global config (flat or multi-context)
        let path = Self::path();
        let content = fs::read_to_string(&path).with_context(
            || "Config not found. Run: lox setup set --host ... --user ... --pass ...",
        )?;

        // Detect format: if the YAML has a "contexts" key, it's multi-context
        let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if value.get("contexts").is_some() {
            Self::load_from_global_config(&content, ctx_flag.as_deref())
        } else {
            // Flat format (backward compatible)
            Self::load_flat(&path, Self::dir(), false)
        }
    }

    /// Load a flat (single-Miniserver) config file.
    fn load_flat(path: &Path, data_dir: PathBuf, is_local: bool) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Config not found at {}", path.display()))?;
        let mut cfg: Self = serde_yaml::from_str(&content)?;
        cfg.data_dir = data_dir;
        cfg.is_local = is_local;
        cfg.normalize_host();
        Ok(cfg)
    }

    /// Load from a multi-context global config, selecting the active context.
    fn load_from_global_config(content: &str, ctx_flag: Option<&str>) -> Result<Self> {
        let global: GlobalConfig = serde_yaml::from_str(content)?;
        let ctx_name = ctx_flag
            .map(String::from)
            .or(global.active_context)
            .context("No active context set. Run: lox ctx use <name>")?;

        let entry = global
            .contexts
            .get(&ctx_name)
            .with_context(|| format!("Context '{}' not found. Run: lox ctx list", ctx_name))?;

        let data_dir = Self::context_data_dir(&ctx_name);
        let mut cfg = entry.clone().into_config(&ctx_name, data_dir);
        cfg.telemetry = global.telemetry;
        cfg.normalize_host();
        Ok(cfg)
    }

    /// Prepend https:// if no scheme is present.
    fn normalize_host(&mut self) {
        if !self.host.is_empty()
            && !self.host.starts_with("http://")
            && !self.host.starts_with("https://")
        {
            self.host = format!("https://{}", self.host);
        }
    }

    /// Save this config as a flat file (for project-local or legacy configs).
    pub fn save(&self) -> Result<PathBuf> {
        // If this config is part of a multi-context global config, save there
        if let Some(ref ctx_name) = self.context_name
            && !self.is_local
        {
            let mut global = GlobalConfig::load_or_default();
            global
                .contexts
                .insert(ctx_name.clone(), ContextEntry::from(self));
            global.save()?;
            return Ok(Self::path());
        }

        // Otherwise save as flat config
        let path = if self.is_local {
            self.data_dir.join("config.yaml")
        } else {
            Self::path()
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, serde_yaml::to_string(self)?)?;
        #[cfg(unix)]
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        Ok(path)
    }
}

// ── GlobalConfig implementation ──────────────────────────────────────────────

impl GlobalConfig {
    /// Load the global config file, returning Default if it doesn't exist or is flat format.
    pub fn load_or_default() -> Self {
        let path = Config::path();
        let Ok(content) = fs::read_to_string(&path) else {
            return Self::default();
        };
        // Only parse as GlobalConfig if it has a contexts key
        let Ok(value): Result<serde_yaml::Value, _> = serde_yaml::from_str(&content) else {
            return Self::default();
        };
        if value.get("contexts").is_some() {
            serde_yaml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Load the global config, returning an error if the file doesn't exist.
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        let path = Config::path();
        let content = fs::read_to_string(&path).with_context(
            || "Config not found. Run: lox setup set --host ... --user ... --pass ...",
        )?;
        let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if value.get("contexts").is_some() {
            Ok(serde_yaml::from_str(&content)?)
        } else {
            // Flat format — no contexts
            Ok(Self::default())
        }
    }

    /// Save the global config to `~/.lox/config.yaml`.
    pub fn save(&self) -> Result<PathBuf> {
        let path = Config::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, serde_yaml::to_string(self)?)?;
        #[cfg(unix)]
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        Ok(path)
    }

    /// Check if the global config file uses the multi-context format.
    pub fn is_multi_context() -> bool {
        let path = Config::path();
        let Ok(content) = fs::read_to_string(&path) else {
            return false;
        };
        let Ok(value): Result<serde_yaml::Value, _> = serde_yaml::from_str(&content) else {
            return false;
        };
        value.get("contexts").is_some()
    }

    /// Read the current flat config (for migration).
    pub fn load_flat_config() -> Result<Config> {
        let path = Config::path();
        let content = fs::read_to_string(&path).with_context(
            || "Config not found. Run: lox setup set --host ... --user ... --pass ...",
        )?;
        let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if value.get("contexts").is_some() {
            bail!("Config is already in multi-context format");
        }
        let mut cfg: Config = serde_yaml::from_str(&content)?;
        cfg.data_dir = Config::dir();
        cfg.normalize_host();
        Ok(cfg)
    }
}

// ── Project-local .lox/ directory discovery ──────────────────────────────────

/// Walk up from the current directory looking for a `.lox/config.yaml` file.
/// Returns the `.lox/` directory path if found, or None.
pub fn find_local_lox_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_local_lox_dir_from(&cwd)
}

/// Walk up from the given directory looking for a `.lox/config.yaml` file.
/// Returns the `.lox/` directory path if found, or None.
pub fn find_local_lox_dir_from(start: &Path) -> Option<PathBuf> {
    let global_dir = Config::dir();
    let mut dir = start;
    loop {
        let candidate = dir.join(".lox");
        // Don't match the global ~/.lox/ directory
        if candidate != global_dir && candidate.join("config.yaml").is_file() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn dir_ends_with_dot_lox() {
        let dir = Config::dir();
        assert!(
            dir.ends_with(".lox"),
            "Config::dir() should end with .lox, got {:?}",
            dir
        );
    }

    #[test]
    fn path_ends_with_config_yaml() {
        let path = Config::path();
        assert!(
            path.ends_with("config.yaml"),
            "Config::path() should end with config.yaml, got {:?}",
            path
        );
    }

    /// Mutex to serialize tests that mutate the LOX_CONFIG env var.
    /// `std::env::set_var` is unsafe because it's not thread-safe;
    /// without this lock, parallel tests race on the shared env.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn load_from_lox_config_env_var() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("test_config.yaml");
        let yaml = "host: myhost.local\nuser: admin\npass: secret\n";
        fs::write(&cfg_path, yaml).unwrap();

        unsafe { std::env::set_var("LOX_CONFIG", cfg_path.to_str().unwrap()) };
        let cfg = Config::load().unwrap();
        unsafe { std::env::remove_var("LOX_CONFIG") };

        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.pass, "secret");
    }

    #[test]
    fn load_prepends_https_to_bare_hostname() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("cfg.yaml");
        fs::write(&cfg_path, "host: miniserver.local\nuser: u\npass: p\n").unwrap();

        unsafe { std::env::set_var("LOX_CONFIG", cfg_path.to_str().unwrap()) };
        let cfg = Config::load().unwrap();
        unsafe { std::env::remove_var("LOX_CONFIG") };

        assert_eq!(cfg.host, "https://miniserver.local");
    }

    #[test]
    fn load_preserves_explicit_http_scheme() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("cfg.yaml");
        fs::write(&cfg_path, "host: http://ms.local\nuser: u\npass: p\n").unwrap();

        unsafe { std::env::set_var("LOX_CONFIG", cfg_path.to_str().unwrap()) };
        let cfg = Config::load().unwrap();
        unsafe { std::env::remove_var("LOX_CONFIG") };

        assert_eq!(cfg.host, "http://ms.local");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        // Config::save() uses the hardcoded path, so we test roundtrip via
        // manual serialization + LOX_CONFIG-based load.
        let mut cfg = Config {
            host: "https://10.0.0.1".to_string(),
            user: "testuser".to_string(),
            pass: "testpass".to_string(),
            serial: "00:11:22:33:44:55".to_string(),
            ..Default::default()
        };
        cfg.aliases
            .insert("light".to_string(), "some-uuid".to_string());

        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let cfg_path = dir.path().join("roundtrip.yaml");
        fs::write(&cfg_path, &yaml).unwrap();

        unsafe { std::env::set_var("LOX_CONFIG", cfg_path.to_str().unwrap()) };
        let loaded = Config::load().unwrap();
        unsafe { std::env::remove_var("LOX_CONFIG") };

        assert_eq!(loaded.host, "https://10.0.0.1");
        assert_eq!(loaded.user, "testuser");
        assert_eq!(loaded.pass, "testpass");
        assert_eq!(loaded.serial, "00:11:22:33:44:55");
        assert_eq!(loaded.aliases.get("light"), Some(&"some-uuid".to_string()));
    }

    #[test]
    fn default_config_has_empty_fields() {
        let cfg = Config::default();
        assert!(cfg.host.is_empty());
        assert!(cfg.user.is_empty());
        assert!(cfg.pass.is_empty());
        assert!(cfg.serial.is_empty());
        assert!(cfg.aliases.is_empty());
        assert!(cfg.verify_ssl.is_none());
        assert!(cfg.config_repo.is_none());
    }

    #[test]
    fn context_data_dir_layout() {
        let dir = Config::context_data_dir("home");
        assert!(dir.ends_with("contexts/home"));
    }

    #[test]
    fn global_config_roundtrip() {
        let mut global = GlobalConfig::default();
        global.active_context = Some("home".to_string());
        global.contexts.insert(
            "home".to_string(),
            ContextEntry {
                host: "https://192.168.1.100".to_string(),
                user: "admin".to_string(),
                pass: "secret".to_string(),
                ..Default::default()
            },
        );
        let yaml = serde_yaml::to_string(&global).unwrap();
        let parsed: GlobalConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.active_context, Some("home".to_string()));
        assert!(parsed.contexts.contains_key("home"));
        assert_eq!(parsed.contexts["home"].host, "https://192.168.1.100");
    }

    #[test]
    fn context_entry_into_config() {
        let entry = ContextEntry {
            host: "https://10.0.0.1".to_string(),
            user: "admin".to_string(),
            pass: "pass".to_string(),
            serial: "ABC123".to_string(),
            ..Default::default()
        };
        let cfg = entry.into_config("office", PathBuf::from("/tmp/test"));
        assert_eq!(cfg.host, "https://10.0.0.1");
        assert_eq!(cfg.context_name, Some("office".to_string()));
        assert_eq!(cfg.data_dir, PathBuf::from("/tmp/test"));
        assert!(!cfg.is_local);
    }

    #[test]
    fn find_local_lox_dir_in_tempdir() {
        let dir = tempdir().unwrap();
        let lox_dir = dir.path().join(".lox");
        fs::create_dir_all(&lox_dir).unwrap();
        fs::write(lox_dir.join("config.yaml"), "host: h\nuser: u\npass: p\n").unwrap();

        // Use the path-based variant to avoid thread-unsafe set_current_dir
        let found = find_local_lox_dir_from(dir.path());
        assert_eq!(found, Some(lox_dir));
    }

    #[test]
    fn find_local_lox_dir_walks_up() {
        let dir = tempdir().unwrap();
        let lox_dir = dir.path().join(".lox");
        fs::create_dir_all(&lox_dir).unwrap();
        fs::write(lox_dir.join("config.yaml"), "host: h\nuser: u\npass: p\n").unwrap();

        // Create a nested subdirectory and search from there
        let nested = dir.path().join("sub").join("deep");
        fs::create_dir_all(&nested).unwrap();
        let found = find_local_lox_dir_from(&nested);
        assert_eq!(found, Some(lox_dir));
    }

    #[test]
    fn validate_context_name_rejects_path_traversal() {
        assert!(validate_context_name("home").is_ok());
        assert!(validate_context_name("my-server_1").is_ok());
        assert!(validate_context_name("../evil").is_err());
        assert!(validate_context_name("foo/bar").is_err());
        assert!(validate_context_name("foo\\bar").is_err());
        assert!(validate_context_name("").is_err());
    }

    #[test]
    fn load_multi_context_via_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("config.yaml");
        let yaml = r#"
active_context: home
contexts:
  home:
    host: https://192.168.1.100
    user: admin
    pass: secret
  office:
    host: https://10.0.0.50
    user: admin
    pass: office_pass
"#;
        fs::write(&cfg_path, yaml).unwrap();

        // LOX_CONFIG with multi-context file is loaded as flat (env var bypasses context logic)
        // So we test load_from_global_config directly
        let cfg = Config::load_from_global_config(yaml, None).unwrap();
        assert_eq!(cfg.host, "https://192.168.1.100");
        assert_eq!(cfg.user, "admin");
        assert_eq!(cfg.pass, "secret");
        assert_eq!(cfg.context_name, Some("home".to_string()));
    }

    #[test]
    fn load_multi_context_with_ctx_override() {
        let yaml = r#"
active_context: home
contexts:
  home:
    host: https://192.168.1.100
    user: admin
    pass: secret
  office:
    host: https://10.0.0.50
    user: admin
    pass: office_pass
"#;
        let cfg = Config::load_from_global_config(yaml, Some("office")).unwrap();
        assert_eq!(cfg.host, "https://10.0.0.50");
        assert_eq!(cfg.pass, "office_pass");
        assert_eq!(cfg.context_name, Some("office".to_string()));
    }

    #[test]
    fn load_multi_context_missing_context_errors() {
        let yaml = r#"
active_context: home
contexts:
  home:
    host: https://192.168.1.100
    user: admin
    pass: secret
"#;
        let result = Config::load_from_global_config(yaml, Some("nonexistent"));
        assert!(result.is_err());
        let err = format!("{:#}", result.unwrap_err());
        assert!(err.contains("nonexistent"), "Error: {}", err);
    }

    #[test]
    fn load_multi_context_no_active_errors() {
        let yaml = r#"
contexts:
  home:
    host: https://192.168.1.100
    user: admin
    pass: secret
"#;
        let result = Config::load_from_global_config(yaml, None);
        assert!(result.is_err());
        let err = format!("{:#}", result.unwrap_err());
        assert!(err.contains("No active context"), "Error: {}", err);
    }

    #[test]
    fn load_flat_config_via_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();
        let cfg_path = dir.path().join("config.yaml");
        fs::write(&cfg_path, "host: myhost.local\nuser: u\npass: p\n").unwrap();

        unsafe { std::env::set_var("LOX_CONFIG", cfg_path.to_str().unwrap()) };
        let cfg = Config::load().unwrap();
        unsafe { std::env::remove_var("LOX_CONFIG") };

        // Flat config loaded via LOX_CONFIG should have data_dir set to parent
        assert_eq!(cfg.data_dir, dir.path().to_path_buf());
        assert!(!cfg.is_local);
        assert!(cfg.context_name.is_none());
    }

    #[test]
    fn config_save_roundtrip_with_context() {
        let entry = ContextEntry {
            host: "https://10.0.0.1".to_string(),
            user: "admin".to_string(),
            pass: "pass".to_string(),
            serial: "SER123".to_string(),
            aliases: HashMap::from([("l".to_string(), "uuid-1".to_string())]),
            verify_ssl: Some(true),
            config_repo: None,
        };
        let cfg = entry.into_config("test", PathBuf::from("/tmp/test"));
        // Round-trip through ContextEntry
        let entry2 = ContextEntry::from(&cfg);
        assert_eq!(entry2.host, "https://10.0.0.1");
        assert_eq!(entry2.serial, "SER123");
        assert_eq!(entry2.aliases.get("l"), Some(&"uuid-1".to_string()));
        assert_eq!(entry2.verify_ssl, Some(true));
    }

    #[test]
    fn config_cache_dir_uses_data_dir() {
        let cfg = Config {
            data_dir: PathBuf::from("/my/data"),
            ..Default::default()
        };
        assert_eq!(cfg.cache_dir(), PathBuf::from("/my/data/cache"));
        assert_eq!(cfg.token_path(), PathBuf::from("/my/data/token.json"));
        assert_eq!(cfg.scenes_dir(), PathBuf::from("/my/data/scenes"));
    }
}
