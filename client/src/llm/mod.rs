//! # LLM Provider Abstractions and Personas
//!
//! MAGI System Node Modules:
//! - Melchior-1 (The Scientist: logic, architecture, algorithms)
//! - Balthasar-2 (The Mother: security, defensive engineering, risk, veto)
//! - Casper-3 (The Woman: pragmatism, developer experience, viability)

pub mod balthasar;
pub mod casper;
pub mod helpers;
pub mod melchior;
pub mod mock;

pub use balthasar::BalthasarNode;
pub use casper::CasperNode;
pub use helpers::dispatch_llm_request;
pub use melchior::MelchiorNode;

use crate::error::MagiError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A single structured finding discovered during node audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub recommendation: String,
}

/// Structured response emitted by a MAGI node evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeEvaluation {
    pub node_id: String,
    pub vote: String,
    pub risk_score: u8,
    #[serde(default)]
    pub findings: Vec<Finding>,
    pub rationale: String,
    pub argument: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    pub cwe_flags: Vec<String>,
    pub execution_time_ms: u32,
    #[serde(default)]
    pub prompt_version: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub initial_argument: Option<String>,
    #[serde(default)]
    pub initial_vote: Option<String>,
    #[serde(default)]
    pub initial_risk_score: Option<u8>,
}

fn default_confidence() -> f32 {
    0.95
}

/// Raw JSON output expected from LLM completion.
#[derive(Debug, Deserialize)]
pub struct RawNodeOutput {
    #[serde(alias = ".vote", alias = "verdict", alias = "decision")]
    pub vote: String,
    #[serde(
        default = "default_risk",
        alias = ".risk_score",
        alias = "riskScore",
        alias = "risk"
    )]
    pub risk_score: u8,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(
        default,
        alias = ".rationale",
        alias = "argument",
        alias = "analysis",
        alias = "reasoning"
    )]
    pub rationale: String,
    #[serde(default = "default_confidence", alias = ".confidence")]
    pub confidence: f32,
    #[serde(default, alias = ".cwe_flags", alias = "cweFlags", alias = "cwe")]
    pub cwe_flags: Vec<String>,
}

fn default_risk() -> u8 {
    5
}

/// Common trait for all MAGI node modules and providers.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Evaluates code context under the node's analytical lens.
    async fn evaluate(
        &self,
        node_id: &str,
        system_prompt: &str,
        user_prompt: &str,
        context_payload: &str,
    ) -> Result<NodeEvaluation, MagiError>;
}
