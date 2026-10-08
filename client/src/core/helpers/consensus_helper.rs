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

    #[test]
    fn test_unanimous_reject() {
        let evals = vec![
            dummy_eval("Melchior-1", "REJECT", 6, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 7, vec![]), // <8, no CWE -> no veto
            dummy_eval("Casper-3", "REJECT", 5, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "REJECTED_UNANIMOUS");
        assert_eq!(outcome.simple_verdict, "REJECTED");
        assert_eq!(outcome.rejects, 3);
        assert_eq!(outcome.approves, 0);
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_casper_systematic_dissent_allows_majority_approve() {
        // Melchior (logic) approves, Balthasar (security) approves, Casper (pragmatism) rejects
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 1, vec![]),
            dummy_eval("Balthasar-2", "APPROVE", 2, vec![]),
            dummy_eval("Casper-3", "REJECT", 7, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "APPROVED_MAJORITY");
        assert_eq!(outcome.simple_verdict, "APPROVED");
        assert_eq!(outcome.approves, 2);
        assert_eq!(outcome.rejects, 1);
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_degraded_quorum_two_approves_one_offline() {
        // 1 node offline / neutral position synthesized by degraded quorum
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "APPROVE", 1, vec![]),
            dummy_eval("Casper-3", "NEUTRAL", 0, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "APPROVED_MAJORITY");
        assert_eq!(outcome.simple_verdict, "APPROVED");
        assert_eq!(outcome.approves, 2);
        assert_eq!(outcome.neutrals, 1);
        assert_eq!(outcome.rejects, 0);
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_degraded_quorum_two_rejects_one_offline() {
        let evals = vec![
            dummy_eval("Melchior-1", "REJECT", 5, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 6, vec![]),
            dummy_eval("Casper-3", "NEUTRAL", 0, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "REJECTED_MAJORITY");
        assert_eq!(outcome.simple_verdict, "REJECTED");
        assert_eq!(outcome.rejects, 2);
        assert_eq!(outcome.neutrals, 1);
        assert_eq!(outcome.approves, 0);
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_split_decision_three_way() {
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "NEUTRAL", 4, vec![]),
            dummy_eval("Casper-3", "REJECT", 5, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "SPLIT_DECISION");
        assert_eq!(outcome.simple_verdict, "SPLIT");
        assert_eq!(outcome.approves, 1);
        assert_eq!(outcome.rejects, 1);
        assert_eq!(outcome.neutrals, 1);
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_split_decision_all_neutral() {
        let evals = vec![
            dummy_eval("Melchior-1", "NEUTRAL", 0, vec![]),
            dummy_eval("Balthasar-2", "NEUTRAL", 0, vec![]),
            dummy_eval("Casper-3", "NEUTRAL", 0, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "SPLIT_DECISION");
        assert_eq!(outcome.simple_verdict, "SPLIT");
        assert_eq!(outcome.neutrals, 3);
        assert_eq!(outcome.approves, 0);
        assert_eq!(outcome.rejects, 0);
    }

    #[test]
    fn test_balthasar_veto_high_risk_without_cwe() {
        // Risk >= 8 triggers veto even if CWE list is empty
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 1, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 9, vec![]),
            dummy_eval("Casper-3", "APPROVE", 1, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "VETO_BALTHASAR");
        assert_eq!(outcome.simple_verdict, "REJECTED");
        assert!(outcome.is_veto);
    }

    #[test]
    fn test_balthasar_veto_cwe_with_low_risk() {
        // CWE flag triggers veto even if risk score is low (< 8)
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 1, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 4, vec!["CWE-79"]),
            dummy_eval("Casper-3", "APPROVE", 1, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "VETO_BALTHASAR");
        assert_eq!(outcome.simple_verdict, "REJECTED");
        assert!(outcome.is_veto);
    }

    #[test]
    fn test_balthasar_reject_below_veto_threshold_no_cwe() {
        // Risk 7 with no CWE does NOT trigger veto; regular majority prevails
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 7, vec![]),
            dummy_eval("Casper-3", "APPROVE", 2, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "APPROVED_MAJORITY");
        assert_eq!(outcome.simple_verdict, "APPROVED");
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_balthasar_high_risk_approve_does_not_veto() {
        // Veto strictly requires a REJECT vote; high risk alone while voting APPROVE is not a veto
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 2, vec![]),
            dummy_eval("Balthasar-2", "APPROVE", 9, vec![]),
            dummy_eval("Casper-3", "APPROVE", 1, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "APPROVED_UNANIMOUS");
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_melchior_high_risk_reject_cannot_veto() {
        // Only Balthasar-2 has veto authority; Melchior-1 with high risk and CWE cannot veto
        let evals = vec![
            dummy_eval("Melchior-1", "REJECT", 10, vec!["CWE-89"]),
            dummy_eval("Balthasar-2", "APPROVE", 2, vec![]),
            dummy_eval("Casper-3", "APPROVE", 2, vec![]),
        ];
        let outcome = calculate_local_consensus(&evals, Language::En);
        assert_eq!(outcome.verdict, "APPROVED_MAJORITY");
        assert_eq!(outcome.simple_verdict, "APPROVED");
        assert!(!outcome.is_veto);
    }

    #[test]
    fn test_consensus_spanish_i18n() {
        let evals = vec![
            dummy_eval("Melchior-1", "APPROVE", 1, vec![]),
            dummy_eval("Balthasar-2", "REJECT", 8, vec!["CWE-89"]),
            dummy_eval("Casper-3", "APPROVE", 1, vec![]),
        ];
        let outcome_en = calculate_local_consensus(&evals, Language::En);
        let outcome_es = calculate_local_consensus(&evals, Language::Es);

        assert_eq!(outcome_en.verdict, outcome_es.verdict);
        assert_eq!(outcome_en.is_veto, outcome_es.is_veto);
        assert_ne!(outcome_en.summary, outcome_es.summary);
        assert!(outcome_es.summary.contains("Veto de seguridad activado"));
    }
}
