//! # JSON Repair Helper for LLM Completions
//!
//! Fixes common edge-case formatting defects emitted by various LLM models,
//! such as unquoted JSON keys and trailing commas before closing braces.

/// Heuristically repairs common LLM JSON syntax anomalies.
pub fn repair_json_text(text: &str) -> String {
    let mut repaired_lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        // Check for missing opening quote on a key: e.g. `key":` instead of `"key":`
        if let Some(colon_pos) = trimmed.find("\":") {
            let candidate_key = &trimmed[..colon_pos];
            let key_only =
                candidate_key.trim_start_matches(['{', '[', ' ', ',']);
            if !key_only.starts_with('"') && !key_only.is_empty() && !key_only.contains(' ') {
                let prefix_len = candidate_key.len() - key_only.len();
                let prefix = &candidate_key[..prefix_len];
                let rest = &trimmed[colon_pos..];
                repaired_lines.push(format!("{}{}\"{}{}", indent, prefix, key_only, rest));
                continue;
            }
        }
        repaired_lines.push(line.to_string());
    }
    let combined = repaired_lines.join("\n");
    // Normalize keys with leading dots e.g. {".vote": ...} -> {"vote": ...}
    let dot_normalized = combined.replace("\".", "\"");
    // Remove trailing commas before closing braces
    dot_normalized
        .replace(",\n}", "\n}")
        .replace(",\r\n}", "\r\n}")
        .replace(",\n  }", "\n  }")
        .replace(",\r\n  }", "\r\n  }")
        .replace(",\n    }", "\n    }")
        .replace(",\n]", "\n]")
        .replace(",\r\n]", "\r\n]")
        .replace(",\n  ]", "\n  ]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_trailing_comma() {
        let input = "{\n  \"vote\": \"APPROVE\",\n}";
        let output = repair_json_text(input);
        assert!(!output.contains(",\n}"));
    }

    #[test]
    fn test_repair_unquoted_key() {
        let input = "{\n  vote\": \"APPROVE\"\n}";
        let output = repair_json_text(input);
        assert!(output.contains("\"vote\": \"APPROVE\""));
    }
}
