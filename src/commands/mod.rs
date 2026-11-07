pub mod blocks_cmd;
pub mod color_cmd;
pub mod config_cmd;
pub mod ctx;
pub mod live_cmd;
pub mod report_cmd;
pub mod sim_cmd;
pub mod stats_cmd;

pub struct RunContext {
    pub json: bool,
    pub quiet: bool,
    pub csv: bool,
    pub dry_run: bool,
    pub no_header: bool,
    #[allow(dead_code)]
    pub non_interactive: bool,
    pub trace_id: Option<String>,
    pub telemetry: Option<std::cell::RefCell<crate::telemetry::Telemetry>>,
}

impl RunContext {
    pub fn track_telemetry(&self, track: impl FnOnce(&mut crate::telemetry::Telemetry)) {
        if !self.dry_run
            && let Some(telemetry) = &self.telemetry
        {
            track(&mut telemetry.borrow_mut());
        }
    }
}
