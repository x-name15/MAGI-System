//! # Structured JSON Output
//!
//! Serializes the final deliberation verdict and per-node evaluations into a
//! machine-readable JSON envelope, suitable for piping into CI scripts, dashboards,
//! or downstream tools.

use crate::llm::NodeEvaluation;
use serde::Serialize;

/// Top-level JSON envelope emitted when `--output json` is requested.
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub deliberation_id: u64,
    pub title: String,
    pub category: String,
    pub context_type: String,
    pub verdict: String,
    pub summary: String,
    pub rounds: u8,
    pub nodes: Vec<JsonNodeResult>,
}

/// Per-node result included in the JSON envelope.
#[derive(Debug, Serialize)]
pub struct JsonNodeResult {
    pub node_id: String,
    pub final_vote: String,
    pub initial_vote: Option<String>,
    pub risk_score: u8,
    pub initial_risk_score: Option<u8>,
    pub confidence: f32,
    pub model: String,
    pub execution_time_ms: u32,
    pub cwe_flags: Vec<String>,
    pub rationale: String,
}

impl JsonOutput {
    /// Constructs the envelope from verdict data and per-node evaluations.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        deliberation_id: u64,
        title: &str,
        category: &str,
        context_type: &str,
        verdict: &str,
        summary: &str,
        rounds: u8,
        evaluations: &[NodeEvaluation],
    ) -> Self {
        let nodes = evaluations
            .iter()
            .map(|e| JsonNodeResult {
                node_id: e.node_id.clone(),
                final_vote: e.vote.clone(),
                initial_vote: e.initial_vote.clone(),
                risk_score: e.risk_score,
                initial_risk_score: e.initial_risk_score,
                confidence: e.confidence,
                model: e.model.clone(),
                execution_time_ms: e.execution_time_ms,
                cwe_flags: e.cwe_flags.clone(),
                rationale: e.argument.clone(),
            })
            .collect();

        Self {
            deliberation_id,
            title: title.to_string(),
            category: category.to_string(),
            context_type: context_type.to_string(),
            verdict: verdict.to_string(),
            summary: summary.to_string(),
            rounds,
            nodes,
        }
    }

    /// Prints the envelope as pretty-printed JSON to stdout.
    pub fn print(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Failed to serialize JSON output: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output_serialization() {
        let evals = vec![NodeEvaluation {
            node_id: "Melchior-1".to_string(),
            vote: "APPROVE".to_string(),
            risk_score: 3,
            findings: vec![],
            rationale: "Clean architecture".to_string(),
            argument: "Clean architecture".to_string(),
            confidence: 0.95,
            cwe_flags: vec![],
            execution_time_ms: 120,
            prompt_version: "v1".to_string(),
            model: "test-model".to_string(),
            initial_argument: Some("Initial thought".to_string()),
            initial_vote: Some("NEUTRAL".to_string()),
            initial_risk_score: Some(5),
        }];

        let json_out = JsonOutput::build(
            42,
            "Architecture Review",
            "CASE_1",
            "MARKDOWN",
            "APPROVED",
            "Consensus approved unanimously",
            3,
            &evals,
        );

        assert_eq!(json_out.deliberation_id, 42);
        assert_eq!(json_out.rounds, 3);
        assert_eq!(json_out.nodes.len(), 1);
        assert_eq!(json_out.nodes[0].node_id, "Melchior-1");
        assert_eq!(json_out.nodes[0].final_vote, "APPROVE");
        assert_eq!(json_out.nodes[0].initial_vote.as_deref(), Some("NEUTRAL"));
        assert_eq!(json_out.nodes[0].initial_risk_score, Some(5));

        let serialized = serde_json::to_string(&json_out).expect("serialization succeeds");
        assert!(serialized.contains("\"verdict\":\"APPROVED\""));
        assert!(serialized.contains("\"rounds\":3"));
    }
}
