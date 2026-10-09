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

/// Configuration for OpenRouter server tools (subagents and web search).
#[derive(Debug, Clone, Default)]
pub struct ServerToolsConfig {
    pub subagent_model: Option<String>,
    pub subagent_web_search: bool,
    pub enable_web_search: bool,
}

/// Builds OpenRouter server tools definitions (subagents, web search) for the request payload.
pub fn build_openrouter_tools(
    node_id: &str,
    current_model: &str,
    tools_cfg: &ServerToolsConfig,
) -> Vec<serde_json::Value> {
    let mut tools = Vec::new();

    // 1. Direct web search tool if requested
    if tools_cfg.enable_web_search {
        tools.push(json!({
            "type": "openrouter:web_search"
        }));
    }

    // 2. Specialized subagent worker tool if configured
    if let Some(ref worker_model) = tools_cfg.subagent_model {
        let worker_model_clean = worker_model.trim();
        // Guard against self-reference cycle (OpenRouter rejects worker == outer model)
        if !worker_model_clean.is_empty() && worker_model_clean != current_model {
            let (subagent_name, role_instructions) = match node_id {
                "Melchior-1" => (
                    "systems_analyst",
                    "You are a specialized analytical worker for Melchior-1. Rapidly analyze algorithmic structures, data flow, concurrency patterns, complexity (O(N)), and technical trade-offs. Be concise, rigorous, and direct."
                ),
                "Balthasar-2" => (
                    "security_scanner",
                    "You are a specialized security audit worker for Balthasar-2. Inspect input code or architecture for CWE vulnerabilities, sanitization issues, injection vectors, and attack surfaces. Be vigilant, concrete, and rigorous."
                ),
                "Casper-3" => (
                    "pragmatic_evaluator",
                    "You are a specialized pragmatic engineering worker for Casper-3. Assess developer ergonomics, operational complexity, cognitive load, migration friction, and delivery feasibility. Be succinct, realistic, and practical."
                ),
                _ => (
                    "subagent_worker",
                    "You are a fast, focused worker. Complete the delegated task thoroughly and concisely."
                ),
            };

            let mut worker_params = json!({
                "name": subagent_name,
                "model": worker_model_clean,
                "instructions": role_instructions
            });

            if tools_cfg.subagent_web_search {
                worker_params["tools"] = json!([
                    { "type": "openrouter:web_search" }
                ]);
            }

            tools.push(json!({
                "type": "openrouter:subagent",
                "parameters": worker_params
            }));
        }
    }

    tools
}

/// Universal HTTP dispatcher that routes queries to any OpenAI-compatible endpoint.
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_llm_request(
    client: &Client,
    node_id: &str,
    _provider: &str,
    base_url: &str,
    model: &str,
    tools_cfg: Option<&ServerToolsConfig>,
    api_key: Option<&str>,
    system_prompt: &str,
    user_prompt: &str,
    context_payload: &str,
    max_retries: u32,
    initial_delay_ms: u64,
    max_context_chars: usize,
    max_tokens: u32,
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

    let is_openrouter =
        base_url.contains("openrouter.ai") || _provider.to_lowercase().contains("openrouter");
    let server_tools = if is_openrouter {
        tools_cfg
            .map(|cfg| build_openrouter_tools(node_id, model, cfg))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut use_tools = !server_tools.is_empty();
    let mut use_json_format = true;
    let mut last_err = String::new();

    for attempt in 0..=max_retries {
        let mut request_body = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": &prompt_format }
            ],
            "max_tokens": max_tokens
        });

        if use_json_format {
            request_body["response_format"] = json!({ "type": "json_object" });
        }

        if use_tools && !server_tools.is_empty() {
            request_body["tools"] = json!(server_tools);
            if attempt == 0 {
                if let Some(cfg) = tools_cfg {
                    if let Some(ref m) = cfg.subagent_model {
                        eprintln!(
                            "[{}] Equipped OpenRouter server tools (subagent worker: {}, web_search: {})",
                            node_id, m, cfg.subagent_web_search || cfg.enable_web_search
                        );
                    }
                }
            }
        }

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

            if let Some(choices) = response_json["choices"].as_array() {
                if let Some(choice) = choices.first() {
                    let tool_calls = choice["message"]["tool_calls"]
                        .as_array()
                        .or_else(|| choice["tool_calls"].as_array());
                    if let Some(calls) = tool_calls {
                        for tc in calls {
                            let name = tc["function"]["name"].as_str().unwrap_or("worker");
                            let args_str = tc["function"]["arguments"].as_str().unwrap_or("");
                            let task = if let Ok(args_val) =
                                serde_json::from_str::<serde_json::Value>(args_str)
                            {
                                args_val["task_name"]
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            if !task.is_empty() {
                                eprintln!(
                                    "[{}] Subagent worker invoked: {} (task: {})",
                                    node_id, name, task
                                );
                            } else {
                                eprintln!("[{}] Server tool executed: {}", node_id, name);
                            }
                        }
                    }
                }
            }

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

            let rationale = if !parsed.rationale.trim().is_empty() {
                parsed.rationale
            } else {
                bundle.prompt.default_rationale.clone()
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

        // Check if 400 is caused by server tools rejection
        if status.as_u16() == 400
            && use_tools
            && (err_text.contains("tool")
                || err_text.contains("subagent")
                || err_text.contains("server_tool"))
        {
            eprintln!(
                "[{}] Provider rejected server tools ({}). Retrying request without server tools...",
                node_id, error_message
            );
            use_tools = false;
            continue;
        }

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

/// Dispatches an LLM evaluation with automatic failover to a contingency backup model
/// if the primary model fails (e.g. out of API credits, persistent 429/5xx errors).
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_llm_request_with_fallback(
    client: &Client,
    node_id: &str,
    provider: &str,
    base_url: &str,
    model: &str,
    fallback_model: Option<&str>,
    tools_cfg: Option<&ServerToolsConfig>,
    api_key: Option<&str>,
    system_prompt: &str,
    user_prompt: &str,
    context_payload: &str,
    max_retries: u32,
    initial_delay_ms: u64,
    max_context_chars: usize,
    max_tokens: u32,
) -> Result<NodeEvaluation, MagiError> {
    match dispatch_llm_request(
        client,
        node_id,
        provider,
        base_url,
        model,
        tools_cfg,
        api_key,
        system_prompt,
        user_prompt,
        context_payload,
        max_retries,
        initial_delay_ms,
        max_context_chars,
        max_tokens,
    )
    .await
    {
        Ok(eval) => Ok(eval),
        Err(err) => {
            if let Some(backup) = fallback_model {
                let backup_clean = backup.trim();
                if !backup_clean.is_empty() && backup_clean != model {
                    eprintln!(
                        "[{}] Primary model ({}) failed: {}. Engaging backup circuit ({})...",
                        node_id, model, err, backup_clean
                    );
                    match dispatch_llm_request(
                        client,
                        node_id,
                        provider,
                        base_url,
                        backup_clean,
                        tools_cfg,
                        api_key,
                        system_prompt,
                        user_prompt,
                        context_payload,
                        max_retries,
                        initial_delay_ms,
                        max_context_chars,
                        max_tokens,
                    )
                    .await
                    {
                        Ok(mut eval) => {
                            eval.model = format!("{} (backup)", backup_clean);
                            return Ok(eval);
                        }
                        Err(backup_err) => {
                            eprintln!(
                                "[{}] Backup circuit ({}) also failed: {}",
                                node_id, backup_clean, backup_err
                            );
                            return Err(backup_err);
                        }
                    }
                }
            }
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_openrouter_tools_subagent_specialization() {
        let cfg = ServerToolsConfig {
            subagent_model: Some("cohere/north-mini-code:free".to_string()),
            subagent_web_search: false,
            enable_web_search: false,
        };

        let tools_m = build_openrouter_tools("Melchior-1", "primary-model", &cfg);
        assert_eq!(tools_m.len(), 1);
        assert_eq!(tools_m[0]["type"], "openrouter:subagent");
        assert_eq!(tools_m[0]["parameters"]["name"], "systems_analyst");
        assert_eq!(
            tools_m[0]["parameters"]["model"],
            "cohere/north-mini-code:free"
        );

        let tools_b = build_openrouter_tools("Balthasar-2", "primary-model", &cfg);
        assert_eq!(tools_b[0]["parameters"]["name"], "security_scanner");

        let tools_c = build_openrouter_tools("Casper-3", "primary-model", &cfg);
        assert_eq!(tools_c[0]["parameters"]["name"], "pragmatic_evaluator");
    }

    #[test]
    fn test_build_openrouter_tools_prevents_self_reference() {
        let cfg = ServerToolsConfig {
            subagent_model: Some("cohere/north-mini-code:free".to_string()),
            subagent_web_search: false,
            enable_web_search: false,
        };

        // When the model executing is the same as the subagent worker model
        let tools = build_openrouter_tools("Casper-3", "cohere/north-mini-code:free", &cfg);
        assert!(tools.is_empty(), "Subagent must not be attached to itself");
    }

    #[test]
    fn test_build_openrouter_tools_web_search() {
        let cfg = ServerToolsConfig {
            subagent_model: Some("cohere/north-mini-code:free".to_string()),
            subagent_web_search: true,
            enable_web_search: true,
        };

        let tools = build_openrouter_tools("Balthasar-2", "primary-model", &cfg);
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["type"], "openrouter:web_search");
        assert_eq!(tools[1]["type"], "openrouter:subagent");
        assert_eq!(
            tools[1]["parameters"]["tools"][0]["type"],
            "openrouter:web_search"
        );
    }
}
