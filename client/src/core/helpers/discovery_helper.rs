//! # Project Context Discovery Helper
//!
//! Automatically inspects the current repository workspace to detect
//! ecosystem manifests (Rust, Node/TypeScript, Go, Python, Java, etc.)
//! and formats a concise architectural context envelope for the Trinity nodes.

use std::fs;
use std::path::{Path, PathBuf};

/// Discovered metadata describing the target project ecosystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    /// High-level ecosystem name (e.g. "Rust", "Node.js (TypeScript)", "Go", "Python").
    pub ecosystem: String,
    /// Detected project or package name.
    pub name: Option<String>,
    /// Detected version if specified in manifest.
    pub version: Option<String>,
    /// Architectural traits (e.g. "Cargo Workspace (2 members)", "Next.js", "Go 1.22").
    pub details: Vec<String>,
    /// Root path where the manifest was located.
    pub root_dir: PathBuf,
}

impl ProjectContext {
    /// Produces a compact, single-line telemetry string for terminal banners and logs.
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("Ecosystem: {}", self.ecosystem)];
        if let Some(ref n) = self.name {
            parts.push(format!("Project: {}", n));
        }
        if let Some(ref v) = self.version {
            parts.push(format!("v{}", v));
        }
        if !self.details.is_empty() {
            parts.push(format!("Traits: [{}]", self.details.join(", ")));
        }
        parts.join(" | ")
    }

    /// Formats an architectural context header injected into Trinity LLM prompts.
    pub fn to_prompt_header(&self) -> String {
        let name_str = self.name.as_deref().unwrap_or("Unknown Project");
        let ver_str = self.version.as_deref().unwrap_or("unversioned");
        let traits_str = if self.details.is_empty() {
            "Standard ecosystem conventions".to_string()
        } else {
            self.details.join("; ")
        };

        format!(
            "## 🧭 Discovered Project Context\n\
            - **Target Ecosystem**: {}\n\
            - **Package Name**: {}\n\
            - **Declared Version**: {}\n\
            - **Detected Architecture/Traits**: {}\n\n",
            self.ecosystem, name_str, ver_str, traits_str
        )
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n")
            + "\n\n"
    }
}

/// Discovers project context starting from the current working directory or specified root.
/// Traverses up to 4 directory levels upward looking for project markers.
pub fn discover_project_context(start_dir: Option<&Path>) -> Option<ProjectContext> {
    let mut current = match start_dir {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    for _ in 0..5 {
        if let Some(ctx) = inspect_directory(&current) {
            return Some(ctx);
        }

        if let Some(parent) = current.parent() {
            if parent == current {
                break;
            }
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    None
}

/// Inspects a single directory for known manifest files.
fn inspect_directory(dir: &Path) -> Option<ProjectContext> {
    // 1. Rust: Cargo.toml
    let cargo_path = dir.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = fs::read_to_string(&cargo_path) {
            let mut name = None;
            let mut version = None;
            let mut edition = None;
            let mut is_workspace = false;
            let mut members = Vec::new();

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("[workspace]") {
                    is_workspace = true;
                }
                if name.is_none() && trimmed.starts_with("name") && trimmed.contains('=') {
                    name = extract_quoted_value(trimmed);
                }
                if version.is_none() && trimmed.starts_with("version") && trimmed.contains('=') {
                    version = extract_quoted_value(trimmed);
                }
                if edition.is_none() && trimmed.starts_with("edition") && trimmed.contains('=') {
                    edition = extract_quoted_value(trimmed);
                }
                if trimmed.starts_with('"') && trimmed.ends_with('"') && is_workspace {
                    if let Some(val) = extract_quoted_value(trimmed) {
                        members.push(val);
                    }
                }
            }

            let mut details = Vec::new();
            if let Some(ed) = edition {
                details.push(format!("Edition {}", ed));
            }
            if is_workspace {
                if !members.is_empty() {
                    details.push(format!("Workspace ({} members)", members.len()));
                } else {
                    details.push("Workspace root".to_string());
                }
            }

            return Some(ProjectContext {
                ecosystem: "Rust (Cargo)".to_string(),
                name,
                version,
                details,
                root_dir: dir.to_path_buf(),
            });
        }
    }

    // 2. Node.js / TypeScript: package.json
    let pkg_path = dir.join("package.json");
    if pkg_path.exists() {
        if let Ok(content) = fs::read_to_string(&pkg_path) {
            let has_ts = dir.join("tsconfig.json").exists() || content.contains("\"typescript\"");
            let ecosystem = if has_ts {
                "Node.js (TypeScript)".to_string()
            } else {
                "Node.js (JavaScript)".to_string()
            };

            let mut name = None;
            let mut version = None;
            let mut details = Vec::new();

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                name = val["name"].as_str().map(|s| s.to_string());
                version = val["version"].as_str().map(|s| s.to_string());

                let deps = val["dependencies"].as_object();
                let dev_deps = val["devDependencies"].as_object();

                let check_dep = |dep_name: &str| -> bool {
                    deps.is_some_and(|d| d.contains_key(dep_name))
                        || dev_deps.is_some_and(|d| d.contains_key(dep_name))
                };

                if check_dep("next") {
                    details.push("Next.js".to_string());
                } else if check_dep("react") {
                    details.push("React".to_string());
                } else if check_dep("express") {
                    details.push("Express".to_string());
                } else if check_dep("@nestjs/core") {
                    details.push("NestJS".to_string());
                }
            }

            return Some(ProjectContext {
                ecosystem,
                name,
                version,
                details,
                root_dir: dir.to_path_buf(),
            });
        }
    }

    // 3. Go: go.mod
    let go_path = dir.join("go.mod");
    if go_path.exists() {
        if let Ok(content) = fs::read_to_string(&go_path) {
            let mut module_name = None;
            let mut go_version = None;

            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(stripped) = trimmed.strip_prefix("module ") {
                    module_name = Some(stripped.trim().to_string());
                } else if let Some(stripped) = trimmed.strip_prefix("go ") {
                    go_version = Some(stripped.trim().to_string());
                }
            }

            let mut details = Vec::new();
            if let Some(gv) = go_version {
                details.push(format!("Go {}", gv));
            }

            return Some(ProjectContext {
                ecosystem: "Go".to_string(),
                name: module_name,
                version: None,
                details,
                root_dir: dir.to_path_buf(),
            });
        }
    }

    // 4. Python: pyproject.toml
    let pyproject = dir.join("pyproject.toml");
    if pyproject.exists() {
        if let Ok(content) = fs::read_to_string(&pyproject) {
            let mut name = None;
            let mut version = None;

            for line in content.lines() {
                let trimmed = line.trim();
                if name.is_none() && trimmed.starts_with("name") && trimmed.contains('=') {
                    name = extract_quoted_value(trimmed);
                }
                if version.is_none() && trimmed.starts_with("version") && trimmed.contains('=') {
                    version = extract_quoted_value(trimmed);
                }
            }

            return Some(ProjectContext {
                ecosystem: "Python (pyproject)".to_string(),
                name,
                version,
                details: vec!["Poetry / Modern Packaging".to_string()],
                root_dir: dir.to_path_buf(),
            });
        }
    }

    None
}

/// Helper to parse a double or single quoted string following an equals sign.
fn extract_quoted_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    let val_part = if parts.len() == 2 { parts[1] } else { parts[0] };
    let trimmed = val_part.trim();

    if let Some(start) = trimmed.find('"') {
        if let Some(end) = trimmed[start + 1..].find('"') {
            return Some(trimmed[start + 1..start + 1 + end].to_string());
        }
    }
    if let Some(start) = trimmed.find('\'') {
        if let Some(end) = trimmed[start + 1..].find('\'') {
            return Some(trimmed[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_extract_quoted_value() {
        assert_eq!(
            extract_quoted_value("name = \"magi-client\""),
            Some("magi-client".to_string())
        );
        assert_eq!(
            extract_quoted_value("edition = '2021'"),
            Some("2021".to_string())
        );
        assert_eq!(extract_quoted_value("invalid line"), None);
    }

    #[test]
    fn test_discover_current_repo_context() {
        // We know we are inside magi-system, which has Cargo.toml
        let ctx = discover_project_context(None).expect("Should discover project context");
        assert_eq!(ctx.ecosystem, "Rust (Cargo)");
        assert!(ctx.details.iter().any(|d| d.contains("Edition 2021")));
        assert!(ctx.summary().contains("Rust (Cargo)"));
        let prompt_header = ctx.to_prompt_header();
        assert!(prompt_header.contains("Discovered Project Context"));
        assert!(prompt_header.contains("**Target Ecosystem**: Rust (Cargo)"));
    }

    #[test]
    fn test_inspect_node_project() {
        let temp_dir = std::env::temp_dir().join("magi_test_node_discovery");
        let _ = fs::create_dir_all(&temp_dir);

        let pkg_json = r#"{
            "name": "my-express-app",
            "version": "2.4.1",
            "dependencies": {
                "express": "^4.18.2"
            }
        }"#;

        let pkg_file = temp_dir.join("package.json");
        let mut file = File::create(&pkg_file).unwrap();
        file.write_all(pkg_json.as_bytes()).unwrap();

        let ctx = inspect_directory(&temp_dir).expect("Discovers Node project");
        assert_eq!(ctx.ecosystem, "Node.js (JavaScript)");
        assert_eq!(ctx.name, Some("my-express-app".to_string()));
        assert_eq!(ctx.version, Some("2.4.1".to_string()));
        assert!(ctx.details.contains(&"Express".to_string()));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
