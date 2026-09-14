//! Identity of the repository the tool runs in, for the state path.

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoIdentity {
    Origin(String),
    TopLevel(String),
}

fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// The origin remote URL, else the absolute git top level, else the
/// working directory itself.
pub fn repo_identity(root: &Path) -> RepoIdentity {
    if let Some(url) = git_stdout(root, &["remote", "get-url", "origin"]) {
        return RepoIdentity::Origin(url);
    }
    let top = git_stdout(root, &["rev-parse", "--show-toplevel"])
        .unwrap_or_else(|| root.to_string_lossy().into_owned());
    RepoIdentity::TopLevel(top)
}

/// Every non-path-safe character replaced by `_`.
pub fn path_safe(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn repo_key(root: &Path) -> String {
    match repo_identity(root) {
        RepoIdentity::Origin(url) | RepoIdentity::TopLevel(url) => path_safe(&url),
    }
}
