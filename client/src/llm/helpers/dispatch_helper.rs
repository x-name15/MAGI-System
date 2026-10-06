//! # Universal LLM HTTP Dispatch Helper
//!
//! Handles standard OpenAI-compatible Chat Completions HTTP requests for all LLM providers
//! (OpenAI, local models like Ollama/vLLM/LM Studio, or custom gateways).

use super::parser_helper::parse_llm_json_response;
use crate::error::MagiError;
use crate::i18n::{self, Language};
use crate::llm::NodeEvaluation;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

/// Universal HTTP dispatcher that routes queries to any OpenAI-compatible endpoint.
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_llm_request(
    client: &Client,
    node_id: &str,
    _provider: &str,
    base_url: &str,
    model: &str,
    api_key: Option<&str>,
    system_prompt: &str,
    user_prompt: &str,
    context_payload: &str,
) -> Result<NodeEvaluation, MagiError> {
    let start_time = Instant::now();
    let bundle = i18n::get_bundle(Language::detect(user_prompt));
    let prompt_format = bundle
        .prompt
        .format_audit_prompt(user_prompt, context_payload);

    let url = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let request_body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": prompt_format }
        ],
        "response_format": { "type": "json_object" }
    });

    let mut req = client
        .post(&url)
        .header("content-type", "application/json")
        .header("HTTP-Referer", "https://github.com/x-name15/magi-system")
        .header("X-Title", "MAGI System");

    if let Some(key) = api_key {
        if !key.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
    }

    let response = req
        .json(&request_body)
        .send()
        .await
        .map_err(|e| MagiError::Provider {
            node: node_id.to_string(),
            message: format!("HTTP request to {} failed: {}", url, e),
        })?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(MagiError::Provider {
            node: node_id.to_string(),
            message: format!("LLM API at {} returned error: {}", url, err_text),
        });
    }

    let response_json: serde_json::Value =
        response.json().await.map_err(|e| MagiError::Provider {
            node: node_id.to_string(),
            message: format!("Failed to parse response JSON: {}", e),
        })?;

    let response_text = response_json["choices"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .ok_or_else(|| MagiError::Provider {
            node: node_id.to_string(),
            message: "No content found in completion choices".to_string(),
        })?
        .to_string();

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
