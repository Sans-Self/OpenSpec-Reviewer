//! Pairing every delta entry with canon, and the inline view of a pairing.

use super::diff::{
    diff_paragraphs, diff_requirements, one_sided_requirement, DiffLine, LineRole, ParaKind,
    RequirementDiff, ScenarioMatch,
};
use super::findings::{collisions, Finding, FindingKind, Location, Summary};
use super::history::HistoryEntry;
use super::normalize::{normalized, text_hash};
use super::ArtefactReview;
use crate::model::{Canon, Change, DeltaKind, DeltaSpec, Requirement};
use crate::state::ItemState;
use serde::Serialize;

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
}

impl Pairing {
    pub fn key(&self) -> String {
        format!("{}/{}", self.capability, self.name)
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

    pub fn worst_severity(&self) -> Option<super::Severity> {
        self.findings.iter().map(|f| f.severity).max()
    }
}

pub fn requirement_text(req: &Requirement) -> String {
    let mut out = normalized(&req.body);
    for s in &req.scenarios {
        out.push_str("\n#### Scenario: ");
        out.push_str(&s.name);
        out.push('\n');
        out.push_str(&normalized(&s.body));
    }
    out
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
