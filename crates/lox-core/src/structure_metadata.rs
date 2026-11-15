//! Read-only metadata projections of an already-acquired LoxApp3 snapshot.

pub use crate::control_catalog::CatalogDecodeError as MetadataDecodeError;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalState {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatingMode {
    pub raw_id: String,
    pub name: String,
}

/// Owned metadata only, not a complete document or identifier validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureMetadata {
    categories: Vec<Category>,
    global_states: Vec<GlobalState>,
    operating_modes: Vec<OperatingMode>,
}

impl StructureMetadata {
    /// Accept any valid JSON root; absent/non-object maps contribute no entries.
    /// Categories require a string `name`; global states and modes require string
    /// values. Empty strings and opaque IDs are retained without normalization.
    ///
    /// Stable sorts preserve JSON map iteration order for equal category names
    /// and mode sort keys. Mode keys use Rust's `i32` parse, falling back to zero.
    /// The caller's serde_json feature set determines map iteration order.
    pub fn from_loxapp3_json(bytes: &[u8]) -> Result<Self, MetadataDecodeError> {
        let structure: Value =
            serde_json::from_slice(bytes).map_err(|error| MetadataDecodeError::InvalidJson {
                line: error.line(),
                column: error.column(),
            })?;
        let mut categories = Vec::new();
        if let Some(map) = structure.get("cats").and_then(Value::as_object) {
            for (uuid, category) in map {
                if let Some(name) = category.get("name").and_then(Value::as_str) {
                    categories.push(Category {
                        uuid: uuid.clone(),
                        name: name.to_owned(),
                    });
                }
            }
        }
        categories.sort_by(|a, b| a.name.cmp(&b.name));

        let mut global_states = Vec::new();
        if let Some(map) = structure.get("globalStates").and_then(Value::as_object) {
            for (name, value) in map {
                if let Some(uuid) = value.as_str() {
                    global_states.push(GlobalState {
                        name: name.clone(),
                        uuid: uuid.to_owned(),
                    });
                }
            }
        }
        global_states.sort_by(|a, b| a.name.cmp(&b.name));

        let mut operating_modes = Vec::new();
        if let Some(map) = structure.get("operatingModes").and_then(Value::as_object) {
            for (raw_id, value) in map {
                if let Some(name) = value.as_str() {
                    operating_modes.push(OperatingMode {
                        raw_id: raw_id.clone(),
                        name: name.to_owned(),
                    });
                }
            }
        }
        operating_modes.sort_by(|a, b| {
            a.raw_id
                .parse::<i32>()
                .unwrap_or(0)
                .cmp(&b.raw_id.parse::<i32>().unwrap_or(0))
        });
        Ok(Self {
            categories,
            global_states,
            operating_modes,
        })
    }

    /// Categories in case-sensitive name order.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// Global-state references in case-sensitive state-name order.
    pub fn global_states(&self) -> &[GlobalState] {
        &self.global_states
    }

    /// Modes in stable numeric-key order, retaining the original key strings.
    pub fn operating_modes(&self) -> &[OperatingMode] {
        &self.operating_modes
    }
}
