//! # Balthasar-2 (The Mother)
//!
//! Embodies the persona of Dr. Naoko Akagi as a protective mother.
//! Balthasar prioritizes security, vulnerability prevention, data integrity,
//! defensive programming, and holds unilateral VETO power on critical risk.
//! Externalized in `skills/magi-system/balthasar.md`.

use crate::config::NodeConfig;
use crate::error::MagiError;
use crate::llm::{dispatch_llm_request_with_fallback, LlmProvider, NodeEvaluation};
use crate::skills::PromptLoader;
use async_trait::async_trait;
use reqwest::Client;

/// Balthasar-2 Node module.
pub struct BalthasarNode {
    client: Client,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub fallback_model: Option<String>,
    pub api_key: Option<String>,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub max_context_chars: usize,
    pub max_tokens: u32,
    prompt_loader: PromptLoader,
}

impl BalthasarNode {
    /// Creates a new Balthasar-2 node from node configuration.
    pub fn new(cfg: &NodeConfig) -> Self {
        Self {
            client: Client::new(),
            provider: cfg.provider.clone(),
            base_url: cfg.base_url.clone(),
            model: cfg.model.clone(),
            fallback_model: cfg.fallback_model.clone(),
            api_key: cfg.api_key.clone(),
            max_retries: cfg.max_retries,
            retry_delay_ms: cfg.retry_delay_ms,
            max_context_chars: cfg.max_context_chars,
            max_tokens: cfg.max_tokens,
            prompt_loader: PromptLoader::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for BalthasarNode {
    async fn evaluate(
        &self,
        node_id: &str,
        system_prompt: &str,
        user_prompt: &str,
        context_payload: &str,
    ) -> Result<NodeEvaluation, MagiError> {
        let active_prompt = if system_prompt.is_empty() {
            self.prompt_loader.load_node_prompt(node_id).0
        } else {
            system_prompt.to_string()
        };

        dispatch_llm_request_with_fallback(
            &self.client,
            node_id,
            &self.provider,
            &self.base_url,
            &self.model,
            self.fallback_model.as_deref(),
            self.api_key.as_deref(),
            &active_prompt,
            user_prompt,
            context_payload,
            self.max_retries,
            self.retry_delay_ms,
            self.max_context_chars,
            self.max_tokens,
        )
        .await
    }
}
