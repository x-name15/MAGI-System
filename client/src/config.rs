//! # Dynamic Configuration Loader for MAGI System
//!
//! Automatically infers provider settings from available environment variables
//! or defaults to offline mock simulation without requiring hardcoded configurations.

use crate::error::MagiError;
use std::env;

/// Configuration for an individual LLM node persona.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
}

/// Global system configuration loaded dynamically from environment or flags.
#[derive(Debug, Clone)]
pub struct MagiConfig {
    pub spacetimedb_uri: String,
    pub spacetimedb_database: String,
    pub timeout_seconds: u64,
    pub author: String,
    pub melchior: NodeConfig,
    pub balthasar: NodeConfig,
    pub casper: NodeConfig,
}

impl MagiConfig {
    /// Loads configuration from environment variables with dynamic provider resolution.
    ///
    /// # Errors
    /// Returns a [`MagiError::Config`] if critical validation fails.
    pub fn from_env() -> Result<Self, MagiError> {
        let _ = dotenvy::dotenv();

        let spacetimedb_uri =
            env::var("SPACETIMEDB_URI").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
        let spacetimedb_database =
            env::var("SPACETIMEDB_DATABASE").unwrap_or_else(|_| "magi-system".to_string());

        let timeout_seconds = env::var("MAGI_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60);

        let author = env::var("MAGI_AUTHOR")
            .or_else(|_| env::var("USER"))
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "developer@magi".to_string());

        // Infer system default provider and model from available keys
        let (default_provider, default_model) = Self::infer_default_provider_and_model();

        let melchior = Self::resolve_node_config("MELCHIOR", &default_provider, &default_model);
        let balthasar = Self::resolve_node_config("BALTHASAR", &default_provider, &default_model);
        let casper = Self::resolve_node_config("CASPER", &default_provider, &default_model);

        Ok(Self {
            spacetimedb_uri,
            spacetimedb_database,
            timeout_seconds,
            author,
            melchior,
            balthasar,
            casper,
        })
    }

    /// Infers the default provider and model based on configured API keys in environment.
    fn infer_default_provider_and_model() -> (String, String) {
        if let Ok(p) = env::var("MAGI_PROVIDER").or_else(|_| env::var("DEFAULT_PROVIDER")) {
            let model = env::var("MAGI_MODEL")
                .or_else(|_| env::var("DEFAULT_MODEL"))
                .unwrap_or_else(|_| Self::default_model_for_provider(&p));
            return (p.to_lowercase(), model);
        }

        if env::var("GEMINI_API_KEY").is_ok() || env::var("GOOGLE_API_KEY").is_ok() {
            ("gemini".to_string(), "gemini-2.5-flash".to_string())
        } else if env::var("OPENAI_API_KEY").is_ok() {
            ("openai".to_string(), "gpt-4o".to_string())
        } else if env::var("ANTHROPIC_API_KEY").is_ok() {
            (
                "anthropic".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
            )
        } else if env::var("GROK_API_KEY").is_ok() || env::var("XAI_API_KEY").is_ok() {
            ("grok".to_string(), "grok-2".to_string())
        } else if env::var("DEEPSEEK_API_KEY").is_ok() {
            ("deepseek".to_string(), "deepseek-chat".to_string())
        } else if env::var("OLLAMA_ENDPOINT").is_ok() || env::var("OLLAMA_BASE_URL").is_ok() {
            ("ollama".to_string(), "llama3".to_string())
        } else {
            ("mock".to_string(), "mock-v1".to_string())
        }
    }

    /// Returns the canonical default model for a given provider name.
    pub fn default_model_for_provider(provider: &str) -> String {
        match provider.to_lowercase().as_str() {
            "gemini" => "gemini-2.5-flash".to_string(),
            "openai" => "gpt-4o".to_string(),
            "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
            "grok" | "xai" => "grok-2".to_string(),
            "deepseek" => "deepseek-chat".to_string(),
            "ollama" => "llama3".to_string(),
            _ => "mock-v1".to_string(),
        }
    }

    /// Returns the canonical default endpoint for a given provider name.
    pub fn default_endpoint_for_provider(provider: &str) -> String {
        match provider.to_lowercase().as_str() {
            "gemini" => "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            "anthropic" => "https://api.anthropic.com".to_string(),
            "grok" | "xai" => "https://api.x.ai/v1".to_string(),
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            "ollama" => "http://localhost:11434".to_string(),
            "mock" => "http://127.0.0.1:0".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        }
    }

    /// Resolves configuration for a specific MAGI persona module.
    pub fn resolve_node_config(
        module_prefix: &str,
        default_provider: &str,
        default_model: &str,
    ) -> NodeConfig {
        let prefix = module_prefix.to_uppercase();

        let provider = env::var(format!("{}_PROVIDER", prefix))
            .unwrap_or_else(|_| default_provider.to_string())
            .to_lowercase();

        let model = env::var(format!("{}_MODEL", prefix))
            .or_else(|_| env::var("MAGI_MODEL"))
            .unwrap_or_else(|_| default_model.to_string());

        let api_key =
            env::var(format!("{}_API_KEY", prefix))
                .ok()
                .or_else(|| match provider.as_str() {
                    "gemini" => env::var("GEMINI_API_KEY")
                        .ok()
                        .or_else(|| env::var("GOOGLE_API_KEY").ok()),
                    "openai" => env::var("OPENAI_API_KEY").ok(),
                    "anthropic" => env::var("ANTHROPIC_API_KEY").ok(),
                    "grok" | "xai" => env::var("GROK_API_KEY")
                        .ok()
                        .or_else(|| env::var("XAI_API_KEY").ok()),
                    "deepseek" => env::var("DEEPSEEK_API_KEY").ok(),
                    "ollama" => env::var("OLLAMA_API_KEY").ok(),
                    _ => None,
                });

        let base_url = env::var(format!("{}_ENDPOINT", prefix))
            .ok()
            .or_else(|| env::var(format!("{}_BASE_URL", prefix)).ok())
            .unwrap_or_else(|| Self::default_endpoint_for_provider(&provider));

        NodeConfig {
            provider,
            model,
            api_key,
            base_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model_and_endpoint_resolution() {
        assert_eq!(
            MagiConfig::default_model_for_provider("gemini"),
            "gemini-2.5-flash"
        );
        assert_eq!(
            MagiConfig::default_endpoint_for_provider("gemini"),
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
        assert_eq!(
            MagiConfig::default_endpoint_for_provider("anthropic"),
            "https://api.anthropic.com"
        );
    }
}
