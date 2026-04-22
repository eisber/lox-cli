pub mod config_cmd;
pub mod ctx;

pub struct RunContext {
    pub json: bool,
    pub quiet: bool,
    #[allow(dead_code)]
    pub csv: bool,
    #[allow(dead_code)]
    pub dry_run: bool,
    #[allow(dead_code)]
    pub no_header: bool,
    pub trace_id: Option<String>,
}
