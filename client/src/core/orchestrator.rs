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
            (
                Arc::new(MelchiorNode::new(&config.melchior)),
                Arc::new(BalthasarNode::new(&config.balthasar)),
                Arc::new(CasperNode::new(&config.casper)),
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
        })
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

        eprintln!();
        eprintln!(
            "{}",
            format!(
                "  \u{27f3} [{}] ({} rounds)",
                bundle.ui.consulting_trinity, effective_rounds
            )
            .bright_yellow()
            .bold()
        );

        let evaluation_future = async {
            // Round 1: independent evaluations
            let (res_m, res_b, res_c) = tokio::join!(
                self.melchior
                    .evaluate("Melchior-1", prompt_m, user_prompt, &effective_payload),
                self.balthasar
                    .evaluate("Balthasar-2", prompt_b, user_prompt, &effective_payload),
                self.casper
                    .evaluate("Casper-3", prompt_c, user_prompt, &effective_payload),
            );

            let first_round = vec![res_m?, res_b?, res_c?];

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

                let (r_m, r_b, r_c) = tokio::join!(
                    self.melchior
                        .evaluate("Melchior-1", prompt_m, &debate_prompt, &debate_context),
                    self.balthasar.evaluate(
                        "Balthasar-2",
                        prompt_b,
                        &debate_prompt,
                        &debate_context
                    ),
                    self.casper
                        .evaluate("Casper-3", prompt_c, &debate_prompt, &debate_context),
                );

                current_round = vec![r_m?, r_b?, r_c?];
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

            let (final_m, final_b, final_c) = tokio::join!(
                self.melchior
                    .evaluate("Melchior-1", prompt_m, &final_prompt, &final_context),
                self.balthasar
                    .evaluate("Balthasar-2", prompt_b, &final_prompt, &final_context),
                self.casper
                    .evaluate("Casper-3", prompt_c, &final_prompt, &final_context),
            );

            let mut final_evals = vec![final_m?, final_b?, final_c?];

            // Annotate with Round 1 positions for audit trail
            for evaluation in &mut final_evals {
                if let Some(first_position) = first_round
                    .iter()
                    .find(|first| first.node_id == evaluation.node_id)
                {
                    let init_arg = if !first_position.rationale.is_empty() {
                        first_position.rationale.clone()
                    } else {
                        first_position.argument.clone()
                    };
                    evaluation.initial_argument = Some(init_arg);
                    evaluation.initial_vote = Some(first_position.vote.clone());
                    evaluation.initial_risk_score = Some(first_position.risk_score);
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
