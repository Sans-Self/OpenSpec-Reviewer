//! Pairing every delta entry with canon, and the inline view of a pairing.

use super::diff::{
    diff_paragraphs, diff_requirements, one_sided_requirement, DiffLine, LineRole, ParaKind,
    RequirementDiff, ScenarioMatch,
};
use super::findings::{collisions, Finding, FindingKind, Location, Summary};
use super::history::HistoryEntry;
use super::normalize::{normalized, text_hash};
use super::ArtefactReview;
use crate::model::{Canon, Change, DeltaKind, DeltaSpec, Requirement, Scenario};
use crate::state::{item_key, ItemState, Note, Posted};
use serde::Serialize;
use std::collections::BTreeMap;

/// A note together with the anchor it hangs on and whether that anchor's
/// text has moved since the note was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnchoredNote {
    /// `None` when the note is on the requirement itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    pub text: String,
    pub outdated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// Where this note stands as a comment, once it does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posted: Option<Posted>,
}

impl AnchoredNote {
    fn new(scenario: Option<String>, note: Note, anchor_hash: u64) -> AnchoredNote {
        AnchoredNote {
            scenario,
            outdated: note.is_outdated(anchor_hash),
            text: note.text,
            at: note.at,
            posted: note.posted,
        }
    }

    /// ` # <scenario>`, or nothing for a note on the requirement.
    pub fn anchor_suffix(&self) -> String {
        self.scenario
            .as_ref()
            .map(|s| format!(" # {s}"))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Pairing {
    pub change: String,
    pub capability: String,
    #[serde(flatten)]
    pub kind: DeltaKind,
    pub name: String,
    pub before: Option<Requirement>,
    pub after: Option<Requirement>,
    pub diff: RequirementDiff,
    pub findings: Vec<Finding>,
    pub history: Vec<HistoryEntry>,
    pub state: ItemState,
    /// Every note under this requirement, its own first. Filled in by the
    /// caller that owns the store.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<AnchoredNote>,
}

impl Pairing {
    pub fn key(&self) -> String {
        item_key(&self.capability, &self.name, None)
    }

    pub fn scenario_key(&self, scenario: &str) -> String {
        item_key(&self.capability, &self.name, Some(scenario))
    }

    pub fn location(&self) -> Location {
        Location {
            change: self.change.clone(),
            capability: self.capability.clone(),
            requirement: self.name.clone(),
            scenario: None,
        }
    }

    /// Hash of the normalized after text: what an approval is tied to.
    pub fn text_hash(&self) -> u64 {
        let side = self.after.as_ref().or(self.before.as_ref());
        text_hash(&side.map(requirement_text).unwrap_or_default())
    }

    /// The normalized text of one scenario, after side when it has one,
    /// so a dropped scenario still reads as the canon text it was.
    pub fn scenario_text(&self, scenario: &str) -> Option<String> {
        let find = |r: &Requirement| r.scenarios.iter().find(|s| s.name == scenario).cloned();
        let s = self
            .after
            .as_ref()
            .and_then(find)
            .or_else(|| self.before.as_ref().and_then(find))?;
        Some(scenario_text(&s))
    }

    /// What a note on this anchor is hashed against. An approval hashes the
    /// whole requirement; a note hashes only what it is about, so editing
    /// one scenario leaves the notes on its siblings alone.
    pub fn anchor_hash(&self, scenario: Option<&str>) -> u64 {
        match scenario {
            None => self.text_hash(),
            Some(s) => text_hash(&self.scenario_text(s).unwrap_or_default()),
        }
    }

    /// Read this requirement's notes and its scenarios' out of a store's
    /// items, in row order.
    pub fn refresh_notes(&mut self, items: &BTreeMap<String, ItemState>) {
        let note_at = |key: &str| items.get(key).and_then(|i| i.note.clone());
        let mut notes = Vec::new();
        if let Some(n) = note_at(&self.key()) {
            notes.push(AnchoredNote::new(None, n, self.anchor_hash(None)));
        }
        let names: Vec<String> = self
            .diff
            .scenarios
            .iter()
            .map(|s| s.name().to_string())
            .collect();
        for name in names {
            if let Some(n) = note_at(&self.scenario_key(&name)) {
                let hash = self.anchor_hash(Some(&name));
                notes.push(AnchoredNote::new(Some(name), n, hash));
            }
        }
        self.notes = notes;
    }

    /// The note on the requirement itself.
    pub fn note(&self) -> Option<&AnchoredNote> {
        self.notes.iter().find(|n| n.scenario.is_none())
    }

    pub fn scenario_note(&self, scenario: &str) -> Option<&AnchoredNote> {
        self.notes
            .iter()
            .find(|n| n.scenario.as_deref() == Some(scenario))
    }

    /// The `✎` marker's question: is there a note anywhere under here.
    pub fn has_note(&self) -> bool {
        !self.notes.is_empty()
    }

    pub fn worst_severity(&self) -> Option<super::Severity> {
        self.findings.iter().map(|f| f.severity).max()
    }
}

pub fn requirement_text(req: &Requirement) -> String {
    let mut out = normalized(&req.body);
    for s in &req.scenarios {
        out.push('\n');
        out.push_str(&scenario_text(s));
    }
    out
}

/// One scenario's normalized text, heading included, as it reads inside
/// `requirement_text`.
pub fn scenario_text(scenario: &Scenario) -> String {
    format!(
        "#### Scenario: {}\n{}",
        scenario.name,
        normalized(&scenario.body)
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityReview {
    pub name: String,
    pub pairings: Vec<Pairing>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeReview {
    pub name: String,
    pub artefacts: Vec<ArtefactReview>,
    pub capabilities: Vec<CapabilityReview>,
}

impl ChangeReview {
    pub fn pairings(&self) -> impl Iterator<Item = &Pairing> {
        self.capabilities.iter().flat_map(|c| c.pairings.iter())
    }

    pub fn pairings_mut(&mut self) -> impl Iterator<Item = &mut Pairing> {
        self.capabilities
            .iter_mut()
            .flat_map(|c| c.pairings.iter_mut())
    }

    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        self.pairings().flat_map(|p| p.findings.iter())
    }

    pub fn summary(&self) -> Summary {
        Summary::of(self.findings())
    }
}

fn pair_entry(
    change: &str,
    delta: &DeltaSpec,
    kind: &DeltaKind,
    entry: &Requirement,
    canon: &Canon,
    others: &[(String, Vec<DeltaSpec>)],
) -> Pairing {
    let capability = &delta.capability;
    let name = entry.name.clone();
    let location = Location {
        change: change.to_string(),
        capability: capability.clone(),
        requirement: name.clone(),
        scenario: None,
    };
    let finding = |kind: FindingKind| Finding::new(kind, location.clone());
    let mut findings = Vec::new();

    let (before, after, diff): (Option<Requirement>, Option<Requirement>, RequirementDiff) =
        match kind {
            DeltaKind::Added => match canon.get(capability, &name) {
                Some(existing) => {
                    findings.push(finding(FindingKind::AddedAlreadyExists));
                    (
                        Some(existing.clone()),
                        Some(entry.clone()),
                        diff_requirements(existing, entry),
                    )
                }
                None => (
                    None,
                    Some(entry.clone()),
                    one_sided_requirement(entry, ParaKind::Added),
                ),
            },
            DeltaKind::Modified => match canon.get(capability, &name) {
                Some(existing) => {
                    let diff = diff_requirements(existing, entry);
                    if !diff.changed {
                        findings.push(finding(FindingKind::UnchangedModified));
                    }
                    (Some(existing.clone()), Some(entry.clone()), diff)
                }
                None => {
                    findings.push(finding(FindingKind::ModifiedWithoutCanon));
                    (
                        None,
                        Some(entry.clone()),
                        one_sided_requirement(entry, ParaKind::Added),
                    )
                }
            },
            DeltaKind::Removed => match canon.get(capability, &name) {
                Some(existing) => (
                    Some(existing.clone()),
                    None,
                    one_sided_requirement(existing, ParaKind::Removed),
                ),
                None => {
                    findings.push(finding(FindingKind::RemovedWithoutCanon));
                    (None, None, one_sided_requirement(entry, ParaKind::Removed))
                }
            },
            DeltaKind::Renamed { from } => {
                if canon.get(capability, &name).is_some() {
                    findings.push(finding(FindingKind::RenameTargetTaken));
                }
                match canon.get(capability, from) {
                    Some(existing) => {
                        let after = if entry.is_bare() {
                            existing.renamed(&name)
                        } else {
                            entry.clone()
                        };
                        let diff = diff_requirements(existing, &after);
                        (Some(existing.clone()), Some(after), diff)
                    }
                    None => {
                        findings.push(finding(FindingKind::RenameSourceMissing {
                            from: from.clone(),
                        }));
                        (
                            None,
                            Some(entry.clone()),
                            one_sided_requirement(entry, ParaKind::Added),
                        )
                    }
                }
            }
        };

    if matches!(kind, DeltaKind::Modified | DeltaKind::Renamed { .. }) && before.is_some() {
        findings.extend(
            diff.scenarios
                .iter()
                .filter_map(|s| match s {
                    ScenarioMatch::Removed { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .map(|scenario| finding(FindingKind::ScenarioDropped { scenario })),
        );
    }
    if !matches!(kind, DeltaKind::Removed) && after.as_ref().is_some_and(|a| a.scenarios.is_empty())
    {
        findings.push(finding(FindingKind::RequirementWithoutScenario));
    }
    findings.extend(collisions(capability, &name, &location, others));
    if let DeltaKind::Renamed { from } = kind {
        findings.extend(collisions(capability, from, &location, others));
    }

    Pairing {
        change: change.to_string(),
        capability: capability.clone(),
        kind: kind.clone(),
        name,
        before,
        after,
        diff,
        findings,
        history: Vec::new(),
        state: ItemState::default(),
        notes: Vec::new(),
    }
}

/// Pair every entry of a change with canon. History and state are filled
/// in afterwards by the caller that owns the filesystem.
pub fn pair_change(
    change: &Change,
    canon: &Canon,
    others: &[(String, Vec<DeltaSpec>)],
) -> ChangeReview {
    let artefacts = change
        .artefacts
        .iter()
        .map(|a| ArtefactReview {
            artefact: a.clone(),
            state: ItemState::default(),
        })
        .collect();
    let capabilities = change
        .deltas
        .iter()
        .map(|delta| CapabilityReview {
            name: delta.capability.clone(),
            pairings: delta
                .entries
                .iter()
                .map(|e| pair_entry(&change.name, delta, &e.kind, &e.requirement, canon, others))
                .collect(),
        })
        .collect();
    ChangeReview {
        name: change.name.clone(),
        artefacts,
        capabilities,
    }
}

/// The inline view: name line(s), body, then scenarios with headings.
pub fn inline_view(p: &Pairing) -> Vec<DiffLine> {
    let mut lines = Vec::new();
    match &p.kind {
        DeltaKind::Renamed { from } if p.before.is_some() => {
            lines.push(DiffLine::plain(
                ParaKind::Removed,
                LineRole::Name,
                format!("Requirement: {from}"),
            ));
            lines.push(DiffLine::plain(
                ParaKind::Added,
                LineRole::Name,
                format!("Requirement: {}", p.name),
            ));
        }
        DeltaKind::Removed => lines.push(DiffLine::plain(
            ParaKind::Removed,
            LineRole::Name,
            format!("Requirement: {}", p.name),
        )),
        DeltaKind::Added | DeltaKind::Modified | DeltaKind::Renamed { .. }
            if p.before.is_none() =>
        {
            lines.push(DiffLine::plain(
                ParaKind::Added,
                LineRole::Name,
                format!("Requirement: {}", p.name),
            ))
        }
        _ => lines.push(DiffLine::plain(
            ParaKind::Equal,
            LineRole::Name,
            format!("Requirement: {}", p.name),
        )),
    }
    lines.push(DiffLine::blank());
    lines.extend(p.diff.body.iter().cloned());
    for s in &p.diff.scenarios {
        lines.push(DiffLine::blank());
        lines.push(DiffLine::plain(
            s.heading_kind(),
            LineRole::Scenario,
            format!("Scenario: {}", s.name()),
        ));
        lines.extend(s.lines().iter().cloned());
    }
    lines
}

/// One scenario of the detail pane: its heading, then its diff lines.
pub fn scenario_view(scenario: &ScenarioMatch) -> Vec<DiffLine> {
    let mut lines = vec![DiffLine::plain(
        scenario.heading_kind(),
        LineRole::Scenario,
        format!("Scenario: {}", scenario.name()),
    )];
    lines.push(DiffLine::blank());
    lines.extend(scenario.lines().iter().cloned());
    lines
}

/// Word diff of two requirement versions, for the history view.
pub fn diff_versions(before: &Requirement, after: &Requirement) -> Vec<DiffLine> {
    let mut lines = diff_paragraphs(&before.body, &after.body, LineRole::Body);
    let diff = diff_requirements(before, after);
    for s in &diff.scenarios {
        lines.push(DiffLine::blank());
        lines.push(DiffLine::plain(
            s.heading_kind(),
            LineRole::Scenario,
            format!("Scenario: {}", s.name()),
        ));
        lines.extend(s.lines().iter().cloned());
    }
    lines
}

/// A requirement's text as unmarked lines, for a single history version.
pub fn version_lines(req: &Requirement) -> Vec<DiffLine> {
    let mut lines = vec![DiffLine::plain(
        ParaKind::Equal,
        LineRole::Name,
        format!("Requirement: {}", req.name),
    )];
    lines.push(DiffLine::blank());
    lines.extend(super::diff::one_sided_requirement(req, ParaKind::Equal).body);
    for s in &req.scenarios {
        lines.push(DiffLine::blank());
        lines.push(DiffLine::plain(
            ParaKind::Equal,
            LineRole::Scenario,
            format!("Scenario: {}", s.name),
        ));
        lines.extend(
            super::normalize::paragraphs(&s.body)
                .into_iter()
                .map(|p| DiffLine::plain(ParaKind::Equal, LineRole::Body, p)),
        );
    }
    lines
}
