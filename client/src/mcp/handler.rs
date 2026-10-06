//! # Model Context Protocol (MCP) Request Handler
//!
//! Handles MCP protocol requests including server initialization, tool discovery,
//! and executing the `deliberate_with_magi` tool against the MAGI Trinity consensus engine.

use crate::config::MagiConfig;
use crate::core::helpers::calculate_local_consensus;
use crate::core::MagiOrchestrator;
use crate::i18n::Language;
use crate::mcp::protocol::*;
use crate::ui::JsonOutput;
use serde_json::json;

/// Handler for Model Context Protocol requests.
pub struct McpHandler {
    config: MagiConfig,
    custom_skill: Option<String>,
    force_mock: bool,
}

impl McpHandler {
    /// Creates a new MCP handler.
    pub fn new(config: MagiConfig, custom_skill: Option<String>, force_mock: bool) -> Self {
        Self {
            config,
            custom_skill,
            force_mock,
        }
    }

    /// Handles a single incoming JSON-RPC request and returns an optional response.
    pub async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();

        match req.method.as_str() {
            "initialize" => {
                let id = id.unwrap_or(json!(1));
                let result = InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: ServerCapabilities {
                        tools: ToolsCapability { list_changed: None },
                    },
                    server_info: ServerInfo {
                        name: "magi".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };
                Some(JsonRpcResponse::success(id, json!(result)))
            }

            "notifications/initialized" | "initialized" => {
                // MCP specification notification: no response expected
                None
            }

            "tools/list" => {
                let id = id.unwrap_or(json!(1));
                let tools = vec![Tool {
                    name: "deliberate_with_magi".to_string(),
                    description: "Submits code, diffs, or architecture designs to the Evangelion MAGI Trinity consensus engine (Melchior-1, Balthasar-2, Casper-3).".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "context": {
                                "type": "string",
                                "description": "The source code, diff, or architectural proposal to audit."
                            },
                            "prompt": {
                                "type": "string",
                                "description": "Optional specific audit prompt or evaluation instructions."
                            },
                            "guidelines": {
                                "type": "string",
                                "description": "Optional engineering guidelines, security standards, or rules to enforce."
                            },
                            "rounds": {
                                "type": "integer",
                                "description": "Number of debate rounds (minimum 2: initial + final). Default is 2."
                            },
                            "mock": {
                                "type": "boolean",
                                "description": "Simulate evaluation without external LLM API calls."
                            },
                            "output_format": {
                                "type": "string",
                                "enum": ["json", "markdown"],
                                "description": "Format for output: 'json' (default) or 'markdown'."
                            }
                        },
                        "required": ["context"]
                    }),
                }];

                let result = ListToolsResult { tools };
                Some(JsonRpcResponse::success(id, json!(result)))
            }

            "tools/call" => {
                let id = id.unwrap_or(json!(1));
                let params = match req.params {
                    Some(p) => p,
                    None => {
                        return Some(JsonRpcResponse::error(
                            id,
                            -32602,
                            "Missing params for tools/call",
                        ))
                    }
                };

                let call_params: CallToolParams = match serde_json::from_value(params) {
                    Ok(cp) => cp,
                    Err(e) => {
                        return Some(JsonRpcResponse::error(
                            id,
                            -32602,
                            format!("Invalid tools/call params: {}", e),
                        ))
                    }
                };

                if call_params.name != "deliberate_with_magi" {
                    return Some(JsonRpcResponse::error(
                        id,
                        -32601,
                        format!("Unknown tool: {}", call_params.name),
                    ));
                }

                let response_result = self.execute_deliberation(call_params.arguments).await;
                match response_result {
                    Ok(tool_result) => Some(JsonRpcResponse::success(id, json!(tool_result))),
                    Err(err_msg) => {
                        let err_tool_result = CallToolResult {
                            content: vec![ToolContent {
                                content_type: "text".to_string(),
                                text: format!("MAGI Deliberation failed: {}", err_msg),
                            }],
                            is_error: true,
                        };
                        Some(JsonRpcResponse::success(id, json!(err_tool_result)))
                    }
                }
            }

            _ => {
                // If it's a notification without an id, ignore
                id.map(|id_val| {
                    JsonRpcResponse::error(
                        id_val,
                        -32601,
                        format!("Method '{}' not found", req.method),
                    )
                })
            }
        }
    }

    /// Executes deliberation based on parameters received in `tools/call`.
    async fn execute_deliberation(
        &self,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult, String> {
        let context = arguments
            .get("context")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required 'context' argument".to_string())?;

        let prompt = arguments
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Deliberate across architecture, security, and pragmatism.");

        let guidelines = arguments.get("guidelines").and_then(|v| v.as_str());

        let rounds = arguments
            .get("rounds")
            .and_then(|v| v.as_u64())
            .map(|r| (r as u8).max(2))
            .unwrap_or(2);

        let is_mock = arguments
            .get("mock")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.force_mock)
            || (self.config.melchior.api_key.is_none()
                && self.config.balthasar.api_key.is_none()
                && self.config.casper.api_key.is_none());

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let orchestrator = MagiOrchestrator::new(self.config.clone(), is_mock)
            .map_err(|e| format!("Failed to create orchestrator: {}", e))?
            .with_custom_skill(self.custom_skill.clone());

        let evaluations = if let Some(guide) = guidelines {
            orchestrator
                .deliberate_maintenance(context, guide, prompt, rounds)
                .await
                .map_err(|e| format!("Deliberation error: {}", e))?
        } else {
            orchestrator
                .deliberate_idea(prompt, context, rounds)
                .await
                .map_err(|e| format!("Deliberation error: {}", e))?
        };

        let lang = Language::detect(context);
        let consensus = calculate_local_consensus(&evaluations, lang);

        let text_content = if output_format == "markdown" {
            let mut md = format!(
                "# MAGI Trinity Consensus: {}\n\n**Verdict**: {}\n**Summary**: {}\n**Rounds**: {}\n\n",
                consensus.simple_verdict, consensus.verdict, consensus.summary, rounds
            );

            md.push_str("## Node Evaluations\n\n");
            for eval in &evaluations {
                md.push_str(&format!(
                    "### {} — Vote: {} (Risk: {}/10)\n",
                    eval.node_id, eval.vote, eval.risk_score
                ));
                if !eval.cwe_flags.is_empty() {
                    md.push_str(&format!("**CWE Flags**: {}\n\n", eval.cwe_flags.join(", ")));
                }
                md.push_str(&format!("{}\n\n", eval.argument));
            }
            md
        } else {
            let json_out = JsonOutput::build(
                0,
                "MCP Trinity Audit",
                "MCP_DELIBERATION",
                "SOURCE",
                &consensus.verdict,
                &consensus.summary,
                rounds,
                &evaluations,
            );
            serde_json::to_string_pretty(&json_out)
                .map_err(|e| format!("Failed to serialize deliberation result: {}", e))?
        };

        Ok(CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: text_content,
            }],
            is_error: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mcp_initialize() {
        let config = MagiConfig::default();
        let handler = McpHandler::new(config, None, true);

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let resp = handler
            .handle_request(req)
            .await
            .expect("response expected");
        assert_eq!(resp.id, json!(1));
        let result = resp.result.expect("result expected");
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "magi");
    }

    #[tokio::test]
    async fn test_mcp_list_tools() {
        let config = MagiConfig::default();
        let handler = McpHandler::new(config, None, true);

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let resp = handler
            .handle_request(req)
            .await
            .expect("response expected");
        let result = resp.result.expect("result expected");
        let tools = result["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "deliberate_with_magi");
    }

    #[tokio::test]
    async fn test_mcp_call_tool_mock() {
        let config = MagiConfig::default();
        let handler = McpHandler::new(config, None, true);

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "deliberate_with_magi",
                "arguments": {
                    "context": "fn authenticate(user: &str) -> bool { true }",
                    "rounds": 2,
                    "mock": true,
                    "output_format": "json"
                }
            })),
        };

        let resp = handler
            .handle_request(req)
            .await
            .expect("response expected");
        let result = resp.result.expect("result expected");
        assert_eq!(result["isError"], false);
        let content = result["content"].as_array().expect("content array");
        assert!(!content.is_empty());
        let text = content[0]["text"].as_str().expect("text string");
        assert!(text.contains("\"verdict\":"));
    }
}
