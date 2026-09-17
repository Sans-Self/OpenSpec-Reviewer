//! `gh <pr>`: the pull request resolved, then `gh pr diff` handed to the
//! diff source. Posting a review goes back out through the same CLI, so
//! authentication stays gh's problem.

use super::{parse_diff, PullRequest, Snapshot, Source, SourceError};
use crate::review::post::ReviewPost;
use serde::Deserialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct GhSource {
    pub root: PathBuf,
    pub pr: String,
}

/// `gh pr view --json number,url,headRefOid`.
#[derive(Deserialize)]
struct PrView {
    number: u64,
    url: String,
    #[serde(rename = "headRefOid")]
    head_ref_oid: String,
}

fn run(root: &Path, args: &[&str]) -> Result<Vec<u8>, SourceError> {
    let out = Command::new("gh")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => SourceError::GhMissing,
            _ => SourceError::Io {
                context: "running gh".to_string(),
                source: e,
            },
        })?;
    if !out.status.success() {
        return Err(SourceError::Gh {
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    Ok(out.stdout)
}

impl Source for GhSource {
    fn fetch(&self) -> Result<Snapshot, SourceError> {
        let view = run(
            &self.root,
            &["pr", "view", &self.pr, "--json", "number,url,headRefOid"],
        )?;
        let view: PrView = serde_json::from_slice(&view).map_err(|_| SourceError::GhAnswer {
            what: "pull request".to_string(),
            body: String::from_utf8_lossy(&view).trim().to_string(),
        })?;
        let diff = run(&self.root, &["pr", "diff", &self.pr])?;
        let text = String::from_utf8_lossy(&diff);
        let mut snapshot = parse_diff(&self.root, &text, format!("gh pr {}", self.pr))?;
        snapshot.pull_request = Some(PullRequest {
            number: view.number,
            url: view.url,
            head: view.head_ref_oid,
        });
        Ok(snapshot)
    }
}

/// Submit the planned review and answer with its URL. GitHub rejects a
/// comment on a line its diff does not show, and the planner cannot see
/// the hunks, so a `422` is answered once with everything in the body
/// rather than with a guess at which comment was the offending one.
pub fn post_review(
    root: &Path,
    pr: &PullRequest,
    post: &ReviewPost,
) -> Result<String, SourceError> {
    let (owner, repo) = pr
        .owner_repo()
        .ok_or_else(|| SourceError::GhPullRequestUrl {
            url: pr.url.clone(),
        })?;
    let endpoint = format!("repos/{owner}/{repo}/pulls/{}/reviews", pr.number);
    match send(root, &endpoint, &post.payload(&pr.head)) {
        Err(SourceError::Gh { stderr }) if stderr.contains("422") => send(
            root,
            &endpoint,
            &post.clone().into_body_only().payload(&pr.head),
        ),
        other => other,
    }
}

/// One `gh api ... --input -`, answering with the review's `html_url`.
fn send(root: &Path, endpoint: &str, payload: &serde_json::Value) -> Result<String, SourceError> {
    let mut child = Command::new("gh")
        .args(["api", endpoint, "--method", "POST", "--input", "-"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => SourceError::GhMissing,
            _ => SourceError::Io {
                context: "running gh".to_string(),
                source: e,
            },
        })?;
    let body = payload.to_string();
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(body.as_bytes())
        .map_err(SourceError::io("writing the review to gh"))?;
    let out = child
        .wait_with_output()
        .map_err(SourceError::io("waiting for gh"))?;
    if !out.status.success() {
        return Err(SourceError::Gh {
            stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        });
    }
    let answer: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|_| SourceError::GhAnswer {
            what: "review".to_string(),
            body: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        })?;
    answer
        .get("html_url")
        .and_then(|u| u.as_str())
        .map(str::to_string)
        .ok_or_else(|| SourceError::GhAnswer {
            what: "review".to_string(),
            body: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        })
}
