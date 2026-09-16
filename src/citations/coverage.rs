//! The ledger: per capability, how many source files cite each requirement
//! of the register. Spec-to-spec citations are cross-links, not evidence,
//! so they do not count.

use super::{Citation, CitationIndex};
use crate::model::{Entry as Requirement, Register};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub requirement: String,
    pub citing_files: usize,
    /// Named by `[[lint.ignore_uncited]]`: counted, never marked.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub ignored: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Coverage {
    pub capabilities: BTreeMap<String, Vec<Entry>>,
    pub cited: usize,
    pub total: usize,
    pub ignored: usize,
}

/// An ignored requirement stays in `total`: a coverage figure that moves
/// when somebody edits a configuration is a figure that lies.
pub fn coverage(
    index: &CitationIndex,
    register: &Register,
    ignored: &BTreeSet<Requirement>,
) -> Coverage {
    let mut out = Coverage::default();
    for (capability, names) in register.by_capability() {
        let entries: Vec<Entry> = names
            .into_iter()
            .map(|requirement| {
                let citing_files = index
                    .citers_of(&Citation::new(capability, requirement))
                    .iter()
                    .filter(|c| c.citing_capability.is_none())
                    .count();
                let ignored = citing_files == 0
                    && ignored.contains(&Requirement::new(capability, requirement));
                out.total += 1;
                if citing_files > 0 {
                    out.cited += 1;
                } else if ignored {
                    out.ignored += 1;
                }
                Entry {
                    requirement: requirement.to_string(),
                    citing_files,
                    ignored,
                }
            })
            .collect();
        out.capabilities.insert(capability.to_string(), entries);
    }
    out
}

impl Coverage {
    pub fn render(&self) -> String {
        let mut out = String::from("requirement coverage (source citations)\n");
        for (cap, entries) in &self.capabilities {
            let _ = writeln!(out, "\n{cap}");
            for e in entries {
                let mark = match (e.citing_files, e.ignored) {
                    (0, true) => "  <- ignored",
                    (0, false) => "  <- uncited",
                    _ => "",
                };
                let _ = writeln!(out, "  [{}] {}{mark}", e.citing_files, e.requirement);
            }
        }
        let ignored = if self.ignored > 0 {
            format!(" ({} ignored)", self.ignored)
        } else {
            String::new()
        };
        let _ = writeln!(out, "\ncoverage: {}/{}{ignored}", self.cited, self.total);
        out
    }
}
