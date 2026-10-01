//! # Mock LLM Provider for Testing and Offline Playground Mode

use crate::error::MagiError;
use crate::llm::{LlmProvider, NodeEvaluation};
use async_trait::async_trait;

/// Configurable mock provider simulating node behavior.
#[derive(Clone, Debug)]
pub struct MockProvider {
    pub default_vote: String,
    pub default_risk_score: u8,
    pub default_argument: String,
    pub default_cwe_flags: Vec<String>,
}

impl MockProvider {
    /// Creates a mock provider tailored for a specific persona.
    pub fn for_node(node_id: &str, simulate_security_risk: bool) -> Self {
        match node_id {
            "Melchior-1" => Self {
                default_vote: "APPROVE".to_string(),
                default_risk_score: 2,
                default_argument: "Architecture conforms to clean separation of concerns. Cyclomatic complexity is acceptable.".to_string(),
                default_cwe_flags: vec![],
            },
            "Balthasar-2" => {
                if simulate_security_risk {
                    Self {
                        default_vote: "REJECT".to_string(),
                        default_risk_score: 9, // Triggers Balthasar security veto
                        default_argument: "Critical security vulnerability detected: Potential SQL/Command injection and unvalidated input.".to_string(),
                        default_cwe_flags: vec!["CWE-89".to_string(), "CWE-20".to_string()],
                    }
                } else {
                    Self {
                        default_vote: "APPROVE".to_string(),
                        default_risk_score: 2,
                        default_argument: "Security review passed. No high-severity CWE patterns or exposed credentials observed.".to_string(),
                        default_cwe_flags: vec![],
                    }
                }
            }
            "Casper-3" => Self {
                default_vote: "APPROVE".to_string(),
                default_risk_score: 1,
                default_argument: "Pragmatic implementation with minimal cognitive overhead and zero overengineering.".to_string(),
                default_cwe_flags: vec![],
            },
            _ => Self {
                default_vote: "NEUTRAL".to_string(),
                default_risk_score: 5,
                default_argument: "Indeterminate analysis from generic evaluator.".to_string(),
                default_cwe_flags: vec![],
            },
        }
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn evaluate(
        &self,
        node_id: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        context_payload: &str,
    ) -> Result<NodeEvaluation, MagiError> {
        // Dynamic inspection: check if context payload contains obvious vulnerability indicators
        let mut vote = self.default_vote.clone();
        let mut risk_score = self.default_risk_score;
        let mut argument = self.default_argument.clone();
        let mut cwe_flags = self.default_cwe_flags.clone();

        if node_id == "Balthasar-2"
            && (context_payload.contains("format!(\"SELECT")
                || context_payload.contains("eval(")
                || context_payload.contains("dangerouslySetInnerHTML")
                || context_payload.contains("Security Warning")
                || context_payload.contains("hardcoded credentials"))
        {
            vote = "REJECT".to_string();
            risk_score = 9;
            argument = "High-severity vulnerability discovered: Unsanitized data leak or security violation (CWE-89 / CWE-798).".to_string();
            cwe_flags = vec!["CWE-89".to_string(), "CWE-798".to_string()];
        }

        let rationale = argument.clone();
        let findings = if vote == "REJECT" {
            vec![crate::llm::Finding {
                category: "security".to_string(),
                severity: "critical".to_string(),
                title: "Vulnerability detected in payload".to_string(),
                evidence: context_payload.chars().take(80).collect(),
                impact: "Possible exploit execution or unauthorized access".to_string(),
                recommendation: "Sanitize inputs and enforce parameterized queries".to_string(),
            }]
        } else {
            vec![]
        };

        Ok(NodeEvaluation {
            node_id: node_id.to_string(),
            vote,
            risk_score,
            findings,
            rationale,
            argument,
            confidence: 0.98,
            cwe_flags,
            execution_time_ms: 42,
            prompt_version: "sha256:mock_v1".to_string(),
            model: "mock-sim".to_string(),
            initial_argument: None,
            initial_vote: None,
            initial_risk_score: None,
        })
    }
}
