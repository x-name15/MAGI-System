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
    max_retries: u32,
    initial_delay_ms: u64,
    max_context_chars: usize,
) -> Result<NodeEvaluation, MagiError> {
    let start_time = Instant::now();

    // Context size safeguard: if payload is gigantic, apply head-tail truncation
    let effective_context = if context_payload.len() > max_context_chars {
        let keep_head = (max_context_chars as f64 * 0.6) as usize;
        let keep_tail = max_context_chars.saturating_sub(keep_head);
        let truncated_count = context_payload.len().saturating_sub(keep_head + keep_tail);
        format!(
            "{}\n\n[... TRUNCATED {} CHARACTERS BY MAGI CONTEXT SAFEGUARD ...]\n\n{}",
            &context_payload[..keep_head],
            truncated_count,
            &context_payload[context_payload.len().saturating_sub(keep_tail)..]
        )
    } else {
        context_payload.to_string()
    };

    let bundle = i18n::get_bundle(Language::detect(user_prompt));
    let prompt_format = bundle
        .prompt
        .format_audit_prompt(user_prompt, &effective_context);

    let url = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let mut use_json_format = true;
    let mut last_err = String::new();

    for attempt in 0..=max_retries {
        let request_body = if use_json_format {
            json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": &prompt_format }
                ],
                "response_format": { "type": "json_object" }
            })
        } else {
            json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": &prompt_format }
                ]
            })
        };

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

        let send_result = req.json(&request_body).send().await;

        let response = match send_result {
            Ok(res) => res,
            Err(e) => {
                last_err = format!("HTTP transport error: {}", e);
                if attempt < max_retries {
                    let delay = initial_delay_ms * 2u64.pow(attempt);
                    eprintln!(
                        "[{}] Transient network error, retrying in {}ms (attempt {}/{}): {}",
                        node_id,
                        delay,
                        attempt + 1,
                        max_retries,
                        e
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    continue;
                }
                return Err(MagiError::Provider {
                    node: node_id.to_string(),
                    message: format!(
                        "HTTP request to {} failed after {} retries: {}",
                        url, max_retries, e
                    ),
                });
            }
        };

        let status = response.status();

        if status.is_success() {
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

            return Ok(NodeEvaluation {
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
            });
        }

        let retry_after_header = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        let err_text = response.text().await.unwrap_or_default();

        // Check if 400 is caused by response_format rejection
        if status.as_u16() == 400
            && use_json_format
            && (err_text.contains("response_format") || err_text.contains("json_object"))
        {
            eprintln!(
                "[{}] Model does not support response_format: json_object. Falling back to freeform text extraction.",
                node_id
            );
            use_json_format = false;
            continue;
        }

        // Clean user-friendly message from JSON error if possible
        let error_message =
            if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&err_text) {
                if let Some(msg) = err_json["error"]["message"].as_str() {
                    format!("{} (HTTP {})", msg, status)
                } else {
                    format!("HTTP {}: {}", status, err_text)
                }
            } else {
                format!("HTTP {}: {}", status, err_text)
            };

        last_err = error_message.clone();

        // Check if transient error (429, 500, 502, 503, 504)
        let is_transient = status.as_u16() == 429 || status.is_server_error();
        if is_transient && attempt < max_retries {
            let delay = retry_after_header
                .map(|secs| secs * 1000)
                .unwrap_or_else(|| initial_delay_ms * 2u64.pow(attempt));
            eprintln!(
                "[{}] Rate limit / server error ({}), retrying in {}ms (attempt {}/{})...",
                node_id,
                status,
                delay,
                attempt + 1,
                max_retries
            );
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            continue;
        }

        return Err(MagiError::Provider {
            node: node_id.to_string(),
            message: format!("LLM API at {} returned error: {}", url, last_err),
        });
    }

    Err(MagiError::Provider {
        node: node_id.to_string(),
        message: format!(
            "LLM API at {} failed after {} retries: {}",
            url, max_retries, last_err
        ),
    })
}
