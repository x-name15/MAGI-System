//! # Core Orchestration Helpers
//!
//! Submodule containing modular helpers for Trinity peer debates, context construction,
//! error routing heuristics, project discovery, and smart noise filtering.

#![allow(unused_imports)]

pub mod consensus_helper;
pub mod debate_helper;
pub mod discovery_helper;
pub mod git_helper;
pub mod noise_filter_helper;
pub mod triage_helper;

pub use consensus_helper::calculate_local_consensus;
pub use debate_helper::{build_debate_prompts, build_peer_summary};
pub use discovery_helper::{discover_project_context, ProjectContext};
pub use git_helper::{get_git_diff, get_git_diff_filtered, get_raw_git_diff};
pub use noise_filter_helper::{filter_git_diff, is_noise_file, FilteredDiff};
pub use triage_helper::select_lead_node_for_error;
