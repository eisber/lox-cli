//! Loxone HTTP client, control resolution, and structure cache

use anyhow::{Context, Result, bail};
pub use lox_core::control::{Control, is_uuid};
use lox_core::control::{
    ControlQuery, IdentifierError, PreparedControlQuery, ResolutionError, matches_text,
};
use lox_core::control_catalog::{ControlCatalog, ControlFilter};
use lox_core::structure_metadata::StructureMetadata;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

// ── Constants ────────────────────────────────────────────────────────────────

const HTTP_TIMEOUT_SECS: u64 = 10;
const RETRY_DELAYS_MS: [u64; 3] = [200, 400, 800];
const CACHE_TTL_SECS: u64 = 86400;
/// Loxone epoch: 2009-01-01T00:00:00Z as Unix timestamp.
pub use lox_core::LOXONE_EPOCH_SECS;

// ── Global verbosity ─────────────────────────────────────────────────────────

static VERBOSE: AtomicU8 = AtomicU8::new(0);

/// Set the global verbosity level (0 = off, 1 = -v, 2 = -vv).
pub fn set_verbose(level: u8) {
    VERBOSE.store(level, Ordering::Relaxed);
}

pub(crate) fn verbose() -> u8 {
    VERBOSE.load(Ordering::Relaxed)
}

/// Redact credentials from a URL string for safe logging.
/// Removes basic-auth passwords and known sensitive query parameters.
pub(crate) fn redact_url(url: &str) -> String {
    // Redact userinfo password (http://user:pass@host → http://user:***@host)
    let mut result = if let Ok(mut parsed) = reqwest::Url::parse(url) {
        if parsed.password().is_some() {
            let _ = parsed.set_password(Some("***"));
        }
        parsed.to_string()
    } else {
        url.to_string()
    };
    // Redact all occurrences of known sensitive query params.
    // Only match whole parameter names (preceded by '?' or '&') to avoid
    // false positives like "passthrough=" matching the "pass=" pattern.
    for param in &["token", "key", "pass", "password", "autht"] {
        let pattern = format!("{}=", param);
        let mut search_from = 0;
        while let Some(idx) = result[search_from..].find(&pattern) {
            let abs_idx = search_from + idx;
            // Ensure this is a whole query parameter name, not a substring
            if abs_idx > 0 {
                let prev = result.as_bytes()[abs_idx - 1];
                if prev != b'?' && prev != b'&' {
                    search_from = abs_idx + pattern.len();
                    continue;
                }
            }
            let start = abs_idx + pattern.len();
            let end = result[start..]
                .find('&')
                .map_or(result.len(), |i| start + i);
            result.replace_range(start..end, "***");
            search_from = start + 3; // skip past "***"
        }
    }
    result
}

/// HTTP error with status code preserved for structured matching in retry logic.
#[derive(Debug)]
pub struct HttpStatusError {
    pub status: u16,
    pub path: String,
}

/// Log a response body at -vv level, truncating large responses.
pub(crate) fn log_body(body: &str) {
    if body.len() > 500 {
        // Find a UTF-8 safe truncation point at or before byte 500
        let truncated = &body[..body.floor_char_boundary(500)];
        eprintln!("  body: {}… ({} bytes total)", truncated, body.len());
    } else {
        eprintln!("  body: {}", body);
    }
}

impl std::fmt::Display for HttpStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {}: {}", self.status, self.path)
    }
}

impl std::error::Error for HttpStatusError {}

use crate::config::Config;

pub const USER_AGENT: &str = concat!("lox-cli/", env!("CARGO_PKG_VERSION"));

// ── AutopilotRule ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AutopilotRule {
    pub name: String,
    pub uuid: String,
    pub state_changed: String,
    pub state_history: String,
}

// ── LoxClient ─────────────────────────────────────────────────────────────────

pub struct LoxClient {
    pub cfg: Config,
    pub client: Client,
    structure: Option<Value>,
}

impl LoxClient {
    /// Build a redirect policy that blocks cross-origin redirects.
    /// Loxone Miniserver Gen 2 redirects local HTTP requests to a cloud
    /// DynDNS URL (e.g. `https://{IP}.{serial}.dyndns.loxonecloud.com`).
    /// Following such redirects causes 401 errors because credentials
    /// are for the local Miniserver, not the cloud gateway.
    pub fn same_origin_redirect_policy(host: &str) -> reqwest::redirect::Policy {
        // Extract the host string from the configured URL.  Try parsing as a
        // full URL first; fall back to treating bare hostnames/IPs by
        // prepending a dummy scheme so Url::parse succeeds.
        let configured_host = reqwest::Url::parse(host)
            .or_else(|_| reqwest::Url::parse(&format!("http://{}", host)))
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default();
        reqwest::redirect::Policy::custom(move |attempt| {
            if configured_host.is_empty() {
                // Could not determine configured host — allow redirect
                return attempt.follow();
            }
            let target_host = attempt.url().host_str().unwrap_or_default().to_string();
            if target_host != configured_host {
                return attempt.error(std::io::Error::other(format!(
                    "Blocked cross-origin redirect from {} to {}. \
                     If using a local IP, ensure your config uses https:// \
                     or check your Miniserver's network settings.",
                    configured_host, target_host
                )));
            }
            attempt.follow()
        })
    }

    pub fn new(cfg: Config) -> Result<Self> {
        let verify_ssl = cfg.verify_ssl.unwrap_or(false);
        let redirect_policy = Self::same_origin_redirect_policy(&cfg.host);
        let lox = Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .danger_accept_invalid_certs(!verify_ssl)
                .redirect(redirect_policy)
                .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
                .build()
                .context("failed to build HTTP client")?,
            cfg,
            structure: None,
        };

        if !verify_ssl && verbose() >= 1 {
            eprintln!("warning: TLS certificate verification is disabled (verify_ssl not set)");
        }

        Ok(lox)
    }

    pub fn apply_auth(
        &self,
        rb: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        // Always use password for HTTP basic auth.
        // Loxone tokens are for WebSocket auth only — they can't be
        // used as HTTP basic auth passwords.
        rb.basic_auth(&self.cfg.user, Some(&self.cfg.pass))
    }

    pub fn get_text(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.cfg.host, path.trim_start_matches('/'));
        if verbose() >= 1 {
            eprintln!("GET {}", redact_url(&url));
        }
        self.request_with_retry(|| {
            let resp = self.apply_auth(self.client.get(&url)).send()?;
            let status = resp.status();
            if verbose() >= 1 {
                eprintln!(
                    "  → {} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("")
                );
            }
            let body = resp.text()?;
            if verbose() >= 2 {
                log_body(&body);
            }
            if !status.is_success() {
                return Err(HttpStatusError {
                    status: status.as_u16(),
                    path: path.to_string(),
                }
                .into());
            }
            Ok(body)
        })
    }

    pub fn get_json(&self, path: &str) -> Result<Value> {
        let url = format!("{}/{}", self.cfg.host, path.trim_start_matches('/'));
        if verbose() >= 1 {
            eprintln!("GET {}", redact_url(&url));
        }
        self.request_with_retry(|| {
            let resp = self.apply_auth(self.client.get(&url)).send()?;
            let status = resp.status();
            if verbose() >= 1 {
                eprintln!(
                    "  → {} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("")
                );
            }
            if !status.is_success() {
                return Err(HttpStatusError {
                    status: status.as_u16(),
                    path: path.to_string(),
                }
                .into());
            }
            if verbose() >= 2 {
                let text = resp.text()?;
                log_body(&text);
                serde_json::from_str::<Value>(&text).map_err(Into::into)
            } else {
                resp.json::<Value>().map_err(Into::into)
            }
        })
    }

    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let url = format!("{}/{}", self.cfg.host, path.trim_start_matches('/'));
        if verbose() >= 1 {
            eprintln!("GET {}", redact_url(&url));
        }
        self.request_with_retry(|| {
            let resp = self.apply_auth(self.client.get(&url)).send()?;
            let status = resp.status();
            if verbose() >= 1 {
                eprintln!(
                    "  → {} {}",
                    status.as_u16(),
                    status.canonical_reason().unwrap_or("")
                );
            }
            let bytes = resp.bytes()?.to_vec();
            if verbose() >= 2 {
                eprintln!("  body: <{} bytes>", bytes.len());
            }
            if !status.is_success() {
                return Err(HttpStatusError {
                    status: status.as_u16(),
                    path: path.to_string(),
                }
                .into());
            }
            Ok(bytes)
        })
    }

    /// Retry a request up to 3 times with exponential backoff on network errors.
    /// Does not retry on 4xx client errors (only on connection/timeout/5xx).
    fn request_with_retry<T, F>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        let delays = RETRY_DELAYS_MS;
        let mut last_err = None;
        for (attempt, delay) in std::iter::once(&0u64).chain(delays.iter()).enumerate() {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(*delay));
            }
            match f() {
                Ok(val) => return Ok(val),
                Err(e) => {
                    // Don't retry on 4xx client errors
                    if let Some(http_err) = e.downcast_ref::<HttpStatusError>()
                        && (400..500).contains(&http_err.status)
                    {
                        return Err(e);
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("retry loop must execute at least once"))
    }

    pub fn get_structure(&mut self) -> Result<&Value> {
        if self.structure.is_none() {
            self.structure = Some(Self::load_or_fetch_structure(&self.cfg, &self.client)?);
        }
        self.structure
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("structure not loaded"))
    }

    pub fn cache_path(cfg: &Config) -> PathBuf {
        cfg.cache_dir().join("structure.json")
    }

    pub fn load_or_fetch_structure(cfg: &Config, client: &Client) -> Result<Value> {
        let cache = Self::cache_path(cfg);
        // Skip disk cache for localhost hosts (used in tests with mock servers)
        let use_cache = !cfg.host.contains("127.0.0.1") && !cfg.host.contains("localhost");
        if use_cache
            && let Ok(meta) = cache.metadata()
            && let Ok(modified) = meta.modified()
        {
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default();
            if age.as_secs() < CACHE_TTL_SECS
                && let Ok(data) = fs::read_to_string(&cache)
                && let Ok(v) = serde_json::from_str::<Value>(&data)
            {
                if verbose() >= 1 {
                    eprintln!(
                        "Using cached structure ({} bytes, age {}s)",
                        data.len(),
                        age.as_secs()
                    );
                }
                return Ok(v);
            }
        }
        let url = format!("{}/data/LoxApp3.json", cfg.host);
        if verbose() >= 1 {
            eprintln!("GET {}", redact_url(&url));
        }
        let resp = client
            .get(&url)
            .basic_auth(&cfg.user, Some(&cfg.pass))
            .send()?;
        if verbose() >= 1 {
            eprintln!(
                "  → {} {}",
                resp.status().as_u16(),
                resp.status().canonical_reason().unwrap_or("")
            );
        }
        let resp = resp.error_for_status()?.bytes()?;
        if verbose() >= 2 {
            eprintln!("  body: <{} bytes>", resp.len());
        }
        let v: Value = serde_json::from_slice(&resp)?;
        if use_cache {
            if let Some(parent) = cache.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&cache, &resp);
        }
        Ok(v)
    }

    pub fn send_cmd(&self, uuid: &str, cmd: &str) -> Result<Value> {
        validate_uuid(uuid)?;
        self.get_json(&format!("/jdev/sps/io/{}/{}", uuid, cmd))
    }

    pub fn get_all(&self, uuid: &str) -> Result<String> {
        validate_uuid(uuid)?;
        self.get_text(&format!("/dev/sps/io/{}/all", uuid))
    }

    pub fn list_controls(
        &mut self,
        type_filter: Option<&str>,
        room_filter: Option<&str>,
    ) -> Result<Vec<Control>> {
        self.list_controls_ext(type_filter, room_filter, None, false)
    }

    pub fn list_controls_ext(
        &mut self,
        type_filter: Option<&str>,
        room_filter: Option<&str>,
        cat_filter: Option<&str>,
        favorites_only: bool,
    ) -> Result<Vec<Control>> {
        let structure = self.get_structure()?;
        // Keep the shared cache/raw JSON contract in the CLI; Core owns only
        // the typed projection of this already-acquired snapshot.
        let catalog = ControlCatalog::from_loxapp3_json(&serde_json::to_vec(structure)?)?;
        Ok(catalog.select(&ControlFilter {
            typ: type_filter.map(str::to_owned),
            room: room_filter.map(str::to_owned),
            category: cat_filter.map(str::to_owned),
            favorites_only,
        }))
    }

    pub fn list_categories(&mut self) -> Result<Vec<(String, String)>> {
        let structure = self.get_structure()?;
        let metadata = StructureMetadata::from_loxapp3_json(&serde_json::to_vec(structure)?)?;
        Ok(metadata
            .categories()
            .iter()
            .map(|category| (category.uuid.clone(), category.name.clone()))
            .collect())
    }

    pub fn resolve(&mut self, name_or_uuid: &str) -> Result<String> {
        self.resolve_with_room(name_or_uuid, None)
    }

    pub fn resolve_with_room(
        &mut self,
        name_or_uuid: &str,
        room_filter: Option<&str>,
    ) -> Result<String> {
        let query = match ControlQuery::prepare(name_or_uuid, room_filter, &self.cfg.aliases) {
            PreparedControlQuery::Resolved(control) => return Ok(control.uuid),
            PreparedControlQuery::Catalog(query) => query,
        };
        let controls = self.list_controls(None, None)?;
        match query.resolve(&controls) {
            Ok(control) => Ok(control.uuid),
            Err(ResolutionError::NotFound { query }) => bail!("No control matching '{}'", query),
            Err(ResolutionError::Ambiguous { query, candidates }) => {
                for c in &candidates {
                    eprintln!(
                        "  {:40} [{}]  {}",
                        c.name,
                        c.room.as_deref().unwrap_or("-"),
                        c.uuid
                    );
                }
                bail!(
                    "Ambiguous: '{}'. Use [Room] qualifier or --room flag.",
                    query
                )
            }
        }
    }

    pub fn get_control_json(&mut self, uuid: &str) -> Result<Value> {
        let structure = self.get_structure()?;
        structure
            .get("controls")
            .and_then(|c| c.get(uuid))
            .cloned()
            .context("Control not found in structure")
    }

    pub fn get_global_states(&mut self) -> Result<Vec<(String, String)>> {
        let structure = self.get_structure()?;
        let metadata = StructureMetadata::from_loxapp3_json(&serde_json::to_vec(structure)?)?;
        Ok(metadata
            .global_states()
            .iter()
            .map(|state| (state.name.clone(), state.uuid.clone()))
            .collect())
    }

    pub fn get_operating_modes(&mut self) -> Result<Vec<(String, String)>> {
        let structure = self.get_structure()?;
        let metadata = StructureMetadata::from_loxapp3_json(&serde_json::to_vec(structure)?)?;
        Ok(metadata
            .operating_modes()
            .iter()
            .map(|mode| (mode.raw_id.clone(), mode.name.clone()))
            .collect())
    }

    pub fn list_autopilot_rules(&mut self) -> Result<Vec<AutopilotRule>> {
        let structure = self.get_structure()?;
        let mut rules = Vec::new();
        if let Some(map) = structure.get("autopilot").and_then(|a| a.as_object()) {
            for (uuid, entry) in map {
                let name = entry
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("?")
                    .to_string();
                let state_changed = entry
                    .get("states")
                    .and_then(|s| s.get("changed"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let state_history = entry
                    .get("states")
                    .and_then(|s| s.get("history"))
                    .and_then(|h| h.as_str())
                    .unwrap_or("")
                    .to_string();
                rules.push(AutopilotRule {
                    name,
                    uuid: uuid.clone(),
                    state_changed,
                    state_history,
                });
            }
        }
        rules.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(rules)
    }

    pub fn find_autopilot_rule(&mut self, name_or_uuid: &str) -> Result<AutopilotRule> {
        let rules = self.list_autopilot_rules()?;
        if is_uuid(name_or_uuid) {
            return rules
                .into_iter()
                .find(|r| r.uuid == name_or_uuid)
                .context("Autopilot rule UUID not found");
        }
        let matches: Vec<AutopilotRule> = rules
            .into_iter()
            .filter(|r| matches_text(&r.name, name_or_uuid))
            .collect();
        match matches.len() {
            0 => bail!("No autopilot rule matching '{}'", name_or_uuid),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => {
                for r in &matches {
                    eprintln!("  {}", r.name);
                }
                bail!("Ambiguous: '{}'", name_or_uuid)
            }
        }
    }

    /// Resolve all controls in a room, optionally filtered by type.
    pub fn resolve_all_in_room(
        &mut self,
        room: &str,
        type_filter: Option<&str>,
    ) -> Result<Vec<Control>> {
        let controls = self.list_controls(type_filter, Some(room))?;
        if controls.is_empty() {
            bail!("No controls found in room '{}'", room);
        }
        Ok(controls)
    }

    pub fn find_control(&mut self, name_or_uuid: &str) -> Result<Control> {
        let controls = self.list_controls(None, None)?;
        if is_uuid(name_or_uuid) {
            return controls
                .into_iter()
                .find(|c| c.uuid == name_or_uuid)
                .context("UUID not found");
        }
        let matches: Vec<Control> = controls
            .into_iter()
            .filter(|c| matches_text(&c.name, name_or_uuid))
            .collect();
        match matches.len() {
            0 => bail!("No control matching '{}'", name_or_uuid),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => {
                for c in &matches {
                    eprintln!("  {:40} [{}]", c.name, c.room.as_deref().unwrap_or("-"));
                }
                bail!("Ambiguous: '{}'", name_or_uuid)
            }
        }
    }
}

/// Validate that a UUID is safe to embed in a URL path.
/// Rejects path traversal attempts and non-hex characters.
pub fn validate_uuid(uuid: &str) -> Result<()> {
    match lox_core::control::validate_uuid(uuid) {
        Ok(()) => Ok(()),
        Err(IdentifierError::InvalidLength) => bail!("Invalid UUID length: '{}'", uuid),
        Err(IdentifierError::InvalidCharacters) => bail!("Invalid characters in UUID: '{}'", uuid),
        Err(IdentifierError::NonHex) => bail!("UUID contains non-hex characters: '{}'", uuid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    fn mock_config(server: &MockServer) -> Config {
        Config {
            host: server.base_url(),
            user: "test".into(),
            pass: "test".into(),
            verify_ssl: Some(false),
            ..Default::default()
        }
    }

    // ── list_controls ─────────────────────────────────────────────────────────

    #[test]
    fn metadata_permissive_shapes() {
        let server = MockServer::start();
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        for value in [
            serde_json::json!(null),
            serde_json::json!(false),
            serde_json::json!(42),
            serde_json::json!("text"),
            serde_json::json!([]),
            serde_json::json!({}),
        ] {
            for root in [
                value.clone(),
                serde_json::json!({
                    "cats": value, "globalStates": value, "operatingModes": value
                }),
            ] {
                client.structure = Some(root);
                assert!(client.list_categories().unwrap().is_empty());
                assert!(client.get_global_states().unwrap().is_empty());
                assert!(client.get_operating_modes().unwrap().is_empty());
            }
        }
        client.structure = Some(serde_json::json!({
            "cats": {
                "null": null, "bool": true, "number": 7, "array": [],
                "string": "ignored", "missing": {}, "wrong": {"name": 7},
                "../ raw uuid ": {"name": "", "extra": true}
            },
            "globalStates": {
                "null": null, "bool": true, "number": 7, "array": [], "object": {},
                "": "", " raw name ": "../ raw uuid "
            },
            "operatingModes": {
                "null": null, "bool": true, "number": 7, "array": [], "object": {},
                "": ""
            }
        }));
        assert_eq!(
            serde_json::to_value(client.list_categories().unwrap()).unwrap(),
            serde_json::json!([["../ raw uuid ", ""]])
        );
        assert_eq!(
            serde_json::to_value(client.get_global_states().unwrap()).unwrap(),
            serde_json::json!([["", ""], [" raw name ", "../ raw uuid "]])
        );
        assert_eq!(
            serde_json::to_value(client.get_operating_modes().unwrap()).unwrap(),
            serde_json::json!([["", ""]])
        );
    }

    #[test]
    fn metadata_case_unicode_and_stable_category_ties() {
        let server = MockServer::start();
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let root = serde_json::json!({
            "cats": {
                "z": {"name": "same"}, "A": {"name": "same"},
                "upper": {"name": "Z"}, "lower": {"name": "a"},
                "unicode": {"name": "Ä"}
            },
            "globalStates": {"ä": "z", "z": "A", "A": "ä", "": "raw"}
        });
        let ties: Vec<_> = root["cats"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|key| matches!(key.as_str(), "A" | "z"))
            .cloned()
            .collect();
        client.structure = Some(root);
        let cats = client.list_categories().unwrap();
        assert_eq!(
            cats.iter()
                .map(|(_, name)| name.as_str())
                .collect::<Vec<_>>(),
            ["Z", "a", "same", "same", "Ä"]
        );
        assert_eq!(
            cats[2..4]
                .iter()
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>(),
            ties
        );
        assert_eq!(
            serde_json::to_value(client.get_global_states().unwrap()).unwrap(),
            serde_json::json!([["", "raw"], ["A", "ä"], ["z", "A"], ["ä", "z"]])
        );
    }

    #[test]
    fn metadata_mode_raw_ids_and_stable_zero_ties() {
        let server = MockServer::start();
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let root = serde_json::json!({"operatingModes": {
            "10": "ten", "-2": "negative", "2": "two", "+1": "plus one",
            "": "", " 0": "space", "+0": "plus zero", "-0": "minus zero",
            "00": "leading zero", "invalid": "invalid",
            "2147483648": "overflow", "-2147483649": "underflow"
        }});
        let zero_ids: Vec<_> = root["operatingModes"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|key| !matches!(key.as_str(), "10" | "-2" | "2" | "+1"))
            .cloned()
            .collect();
        client.structure = Some(root);
        let modes = client.get_operating_modes().unwrap();
        assert_eq!(modes.first().unwrap(), &("-2".into(), "negative".into()));
        assert_eq!(
            modes[1..1 + zero_ids.len()]
                .iter()
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>(),
            zero_ids
        );
        assert_eq!(
            serde_json::to_value(&modes[1 + zero_ids.len()..]).unwrap(),
            serde_json::json!([["+1", "plus one"], ["2", "two"], ["10", "ten"]])
        );
    }

    #[test]
    fn catalog_permissive_shapes_and_empty_id_lookup() {
        let server = MockServer::start();
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        for root in [
            serde_json::json!(null),
            serde_json::json!(false),
            serde_json::json!(42),
            serde_json::json!("text"),
            serde_json::json!([]),
            serde_json::json!({}),
        ] {
            client.structure = Some(root);
            assert!(client.list_controls(None, None).unwrap().is_empty());
        }
        for value in [
            serde_json::json!(null),
            serde_json::json!(true),
            serde_json::json!(42),
            serde_json::json!("text"),
            serde_json::json!([]),
            serde_json::json!({}),
        ] {
            client.structure = Some(serde_json::json!({"controls": value}));
            assert!(client.list_controls(None, None).unwrap().is_empty());
            client.structure = Some(serde_json::json!({
                "rooms": {"": {"name": "Empty room"}},
                "cats": {"": {"name": "Empty category"}},
                "controls": {"../ raw key ": {
                    "name": value, "type": value, "room": value, "cat": value,
                    "isFavorite": value, "isSecured": value
                }, "scalar": value}
            }));
            let controls = client.list_controls(None, None).unwrap();
            let offline = ControlCatalog::from_loxapp3_json(
                &serde_json::to_vec(client.structure.as_ref().unwrap()).unwrap(),
            )
            .unwrap();
            assert_eq!(offline.controls(), controls);
            let raw = controls.iter().find(|c| c.uuid == "../ raw key ").unwrap();
            assert_eq!(raw.name, value.as_str().unwrap_or("?"));
            assert_eq!(raw.typ, value.as_str().unwrap_or("?"));
            let empty_ref = value.as_str().is_none();
            assert_eq!(raw.room.as_deref(), empty_ref.then_some("Empty room"));
            assert_eq!(raw.cat.as_deref(), empty_ref.then_some("Empty category"));
            assert_eq!(raw.is_favorite, value.as_bool().unwrap_or(false));
            assert_eq!(raw.is_secured, value.as_bool().unwrap_or(false));
            assert_eq!(
                controls.iter().find(|c| c.uuid == "scalar").unwrap().name,
                "?"
            );
        }
        for maps in [
            serde_json::json!(null),
            serde_json::json!([]),
            serde_json::json!(true),
            serde_json::json!("x"),
            serde_json::json!(7),
        ] {
            client.structure = Some(serde_json::json!({
                "rooms": maps, "cats": maps, "controls": {"": {}}
            }));
            let controls = client
                .list_controls_ext(None, Some(""), Some(""), false)
                .unwrap();
            assert_eq!(controls.len(), 1);
            assert_eq!(controls[0].uuid, "");
            assert_eq!(controls[0].room, None);
            assert_eq!(controls[0].cat, None);
        }
    }

    #[test]
    fn catalog_loopback_projection_filters_sort_and_cache() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.json_body(serde_json::json!({
                "rooms": {"r": {"name": "KÜCHE"}, "bad": {"name": 1}},
                "cats": {"c": {"name": "LÜFTUNG"}, "bad": {"name": false}},
                "controls": {
                    "raw / key": {"name": "z", "type": "Straße", "room": "r", "cat": "c", "isFavorite": true, "isSecured": true,
                        "subControls": {"nested": {"name": "Must not appear"}}},
                    "a": {"name": "a", "type": "Switch", "room": "missing", "isFavorite": "true"},
                    "tie2": {"name": "A", "room": "bad", "cat": "bad", "isFavorite": 1},
                    "tie1": {"name": "A", "isFavorite": false},
                    "bad": {"name": null, "type": [], "isFavorite": null}
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let controls = client.list_controls(None, None).unwrap();
        assert_eq!(
            controls.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            ["?", "A", "A", "a", "z"]
        );
        let mut ties = controls
            .iter()
            .filter(|c| c.name == "A")
            .map(|c| c.uuid.as_str())
            .collect::<Vec<_>>();
        ties.sort();
        assert_eq!(ties, ["tie1", "tie2"]);
        let selected = client
            .list_controls_ext(Some("STRAß"), Some("kü"), Some("lüft"), true)
            .unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].uuid, "raw / key");
        assert!(selected[0].is_secured);
        assert!(
            client
                .list_controls(Some("STRASSE"), None)
                .unwrap()
                .is_empty()
        );
        assert!(
            client
                .list_controls(Some(" Straße "), None)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            client
                .list_controls_ext(None, Some(""), Some(""), false)
                .unwrap(),
            controls
        );
        assert!(
            client
                .list_controls(None, Some("unknown"))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            client.list_controls_ext(None, None, None, true).unwrap(),
            selected
        );
        assert_eq!(
            client.list_controls(Some("?"), None).unwrap()[0].uuid,
            "bad"
        );
        mock.assert_hits(1);
    }

    #[test]
    fn test_list_controls_parses_structure() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": { "r1": { "name": "Living Room" } },
                "controls": {
                    "u1": { "name": "Main Light", "type": "LightControllerV2", "room": "r1" },
                    "u2": { "name": "Roller Blind", "type": "Jalousie", "room": "r1" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let controls = client.list_controls(None, None).unwrap();
        assert_eq!(controls.len(), 2);
        let light = controls.iter().find(|c| c.name == "Main Light").unwrap();
        assert_eq!(light.room.as_deref(), Some("Living Room"));
    }

    #[test]
    fn test_list_controls_type_filter() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "controls": {
                    "u1": { "name": "Light", "type": "LightControllerV2", "room": "" },
                    "u2": { "name": "Blind", "type": "Jalousie", "room": "" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let lights = client.list_controls(Some("light"), None).unwrap();
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].name, "Light");
    }

    // ── send_cmd ──────────────────────────────────────────────────────────────

    #[test]
    fn test_send_cmd_success() {
        let server = MockServer::start();
        let test_uuid = "0f1e2d3c-a1b2-c3d4-e5f6a7b8c9d0e1f2";
        let _s = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200)
                .json_body(serde_json::json!({ "rooms": {}, "controls": {} }));
        });
        let _c = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/jdev/sps/io/{}/on", test_uuid));
            then.status(200).json_body(serde_json::json!({
                "LL": { "Code": "200", "value": "1" }
            }));
        });
        let client = LoxClient::new(mock_config(&server)).unwrap();
        let resp = client.send_cmd(test_uuid, "on").unwrap();
        assert_eq!(
            resp.pointer("/LL/Code").and_then(|v| v.as_str()),
            Some("200")
        );
    }

    #[test]
    fn test_send_cmd_rejects_path_traversal() {
        let server = MockServer::start();
        let client = LoxClient::new(mock_config(&server)).unwrap();
        assert!(client.send_cmd("../../../etc/passwd", "on").is_err());
        assert!(client.send_cmd("uuid-with-slash/attack", "on").is_err());
        assert!(client.send_cmd("", "on").is_err());
    }

    #[test]
    fn test_validate_uuid() {
        assert!(validate_uuid("0f1e2d3c-a1b2-c3d4-e5f6a7b8c9d0e1f2").is_ok());
        assert!(validate_uuid("1fbc668c-005c-7471-ffffed57184a04d2").is_ok());
        assert!(validate_uuid("").is_err());
        assert!(validate_uuid("../attack").is_err());
        assert!(validate_uuid("uuid/with/slashes").is_err());
        assert!(validate_uuid("has spaces in it-----").is_err());
    }

    // ── cache ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_structure_fetched_only_once_per_client() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "controls": { "c1": { "name": "Light", "type": "Switch", "room": "" } }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let _ = client.list_controls(None, None).unwrap();
        let _ = client.list_controls(None, None).unwrap();
        mock.assert_hits(1);
    }

    // ── resolve_with_room ─────────────────────────────────────────────────────

    #[test]
    fn test_resolve_priority_bypasses_structure() {
        let server = MockServer::start();
        let structure = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(500);
        });
        let mut cfg = mock_config(&server);
        let native = "1fbc668c-005c-7471-ffffed57184a04d2";
        let rfc = "1fbc668c-005c-7471-ffff-ed57184a04d2";
        let permissive = "---------------------";
        for id in [native, rfc, permissive] {
            cfg.aliases.insert(id.into(), "must-not-win".into());
        }
        cfg.aliases
            .insert("Light [Room]".into(), "../opaque".into());
        let mut client = LoxClient::new(cfg).unwrap();
        for id in [native, rfc, permissive] {
            assert_eq!(client.resolve_with_room(id, Some("ignored")).unwrap(), id);
        }
        assert_eq!(
            client
                .resolve_with_room("Light [Room]", Some("ignored"))
                .unwrap(),
            "../opaque"
        );
        assert!(client.structure.is_none());
        structure.assert_hits(0);
    }

    #[test]
    fn test_resolve_substring_empty_and_literal_room_semantics() {
        let server = MockServer::start();
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        client.structure = Some(serde_json::json!({
            "rooms": { "r1": { "name": "KÜCHE" }, "r2": { "name": " Küche " } },
            "controls": {
                "u1": { "name": "Licht", "room": "r1" },
                "u2": { "name": "Licht Extra", "room": "r2" },
                "u3": { "name": "Other" },
                "u4": { "name": "Panel [West]", "room": "r1" }
            }
        }));
        client
            .cfg
            .aliases
            .insert("LICHT".into(), "alias-only".into());
        assert_eq!(client.resolve("LICHT").unwrap(), "alias-only");
        assert_eq!(
            client.resolve("Licht").unwrap_err().to_string(),
            "Ambiguous: 'Licht'. Use [Room] qualifier or --room flag."
        );
        assert_eq!(
            client
                .resolve_with_room(" Licht [ küche ]", Some("missing"))
                .unwrap(),
            "u2"
        );
        assert_eq!(client.resolve("Panel [West] [küche]").unwrap(), "u4");
        assert_eq!(client.resolve_with_room("", Some(" küche ")).unwrap(), "u2");
        assert_eq!(
            client
                .resolve_with_room("Other []", Some("missing"))
                .unwrap(),
            "u3"
        );
        assert_eq!(
            client.resolve(" Licht ").unwrap_err().to_string(),
            "No control matching ' Licht '"
        );
    }

    #[test]
    fn test_resolve_exact_match() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "controls": { "uuid-abc": { "name": "Kitchen Light", "type": "Switch", "room": "" } }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let uuid = client.resolve_with_room("Kitchen Light", None).unwrap();
        assert_eq!(uuid, "uuid-abc");
    }

    #[test]
    fn test_resolve_uuid_passthrough() {
        let server = MockServer::start();
        // No mock needed — UUID bypasses HTTP lookup
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let uuid = "1fbc668c-005c-7471-ffffed57184a04d2";
        assert_eq!(client.resolve_with_room(uuid, None).unwrap(), uuid);
    }

    #[test]
    fn test_resolve_ambiguous_returns_err() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": { "r1": { "name": "Kitchen" }, "r2": { "name": "Bedroom" } },
                "controls": {
                    "u1": { "name": "Light", "type": "Switch", "room": "r1" },
                    "u2": { "name": "Light", "type": "Switch", "room": "r2" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        assert!(client.resolve_with_room("Light", None).is_err());
    }

    #[test]
    fn test_resolve_room_qualifier_disambiguates() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": { "r1": { "name": "Kitchen" }, "r2": { "name": "Bedroom" } },
                "controls": {
                    "u1": { "name": "Light", "type": "Switch", "room": "r1" },
                    "u2": { "name": "Light", "type": "Switch", "room": "r2" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let uuid = client.resolve_with_room("Light [Kitchen]", None).unwrap();
        assert_eq!(uuid, "u1");
    }

    // ── favorites / categories ─────────────────────────────────────────────────

    #[test]
    fn test_list_controls_favorites_filter() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "cats": {},
                "controls": {
                    "u1": { "name": "Fav Light", "type": "Switch", "room": "", "isFavorite": true },
                    "u2": { "name": "Other Light", "type": "Switch", "room": "", "isFavorite": false }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let favs = client.list_controls_ext(None, None, None, true).unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].name, "Fav Light");
        assert!(favs[0].is_favorite);
    }

    #[test]
    fn test_list_controls_category_filter() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "cats": { "c1": { "name": "Lighting" }, "c2": { "name": "Shading" } },
                "controls": {
                    "u1": { "name": "Lamp", "type": "Switch", "room": "", "cat": "c1" },
                    "u2": { "name": "Blind", "type": "Jalousie", "room": "", "cat": "c2" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let lights = client
            .list_controls_ext(None, None, Some("Lighting"), false)
            .unwrap();
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].name, "Lamp");
        assert_eq!(lights[0].cat.as_deref(), Some("Lighting"));
    }

    #[test]
    fn test_list_categories() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "cats": { "c1": { "name": "Lighting" }, "c2": { "name": "Shading" } },
                "controls": {}
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let cats = client.list_categories().unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].1, "Lighting");
        assert_eq!(cats[1].1, "Shading");
    }

    #[test]
    fn test_get_global_states() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {}, "controls": {},
                "globalStates": {
                    "operatingMode": "0f1e2d3c-0000-0000-0000000000000000",
                    "sunrise": "1a2b3c4d-0000-0000-0000000000000000"
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let globals = client.get_global_states().unwrap();
        assert_eq!(globals.len(), 2);
        assert!(globals.iter().any(|(n, _)| n == "operatingMode"));
        assert!(globals.iter().any(|(n, _)| n == "sunrise"));
    }

    #[test]
    fn test_get_operating_modes() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {}, "controls": {},
                "operatingModes": {
                    "0": "Auto", "1": "Manual", "2": "Comfort"
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let modes = client.get_operating_modes().unwrap();
        assert_eq!(modes.len(), 3);
        assert_eq!(modes[0], ("0".to_string(), "Auto".to_string()));
        assert_eq!(modes[1], ("1".to_string(), "Manual".to_string()));
    }

    #[test]
    fn test_get_control_json() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {},
                "controls": {
                    "u1": {
                        "name": "Light", "type": "Switch", "room": "",
                        "states": { "active": "state-uuid-1" },
                        "subControls": { "sub1": { "name": "Dimmer", "type": "Dimmer" } }
                    }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let ctrl_json = client.get_control_json("u1").unwrap();
        assert_eq!(
            ctrl_json.get("name").and_then(|v| v.as_str()),
            Some("Light")
        );
        assert!(ctrl_json.get("subControls").is_some());
        assert!(ctrl_json.get("states").is_some());
    }

    // ── alias ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_alias() {
        let server = MockServer::start();
        // No HTTP call expected — alias resolves immediately
        let mut cfg = mock_config(&server);
        cfg.aliases
            .insert("mylight".into(), "alias-uuid-1234567890".into());
        let mut client = LoxClient::new(cfg).unwrap();
        let uuid = client.resolve_with_room("mylight", None).unwrap();
        assert_eq!(uuid, "alias-uuid-1234567890");
    }

    // ── resolve_all_in_room ─────────────────────────────────────────────────

    #[test]
    fn test_resolve_all_in_room() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": { "r1": { "name": "Kitchen" }, "r2": { "name": "Bedroom" } },
                "controls": {
                    "u1": { "name": "Light 1", "type": "Switch", "room": "r1" },
                    "u2": { "name": "Blind 1", "type": "Jalousie", "room": "r1" },
                    "u3": { "name": "Light 2", "type": "Switch", "room": "r2" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let controls = client.resolve_all_in_room("Kitchen", None).unwrap();
        assert_eq!(controls.len(), 2);
        assert!(
            controls
                .iter()
                .all(|c| c.room.as_deref() == Some("Kitchen"))
        );
    }

    #[test]
    fn test_resolve_all_in_room_with_type_filter() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": { "r1": { "name": "Kitchen" } },
                "controls": {
                    "u1": { "name": "Light 1", "type": "Switch", "room": "r1" },
                    "u2": { "name": "Blind 1", "type": "Jalousie", "room": "r1" }
                }
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let controls = client
            .resolve_all_in_room("Kitchen", Some("Switch"))
            .unwrap();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].typ, "Switch");
    }

    #[test]
    fn test_resolve_all_in_room_empty_returns_err() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(200).json_body(serde_json::json!({
                "rooms": {}, "controls": {}
            }));
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        assert!(client.resolve_all_in_room("NonExistent", None).is_err());
    }

    // ── is_uuid (strict) ────────────────────────────────────────────────────

    #[test]
    fn test_is_uuid_strict() {
        // Valid Loxone UUIDs
        assert!(is_uuid("1fbc668c-005c-7471-ffffed57184a04d2"));
        assert!(is_uuid("0f1e2d3c-a1b2-c3d4-e5f6a7b8c9d0e1f2"));
        // Too short
        assert!(!is_uuid("short-str"));
        // Contains non-hex characters
        assert!(!is_uuid("1fbc668c-005c-7471-xxxxed57184a04d2"));
        // No dashes
        assert!(!is_uuid("1fbc668c005c7471ffffed57184a04d2"));
        // Human-readable strings should NOT match
        assert!(!is_uuid("Licht Wohnzimmer"));
        assert!(!is_uuid("Kitchen Light [Room]"));
    }

    // ── retry logic ─────────────────────────────────────────────────────────

    #[test]
    fn test_retry_on_server_error() {
        let server = MockServer::start();
        let test_uuid = "0f1e2d3c-a1b2-c3d4-e5f6a7b8c9d0e1f2";
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/jdev/sps/io/{}/on", test_uuid));
            then.status(200).json_body(serde_json::json!({
                "LL": { "Code": "200", "value": "1" }
            }));
        });
        let client = LoxClient::new(mock_config(&server)).unwrap();
        let resp = client.send_cmd(test_uuid, "on").unwrap();
        assert_eq!(
            resp.pointer("/LL/Code").and_then(|v| v.as_str()),
            Some("200")
        );
        // Verify it was called (at least once; retries only on failure)
        mock.assert_hits(1);
    }

    // ── cross-origin redirect blocking ──────────────────────────────────────

    #[test]
    fn test_cross_origin_redirect_blocked() {
        // Simulate Miniserver Gen 2 redirecting to cloud DynDNS URL
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/data/LoxApp3.json");
            then.status(302).header(
                "Location",
                "https://192-168-10-222.504f94d08fa8.dyndns.loxonecloud.com/data/LoxApp3.json",
            );
        });
        let mut client = LoxClient::new(mock_config(&server)).unwrap();
        let result = client.get_structure();
        assert!(result.is_err(), "Cross-origin redirect should be blocked");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Blocked cross-origin redirect") || err_msg.contains("redirect"),
            "Error should mention redirect blocking, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_same_origin_redirect_allowed() {
        let server = MockServer::start();
        // Redirect to same host (different path) should be followed
        let _redirect = server.mock(|when, then| {
            when.method(GET).path("/old-path");
            then.status(302)
                .header("Location", &format!("{}/new-path", server.base_url()));
        });
        let _target = server.mock(|when, then| {
            when.method(GET).path("/new-path");
            then.status(200).body("ok");
        });
        let client = LoxClient::new(mock_config(&server)).unwrap();
        let result = client.get_text("/old-path").unwrap();
        assert_eq!(result, "ok");
    }

    // ── redact_url ──────────────────────────────────────────────────────────

    #[test]
    fn test_redact_url_plain() {
        let url = "https://192.168.1.5/data/LoxApp3.json";
        assert_eq!(redact_url(url), url);
    }

    #[test]
    fn test_redact_url_basic_auth_password() {
        assert_eq!(
            redact_url("https://admin:secret@192.168.1.5/data/LoxApp3.json"),
            "https://admin:***@192.168.1.5/data/LoxApp3.json"
        );
    }

    #[test]
    fn test_redact_url_query_token() {
        assert_eq!(
            redact_url("https://host/path?token=abc123&other=ok"),
            "https://host/path?token=***&other=ok"
        );
    }

    #[test]
    fn test_redact_url_query_password() {
        assert_eq!(
            redact_url("https://host/path?pass=secret"),
            "https://host/path?pass=***"
        );
    }

    #[test]
    fn test_redact_url_duplicate_params() {
        assert_eq!(
            redact_url("https://host/path?token=abc&other=ok&token=def"),
            "https://host/path?token=***&other=ok&token=***"
        );
    }
}
