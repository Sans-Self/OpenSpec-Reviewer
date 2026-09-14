//! The closed set of findings and the checks that produce them.

use crate::model::{DeltaKind, DeltaSpec};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Note,
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FindingKind {
    ModifiedWithoutCanon,
    AddedAlreadyExists,
    RemovedWithoutCanon,
    RenameSourceMissing { from: String },
    RenameTargetTaken,
    ScenarioDropped { scenario: String },
    RequirementWithoutScenario,
    CrossChangeCollision { change: String },
    UnchangedModified,
    HistoryUnreadable { archive: String, reason: String },
}

impl FindingKind {
    /// Findings of one kind always have the same severity.
    pub fn severity(&self) -> Severity {
        match self {
            FindingKind::ModifiedWithoutCanon
            | FindingKind::AddedAlreadyExists
            | FindingKind::RemovedWithoutCanon
            | FindingKind::RenameSourceMissing { .. }
            | FindingKind::RenameTargetTaken => Severity::Error,
            FindingKind::ScenarioDropped { .. }
            | FindingKind::RequirementWithoutScenario
            | FindingKind::CrossChangeCollision { .. } => Severity::Warning,
            FindingKind::UnchangedModified | FindingKind::HistoryUnreadable { .. } => {
                Severity::Note
            }
        }
    }

    pub fn message(&self, requirement: &str) -> String {
        match self {
            FindingKind::ModifiedWithoutCanon => {
                format!("modified without canon: no requirement `{requirement}` in canon")
            }
            FindingKind::AddedAlreadyExists => {
                format!("added already exists: canon already has `{requirement}`")
            }
            FindingKind::RemovedWithoutCanon => {
                format!("removed without canon: no requirement `{requirement}` in canon")
            }
            FindingKind::RenameSourceMissing { from } => {
                format!("rename source missing: no requirement `{from}` in canon")
            }
            FindingKind::RenameTargetTaken => {
                format!("rename target taken: canon already has `{requirement}`")
            }
            FindingKind::ScenarioDropped { scenario } => {
                format!("scenario dropped: `{scenario}`")
            }
            FindingKind::RequirementWithoutScenario => "no scenarios".to_string(),
            FindingKind::CrossChangeCollision { change } => format!("also touched by {change}"),
            FindingKind::UnchangedModified => "no change".to_string(),
            FindingKind::HistoryUnreadable { archive, reason } => {
                format!("history skips {archive}: {reason}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub change: String,
    pub capability: String,
    pub requirement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.change, self.capability, self.requirement
        )?;
        if let Some(s) = &self.scenario {
            write!(f, "#{s}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    #[serde(flatten)]
    pub kind: FindingKind,
    pub severity: Severity,
    pub location: Location,
    pub message: String,
}

impl Finding {
    pub fn new(kind: FindingKind, location: Location) -> Finding {
        let severity = kind.severity();
        let message = kind.message(&location.requirement);
        let location = match &kind {
            FindingKind::ScenarioDropped { scenario } => Location {
                scenario: Some(scenario.clone()),
                ..location
            },
            _ => location,
        };
        Finding {
            kind,
            severity,
            location,
            message,
        }
    }
}

/// Counts per severity; the exit status comes from the worst one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub notes: usize,
}

impl Summary {
    pub fn of<'a>(findings: impl IntoIterator<Item = &'a Finding>) -> Summary {
        findings.into_iter().fold(Summary::default(), |mut s, f| {
            match f.severity {
                Severity::Error => s.errors += 1,
                Severity::Warning => s.warnings += 1,
                Severity::Note => s.notes += 1,
            }
            s
        })
    }

    pub fn exit_code(&self) -> i32 {
        if self.errors > 0 {
            2
        } else if self.warnings > 0 {
            1
        } else {
            0
        }
    }

    pub fn total(&self) -> usize {
        self.errors + self.warnings + self.notes
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} error{}, {} warning{}, {} note{}",
            self.errors,
            if self.errors == 1 { "" } else { "s" },
            self.warnings,
            if self.warnings == 1 { "" } else { "s" },
            self.notes,
            if self.notes == 1 { "" } else { "s" },
        )
    }
}

fn touches(entry_kind: &DeltaKind, entry_name: &str, name: &str) -> bool {
    entry_name == name || matches!(entry_kind, DeltaKind::Renamed { from } if from == name)
}

/// Other open changes with an entry for the same capability and name.
pub fn collisions(
    capability: &str,
    name: &str,
    location: &Location,
    others: &[(String, Vec<DeltaSpec>)],
) -> Vec<Finding> {
    others
        .iter()
        .filter(|(_, deltas)| {
            deltas.iter().any(|d| {
                d.capability == capability
                    && d.entries
                        .iter()
                        .any(|e| touches(&e.kind, &e.requirement.name, name))
            })
        })
        .map(|(change, _)| {
            Finding::new(
                FindingKind::CrossChangeCollision {
                    change: change.clone(),
                },
                location.clone(),
            )
        })
        .collect()
}
