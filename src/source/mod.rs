//! Where a change comes from. Every source yields the same `Snapshot`;
//! everything downstream reads only the snapshot and canon.

mod archive;
mod canon;
mod change;
mod diff;
mod gh;
mod git;
pub mod lint;
mod repo;
pub mod skills;

pub use archive::{load_archives, load_open_changes, Archive};
pub use canon::{load_canon, CanonError};
pub use change::ChangeSource;
pub use diff::{parse_diff, DiffSource};
pub use gh::{post_review, GhSource};
pub use git::GitSource;
pub use lint::{survey, Workspace, WorkspaceError};
pub use repo::{repo_key, RepoIdentity};

use std::path::PathBuf;
use thiserror::Error;

/// One file under `openspec/`, before and after. Either side is absent for
/// a created or deleted file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: PathBuf,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// The pull request a snapshot came from. Only the `gh` source fills it,
/// and only a snapshot that has one has somewhere to post notes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequest {
    pub number: u64,
    /// The web URL, which is also where the owner and repository come from.
    pub url: String,
    /// The head commit at open. Comments are pinned to it, so a push while
    /// the review is on screen shows them as outdated rather than moving
    /// them onto lines nobody reviewed.
    pub head: String,
}

impl PullRequest {
    /// `owner` and `repo` out of `https://github.com/<owner>/<repo>/pull/n`,
    /// so a pull request given as a URL posts to the repository it names
    /// rather than the one the working directory happens to be.
    pub fn owner_repo(&self) -> Option<(&str, &str)> {
        let rest = self
            .url
            .split_once("://")
            .map(|(_, rest)| rest)
            .unwrap_or(&self.url);
        let mut parts = rest.split('/').skip(1).filter(|p| !p.is_empty());
        let owner = parts.next()?;
        let repo = parts.next()?;
        Some((owner, repo))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Snapshot {
    pub files: Vec<FileChange>,
    /// Shown in the status line: `change foo`, `git feature/x against main`.
    pub origin: String,
    pub pull_request: Option<PullRequest>,
}

impl Snapshot {
    /// Files under `openspec/specs/`: canon edits, listed as plain diffs.
    pub fn canon_files(&self) -> impl Iterator<Item = &FileChange> {
        self.files
            .iter()
            .filter(|f| f.path.starts_with("openspec/specs"))
    }
}

pub trait Source {
    fn fetch(&self) -> Result<Snapshot, SourceError>;
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("no change at {path}; changes found: {}", if found.is_empty() { "(none)".to_string() } else { found.join(", ") })]
    UnknownChange { path: PathBuf, found: Vec<String> },
    #[error("change `{0}` is archived; archived changes are history, not something to review")]
    ArchivedChange(String),
    #[error("the diff touches no file under openspec/")]
    NoOpenSpecContent,
    #[error("cannot parse patch for {file}: {message}")]
    PatchParse { file: String, message: String },
    #[error("hunk {hunk} of {file} does not apply to the working-tree file")]
    HunkMismatch { file: String, hunk: String },
    #[error("{file} is modified by the diff but missing from the working tree")]
    MissingPreimage { file: String },
    #[error("git failed:\n{stderr}")]
    Git { stderr: String },
    #[error("neither `main` nor `master` resolves; pass --base <ref>")]
    NoBase,
    #[error("the `gh` source needs the `gh` CLI on the path")]
    GhMissing,
    #[error("gh failed:\n{stderr}")]
    Gh { stderr: String },
    #[error("cannot tell the owner and repository from {url}")]
    GhPullRequestUrl { url: String },
    #[error("gh answered with something that is not a {what}:\n{body}")]
    GhAnswer { what: String, body: String },
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

impl SourceError {
    pub fn io(context: impl Into<String>) -> impl FnOnce(std::io::Error) -> SourceError {
        let context = context.into();
        move |source| SourceError::Io { context, source }
    }
}
