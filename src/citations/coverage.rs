//! The ledger: per capability, how many source files cite each canon or
//! in-flight requirement. Spec-to-spec citations are cross-links, not
//! evidence, so they do not count.

use super::{Citation, CitationIndex};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub requirement: String,
    pub citing_files: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Coverage {
    pub capabilities: BTreeMap<String, Vec<Entry>>,
    pub cited: usize,
    pub total: usize,
}

pub fn coverage(index: &CitationIndex) -> Coverage {
    let mut ledger: BTreeMap<String, Vec<String>> = index
        .canon
        .iter()
        .map(|(cap, names)| (cap.clone(), names.iter().cloned().collect()))
        .collect();
    for c in &index.in_flight {
        let names = ledger.entry(c.capability.clone()).or_default();
        if !names.contains(&c.requirement) {
            names.push(c.requirement.clone());
        }
    }
    let mut out = Coverage::default();
    for (cap, mut names) in ledger {
        names.sort();
        let entries: Vec<Entry> = names
            .into_iter()
            .map(|requirement| {
                let citing_files = index
                    .citers_of(&Citation::new(&cap, &requirement))
                    .iter()
                    .filter(|c| c.citing_capability.is_none())
                    .count();
                out.total += 1;
                if citing_files > 0 {
                    out.cited += 1;
                }
                Entry {
                    requirement,
                    citing_files,
                }
            })
            .collect();
        out.capabilities.insert(cap, entries);
    }
    out
}

impl Coverage {
    pub fn render(&self) -> String {
        let mut out = String::from("requirement coverage (source citations)\n");
        for (cap, entries) in &self.capabilities {
            let _ = writeln!(out, "\n{cap}");
            for e in entries {
                let _ = writeln!(
                    out,
                    "  [{}] {}{}",
                    e.citing_files,
                    e.requirement,
                    if e.citing_files == 0 {
                        "  <- uncited"
                    } else {
                        ""
                    }
                );
            }
        }
        let _ = writeln!(out, "\ncoverage: {}/{}", self.cited, self.total);
        out
    }
}
