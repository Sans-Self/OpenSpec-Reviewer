//! Candidates for the shell's completion requests: the changes on disk, the
//! refs git knows, the pull requests gh lists. Every lookup answers with a
//! list, empty when it cannot answer: a completer runs inside the user's
//! command line, where an error message would land in the middle of what
//! they are typing.

use clap_complete::CompletionCandidate;
use serde::Deserialize;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long `gh pr list` may take before the prompt matters more than the
/// candidates. fish and zsh block until a completer returns.
const GH_DEADLINE: Duration = Duration::from_secs(1);
const GH_POLL: Duration = Duration::from_millis(20);

/// The open changes under `openspec/changes/`, sorted, without the `archive`
/// directory the archived ones move into.
pub fn change_names(root: &Path) -> Vec<CompletionCandidate> {
    let Ok(entries) = std::fs::read_dir(root.join("openspec").join("changes")) else {
        return Vec::new();
    };
    let mut names = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "archive")
        .collect::<Vec<_>>();
    names.sort();
    names.into_iter().map(CompletionCandidate::new).collect()
}

/// The short names of the repository's heads and tags. A commit hash is
/// typed, not completed.
pub fn git_refs(root: &Path) -> Vec<CompletionCandidate> {
    let Ok(out) = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/tags",
        ])
        .current_dir(root)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(CompletionCandidate::new)
        .collect()
}

#[derive(Deserialize)]
struct PullRequest {
    number: u64,
    title: String,
}

/// The open pull requests, number as the value and title as the help text, so
/// `gh <TAB>` reads as a list of pull requests rather than of integers. `gh`
/// has no timeout flag of its own, so the deadline lives here.
pub fn pull_requests(root: &Path) -> Vec<CompletionCandidate> {
    let Ok(mut child) = Command::new("gh")
        .args(["pr", "list", "--json", "number,title"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    let Some(true) = exited_within(&mut child, GH_DEADLINE) else {
        return Vec::new();
    };
    let mut json = String::new();
    let read = child
        .stdout
        .as_mut()
        .map(|out| out.read_to_string(&mut json));
    if !matches!(read, Some(Ok(_))) {
        return Vec::new();
    }
    let Ok(pulls) = serde_json::from_str::<Vec<PullRequest>>(&json) else {
        return Vec::new();
    };
    pulls
        .into_iter()
        .map(|pr| CompletionCandidate::new(pr.number.to_string()).help(Some(pr.title.into())))
        .collect()
}

/// `Some(success)` when the child exited inside the deadline, `None` when it
/// was killed for overrunning it or could not be waited on. A killed child's
/// output is half-written, so the caller reads stdout only on `Some`.
fn exited_within(child: &mut Child, deadline: Duration) -> Option<bool> {
    let expiry = Instant::now() + deadline;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.success()),
            Ok(None) => {}
            Err(_) => break,
        }
        if Instant::now() >= expiry {
            break;
        }
        std::thread::sleep(GH_POLL);
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}
