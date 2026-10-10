//! # UI Rendering Helpers
//!
//! Submodules for Unicode layout calculations, triangular monitor bus rendering,
//! and CRT phosphor frame generation.

pub mod history_loader;
pub mod layout_helper;
pub mod monitor_helper;

#[allow(unused_imports)]
pub use history_loader::{
    load_hybrid, parse_deliberation_markdown, DeliberationHistoryEntry, NodePositionSummary,
};
#[allow(unused_imports)]
pub use layout_helper::{
    render_triangular_screen, safe_truncate_str, str_display_width, truncate_with_ellipsis,
};
pub use monitor_helper::{
    format_blackout_monitor, format_deliberating_monitor, format_node_monitor,
};
