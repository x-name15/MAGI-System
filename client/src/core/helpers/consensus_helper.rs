//! # Consensus Evaluation Helper
//!
//! Provides pure consensus calculation and verdict resolution across Trinity node evaluations,
//! applying the Balthasar-2 security veto rule and i18n summary resolution.

use crate::i18n::{get_bundle, Language};
use crate::llm::NodeEvaluation;

/// Structure representing the resolved outcome of a MAGI deliberation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusOutcome {
    /// Full verdict key (e.g. APPROVED_UNANIMOUS, APPROVED_MAJORITY, VETO_BALTHASAR, SPLIT_DECISION).
    pub verdict: String,
    /// High-level simplified status (APPROVED, REJECTED, SPLIT).
    pub simple_verdict: String,
    /// Explanatory summary text according to active language.
    pub summary: String,
    /// Whether Balthasar-2 activated the security veto.
    pub is_veto: bool,
    /// Approving node count.
    pub approves: u8,
    /// Rejecting node count.
    pub rejects: u8,
    /// Neutral / other node count.
    pub neutrals: u8,
}

/// Computes the consensus outcome from a slice of node evaluations.
pub fn calculate_local_consensus(
    evaluations: &[NodeEvaluation],
    lang: Language,
) -> ConsensusOutcome {
    let bundle = get_bundle(lang);
    let mut approves: u8 = 0;
    let mut rejects: u8 = 0;
    let mut neutrals: u8 = 0;
    let mut is_veto = false;

    for eval in evaluations {
        let v = eval.vote.trim().to_uppercase();
        if eval.node_id == "Balthasar-2"
            && v == "REJECT"
            && (eval.risk_score >= 8 || !eval.cwe_flags.is_empty())
        {
            is_veto = true;
        }

        match v.as_str() {
            "APPROVE" => approves += 1,
            "REJECT" => rejects += 1,
            _ => neutrals += 1,
        }
    }

    if is_veto {
        ConsensusOutcome {
            verdict: "VETO_BALTHASAR".to_string(),
            simple_verdict: "REJECTED".to_string(),
            summary: bundle.ui.verdict_veto_detail.clone(),
            is_veto: true,
            approves,
            rejects,
            neutrals,
        }
    } else if approves == 3 {
        ConsensusOutcome {
            verdict: "APPROVED_UNANIMOUS".to_string(),
            simple_verdict: "APPROVED".to_string(),
            summary: bundle.ui.verdict_unanimous_approve_detail.clone(),
            is_veto: false,
            approves,
            rejects,
            neutrals,
        }
    } else if approves >= 2 {
        ConsensusOutcome {
            verdict: "APPROVED_MAJORITY".to_string(),
            simple_verdict: "APPROVED".to_string(),
            summary: bundle.ui.verdict_majority_approve_detail.clone(),
            is_veto: false,
            approves,
            rejects,
            neutrals,
        }
    } else if rejects == 3 {
        ConsensusOutcome {
            verdict: "REJECTED_UNANIMOUS".to_string(),
            simple_verdict: "REJECTED".to_string(),
            summary: bundle.ui.verdict_unanimous_reject_detail.clone(),
            is_veto: false,
            approves,
            rejects,
            neutrals,
        }
    } else if rejects >= 2 {
        ConsensusOutcome {
            verdict: "REJECTED_MAJORITY".to_string(),
            simple_verdict: "REJECTED".to_string(),
            summary: bundle.ui.verdict_majority_reject_detail.clone(),
            is_veto: false,
            approves,
            rejects,
            neutrals,
        }
    } else {
        ConsensusOutcome {
            verdict: "SPLIT_DECISION".to_string(),
            simple_verdict: "SPLIT".to_string(),
            summary: bundle.ui.verdict_split_detail.clone(),
            is_veto: false,
            approves,
            rejects,
            neutrals,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_eval(node_id: &str, vote: &str, risk: u8, cwe: Vec<&str>) -> NodeEvaluation {
        NodeEvaluation {
            node_id: node_id.to_string(),
            vote: vote.to_string(),
            risk_score: risk,
            findings: vec![],
            rationale: "test".to_string(),
            argument: "test".to_string(),
            confidence: 0.95,
            cwe_flags: cwe.into_iter().map(String::from).collect(),
            execution_time_ms: 100,
            prompt_version: "v1".to_string(),
            model: "test".to_string(),
            initial_argument: None,
            initial_vote: None,
            initial_risk_score: None,
        }
    }

    #[test]
    fn test_unanimous_approve() {
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "APPROVE", 3, vec![]),
            dummy_eval("Casper-3", "APPROVE", 1, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "APPROVED_UNANIMOUS");
        assert_eq!(outcome.simple_verdict, "APPROVED");
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_balthasar_veto_overrides() {
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 8, vec!["CWE-89"]),
            dummy_eval("Casper-3", "APPROVE", 1, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "VETO_BALTHASAR");
        assert_eq!(outcome.simple_verdict, "REJECTED");
        assert!(outcome.is_veto);
    }

    #[test]
    fn test_majority_reject() {
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 4, vec![]),
            dummy_eval("Casper-3", "REJECT", 5, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "REJECTED_MAJORITY");
        assert_eq!(outcome.simple_verdict, "REJECTED");
        assert!(!outcome.is_veto);
    }
}
