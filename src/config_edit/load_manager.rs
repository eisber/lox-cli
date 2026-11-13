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
                        io_index: resolve_io_index(load).unwrap(),
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

    /// Edit the Configure loads Name cell, addressed by its stable connector identity.
    pub fn set_load_manager_load_name(
        &mut self,
        selector: &str,
        io_index: u8,
        name: &str,
    ) -> Result<bool> {
        if io_index > 11 {
            bail!("Load Manager io-index must be 0..11, not a 1-based priority.");
        }
        // Single-cell native paste preserves spaces/UTF-8; tab/newline enter a
        // different, multi-cell path. The length bound is a conservative CLI subset.
        if name.encode_utf16().count() > 32767
            || name.chars().any(|c| {
                c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}')
            })
        {
            bail!(
                "Unsupported Load Manager name: use single-line Unicode without control characters, at most 32767 UTF-16 code units. This is a conservative CLI subset, not a recovered native maximum; nothing was changed."
            );
        }
        let path = self.load_manager_path(selector)?;
        let block = self.get_element(&path);
        let positions = load_positions(block)?;
        let position = positions
            .into_iter()
            .find(|position| {
                resolve_io_index(block.children[*position].as_element().unwrap()).unwrap()
                    == io_index
            })
            .unwrap();
        let load = block.children[position].as_element().unwrap();
        if load.attributes.get("nm").map_or("", String::as_str) == name {
            return Ok(false);
        }
        let load = self.get_element_mut(&path).children[position]
            .as_mut_element()
            .unwrap();
        // The native string serializer omits an empty nm. A semantic no-op above
        // deliberately preserves an already absent or explicitly empty attribute.
        if name.is_empty() {
            load.attributes.shift_remove("nm");
        } else {
            load.attributes.insert("nm".into(), name.into());
        }
        Ok(true)
    }

    /// Move the complete record in Configure loads/XML order, keeping its connector identity.
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
                resolve_io_index(block.children[*position].as_element().unwrap()).unwrap()
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
        let io_index = usize::from(resolve_io_index(load)?);
        if seen[io_index] {
            bail!(
                "Unsupported Load Manager shape: duplicate ioIdx {io_index}; identities must be unique."
            );
        }
        seen[io_index] = true;
    }
    Ok(positions)
}

fn resolve_io_index(load: &Element) -> Result<u8> {
    // The native serializer omits zero; an absent attribute reads back as zero.
    let Some(raw) = load.attributes.get("ioIdx") else {
        return Ok(0);
    };
    if !raw.is_empty()
        && raw.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(index @ 0..=11) = raw.parse::<u8>()
    {
        return Ok(index);
    }
    bail!(
        "Unsupported Load Manager shape: ioIdx must be absent (native zero) or decimal digits in 0..11. Explicit empty, signed, whitespace and nonnumeric identities are not supported."
    )
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

    fn native_fixture() -> ConfigEditor {
        let xml = String::from_utf8(fixture().to_bytes().unwrap())
            .unwrap()
            .replacen(" ioIdx=\"0\"", "", 1);
        ConfigEditor::load(xml.as_bytes()).unwrap()
    }

    #[test]
    fn load_manager_name_uses_stable_identity_and_preserves_every_other_field() {
        let mut editor = native_fixture();
        editor
            .move_load_manager_load("Loads", 0, LoadMoveDirection::Down)
            .unwrap();
        let path = editor.require_one("Loads").unwrap();
        let mut expected = editor.get_element(&path).clone();
        let name = "  Wärmepumpe 東京 🔌 & \"Haus\"  ";
        expected.children[3]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("nm".into(), name.into());
        assert!(editor.set_load_manager_load_name("Loads", 0, name).unwrap());
        assert_eq!(*editor.get_element(&path), expected);
        let bytes = editor.to_bytes().unwrap();
        assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(bytes.windows(2).any(|window| window == b"\r\n"));
        let reloaded = ConfigEditor::load(&bytes).unwrap();
        let loads = reloaded.inspect_load_manager("Loads").unwrap().loads;
        assert_eq!(loads[1].attributes["nm"], name);
        assert!(!loads[1].attributes.contains_key("ioIdx"));
        assert!(!editor.set_load_manager_load_name("Loads", 0, name).unwrap());
        assert_eq!(editor.to_bytes().unwrap(), bytes);
        // Native validation does not require names to be unique.
        assert!(editor.set_load_manager_load_name("Loads", 1, name).unwrap());
        assert!(editor.set_load_manager_load_name("Loads", 1, "  ").unwrap());
        assert_eq!(
            editor.inspect_load_manager("Loads").unwrap().loads[0].attributes["nm"],
            "  "
        );
    }

    #[test]
    fn load_manager_name_clear_omits_nm_but_noop_preserves_explicit_empty() {
        let mut editor = native_fixture();
        assert!(editor.set_load_manager_load_name("Loads", 0, "").unwrap());
        assert!(
            !editor.inspect_load_manager("Loads").unwrap().loads[0]
                .attributes
                .contains_key("nm")
        );
        let before = editor.to_bytes().unwrap();
        assert!(!editor.set_load_manager_load_name("Loads", 0, "").unwrap());
        assert_eq!(editor.to_bytes().unwrap(), before);
        let path = editor.require_one("Loads").unwrap();
        editor.get_element_mut(&path).children[2]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("nm".into(), "".into());
        let before = editor.to_bytes().unwrap();
        assert!(!editor.set_load_manager_load_name("Loads", 0, "").unwrap());
        assert_eq!(editor.to_bytes().unwrap(), before);
    }

    #[test]
    fn load_manager_name_invalid_input_or_shape_is_atomic_and_not_generic_param() {
        let mut editor = native_fixture();
        let before = editor.to_bytes().unwrap();
        for name in [
            "one\ttwo".to_string(),
            "one\ntwo".to_string(),
            "one\rtwo".to_string(),
            "\0".to_string(),
            "\u{ffff}".to_string(),
            "🔌".repeat(16384),
        ] {
            assert!(
                editor
                    .set_load_manager_load_name("Loads", 0, &name)
                    .is_err()
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
        assert!(
            editor
                .set_load_manager_load_name("Loads", 12, "New")
                .is_err()
        );
        assert!(editor.set_param("Loads", "nm", "New").is_err());
        assert!(editor.set_param("Loads", "Load.0.nm", "New").is_err());
        assert_eq!(editor.to_bytes().unwrap(), before);
        let path = editor.require_one("Loads").unwrap();
        editor.get_element_mut(&path).children[3]
            .as_mut_element()
            .unwrap()
            .attributes
            .shift_remove("ioIdx");
        let before = editor.to_bytes().unwrap();
        assert!(
            editor
                .set_load_manager_load_name("Loads", 0, "New")
                .unwrap_err()
                .to_string()
                .contains("duplicate ioIdx 0")
        );
        assert_eq!(editor.to_bytes().unwrap(), before);
    }

    #[test]
    fn load_manager_native_omitted_zero_moves_without_filling_attributes() {
        let mut down = native_fixture();
        let original = down.to_bytes().unwrap();
        let inspection = down.inspect_load_manager("Loads").unwrap();
        assert_eq!(inspection.loads[0].io_index, 0);
        assert!(!inspection.loads[0].attributes.contains_key("ioIdx"));
        let path = down.require_one("Loads").unwrap();
        let mut expected = down.get_element(&path).clone();
        expected.children.swap(2, 3);
        down.move_load_manager_load("Loads", 0, LoadMoveDirection::Down)
            .unwrap();
        assert_eq!(*down.get_element(&path), expected);
        let loads = down.inspect_load_manager("Loads").unwrap().loads;
        assert_eq!(loads[1].io_index, 0);
        assert!(!loads[1].attributes.contains_key("ioIdx"));
        assert_eq!(loads[1].attributes["Unknown"], "keep-0");
        let mut up = native_fixture();
        up.move_load_manager_load("Loads", 1, LoadMoveDirection::Up)
            .unwrap();
        assert_eq!(up.to_bytes().unwrap(), down.to_bytes().unwrap());
        down.move_load_manager_load("Loads", 0, LoadMoveDirection::Up)
            .unwrap();
        assert_eq!(down.to_bytes().unwrap(), original);
    }

    #[test]
    fn load_manager_reads_native_golden_omitted_zero_without_mutation() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("eval")
            .join("golden")
            .join("configs")
            .join("Hotwater_Heating_Rod.Loxone");
        let bytes = std::fs::read(file).unwrap();
        let editor = ConfigEditor::load(&bytes).unwrap();
        let before = editor.to_bytes().unwrap();
        let blocks = editor.find_elements("Type:LoadShed");
        assert!(!blocks.is_empty());
        for path in blocks {
            let selector = format!("uuid:{}", editor.get_element(&path).attributes["U"]);
            let inspection = editor.inspect_load_manager(&selector).unwrap();
            assert_eq!(inspection.loads[0].io_index, 0);
            assert!(!inspection.loads[0].attributes.contains_key("ioIdx"));
        }
        assert_eq!(editor.to_bytes().unwrap(), before);
    }

    #[test]
    fn load_manager_duplicate_resolved_zero_and_malformed_identity_fail_atomically() {
        for identity in [
            None,
            Some("0"),
            Some(""),
            Some("future"),
            Some(" "),
            Some("-1"),
            Some("+1"),
        ] {
            let mut editor = native_fixture();
            let path = editor.require_one("Loads").unwrap();
            let load = editor.get_element_mut(&path).children[3]
                .as_mut_element()
                .unwrap();
            match identity {
                Some(value) => {
                    load.attributes.insert("ioIdx".into(), value.into());
                }
                None => {
                    load.attributes.shift_remove("ioIdx");
                }
            }
            let before = editor.to_bytes().unwrap();
            let error = editor
                .move_load_manager_load("Loads", 0, LoadMoveDirection::Down)
                .unwrap_err();
            if identity.is_none() || identity == Some("0") {
                assert!(error.to_string().contains("duplicate ioIdx 0"));
            }
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
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
        for mutation in [
            "duplicate",
            "omitted-with-explicit-zero",
            "extra",
            "sparse",
            "out-of-range",
        ] {
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
                "omitted-with-explicit-zero" => {
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
