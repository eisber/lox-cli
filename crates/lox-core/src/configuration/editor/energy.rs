use super::{EditError, legacy::State};
use std::{collections::BTreeMap, error::Error, fmt};
use xmltree::XMLNode;

/// Stored observations used by the EnergyManager2 inspection.
#[derive(Debug, PartialEq, Eq)]
pub struct EnergyInspection {
    pub title: Option<String>,
    pub uuid: Option<String>,
    /// Direct `Co` records keyed `O`, `MinSoc`, `MaxSpwr` or `Soc`, in document order.
    /// Duplicate connectors are retained.
    pub connectors: Vec<StoredEnergyConnector>,
    /// Direct `Load` records in document order, independent of `Id` and `OIdx`.
    pub loads: Vec<StoredEnergyLoad>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredEnergyConnector {
    pub key: String,
    /// Stored `Def`, without interpreting it or supplying a default.
    pub stored_default: Option<String>,
    /// Stored `Nc`, without validating the connection count.
    pub stored_connection_count: Option<String>,
    /// One stored `Input` per direct `In`, retaining absence, emptiness and order.
    pub input_references: Vec<Option<String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StoredEnergyLoad {
    /// Loader-decoded attributes, including unrecognized ones.
    pub attributes: BTreeMap<String, String>,
    /// Names of direct element children, not their contents.
    pub child_elements: Vec<String>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum EnergyInspectionError {
    Edit(EditError),
    NotEnergyManager {
        selector: String,
        title: String,
        block_type: String,
    },
}

impl fmt::Display for EnergyInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::NotEnergyManager {
                selector,
                block_type,
                ..
            } => write!(
                f,
                "Energy inspection requires EnergyManager2; '{selector}' has type '{block_type}'"
            ),
        }
    }
}

impl Error for EnergyInspectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            Self::NotEnergyManager { .. } => None,
        }
    }
}

/// Inspect the stored XML projection for one legacy-selected `C Type="EnergyManager2"`.
///
/// The legacy digit-name/quoted-text loader limitation (lox-n37w) is unchanged.
/// Values are from that loader, not raw XML bytes. Malformed or incomplete
/// records are observations, not refusals. This API does not calculate effective
/// defaults, resolve icons or connections, infer runtime state, or modify files.
pub fn inspect_energy(
    bytes: &[u8],
    selector: &str,
) -> Result<EnergyInspection, EnergyInspectionError> {
    State::load(bytes)
        .map_err(EnergyInspectionError::Edit)?
        .inspect_energy_selected(selector)
}

impl State {
    pub fn select_energy_path(&self, selector: &str) -> Result<Vec<usize>, EnergyInspectionError> {
        let path = self
            .require_one(selector)
            .map_err(|error| EnergyInspectionError::Edit(EditError::Selection(error)))?;
        let block = self.get_element(&path);
        if block.name != "C"
            || block.attributes.get("Type").map(String::as_str) != Some("EnergyManager2")
        {
            return Err(EnergyInspectionError::NotEnergyManager {
                selector: selector.into(),
                title: block.attributes.get("Title").cloned().unwrap_or_default(),
                block_type: block.attributes.get("Type").cloned().unwrap_or_default(),
            });
        }
        Ok(path)
    }

    pub fn inspect_energy_selected(
        &self,
        selector: &str,
    ) -> Result<EnergyInspection, EnergyInspectionError> {
        let path = self.select_energy_path(selector)?;
        let block = self.get_element(&path);
        let mut connectors = Vec::new();
        let mut loads = Vec::new();
        for child in block.children.iter().filter_map(XMLNode::as_element) {
            match child.name.as_str() {
                "Co" => {
                    if let Some(key) = child.attributes.get("K")
                        && matches!(key.as_str(), "O" | "MinSoc" | "MaxSpwr" | "Soc")
                    {
                        connectors.push(StoredEnergyConnector {
                            key: key.clone(),
                            stored_default: child.attributes.get("Def").cloned(),
                            stored_connection_count: child.attributes.get("Nc").cloned(),
                            input_references: child
                                .children
                                .iter()
                                .filter_map(XMLNode::as_element)
                                .filter(|input| input.name == "In")
                                .map(|input| input.attributes.get("Input").cloned())
                                .collect(),
                        });
                    }
                }
                "Load" => loads.push(StoredEnergyLoad {
                    attributes: child
                        .attributes
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect(),
                    child_elements: child
                        .children
                        .iter()
                        .filter_map(XMLNode::as_element)
                        .map(|element| element.name.clone())
                        .collect(),
                }),
                _ => {}
            }
        }
        Ok(EnergyInspection {
            title: block.attributes.get("Title").cloned(),
            uuid: block.attributes.get("U").cloned(),
            connectors,
            loads,
        })
    }
}
