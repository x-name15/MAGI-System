//! # Host Deliberation & Context Persistence
//!
//! Saves human-readable Markdown reports and full context payloads directly
//! into the host `deliberations/` directory, ensuring that even when ephemeral
//! Docker containers are killed, developer context, structured findings, and
//! deliberation verdicts remain immediately inspectable and reproducible.

use crate::llm::NodeEvaluation;
use std::fs;
use std::path::Path;

/// Writes a structured Markdown audit report and prompt context to the `deliberations/` directory.
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
    let dir = Path::new("deliberations");
    if !dir.exists() {
        let _ = fs::create_dir_all(dir);
    }

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
    let filename = format!("deliberation_{:04}_{}.md", deliberation_id, slug);
    let filepath = dir.join(filename);

    let mut report = String::new();
    report.push_str(&format!(
        "# 🧠 MAGI Deliberation #{:04}: {}\n\n",
        deliberation_id, title
    ));
    report.push_str(&format!("- **Category**: {}\n", category));
    report.push_str(&format!("- **Context Type**: {}\n", context_type));
    report.push_str(&format!("- **Consensus Verdict**: **{}**\n", verdict));
    report.push_str(&format!("- **Summary**: {}\n\n", summary));
    report.push_str("---\n\n");
    report.push_str("## 🧬 The Trinity Votes & Analytical Arguments\n\n");

    for eval in evaluations {
        report.push_str(&format!(
            "### Node: {} — Vote: `{}`\n\n",
            eval.node_id, eval.vote
        ));
        report.push_str(&format!("- **Risk Score**: {} / 10\n", eval.risk_score));
        report.push_str(&format!("- **Confidence**: {:.0}%\n", eval.confidence * 100.0));
        if !eval.model.is_empty() {
            report.push_str(&format!("- **Model**: `{}`\n", eval.model));
        }
        if !eval.prompt_version.is_empty() {
            report.push_str(&format!("- **Prompt Version**: `{}`\n", eval.prompt_version));
        }
        report.push_str(&format!(
            "- **Execution Latency**: {} ms\n",
            eval.execution_time_ms
        ));
        if !eval.cwe_flags.is_empty() {
            report.push_str(&format!(
                "- **CWE Flags Detected**: {}\n",
                eval.cwe_flags.join(", ")
            ));
        }

        // Render structured findings table if findings were reported
        if !eval.findings.is_empty() {
            report.push_str("\n#### Structured Findings:\n\n");
            report.push_str("| Category | Severity | Title | Impact | Recommendation |\n");
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
            report.push_str("\n");

            // Evidence details
            for (idx, f) in eval.findings.iter().enumerate() {
                if !f.evidence.is_empty() {
                    report.push_str(&format!("> **Evidence [{}] ({})**: {}\n\n", idx + 1, f.title, f.evidence));
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
                "\n#### 🔍 Ronda 1: Postura Inicial Independiente\n- **Voto Inicial**: `{}` | **Riesgo Inicial**: {} / 10\n\n{}\n\n",
                init_vote, init_risk, init_arg
            ));
            report.push_str(&format!(
                "#### ⚔️ Ronda 2: Dictamen Final tras Debate Cruzado\n- **Voto Final**: `{}` | **Riesgo Final**: {} / 10\n\n{}\n\n",
                eval.vote, eval.risk_score, rationale_display
            ));
        } else {
            report.push_str(&format!(
                "\n#### Analysis / Rationale:\n\n{}\n\n",
                rationale_display
            ));
        }
    }

    report.push_str("---\n\n");
    report.push_str("## 📄 Evaluated Context / Source Code\n\n```text\n");
    report.push_str(input_context);
    report.push_str("\n```\n");

    match fs::write(&filepath, report) {
        Ok(_) => println!(
            "[HOST PERSISTENCE] Deliberation and context saved: {}",
            filepath.display()
        ),
        Err(e) => eprintln!(
            "Warning: Unable to save host deliberation report {}: {}",
            filepath.display(),
            e
        ),
    }
}
