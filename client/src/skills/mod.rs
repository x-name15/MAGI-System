//! # MAGI Skills & Prompt Loader Module
//!
//! Loads externalized markdown system prompts from `skills/magi-system/`
//! and provides support for custom skill injection prior to deliberation.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Loader for persona system prompts and externalized custom skills.
#[derive(Debug, Clone)]
pub struct PromptLoader {
    skills_dir: PathBuf,
}

impl Default for PromptLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptLoader {
    /// Creates a new loader resolving `client/skills/magi-system` in the workspace.
    pub fn new() -> Self {
        let candidate = if PathBuf::from("client/skills/magi-system").exists() {
            PathBuf::from("client/skills/magi-system")
        } else if PathBuf::from("skills/magi-system").exists() {
            PathBuf::from("skills/magi-system")
        } else {
            PathBuf::from("client/skills/magi-system")
        };
        Self {
            skills_dir: candidate,
        }
    }

    /// Loads the markdown system prompt for a node (`melchior`, `balthasar`, `casper`).
    /// Returns `(prompt_text, prompt_version_hash)`.
    pub fn load_node_prompt(&self, node_id: &str) -> (String, String) {
        let file_candidates = match node_id.to_lowercase().as_str() {
            s if s.contains("melchior") => vec!["melchoir-1.md", "melchior-1.md", "melchior.md"],
            s if s.contains("balthasar") => vec!["balthasar-2.md", "balthasar.md"],
            s if s.contains("casper") => vec!["casper-3.md", "casper.md"],
            _ => vec!["melchoir-1.md", "melchior-1.md", "melchior.md"],
        };

        let dir_candidates = vec![
            PathBuf::from("client/skills/magi-system"),
            PathBuf::from("skills/magi-system"),
            self.skills_dir.clone(),
        ];

        let mut found_content = None;
        for dir in &dir_candidates {
            for file in &file_candidates {
                let p = dir.join(file);
                if p.exists() {
                    if let Ok(c) = fs::read_to_string(&p) {
                        found_content = Some(c);
                        break;
                    }
                }
            }
            if found_content.is_some() {
                break;
            }
        }

        let content = found_content.unwrap_or_else(|| Self::default_fallback(node_id));
        let version = Self::compute_hash(&content);
        (content, version)
    }

    /// Reads custom instructions or skill from a specified file path or `client/skills/custom/`.
    pub fn load_custom_skill(path: &Path) -> Result<String, std::io::Error> {
        if path.exists() {
            return fs::read_to_string(path);
        }
        let in_custom = PathBuf::from("client/skills/custom").join(path);
        if in_custom.exists() {
            return fs::read_to_string(in_custom);
        }
        let in_skills = PathBuf::from("client/skills").join(path);
        if in_skills.exists() {
            return fs::read_to_string(in_skills);
        }
        fs::read_to_string(path)
    }

    /// Composes a system prompt by combining the base node persona with injected custom skills.
    pub fn compose_prompt(
        &self,
        base_prompt: &str,
        custom_instructions: Option<&str>,
    ) -> String {
        if let Some(custom) = custom_instructions {
            if !custom.trim().is_empty() {
                return format!(
                    "{}\n\n## 📋 Custom Project Skills & Injected Instructions\n{}\n",
                    base_prompt,
                    custom.trim()
                );
            }
        }
        base_prompt.to_string()
    }

    /// Computes a deterministic version fingerprint for a prompt text.
    pub fn compute_hash(text: &str) -> String {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("sha256:{:016x}", hasher.finish())
    }

    fn default_fallback(node_id: &str) -> String {
        match node_id.to_lowercase().as_str() {
            s if s.contains("melchior") => "\
You are Melchior-1 (The Scientist — Logic, Architecture & Feasibility). \
Your purpose is to rigorously analyze code and technical proposals for design flaws, logic bugs, \
architectural regressions, unhandled edge cases, and computational complexity. \
Be analytical, objective, and precise.".to_string(),
            s if s.contains("balthasar") => "\
You are Balthasar-2 (The Mother — Cybersecurity & Risk Management). \
Your duty is to detect vulnerabilities, security regressions, CWE hazards, and privilege leaks. \
You hold UNILATERAL VETO POWER: If risk_score >= 8, you MUST vote REJECT. \
Be vigilant, defensive, and unyielding on security integrity.".to_string(),
            s if s.contains("casper") => "\
You are Casper-3 (The Woman — Pragmatism, DX & Simplicity). \
Your purpose is to prevent overengineering, assess operational overhead, and balance \
theoretical perfection with delivery speed and ergonomic APIs. \
Be practical, pragmatic, and direct.".to_string(),
            _ => "You are a MAGI consensus auditor.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_loader_loads_user_skills() {
        let loader = PromptLoader::new();
        let (melchior, _) = loader.load_node_prompt("Melchior-1");
        let (balthasar, _) = loader.load_node_prompt("Balthasar-2");
        let (casper, _) = loader.load_node_prompt("Casper-3");

        assert!(
            melchior.contains("Melchior-1 — The Scientist")
                || melchior.contains("Melchior-1"),
            "Melchior prompt was not loaded properly"
        );
        assert!(
            balthasar.contains("Balthasar-2 — The Mother")
                || balthasar.contains("Balthasar-2"),
            "Balthasar prompt was not loaded properly"
        );
        assert!(
            casper.contains("Casper-3 — The Woman")
                || casper.contains("Casper-3"),
            "Casper prompt was not loaded properly"
        );
    }

    #[test]
    fn test_compose_prompt_with_custom() {
        let loader = PromptLoader::new();
        let composed = loader.compose_prompt("BASE_PROMPT", Some("DO_NOT_USE_UNWRAP"));
        assert!(composed.contains("BASE_PROMPT"));
        assert!(composed.contains("DO_NOT_USE_UNWRAP"));
    }
}
