//! Approvals and notes, one JSON file per repository and change.

mod editor;
mod time;

pub use editor::{edit_note, edit_note_with};

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

/// Where a note stands as a comment: when it was posted, and the review
/// it was posted in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Posted {
    /// RFC 3339.
    pub at: String,
    pub url: String,
}

impl Posted {
    /// The date alone, which is what a pane under a note has room for.
    pub fn date(&self) -> &str {
        self.at.get(..10).unwrap_or(&self.at)
    }
}

/// One free-text note, on a requirement or on one of its scenarios,
/// together with the text it was written about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "NoteRepr")]
pub struct Note {
    pub text: String,
    /// Absent in a note an older state file held as a bare string, which
    /// therefore never reads as outdated until it is next saved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_hash: Option<u64>,
    /// RFC 3339.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// Absent until the note reaches the pull request, and absent again
    /// after an edit: `Note::new` builds a fresh note, so saving a new
    /// text makes it unposted without a rule of its own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posted: Option<Posted>,
}

/// What a state file may hold for a note: today's record, or the bare
/// string every file written before notes carried a hash holds.
#[derive(Deserialize)]
#[serde(untagged)]
enum NoteRepr {
    Full {
        text: String,
        #[serde(default)]
        text_hash: Option<u64>,
        #[serde(default)]
        at: Option<String>,
        #[serde(default)]
        posted: Option<Posted>,
    },
    Bare(String),
}

impl From<NoteRepr> for Note {
    fn from(repr: NoteRepr) -> Note {
        match repr {
            NoteRepr::Full {
                text,
                text_hash,
                at,
                posted,
            } => Note {
                text,
                text_hash,
                at,
                posted,
            },
            NoteRepr::Bare(text) => Note {
                text,
                text_hash: None,
                at: None,
                posted: None,
            },
        }
    }
}

impl Note {
    pub fn new(text: String, anchor_hash: u64) -> Note {
        Note {
            text,
            text_hash: Some(anchor_hash),
            at: Some(time::now_rfc3339()),
            posted: None,
        }
    }

    /// The anchor's text has moved since the note was written.
    pub fn is_outdated(&self, anchor_hash: u64) -> bool {
        self.text_hash.is_some_and(|h| h != anchor_hash)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved: Option<Approval>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Note>,
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
        self.approved.is_none() && self.note.is_none()
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

/// The key an item is stored under: `<capability>/<requirement>` for a
/// requirement, `<capability>/<requirement>#<scenario>` for one of its
/// scenarios. `#` is the separator plain output already prints.
pub fn item_key(capability: &str, requirement: &str, scenario: Option<&str>) -> String {
    match scenario {
        None => format!("{capability}/{requirement}"),
        Some(s) => format!("{capability}/{requirement}#{s}"),
    }
}

/// The anchor a key names: everything before the `#`, then the scenario.
pub fn split_key(key: &str) -> (&str, Option<&str>) {
    match key.split_once('#') {
        Some((requirement, scenario)) => (requirement, Some(scenario)),
        None => (key, None),
    }
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

    /// Stamp every named note as posted in the review at `url`. A key with
    /// no note is skipped: the reviewer may have deleted it while gh was
    /// running, and a posted record on nothing helps nobody.
    pub fn mark_posted(&mut self, keys: &[String], url: &str) -> Result<(), StateError> {
        let at = time::now_rfc3339();
        for key in keys {
            if let Some(note) = self
                .state
                .items
                .get_mut(key)
                .and_then(|item| item.note.as_mut())
            {
                note.posted = Some(Posted {
                    at: at.clone(),
                    url: url.to_string(),
                });
            }
        }
        self.save()
    }

    /// Store a note against `anchor_hash`, the hash of the text it is
    /// written about; an empty note removes it.
    pub fn set_note(
        &mut self,
        key: &str,
        note: Option<String>,
        anchor_hash: u64,
    ) -> Result<ItemState, StateError> {
        self.update(key, |item| {
            item.note = note
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty())
                .map(|n| Note::new(n, anchor_hash));
        })
    }
}
