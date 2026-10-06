//! # Git Diff Integration Helper
//!
//! Provides utilities for inspecting uncommitted, staged, or branch-level git changes.

use crate::error::MagiError;
use std::process::Command;

/// Retrieves the git diff from the current repository workspace.
///
/// # Arguments
/// * `staged` - If true, retrieves staged changes (`git diff --staged`).
/// * `branch` - Optional git reference or branch to compare against (e.g. `origin/main`).
pub fn get_git_diff(staged: bool, branch: Option<&str>) -> Result<String, MagiError> {
    let mut cmd = Command::new("git");
    cmd.arg("diff");

    if staged {
        cmd.arg("--staged");
    } else if let Some(target) = branch {
        cmd.arg(target);
    }

    let output = cmd
        .output()
        .map_err(|e| MagiError::Internal(format!("Failed to execute 'git diff': {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MagiError::Internal(format!(
            "git diff command failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    let diff_text = String::from_utf8(output.stdout)
        .map_err(|e| MagiError::Internal(format!("git diff output is not valid UTF-8: {}", e)))?;

    Ok(diff_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_diff_execution() {
        // Inside this repository, git diff should execute without errors
        let result = get_git_diff(false, None);
        assert!(result.is_ok(), "Expected git diff to succeed in repo");
    }

    #[test]
    fn test_git_diff_staged_execution() {
        let result = get_git_diff(true, None);
        assert!(
            result.is_ok(),
            "Expected git diff --staged to succeed in repo"
        );
    }
}
