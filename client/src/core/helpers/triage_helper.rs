//! # Incident Triage and Lead Node Selection Helper
//!
//! Provides deterministic heuristic classification for errors, panics, and stack traces
//! to route opening specialist evaluations to the most relevant MAGI persona.

/// Heuristically classifies an error text to select the lead specialist node.
pub fn select_lead_node_for_error(error_text: &str) -> &'static str {
    let lower = error_text.to_lowercase();

    // Security, Permissions, Tokens, Vulnerabilities -> Balthasar-2
    if lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("permission")
        || lower.contains("token")
        || lower.contains("auth")
        || lower.contains("cwe")
        || lower.contains("injection")
        || lower.contains("secret")
        || lower.contains("leak")
        || lower.contains("ssl")
        || lower.contains("certificate")
    {
        return "Balthasar-2";
    }

    // Configuration, Tooling, Environment, Dependencies, Missing Files -> Casper-3
    if lower.contains("not found")
        || lower.contains("command not found")
        || lower.contains("connection refused")
        || lower.contains("env")
        || lower.contains("missing file")
        || lower.contains("syntaxerror: unexpected")
        || lower.contains("docker")
        || lower.contains("package")
        || lower.contains("dependency")
        || lower.contains("cannot find module")
    {
        return "Casper-3";
    }

    // Logic, Panics, Concurrency, Architecture, Algorithms -> Melchior-1
    "Melchior-1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_lead_node_routing() {
        assert_eq!(
            select_lead_node_for_error("Error: Unauthorized token expired"),
            "Balthasar-2"
        );
        assert_eq!(
            select_lead_node_for_error("Error: command not found: spacetime"),
            "Casper-3"
        );
        assert_eq!(
            select_lead_node_for_error("panic: index out of bounds at thread worker"),
            "Melchior-1"
        );
    }
}
