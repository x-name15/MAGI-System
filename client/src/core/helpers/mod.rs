//! # Core Orchestration Helpers
//!
//! Submodule containing modular helpers for Trinity peer debates, context construction,
//! and error routing heuristics.

pub mod consensus_helper;
pub mod debate_helper;
pub mod triage_helper;

pub use consensus_helper::calculate_local_consensus;
pub use debate_helper::{build_debate_prompts, build_peer_summary};
pub use triage_helper::select_lead_node_for_error;
