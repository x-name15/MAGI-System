//! # Host Deliberation & Context Persistence
//!
//! Saves human-readable Markdown reports and full context payloads directly
//! into the host `deliberations/` directory, ensuring that even when ephemeral
//! Docker containers are killed, developer context, structured findings, and
//! deliberation verdicts remain immediately inspectable and reproducible.

use crate::i18n::{get_bundle, Language};
use crate::llm::NodeEvaluation;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the next sequential deliberation ID based on files in `deliberations/`.
pub fn get_next_local_deliberation_id() -> u64 {
    let dir = Path::new("deliberations");
    if !dir.exists() {
        return 1;
    }
    let mut max_id = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.starts_with("deliberation_") && filename.ends_with(".md") {
                let parts: Vec<&str> = filename.split('_').collect();
                if parts.len() >= 2 {
                    if let Ok(id) = parts[1].parse::<u64>() {
                        if id > max_id {
                            max_id = id;
                        }
                    }
                }
            }
        }
    }
    max_id + 1
}

/// Writes a structured Markdown audit report and prompt context to the `deliberations/` directory.
#[allow(clippy::too_many_arguments)]
pub fn save_host_deliberation_report_opts(
    deliberation_id: u64,
    title: &str,
    category: &str,
    context_type: &str,
    input_context: &str,
    evaluations: &[NodeEvaluation],
    verdict: &str,
    summary: &str,
    silent: bool,
) -> (u64, PathBuf) {
    let dir = Path::new("deliberations");
    if !dir.exists() {
        let _ = fs::create_dir_all(dir);
    }

    let effective_id = if deliberation_id == 0 {
        get_next_local_deliberation_id()
    } else {
        deliberation_id
    };

    // Detect language from title, summary, or input context
    let combined_text = format!("{} {} {}", title, summary, input_context);
    let lang = Language::detect(&combined_text);
    let bundle = get_bundle(lang);
    let r = &bundle.report;

    let sanitized_title = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let clean_title = sanitized_title.trim_matches('_');
    let slug = if clean_title.is_empty() {
        "deliberation"
    } else {
        clean_title
    };
    let filename = format!("deliberation_{:04}_{}.md", effective_id, slug);
    let filepath = dir.join(filename);

    let mut report = String::new();
    report.push_str(&format!(
        "# {} #{:04}: {}\n\n",
        r.title_prefix, effective_id, title
    ));
    report.push_str(&format!("- **{}**: {}\n", r.category, category));
    report.push_str(&format!("- **{}**: {}\n", r.context_type, context_type));
    report.push_str(&format!("- **{}**: **{}**\n", r.consensus_verdict, verdict));
    report.push_str(&format!("- **{}**: {}\n\n", r.summary, summary));
    report.push_str("---\n\n");
    report.push_str(&format!("## {}\n\n", r.trinity_header));

    for eval in evaluations {
        report.push_str(&format!(
            "### {}: {} — {}: `{}`\n\n",
            r.node_prefix, eval.node_id, r.vote_prefix, eval.vote
        ));
        report.push_str(&format!(
            "- **{}**: {} / 10\n",
            r.risk_score, eval.risk_score
        ));
        report.push_str(&format!(
            "- **{}**: {:.0}%\n",
            r.confidence,
            eval.confidence * 100.0
        ));
        if !eval.model.is_empty() {
            report.push_str(&format!("- **{}**: `{}`\n", r.model, eval.model));
        }
        if !eval.prompt_version.is_empty() {
            report.push_str(&format!(
                "- **{}**: `{}`\n",
                r.prompt_version, eval.prompt_version
            ));
        }
        report.push_str(&format!(
            "- **{}**: {} ms\n",
            r.execution_latency, eval.execution_time_ms
        ));
        if !eval.cwe_flags.is_empty() {
            report.push_str(&format!(
                "- **{}**: {}\n",
                r.cwe_flags,
                eval.cwe_flags.join(", ")
            ));
        }

        // Render structured findings table if findings were reported
        if !eval.findings.is_empty() {
            report.push_str(&format!("\n#### {}:\n\n", r.structured_findings));
            report.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                r.table_category,
                r.table_severity,
                r.table_title,
                r.table_impact,
                r.table_recommendation
            ));
            report.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
            for f in &eval.findings {
                report.push_str(&format!(
                    "| `{}` | **{}** | {} | {} | {} |\n",
                    f.category,
                    f.severity.to_uppercase(),
                    f.title.replace('|', "\\|"),
                    f.impact.replace('|', "\\|"),
                    f.recommendation.replace('|', "\\|")
                ));
            }
            report.push('\n');

            // Evidence details
            for (idx, f) in eval.findings.iter().enumerate() {
                if !f.evidence.is_empty() {
                    report.push_str(&format!(
                        "> **{} [{}] ({})**: {}\n\n",
                        r.evidence_prefix,
                        idx + 1,
                        f.title,
                        f.evidence
                    ));
                }
            }
        }

        let rationale_display = if !eval.rationale.is_empty() {
            &eval.rationale
        } else {
            &eval.argument
        };

        if let Some(ref init_arg) = eval.initial_argument {
            let init_vote = eval.initial_vote.as_deref().unwrap_or("N/A");
            let init_risk = eval.initial_risk_score.unwrap_or(0);
            report.push_str(&format!(
                "\n#### {}\n- **{}**: `{}` | **{}**: {} / 10\n\n{}\n\n",
                r.round1_title, r.round1_vote, init_vote, r.round1_risk, init_risk, init_arg
            ));
            report.push_str(&format!(
                "#### {}\n- **{}**: `{}` | **{}**: {} / 10\n\n{}\n\n",
                r.round2_title,
                r.round2_vote,
                eval.vote,
                r.round2_risk,
                eval.risk_score,
                rationale_display
            ));
        } else {
            report.push_str(&format!(
                "\n#### {}:\n\n{}\n\n",
                r.analysis_title, rationale_display
            ));
        }
    }

    report.push_str("---\n\n");
    report.push_str(&format!("## {}\n\n```text\n", r.context_header));
    report.push_str(input_context);
    report.push_str("\n```\n");

    match fs::write(&filepath, report) {
        Ok(_) => {
            if !silent {
                eprintln!("[HOST PERSISTENCE] {}: {}", r.saved_msg, filepath.display());
            }
        }
        Err(e) => eprintln!(
            "Warning: Unable to save host deliberation report {}: {}",
            filepath.display(),
            e
        ),
    }

    (effective_id, filepath)
}

/// Backwards-compatible wrapper that prints the saved report location to stderr.
#[allow(clippy::too_many_arguments)]
pub fn save_host_deliberation_report(
    deliberation_id: u64,
    title: &str,
    category: &str,
    context_type: &str,
    input_context: &str,
    evaluations: &[NodeEvaluation],
    verdict: &str,
    summary: &str,
) {
    let _ = save_host_deliberation_report_opts(
        deliberation_id,
        title,
        category,
        context_type,
        input_context,
        evaluations,
        verdict,
        summary,
        false,
    );
}
