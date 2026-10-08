//! # Melchior-1 (The Scientist)
//!
//! Embodies the persona of Dr. Naoko Akagi as a scientist.
//! Melchior prioritizes logic, algorithms, architectural purity, system scalability,
//! and functional correctness. Externalized in `skills/magi-system/melchior.md`.

use crate::config::NodeConfig;
use crate::error::MagiError;
use crate::llm::{dispatch_llm_request_with_fallback, LlmProvider, NodeEvaluation};
use crate::skills::PromptLoader;
use async_trait::async_trait;
use reqwest::Client;

/// Melchior-1 Node module.
pub struct MelchiorNode {
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

impl MelchiorNode {
    /// Creates a new Melchior-1 node from node configuration.
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
impl LlmProvider for MelchiorNode {
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
