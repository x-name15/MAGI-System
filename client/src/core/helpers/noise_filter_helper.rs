//! # Smart Diff & Noise Filtering Helper
//!
//! Excludes build artifacts, lockfiles, minified bundles, and generated assets
//! from git diff payloads to optimize LLM token budget and preserve analytical focus.

/// Represents the result of filtering a git diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredDiff {
    /// The filtered git diff text excluding noise files.
    pub content: String,
    /// List of file paths excluded from the diff.
    pub ignored_files: Vec<String>,
    /// Number of lines in the raw diff before filtering.
    pub original_lines: usize,
    /// Number of lines in the clean diff after filtering.
    pub filtered_lines: usize,
}

impl FilteredDiff {
    /// Returns true if the filtered diff is effectively empty.
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
}

/// Known lockfile names across mainstream package managers.
const NOISE_FILENAMES: &[&str] = &[
    "cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "composer.lock",
    "poetry.lock",
    "pipfile.lock",
    "flake.lock",
    "gemfile.lock",
    "mix.lock",
    "go.sum",
];

/// Known noise extensions (minified files, source maps, binaries).
const NOISE_EXTENSIONS: &[&str] = &[
    ".min.js", ".min.css", ".map", ".wasm", ".exe", ".dll", ".dylib", ".so", ".bin", ".png",
    ".jpg", ".jpeg", ".gif", ".ico", ".pdf", ".zip", ".tar.gz", ".tgz",
];

/// Known generated or dependency directory prefixes.
const NOISE_DIR_PREFIXES: &[&str] = &[
    "target/",
    "dist/",
    "build/",
    "node_modules/",
    "vendor/",
    ".spacetimedb/",
    ".next/",
    "out/",
    ".git/",
];

/// Determines whether a given file path represents a noise or auto-generated artifact.
pub fn is_noise_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    let norm = lower.replace('\\', "/");
    let file_name = norm.rsplit('/').next().unwrap_or(&norm);

    // 1. Check exact lockfile matches
    if NOISE_FILENAMES.contains(&file_name) {
        return true;
    }

    // 2. Check noise extensions
    if NOISE_EXTENSIONS.iter().any(|&ext| norm.ends_with(ext)) {
        return true;
    }

    // 3. Check noise directory prefixes
    for &prefix in NOISE_DIR_PREFIXES {
        if norm.starts_with(prefix) || norm.contains(&format!("/{}", prefix)) {
            return true;
        }
    }

    false
}

/// Parses a multi-file git diff string and filters out noise files.
pub fn filter_git_diff(raw_diff: &str) -> FilteredDiff {
    let original_lines = raw_diff.lines().count();
    if raw_diff.trim().is_empty() {
        return FilteredDiff {
            content: String::new(),
            ignored_files: Vec::new(),
            original_lines: 0,
            filtered_lines: 0,
        };
    }

    let mut accepted_sections: Vec<String> = Vec::new();
    let mut ignored_files: Vec<String> = Vec::new();

    // Split diff by `diff --git ` boundaries
    let diff_marker = "diff --git ";
    let mut chunks: Vec<&str> = Vec::new();
    let mut last_idx = 0;

    for (idx, _) in raw_diff.match_indices(diff_marker) {
        if idx > last_idx {
            chunks.push(&raw_diff[last_idx..idx]);
        }
        last_idx = idx;
    }
    if last_idx < raw_diff.len() {
        chunks.push(&raw_diff[last_idx..]);
    }

    for chunk in chunks {
        let chunk_trimmed = chunk.trim_start();
        if !chunk_trimmed.starts_with(diff_marker) {
            // Header or unclassified prefix before first diff marker
            if !chunk.trim().is_empty() {
                accepted_sections.push(chunk.to_string());
            }
            continue;
        }

        // Extract target path: "diff --git a/foo/bar.rs b/foo/bar.rs"
        let first_line = chunk_trimmed.lines().next().unwrap_or("");
        let file_path = extract_path_from_diff_header(first_line);

        if let Some(path) = file_path {
            if is_noise_file(&path) {
                ignored_files.push(path);
                continue;
            }
        }

        accepted_sections.push(chunk.to_string());
    }

    let mut clean_content = accepted_sections.join("");
    if !ignored_files.is_empty() && !clean_content.trim().is_empty() {
        let banner = format!(
            "// [MAGI SMART INGESTION: Filtered {} lock/generated noise file(s): {}]\n\n",
            ignored_files.len(),
            ignored_files.join(", ")
        );
        clean_content = format!("{}{}", banner, clean_content);
    }

    let filtered_lines = clean_content.lines().count();

    FilteredDiff {
        content: clean_content,
        ignored_files,
        original_lines,
        filtered_lines,
    }
}

/// Extracts the target file path from a git diff header line.
/// e.g. "diff --git a/Cargo.lock b/Cargo.lock" -> "Cargo.lock"
fn extract_path_from_diff_header(header_line: &str) -> Option<String> {
    let parts: Vec<&str> = header_line.split_whitespace().collect();
    if parts.len() >= 4 && parts[0] == "diff" && parts[1] == "--git" {
        let b_part = parts[3];
        let path = b_part.strip_prefix("b/").unwrap_or(b_part);
        return Some(path.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_noise_file_detection() {
        assert!(is_noise_file("Cargo.lock"));
        assert!(is_noise_file("client/Cargo.lock"));
        assert!(is_noise_file("package-lock.json"));
        assert!(is_noise_file("frontend/pnpm-lock.yaml"));
        assert!(is_noise_file("dist/bundle.js"));
        assert!(is_noise_file("target/debug/magi"));
        assert!(is_noise_file("assets/app.min.js"));
        assert!(is_noise_file("styles/theme.min.css"));
        assert!(is_noise_file("bundle.js.map"));
        assert!(is_noise_file("public/logo.png"));

        // Legitimate source code should NOT be detected as noise
        assert!(!is_noise_file("src/main.rs"));
        assert!(!is_noise_file("client/Cargo.toml"));
        assert!(!is_noise_file("package.json"));
        assert!(!is_noise_file("README.md"));
        assert!(!is_noise_file("docs/guides/CLI.md"));
    }

    #[test]
    fn test_filter_git_diff_removes_lockfiles() {
        let raw = r#"diff --git a/Cargo.lock b/Cargo.lock
index 1111..2222 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1,5 +1,6 @@
+new-dependency v1.0.0
diff --git a/src/main.rs b/src/main.rs
index 3333..4444 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,3 +10,4 @@
+println!("hello world");
"#;

        let filtered = filter_git_diff(raw);
        assert_eq!(filtered.ignored_files, vec!["Cargo.lock"]);
        assert!(filtered.content.contains("src/main.rs"));
        assert!(!filtered.content.contains("new-dependency v1.0.0"));
        assert!(filtered
            .content
            .contains("Filtered 1 lock/generated noise file(s)"));
    }

    #[test]
    fn test_filter_git_diff_preserves_clean_diff() {
        let raw = r#"diff --git a/src/auth.rs b/src/auth.rs
index 1234..5678 100644
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -1 +1 @@
-fn old() {}
+fn new() {}
"#;

        let filtered = filter_git_diff(raw);
        assert!(filtered.ignored_files.is_empty());
        assert_eq!(filtered.content, raw);
    }
}
