use super::{ConfigEditor, editor_error};
use anyhow::Result;

impl ConfigEditor {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.kernel.to_bytes().map_err(editor_error)
    }
}
