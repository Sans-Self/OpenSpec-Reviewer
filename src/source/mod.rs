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

pub use archive::{load_archives, load_open_changes, Archive};
pub use canon::{load_canon, CanonError};
pub use change::ChangeSource;
pub use diff::{parse_diff, DiffSource};
pub use gh::GhSource;
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Snapshot {
    pub files: Vec<FileChange>,
    /// Shown in the status line: `change foo`, `git feature/x against main`.
    pub origin: String,
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
