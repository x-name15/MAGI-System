//! # Error handling for MAGI System

use thiserror::Error;

/// Central error type for all client and orchestrator operations.
#[derive(Error, Debug)]
pub enum MagiError {
    #[allow(dead_code)]
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("LLM Provider error [{node}]: {message}")]
    Provider { node: String, message: String },

    #[error("SpacetimeDB communication error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Deliberation timeout after {0} seconds")]
    Timeout(u64),
}
