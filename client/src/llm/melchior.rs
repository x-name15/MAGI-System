//! # Melchior-1 (The Scientist)
//!
//! Embodies the persona of Dr. Naoko Akagi as a scientist.
//! Melchior prioritizes logic, algorithms, architectural purity, system scalability,
//! and functional correctness. Externalized in `skills/magi-system/melchior.md`.

use crate::config::NodeConfig;
use crate::error::MagiError;
use crate::llm::{dispatch_llm_request, LlmProvider, NodeEvaluation};
use crate::skills::PromptLoader;
use async_trait::async_trait;
use reqwest::Client;

/// Melchior-1 Node module.
pub struct MelchiorNode {
    client: Client,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
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
            api_key: cfg.api_key.clone(),
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
