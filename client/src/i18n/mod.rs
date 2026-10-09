//! # Internationalization (i18n) Engine for MAGI System
//!
//! Loads localized JSON message catalogs (English and Spanish) and provides
//! dynamic language detection based on prompt and context contents.

use serde::Deserialize;
use std::sync::OnceLock;

/// Supported system languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Es,
}

impl Language {
    /// Detects active language from `MAGI_LANG` (or system `LANG`), defaulting to English.
    pub fn detect(_text: &str) -> Self {
        if let Ok(lang_override) = std::env::var("MAGI_LANG").or_else(|_| std::env::var("LANG")) {
            let lower = lang_override.to_lowercase();
            if lower.starts_with("es") || lower.contains("spanish") || lower.contains("español") {
                return Language::Es;
            } else if lower.starts_with("en") {
                return Language::En;
            }
        }
        Language::En
    }
}

/// Localized strings for Markdown deliberation reports.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ReportMessages {
    pub title_prefix: String,
    pub category: String,
    pub context_type: String,
    pub consensus_verdict: String,
    pub summary: String,
    pub trinity_header: String,
    pub node_prefix: String,
    pub vote_prefix: String,
    pub risk_score: String,
    pub confidence: String,
    pub model: String,
    pub prompt_version: String,
    pub execution_latency: String,
    pub cwe_flags: String,
    pub structured_findings: String,
    pub table_category: String,
    pub table_severity: String,
    pub table_title: String,
    pub table_impact: String,
    pub table_recommendation: String,
    pub evidence_prefix: String,
    pub round1_title: String,
    pub round1_vote: String,
    pub round1_risk: String,
    pub round2_title: String,
    pub round2_vote: String,
    pub round2_risk: String,
    pub analysis_title: String,
    pub context_header: String,
    pub saved_msg: String,
}

/// Localized strings for terminal UI, animation monitors, and telemetry.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct UiMessages {
    pub consulting_trinity: String,
    pub round1_eval: String,
    pub round2_debate: String,
    pub consensus_achieved: String,
    pub synthesis: String,
    pub monitors_section: String,
    pub rationales_title: String,
    pub central_dogma_section: String,
    pub phase_0_inquiry: String,
    pub phase_0_resolution: String,
    pub phase_1_inquiry: String,
    pub phase_1_resolution: String,
    pub phase_2_inquiry: String,
    pub phase_2_resolution: String,
    pub phase_3_inquiry: String,
    pub phase_3_resolution: String,
    pub phase_complete_inquiry: String,
    pub phase_complete_resolution: String,
    pub verdict_unavailable_title: String,
    pub verdict_unavailable_detail: String,
    pub verdict_veto_title: String,
    pub verdict_veto_detail: String,
    pub verdict_unanimous_approve_title: String,
    pub verdict_unanimous_approve_detail: String,
    pub verdict_majority_approve_title: String,
    pub verdict_majority_approve_detail: String,
    pub verdict_majority_reject_title: String,
    pub verdict_majority_reject_detail: String,
    pub verdict_unanimous_reject_title: String,
    pub verdict_unanimous_reject_detail: String,
    pub verdict_split_title: String,
    pub verdict_split_detail: String,
    pub verdict_unrecognized_detail: String,
    pub trajectory_format: String,
    pub cwe_detected: String,
    pub round1_initial_position: String,
    pub round2_post_debate_resolution: String,
    pub status_label: String,
    pub synth_label: String,
}

impl UiMessages {
    /// Formats voting trajectory delta between rounds.
    pub fn format_trajectory(
        &self,
        init_vote: &str,
        init_risk: u8,
        final_vote: &str,
        final_risk: u8,
    ) -> String {
        self.trajectory_format
            .replace("{init_vote}", init_vote)
            .replace("{init_risk}", &init_risk.to_string())
            .replace("{final_vote}", final_vote)
            .replace("{final_risk}", &final_risk.to_string())
    }
}

fn default_maintains_position_rationale() -> String {
    "Maintains initial position after reviewing peer arguments.".to_string()
}

fn default_rationale() -> String {
    "Evaluated under node analytical lens.".to_string()
}

/// Localized strings for Trinity debate and orchestration.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DebateMessages {
    pub prompt_final: String,
    pub prompt_intermediate: String,
    pub context_original_header: String,
    pub peer_positions_header: String,
    pub opening_instruction: String,
    pub incident_deliberation_prompt: String,
    pub lead_node_label: String,
    #[serde(default = "default_maintains_position_rationale")]
    pub maintains_position_rationale: String,
}

impl DebateMessages {
    /// Formats a multi-round debate prompt with localized templates.
    pub fn format_prompt(
        &self,
        is_final: bool,
        user_prompt: &str,
        round_num: u8,
        total_rounds: u8,
    ) -> String {
        let tmpl = if is_final {
            &self.prompt_final
        } else {
            &self.prompt_intermediate
        };
        tmpl.replace("{user_prompt}", user_prompt)
            .replace("{round_num}", &round_num.to_string())
            .replace("{total_rounds}", &total_rounds.to_string())
    }

    /// Formats the peer context payload with localized headers.
    pub fn format_context(
        &self,
        context_payload: &str,
        peer_positions: &str,
        round_num: u8,
    ) -> String {
        let orig = self
            .context_original_header
            .replace("{context_payload}", context_payload);
        let peers = self
            .peer_positions_header
            .replace("{round_num}", &round_num.to_string())
            .replace("{peer_positions}", peer_positions);
        format!("{}\n\n{}", orig, peers)
    }

    /// Formats specialist opening triage instruction.
    pub fn format_opening_instruction(&self, lead_node: &str) -> String {
        self.opening_instruction.replace("{lead_node}", lead_node)
    }

    /// Formats the incident full Trinity deliberation prompt.
    pub fn format_incident_deliberation(&self, lead_node: &str, argument: &str) -> String {
        self.incident_deliberation_prompt
            .replace("{lead_node}", lead_node)
            .replace("{argument}", argument)
    }

    /// Formats the lead node context label.
    pub fn format_lead_node_label(&self, lead_node: &str) -> String {
        self.lead_node_label.replace("{lead_node}", lead_node)
    }
}

/// Localized prompt formatting templates and schema instructions.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PromptMessages {
    pub audit_query_label: String,
    pub code_context_label: String,
    pub language_instruction: String,
    pub json_schema_instruction: String,
    #[serde(default = "default_rationale")]
    pub default_rationale: String,
}

impl PromptMessages {
    /// Formats the audit query, code context, and schema output constraints.
    pub fn format_audit_prompt(&self, user_prompt: &str, context_payload: &str) -> String {
        format!(
            "{}:\n{}\n\n{}:\n{}\n\n{}\n\n{}",
            self.audit_query_label,
            user_prompt,
            self.code_context_label,
            context_payload,
            self.language_instruction,
            self.json_schema_instruction
        )
    }
}

/// Complete i18n bundle.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct I18nBundle {
    pub report: ReportMessages,
    pub ui: UiMessages,
    pub debate: DebateMessages,
    pub prompt: PromptMessages,
}

static EN_BUNDLE: OnceLock<I18nBundle> = OnceLock::new();
static ES_BUNDLE: OnceLock<I18nBundle> = OnceLock::new();

/// Returns the static i18n bundle for the given language.
pub fn get_bundle(lang: Language) -> &'static I18nBundle {
    match lang {
        Language::En => EN_BUNDLE.get_or_init(|| {
            let json_str = include_str!("../../i18n/en.json");
            serde_json::from_str(json_str).expect("Failed to parse English i18n bundle")
        }),
        Language::Es => ES_BUNDLE.get_or_init(|| {
            let json_str = include_str!("../../i18n/es.json");
            serde_json::from_str(json_str).expect("Failed to parse Spanish i18n bundle")
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_bundles_load() {
        let en = get_bundle(Language::En);
        let es = get_bundle(Language::Es);

        assert_eq!(en.report.category, "Category");
        assert_eq!(es.report.category, "Categoría");
        assert_eq!(en.ui.synthesis, "SYNTHESIS");
        assert_eq!(es.ui.synthesis, "SÍNTESIS");
        assert!(en.debate.prompt_final.contains("FINAL DEBATE ROUND"));
        assert!(es.debate.prompt_final.contains("DEBATE RONDA FINAL"));
        assert!(!en.debate.maintains_position_rationale.is_empty());
        assert!(!es.debate.maintains_position_rationale.is_empty());
        assert!(!en.prompt.default_rationale.is_empty());
        assert!(!es.prompt.default_rationale.is_empty());
    }

    #[test]
    fn test_language_detection() {
        std::env::set_var("MAGI_LANG", "es");
        assert_eq!(Language::detect("any text"), Language::Es);
        std::env::set_var("MAGI_LANG", "en");
        assert_eq!(Language::detect("any text"), Language::En);
        std::env::remove_var("MAGI_LANG");
        assert_eq!(Language::detect("any text"), Language::En);
    }

    #[test]
    fn test_debate_formatting() {
        let en = get_bundle(Language::En);
        let prompt = en.debate.format_prompt(true, "Check code", 3, 3);
        assert!(prompt.contains("FINAL DEBATE ROUND (3 of 3)"));
        assert!(prompt.contains("Check code"));

        let es = get_bundle(Language::Es);
        let prompt_es = es.debate.format_prompt(false, "Revisa código", 2, 4);
        assert!(prompt_es.contains("DEBATE RONDA 2 de 4"));
        assert!(prompt_es.contains("Revisa código"));
    }

    #[test]
    fn test_audit_prompt_formatting() {
        let en = get_bundle(Language::En);
        let p_en = en.prompt.format_audit_prompt("Review this", "fn main() {}");
        assert!(p_en.contains("AUDIT QUERY:"));
        assert!(p_en.contains("Review this"));
        assert!(p_en.contains("CODE CONTEXT:"));
        assert!(p_en.contains("fn main() {}"));
        assert!(p_en.contains("LANGUAGE INSTRUCTION:"));

        let es = get_bundle(Language::Es);
        let p_es = es.prompt.format_audit_prompt("Revisa esto", "fn main() {}");
        assert!(p_es.contains("CONSULTA DE AUDITORÍA:"));
        assert!(p_es.contains("Revisa esto"));
        assert!(p_es.contains("CONTEXTO DE CÓDIGO:"));
        assert!(p_es.contains("fn main() {}"));
        assert!(p_es.contains("INSTRUCCIÓN DE IDIOMA:"));
    }
}
