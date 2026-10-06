//! # UI Rendering Helpers
//!
//! Submodules for Unicode layout calculations, triangular monitor bus rendering,
//! and CRT phosphor frame generation.

pub mod layout_helper;
pub mod monitor_helper;

pub use layout_helper::{render_triangular_screen, str_display_width};
pub use monitor_helper::{
    format_blackout_monitor, format_deliberating_monitor, format_node_monitor,
};
