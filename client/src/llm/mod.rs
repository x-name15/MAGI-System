//! # LLM Provider Abstractions and Personas
//!
//! MAGI System Node Modules:
//! - Melchior-1 (The Scientist: logic, architecture, algorithms)
//! - Balthasar-2 (The Mother: security, defensive engineering, risk, veto)
//! - Casper-3 (The Woman: pragmatism, developer experience, viability)

pub mod balthasar;
pub mod casper;
pub mod melchior;
pub mod mock;

pub use balthasar::BalthasarNode;
pub use casper::CasperNode;
pub use melchior::MelchiorNode;

use crate::error::MagiError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;

/// A single structured finding discovered during node audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub recommendation: String,
}

/// Structured response emitted by a MAGI node evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeEvaluation {
    pub node_id: String,
    pub vote: String,
    pub risk_score: u8,
    #[serde(default)]
    pub findings: Vec<Finding>,
    pub rationale: String,
    pub argument: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    pub cwe_flags: Vec<String>,
    pub execution_time_ms: u32,
    #[serde(default)]
    pub prompt_version: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub initial_argument: Option<String>,
    #[serde(default)]
    pub initial_vote: Option<String>,
    #[serde(default)]
    pub initial_risk_score: Option<u8>,
}

fn default_confidence() -> f32 {
    0.95
}

/// Determines if a text contains predominantly Spanish tokens.
pub fn is_spanish_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    let spanish_tokens = [
        " de ",
        " la ",
        " el ",
        " en ",
        " que ",
        " los ",
        " las ",
        " por ",
        " un ",
        " una ",
        " con ",
        " para ",
        " este ",
        " esta ",
        " como ",
        " evalua ",
        " audita ",
        " seguridad ",
        " propuesta ",
        " viabilidad ",
        " riesgo ",
        " concurrencia ",
        " arquitectura ",
        " solución ",
        " función ",
    ];
    spanish_tokens.iter().any(|t| lower.contains(t))
}

/// Raw JSON output expected from LLM completion.
#[derive(Debug, Deserialize)]
pub struct RawNodeOutput {
    pub vote: String,
    pub risk_score: u8,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(default, alias = "argument")]
    pub rationale: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub cwe_flags: Vec<String>,
}

/// Common trait for all MAGI node modules and providers.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Evaluates code context under the node's analytical lens.
    async fn evaluate(
        &self,
        node_id: &str,
        system_prompt: &str,
        user_prompt: &str,
        context_payload: &str,
    ) -> Result<NodeEvaluation, MagiError>;
}

fn repair_json_text(text: &str) -> String {
    let mut repaired_lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        // Check for missing opening quote on a key: e.g. `key":` instead of `"key":`
        if let Some(colon_pos) = trimmed.find("\":") {
            let candidate_key = &trimmed[..colon_pos];
            if !candidate_key.starts_with('"')
                && !candidate_key.contains(' ')
                && !candidate_key.is_empty()
            {
                let rest = &trimmed[colon_pos..];
                repaired_lines.push(format!("{}\"{}{}", indent, candidate_key, rest));
                continue;
            }
        }
        repaired_lines.push(line.to_string());
    }
    let combined = repaired_lines.join("\n");
    // Remove trailing commas before closing braces
    combined
        .replace(",\n}", "\n}")
        .replace(",\r\n}", "\r\n}")
        .replace(",\n  }", "\n  }")
        .replace(",\r\n  }", "\r\n  }")
        .replace(",\n    }", "\n    }")
        .replace(",\n]", "\n]")
        .replace(",\r\n]", "\r\n]")
        .replace(",\n  ]", "\n  ]")
}

/// Helper function to parse JSON response from an LLM output string.
pub fn parse_llm_json_response(raw_text: &str) -> Result<RawNodeOutput, MagiError> {
    let cleaned = if let Some(start) = raw_text.find("```json") {
        let after_start = &raw_text[start + 7..];
        if let Some(end) = after_start.find("```") {
            &after_start[..end]
        } else {
            after_start
        }
    } else if let Some(start) = raw_text.find('{') {
        if let Some(end) = raw_text.rfind('}') {
            &raw_text[start..=end]
        } else {
            raw_text
        }
    } else {
        raw_text
    };

    let trimmed = cleaned.trim();
    if let Ok(parsed) = serde_json::from_str::<RawNodeOutput>(trimmed) {
        return Ok(parsed);
    }

    // Try repairing common LLM formatting glitches (missing quotes on keys, trailing commas)
    let repaired = repair_json_text(trimmed);
    serde_json::from_str::<RawNodeOutput>(&repaired).map_err(|e| MagiError::Provider {
        node: "JSON_PARSER".to_string(),
        message: format!(
            "Failed to parse LLM structured JSON response: {}. Content: {}",
            e, raw_text
        ),
    })
}

/// Universal HTTP dispatcher that routes queries to OpenAI-compatible, Anthropic, or Ollama endpoints.
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_llm_request(
    client: &Client,
    node_id: &str,
    provider: &str,
    base_url: &str,
    model: &str,
    api_key: Option<&str>,
    system_prompt: &str,
    user_prompt: &str,
    context_payload: &str,
) -> Result<NodeEvaluation, MagiError> {
    let start_time = Instant::now();
    let prompt_format = format!(
        "AUDIT QUERY:\n{}\n\nCODE CONTEXT:\n{}\n\n\
        LANGUAGE INSTRUCTION:\n\
        Respond in the SAME language as the query and context. If the query or context is in Spanish, write all your analysis, rationale, findings titles, impacts, and recommendations in Spanish. If in English, write in English.\n\n\
        Respond ONLY with a valid JSON object matching:\n\
        {{\n  \"vote\": \"APPROVE\" | \"REJECT\" | \"NEUTRAL\",\n  \"risk_score\": <number 1-10>,\n  \"confidence\": <float 0.0-1.0>,\n  \"findings\": [\n    {{\n      \"category\": \"<architecture | security | pragmatism | ...>\",\n      \"severity\": \"<critical | high | medium | low | info>\",\n      \"title\": \"<short summary>\",\n      \"evidence\": \"<exact code snippet or pattern>\",\n      \"impact\": \"<concrete consequence>\",\n      \"recommendation\": \"<actionable technical fix>\"\n    }}\n  ],\n  \"rationale\": \"<concise analytical explanation>\",\n  \"cwe_flags\": [\"CWE-...\"]\n}}",
        user_prompt, context_payload
    );

    let norm_provider = provider.to_lowercase();
    let response_text = match norm_provider.as_str() {
        "anthropic" => {
            let key = api_key.ok_or_else(|| {
                MagiError::Config(format!("Missing API key for node {}", node_id))
            })?;
            let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
            let request_body = json!({
                "model": model,
                "max_tokens": 2048,
                "system": system_prompt,
                "messages": [{ "role": "user", "content": prompt_format }]
            });

            let response = client
                .post(&url)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Anthropic HTTP request failed: {}", e),
                })?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Anthropic API returned error: {}", err_text),
                });
            }

            let response_json: serde_json::Value =
                response.json().await.map_err(|e| MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Failed to parse Anthropic JSON: {}", e),
                })?;

            response_json["content"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|item| item["text"].as_str())
                .ok_or_else(|| MagiError::Provider {
                    node: node_id.to_string(),
                    message: "No text found in Anthropic response".to_string(),
                })?
                .to_string()
        }

        "ollama" => {
            let url = format!("{}/api/generate", base_url.trim_end_matches('/'));
            let request_body = json!({
                "model": model,
                "system": system_prompt,
                "prompt": prompt_format,
                "format": "json",
                "stream": false
            });

            let response = client
                .post(&url)
                .json(&request_body)
                .send()
                .await
                .map_err(|e| MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Ollama HTTP request failed: {}", e),
                })?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Ollama API returned error: {}", err_text),
                });
            }

            let response_json: serde_json::Value =
                response.json().await.map_err(|e| MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Failed to parse Ollama JSON: {}", e),
                })?;

            response_json["response"]
                .as_str()
                .ok_or_else(|| MagiError::Provider {
                    node: node_id.to_string(),
                    message: "No 'response' found in Ollama output".to_string(),
                })?
                .to_string()
        }

        // Default: OpenAI-compatible (OpenAI, Gemini v1beta/openai, Grok xAI, DeepSeek, etc.)
        _ => {
            let key = api_key.ok_or_else(|| {
                MagiError::Config(format!("Missing API key for node {}", node_id))
            })?;
            let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
            let request_body = json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": prompt_format }
                ],
                "response_format": { "type": "json_object" }
            });

            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", key))
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("OpenAI-compatible HTTP request failed: {}", e),
                })?;

            if !response.status().is_success() {
                let err_text = response.text().await.unwrap_or_default();
                return Err(MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("OpenAI-compatible API returned error: {}", err_text),
                });
            }

            let response_json: serde_json::Value =
                response.json().await.map_err(|e| MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!("Failed to parse response JSON: {}", e),
                })?;

            response_json["choices"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|choice| choice["message"]["content"].as_str())
                .ok_or_else(|| MagiError::Provider {
                    node: node_id.to_string(),
                    message: "No content found in completion choices".to_string(),
                })?
                .to_string()
        }
    };

    let parsed = parse_llm_json_response(&response_text)?;
    let execution_time_ms = start_time.elapsed().as_millis() as u32;

    let rationale = if !parsed.rationale.is_empty() {
        parsed.rationale
    } else {
        "Evaluated under node analytical lens".to_string()
    };

    let prompt_version = crate::skills::PromptLoader::compute_hash(system_prompt);

    Ok(NodeEvaluation {
        node_id: node_id.to_string(),
        vote: parsed.vote.trim().to_uppercase(),
        risk_score: parsed.risk_score.clamp(1, 10),
        findings: parsed.findings,
        argument: rationale.clone(),
        rationale,
        confidence: parsed.confidence.clamp(0.0, 1.0),
        cwe_flags: parsed.cwe_flags,
        execution_time_ms,
        prompt_version,
        model: model.to_string(),
        initial_argument: None,
        initial_vote: None,
        initial_risk_score: None,
    })
}
