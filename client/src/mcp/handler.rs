//! # Model Context Protocol (MCP) Request Handler
//!
//! Handles MCP protocol requests including server initialization, tool discovery,
//! and executing tools against the MAGI Trinity consensus engine:
//! - `deliberate_with_magi`: General code/RFC deliberation.
//! - `audit_git_changes`: Inspects and deliberates over git diff / staged changes.
//! - `triage_incident_with_magi`: Specialist triage and resolution of runtime errors.
//! - `check_security_veto`: Fast defensive security audit with Balthasar-2.
//! - `debate_technical_dilemma`: Multi-agent trade-off debate on architectural decisions.

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

            "server/discover" => {
                let id = id.unwrap_or(json!(1));
                Some(JsonRpcResponse::success(id, json!({})))
            }

            "ping" => {
                let id = id.unwrap_or(json!(1));
                Some(JsonRpcResponse::success(id, json!({})))
            }

            "tools/list" => {
                let id = id.unwrap_or(json!(1));
                let tools = vec![
                    Tool {
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
                    },
                    Tool {
                        name: "audit_git_changes".to_string(),
                        description: "Inspects repository git diff (uncommitted, staged, or against a branch) and audits changes across architecture, security, and pragmatism.".to_string(),
                        input_schema: json!({
                            "type": "object",
                            "properties": {
                                "staged": {
                                    "type": "boolean",
                                    "description": "If true, audits staged changes only (`git diff --staged`). Defaults to false."
                                },
                                "branch": {
                                    "type": "string",
                                    "description": "Optional git reference or target branch to compare against (e.g. 'origin/main')."
                                },
                                "guidelines": {
                                    "type": "string",
                                    "description": "Optional engineering rules, coding standards, or architectural guidelines."
                                },
                                "prompt": {
                                    "type": "string",
                                    "description": "Specific focus for the audit."
                                },
                                "rounds": {
                                    "type": "integer",
                                    "description": "Number of debate rounds (minimum 2). Default is 2."
                                },
                                "output_format": {
                                    "type": "string",
                                    "enum": ["json", "markdown"],
                                    "description": "Format for output: 'json' (default) or 'markdown'."
                                }
                            }
                        }),
                    },
                    Tool {
                        name: "triage_incident_with_magi".to_string(),
                        description: "Triages runtime errors, panic stack traces, or compiler failures. Routes opening analysis to the best specialist node followed by full Trinity debate.".to_string(),
                        input_schema: json!({
                            "type": "object",
                            "properties": {
                                "error_log": {
                                    "type": "string",
                                    "description": "The stack trace, error log, or panic output to triage."
                                },
                                "code_context": {
                                    "type": "string",
                                    "description": "Optional related source code snippet or surrounding file context."
                                },
                                "rounds": {
                                    "type": "integer",
                                    "description": "Number of debate rounds (minimum 2). Default is 2."
                                },
                                "output_format": {
                                    "type": "string",
                                    "enum": ["json", "markdown"],
                                    "description": "Format for output: 'json' (default) or 'markdown'."
                                }
                            },
                            "required": ["error_log"]
                        }),
                    },
                    Tool {
                        name: "check_security_veto".to_string(),
                        description: "Runs a fast defensive security audit with Balthasar-2 (The Mother). Analyzes CWE flags, threat vectors, and determines if security veto is triggered.".to_string(),
                        input_schema: json!({
                            "type": "object",
                            "properties": {
                                "context": {
                                    "type": "string",
                                    "description": "Source code, configuration, or architectural design to audit for security vulnerabilities."
                                },
                                "prompt": {
                                    "type": "string",
                                    "description": "Optional specific security focus or threat scenario."
                                },
                                "output_format": {
                                    "type": "string",
                                    "enum": ["json", "markdown"],
                                    "description": "Format for output: 'json' (default) or 'markdown'."
                                }
                            },
                            "required": ["context"]
                        }),
                    },
                    Tool {
                        name: "debate_technical_dilemma".to_string(),
                        description: "Submits a technical question, architectural decision, or technology choice to the MAGI Trinity for multi-agent trade-off debate.".to_string(),
                        input_schema: json!({
                            "type": "object",
                            "properties": {
                                "dilemma": {
                                    "type": "string",
                                    "description": "The technical decision, dilemma, or question to debate."
                                },
                                "context": {
                                    "type": "string",
                                    "description": "Optional additional architecture or project context."
                                },
                                "rounds": {
                                    "type": "integer",
                                    "description": "Number of debate rounds (minimum 2). Default is 2."
                                },
                                "output_format": {
                                    "type": "string",
                                    "enum": ["json", "markdown"],
                                    "description": "Format for output: 'json' (default) or 'markdown'."
                                }
                            },
                            "required": ["dilemma"]
                        }),
                    },
                ];

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

                let response_result = match call_params.name.as_str() {
                    "deliberate_with_magi" => {
                        self.execute_deliberation(call_params.arguments).await
                    }
                    "audit_git_changes" => self.execute_git_diff_audit(call_params.arguments).await,
                    "triage_incident_with_magi" => self.execute_triage(call_params.arguments).await,
                    "check_security_veto" => {
                        self.execute_security_check(call_params.arguments).await
                    }
                    "debate_technical_dilemma" => self.execute_debate(call_params.arguments).await,
                    unknown => Err(format!("Unknown tool: {}", unknown)),
                };

                match response_result {
                    Ok(tool_result) => Some(JsonRpcResponse::success(id, json!(tool_result))),
                    Err(err_msg) => {
                        let err_tool_result = CallToolResult {
                            content: vec![ToolContent {
                                content_type: "text".to_string(),
                                text: format!("MAGI tool execution failed: {}", err_msg),
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

    /// Registers deliberation in SpacetimeDB and saves the Markdown audit report to `deliberations/`.
    #[allow(clippy::too_many_arguments)]
    async fn record_and_persist(
        &self,
        category: &str,
        title: &str,
        prompt: &str,
        context_type: &str,
        context_payload: &str,
        evaluations: &[crate::llm::NodeEvaluation],
        verdict: &str,
        summary: &str,
    ) -> u64 {
        // Skip persisting to disk and database when in mock/test mode
        if self.force_mock {
            return 0;
        }

        let db_client = crate::db::SpacetimeClient::new(
            self.config.spacetimedb_uri.clone(),
            self.config.spacetimedb_database.clone(),
        );

        let deliberation_id = match db_client
            .create_deliberation(
                &self.config.author,
                category,
                title,
                prompt,
                context_type,
                context_payload,
                "ALL",
            )
            .await
        {
            Ok(id) => {
                let _ = db_client.submit_evaluations(id, evaluations).await;
                id
            }
            Err(e) => {
                eprintln!(
                    "[MAGI MCP] SpacetimeDB registration skipped or unavailable: {}",
                    e
                );
                crate::ui::report::get_next_local_deliberation_id()
            }
        };

        let (effective_id, _) = crate::ui::report::save_host_deliberation_report_opts(
            deliberation_id,
            title,
            category,
            context_type,
            context_payload,
            evaluations,
            verdict,
            summary,
            true, // silent mode: stdout must remain pure JSON-RPC
        );

        effective_id
    }

    /// Executes general deliberation based on parameters received in `tools/call`.
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
            .with_custom_skill(self.custom_skill.clone())
            .silent(true);

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

        let title = if prompt.len() > 60 {
            format!("{}...", &prompt[..57])
        } else {
            prompt.to_string()
        };

        let delib_id = self
            .record_and_persist(
                "MCP_DELIBERATION",
                &title,
                prompt,
                "SOURCE",
                context,
                &evaluations,
                &consensus.verdict,
                &consensus.summary,
            )
            .await;

        let text_content = if output_format == "markdown" {
            let mut md = format!(
                "# MAGI Trinity Consensus #{:04}: {}\n\n**Verdict**: {}\n**Summary**: {}\n**Rounds**: {}\n\n",
                delib_id, consensus.simple_verdict, consensus.verdict, consensus.summary, rounds
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
                delib_id,
                &title,
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

    /// Executes git diff audit on repository changes.
    async fn execute_git_diff_audit(
        &self,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult, String> {
        let staged = arguments
            .get("staged")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let branch = arguments.get("branch").and_then(|v| v.as_str());
        let guidelines = arguments.get("guidelines").and_then(|v| v.as_str());
        let prompt = arguments
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Audit these git changes for architectural quality, regressions, security risks, and overengineering.");

        let rounds = arguments
            .get("rounds")
            .and_then(|v| v.as_u64())
            .map(|r| (r as u8).max(2))
            .unwrap_or(2);

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let is_mock = arguments
            .get("mock")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.force_mock)
            || (self.config.melchior.api_key.is_none()
                && self.config.balthasar.api_key.is_none()
                && self.config.casper.api_key.is_none());

        let filtered_diff = crate::core::helpers::get_git_diff_filtered(staged, branch)
            .map_err(|e| format!("Failed to read git diff: {}", e))?;

        if filtered_diff.is_empty() {
            return Ok(CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: if output_format == "json" {
                        json!({
                            "status": "clean",
                            "message": "No git changes detected to audit (working tree is clean)."
                        })
                        .to_string()
                    } else {
                        "No git changes detected to audit (working tree is clean).".to_string()
                    },
                }],
                is_error: false,
            });
        }

        let diff_text = filtered_diff.content;

        let orchestrator = MagiOrchestrator::new(self.config.clone(), is_mock)
            .map_err(|e| format!("Failed to create orchestrator: {}", e))?
            .with_custom_skill(self.custom_skill.clone())
            .silent(true);

        let evaluations = if let Some(guide) = guidelines {
            orchestrator
                .deliberate_maintenance(&diff_text, guide, prompt, rounds)
                .await
                .map_err(|e| format!("Deliberation error: {}", e))?
        } else {
            orchestrator
                .deliberate_idea(prompt, &diff_text, rounds)
                .await
                .map_err(|e| format!("Deliberation error: {}", e))?
        };

        let lang = Language::detect(&diff_text);
        let consensus = calculate_local_consensus(&evaluations, lang);

        let delib_id = self
            .record_and_persist(
                "GIT_DIFF_AUDIT",
                "Git Diff Audit",
                prompt,
                "CODE_DIFF",
                &diff_text,
                &evaluations,
                &consensus.verdict,
                &consensus.summary,
            )
            .await;

        let text_content = if output_format == "markdown" {
            let mut md = format!(
                "# MAGI Git Diff Audit Consensus #{:04}: {}\n\n**Verdict**: {}\n**Summary**: {}\n**Rounds**: {}\n\n",
                delib_id, consensus.simple_verdict, consensus.verdict, consensus.summary, rounds
            );
            if let Some(ctx) = orchestrator.project_context() {
                md.push_str(&format!("**Project Context**: {}\n\n", ctx.summary()));
            }
            if !filtered_diff.ignored_files.is_empty() {
                md.push_str(&format!(
                    "> ℹ️ **Smart Ingestion**: Excluded {} lock/generated file(s) from diff: `{}`\n\n",
                    filtered_diff.ignored_files.len(),
                    filtered_diff.ignored_files.join(", ")
                ));
            }
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
                delib_id,
                "Git Diff Audit",
                "GIT_DIFF_AUDIT",
                "CODE_DIFF",
                &consensus.verdict,
                &consensus.summary,
                rounds,
                &evaluations,
            );
            serde_json::to_string_pretty(&json_out)
                .map_err(|e| format!("Failed to serialize result: {}", e))?
        };

        Ok(CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: text_content,
            }],
            is_error: false,
        })
    }

    /// Executes triage on runtime error / panic log.
    async fn execute_triage(&self, arguments: serde_json::Value) -> Result<CallToolResult, String> {
        let error_log = arguments
            .get("error_log")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required 'error_log' argument".to_string())?;

        let code_context = arguments.get("code_context").and_then(|v| v.as_str());

        let rounds = arguments
            .get("rounds")
            .and_then(|v| v.as_u64())
            .map(|r| (r as u8).max(2))
            .unwrap_or(2);

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let is_mock = arguments
            .get("mock")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.force_mock)
            || (self.config.melchior.api_key.is_none()
                && self.config.balthasar.api_key.is_none()
                && self.config.casper.api_key.is_none());

        let orchestrator = MagiOrchestrator::new(self.config.clone(), is_mock)
            .map_err(|e| format!("Failed to create orchestrator: {}", e))?
            .with_custom_skill(self.custom_skill.clone())
            .silent(true);

        let (lead_node, opening_eval, trinity_evals_opt) = orchestrator
            .triage_error(error_log, code_context, rounds)
            .await
            .map_err(|e| format!("Triage error: {}", e))?;

        let evaluations = trinity_evals_opt.unwrap_or_else(|| vec![opening_eval.clone()]);
        let lang = Language::detect(error_log);
        let consensus = calculate_local_consensus(&evaluations, lang);

        let title = format!("Incident Triage: routed to {}", lead_node);
        let delib_id = self
            .record_and_persist(
                "INCIDENT_TRIAGE",
                &title,
                "Triage runtime error and recommend mitigation",
                "ERROR_LOG",
                error_log,
                &evaluations,
                &consensus.verdict,
                &consensus.summary,
            )
            .await;

        let text_content = if output_format == "markdown" {
            let mut md = format!(
                "# MAGI Incident Triage #{:04}: {}\n\n**Specialist Routing**: {}\n**Consensus Verdict**: {}\n**Summary**: {}\n\n",
                delib_id, consensus.simple_verdict, lead_node, consensus.verdict, consensus.summary
            );
            md.push_str("## Specialist Opening Finding\n\n");
            md.push_str(&format!(
                "**{}** (Risk: {}/10, Vote: {}):\n{}\n\n",
                opening_eval.node_id,
                opening_eval.risk_score,
                opening_eval.vote,
                opening_eval.argument
            ));
            md.push_str("## Final Trinity Debate Evaluations\n\n");
            for eval in &evaluations {
                md.push_str(&format!(
                    "### {} — Vote: {} (Risk: {}/10)\n{}\n\n",
                    eval.node_id, eval.vote, eval.risk_score, eval.argument
                ));
            }
            md
        } else {
            let json_out = JsonOutput::build(
                delib_id,
                &title,
                "INCIDENT_TRIAGE",
                "ERROR_LOG",
                &consensus.verdict,
                &consensus.summary,
                rounds,
                &evaluations,
            );
            serde_json::to_string_pretty(&json_out)
                .map_err(|e| format!("Failed to serialize result: {}", e))?
        };

        Ok(CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: text_content,
            }],
            is_error: false,
        })
    }

    /// Executes defensive security audit via Balthasar-2.
    async fn execute_security_check(
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
            .unwrap_or("Perform an authoritative defensive security analysis. Detect Common Weakness Enumerations (CWE), assess attack surface, and determine whether risk warrants a security veto.");

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let is_mock = arguments
            .get("mock")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.force_mock)
            || self.config.balthasar.api_key.is_none();

        let orchestrator = MagiOrchestrator::new(self.config.clone(), is_mock)
            .map_err(|e| format!("Failed to create orchestrator: {}", e))?
            .with_custom_skill(self.custom_skill.clone())
            .silent(true);

        let eval = orchestrator
            .evaluate_security_veto(context, prompt)
            .await
            .map_err(|e| format!("Security evaluation error: {}", e))?;

        let veto_enforced = eval.risk_score >= 8 && eval.vote == "REJECT";

        let delib_id = self
            .record_and_persist(
                "SECURITY_VETO",
                "Balthasar-2 Security Audit",
                prompt,
                "SOURCE",
                context,
                std::slice::from_ref(&eval),
                &eval.vote,
                &eval.argument,
            )
            .await;

        let text_content = if output_format == "markdown" {
            let mut md = format!(
                "# Balthasar-2 Security Audit #{:04}\n\n**Vote**: {}\n**Risk Score**: {}/10\n**Security Veto Enforced**: {}\n**Confidence**: {:.0}%\n\n",
                delib_id, eval.vote, eval.risk_score, veto_enforced, eval.confidence * 100.0
            );
            if !eval.cwe_flags.is_empty() {
                md.push_str(&format!(
                    "**CWE Flags Detected**: {}\n\n",
                    eval.cwe_flags.join(", ")
                ));
            }
            md.push_str("## Findings\n\n");
            for f in &eval.findings {
                md.push_str(&format!(
                    "### [{}] {}\n- **Impact**: {}\n- **Evidence**: `{}`\n- **Recommendation**: {}\n\n",
                    f.severity.to_uppercase(), f.title, f.impact, f.evidence, f.recommendation
                ));
            }
            md.push_str(&format!("## Analysis\n\n{}\n", eval.argument));
            md
        } else {
            let json_val = json!({
                "deliberation_id": delib_id,
                "node_id": eval.node_id,
                "vote": eval.vote,
                "risk_score": eval.risk_score,
                "veto_enforced": veto_enforced,
                "confidence": eval.confidence,
                "cwe_flags": eval.cwe_flags,
                "findings": eval.findings,
                "rationale": eval.argument,
                "model": eval.model,
                "execution_time_ms": eval.execution_time_ms
            });
            serde_json::to_string_pretty(&json_val)
                .map_err(|e| format!("Failed to serialize result: {}", e))?
        };

        Ok(CallToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: text_content,
            }],
            is_error: false,
        })
    }

    /// Executes multi-agent debate on a technical dilemma.
    async fn execute_debate(&self, arguments: serde_json::Value) -> Result<CallToolResult, String> {
        let dilemma = arguments
            .get("dilemma")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required 'dilemma' argument".to_string())?;

        let context = arguments.get("context").and_then(|v| v.as_str());

        let rounds = arguments
            .get("rounds")
            .and_then(|v| v.as_u64())
            .map(|r| (r as u8).max(2))
            .unwrap_or(2);

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let is_mock = arguments
            .get("mock")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.force_mock)
            || (self.config.melchior.api_key.is_none()
                && self.config.balthasar.api_key.is_none()
                && self.config.casper.api_key.is_none());

        let orchestrator = MagiOrchestrator::new(self.config.clone(), is_mock)
            .map_err(|e| format!("Failed to create orchestrator: {}", e))?
            .with_custom_skill(self.custom_skill.clone())
            .silent(true);

        let evaluations = orchestrator
            .deliberate_debate(dilemma, context, rounds)
            .await
            .map_err(|e| format!("Debate error: {}", e))?;

        let lang = Language::detect(dilemma);
        let consensus = calculate_local_consensus(&evaluations, lang);

        let delib_id = self
            .record_and_persist(
                "TECHNICAL_DEBATE",
                dilemma,
                dilemma,
                "DILEMMA",
                context.unwrap_or(dilemma),
                &evaluations,
                &consensus.verdict,
                &consensus.summary,
            )
            .await;

        let text_content = if output_format == "markdown" {
            let mut md = format!(
                "# MAGI Technical Debate #{:04}: {}\n\n**Question**: {}\n**Consensus Verdict**: {}\n**Synthesis**: {}\n**Rounds**: {}\n\n",
                delib_id, consensus.simple_verdict, dilemma, consensus.verdict, consensus.summary, rounds
            );
            md.push_str("## Node Arguments & Trade-Offs\n\n");
            for eval in &evaluations {
                md.push_str(&format!(
                    "### {} — Vote: {} (Risk: {}/10)\n{}\n\n",
                    eval.node_id, eval.vote, eval.risk_score, eval.argument
                ));
            }
            md
        } else {
            let json_out = JsonOutput::build(
                delib_id,
                dilemma,
                "TECHNICAL_DEBATE",
                "DILEMMA",
                &consensus.verdict,
                &consensus.summary,
                rounds,
                &evaluations,
            );
            serde_json::to_string_pretty(&json_out)
                .map_err(|e| format!("Failed to serialize result: {}", e))?
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
        assert_eq!(tools.len(), 5);
        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"deliberate_with_magi"));
        assert!(tool_names.contains(&"audit_git_changes"));
        assert!(tool_names.contains(&"triage_incident_with_magi"));
        assert!(tool_names.contains(&"check_security_veto"));
        assert!(tool_names.contains(&"debate_technical_dilemma"));
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

    #[tokio::test]
    async fn test_mcp_call_security_check_mock() {
        let config = MagiConfig::default();
        let handler = McpHandler::new(config, None, true);

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "check_security_veto",
                "arguments": {
                    "context": "let secret = \"plaintext_password\";",
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
        assert!(
            text.contains("\"node_id\": \"balthasar-2\"")
                || text.contains("\"node_id\": \"Balthasar-2\"")
                || text.contains("balthasar")
        );
    }

    #[tokio::test]
    async fn test_mcp_call_debate_mock() {
        let config = MagiConfig::default();
        let handler = McpHandler::new(config, None, true);

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(5)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "debate_technical_dilemma",
                "arguments": {
                    "dilemma": "WebSockets vs SSE for notifications",
                    "rounds": 2,
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
