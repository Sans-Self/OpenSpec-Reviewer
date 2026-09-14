//! Blast radius: what an open change does to the citers of the
//! requirements it removes or modifies.

use super::scan::OpenChange;
use super::{Citation, CitationIndex};
use crate::model::DeltaKind;
use crate::review::{Finding, FindingKind, Location};

fn location(change: &str, capability: &str, requirement: &str) -> Location {
    Location {
        change: change.to_string(),
        capability: capability.to_string(),
        requirement: requirement.to_string(),
        scenario: None,
    }
}

/// Findings for one change, keyed by location so the review can attach
/// them to pairings and the lint can list them.
pub fn blast_radius(index: &CitationIndex, change: &OpenChange) -> Vec<Finding> {
    let touched: Vec<&str> = change
        .deltas
        .iter()
        .map(|d| d.capability.as_str())
        .collect();
    let mut findings = Vec::new();
    for delta in &change.deltas {
        for entry in &delta.entries {
            let name = &entry.requirement.name;
            let citation = Citation::new(&delta.capability, name);
            let outside = index.outside_citers(&citation);
            match entry.kind {
                DeltaKind::Removed => {
                    for citer in outside {
                        let handled = citer
                            .citing_capability
                            .as_deref()
                            .is_some_and(|cap| touched.contains(&cap));
                        if handled {
                            continue;
                        }
                        findings.push(Finding::new(
                            FindingKind::RemovedStillCited {
                                file: citer.file.to_string_lossy().into_owned(),
                                citing_capability: citer.citing_capability.clone(),
                            },
                            location(&change.name, &delta.capability, name),
                        ));
                    }
                }
                DeltaKind::Modified => {
                    if !outside.is_empty() {
                        findings.push(Finding::new(
                            FindingKind::ModifiedHasCiters {
                                files: outside
                                    .iter()
                                    .map(|c| c.file.to_string_lossy().into_owned())
                                    .collect(),
                            },
                            location(&change.name, &delta.capability, name),
                        ));
                    }
                }
                DeltaKind::Added | DeltaKind::Renamed { .. } => {}
            }
        }
    }
    findings
}
