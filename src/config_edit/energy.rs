//! Version-bound EnergyManager2 storage edits, not a runtime or GUI parity engine.

use std::collections::{BTreeMap, HashSet};

use anyhow::Result;
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::ConfigEditor;
use super::icons::{BuiltinIconCatalog, IconCatalogProvenance, IconResolution, native_identity};
use super::schema::{Diagnostic, NumericBinding, StoredValue, minutes_to_seconds};

#[derive(Debug, Clone, Copy)]
pub enum EnergyParameter {
    Offset,
    MinimumStateOfCharge,
    MaximumStoragePower,
}

impl EnergyParameter {
    fn binding(self) -> NumericBinding {
        match self {
            Self::Offset => NumericBinding {
                key: "O",
                unit: "kW",
                default: 0.0,
                min: None,
                max: None,
            },
            Self::MinimumStateOfCharge => NumericBinding {
                key: "MinSoc",
                unit: "%",
                default: 0.0,
                min: Some(0.0),
                max: Some(100.0),
            },
            Self::MaximumStoragePower => NumericBinding {
                key: "MaxSpwr",
                unit: "kW",
                default: 0.0,
                min: Some(0.0),
                max: None,
            },
        }
    }
}

impl TryFrom<&str> for EnergyParameter {
    type Error = Diagnostic;

    fn try_from(key: &str) -> std::result::Result<Self, Self::Error> {
        match key {
            "O" => Ok(Self::Offset),
            "MinSoc" => Ok(Self::MinimumStateOfCharge),
            "MaxSpwr" => Ok(Self::MaximumStoragePower),
            _ => Err(Diagnostic::new(
                "unsupported_parameter",
                key,
                "EnergyManager2 supports O, MinSoc and MaxSpwr; inspect the block before editing.",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EnergyLoadMode {
    Digital,
    Analog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyDeadline {
    Sunset,
    Dusk,
    Sunrise,
    Dawn,
    Hour(u8),
}

impl std::str::FromStr for EnergyDeadline {
    type Err = Diagnostic;

    fn from_str(raw: &str) -> std::result::Result<Self, Self::Err> {
        match raw {
            "sunset" => Ok(Self::Sunset),
            "dusk" => Ok(Self::Dusk),
            "sunrise" => Ok(Self::Sunrise),
            "dawn" => Ok(Self::Dawn),
            _ => raw
                .strip_suffix(":00")
                .filter(|hour| hour.len() == 2 && hour.bytes().all(|byte| byte.is_ascii_digit()))
                .and_then(|hour| hour.parse::<u8>().ok())
                .filter(|hour| (1..=24).contains(hour))
                .map(Self::Hour)
                .ok_or_else(Self::invalid),
        }
    }
}

impl EnergyDeadline {
    fn invalid() -> Diagnostic {
        Diagnostic::new(
            "invalid_deadline",
            "Load/@tpR",
            "Use sunset, dusk, sunrise, dawn, or an hourly time 01:00..24:00.",
        )
    }

    fn storage(self) -> std::result::Result<i32, Diagnostic> {
        match self {
            Self::Sunset => Ok(0),
            Self::Dusk => Ok(1),
            Self::Sunrise => Ok(2),
            Self::Dawn => Ok(3),
            Self::Hour(hour @ 1..=24) => Ok(i32::from(hour) + 3),
            Self::Hour(_) => Err(Self::invalid()),
        }
    }
}

#[derive(Debug, Default)]
pub struct EnergyLoadPatch {
    pub name: Option<String>,
    pub mode: Option<EnergyLoadMode>,
    pub preparation_seconds: Option<i64>,
    pub min_runtime_minutes: Option<i64>,
    pub daily_minutes: Option<i64>,
    pub deadline: Option<EnergyDeadline>,
    pub rated_power: Option<String>,
    pub switch_on_power: Option<String>,
    pub icon: EnergyIconChange,
}

#[derive(Debug, Default)]
pub enum EnergyIconChange {
    #[default]
    Preserve,
    SetBuiltin(String),
    Clear,
}

#[derive(Debug, Serialize)]
pub struct EnergyScalar {
    pub key: String,
    pub direction: &'static str,
    pub unit: &'static str,
    pub connector_present: bool,
    pub value: StoredValue<f64>,
}

#[derive(Debug, Serialize)]
pub struct EnergyLoad {
    pub priority: usize,
    pub id: Option<String>,
    pub output_index: StoredValue<i32>,
    pub output: Option<String>,
    pub used: StoredValue<bool>,
    pub name: StoredValue<String>,
    pub name_customized: StoredValue<bool>,
    pub analog: StoredValue<bool>,
    pub mode_customized: StoredValue<bool>,
    pub preparation_seconds: StoredValue<i32>,
    pub min_runtime_seconds: StoredValue<i32>,
    pub daily_seconds: StoredValue<i32>,
    pub deadline: StoredValue<i32>,
    pub deadline_label: Option<String>,
    pub deadline_gui_verified: bool,
    pub switch_on_power: StoredValue<f64>,
    pub rated_power: StoredValue<f64>,
    pub unused_feedback: Option<bool>,
    pub icon: Option<String>,
    pub icon_resolution: IconResolution,
    /// Inspection only: this is not a replacement/export format for XML records.
    pub attributes: BTreeMap<String, String>,
    pub child_elements: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EnergyInspection {
    pub block_type: &'static str,
    pub title: String,
    pub uuid: String,
    pub verification: &'static str,
    pub scalar_parameters: Vec<EnergyScalar>,
    pub loads: Vec<EnergyLoad>,
    pub load_edit_supported: bool,
    pub builtin_icon_catalog: IconCatalogProvenance,
    pub storage_power_feedback: EnergyStoragePowerFeedback,
    pub diagnostics: Vec<Diagnostic>,
    pub limitations: Vec<&'static str>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "status", content = "reason", rename_all = "snake_case")]
pub enum EnergyStoragePowerFeedback {
    NotTriggered,
    Missing,
    Unknown(&'static str),
}

fn energy_connector<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    let mut connectors = element
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|co| co.name == "Co" && co.attributes.get("K").map(String::as_str) == Some(key));
    let connector = connectors.next()?;
    connectors.next().is_none().then_some(connector)
}

fn native_connection_count(connector: &Element) -> Option<usize> {
    let inputs: Vec<_> = connector
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|child| child.name == "In")
        .collect();
    let count = match connector.attributes.get("Nc") {
        Some(raw) => usize::try_from(raw.parse::<i32>().ok()?).ok()?,
        None => 0,
    };
    // The native reader uses Nc, not the mere presence of In elements.
    (count == inputs.len()
        && inputs.iter().all(|input| {
            input
                .attributes
                .get("Input")
                .and_then(|raw| native_identity(raw))
                .is_some_and(|id| !id.is_nil())
        }))
    .then_some(count)
}

fn storage_power_feedback(
    element: &Element,
    scalars: &[EnergyScalar],
) -> EnergyStoragePowerFeedback {
    use EnergyStoragePowerFeedback::{Missing, NotTriggered, Unknown};

    let Some(soc) = energy_connector(element, "Soc") else {
        return Unknown("Soc connector is absent or duplicated; no native connection is guessed.");
    };
    let Some(soc_connections) = native_connection_count(soc) else {
        return Unknown("Soc Nc/In storage is inconsistent or invalid.");
    };
    if soc_connections == 0 {
        return NotTriggered;
    }
    let Some(maximum) = energy_connector(element, "MaxSpwr") else {
        return Unknown("MaxSpwr connector is absent or duplicated.");
    };
    let Some(maximum_connections) = native_connection_count(maximum) else {
        return Unknown("MaxSpwr Nc/In storage is inconsistent or invalid.");
    };
    if maximum_connections > 0 {
        return NotTriggered;
    }
    let scalar = |key: &str| {
        scalars
            .iter()
            .find(|scalar| scalar.key == key)
            .and_then(|scalar| scalar.value.effective)
    };
    let Some(maximum_value) = scalar("MaxSpwr") else {
        return Unknown("MaxSpwr default is invalid or ambiguous.");
    };
    if maximum_value > 0.0 {
        return NotTriggered;
    }
    let Some(minimum) = energy_connector(element, "MinSoc") else {
        return Unknown("MinSoc connector is absent or duplicated.");
    };
    let Some(minimum_connections) = native_connection_count(minimum) else {
        return Unknown("MinSoc Nc/In storage is inconsistent or invalid.");
    };
    if minimum_connections > 0 {
        return Missing;
    }
    match scalar("MinSoc") {
        Some(0.0) => NotTriggered,
        Some(_) => Missing,
        None => Unknown("MinSoc default is invalid or ambiguous."),
    }
}

fn stored<T: Clone>(
    raw: Option<&String>,
    default: T,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    parse: impl FnOnce(&str) -> Option<T>,
) -> StoredValue<T> {
    let effective = match raw {
        Some(raw) => {
            let value = parse(raw);
            if value.is_none() {
                diagnostics.push(Diagnostic::new(
                    "invalid_storage",
                    path,
                    "Stored value could not be interpreted; omission and invalid storage are distinct.",
                ));
            }
            value
        }
        None => Some(default.clone()),
    };
    StoredValue {
        stored: raw.cloned(),
        effective,
        default: Some(default),
    }
}

fn boolean(raw: &str) -> Option<bool> {
    match raw {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn power(key: &'static str) -> NumericBinding {
    NumericBinding {
        key,
        unit: "kW",
        default: 0.0,
        min: Some(0.0),
        max: None,
    }
}

fn load_positions(element: &Element) -> Vec<usize> {
    element
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.as_element()
                .filter(|child| child.name == "Load")
                .map(|_| index)
        })
        .collect()
}

fn shape_diagnostics(element: &Element) -> Vec<Diagnostic> {
    let positions = load_positions(element);
    let mut diagnostics = Vec::new();
    if positions.len() != 12 {
        diagnostics.push(Diagnostic::new(
            "unsupported_load_shape",
            "EnergyManager2/Load",
            format!("Expected the verified 12 existing direct Load slots, found {}; add/delete/sparse configurations are unverified and will not be repaired.", positions.len()),
        ));
    }
    let mut ids = HashSet::new();
    let mut outputs = HashSet::new();
    for (priority, position) in positions.into_iter().enumerate() {
        let load = element.children[position].as_element().unwrap();
        let path = format!("EnergyManager2/Load[{}]", priority + 1);
        match load.attributes.get("Id").and_then(|id| native_identity(id)) {
            Some(id) if ids.insert(id) => {}
            Some(_) => diagnostics.push(Diagnostic::new("ambiguous_load_id", &format!("{path}/@Id"), "Duplicate persistent load UUID; select an independently verified configuration.")),
            None => diagnostics.push(Diagnostic::new("invalid_load_id", &format!("{path}/@Id"), "Each existing Load must have a hexadecimal native 8-4-4-16 or RFC 8-4-4-4-12 UUID Id; U is a boolean, not an identity.")),
        }
        match load.attributes.get("OIdx").and_then(|raw| raw.parse::<i32>().ok()) {
            Some(output) if (0..12).contains(&output) && outputs.insert(output) => {}
            _ => diagnostics.push(Diagnostic::new("ambiguous_output_association", &format!("{path}/@OIdx"), "Require unique explicit OIdx values 0..11. OIdx is output association, not priority; it will never be renumbered.")),
        }
        if load
            .attributes
            .get("U")
            .is_some_and(|raw| boolean(raw).is_none())
        {
            diagnostics.push(Diagnostic::new(
                "invalid_used_flag",
                &format!("{path}/@U"),
                "U must be a boolean connected/used flag, not an editable enable flag or UUID.",
            ));
        }
    }
    diagnostics
}

impl ConfigEditor {
    fn energy_path(&self, selector: &str) -> Result<Vec<usize>> {
        let path = self.require_one(selector)?;
        let element = self.get_element(&path);
        if element.name != "C"
            || element.attributes.get("Type").map(String::as_str) != Some("EnergyManager2")
        {
            return Err(Diagnostic::new(
                "wrong_block_type",
                selector,
                format!("Selected '{}' is type '{}', not EnergyManager2. Use `lox config controls FILE --type EnergyManager2` to find an energy manager.", element.attributes.get("Title").map_or("", String::as_str), element.attributes.get("Type").map_or("", String::as_str)),
            ).into());
        }
        Ok(path)
    }

    pub fn inspect_energy(&self, selector: &str) -> Result<EnergyInspection> {
        let path = self.energy_path(selector)?;
        let catalog = BuiltinIconCatalog::embedded()?;
        Ok(Self::inspect_energy_element(
            self.get_element(&path),
            &catalog,
        ))
    }

    fn inspect_energy_element(element: &Element, catalog: &BuiltinIconCatalog) -> EnergyInspection {
        let shape = shape_diagnostics(element);
        let load_edit_supported = shape.is_empty();
        let mut diagnostics = shape;
        let mut scalar_parameters = Vec::new();
        for parameter in [
            EnergyParameter::Offset,
            EnergyParameter::MinimumStateOfCharge,
            EnergyParameter::MaximumStoragePower,
        ] {
            let binding = parameter.binding();
            let path = format!("EnergyManager2/Co[K={}]/@Def", binding.key);
            let connectors: Vec<_> = element
                .children
                .iter()
                .filter_map(XMLNode::as_element)
                .filter(|co| {
                    co.name == "Co"
                        && co.attributes.get("K").map(String::as_str) == Some(binding.key)
                })
                .collect();
            let raw = connectors.first().and_then(|co| co.attributes.get("Def"));
            let mut value = stored(raw, binding.default, &path, &mut diagnostics, |raw| {
                binding.parse(raw, &path).ok()
            });
            if connectors.len() != 1 {
                value.effective = None;
                diagnostics.push(Diagnostic::new("ambiguous_scalar_binding", &path, format!("Expected one existing connector, found {}; no connector will be created or guessed.", connectors.len())));
            }
            scalar_parameters.push(EnergyScalar {
                key: binding.key.into(),
                direction: "parameter",
                unit: binding.unit,
                connector_present: !connectors.is_empty(),
                value,
            });
        }
        let storage_power_feedback = storage_power_feedback(element, &scalar_parameters);
        if storage_power_feedback == EnergyStoragePowerFeedback::Missing {
            diagnostics.push(Diagnostic::new(
                "missing_storage_power",
                "EnergyManager2/Co[K=MaxSpwr]/@Def",
                "Soc is connected and MinSoc is connected or nonzero, but MaxSpwr is unconnected and zero. Config 17.2.10.21 marks this incomplete. Set a positive MaxSpwr or connect its power input; zero remains valid otherwise. This feedback does not prohibit native simulation.",
            ));
        }
        let mut loads = Vec::new();
        for (priority, position) in load_positions(element).into_iter().enumerate() {
            let load = element.children[position].as_element().unwrap();
            let mut read_bool = |key: &str| {
                stored(
                    load.attributes.get(key),
                    false,
                    &format!("Load[{}]/@{key}", priority + 1),
                    &mut diagnostics,
                    boolean,
                )
            };
            let used = read_bool("U");
            let name_customized = read_bool("Ncu");
            let analog = read_bool("An");
            let mode_customized = read_bool("Anc");
            let mut read_int = |key: &str, default| {
                stored(
                    load.attributes.get(key),
                    default,
                    &format!("Load[{}]/@{key}", priority + 1),
                    &mut diagnostics,
                    |raw| raw.parse::<i32>().ok(),
                )
            };
            let mut output_index = read_int("OIdx", 0);
            output_index.default = None;
            if output_index.stored.is_none() {
                output_index.effective = None;
            }
            let preparation_seconds = read_int("RT", 60);
            let min_runtime_seconds = read_int("MinRt", 0);
            let daily_seconds = read_int("MDay", 0);
            let deadline = read_int("tpR", 0);
            let mut read_power = |key| {
                let path = format!("Load[{}]/@{key}", priority + 1);
                stored(
                    load.attributes.get(key),
                    0.0,
                    &path,
                    &mut diagnostics,
                    |raw| power(key).parse(raw, &path).ok(),
                )
            };
            let switch_on_power = read_power("SwPwr");
            let rated_power = read_power("MaxPwr");
            let unused_feedback = used
                .effective
                .zip(rated_power.effective)
                .map(|(used, rated)| !used || rated <= 0.0);
            let deadline_label = deadline.effective.and_then(|value| match value {
                0 => Some("sunset".into()),
                1 => Some("dusk".into()),
                2 => Some("sunrise".into()),
                3 => Some("dawn".into()),
                4..=27 => Some(format!("{:02}:00", value - 3)),
                _ => None,
            });
            loads.push(EnergyLoad {
                priority: priority + 1,
                id: load.attributes.get("Id").cloned(),
                output: load
                    .attributes
                    .get("OIdx")
                    .and_then(|raw| raw.parse::<i32>().ok())
                    .filter(|value| (0..12).contains(value))
                    .map(|index| format!("L{}", index + 1)),
                output_index,
                used,
                name: StoredValue {
                    stored: load.attributes.get("Na").cloned(),
                    effective: Some(load.attributes.get("Na").cloned().unwrap_or_default()),
                    default: Some(String::new()),
                },
                name_customized,
                analog,
                mode_customized,
                preparation_seconds,
                min_runtime_seconds,
                daily_seconds,
                deadline_gui_verified: matches!(deadline.effective, Some(0..=27)),
                deadline,
                deadline_label,
                switch_on_power,
                rated_power,
                unused_feedback,
                icon: load.attributes.get("ico").cloned(),
                icon_resolution: catalog.resolve(load.attributes.get("ico").map(String::as_str)),
                attributes: load
                    .attributes
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
                child_elements: load
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .map(|child| child.name.clone())
                    .collect(),
            });
        }
        EnergyInspection {
            block_type: "EnergyManager2",
            title: element.attributes.get("Title").cloned().unwrap_or_default(),
            uuid: element.attributes.get("U").cloned().unwrap_or_default(),
            verification: "verified_storage_subset_not_complete_gui_or_runtime_parity",
            scalar_parameters,
            loads,
            load_edit_supported,
            builtin_icon_catalog: catalog.catalog.clone(),
            storage_power_feedback,
            diagnostics,
            limitations: vec![
                "Bindings recovered for Config 17.2.10.21; CLI validation is not identical to GUI prefix filtering.",
                "No load add/delete, U/Id/OIdx editing, live server writes or runtime semantics.",
                "Icon selection is exact versioned built-in membership, not Qt SVG rendering parity. Explicit clear removes only this Load's ico; its GUI clear oracle is still missing.",
                "Existing unknown icon references are preserved, not validated or sanitized. Project/custom icon selection, creation, import, replacement, deletion and image decoding are unsupported.",
                "All 28 deadline choices have GUI-save observations; deadline/day-cycle execution remains unverified.",
                "Name/mode setters mark Ncu/Anc, including unchanged values; observed GUI transitions retain customization markers on revert.",
                "Minimum-runtime CLI decimals round half away from zero to whole minutes before checked signed 32-bit seconds; daily CLI minutes remain integers. Scientific input and GUI overflow are not emulated.",
                "MaxSpwr=0 runtime meaning remains unverified. Inspection is not an export/reapply format.",
                "Storage-power feedback is the recovered Config 17.2.10.21 condition only, not whole-block completeness. It uses connections/defaults, not current Soc or Spwr values; malformed Nc/In or absent/duplicate bindings are explicitly unknown. Fresh native countercases remain pending.",
            ],
        }
    }

    pub fn set_energy_parameter(
        &mut self,
        selector: &str,
        parameter: EnergyParameter,
        raw: &str,
    ) -> Result<()> {
        let path = self.energy_path(selector)?;
        let binding = parameter.binding();
        let binding_path = format!("EnergyManager2/Co[K={}]/@Def", binding.key);
        let value = binding.parse(raw, &binding_path)?;
        let mut staged = self.get_element(&path).clone();
        let positions: Vec<_> = staged
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.as_element()
                    .filter(|co| {
                        co.name == "Co"
                            && co.attributes.get("K").map(String::as_str) == Some(binding.key)
                    })
                    .map(|_| index)
            })
            .collect();
        if positions.len() != 1 {
            return Err(Diagnostic::new("ambiguous_scalar_binding", &binding_path, format!("Expected one existing connector, found {}; inspect the block, do not guess or create storage.", positions.len())).into());
        }
        let connector = staged.children[positions[0]].as_mut_element().unwrap();
        set_default_zero(connector, "Def", value);
        *self.get_element_mut(&path) = staged;
        Ok(())
    }

    fn editable_load(&self, selector: &str, id: &str) -> Result<(Vec<usize>, usize)> {
        let path = self.energy_path(selector)?;
        let element = self.get_element(&path);
        if let Some(diagnostic) = shape_diagnostics(element).into_iter().next() {
            return Err(diagnostic.into());
        }
        let uuid = native_identity(id.strip_prefix("uuid:").unwrap_or(id))
            .ok_or_else(|| Diagnostic::new("invalid_load_selector", id, "Select a native 8-4-4-16 or RFC 8-4-4-4-12 Load Id from `config energy inspect`, not a priority, name, U flag or malformed dash spelling."))?;
        let position = load_positions(element).into_iter().find(|&index| {
            element.children[index].as_element().unwrap().attributes.get("Id").and_then(|id| native_identity(id)) == Some(uuid)
        }).ok_or_else(|| Diagnostic::new("load_not_found", id, "No existing Load has that Id; inspect the block. Adding slots is unverified and unsupported."))?;
        Ok((path, position))
    }

    pub fn configure_energy_load(
        &mut self,
        selector: &str,
        id: &str,
        patch: &EnergyLoadPatch,
    ) -> Result<()> {
        let (path, position) = self.editable_load(selector, id)?;
        if patch.name.is_none()
            && patch.mode.is_none()
            && patch.preparation_seconds.is_none()
            && patch.min_runtime_minutes.is_none()
            && patch.daily_minutes.is_none()
            && patch.deadline.is_none()
            && patch.rated_power.is_none()
            && patch.switch_on_power.is_none()
            && matches!(patch.icon, EnergyIconChange::Preserve)
        {
            return Err(Diagnostic::new(
                "empty_patch",
                id,
                "Provide at least one supported load field; inspect is read-only.",
            )
            .into());
        }
        let mut staged = self.get_element(&path).clone();
        let load = staged.children[position].as_mut_element().unwrap();
        if let Some(name) = &patch.name {
            if name.chars().any(|character| {
                !matches!(character, '\t' | '\n' | '\r' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')
            }) {
                return Err(Diagnostic::new(
                    "invalid_xml_text",
                    "Load/@Na",
                    "Name contains a character that cannot be stored in XML 1.0.",
                ).into());
            }
            load.attributes.insert("Na".into(), name.clone());
            load.attributes.insert("Ncu".into(), "true".into());
        }
        if let Some(mode) = patch.mode {
            match mode {
                EnergyLoadMode::Analog => {
                    load.attributes.insert("An".into(), "true".into());
                }
                EnergyLoadMode::Digital => {
                    load.attributes.shift_remove("An");
                }
            }
            load.attributes.insert("Anc".into(), "true".into());
        }
        if let Some(seconds) = patch.preparation_seconds {
            if !(30..=86400).contains(&seconds) {
                return Err(Diagnostic::new("out_of_range", "Load/@RT", "Preparation must be 30..86400 integer seconds; this CLI rejects rather than emulating GUI filtering/clamping.").into());
            }
            load.attributes.insert("RT".into(), seconds.to_string());
        }
        if let Some(minutes) = patch.min_runtime_minutes {
            set_integer_zero(load, "MinRt", minutes_to_seconds(minutes, "Load/@MinRt")?);
        }
        if let Some(minutes) = patch.daily_minutes {
            if !(0..=1440).contains(&minutes) {
                return Err(Diagnostic::new(
                    "out_of_range",
                    "Load/@MDay",
                    "Daily runtime must be 0..1440 integer minutes/day.",
                )
                .into());
            }
            set_integer_zero(load, "MDay", minutes_to_seconds(minutes, "Load/@MDay")?);
        }
        if let Some(deadline) = patch.deadline {
            set_integer_zero(load, "tpR", deadline.storage()?);
        }
        // Apply rated power first, then explicit switch-on power against that rated value.
        if patch.rated_power.is_some() || patch.switch_on_power.is_some() {
            let mut rated = power("MaxPwr").parse(
                load.attributes.get("MaxPwr").map_or("0", String::as_str),
                "Load/@MaxPwr",
            )?;
            let switch = power("SwPwr").parse(
                load.attributes.get("SwPwr").map_or("0", String::as_str),
                "Load/@SwPwr",
            )?;
            if let Some(raw) = &patch.rated_power {
                rated = power("MaxPwr").parse(raw, "Load/@MaxPwr")?;
                set_default_zero(load, "MaxPwr", rated);
                if switch == 0.0 {
                    set_default_zero(load, "SwPwr", rated);
                }
            }
            if let Some(raw) = &patch.switch_on_power {
                let value = power("SwPwr").parse(raw, "Load/@SwPwr")?;
                set_default_zero(load, "SwPwr", if value == 0.0 { rated } else { value });
            }
        }
        match &patch.icon {
            EnergyIconChange::Preserve => {}
            EnergyIconChange::Clear => {
                load.attributes.shift_remove("ico");
            }
            EnergyIconChange::SetBuiltin(selection) => {
                let catalog = BuiltinIconCatalog::embedded()?;
                let icon = catalog.select(selection)?;
                load.attributes.insert("ico".into(), icon.native_id.clone());
            }
        }
        *self.get_element_mut(&path) = staged;
        Ok(())
    }

    pub fn move_energy_load(&mut self, selector: &str, id: &str, priority: usize) -> Result<()> {
        let (path, position) = self.editable_load(selector, id)?;
        if !(1..=12).contains(&priority) {
            return Err(Diagnostic::new(
                "out_of_range",
                "Load/priority",
                "Priority must be 1..12; output association OIdx is never renumbered.",
            )
            .into());
        }
        let mut staged = self.get_element(&path).clone();
        let positions = load_positions(&staged);
        let current = positions
            .iter()
            .position(|&index| index == position)
            .unwrap();
        let mut records: Vec<_> = positions
            .iter()
            .map(|&index| staged.children[index].clone())
            .collect();
        let moved = records.remove(current);
        records.insert(priority - 1, moved);
        for (index, record) in positions.into_iter().zip(records) {
            staged.children[index] = record;
        }
        *self.get_element_mut(&path) = staged;
        Ok(())
    }
}

fn set_default_zero(element: &mut Element, key: &str, value: f64) {
    if value == 0.0 {
        element.attributes.shift_remove(key);
    } else {
        element.attributes.insert(key.into(), value.to_string());
    }
}

fn set_integer_zero(element: &mut Element, key: &str, value: i32) {
    if value == 0 {
        element.attributes.shift_remove(key);
    } else {
        element.attributes.insert(key.into(), value.to_string());
    }
}
