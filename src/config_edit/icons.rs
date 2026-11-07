//! Exact built-in catalog membership; no project image decoding or GUI rendering.

use std::collections::HashSet;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::schema::Diagnostic;

pub(super) fn native_identity(raw: &str) -> Option<Uuid> {
    let groups: Vec<_> = raw.split('-').collect();
    let lengths: Vec<_> = groups.iter().map(|group| group.len()).collect();
    if !matches!(lengths.as_slice(), [8, 4, 4, 16] | [8, 4, 4, 4, 12])
        || !groups
            .iter()
            .all(|group| group.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return None;
    }
    Uuid::parse_str(&groups.concat()).ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IconCatalogProvenance {
    pub version: String,
    pub archive_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuiltinIcon {
    pub native_id: String,
    pub filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuiltinIconCatalog {
    schema_version: u32,
    pub catalog: IconCatalogProvenance,
    pub icons: Vec<BuiltinIcon>,
}

#[derive(Debug, Serialize)]
pub struct IconResolution {
    pub status: &'static str,
    pub native_id: Option<String>,
    pub filename: Option<String>,
}

impl BuiltinIconCatalog {
    pub fn embedded() -> Result<Self> {
        Self::parse(include_str!("../../docs/schemas/loxone-icons.json"))
    }

    pub(super) fn parse(raw: &str) -> Result<Self> {
        let invalid = |reason: &str| {
            Diagnostic::new(
                "invalid_icon_catalog",
                "docs/schemas/loxone-icons.json",
                format!(
                    "{reason}; regenerate the built-in catalog with scripts/inventory-loxone-icons.py and rebuild."
                ),
            )
        };
        let parsed: Self = serde_json::from_str(raw)
            .map_err(|error| invalid(&format!("Cannot parse embedded icon catalog: {error}")))?;
        if parsed.schema_version != 1
            || parsed.catalog.version.is_empty()
            || !parsed
                .catalog
                .version
                .bytes()
                .all(|byte| byte.is_ascii_digit())
            || parsed.catalog.archive_sha256.len() != 64
            || !parsed
                .catalog
                .archive_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || parsed.icons.is_empty()
        {
            return Err(invalid("Unsupported schema or inconsistent catalog provenance").into());
        }
        let mut ids = HashSet::new();
        let mut filenames = HashSet::new();
        for icon in &parsed.icons {
            let id = native_identity(&icon.native_id)
                .filter(|id| !id.is_nil())
                .filter(|_| icon.native_id.len() == 35)
                .ok_or_else(|| invalid("Malformed or zero native catalog identity"))?;
            if !ids.insert(id) || !filenames.insert(&icon.filename) {
                return Err(invalid("Duplicate catalog identity or canonical filename").into());
            }
            if !icon.filename.ends_with(".svg")
                || icon.filename.contains(['/', '\\'])
                || icon.filename.chars().any(char::is_control)
            {
                return Err(invalid("Invalid canonical catalog filename").into());
            }
        }
        Ok(parsed)
    }

    pub fn select(&self, selection: &str) -> Result<&BuiltinIcon> {
        let invalid = |code, reason: &str| {
            Diagnostic::new(
                code,
                "Load/@ico",
                format!(
                    "{reason} Use `lox config energy icons` for exact case-sensitive filenames or native IDs; use --no-icon to clear."
                ),
            )
        };
        if selection.starts_with("project:") {
            return Err(invalid(
                "unsupported_custom_icon",
                "Project/custom icon selection and import are not yet verified or supported.",
            )
            .into());
        }
        let selection = selection.strip_prefix("builtin:").unwrap_or(selection);
        if selection.is_empty() {
            return Err(invalid(
                "invalid_icon_selection",
                "An empty icon is not a selection.",
            )
            .into());
        }
        if let Some(icon) = self.icons.iter().find(|icon| icon.filename == selection) {
            return Ok(icon);
        }
        if let Some(id) = native_identity(selection) {
            if id.is_nil() {
                return Err(invalid(
                    "explicit_icon_clear_required",
                    "The all-zero ID requires explicit --no-icon.",
                )
                .into());
            }
            if let Some(icon) = self
                .icons
                .iter()
                .find(|icon| native_identity(&icon.native_id) == Some(id))
            {
                return Ok(icon);
            }
        }
        Err(invalid(
            "unknown_builtin_icon",
            &format!("'{selection}' is not an exact loaded built-in member of catalog {}; disabled, malformed, unknown and custom references cannot be newly assigned.", self.catalog.version),
        ).into())
    }

    pub fn resolve(&self, raw: Option<&str>) -> IconResolution {
        let (status, found) = match raw {
            None => ("none", None),
            Some(raw) => match native_identity(raw) {
                Some(id) if id.is_nil() => ("none", None),
                Some(id) => match self
                    .icons
                    .iter()
                    .find(|icon| native_identity(&icon.native_id) == Some(id))
                {
                    Some(icon) => ("known_builtin", Some(icon)),
                    None => ("unresolved", None),
                },
                None => ("malformed_id", None),
            },
        };
        IconResolution {
            status,
            native_id: found.map(|icon| icon.native_id.clone()),
            filename: found.map(|icon| icon.filename.clone()),
        }
    }
}
