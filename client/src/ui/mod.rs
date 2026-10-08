//! # NERV Terminal User Interface Module

pub mod helpers;
pub mod history_browser;
pub mod intent;
pub mod nerv_theme;
pub mod output;
pub mod report;
pub mod tui;

pub use history_browser::{render_history_table, run_history_browser};
pub use nerv_theme::NervTheme;
pub use output::JsonOutput;
#[allow(unused_imports)]
pub use report::{
    get_deliberations_dir, get_next_local_deliberation_id, save_host_deliberation_report,
    save_host_deliberation_report_opts,
};
pub use tui::run_interactive_session;
