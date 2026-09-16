//! Canon and delta specs as values.

mod change;
mod parse;
pub mod register;

pub use change::{load_change, ChangeError};
pub use parse::{parse_canon_spec, parse_delta_spec, ParseError};
pub use register::{Entry, Ignores, Register, Scope, TermIgnore};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub name: String,
    pub body: String,
    pub scenarios: Vec<Scenario>,
}

impl Requirement {
    /// A requirement with no body and no scenarios: what a RENAMED pair
    /// yields before it is joined with a MODIFIED entry.
    pub fn is_bare(&self) -> bool {
        self.body.trim().is_empty() && self.scenarios.is_empty()
    }

    pub fn renamed(&self, name: &str) -> Requirement {
        Requirement {
            name: name.to_string(),
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DeltaKind {
    Added,
    Modified,
    Removed,
    Renamed { from: String },
}

impl DeltaKind {
    pub fn glyph(&self) -> char {
        match self {
            DeltaKind::Added => '+',
            DeltaKind::Modified => '~',
            DeltaKind::Removed => '-',
            DeltaKind::Renamed { .. } => '>',
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DeltaKind::Added => "added",
            DeltaKind::Modified => "modified",
            DeltaKind::Removed => "removed",
            DeltaKind::Renamed { .. } => "renamed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaEntry {
    #[serde(flatten)]
    pub kind: DeltaKind,
    pub requirement: Requirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSpec {
    pub capability: String,
    pub entries: Vec<DeltaEntry>,
}

impl DeltaSpec {
    /// A RENAMED entry from A to B and a MODIFIED entry named B become one
    /// entry: kind RENAMED from A, body from the MODIFIED entry. A rename
    /// with no MODIFIED counterpart keeps a bare requirement; pairing fills
    /// it with the canon body of A.
    pub fn join_renames(self) -> DeltaSpec {
        let renamed_to: Vec<String> = self
            .entries
            .iter()
            .filter(|e| matches!(e.kind, DeltaKind::Renamed { .. }))
            .map(|e| e.requirement.name.clone())
            .collect();

        let modified_bodies: BTreeMap<String, Requirement> = self
            .entries
            .iter()
            .filter(|e| e.kind == DeltaKind::Modified && renamed_to.contains(&e.requirement.name))
            .map(|e| (e.requirement.name.clone(), e.requirement.clone()))
            .collect();

        let entries = self
            .entries
            .into_iter()
            .filter(|e| {
                !(e.kind == DeltaKind::Modified && renamed_to.contains(&e.requirement.name))
            })
            .map(|e| match &e.kind {
                DeltaKind::Renamed { .. } => match modified_bodies.get(&e.requirement.name) {
                    Some(body) => DeltaEntry {
                        kind: e.kind.clone(),
                        requirement: body.clone(),
                    },
                    None => e,
                },
                _ => e,
            })
            .collect();

        DeltaSpec {
            capability: self.capability,
            entries,
        }
    }
}

/// Every canonical spec, keyed by capability.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Canon {
    pub specs: BTreeMap<String, Vec<Requirement>>,
}

impl Canon {
    pub fn get(&self, capability: &str, name: &str) -> Option<&Requirement> {
        self.specs.get(capability)?.iter().find(|r| r.name == name)
    }
}

/// One of `proposal.md`, `design.md`, `tasks.md`, before and after.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Artefact {
    pub name: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// A change as read from a snapshot: its artefacts plus its deltas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub name: String,
    pub artefacts: Vec<Artefact>,
    pub deltas: Vec<DeltaSpec>,
}

pub const ARTEFACT_NAMES: [&str; 3] = ["proposal.md", "design.md", "tasks.md"];
