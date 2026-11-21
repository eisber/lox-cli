use std::collections::BTreeMap;

use anyhow::{Result, bail};
pub use lox_core::configuration::editor::LoadManagerMoveDirection as LoadMoveDirection;
use lox_core::configuration::editor::{
    LoadManagerInspectionError, LoadManagerMoveError, LoadManagerNameError, LoadManagerPowerError,
    legacy_load_manager,
};
use serde::Serialize;
use xmltree::Element;

use super::{ConfigEditor, editor_error};

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
    /// Edit only Minimum on time [s], including the native zero storage default.
    pub fn set_load_manager_load_min_on(
        &mut self,
        selector: &str,
        io_index: u8,
        min_on_seconds: &str,
    ) -> Result<bool> {
        if io_index > 11 {
            bail!("Load Manager io-index must be 0..11, not a 1-based priority.");
        }
        let seconds = (!min_on_seconds.is_empty()
            && min_on_seconds.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| min_on_seconds.parse::<i32>().ok())
        .flatten()
        .filter(|seconds| *seconds == 0 || *seconds >= 10)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Load Manager minimum on time must be 0 or whole seconds in 10..2147483647, using decimal digits only. Native values 1..9 clamp to 10; this CLI rejects them instead. No rounding, clamping or unit conversion; nothing was changed."
            )
        })?;
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
        let previous = match load.attributes.get("minOn") {
            None => 0,
            Some(raw) => {
                let digits = raw.strip_prefix('+').unwrap_or(raw);
                (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
                    .then(|| digits.parse::<i32>().ok())
                    .flatten()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Unsupported Load Manager minOn storage for io-index {io_index}: expected a nonnegative signed-32-bit integer in seconds (an optional leading plus is supported). Empty, negative, fractional, exponent, whitespace and malformed storage are rejected; nothing was changed."
                        )
                    })?
            }
        };
        if previous == seconds {
            return Ok(false);
        }
        let load = self.get_element_mut(&path).children[position]
            .as_mut_element()
            .unwrap();
        // Native minOn uses type 5 with omit-zero enabled. A semantic no-op
        // above preserves even explicitly spelled zero instead of normalizing it.
        if seconds == 0 {
            load.attributes.shift_remove("minOn");
        } else {
            load.attributes.insert("minOn".into(), seconds.to_string());
        }
        Ok(true)
    }

    /// Edit only Maximum off time [s], using an exact positive integer subset.
    pub fn set_load_manager_load_max_off(
        &mut self,
        selector: &str,
        io_index: u8,
        max_off_seconds: &str,
    ) -> Result<bool> {
        if io_index > 11 {
            bail!("Load Manager io-index must be 0..11, not a 1-based priority.");
        }
        let seconds = load_max_off_integer(max_off_seconds)
            .filter(|seconds| *seconds >= 10)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Load Manager maximum off time must be whole seconds in 10..2147483647, using decimal digits only. Zero's UI meaning is unqualified; native values 1..9 clamp to 10, so this conservative CLI subset rejects them instead. No rounding, clamping or unit conversion; nothing was changed."
                )
            })?;
        // Requests intentionally exclude even a leading plus; existing integer
        // storage may retain it because the native integer reader accepts it.
        if !max_off_seconds.bytes().all(|byte| byte.is_ascii_digit()) {
            bail!(
                "Load Manager maximum off time requires decimal digits only, in whole seconds; nothing was changed."
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
        let previous = match load.attributes.get("maxOff") {
            // Native type-5 read with preserve-missing disabled parses absence as zero.
            None => 0,
            Some(raw) => load_max_off_integer(raw).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unsupported Load Manager maxOff storage for io-index {io_index}: expected a nonnegative signed-32-bit integer in seconds (an optional leading plus is supported). Explicit empty, negative, fractional, exponent, whitespace and malformed storage are rejected; nothing was changed."
                )
            })?,
        };
        if previous == seconds {
            return Ok(false);
        }
        self.get_element_mut(&path).children[position]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("maxOff".into(), seconds.to_string());
        Ok(true)
    }

    /// Edit the Configure loads Power [kW] cell without rounding the request.
    pub fn set_load_manager_load_power(
        &mut self,
        selector: &str,
        io_index: u8,
        power_kw: &str,
    ) -> Result<bool> {
        self.kernel
            .set_load_manager_load_power(selector, io_index, power_kw)
            .map_err(|error| match error {
                LoadManagerPowerError::Inspection(error) => load_manager_error(error),
                error => anyhow::anyhow!("{error}"),
            })
    }

    fn load_manager_path(&self, selector: &str) -> Result<Vec<usize>> {
        self.kernel
            .load_manager_path(selector)
            .map_err(load_manager_error)
    }

    pub fn inspect_load_manager(&self, selector: &str) -> Result<LoadManagerInspection> {
        let inspection = self
            .kernel
            .load_manager_inspection_selected(selector)
            .map_err(load_manager_error)?;
        Ok(LoadManagerInspection {
            uuid: inspection.uuid,
            title: inspection.title,
            loads: inspection
                .loads
                .into_iter()
                .map(|load| LoadManagerLoad {
                    priority: load.priority,
                    io_index: load.io_index,
                    attributes: load.attributes,
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
        self.kernel
            .set_load_manager_load_name(selector, io_index, name)
            .map_err(|error| match error {
                LoadManagerNameError::Inspection(error) => load_manager_error(error),
                error => anyhow::anyhow!("{error}"),
            })
    }

    /// Move the complete record in Configure loads/XML order, keeping its connector identity.
    pub fn move_load_manager_load(
        &mut self,
        selector: &str,
        io_index: u8,
        direction: LoadMoveDirection,
    ) -> Result<()> {
        self.kernel
            .move_load_manager_load(selector, io_index, direction)
            .map_err(|error| match error {
                LoadManagerMoveError::Inspection(error) => load_manager_error(error),
                error => anyhow::anyhow!("{error}"),
            })
    }
}

fn load_max_off_integer(raw: &str) -> Option<i32> {
    let digits = raw.strip_prefix('+').unwrap_or(raw);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn load_positions(block: &Element) -> Result<Vec<usize>> {
    legacy_load_manager::load_positions(block).map_err(load_manager_error)
}

fn resolve_io_index(load: &Element) -> Result<u8> {
    legacy_load_manager::resolve_io_index(load).map_err(load_manager_error)
}

fn load_manager_error(error: LoadManagerInspectionError) -> anyhow::Error {
    match error {
        LoadManagerInspectionError::Edit(error) => editor_error(error),
        error => anyhow::anyhow!("{error}"),
    }
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
    fn load_manager_power_preserves_identity_order_and_every_other_field() {
        let mut editor = native_fixture();
        editor
            .move_load_manager_load("Loads", 0, LoadMoveDirection::Down)
            .unwrap();
        let path = editor.require_one("Loads").unwrap();
        let mut expected = editor.get_element(&path).clone();
        let position = load_positions(&expected).unwrap()[1];
        expected.children[position]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("pwr".into(), "2.125".into());
        assert!(
            editor
                .set_load_manager_load_power("Loads", 0, "0002.125")
                .unwrap()
        );
        assert_eq!(*editor.get_element(&path), expected);
        let bytes = editor.to_bytes().unwrap();
        assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(bytes.windows(2).any(|window| window == b"\r\n"));
        let reloaded = ConfigEditor::load(&bytes).unwrap();
        let loads = reloaded.inspect_load_manager("Loads").unwrap().loads;
        assert_eq!(loads[1].attributes["pwr"], "2.125");
        assert!(!loads[1].attributes.contains_key("ioIdx"));
        assert!(editor.set_param("Loads", "pwr", "4").is_err());
        assert!(editor.set_param("Loads", "Load.0.pwr", "4").is_err());
    }

    #[test]
    fn load_manager_power_decimal_boundaries_and_noop_raw_spellings() {
        let mut editor = native_fixture();
        let path = editor.require_one("Loads").unwrap();
        let position = load_positions(editor.get_element(&path)).unwrap()[0];
        editor.get_element_mut(&path).children[position]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("pwr".into(), "2.5000e0".into());
        let bytes = editor.to_bytes().unwrap();
        assert!(
            !editor
                .set_load_manager_load_power("Loads", 0, "2.500")
                .unwrap()
        );
        assert_eq!(editor.to_bytes().unwrap(), bytes);
        for value in ["0", "0.001", "0.999", "2.5", "9007199254740992"] {
            assert!(
                editor
                    .set_load_manager_load_power("Loads", 0, value)
                    .unwrap()
            );
            assert_eq!(
                editor.inspect_load_manager("Loads").unwrap().loads[0].attributes["pwr"],
                value
            );
        }
        for value in [
            "",
            "-1",
            "-0",
            "+1",
            "NaN",
            "inf",
            "1e3",
            "2,5",
            " 2",
            "2 ",
            ".5",
            "2.",
            "1.0001",
            "0.0005",
            "9007199254740993",
            "9007199254740992.001",
            "281474976710656.062",
        ] {
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .set_load_manager_load_power("Loads", 0, value)
                    .is_err(),
                "{value}"
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
        assert!(
            editor
                .set_load_manager_load_power("Loads", 0, &format!("1{}", "0".repeat(309)))
                .is_err()
        );
    }

    #[test]
    fn load_manager_power_rejects_malformed_target_storage_without_defaulting_others() {
        for raw in [
            None,
            Some(""),
            Some("junk"),
            Some("-1"),
            Some("NaN"),
            Some("inf"),
            Some("2e1"),
            Some("2.5e1"),
            Some(" 2.5"),
        ] {
            let mut editor = native_fixture();
            let path = editor.require_one("Loads").unwrap();
            let positions = load_positions(editor.get_element(&path)).unwrap();
            for position in &positions {
                editor.get_element_mut(&path).children[*position]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .shift_remove("maxOff");
            }
            let load = editor.get_element_mut(&path).children[positions[0]]
                .as_mut_element()
                .unwrap();
            if let Some(raw) = raw {
                load.attributes.insert("pwr".into(), raw.into());
            } else {
                load.attributes.shift_remove("pwr");
            }
            let before = editor.to_bytes().unwrap();
            assert!(
                editor
                    .set_load_manager_load_power("Loads", 0, "2.5")
                    .is_err()
            );
            assert_eq!(editor.to_bytes().unwrap(), before);
        }
        let mut editor = native_fixture();
        let path = editor.require_one("Loads").unwrap();
        let positions = load_positions(editor.get_element(&path)).unwrap();
        editor.get_element_mut(&path).children[positions[1]]
            .as_mut_element()
            .unwrap()
            .attributes
            .shift_remove("pwr");
        assert!(editor.set_load_manager_load_power("Loads", 0, "0").unwrap());
        assert!(
            !editor.inspect_load_manager("Loads").unwrap().loads[1]
                .attributes
                .contains_key("pwr")
        );
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
