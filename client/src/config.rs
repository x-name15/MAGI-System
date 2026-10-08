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
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub max_context_chars: usize,
    pub max_tokens: u32,
}

/// Global system configuration loaded dynamically from environment or flags.
#[derive(Debug, Clone)]
pub struct MagiConfig {
    pub spacetimedb_uri: String,
    pub spacetimedb_database: String,
    pub timeout_seconds: u64,
    pub author: String,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub allow_degraded_quorum: bool,
    pub max_context_chars: usize,
    #[allow(dead_code)]
    pub max_tokens: u32,
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

        let max_retries = env::var("MAGI_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);

        let retry_delay_ms = env::var("MAGI_RETRY_DELAY_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1000);

        let allow_degraded_quorum = env::var("MAGI_ALLOW_DEGRADED_QUORUM")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        let max_context_chars = env::var("MAGI_MAX_CONTEXT_CHARS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(60_000);

        let max_tokens = env::var("MAGI_MAX_TOKENS")
            .or_else(|_| env::var("LLM_MAX_TOKENS"))
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(4096);

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
            max_retries,
            retry_delay_ms,
            max_context_chars,
            max_tokens,
        );
        let balthasar = Self::resolve_node_config(
            "BALTHASAR",
            &default_provider,
            &default_model,
            &default_endpoint,
            default_api_key.as_deref(),
            max_retries,
            retry_delay_ms,
            max_context_chars,
            max_tokens,
        );
        let casper = Self::resolve_node_config(
            "CASPER",
            &default_provider,
            &default_model,
            &default_endpoint,
            default_api_key.as_deref(),
            max_retries,
            retry_delay_ms,
            max_context_chars,
            max_tokens,
        );

        Ok(Self {
            spacetimedb_uri,
            spacetimedb_database,
            timeout_seconds,
            author,
            max_retries,
            retry_delay_ms,
            allow_degraded_quorum,
            max_context_chars,
            max_tokens,
            melchior,
            balthasar,
            casper,
        })
    }

    /// Resolves configuration for a specific MAGI persona node, checking node-specific
    /// variables first, then falling back to system defaults.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_node_config(
        module_prefix: &str,
        default_provider: &str,
        default_model: &str,
        default_endpoint: &str,
        default_api_key: Option<&str>,
        max_retries: u32,
        retry_delay_ms: u64,
        max_context_chars: usize,
        default_max_tokens: u32,
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

        let max_tokens = env::var(format!("{}_MAX_TOKENS", prefix))
            .or_else(|_| env::var("MAGI_MAX_TOKENS"))
            .or_else(|_| env::var("LLM_MAX_TOKENS"))
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(default_max_tokens);

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
            max_retries,
            retry_delay_ms,
            max_context_chars,
            max_tokens,
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
            max_retries: 3,
            retry_delay_ms: 1000,
            allow_degraded_quorum: true,
            max_context_chars: 60_000,
            max_tokens: 4096,
            melchior: NodeConfig {
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                api_key: None,
                base_url: "http://127.0.0.1:0".to_string(),
                max_retries: 3,
                retry_delay_ms: 1000,
                max_context_chars: 60_000,
                max_tokens: 4096,
            },
            balthasar: NodeConfig {
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                api_key: None,
                base_url: "http://127.0.0.1:0".to_string(),
                max_retries: 3,
                retry_delay_ms: 1000,
                max_context_chars: 60_000,
                max_tokens: 4096,
            },
            casper: NodeConfig {
                provider: "mock".to_string(),
                model: "mock-v1".to_string(),
                api_key: None,
                base_url: "http://127.0.0.1:0".to_string(),
                max_retries: 3,
                retry_delay_ms: 1000,
                max_context_chars: 60_000,
                max_tokens: 4096,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_agnostic_env_resolution() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::set_var("MAGI_PROVIDER", "openai-compatible");
        env::set_var("MAGI_MODEL", "qwen2.5-coder:7b");
        env::set_var("MAGI_ENDPOINT", "http://localhost:8000/v1");

        let node = MagiConfig::resolve_node_config(
            "TEST_CUSTOM",
            "openai-compatible",
            "qwen2.5-coder:7b",
            "http://localhost:8000/v1",
            None,
            3,
            1000,
            60_000,
            4096,
        );
        assert_eq!(node.provider, "openai-compatible");
        assert_eq!(node.model, "qwen2.5-coder:7b");
        assert_eq!(node.base_url, "http://localhost:8000/v1");
        assert_eq!(node.max_retries, 3);
        assert_eq!(node.retry_delay_ms, 1000);
        assert_eq!(node.max_context_chars, 60_000);
        assert_eq!(node.max_tokens, 4096);

        env::remove_var("MAGI_PROVIDER");
        env::remove_var("MAGI_MODEL");
        env::remove_var("MAGI_ENDPOINT");
    }

    #[test]
    fn test_node_specific_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::set_var("BALTHASAR_MODEL", "deepseek-r1");
        env::set_var("BALTHASAR_ENDPOINT", "http://sec-cluster:8000/v1");

        let node = MagiConfig::resolve_node_config(
            "BALTHASAR",
            "openai-compatible",
            "default",
            "http://localhost:11434/v1",
            None,
            3,
            1000,
            60_000,
            4096,
        );
        assert_eq!(node.model, "deepseek-r1");
        assert_eq!(node.base_url, "http://sec-cluster:8000/v1");

        env::remove_var("BALTHASAR_MODEL");
        env::remove_var("BALTHASAR_ENDPOINT");
    }

    #[test]
    fn test_openrouter_auto_resolution() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::remove_var("MAGI_ENDPOINT");
        env::remove_var("LLM_ENDPOINT");
        env::set_var("TEST_OR_API_KEY", "sk-or-v1-melchior-test-key");
        env::set_var("TEST_OR_MODEL", "anthropic/claude-3.5-sonnet");

        let node = MagiConfig::resolve_node_config(
            "TEST_OR",
            "openai-compatible",
            "default",
            "http://localhost:11434/v1",
            None,
            3,
            1000,
            60_000,
            4096,
        );

        assert_eq!(node.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(node.model, "anthropic/claude-3.5-sonnet");
        assert_eq!(node.api_key.as_deref(), Some("sk-or-v1-melchior-test-key"));
        assert_eq!(node.max_tokens, 4096);

        env::remove_var("TEST_OR_API_KEY");
        env::remove_var("TEST_OR_MODEL");
    }

    #[test]
    fn test_max_tokens_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        env::set_var("CASPER_MAX_TOKENS", "8192");

        let node = MagiConfig::resolve_node_config(
            "CASPER",
            "openai-compatible",
            "default",
            "http://localhost:11434/v1",
            None,
            3,
            1000,
            60_000,
            4096,
        );
        assert_eq!(node.max_tokens, 8192);

        env::remove_var("CASPER_MAX_TOKENS");
    }
}
