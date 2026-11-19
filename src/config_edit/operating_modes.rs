use anyhow::Result;
use lox_core::configuration::editor::operating_modes::{
    OperatingModeAdded, OperatingModeError, add_operating_mode,
};

use super::{ConfigEditor, editor_error};

impl ConfigEditor {
    pub fn add_operating_mode(data: &[u8], title: &str) -> Result<(Vec<u8>, OperatingModeAdded)> {
        add_operating_mode(data, title, rand::random, || {
            chrono::Local::now().fixed_offset()
        })
        .map_err(|error| match error {
            OperatingModeError::RawXml(cause) => cause.into(),
            OperatingModeError::RawAttribute(cause) => cause.into(),
            OperatingModeError::Load(cause) => editor_error(cause),
            OperatingModeError::SerializeFragment(cause) => cause.into(),
            OperatingModeError::FragmentUtf8(cause) => cause.into(),
            refusal => refusal.into(),
        })
    }
}
