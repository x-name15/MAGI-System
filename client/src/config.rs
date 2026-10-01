//! # Dynamic Configuration Loader for MAGI System

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
            .unwrap_or_else(|_| "auditor@magi".to_string());

        // Dynamic resolution for each of the three MAGI persona modules:
        let melchior =
            Self::resolve_node_config("MELCHIOR", "anthropic", "claude-3-5-sonnet-20241022");
        let balthasar = Self::resolve_node_config("BALTHASAR", "openai", "gpt-4o");
        let casper = Self::resolve_node_config("CASPER", "ollama", "llama3");

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

    /// Resolves configuration for a specific MAGI persona module.
    /// Supports assigning ANY provider (OpenAI, Gemini, Grok, Anthropic, DeepSeek, Ollama)
    /// to ANY persona module (Melchior, Balthasar, Casper).
    pub fn resolve_node_config(
        module_prefix: &str,
        default_provider: &str,
        default_model: &str,
    ) -> NodeConfig {
        let prefix = module_prefix.to_uppercase();

        let provider = env::var(format!("{}_PROVIDER", prefix))
            .unwrap_or_else(|_| default_provider.to_string())
            .to_lowercase();

        let model =
            env::var(format!("{}_MODEL", prefix)).unwrap_or_else(|_| default_model.to_string());

        // Dedicated node-level API key takes priority over provider-level API key
        let api_key =
            env::var(format!("{}_API_KEY", prefix))
                .ok()
                .or_else(|| match provider.as_str() {
                    "anthropic" => env::var("ANTHROPIC_API_KEY").ok(),
                    "openai" => env::var("OPENAI_API_KEY").ok(),
                    "gemini" => env::var("GEMINI_API_KEY")
                        .ok()
                        .or_else(|| env::var("GOOGLE_API_KEY").ok()),
                    "grok" | "xai" => env::var("GROK_API_KEY")
                        .ok()
                        .or_else(|| env::var("XAI_API_KEY").ok()),
                    "deepseek" => env::var("DEEPSEEK_API_KEY").ok(),
                    "ollama" => env::var("OLLAMA_API_KEY").ok(),
                    _ => None,
                });

        // Resolve base URL / endpoint: node-level override > provider-level override > canonical default
        let base_url = env::var(format!("{}_ENDPOINT", prefix))
            .ok()
            .or_else(|| env::var(format!("{}_BASE_URL", prefix)).ok())
            .or_else(|| match provider.as_str() {
                "anthropic" => env::var("ANTHROPIC_BASE_URL")
                    .ok()
                    .or_else(|| env::var("ANTHROPIC_ENDPOINT").ok()),
                "openai" => env::var("OPENAI_BASE_URL")
                    .ok()
                    .or_else(|| env::var("OPENAI_ENDPOINT").ok()),
                "gemini" => env::var("GEMINI_BASE_URL")
                    .ok()
                    .or_else(|| env::var("GEMINI_ENDPOINT").ok()),
                "grok" | "xai" => env::var("GROK_BASE_URL")
                    .ok()
                    .or_else(|| env::var("GROK_ENDPOINT").ok()),
                "deepseek" => env::var("DEEPSEEK_BASE_URL")
                    .ok()
                    .or_else(|| env::var("DEEPSEEK_ENDPOINT").ok()),
                "ollama" => env::var("OLLAMA_BASE_URL")
                    .ok()
                    .or_else(|| env::var("OLLAMA_ENDPOINT").ok()),
                _ => None,
            })
            .unwrap_or_else(|| match provider.as_str() {
                "anthropic" => "https://api.anthropic.com".to_string(),
                "gemini" => "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
                "grok" | "xai" => "https://api.x.ai/v1".to_string(),
                "deepseek" => "https://api.deepseek.com/v1".to_string(),
                "ollama" => "http://localhost:11434".to_string(),
                _ => "https://api.openai.com/v1".to_string(),
            });

        // If provider wasn't explicitly given, auto-infer from endpoint
        let resolved_provider = if env::var(format!("{}_PROVIDER", prefix)).is_err() {
            if base_url.contains("anthropic.com") {
                "anthropic".to_string()
            } else if base_url.contains("11434") {
                "ollama".to_string()
            } else {
                provider
            }
        } else {
            provider
        };

        NodeConfig {
            provider: resolved_provider,
            model,
            api_key,
            base_url,
        }
    }
}
