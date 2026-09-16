//! Agent CLIs behind one interface, and the hints their replies become.
//!
//! An adapter owns the flags of one CLI and nothing else: it opens an
//! interactive session on a prompt file, or runs one non-interactively and
//! hands back the raw reply. Parsing lives here so every agent's answer
//! goes through the same parser.

pub mod files;
pub mod parse;
pub mod prompt;
pub mod session;

mod adapters;

pub use adapters::{assistant, Claude, Codex, Custom, Opencode};
pub use files::{export_prompts, load_template, spec_rules, Export};
pub use parse::parse_hints;
pub use prompt::{change_prompt, glossary_terms, pairing_prompt, PromptContext, SiblingText};
pub use session::Session;

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The two ways the reviewer uses an agent. `handoff` inherits the
/// terminal; `review` captures stdout.
pub trait Assistant {
    fn handoff(&self, prompt: &Path) -> Result<(), AssistError>;
    fn review(&self, prompt: &Path) -> Result<String, AssistError>;
}

#[derive(Debug, Error)]
pub enum AssistError {
    #[error("assist is not configured; add an [assist] section to openspec/reviewer.toml")]
    NotConfigured,
    #[error("the `{binary}` CLI was not found")]
    NotFound { binary: String },
    #[error("assist.agent = \"custom\" needs assist.{field}")]
    NoCommand { field: &'static str },
    #[error("cannot read the prompt file {path}: {source}")]
    Prompt {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot run {binary}: {source}")]
    Spawn {
        binary: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{binary} exited with {code}: {stderr}")]
    Failed {
        binary: String,
        code: i32,
        stderr: String,
    },
}

/// What a hint is about. Closed: the six judgment tasks the default
/// prompt asks for, plus the one the parser makes up when a reply does
/// not fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintKind {
    CompoundCondition,
    UncoveredMust,
    PlainLanguage,
    TermMisuse,
    SiblingInvalidated,
    RejectedApproach,
    Unparsed,
}

impl HintKind {
    pub const ALL: [HintKind; 7] = [
        HintKind::CompoundCondition,
        HintKind::UncoveredMust,
        HintKind::PlainLanguage,
        HintKind::TermMisuse,
        HintKind::SiblingInvalidated,
        HintKind::RejectedApproach,
        HintKind::Unparsed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            HintKind::CompoundCondition => "compound_condition",
            HintKind::UncoveredMust => "uncovered_must",
            HintKind::PlainLanguage => "plain_language",
            HintKind::TermMisuse => "term_misuse",
            HintKind::SiblingInvalidated => "sibling_invalidated",
            HintKind::RejectedApproach => "rejected_approach",
            HintKind::Unparsed => "unparsed",
        }
    }

    pub fn parse(text: &str) -> Option<HintKind> {
        HintKind::ALL.into_iter().find(|k| k.as_str() == text)
    }
}

impl std::fmt::Display for HintKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One thing the agent noticed. Never an approval, never an exit status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hint {
    pub kind: HintKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    #[serde(default)]
    pub dismissed: bool,
}

impl Hint {
    pub fn new(kind: HintKind, message: impl Into<String>) -> Hint {
        Hint {
            kind,
            message: message.into(),
            quote: None,
            scenario: None,
            dismissed: false,
        }
    }

    /// What a dismissal is keyed by: kind and message, not position.
    pub fn key(&self) -> (HintKind, &str) {
        (self.kind, self.message.as_str())
    }
}

/// `[assist]` in `openspec/reviewer.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assist {
    pub agent: Agent,
    pub handoff_command: Option<String>,
    pub review_command: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
    Opencode,
    Custom,
}

impl Agent {
    /// The executable the adapter looks for. `custom` runs its templates
    /// through a shell.
    pub fn binary(self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Opencode => "opencode",
            Agent::Custom => "sh",
        }
    }
}

/// The first entry of `path` holding an executable named `binary`.
/// Taking the search path as a value keeps the lookup testable without
/// touching the process environment.
pub fn which(binary: &str, path: Option<&OsString>) -> Option<PathBuf> {
    std::env::split_paths(path?)
        .map(|dir| dir.join(binary))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
