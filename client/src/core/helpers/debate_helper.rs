//! # Debate Helpers for MAGI Orchestrator
//!
//! Provides formatting and serialization utilities for peer evaluations across
//! iterative deliberation rounds.

use crate::i18n::Language;
use crate::llm::NodeEvaluation;

/// Serializes previous-round peer positions into a plain-text debate context block.
pub fn build_peer_summary(evals: &[NodeEvaluation]) -> String {
    evals
        .iter()
        .map(|e| {
            format!(
                "{} | vote={} | risk={} | argument={} | cwe={}",
                e.node_id,
                e.vote,
                e.risk_score,
                e.argument,
                e.cwe_flags.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Constructs the debate prompt and peer context for an intermediate or final round.
pub fn build_debate_prompts(
    lang: Language,
    user_prompt: &str,
    context_payload: &str,
    peer_positions: &str,
    round_num: u8,
    total_rounds: u8,
) -> (String, String) {
    let bundle = crate::i18n::get_bundle(lang);
    let is_final = round_num == total_rounds;

    let debate_prompt = bundle
        .debate
        .format_prompt(is_final, user_prompt, round_num, total_rounds);
    let debate_context =
        bundle
            .debate
            .format_context(context_payload, peer_positions, round_num.saturating_sub(1));

    (debate_prompt, debate_context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_peer_summary() {
        let evals = vec![NodeEvaluation {
            node_id: "Melchior-1".to_string(),
            vote: "APPROVE".to_string(),
            risk_score: 2,
            findings: vec![],
            rationale: "Solid architecture".to_string(),
            argument: "Solid architecture".to_string(),
            confidence: 0.95,
            cwe_flags: vec!["CWE-200".to_string()],
            execution_time_ms: 100,
            prompt_version: "v1".to_string(),
            model: "test".to_string(),
            initial_argument: None,
            initial_vote: None,
            initial_risk_score: None,
        }];

        let summary = build_peer_summary(&evals);
        assert!(summary.contains("Melchior-1 | vote=APPROVE | risk=2"));
        assert!(summary.contains("cwe=CWE-200"));
    }
}
