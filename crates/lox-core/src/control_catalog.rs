//! Read-only projection of top-level LoxApp3 controls, without acquisition or I/O.

use crate::control::{Control, matches_text};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Owned control projection, not a model of the complete LoxApp3 document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlCatalog {
    controls: Vec<Control>,
}

/// Unicode-lowercased substring filters, using the existing control matcher.
/// Missing room/category names are matched as empty strings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlFilter {
    pub typ: Option<String>,
    pub room: Option<String>,
    pub category: Option<String>,
    pub favorites_only: bool,
}

/// Invalid JSON syntax/encoding, not a rejection of a permissive document shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogDecodeError {
    InvalidJson { line: usize, column: usize },
}

impl fmt::Display for CatalogDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { line, column } => {
                write!(f, "invalid LoxApp3 JSON at line {line} column {column}")
            }
        }
    }
}

impl std::error::Error for CatalogDecodeError {}

impl ControlCatalog {
    /// Decode serialized LoxApp3 JSON into the existing seven-field control model.
    ///
    /// Any valid JSON root is accepted. Missing/null/non-object maps are empty;
    /// every top-level control key is eligible, even if its value is not an object.
    /// Missing/non-string name and type become `"?"`. Room/category references
    /// default to the empty ID and resolve only string names. Flags are bool-only.
    /// IDs remain opaque; subcontrols and unrelated fields are ignored.
    ///
    /// Controls sort only by case-sensitive name; equal-name order is unspecified.
    /// This is not a document validator, raw-tree API or resource-limit policy.
    pub fn from_loxapp3_json(bytes: &[u8]) -> Result<Self, CatalogDecodeError> {
        let structure: Value =
            serde_json::from_slice(bytes).map_err(|error| CatalogDecodeError::InvalidJson {
                line: error.line(),
                column: error.column(),
            })?;
        let rooms = named_entries(&structure, "rooms");
        let categories = named_entries(&structure, "cats");
        let mut controls = Vec::new();
        if let Some(map) = structure.get("controls").and_then(Value::as_object) {
            for (uuid, control) in map {
                let string = |field| control.get(field).and_then(Value::as_str);
                controls.push(Control {
                    name: string("name").unwrap_or("?").to_owned(),
                    uuid: uuid.clone(),
                    typ: string("type").unwrap_or("?").to_owned(),
                    room: rooms
                        .get(string("room").unwrap_or(""))
                        .map(|s| (*s).to_owned()),
                    cat: categories
                        .get(string("cat").unwrap_or(""))
                        .map(|s| (*s).to_owned()),
                    is_favorite: control
                        .get("isFavorite")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    is_secured: control
                        .get("isSecured")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                });
            }
        }
        controls.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { controls })
    }

    /// All projected controls, in case-sensitive name order.
    pub fn controls(&self) -> &[Control] {
        &self.controls
    }

    /// Return an owned selection without mutating the snapshot.
    pub fn select(&self, filter: &ControlFilter) -> Vec<Control> {
        self.controls
            .iter()
            .filter(|control| {
                filter
                    .typ
                    .as_deref()
                    .is_none_or(|query| matches_text(&control.typ, query))
                    && filter.room.as_deref().is_none_or(|query| {
                        matches_text(control.room.as_deref().unwrap_or(""), query)
                    })
                    && filter.category.as_deref().is_none_or(|query| {
                        matches_text(control.cat.as_deref().unwrap_or(""), query)
                    })
                    && (!filter.favorites_only || control.is_favorite)
            })
            .cloned()
            .collect()
    }
}

fn named_entries<'a>(structure: &'a Value, key: &str) -> HashMap<&'a str, &'a str> {
    structure
        .get(key)
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(id, entry)| Some((id.as_str(), entry.get("name")?.as_str()?)))
        .collect()
}
