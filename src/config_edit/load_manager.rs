use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;

#[derive(Debug, Clone, Copy)]
pub enum LoadMoveDirection {
    Up,
    Down,
}

#[derive(Debug, Serialize)]
pub struct LoadManagerLoad {
    pub priority: usize,
    pub io_index: u8,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct LoadManagerInspection {
    pub uuid: String,
    pub title: String,
    pub loads: Vec<LoadManagerLoad>,
}

impl ConfigEditor {
    fn load_manager_path(&self, selector: &str) -> Result<Vec<usize>> {
        let path = self.require_one(selector)?;
        let block = self.get_element(&path);
        if block.name != "C" || block.attributes.get("Type").map(String::as_str) != Some("LoadShed")
        {
            bail!(
                "'{selector}' is not a LoadShed (Load Manager). Use `lox config controls FILE --type LoadShed` to find one."
            );
        }
        Ok(path)
    }

    pub fn inspect_load_manager(&self, selector: &str) -> Result<LoadManagerInspection> {
        let path = self.load_manager_path(selector)?;
        let block = self.get_element(&path);
        let positions = load_positions(block)?;
        Ok(LoadManagerInspection {
            uuid: block.attributes.get("U").cloned().unwrap_or_default(),
            title: block.attributes.get("Title").cloned().unwrap_or_default(),
            loads: positions
                .iter()
                .enumerate()
                .map(|(index, position)| {
                    let load = block.children[*position].as_element().unwrap();
                    LoadManagerLoad {
                        priority: index + 1,
                        io_index: load.attributes["ioIdx"].parse().unwrap(),
                        attributes: load
                            .attributes
                            .iter()
                            .map(|(key, value)| (key.clone(), value.clone()))
                            .collect(),
                    }
                })
                .collect(),
        })
    }

    /// Move the complete record, not its connector identity or a fabricated priority attribute.
    pub fn move_load_manager_load(
        &mut self,
        selector: &str,
        io_index: u8,
        direction: LoadMoveDirection,
    ) -> Result<()> {
        if io_index > 11 {
            bail!("Load Manager io-index must be 0..11, not a 1-based priority.");
        }
        let path = self.load_manager_path(selector)?;
        let block = self.get_element(&path);
        let positions = load_positions(block)?;
        let index = positions
            .iter()
            .position(|position| {
                block.children[*position].as_element().unwrap().attributes["ioIdx"]
                    .parse::<u8>()
                    .unwrap()
                    == io_index
            })
            .unwrap();
        let next = match direction {
            LoadMoveDirection::Up if index > 0 => index - 1,
            LoadMoveDirection::Down if index + 1 < positions.len() => index + 1,
            _ => bail!(
                "Load io-index {io_index} is already at the {} priority boundary; nothing was changed.",
                if index == 0 { "first" } else { "last" }
            ),
        };
        // Native dialog accept copies the reordered row fields by position.
        // XML serializes that order; ioIdx travels with the name/power/timing settings.
        self.get_element_mut(&path)
            .children
            .swap(positions[index], positions[next]);
        Ok(())
    }
}

fn load_positions(block: &Element) -> Result<Vec<usize>> {
    let positions: Vec<_> = block
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.as_element()
                .filter(|element| element.name == "Load")
                .map(|_| index)
        })
        .collect();
    if positions.len() != 12 {
        bail!(
            "Unsupported Load Manager shape: expected exactly 12 direct Load records, found {}. Sparse/extended lists are not supported; no defaults were fabricated.",
            positions.len()
        );
    }
    let mut seen = [false; 12];
    for position in &positions {
        let load = block.children[*position].as_element().unwrap();
        let io_index = load
            .attributes
            .get("ioIdx")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|index| *index < seen.len())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported Load Manager shape: every Load needs an explicit ioIdx in 0..11; omitted identities are not inferred."
                )
            })?;
        if seen[io_index] {
            bail!(
                "Unsupported Load Manager shape: duplicate ioIdx {io_index}; identities must be unique."
            );
        }
        seen[io_index] = true;
    }
    Ok(positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ConfigEditor {
        let loads: String = (0..12)
            .map(|i| {
                format!(
                    "<Load ioIdx=\"{i}\" nm=\"Load {i}\" pwr=\"{i}.5\" maxOff=\"123\" minOn=\"45\" Unknown=\"keep-{i}\"><Extra U=\"extra-{i}\"/></Load>{}",
                    if i == 3 { "<Between U=\"retain-position\"/>" } else { "" }
                )
            })
            .collect();
        ConfigEditor::load(
            format!(
                "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<Loxone><C Type=\"LoadShed\" U=\"manager\" Title=\"Loads\" Unknown=\"keep\"><Co K=\"AQl1\" U=\"wire\"/><Before/>{loads}<After/></C></Loxone>\r\n"
            )
            .as_bytes(),
        )
        .unwrap()
    }

    #[test]
    fn load_manager_move_preserves_complete_records_and_unrelated_storage() {
        let mut editor = fixture();
        let original = editor.to_bytes().unwrap();
        let path = editor.require_one("uuid:manager").unwrap();
        let before = editor.get_element(&path).clone();
        editor
            .move_load_manager_load("Loads", 4, LoadMoveDirection::Up)
            .unwrap();
        let mut expected = before;
        expected.children.swap(7, 5);
        assert_eq!(*editor.get_element(&path), expected);
        let bytes = editor.to_bytes().unwrap();
        assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(bytes.windows(2).any(|window| window == b"\r\n"));
        let reloaded = ConfigEditor::load(&bytes).unwrap();
        let loads = reloaded.inspect_load_manager("Loads").unwrap().loads;
        assert_eq!(loads[3].io_index, 4);
        assert_eq!(loads[4].io_index, 3);
        assert_eq!(loads[3].attributes["Unknown"], "keep-4");
        editor
            .move_load_manager_load("Loads", 4, LoadMoveDirection::Down)
            .unwrap();
        assert_eq!(editor.to_bytes().unwrap(), original);
    }

    #[test]
    fn load_manager_move_rejects_boundaries_and_malformed_shapes_atomically() {
        for (io_index, direction) in [
            (0, LoadMoveDirection::Up),
            (11, LoadMoveDirection::Down),
            (12, LoadMoveDirection::Up),
        ] {
            let mut editor = fixture();
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .move_load_manager_load("Loads", io_index, direction)
                    .is_err()
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
        for mutation in ["duplicate", "missing", "extra", "sparse", "out-of-range"] {
            let mut editor = fixture();
            let path = editor.require_one("Loads").unwrap();
            let block = editor.get_element_mut(&path);
            match mutation {
                "duplicate" => {
                    block.children[3]
                        .as_mut_element()
                        .unwrap()
                        .attributes
                        .insert("ioIdx".into(), "0".into());
                }
                "missing" => {
                    block.children[3]
                        .as_mut_element()
                        .unwrap()
                        .attributes
                        .shift_remove("ioIdx");
                }
                "extra" => block.children.push(block.children[2].clone()),
                "sparse" => {
                    block.children.remove(2);
                }
                _ => {
                    block.children[3]
                        .as_mut_element()
                        .unwrap()
                        .attributes
                        .insert("ioIdx".into(), "12".into());
                }
            }
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .move_load_manager_load("Loads", 4, LoadMoveDirection::Up)
                    .is_err(),
                "{mutation}"
            );
            assert_eq!(editor.to_bytes().unwrap(), before, "{mutation}");
        }
    }
}
