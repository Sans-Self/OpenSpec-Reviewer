//! Approvals and notes, one JSON file per repository and change.

mod editor;
mod time;

pub use editor::{edit_note, edit_note_with};

use crate::assist::{Hint, HintKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub text_hash: u64,
    /// RFC 3339.
    pub at: String,
}

/// An agent's hints and the text they were made for. The hash is the same
/// normalized-after-text hash an approval carries, so one hash function
/// serves both.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hints {
    pub text_hash: u64,
    #[serde(default)]
    pub items: Vec<Hint>,
}

/// A hint the reviewer said no to, keyed by what it says rather than
/// where it sat, so the same hint coming back stays hidden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dismissal {
    pub kind: HintKind,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved: Option<Approval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hints: Option<Hints>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dismissed: Vec<Dismissal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    /// Approved, but the text changed since.
    Stale,
}

impl ApprovalStatus {
    pub fn mark(self) -> &'static str {
        match self {
            ApprovalStatus::Pending => "[ ]",
            ApprovalStatus::Approved => "[√]",
            ApprovalStatus::Stale => "[~]",
        }
    }
}

impl ItemState {
    pub fn status(&self, current_hash: u64) -> ApprovalStatus {
        match &self.approved {
            None => ApprovalStatus::Pending,
            Some(a) if a.text_hash == current_hash => ApprovalStatus::Approved,
            Some(_) => ApprovalStatus::Stale,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.approved.is_none()
            && self.note.is_none()
            && self.hints.is_none()
            && self.dismissed.is_empty()
    }

    /// The cached hints for `current_hash`, each marked dismissed when a
    /// dismissal matches it. A hash that does not match has no hints: the
    /// text moved on, the reading did not.
    pub fn hints_for(&self, current_hash: u64) -> Vec<Hint> {
        let Some(hints) = &self.hints else {
            return Vec::new();
        };
        if hints.text_hash != current_hash {
            return Vec::new();
        }
        hints
            .items
            .iter()
            .map(|h| Hint {
                dismissed: h.dismissed || self.is_dismissed(h),
                ..h.clone()
            })
            .collect()
    }

    fn is_dismissed(&self, hint: &Hint) -> bool {
        self.dismissed
            .iter()
            .any(|d| d.kind == hint.kind && d.message == hint.message)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeState {
    #[serde(default)]
    pub items: BTreeMap<String, ItemState>,
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("cannot read state file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("state file {path} is not valid JSON: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("cannot write state file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// `$XDG_STATE_HOME`, defaulting to `~/.local/state`.
pub fn state_home() -> Option<PathBuf> {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| directories::BaseDirs::new().map(|d| d.home_dir().join(".local").join("state")))
}

/// `<home>/openspec-reviewer/<repo>/<change>.json`.
pub fn state_path(home: &Path, repo_key: &str, change: &str) -> PathBuf {
    home.join("openspec-reviewer")
        .join(repo_key)
        .join(format!("{change}.json"))
}

/// The store writes after every change; there is no save step to forget.
/// With no path it is `--no-state`: reads and writes go nowhere.
#[derive(Debug, Clone, Default)]
pub struct Store {
    pub path: Option<PathBuf>,
    pub state: ChangeState,
}

impl Store {
    pub fn disabled() -> Store {
        Store::default()
    }

    pub fn open(path: PathBuf) -> Result<Store, StateError> {
        let state = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).map_err(|source| StateError::Parse {
                path: path.clone(),
                source,
            })?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => ChangeState::default(),
            Err(source) => return Err(StateError::Read { path, source }),
        };
        Ok(Store {
            path: Some(path),
            state,
        })
    }

    pub fn get(&self, key: &str) -> ItemState {
        self.state.items.get(key).cloned().unwrap_or_default()
    }

    fn save(&self) -> Result<(), StateError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let write = |p: &Path| -> std::io::Result<()> {
            if let Some(dir) = p.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let text = serde_json::to_string_pretty(&self.state).expect("state serializes");
            std::fs::write(p, text)
        };
        write(path).map_err(|source| StateError::Write {
            path: path.clone(),
            source,
        })
    }

    fn update(
        &mut self,
        key: &str,
        f: impl FnOnce(&mut ItemState),
    ) -> Result<ItemState, StateError> {
        let mut item = self.get(key);
        f(&mut item);
        if item.is_empty() {
            self.state.items.remove(key);
        } else {
            self.state.items.insert(key.to_string(), item.clone());
        }
        self.save()?;
        Ok(item)
    }

    /// Approve against `hash`, or unapprove when already approved against
    /// it. A stale approval re-approves against the new text.
    pub fn toggle_approval(&mut self, key: &str, hash: u64) -> Result<ItemState, StateError> {
        self.update(key, |item| {
            item.approved = match &item.approved {
                Some(a) if a.text_hash == hash => None,
                _ => Some(Approval {
                    text_hash: hash,
                    at: time::now_rfc3339(),
                }),
            };
        })
    }

    pub fn set_note(&mut self, key: &str, note: Option<String>) -> Result<ItemState, StateError> {
        self.update(key, |item| {
            item.note = note.filter(|n| !n.trim().is_empty());
        })
    }

    /// Replace the cached hints for `key`. A re-run overwrites; a hint the
    /// reviewer already dismissed comes back dismissed.
    pub fn set_hints(
        &mut self,
        key: &str,
        text_hash: u64,
        items: Vec<Hint>,
    ) -> Result<ItemState, StateError> {
        self.update(key, |item| {
            let items = items
                .into_iter()
                .map(|h| Hint {
                    dismissed: item.is_dismissed(&h),
                    ..h
                })
                .collect();
            item.hints = Some(Hints { text_hash, items });
        })
    }

    pub fn dismiss_hint(
        &mut self,
        key: &str,
        kind: HintKind,
        message: &str,
    ) -> Result<ItemState, StateError> {
        self.update(key, |item| {
            if !item
                .dismissed
                .iter()
                .any(|d| d.kind == kind && d.message == message)
            {
                item.dismissed.push(Dismissal {
                    kind,
                    message: message.to_string(),
                });
            }
            if let Some(hints) = &mut item.hints {
                for hint in &mut hints.items {
                    if hint.kind == kind && hint.message == message {
                        hint.dismissed = true;
                    }
                }
            }
        })
    }
}
