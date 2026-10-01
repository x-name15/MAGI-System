//! # Balthasar-2 (The Mother)
//!
//! Embodies the persona of Dr. Naoko Akagi as a protective mother.
//! Balthasar prioritizes security, vulnerability prevention, data integrity,
//! defensive programming, and holds unilateral VETO power on critical risk.
//! Externalized in `skills/magi-system/balthasar.md`.

use crate::config::NodeConfig;
use crate::error::MagiError;
use crate::llm::{dispatch_llm_request, LlmProvider, NodeEvaluation};
use crate::skills::PromptLoader;
use async_trait::async_trait;
use reqwest::Client;

/// Balthasar-2 Node module.
pub struct BalthasarNode {
    client: Client,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
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
            api_key: cfg.api_key.clone(),
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

        dispatch_llm_request(
            &self.client,
            node_id,
            &self.provider,
            &self.base_url,
            &self.model,
            self.api_key.as_deref(),
            &active_prompt,
            user_prompt,
            context_payload,
        )
        .await
    }
}
