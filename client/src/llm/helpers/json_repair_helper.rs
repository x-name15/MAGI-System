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
            let key_only = candidate_key.trim_start_matches(['{', '[', ' ', ',']);
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
    let cleaned_commas = dot_normalized
        .replace(",\n}", "\n}")
        .replace(",\r\n}", "\r\n}")
        .replace(",\n  }", "\n  }")
        .replace(",\r\n  }", "\r\n  }")
        .replace(",\n    }", "\n    }")
        .replace(",\n]", "\n]")
        .replace(",\r\n]", "\r\n]")
        .replace(",\n  ]", "\n  ]");

    // Balance quotes and close open brackets/braces if the completion was truncated
    let mut balanced = cleaned_commas;
    let mut in_string = false;
    let mut escape = false;
    let mut stack = Vec::new();

    for ch in balanced.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            match ch {
                '{' => stack.push('}'),
                '[' => stack.push(']'),
                '}' => {
                    if let Some(&top) = stack.last() {
                        if top == '}' {
                            stack.pop();
                        }
                    }
                }
                ']' => {
                    if let Some(&top) = stack.last() {
                        if top == ']' {
                            stack.pop();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if in_string {
        balanced.push('"');
    }

    // Strip trailing comma before closing if text ended with a dangling comma
    let trimmed_end = balanced.trim_end();
    if let Some(stripped) = trimmed_end.strip_suffix(',') {
        balanced = stripped.to_string();
    }

    while let Some(closing) = stack.pop() {
        balanced.push(closing);
    }

    balanced
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

    #[test]
    fn test_repair_truncated_json_unclosed_object() {
        let input = "{\n  \"vote\": \"APPROVE\",\n  \"risk_score\": 2,\n  \"confidence\": 0.9";
        let output = repair_json_text(input);
        assert!(output.ends_with('}'));
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["vote"], "APPROVE");
        assert_eq!(parsed["risk_score"], 2);
    }

    #[test]
    fn test_repair_truncated_json_unclosed_string_and_array() {
        let input =
            "{\n  \"vote\": \"APPROVE\",\n  \"findings\": [\"race condition\", \"buffer over";
        let output = repair_json_text(input);
        assert!(output.ends_with("]}"));
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["vote"], "APPROVE");
        assert_eq!(parsed["findings"][0], "race condition");
        assert_eq!(parsed["findings"][1], "buffer over");
    }
}
