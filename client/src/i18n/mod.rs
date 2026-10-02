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
    /// Detects language from prompt/context text or `MAGI_LANG` environment variable.
    pub fn detect(text: &str) -> Self {
        if let Ok(lang_override) = std::env::var("MAGI_LANG") {
            let lower = lang_override.to_lowercase();
            if lower.starts_with("es") {
                return Language::Es;
            } else if lower.starts_with("en") {
                return Language::En;
            }
        }
        if crate::llm::is_spanish_text(text) {
            Language::Es
        } else {
            Language::En
        }
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

/// Localized strings for terminal UI and telemetry.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct UiMessages {
    pub consulting_trinity: String,
    pub round1_eval: String,
    pub round2_debate: String,
    pub consensus_achieved: String,
    pub synthesis: String,
}

/// Complete i18n bundle.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct I18nBundle {
    pub report: ReportMessages,
    pub ui: UiMessages,
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
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(
            Language::detect("Revisa la función de autenticación"),
            Language::Es
        );
        assert_eq!(
            Language::detect("Audit this memory safety bug in the auth handler"),
            Language::En
        );
    }
}
