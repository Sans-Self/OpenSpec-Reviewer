//! A requirement's past, from archived changes, renames chased backwards.

use super::findings::{Finding, FindingKind, Location};
use crate::model::{DeltaEntry, DeltaKind, Requirement};
use crate::source::Archive;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryEntry {
    pub change: String,
    #[serde(flatten)]
    pub kind: DeltaKind,
    pub name: String,
    pub text: Requirement,
}

impl HistoryEntry {
    pub fn label(&self) -> String {
        match &self.kind {
            DeltaKind::Renamed { from } => {
                format!("{} renamed {} → {}", self.change, from, self.name)
            }
            kind => format!("{} {}", self.change, kind.label()),
        }
    }
}

fn entry_for<'a>(entries: &'a [DeltaEntry], name: &str) -> Option<&'a DeltaEntry> {
    entries.iter().find(|e| e.requirement.name == name)
}

/// Oldest first. Walks newest to oldest so a rename to `name` switches the
/// name being followed for everything earlier.
pub fn collect_history(
    archives: &[Archive],
    capability: &str,
    name: &str,
    location: &Location,
) -> (Vec<HistoryEntry>, Vec<Finding>) {
    let mut entries = Vec::new();
    let mut findings = Vec::new();
    let mut current = name.to_string();
    for archive in archives.iter().rev() {
        let deltas = match &archive.deltas {
            Ok(d) => d,
            Err(e) => {
                findings.push(Finding::new(
                    FindingKind::HistoryUnreadable {
                        archive: archive.name.clone(),
                        reason: e.to_string(),
                    },
                    location.clone(),
                ));
                continue;
            }
        };
        let Some(delta) = deltas.iter().find(|d| d.capability == capability) else {
            continue;
        };
        if let Some(entry) = entry_for(&delta.entries, &current) {
            entries.push(HistoryEntry {
                change: archive.name.clone(),
                kind: entry.kind.clone(),
                name: entry.requirement.name.clone(),
                text: entry.requirement.clone(),
            });
            if let DeltaKind::Renamed { from } = &entry.kind {
                current = from.clone();
            }
        }
    }
    entries.reverse();
    (entries, findings)
}
