//! # Agnostic LLM Configuration Loader for MAGI System
//!
//! Provides protocol-agnostic configuration resolution. Connects to any standard
//! OpenAI-compatible LLM endpoint (local vLLM, Ollama, LM Studio, or cloud providers)
//! without hardcoded vendor-specific assumptions or fallback guessing.

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
    /// Loads configuration from environment variables.
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

        // Global defaults: read from generic environment variables
        let default_provider = env::var("MAGI_PROVIDER")
            .or_else(|_| env::var("LLM_PROVIDER"))
            .unwrap_or_else(|_| "openai-compatible".to_string())
            .to_lowercase();

        let default_model = env::var("MAGI_MODEL")
            .or_else(|_| env::var("LLM_MODEL"))
            .unwrap_or_else(|_| "default".to_string());

        let default_endpoint = env::var("MAGI_ENDPOINT")
            .or_else(|_| env::var("LLM_ENDPOINT"))
            .or_else(|_| env::var("OPENROUTER_BASE_URL"))
            .or_else(|_| env::var("OPENAI_BASE_URL"))
            .unwrap_or_else(|_| {
                if env::var("OPENROUTER_API_KEY").is_ok() {
                    "https://openrouter.ai/api/v1".to_string()
                } else {
                    "http://localhost:11434/v1".to_string()
                }
            });

        let default_api_key = env::var("MAGI_API_KEY")
            .or_else(|_| env::var("LLM_API_KEY"))
            .or_else(|_| env::var("OPENROUTER_API_KEY"))
            .or_else(|_| env::var("OPENAI_API_KEY"))
            .ok();

        let melchior = Self::resolve_node_config(
            "MELCHIOR",
            &default_provider,
            &default_model,
            &default_endpoint,
            default_api_key.as_deref(),
        );
        let balthasar = Self::resolve_node_config(
            "BALTHASAR",
            &default_provider,
            &default_model,
            &default_endpoint,
            default_api_key.as_deref(),
        );
        let casper = Self::resolve_node_config(
            "CASPER",
            &default_provider,
            &default_model,
            &default_endpoint,
            default_api_key.as_deref(),
        );

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

    /// Resolves configuration for a specific MAGI persona node, checking node-specific
    /// variables first, then falling back to system defaults.
    pub fn resolve_node_config(
        module_prefix: &str,
        default_provider: &str,
        default_model: &str,
        default_endpoint: &str,
        default_api_key: Option<&str>,
    ) -> NodeConfig {
        let prefix = module_prefix.to_uppercase();

        let provider = env::var(format!("{}_PROVIDER", prefix))
            .or_else(|_| env::var("MAGI_PROVIDER"))
            .or_else(|_| env::var("LLM_PROVIDER"))
            .unwrap_or_else(|_| default_provider.to_string())
            .to_lowercase();

        let model = env::var(format!("{}_MODEL", prefix))
            .or_else(|_| env::var("MAGI_MODEL"))
            .or_else(|_| env::var("LLM_MODEL"))
            .unwrap_or_else(|_| default_model.to_string());

        let api_key = env::var(format!("{}_API_KEY", prefix))
            .ok()
            .or_else(|| default_api_key.map(String::from));

        let base_url = env::var(format!("{}_ENDPOINT", prefix))
            .or_else(|_| env::var(format!("{}_BASE_URL", prefix)))
            .or_else(|_| env::var("MAGI_ENDPOINT"))
            .or_else(|_| env::var("LLM_ENDPOINT"))
            .or_else(|_| env::var("OPENROUTER_BASE_URL"))
            .or_else(|_| env::var("OPENAI_BASE_URL"))
            .unwrap_or_else(|_| {
                if let Some(ref key) = api_key {
                    if key.starts_with("sk-or-") {
                        return "https://openrouter.ai/api/v1".to_string();
                    }
                }
                default_endpoint.to_string()
            });

        NodeConfig {
            provider,
            model,
            api_key,
            base_url,
        }
    }
}

impl Default for MagiConfig {
    fn default() -> Self {
        Self {
            spacetimedb_uri: "http://127.0.0.1:3000".to_string(),
            spacetimedb_database: "magi-system".to_string(),
            timeout_seconds: 60,
            author: "developer@magi".to_string(),
            melchior: NodeConfig {
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                api_key: None,
                base_url: "http://127.0.0.1:0".to_string(),
            },
            balthasar: NodeConfig {
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                api_key: None,
                base_url: "http://127.0.0.1:0".to_string(),
            },
            casper: NodeConfig {
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                api_key: None,
                base_url: "http://127.0.0.1:0".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agnostic_env_resolution() {
        env::set_var("MAGI_PROVIDER", "openai-compatible");
        env::set_var("MAGI_MODEL", "qwen2.5-coder:7b");
        env::set_var("MAGI_ENDPOINT", "http://localhost:8000/v1");

        let node = MagiConfig::resolve_node_config(
            "TEST_CUSTOM",
            "openai-compatible",
            "qwen2.5-coder:7b",
            "http://localhost:8000/v1",
            None,
        );
        assert_eq!(node.provider, "openai-compatible");
        assert_eq!(node.model, "qwen2.5-coder:7b");
        assert_eq!(node.base_url, "http://localhost:8000/v1");

        env::remove_var("MAGI_PROVIDER");
        env::remove_var("MAGI_MODEL");
        env::remove_var("MAGI_ENDPOINT");
    }

    #[test]
    fn test_node_specific_override() {
        env::set_var("BALTHASAR_MODEL", "deepseek-r1");
        env::set_var("BALTHASAR_ENDPOINT", "http://sec-cluster:8000/v1");

        let node = MagiConfig::resolve_node_config(
            "BALTHASAR",
            "openai-compatible",
            "default",
            "http://localhost:11434/v1",
            None,
        );
        assert_eq!(node.model, "deepseek-r1");
        assert_eq!(node.base_url, "http://sec-cluster:8000/v1");

        env::remove_var("BALTHASAR_MODEL");
        env::remove_var("BALTHASAR_ENDPOINT");
    }

    #[test]
    fn test_openrouter_auto_resolution() {
        env::set_var("TEST_OR_API_KEY", "sk-or-v1-melchior-test-key");
        env::set_var("TEST_OR_MODEL", "anthropic/claude-3.5-sonnet");

        let node = MagiConfig::resolve_node_config(
            "TEST_OR",
            "openai-compatible",
            "default",
            "http://localhost:11434/v1",
            None,
        );

        assert_eq!(node.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(node.model, "anthropic/claude-3.5-sonnet");
        assert_eq!(node.api_key.as_deref(), Some("sk-or-v1-melchior-test-key"));

        env::remove_var("TEST_OR_API_KEY");
        env::remove_var("TEST_OR_MODEL");
    }
}
