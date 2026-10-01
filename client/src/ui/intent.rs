//! # Natural Intent Processor for MAGI System
//!
//! Understands natural language inputs, dragged file paths, and general developer queries
//! without requiring strict CLI subcommands or rigid syntax.

use std::path::{Path, PathBuf};

/// Inferred intent from raw user input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferredIntent {
    /// Case 1: Proposal / Idea assessment
    IdeaAssessment { path: PathBuf, question: String },
    /// Case 2: Code maintenance under rules/guidelines
    CodeMaintenance {
        code_path: PathBuf,
        guidelines_path: Option<PathBuf>,
        instructions: String,
    },
    /// Case 3: Error / Incident triage to the single lead node
    ErrorTriage {
        error_text: String,
        code_context: Option<PathBuf>,
    },
    /// General multi-agent deliberation on arbitrary code, text, or architectural dilemma
    UniversalDeliberation {
        prompt: String,
        context_payload: String,
        context_type: String,
    },
    /// Built-in interactive console commands
    SystemCommand(String),
}

/// Analyzes arbitrary raw text input and resolves the developer's intent.
pub fn process_user_intent(raw_input: &str) -> InferredIntent {
    let trimmed = raw_input.trim();

    // Check system console commands
    match trimmed.to_lowercase().as_str() {
        "exit" | "quit" | "q" => return InferredIntent::SystemCommand("exit".to_string()),
        "clear" | "cls" => return InferredIntent::SystemCommand("clear".to_string()),
        "history" | "hist" => return InferredIntent::SystemCommand("history".to_string()),
        "status" | "ping" => return InferredIntent::SystemCommand("status".to_string()),
        "help" | "?" => return InferredIntent::SystemCommand("help".to_string()),
        _ => {}
    }

    // Try extracting any file paths referenced in the input
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let mut found_files: Vec<PathBuf> = Vec::new();

    for word in &words {
        let clean_word = word.trim_matches(|c| c == '\'' || c == '"' || c == ',' || c == ';');
        let path = Path::new(clean_word);
        if path.exists() && path.is_file() {
            found_files.push(path.to_path_buf());
        }
    }

    let lower = trimmed.to_lowercase();

    // 1. Check if it's an Error / Stack Trace / Exception
    let is_error = lower.contains("error")
        || lower.contains("exception")
        || lower.contains("panic")
        || lower.contains("traceback")
        || lower.contains("failed")
        || lower.contains("segfault")
        || lower.contains("fatal:");

    if is_error {
        let code_context = found_files.into_iter().find(|p| {
            p.extension()
                .map(|ext| ext != "log" && ext != "txt")
                .unwrap_or(true)
        });

        return InferredIntent::ErrorTriage {
            error_text: trimmed.to_string(),
            code_context,
        };
    }

    // 2. Check if it's a Markdown Idea / Proposal / Viability review
    if let Some(md_file) = found_files.iter().find(|p| {
        p.extension()
            .map(|ext| ext == "md" || ext == "markdown")
            .unwrap_or(false)
    }) {
        let question = if lower.contains("viable")
            || lower.contains("revisa")
            || lower.contains("mira")
            || lower.contains("evalua")
            || lower.contains("idea")
        {
            trimmed.to_string()
        } else {
            "Evaluate technical viability, architecture, security risks, and implementation effort of this proposal.".to_string()
        };

        return InferredIntent::IdeaAssessment {
            path: md_file.clone(),
            question,
        };
    }

    // 3. Check if it's Code Maintenance under guidelines
    let is_maintenance = lower.contains("maintain")
        || lower.contains("mantener")
        || lower.contains("directrices")
        || lower.contains("guidelines")
        || lower.contains("refactor")
        || lower.contains("reglas");

    if is_maintenance && !found_files.is_empty() {
        let code_file = found_files[0].clone();
        let guidelines_file = found_files.get(1).cloned();

        return InferredIntent::CodeMaintenance {
            code_path: code_file,
            guidelines_path: guidelines_file,
            instructions: trimmed.to_string(),
        };
    }

    // 4. Default: Universal Deliberation
    // If a file was referenced, load its contents; otherwise evaluate the text query directly
    if let Some(first_file) = found_files.into_iter().next() {
        let content = std::fs::read_to_string(&first_file).unwrap_or_default();
        let context_type = first_file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("TEXT")
            .to_uppercase();

        InferredIntent::UniversalDeliberation {
            prompt: trimmed.to_string(),
            context_payload: content,
            context_type,
        }
    } else {
        InferredIntent::UniversalDeliberation {
            prompt: trimmed.to_string(),
            context_payload: String::new(),
            context_type: "ABSTRACT_QUERY".to_string(),
        }
    }
}
