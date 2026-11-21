use std::collections::BTreeMap;

use anyhow::{Result, bail};
use lox_core::configuration::editor::{
    MatchedStatusMonitor, StatusMonitorInspectionError, status_monitor_identity_key as identity_key,
};
use serde::Serialize;

use super::{ConfigEditor, editor_error};

#[derive(Debug, Clone, Copy)]
pub enum StatusPriorityDirection {
    Up,
    Down,
}

#[derive(Debug, Serialize)]
pub struct StatusMonitorState {
    pub priority: usize,
    pub id: String,
    pub output_index: u8,
    pub configured: bool,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct StatusMonitorInspection {
    pub uuid: String,
    pub title: String,
    pub states: Vec<StatusMonitorState>,
}

#[derive(Debug, Serialize)]
pub struct StatusPriorityMove {
    pub id: String,
    pub from_priority: usize,
    pub to_priority: usize,
    pub changed: bool,
}

impl ConfigEditor {
    pub fn inspect_status_monitor(&self, selector: &str) -> Result<StatusMonitorInspection> {
        let inspection = self
            .kernel
            .status_monitor_inspection_selected(selector)
            .map_err(status_monitor_error)?;
        Ok(StatusMonitorInspection {
            uuid: inspection.uuid,
            title: inspection.title,
            states: inspection
                .states
                .into_iter()
                .map(|state| StatusMonitorState {
                    priority: state.priority,
                    id: state.id,
                    output_index: state.output_index,
                    configured: state.configured,
                    attributes: state.attributes,
                })
                .collect(),
        })
    }

    pub fn move_status_monitor_state(
        &mut self,
        selector: &str,
        id: &str,
        direction: StatusPriorityDirection,
    ) -> Result<StatusPriorityMove> {
        let key = identity_key(id).ok_or_else(|| {
            anyhow::anyhow!(
                "Status Monitor state Id must be a nonzero native UUID (8-4-4-16 hexadecimal digits). Use `config status-monitor inspect` to find it; nothing was changed."
            )
        })?;
        let MatchedStatusMonitor { path, states } = self
            .kernel
            .match_status_monitor(selector)
            .map_err(status_monitor_error)?;
        let index = states
            .iter()
            .position(|(_, state)| identity_key(&state.id).as_ref() == Some(&key))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No Status Monitor state with Id '{id}'. Select an existing Id, not OIdx, name or priority; nothing was changed."
                )
            })?;
        let selected = &states[index].1;
        if selected.output_index == 10 {
            bail!("The Status Monitor Rest state (OIdx=10) cannot move; nothing was changed.");
        }
        if !selected.configured {
            bail!(
                "Select a named Status Monitor state with an explicit finite Val; nothing was changed."
            );
        }
        // Native priority is the State ordinal, not its persistent OIdx.
        // The last (Rest) record never participates in an adjacent swap.
        let destination = match direction {
            StatusPriorityDirection::Up => index.saturating_sub(1),
            StatusPriorityDirection::Down => (index + 1).min(9),
        };
        let result = StatusPriorityMove {
            id: selected.id.clone(),
            from_priority: index + 1,
            to_priority: destination + 1,
            changed: index != destination,
        };
        if result.changed {
            let from = states[index].0;
            let to = states[destination].0;
            self.get_element_mut(&path).children.swap(from, to);
        }
        Ok(result)
    }
}

fn status_monitor_error(error: StatusMonitorInspectionError) -> anyhow::Error {
    match error {
        StatusMonitorInspectionError::Edit(error) => editor_error(error),
        error => anyhow::anyhow!("{error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(index: u8) -> String {
        format!("abcdef01-0002-0003-{index:016x}")
    }

    fn fixture() -> String {
        let states: String = [5, 3, 7, 0, 9, 1, 6, 2, 8, 4, 10]
            .into_iter()
            .map(|index| {
                let value = if index == 10 {
                    String::new()
                } else {
                    format!(" Val=\"{index}\"")
                };
                format!(
                    "<State Id=\"{}\" OIdx=\"{index}\" Na=\"State {index}\"{value} Cl=\"1,2,3\" Future=\"preserve-{index}\"><Unknown keep=\"{index}\"/></State>",
                    id(index)
                )
            })
            .collect();
        format!(
            "<Loxone><C Type=\"StatusMonitor\" U=\"monitor\" Title=\"Status\"><Co K=\"AQ1\" U=\"wire\"/>{states}<Unknown keep=\"block\"/></C><C Type=\"StateV\" U=\"consumer\"><Co K=\"I\"><In Input=\"wire\"/></Co></C></Loxone>"
        )
    }

    #[test]
    fn status_monitor_moves_complete_identity_not_output_number_and_roundtrips() {
        let mut editor = ConfigEditor::load(fixture().as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let before_view =
            serde_json::to_value(editor.inspect_status_monitor("Status").unwrap()).unwrap();
        let movement = editor
            .move_status_monitor_state("Status", &id(3).to_uppercase(), StatusPriorityDirection::Up)
            .unwrap();
        assert_eq!((movement.from_priority, movement.to_priority), (2, 1));
        assert!(movement.changed);
        let view = editor.inspect_status_monitor("Status").unwrap();
        assert_eq!(view.states[0].output_index, 3);
        assert_eq!(view.states[1].output_index, 5);
        assert_eq!(view.states[10].output_index, 10);
        assert_eq!(
            serde_json::to_value(&view.states[0].attributes).unwrap(),
            before_view["states"][1]["attributes"]
        );
        let xml = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(xml.contains("keep=\"3\""));
        assert!(xml.contains("Input=\"wire\""));
        assert!(xml.contains("keep=\"block\""));
        let reparsed = ConfigEditor::load(xml.as_bytes()).unwrap();
        assert_eq!(
            reparsed.inspect_status_monitor("Status").unwrap().states[0].id,
            id(3)
        );
        editor
            .move_status_monitor_state("Status", &id(3), StatusPriorityDirection::Down)
            .unwrap();
        assert_eq!(editor.to_bytes().unwrap(), before);
    }

    #[test]
    fn status_monitor_pins_rest_and_movable_boundaries_are_noops() {
        let mut editor = ConfigEditor::load(fixture().as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        for (index, direction) in [
            (5, StatusPriorityDirection::Up),
            (4, StatusPriorityDirection::Down),
        ] {
            assert!(
                !editor
                    .move_status_monitor_state("Status", &id(index), direction)
                    .unwrap()
                    .changed
            );
        }
        assert!(
            editor
                .move_status_monitor_state("Status", &id(10), StatusPriorityDirection::Up)
                .is_err()
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }

    #[test]
    fn status_monitor_rejects_ambiguous_malformed_and_inherited_storage_atomically() {
        let source = fixture();
        for invalid in [
            source.replacen(&id(3), &id(5), 1),
            source.replacen("OIdx=\"3\"", "OIdx=\"5\"", 1),
            source.replacen(" OIdx=\"0\"", "", 1),
            source.replacen("OIdx=\"10\"", "OIdx=\"11\"", 1),
            source.replacen("OIdx=\"5\"", "OIdx=\"10\"", 1),
            source.replacen("Val=\"3\"", "Val=\"5\"", 1),
            source.replacen("Val=\"3\"", "Val=\"-0\"", 1),
            source.replacen("Val=\"3\"", "Val=\"NaN\"", 1),
            source.replacen("Val=\"3\"", "Val=\"inf\"", 1),
            source.replacen("Val=\"3\"", &format!("Val=\"{}\"", f64::MAX), 1),
            source.replacen("OIdx=\"10\"", "OIdx=\"10\" Val=\"99\"", 1),
            source.replacen("Type=\"StateV\"", "Type=\"StateV\" iObjS=\"monitor\"", 1),
            source.replacen(
                "Type=\"StatusMonitor\"",
                "Type=\"StatusMonitor\" iObjS=\"\"",
                1,
            ),
            source.replacen(&id(3), "00000000-0000-0000-0000000000000000", 1),
            source.replacen(&id(3), "malformed-id", 1),
            source
                .replacen("<State ", "<Other ", 1)
                .replacen("</State>", "</Other>", 1),
        ] {
            let mut editor = ConfigEditor::load(invalid.as_bytes()).unwrap();
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .move_status_monitor_state("Status", &id(3), StatusPriorityDirection::Up)
                    .is_err(),
                "accepted invalid storage: {invalid}"
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
    }

    #[test]
    fn status_monitor_moves_configured_state_past_unconfigured_neighbor_without_defaults() {
        let source = fixture().replacen(" Val=\"5\"", "", 1);
        let mut editor = ConfigEditor::load(source.as_bytes()).unwrap();
        assert!(
            editor
                .move_status_monitor_state("Status", &id(5), StatusPriorityDirection::Down)
                .is_err()
        );
        editor
            .move_status_monitor_state("Status", &id(3), StatusPriorityDirection::Up)
            .unwrap();
        let view = editor.inspect_status_monitor("Status").unwrap();
        assert_eq!(view.states[0].id, id(3));
        assert!(!view.states[1].configured);
        assert!(!view.states[1].attributes.contains_key("Val"));
    }

    #[test]
    fn status_monitor_cached_adapter_preserves_move_refusal_order_and_text() {
        let xml = fixture().replace("Type=\"StateV\"", "Type=\"StateV\" iObjS=\"\"");
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let invalid_id = editor
            .move_status_monitor_state("Status", "bad-id", StatusPriorityDirection::Up)
            .unwrap_err();
        assert_eq!(
            invalid_id.to_string(),
            "Status Monitor state Id must be a nonzero native UUID (8-4-4-16 hexadecimal digits). Use `config status-monitor inspect` to find it; nothing was changed."
        );
        let inspect_error = editor.inspect_status_monitor("Status").unwrap_err();
        let move_error = editor
            .move_status_monitor_state("Status", &id(3), StatusPriorityDirection::Up)
            .unwrap_err();
        assert_eq!(
            inspect_error.to_string(),
            "Status Monitor priority editing supports standalone configurations only: an iObjS inheritance attribute is present in this file. Native inherited-owner redirection and propagation are not implemented; nothing was changed."
        );
        assert_eq!(inspect_error.to_string(), move_error.to_string());
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}
