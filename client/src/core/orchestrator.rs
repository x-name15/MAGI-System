//! # MAGI Asynchronous Orchestrator
//!
//! Provider-agnostic coordinator supporting the three core MAGI deliberation workflows:
//! - Case 1: Idea / Markdown Viability Review
//! - Case 2: Code Maintenance & Guidelines Compliance
//! - Case 3: Error / Incident Triage with specialist routing and Trinity verdict

use crate::config::MagiConfig;
use crate::error::MagiError;
use crate::llm::mock::MockProvider;
use crate::llm::{BalthasarNode, CasperNode, LlmProvider, MelchiorNode, NodeEvaluation};
use colored::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Core orchestrator for multi-node evaluation.
pub struct MagiOrchestrator {
    config: MagiConfig,
    melchior: Arc<dyn LlmProvider>,
    balthasar: Arc<dyn LlmProvider>,
    casper: Arc<dyn LlmProvider>,
    prompt_loader: crate::skills::PromptLoader,
    custom_skill: Option<String>,
    project_context: Option<crate::core::helpers::ProjectContext>,
    silent: bool,
}

impl MagiOrchestrator {
    /// Builds a new orchestrator, initializing providers according to configuration.
    pub fn new(config: MagiConfig, use_mock: bool) -> Result<Self, MagiError> {
        let (melchior, balthasar, casper): (
            Arc<dyn LlmProvider>,
            Arc<dyn LlmProvider>,
            Arc<dyn LlmProvider>,
        ) = if use_mock {
            (
                Arc::new(MockProvider::for_node("Melchior-1", false)),
                Arc::new(MockProvider::for_node("Balthasar-2", false)),
                Arc::new(MockProvider::for_node("Casper-3", false)),
            )
        } else {
            let mut melchior_cfg = config.melchior.clone();
            melchior_cfg.max_retries = config.max_retries;
            melchior_cfg.retry_delay_ms = config.retry_delay_ms;
            melchior_cfg.max_context_chars = config.max_context_chars;

            let mut balthasar_cfg = config.balthasar.clone();
            balthasar_cfg.max_retries = config.max_retries;
            balthasar_cfg.retry_delay_ms = config.retry_delay_ms;
            balthasar_cfg.max_context_chars = config.max_context_chars;

            let mut casper_cfg = config.casper.clone();
            casper_cfg.max_retries = config.max_retries;
            casper_cfg.retry_delay_ms = config.retry_delay_ms;
            casper_cfg.max_context_chars = config.max_context_chars;

            (
                Arc::new(MelchiorNode::new(&melchior_cfg)),
                Arc::new(BalthasarNode::new(&balthasar_cfg)),
                Arc::new(CasperNode::new(&casper_cfg)),
            )
        };

        let project_context = crate::core::helpers::discover_project_context(None);

        Ok(Self {
            config,
            melchior,
            balthasar,
            casper,
            prompt_loader: crate::skills::PromptLoader::new(),
            custom_skill: None,
            project_context,
            silent: false,
        })
    }

    /// Sets silent mode to suppress console spinner and progress logs.
    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// Injects custom project skill or instruction markdown before deliberation.
    pub fn with_custom_skill(mut self, custom_skill: Option<String>) -> Self {
        self.custom_skill = custom_skill;
        self
    }

    /// Sets or overrides the discovered project ecosystem context.
    #[allow(dead_code)]
    pub fn with_project_context(
        mut self,
        context: Option<crate::core::helpers::ProjectContext>,
    ) -> Self {
        self.project_context = context;
        self
    }

    /// Returns a reference to the discovered project context, if any.
    pub fn project_context(&self) -> Option<&crate::core::helpers::ProjectContext> {
        self.project_context.as_ref()
    }

    /// Case 1: Evaluates an Idea / Markdown specification across the Trinity.
    pub async fn deliberate_idea(
        &self,
        instructions: &str,
        idea_markdown: &str,
        rounds: u8,
    ) -> Result<Vec<NodeEvaluation>, MagiError> {
        let (base_m, _) = self.prompt_loader.load_node_prompt("Melchior-1");
        let (base_b, _) = self.prompt_loader.load_node_prompt("Balthasar-2");
        let (base_c, _) = self.prompt_loader.load_node_prompt("Casper-3");

        let prompt_m = self
            .prompt_loader
            .compose_prompt(&base_m, self.custom_skill.as_deref());
        let prompt_b = self
            .prompt_loader
            .compose_prompt(&base_b, self.custom_skill.as_deref());
        let prompt_c = self
            .prompt_loader
            .compose_prompt(&base_c, self.custom_skill.as_deref());

        self.deliberate_trinity(
            &prompt_m,
            &prompt_b,
            &prompt_c,
            instructions,
            idea_markdown,
            rounds,
        )
        .await
    }

    /// Case 2: Evaluates code maintenance under specific team guidelines.
    pub async fn deliberate_maintenance(
        &self,
        code_context: &str,
        guidelines: &str,
        instructions: &str,
        rounds: u8,
    ) -> Result<Vec<NodeEvaluation>, MagiError> {
        let combined_payload = format!(
            "GUIDELINES TO ENFORCE:\n{}\n\nCODE TO MAINTAIN/AUDIT:\n{}",
            guidelines, code_context
        );

        let (base_m, _) = self.prompt_loader.load_node_prompt("Melchior-1");
        let (base_b, _) = self.prompt_loader.load_node_prompt("Balthasar-2");
        let (base_c, _) = self.prompt_loader.load_node_prompt("Casper-3");

        let prompt_m = self
            .prompt_loader
            .compose_prompt(&base_m, self.custom_skill.as_deref());
        let prompt_b = self
            .prompt_loader
            .compose_prompt(&base_b, self.custom_skill.as_deref());
        let prompt_c = self
            .prompt_loader
            .compose_prompt(&base_c, self.custom_skill.as_deref());

        self.deliberate_trinity(
            &prompt_m,
            &prompt_b,
            &prompt_c,
            instructions,
            &combined_payload,
            rounds,
        )
        .await
    }

    /// Case 3: Routes an error to a specialist for the opening analysis, then
    /// always convenes the full Trinity for the final decision.
    pub async fn triage_error(
        &self,
        error_text: &str,
        code_context: Option<&str>,
        rounds: u8,
    ) -> Result<(String, NodeEvaluation, Option<Vec<NodeEvaluation>>), MagiError> {
        let lead_node = Self::select_lead_node_for_error(error_text);
        let context_str = code_context.unwrap_or("No surrounding code supplied.");
        let combined = format!(
            "ERROR / STACKTRACE:\n{}\n\nCODE CONTEXT:\n{}",
            error_text, context_str
        );

        let (base_m, _) = self.prompt_loader.load_node_prompt("Melchior-1");
        let (base_b, _) = self.prompt_loader.load_node_prompt("Balthasar-2");
        let (base_c, _) = self.prompt_loader.load_node_prompt("Casper-3");

        let provider = match lead_node {
            "Melchior-1" => Arc::clone(&self.melchior),
            "Balthasar-2" => Arc::clone(&self.balthasar),
            _ => Arc::clone(&self.casper),
        };

        let base_lead = match lead_node {
            "Melchior-1" => &base_m,
            "Balthasar-2" => &base_b,
            _ => &base_c,
        };

        let opening_prompt = self
            .prompt_loader
            .compose_prompt(base_lead, self.custom_skill.as_deref());

        let lang = crate::i18n::Language::detect(&combined);
        let bundle = crate::i18n::get_bundle(lang);

        let opening_instruction = bundle.debate.format_opening_instruction(lead_node);

        let evaluation = provider
            .evaluate(lead_node, &opening_prompt, &opening_instruction, &combined)
            .await?;

        let full_prompt = bundle
            .debate
            .format_incident_deliberation(lead_node, &evaluation.argument);
        let full_context = format!(
            "{}\n\n{}",
            combined,
            bundle.debate.format_lead_node_label(lead_node)
        );

        let prompt_full_m = self
            .prompt_loader
            .compose_prompt(&base_m, self.custom_skill.as_deref());
        let prompt_full_b = self
            .prompt_loader
            .compose_prompt(&base_b, self.custom_skill.as_deref());
        let prompt_full_c = self
            .prompt_loader
            .compose_prompt(&base_c, self.custom_skill.as_deref());

        let full_trinity = self
            .deliberate_trinity(
                &prompt_full_m,
                &prompt_full_b,
                &prompt_full_c,
                &full_prompt,
                &full_context,
                rounds,
            )
            .await?;

        Ok((lead_node.to_string(), evaluation, Some(full_trinity)))
    }

    /// Evaluates a code or architecture snippet solely through Balthasar-2 (The Mother)
    /// to determine if any critical security vulnerabilities or unilateral veto conditions exist.
    pub async fn evaluate_security_veto(
        &self,
        context: &str,
        instructions: &str,
    ) -> Result<NodeEvaluation, MagiError> {
        let (base_b, _) = self.prompt_loader.load_node_prompt("Balthasar-2");
        let prompt_b = self
            .prompt_loader
            .compose_prompt(&base_b, self.custom_skill.as_deref());

        let effective_context = if let Some(ref ctx) = self.project_context {
            format!("{}\n\n{}", ctx.to_prompt_header(), context)
        } else {
            context.to_string()
        };

        self.balthasar
            .evaluate("Balthasar-2", &prompt_b, instructions, &effective_context)
            .await
    }

    /// Deliberates a technical dilemma, architectural decision, or technology choice across the Trinity.
    pub async fn deliberate_debate(
        &self,
        dilemma: &str,
        context: Option<&str>,
        rounds: u8,
    ) -> Result<Vec<NodeEvaluation>, MagiError> {
        let (base_m, _) = self.prompt_loader.load_node_prompt("Melchior-1");
        let (base_b, _) = self.prompt_loader.load_node_prompt("Balthasar-2");
        let (base_c, _) = self.prompt_loader.load_node_prompt("Casper-3");

        let prompt_m = self
            .prompt_loader
            .compose_prompt(&base_m, self.custom_skill.as_deref());
        let prompt_b = self
            .prompt_loader
            .compose_prompt(&base_b, self.custom_skill.as_deref());
        let prompt_c = self
            .prompt_loader
            .compose_prompt(&base_c, self.custom_skill.as_deref());

        let instructions = format!(
            "TECHNICAL DILEMMA & ARCHITECTURAL CHOICE:\n{}\n\n\
            Evaluate the trade-offs: Melchior addresses algorithmic/distributed systems architecture; \
            Balthasar addresses security threat surfaces, permissions, and DOS/failure resilience; \
            Casper addresses operational pragmatism, cognitive load, DX, and delivery feasibility.",
            dilemma
        );

        let context_payload = context.unwrap_or("No additional context provided.");

        self.deliberate_trinity(
            &prompt_m,
            &prompt_b,
            &prompt_c,
            &instructions,
            context_payload,
            rounds,
        )
        .await
    }

    /// Helper to classify the nature of an error and select the lead node persona.
    pub fn select_lead_node_for_error(error_text: &str) -> &'static str {
        crate::core::helpers::select_lead_node_for_error(error_text)
    }

    /// Executes concurrent deliberation across all 3 nodes for the specified number of rounds.
    ///
    /// Round 1 is always the initial independent evaluation. Rounds 2..N are iterative peer
    /// debate passes where each node reviews the previous round's positions before issuing a
    /// revised vote. The final round's evaluations are returned and annotated with the Round 1
    /// initial positions for audit trail purposes.
    async fn deliberate_trinity(
        &self,
        prompt_m: &str,
        prompt_b: &str,
        prompt_c: &str,
        user_prompt: &str,
        context_payload: &str,
        rounds: u8,
    ) -> Result<Vec<NodeEvaluation>, MagiError> {
        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
        // Clamp to at least 2 rounds (1 initial + 1 final) so the protocol always terminates
        // with a peer-reviewed result.
        let effective_rounds = rounds.max(2);

        let effective_payload = if let Some(ref ctx) = self.project_context {
            format!("{}\n\n{}", ctx.to_prompt_header(), context_payload)
        } else {
            context_payload.to_string()
        };

        let combined_text = format!("{} {}", user_prompt, effective_payload);
        let lang = crate::i18n::Language::detect(&combined_text);
        let bundle = crate::i18n::get_bundle(lang);

        if !self.silent {
            eprintln!();
            eprintln!(
                "{}",
                format!(
                    "  [*] [{}] ({} rounds)",
                    bundle.ui.consulting_trinity, effective_rounds
                )
                .bright_yellow()
                .bold()
            );
        }

        let evaluation_future = async {
            // Helper closure to construct a fallback evaluation for an offline node
            let synthesize_degraded = |node_id: &str, error_msg: &str| -> NodeEvaluation {
                let msg = format!(
                    "[OFFLINE/DEGRADED QUORUM] Node failed to respond: {}",
                    error_msg
                );
                NodeEvaluation {
                    node_id: node_id.to_string(),
                    vote: "NEUTRAL".to_string(),
                    risk_score: 5,
                    findings: Vec::new(),
                    rationale: msg.clone(),
                    argument: msg.clone(),
                    confidence: 0.0,
                    cwe_flags: Vec::new(),
                    execution_time_ms: 0,
                    prompt_version: "v1.0".to_string(),
                    model: "offline".to_string(),
                    initial_argument: Some(msg),
                    initial_vote: Some("NEUTRAL".to_string()),
                    initial_risk_score: Some(5),
                }
            };

            let mut m_online = true;
            let mut b_online = true;
            let mut c_online = true;

            // Round 1: independent evaluations
            let (res_m, res_b, res_c) = tokio::join!(
                self.melchior
                    .evaluate("Melchior-1", prompt_m, user_prompt, &effective_payload),
                self.balthasar
                    .evaluate("Balthasar-2", prompt_b, user_prompt, &effective_payload),
                self.casper
                    .evaluate("Casper-3", prompt_c, user_prompt, &effective_payload),
            );

            // Verify quorum in Round 1
            let failures = (res_m.is_err() as u8) + (res_b.is_err() as u8) + (res_c.is_err() as u8);
            if failures > 1 || (failures > 0 && !self.config.allow_degraded_quorum) {
                let err = res_m
                    .err()
                    .or_else(|| res_b.err())
                    .or_else(|| res_c.err())
                    .unwrap();
                return Err(err);
            }

            let eval_m = match res_m {
                Ok(ev) => ev,
                Err(e) => {
                    m_online = false;
                    if !self.silent {
                        eprintln!(
                            "{}",
                            format!(
                                "  [!] [DEGRADED QUORUM] Node Melchior-1 is offline: {}. Proceeding with 2-of-3 quorum.",
                                e
                            )
                            .bright_yellow()
                            .bold()
                        );
                    }
                    synthesize_degraded("Melchior-1", &e.to_string())
                }
            };

            let eval_b = match res_b {
                Ok(ev) => ev,
                Err(e) => {
                    b_online = false;
                    if !self.silent {
                        eprintln!(
                            "{}",
                            format!(
                                "  [!] [DEGRADED QUORUM] Node Balthasar-2 is offline: {}. Proceeding with 2-of-3 quorum.",
                                e
                            )
                            .bright_yellow()
                            .bold()
                        );
                    }
                    synthesize_degraded("Balthasar-2", &e.to_string())
                }
            };

            let eval_c = match res_c {
                Ok(ev) => ev,
                Err(e) => {
                    c_online = false;
                    if !self.silent {
                        eprintln!(
                            "{}",
                            format!(
                                "  [!] [DEGRADED QUORUM] Node Casper-3 is offline: {}. Proceeding with 2-of-3 quorum.",
                                e
                            )
                            .bright_yellow()
                            .bold()
                        );
                    }
                    synthesize_degraded("Casper-3", &e.to_string())
                }
            };

            let first_round = vec![eval_m.clone(), eval_b.clone(), eval_c.clone()];

            // Intermediate rounds (2 .. effective_rounds - 1)
            let mut current_round = first_round.clone();
            for round_num in 2..effective_rounds {
                let peer_positions = crate::core::helpers::build_peer_summary(&current_round);
                let (debate_prompt, debate_context) = crate::core::helpers::build_debate_prompts(
                    lang,
                    user_prompt,
                    &effective_payload,
                    &peer_positions,
                    round_num,
                    effective_rounds,
                );

                let fut_m = async {
                    if m_online {
                        Some(
                            self.melchior
                                .evaluate("Melchior-1", prompt_m, &debate_prompt, &debate_context)
                                .await,
                        )
                    } else {
                        None
                    }
                };
                let fut_b = async {
                    if b_online {
                        Some(
                            self.balthasar
                                .evaluate("Balthasar-2", prompt_b, &debate_prompt, &debate_context)
                                .await,
                        )
                    } else {
                        None
                    }
                };
                let fut_c = async {
                    if c_online {
                        Some(
                            self.casper
                                .evaluate("Casper-3", prompt_c, &debate_prompt, &debate_context)
                                .await,
                        )
                    } else {
                        None
                    }
                };

                let (r_m, r_b, r_c) = tokio::join!(fut_m, fut_b, fut_c);

                let online_before = (m_online as u8) + (b_online as u8) + (c_online as u8);
                let new_failures = r_m.as_ref().map_or(0, |r| r.is_err() as u8)
                    + r_b.as_ref().map_or(0, |r| r.is_err() as u8)
                    + r_c.as_ref().map_or(0, |r| r.is_err() as u8);
                let online_after = online_before.saturating_sub(new_failures);

                if online_after < 2 || (new_failures > 0 && !self.config.allow_degraded_quorum) {
                    if let Some(Err(e)) = r_m {
                        return Err(e);
                    }
                    if let Some(Err(e)) = r_b {
                        return Err(e);
                    }
                    if let Some(Err(e)) = r_c {
                        return Err(e);
                    }
                }

                if let Some(res) = r_m {
                    match res {
                        Ok(ev) => current_round[0] = ev,
                        Err(e) => {
                            m_online = false;
                            if !self.silent {
                                eprintln!(
                                    "{}",
                                    format!(
                                        "  [!] [DEGRADED QUORUM] Node Melchior-1 failed in round {}: {}. Proceeding with 2-of-3 quorum.",
                                        round_num, e
                                    )
                                    .bright_yellow()
                                    .bold()
                                );
                            }
                            current_round[0] = synthesize_degraded("Melchior-1", &e.to_string());
                        }
                    }
                }
                if let Some(res) = r_b {
                    match res {
                        Ok(ev) => current_round[1] = ev,
                        Err(e) => {
                            b_online = false;
                            if !self.silent {
                                eprintln!(
                                    "{}",
                                    format!(
                                        "  [!] [DEGRADED QUORUM] Node Balthasar-2 failed in round {}: {}. Proceeding with 2-of-3 quorum.",
                                        round_num, e
                                    )
                                    .bright_yellow()
                                    .bold()
                                );
                            }
                            current_round[1] = synthesize_degraded("Balthasar-2", &e.to_string());
                        }
                    }
                }
                if let Some(res) = r_c {
                    match res {
                        Ok(ev) => current_round[2] = ev,
                        Err(e) => {
                            c_online = false;
                            if !self.silent {
                                eprintln!(
                                    "{}",
                                    format!(
                                        "  [!] [DEGRADED QUORUM] Node Casper-3 failed in round {}: {}. Proceeding with 2-of-3 quorum.",
                                        round_num, e
                                    )
                                    .bright_yellow()
                                    .bold()
                                );
                            }
                            current_round[2] = synthesize_degraded("Casper-3", &e.to_string());
                        }
                    }
                }
            }

            // Final round (uses last intermediate as peer context)
            let peer_positions = crate::core::helpers::build_peer_summary(&current_round);
            let (final_prompt, final_context) = crate::core::helpers::build_debate_prompts(
                lang,
                user_prompt,
                &effective_payload,
                &peer_positions,
                effective_rounds,
                effective_rounds,
            );

            let fut_final_m = async {
                if m_online {
                    Some(
                        self.melchior
                            .evaluate("Melchior-1", prompt_m, &final_prompt, &final_context)
                            .await,
                    )
                } else {
                    None
                }
            };
            let fut_final_b = async {
                if b_online {
                    Some(
                        self.balthasar
                            .evaluate("Balthasar-2", prompt_b, &final_prompt, &final_context)
                            .await,
                    )
                } else {
                    None
                }
            };
            let fut_final_c = async {
                if c_online {
                    Some(
                        self.casper
                            .evaluate("Casper-3", prompt_c, &final_prompt, &final_context)
                            .await,
                    )
                } else {
                    None
                }
            };

            let (final_m, final_b, final_c) = tokio::join!(fut_final_m, fut_final_b, fut_final_c);

            let online_before = (m_online as u8) + (b_online as u8) + (c_online as u8);
            let new_failures = final_m.as_ref().map_or(0, |r| r.is_err() as u8)
                + final_b.as_ref().map_or(0, |r| r.is_err() as u8)
                + final_c.as_ref().map_or(0, |r| r.is_err() as u8);
            let online_after = online_before.saturating_sub(new_failures);

            if online_after < 2 || (new_failures > 0 && !self.config.allow_degraded_quorum) {
                if let Some(Err(e)) = final_m {
                    return Err(e);
                }
                if let Some(Err(e)) = final_b {
                    return Err(e);
                }
                if let Some(Err(e)) = final_c {
                    return Err(e);
                }
            }

            let mut final_evals = current_round.clone();
            if let Some(res) = final_m {
                match res {
                    Ok(ev) => final_evals[0] = ev,
                    Err(e) => {
                        if !self.silent {
                            eprintln!(
                                "{}",
                                format!(
                                    "  [!] [DEGRADED QUORUM] Node Melchior-1 failed in final round: {}. Proceeding with 2-of-3 quorum.",
                                    e
                                )
                                .bright_yellow()
                                .bold()
                            );
                        }
                        final_evals[0] = synthesize_degraded("Melchior-1", &e.to_string());
                    }
                }
            }
            if let Some(res) = final_b {
                match res {
                    Ok(ev) => final_evals[1] = ev,
                    Err(e) => {
                        if !self.silent {
                            eprintln!(
                                "{}",
                                format!(
                                    "  [!] [DEGRADED QUORUM] Node Balthasar-2 failed in final round: {}. Proceeding with 2-of-3 quorum.",
                                    e
                                )
                                .bright_yellow()
                                .bold()
                            );
                        }
                        final_evals[1] = synthesize_degraded("Balthasar-2", &e.to_string());
                    }
                }
            }
            if let Some(res) = final_c {
                match res {
                    Ok(ev) => final_evals[2] = ev,
                    Err(e) => {
                        if !self.silent {
                            eprintln!(
                                "{}",
                                format!(
                                    "  [!] [DEGRADED QUORUM] Node Casper-3 failed in final round: {}. Proceeding with 2-of-3 quorum.",
                                    e
                                )
                                .bright_yellow()
                                .bold()
                            );
                        }
                        final_evals[2] = synthesize_degraded("Casper-3", &e.to_string());
                    }
                }
            }

            // Annotate with Round 1 positions for audit trail and contextualize default rationales
            for evaluation in &mut final_evals {
                if let Some(first_position) = first_round
                    .iter()
                    .find(|first| first.node_id == evaluation.node_id)
                {
                    let init_arg = if !first_position.rationale.trim().is_empty() {
                        first_position.rationale.clone()
                    } else {
                        first_position.argument.clone()
                    };
                    evaluation.initial_argument = Some(init_arg.clone());
                    evaluation.initial_vote = Some(first_position.vote.clone());
                    evaluation.initial_risk_score = Some(first_position.risk_score);

                    // If final post-debate rationale is empty or matches generic fallback,
                    // contextualize with round 1 rationale
                    if evaluation.rationale.trim().is_empty()
                        || evaluation.rationale == bundle.prompt.default_rationale
                    {
                        if evaluation.vote == first_position.vote {
                            evaluation.rationale = format!(
                                "{}: {}",
                                bundle.debate.maintains_position_rationale, init_arg
                            );
                        } else {
                            evaluation.rationale = init_arg;
                        }
                        evaluation.argument = evaluation.rationale.clone();
                    }
                }
            }

            Ok::<Vec<NodeEvaluation>, MagiError>(final_evals)
        };

        match timeout(timeout_duration, evaluation_future).await {
            Ok(result) => result,
            Err(_) => Err(MagiError::Timeout(self.config.timeout_seconds)),
        }
    }
}
