//! # Hybrid Deliberation Archive Loader
//!
//! Loads and merges past deliberation records from both the host `deliberations/`
//! directory and SpacetimeDB in-memory tables. Ensures that past decisions remain
//! inspectable and searchable even if the database container is offline or purged.

use crate::ui::helpers::layout_helper::safe_truncate_str;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Summarized vote and analysis from an individual MAGI persona node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodePositionSummary {
    pub node_id: String,
    pub vote: String,
    pub risk_score: u8,
    pub confidence: f32,
    pub model: String,
    pub argument: String,
}

/// Unified deliberation archive entry combining disk metadata and DB state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeliberationHistoryEntry {
    pub id: u64,
    pub title: String,
    pub category: String,
    pub context_type: String,
    pub verdict: String,
    pub summary: String,
    pub status: String,
    pub node_votes: Vec<NodePositionSummary>,
    pub file_path: Option<PathBuf>,
    pub raw_content: Option<String>,
}

impl DeliberationHistoryEntry {
    /// Returns a short display verdict with a status indicator emoji/icon.
    pub fn verdict_badge(&self) -> (&str, &str) {
        let v = self.verdict.trim().to_uppercase();
        if v.contains("VETO") {
            ("VETO", "🛑")
        } else if v.contains("UNANIMOUS") && v.contains("APPROVE") {
            ("APPROVED (3-0)", "✅")
        } else if v.contains("APPROVE") {
            ("APPROVED (2-1)", "✓")
        } else if v.contains("UNANIMOUS") && v.contains("REJECT") {
            ("REJECTED (3-0)", "❌")
        } else if v.contains("REJECT") {
            ("REJECTED (2-1)", "✗")
        } else if v.contains("SPLIT") {
            ("SPLIT (1-1-1)", "⚖")
        } else if v.is_empty() {
            ("PENDING", "⏳")
        } else {
            (&self.verdict, "ℹ")
        }
    }

    /// Returns the full raw markdown content if present, or synthesizes a clean Markdown report
    /// from structured DB fields and node votes if no disk file was available.
    pub fn get_or_synthesize_markdown(&self) -> String {
        if let Some(ref raw) = self.raw_content {
            if !raw.trim().is_empty() {
                return raw.clone();
            }
        }

        // Synthesize full Markdown report from structured database metadata
        let mut md = format!(
            "# MAGI Trinity Deliberation #{:04}: {}\n\n",
            self.id, self.title
        );
        md.push_str(&format!("- **Category**: {}\n", self.category));
        md.push_str(&format!("- **Context Type**: {}\n", self.context_type));
        md.push_str(&format!("- **Consensus Verdict**: **{}**\n", self.verdict));
        if !self.summary.is_empty() {
            md.push_str(&format!("- **Summary**: {}\n", self.summary));
        }
        md.push_str("\n---\n\n## Trinity Node Evaluations\n\n");

        if self.node_votes.is_empty() {
            md.push_str("*(No individual node evaluation records found)*\n");
        } else {
            for node in &self.node_votes {
                md.push_str(&format!(
                    "### {} — Vote: `{}` (Risk: {}/10)\n\n",
                    node.node_id, node.vote, node.risk_score
                ));
                if node.confidence > 0.0 {
                    md.push_str(&format!(
                        "- **Confidence**: {:.0}%\n",
                        node.confidence * 100.0
                    ));
                }
                if !node.model.is_empty() {
                    md.push_str(&format!("- **Model**: `{}`\n", node.model));
                }
                if !node.argument.is_empty() {
                    md.push_str(&format!(
                        "\n#### Analysis / Rationale:\n\n{}\n\n",
                        node.argument
                    ));
                }
            }
        }

        md
    }
}

/// Parses a local Markdown deliberation report into a structured `DeliberationHistoryEntry`.
pub fn parse_deliberation_markdown(path: &Path, content: &str) -> Option<DeliberationHistoryEntry> {
    // 1. Extract ID from filename: deliberation_0001_slug.md or deliberation_1.md
    let filename = path.file_name()?.to_string_lossy();
    let mut id = 0u64;
    if filename.starts_with("deliberation_") {
        let after_prefix = filename.trim_start_matches("deliberation_");
        let id_part = after_prefix.split('_').next().unwrap_or(after_prefix);
        let id_str = id_part.trim_end_matches(".md");
        id = id_str.parse::<u64>().unwrap_or(0);
    }

    let mut title = String::new();
    let mut category = String::new();
    let mut context_type = String::new();
    let mut verdict = String::new();
    let mut summary = String::new();
    let mut node_votes = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut current_node: Option<NodePositionSummary> = None;
    let mut reading_argument = false;
    let mut current_arg_lines: Vec<String> = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Title from main H1 header (e.g. # Deliberación MAGI #0001: My Title)
        if trimmed.starts_with("# ") && title.is_empty() {
            let h1 = trimmed.trim_start_matches('#').trim();
            if let Some(colon_idx) = h1.find(':') {
                title = h1[colon_idx + 1..].trim().to_string();
                if id == 0 {
                    // Try parsing ID from header if filename lacked it
                    if let Some(hash_idx) = h1[..colon_idx].find('#') {
                        let candidate_id = &h1[..colon_idx][hash_idx + 1..].trim();
                        if let Ok(parsed) = candidate_id.parse::<u64>() {
                            id = parsed;
                        }
                    }
                }
            } else {
                title = h1.to_string();
            }
            continue;
        }

        // Section metadata
        if (trimmed.starts_with("- **Category**:") || trimmed.starts_with("- **Categoría**:"))
            && category.is_empty()
        {
            if let Some(colon_idx) = trimmed.find(':') {
                category = trimmed[colon_idx + 1..].trim().to_string();
            }
            continue;
        }

        if (trimmed.starts_with("- **Context Type**:")
            || trimmed.starts_with("- **Tipo de Contexto**:"))
            && context_type.is_empty()
        {
            if let Some(colon_idx) = trimmed.find(':') {
                context_type = trimmed[colon_idx + 1..].trim().to_string();
            }
            continue;
        }

        if (trimmed.starts_with("- **Consensus Verdict**:")
            || trimmed.starts_with("- **Veredicto de Consenso**:"))
            && verdict.is_empty()
        {
            if let Some(colon_idx) = trimmed.find(':') {
                let raw_v = trimmed[colon_idx + 1..].trim();
                verdict = raw_v.trim_matches('*').trim().to_string();
            }
            continue;
        }

        if (trimmed.starts_with("- **Summary**:") || trimmed.starts_with("- **Resumen**:"))
            && summary.is_empty()
        {
            if let Some(colon_idx) = trimmed.find(':') {
                summary = trimmed[colon_idx + 1..].trim().to_string();
            }
            continue;
        }

        // Detect node section: ### Node Melchior-1: ... or ### Melchior-1: ...
        if trimmed.starts_with("### ") {
            if let Some(mut prev) = current_node.take() {
                if !current_arg_lines.is_empty() {
                    prev.argument = current_arg_lines.join(" ").trim().to_string();
                    current_arg_lines.clear();
                }
                node_votes.push(prev);
            }
            reading_argument = false;

            let header_text = trimmed.trim_start_matches('#').trim();
            let mut node_id = String::new();
            if header_text.contains("Melchior") {
                node_id = "Melchior-1".to_string();
            } else if header_text.contains("Balthasar") {
                node_id = "Balthasar-2".to_string();
            } else if header_text.contains("Casper") {
                node_id = "Casper-3".to_string();
            }

            // Extract vote from header if formatted as "— VOTE: `APPROVE`"
            let mut vote = "NEUTRAL".to_string();
            if let Some(backtick_start) = header_text.find('`') {
                if let Some(backtick_end) = header_text[backtick_start + 1..].find('`') {
                    vote = header_text[backtick_start + 1..backtick_start + 1 + backtick_end]
                        .trim()
                        .to_string();
                }
            }

            if !node_id.is_empty() {
                current_node = Some(NodePositionSummary {
                    node_id,
                    vote,
                    risk_score: 5,
                    confidence: 0.95,
                    model: String::new(),
                    argument: String::new(),
                });
            }
            continue;
        }

        // Parse node fields
        if let Some(ref mut node) = current_node {
            if trimmed.starts_with("- **Risk Score**:")
                || trimmed.starts_with("- **Nivel de Riesgo**:")
            {
                if let Some(colon_idx) = trimmed.find(':') {
                    let val_str = trimmed[colon_idx + 1..].trim();
                    let num_str = val_str.split('/').next().unwrap_or(val_str).trim();
                    if let Ok(score) = num_str.parse::<u8>() {
                        node.risk_score = score;
                    }
                }
                continue;
            }

            if trimmed.starts_with("- **Confidence**:") || trimmed.starts_with("- **Confianza**:") {
                if let Some(colon_idx) = trimmed.find(':') {
                    let val_str = trimmed[colon_idx + 1..].trim().trim_end_matches('%');
                    if let Ok(conf) = val_str.parse::<f32>() {
                        node.confidence = conf / 100.0;
                    }
                }
                continue;
            }

            if trimmed.starts_with("- **Model**:") || trimmed.starts_with("- **Modelo**:") {
                if let Some(colon_idx) = trimmed.find(':') {
                    node.model = trimmed[colon_idx + 1..]
                        .trim()
                        .trim_matches('`')
                        .to_string();
                }
                continue;
            }

            if trimmed.starts_with("- **Analysis / Rationale**:")
                || trimmed.starts_with("- **Análisis / Justificación**:")
            {
                reading_argument = true;
                continue;
            }

            // If reading argument lines
            if reading_argument {
                if trimmed.starts_with("---") || trimmed.starts_with("## ") {
                    reading_argument = false;
                } else if !trimmed.is_empty() && !trimmed.starts_with("- **") {
                    current_arg_lines.push(trimmed.to_string());
                }
            }
        }
    }

    if let Some(mut last_node) = current_node.take() {
        if !current_arg_lines.is_empty() {
            last_node.argument = current_arg_lines.join(" ").trim().to_string();
        }
        node_votes.push(last_node);
    }

    let status = if !verdict.is_empty() {
        "RESOLVED".to_string()
    } else {
        "COMPLETED".to_string()
    };

    Some(DeliberationHistoryEntry {
        id,
        title: if title.is_empty() {
            filename.to_string()
        } else {
            title
        },
        category: if category.is_empty() {
            "General".to_string()
        } else {
            category
        },
        context_type: if context_type.is_empty() {
            "audit".to_string()
        } else {
            context_type
        },
        verdict,
        summary,
        status,
        node_votes,
        file_path: Some(path.to_path_buf()),
        raw_content: Some(content.to_string()),
    })
}

/// Scans the given `deliberations/` directory and parses all Markdown reports.
pub fn load_from_disk(deliberations_dir: &Path) -> Vec<DeliberationHistoryEntry> {
    if !deliberations_dir.exists() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    if let Ok(dir_entries) = fs::read_dir(deliberations_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name.starts_with("deliberation_") && name.ends_with(".md") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(parsed) = parse_deliberation_markdown(&path, &content) {
                            entries.push(parsed);
                        }
                    }
                }
            }
        }
    }

    entries.sort_by(|a, b| b.id.cmp(&a.id));
    entries
}

/// Loads recent deliberation records from SpacetimeDB with short timeout.
pub async fn load_from_spacetimedb(
    db_client: &crate::db::SpacetimeClient,
    limit: usize,
) -> Vec<DeliberationHistoryEntry> {
    let records = match tokio::time::timeout(
        std::time::Duration::from_secs(3),
        db_client.list_history(limit),
    )
    .await
    {
        Ok(Ok(recs)) => recs,
        _ => return Vec::new(),
    };

    let mut entries = Vec::new();
    for r in records {
        let details = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            db_client.get_deliberation_details(r.id),
        )
        .await
        .ok()
        .and_then(|res| res.ok())
        .flatten();

        let (node_votes, verdict, summary) = if let Some((_, evals, consensus)) = details {
            let votes = evals
                .into_iter()
                .map(|e| NodePositionSummary {
                    node_id: e.node_id,
                    vote: e.vote,
                    risk_score: e.risk_score,
                    confidence: e.confidence,
                    model: String::new(),
                    argument: e.argument,
                })
                .collect();
            let v = consensus
                .as_ref()
                .map(|c| c.verdict.clone())
                .unwrap_or_else(|| r.status.clone());
            let s = consensus
                .as_ref()
                .map(|c| c.summary.clone())
                .unwrap_or_default();
            (votes, v, s)
        } else {
            (Vec::new(), r.status.clone(), String::new())
        };

        entries.push(DeliberationHistoryEntry {
            id: r.id,
            title: r.title,
            category: r.context_type.clone(),
            context_type: r.context_type,
            verdict,
            summary,
            status: r.status,
            node_votes,
            file_path: None,
            raw_content: None,
        });
    }

    entries
}

/// Merges deliberation history from both disk files and SpacetimeDB without duplicates.
pub async fn load_hybrid(
    db_client: Option<&crate::db::SpacetimeClient>,
    deliberations_dir: &Path,
    limit: usize,
) -> Vec<DeliberationHistoryEntry> {
    let mut map: std::collections::HashMap<u64, DeliberationHistoryEntry> =
        std::collections::HashMap::new();

    // 1. Load from disk first (zero latency)
    let disk_entries = load_from_disk(deliberations_dir);
    for entry in disk_entries {
        map.insert(entry.id, entry);
    }

    // 2. Load from SpacetimeDB and enrich/insert
    if let Some(client) = db_client {
        let db_entries = load_from_spacetimedb(client, limit).await;
        for mut db_entry in db_entries {
            // Ensure raw_content is always populated (synthesize from DB if disk file is missing)
            if db_entry.raw_content.is_none() {
                let synthesized = db_entry.get_or_synthesize_markdown();
                db_entry.raw_content = Some(synthesized.clone());

                // Automatically persist missing Markdown file to host disk
                if !map.contains_key(&db_entry.id) {
                    if !deliberations_dir.exists() {
                        let _ = fs::create_dir_all(deliberations_dir);
                    }
                    let sanitized = db_entry
                        .title
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
                    let clean = sanitized.trim_matches('_');
                    let truncated_slug = safe_truncate_str(clean, 48).trim_end_matches('_');
                    let slug = if truncated_slug.is_empty() {
                        "deliberation"
                    } else {
                        truncated_slug
                    };
                    let fname = format!("deliberation_{:04}_{}.md", db_entry.id, slug);
                    let target_path = deliberations_dir.join(fname);
                    if !target_path.exists() {
                        let _ = fs::write(&target_path, &synthesized);
                        db_entry.file_path = Some(target_path);
                    }
                }
            }

            map.entry(db_entry.id)
                .and_modify(|existing| {
                    if existing.verdict.is_empty() {
                        existing.verdict = db_entry.verdict.clone();
                    }
                    if existing.node_votes.is_empty() {
                        existing.node_votes = db_entry.node_votes.clone();
                    }
                    if existing.raw_content.is_none() {
                        existing.raw_content = db_entry.raw_content.clone();
                    }
                    if existing.file_path.is_none() {
                        existing.file_path = db_entry.file_path.clone();
                    }
                })
                .or_insert(db_entry);
        }
    }

    let mut all: Vec<DeliberationHistoryEntry> = map.into_values().collect();
    all.sort_by(|a, b| b.id.cmp(&a.id));
    if all.len() > limit {
        all.truncate(limit);
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deliberation_markdown() {
        let sample = r#"# MAGI Deliberation #0042: OAuth Token Encapsulation

- **Category**: Case 2: Code Maintenance
- **Context Type**: diff
- **Consensus Verdict**: **APPROVED_UNANIMOUS**
- **Summary**: All nodes approved the closure approach.

---

## The Trinity Votes & Analytical Arguments

### Node Melchior-1: Melchior-1 — Vote: `APPROVE`

- **Risk Score**: 2 / 10
- **Confidence**: 95%
- **Model**: `claude-3.5-sonnet`
- **Analysis / Rationale**:
Architecture adheres to solid encapsulation without window pollution.

### Node Balthasar-2: Balthasar-2 — Vote: `APPROVE`

- **Risk Score**: 1 / 10
- **Confidence**: 98%
- **Model**: `claude-3.5-sonnet`
- **Analysis / Rationale**:
No credential exposure found.
"#;

        let path = PathBuf::from("deliberations/deliberation_0042_oauth_token.md");
        let parsed = parse_deliberation_markdown(&path, sample).expect("failed to parse markdown");

        assert_eq!(parsed.id, 42);
        assert_eq!(parsed.title, "OAuth Token Encapsulation");
        assert_eq!(parsed.category, "Case 2: Code Maintenance");
        assert_eq!(parsed.context_type, "diff");
        assert_eq!(parsed.verdict, "APPROVED_UNANIMOUS");
        assert_eq!(parsed.node_votes.len(), 2);
        assert_eq!(parsed.node_votes[0].node_id, "Melchior-1");
        assert_eq!(parsed.node_votes[0].vote, "APPROVE");
        assert_eq!(parsed.node_votes[0].risk_score, 2);
        assert_eq!(parsed.node_votes[1].node_id, "Balthasar-2");
        assert_eq!(parsed.node_votes[1].risk_score, 1);
    }

    #[test]
    fn test_get_or_synthesize_markdown() {
        let entry = DeliberationHistoryEntry {
            id: 99,
            title: "Test DB Deliberation".to_string(),
            category: "DILEMMA".to_string(),
            context_type: "DILEMMA".to_string(),
            verdict: "APPROVED_MAJORITY".to_string(),
            summary: "2-1 consensus reached.".to_string(),
            status: "RESOLVED".to_string(),
            node_votes: vec![
                NodePositionSummary {
                    node_id: "Melchior-1".to_string(),
                    vote: "APPROVE".to_string(),
                    risk_score: 3,
                    confidence: 0.9,
                    model: "anthropic/claude".to_string(),
                    argument: "Solid architectural pattern.".to_string(),
                },
                NodePositionSummary {
                    node_id: "Balthasar-2".to_string(),
                    vote: "REJECT".to_string(),
                    risk_score: 6,
                    confidence: 0.8,
                    model: "openai/gpt-4o".to_string(),
                    argument: "Potential race condition.".to_string(),
                },
            ],
            file_path: None,
            raw_content: None,
        };

        let synthesized = entry.get_or_synthesize_markdown();
        assert!(synthesized.contains("# MAGI Trinity Deliberation #0099: Test DB Deliberation"));
        assert!(synthesized.contains("- **Consensus Verdict**: **APPROVED_MAJORITY**"));
        assert!(synthesized.contains("### Melchior-1 — Vote: `APPROVE` (Risk: 3/10)"));
        assert!(synthesized.contains("Solid architectural pattern."));
        assert!(synthesized.contains("### Balthasar-2 — Vote: `REJECT` (Risk: 6/10)"));
    }
}
