//! # Casper-3 (The Woman)
//!
//! Embodies the persona of Dr. Naoko Akagi as a pragmatic woman.
//! Casper prioritizes real-world deliverability, developer ergonomics (DX),
//! maintainability, preventing over-engineering, and balancing architectural ideals with reality.
//! Externalized in `skills/magi-system/casper.md`.

use crate::config::NodeConfig;
use crate::error::MagiError;
use crate::llm::{
    dispatch_llm_request_with_fallback, LlmProvider, NodeEvaluation, ServerToolsConfig,
};
use crate::skills::PromptLoader;
use async_trait::async_trait;
use reqwest::Client;

/// Casper-3 Node module.
pub struct CasperNode {
    client: Client,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub fallback_model: Option<String>,
    pub subagent_model: Option<String>,
    pub subagent_web_search: bool,
    pub enable_web_search: bool,
    pub api_key: Option<String>,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub max_context_chars: usize,
    pub max_tokens: u32,
    prompt_loader: PromptLoader,
}

impl CasperNode {
    /// Creates a new Casper-3 node from node configuration.
    pub fn new(cfg: &NodeConfig) -> Self {
        Self {
            client: Client::new(),
            provider: cfg.provider.clone(),
            base_url: cfg.base_url.clone(),
            model: cfg.model.clone(),
            fallback_model: cfg.fallback_model.clone(),
            subagent_model: cfg.subagent_model.clone(),
            subagent_web_search: cfg.subagent_web_search,
            enable_web_search: cfg.enable_web_search,
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
impl LlmProvider for CasperNode {
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

        let tools_cfg = ServerToolsConfig {
            subagent_model: self.subagent_model.clone(),
            subagent_web_search: self.subagent_web_search,
            enable_web_search: self.enable_web_search,
        };

        dispatch_llm_request_with_fallback(
            &self.client,
            node_id,
            &self.provider,
            &self.base_url,
            &self.model,
            self.fallback_model.as_deref(),
            Some(&tools_cfg),
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
