//! Read-only connector observations, not live physical state or wiring validation.
//!
//! Uses the shared legacy loader and selector, including UUID mirror de-duplication
//! and the quoted-text/digit-name limitation tracked as lox-n37w.

use super::editor::{EditError, legacy::State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectorDirection {
    Input,
    Output,
    Parameter,
}

/// One direct `Co` child, in source order. Duplicate names are retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorWiring {
    pub connector: String,
    pub direction: ConnectorDirection,
    /// Raw `U`, or empty when absent; intentionally not validated as a UUID.
    pub target_uuid: String,
    /// A direct `In` child, or a nonempty `U` other than the exact legacy sentinel.
    pub connected: bool,
}

/// Observe one legacy-selected element without modifying the borrowed bytes.
pub fn connector_wiring(bytes: &[u8], selector: &str) -> Result<Vec<ConnectorWiring>, EditError> {
    State::load(bytes)?.connector_wiring(selector)
}

impl State {
    /// Shared read-only producer for the byte API and the unstable CLI editor bridge.
    pub fn connector_wiring(&self, selector: &str) -> Result<Vec<ConnectorWiring>, EditError> {
        let path = self.require_one(selector)?;
        let elem = self.get_element(&path);
        let mut wires = Vec::new();
        for child in &elem.children {
            if let Some(co) = child.as_element()
                && co.name == "Co"
            {
                let name = co.attributes.get("K").cloned().unwrap_or_default();
                let target_uuid = co.attributes.get("U").cloned().unwrap_or_default();
                let direction =
                    if name.starts_with('I') || name.starts_with("AI") || name == "Input" {
                        ConnectorDirection::Input
                    } else if name.starts_with('Q')
                        || name.starts_with("AQ")
                        || name.starts_with("Output")
                    {
                        ConnectorDirection::Output
                    } else {
                        ConnectorDirection::Parameter
                    };
                let has_in_children = co
                    .children
                    .iter()
                    .any(|c| c.as_element().map(|e| e.name == "In").unwrap_or(false));
                let connected = has_in_children
                    || (!target_uuid.is_empty()
                        && target_uuid != "00000000-0000-0000-0000000000000000");
                wires.push(ConnectorWiring {
                    connector: name,
                    direction,
                    target_uuid,
                    connected,
                });
            }
        }
        Ok(wires)
    }
}
