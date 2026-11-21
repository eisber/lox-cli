use super::{ConfigEditor, selection_error};
use anyhow::Result;

impl ConfigEditor {
    pub fn require_one(&self, selector: &str) -> Result<Vec<usize>> {
        self.kernel.require_one(selector).map_err(selection_error)
    }
}
