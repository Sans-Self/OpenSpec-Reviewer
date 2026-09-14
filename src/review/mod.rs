//! Pairing, diffing, findings and history. Pure: values in, values out.

pub mod diff;
pub mod findings;
pub mod history;
pub mod normalize;
pub mod pair;

pub use diff::{
    diff_lines, diff_requirements, DiffLine, LineRole, ParaKind, RequirementDiff, ScenarioMatch,
    Span, SpanMark,
};
pub use findings::{collisions, Finding, FindingKind, Location, Severity, Summary};
pub use history::{collect_history, HistoryEntry};
pub use pair::{inline_view, pair_change, CapabilityReview, ChangeReview, Pairing};

use crate::model::Artefact;
use crate::source::FileChange;
use crate::state::ItemState;
use serde::Serialize;

/// An artefact row: `proposal.md`, `design.md` or `tasks.md`.
#[derive(Debug, Clone, Serialize)]
pub struct ArtefactReview {
    #[serde(flatten)]
    pub artefact: Artefact,
    pub state: ItemState,
}

impl ArtefactReview {
    pub fn key(&self) -> String {
        self.artefact.name.clone()
    }

    pub fn lines(&self) -> Vec<DiffLine> {
        match (&self.artefact.before, &self.artefact.after) {
            (Some(b), Some(a)) => diff_lines(b, a),
            (None, Some(a)) => diff::plain_lines(a),
            (Some(b), None) => diff::marked_lines(b, ParaKind::Removed),
            (None, None) => Vec::new(),
        }
    }
}

/// A canon file the snapshot edits directly, shown as a plain line diff.
#[derive(Debug, Clone, Serialize)]
pub struct CanonEdit {
    pub path: String,
    #[serde(skip)]
    pub lines: Vec<DiffLine>,
}

impl CanonEdit {
    pub fn from_file(file: &FileChange) -> CanonEdit {
        let lines = match (&file.before, &file.after) {
            (Some(b), Some(a)) => diff_lines(b, a),
            (None, Some(a)) => diff::marked_lines(a, ParaKind::Added),
            (Some(b), None) => diff::marked_lines(b, ParaKind::Removed),
            (None, None) => Vec::new(),
        };
        CanonEdit {
            path: file.path.to_string_lossy().into_owned(),
            lines,
        }
    }
}

/// The whole review: what every renderer consumes.
#[derive(Debug, Clone, Serialize)]
pub struct Review {
    pub origin: String,
    pub changes: Vec<ChangeReview>,
    pub canon_edits: Vec<CanonEdit>,
    pub summary: Summary,
}

impl Review {
    pub fn new(origin: String, changes: Vec<ChangeReview>, canon_edits: Vec<CanonEdit>) -> Review {
        let summary = Summary::of(changes.iter().flat_map(ChangeReview::findings));
        Review {
            origin,
            changes,
            canon_edits,
            summary,
        }
    }

    pub fn recount(&mut self) {
        self.summary = Summary::of(self.changes.iter().flat_map(ChangeReview::findings));
    }

    pub fn pairings(&self) -> impl Iterator<Item = &Pairing> {
        self.changes.iter().flat_map(ChangeReview::pairings)
    }

    pub fn pairings_mut(&mut self) -> impl Iterator<Item = &mut Pairing> {
        self.changes.iter_mut().flat_map(ChangeReview::pairings_mut)
    }
}
