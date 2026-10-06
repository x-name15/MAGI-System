//! # LLM Response Parser Helper
//!
//! Extracts structured JSON blocks from markdown fences and deserializes them
//! into `RawNodeOutput` with fallback to the json repair engine.

use super::json_repair_helper::repair_json_text;
use crate::error::MagiError;
use crate::llm::RawNodeOutput;

/// Parses and extracts structured JSON response from an LLM text completion.
pub fn parse_llm_json_response(raw_text: &str) -> Result<RawNodeOutput, MagiError> {
    let cleaned = if let Some(start) = raw_text.find("```json") {
        let after_start = &raw_text[start + 7..];
        if let Some(end) = after_start.find("```") {
            &after_start[..end]
        } else {
            after_start
        }
    } else if let Some(start) = raw_text.find('{') {
        if let Some(end) = raw_text.rfind('}') {
            &raw_text[start..=end]
        } else {
            raw_text
        }
    } else {
        raw_text
    };

    let trimmed = cleaned.trim();
    if let Ok(parsed) = serde_json::from_str::<RawNodeOutput>(trimmed) {
        return Ok(parsed);
    }

    // Try normalizing keys with leading dots e.g. {".vote": ...} -> {"vote": ...}
    let dot_normalized = trimmed.replace("\".", "\"");
    if let Ok(parsed) = serde_json::from_str::<RawNodeOutput>(&dot_normalized) {
        return Ok(parsed);
    }

    // Try repairing common LLM formatting glitches (missing quotes on keys, trailing commas)
    let repaired = repair_json_text(&dot_normalized);
    if let Ok(parsed) = serde_json::from_str::<RawNodeOutput>(&repaired) {
        return Ok(parsed);
    }

    // Fallback: parse as dynamic JSON Value to extract fields resiliently
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&repaired)
        .or_else(|_| serde_json::from_str::<serde_json::Value>(trimmed))
    {
        if let Some(obj) = val.as_object() {
            let vote = obj
                .get("vote")
                .or_else(|| obj.get(".vote"))
                .or_else(|| obj.get("verdict"))
                .and_then(|v| v.as_str())
                .unwrap_or("NEUTRAL")
                .to_string();

            let risk_score = obj
                .get("risk_score")
                .or_else(|| obj.get(".risk_score"))
                .or_else(|| obj.get("riskScore"))
                .or_else(|| obj.get("risk"))
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as u8;

            let rationale = obj
                .get("rationale")
                .or_else(|| obj.get(".rationale"))
                .or_else(|| obj.get("argument"))
                .or_else(|| obj.get("analysis"))
                .or_else(|| obj.get("reasoning"))
                .and_then(|v| v.as_str())
                .unwrap_or("Evaluated under node analytical lens")
                .to_string();

            let confidence = obj
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.95) as f32;

            let findings = obj
                .get("findings")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let cwe_flags = obj
                .get("cwe_flags")
                .or_else(|| obj.get("cwe"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            return Ok(RawNodeOutput {
                vote,
                risk_score,
                findings,
                rationale,
                confidence,
                cwe_flags,
            });
        }
    }

    Err(MagiError::Provider {
        node: "JSON_PARSER".to_string(),
        message: format!(
            "Failed to parse LLM structured JSON response. Content: {}",
            raw_text
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_fence() {
        let input = "Here is my evaluation:\n```json\n{\n  \"vote\": \"APPROVE\",\n  \"risk_score\": 2,\n  \"rationale\": \"Good code\"\n}\n```";
        let parsed = parse_llm_json_response(input).expect("parse succeeds");
        assert_eq!(parsed.vote, "APPROVE");
        assert_eq!(parsed.risk_score, 2);
    }
}
