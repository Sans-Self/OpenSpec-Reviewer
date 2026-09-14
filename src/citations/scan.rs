//! Fold spec, delta and source texts into a `CitationIndex`.

use super::{Citation, CitationIndex, Citer, Grammar, Resolution};
use crate::model::{Canon, DeltaKind, DeltaSpec};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// A canon spec as text, with the capability its directory names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecFile {
    pub capability: String,
    pub path: PathBuf,
    pub text: String,
}

/// A source file under a configured root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
}

/// One open change: its parsed deltas and the raw delta texts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenChange {
    pub name: String,
    pub deltas: Vec<DeltaSpec>,
    pub delta_files: Vec<SpecFile>,
}

/// A citation and where it was found, before resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sighting {
    pub file: PathBuf,
    pub citation: Citation,
    /// The name stopped at a newline; if it dangles, wrapping is the
    /// likely reason.
    pub ends_at_line_break: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scanned {
    pub index: CitationIndex,
    pub sightings: Vec<Sighting>,
}

impl Scanned {
    pub fn dangling(&self) -> impl Iterator<Item = (&Sighting, Resolution)> {
        self.sightings.iter().filter_map(|s| {
            let r = self.index.resolve(&s.citation);
            (r != Resolution::Resolved).then_some((s, r))
        })
    }
}

pub fn reason(r: &Resolution, c: &Citation) -> String {
    match r {
        Resolution::UnknownCapability => format!("capability `{}` does not exist", c.capability),
        Resolution::UnknownRequirement => {
            format!("no requirement `{}` in {}", c.requirement, c.capability)
        }
        Resolution::Resolved => String::new(),
    }
}

pub const WRAP_HINT: &str = "; the name looks cut off by a line break, a backtick span may wrap";

/// The reason, plus the wrap hint when the sighting ended at a newline.
pub fn reason_for(s: &Sighting, r: &Resolution) -> String {
    let mut text = reason(r, &s.citation);
    if s.ends_at_line_break {
        text.push_str(WRAP_HINT);
    }
    text
}

fn in_flight(changes: &[OpenChange]) -> BTreeSet<Citation> {
    changes
        .iter()
        .flat_map(|c| c.deltas.iter())
        .flat_map(|d| {
            d.entries
                .iter()
                .filter(|e| e.kind == DeltaKind::Added)
                .map(move |e| Citation::new(&d.capability, &e.requirement.name))
        })
        .collect()
}

/// Canon and in-flight requirements resolve citations; specs and source
/// record citers; deltas are checked but do not count as citers.
pub fn scan(
    canon: &Canon,
    specs: &[SpecFile],
    changes: &[OpenChange],
    sources: &[SourceFile],
    grammar: &Grammar,
) -> Scanned {
    let mut index = CitationIndex {
        canon: canon
            .specs
            .iter()
            .map(|(cap, reqs)| {
                (
                    cap.clone(),
                    reqs.iter()
                        .map(|r| super::normalize_name(&r.name))
                        .collect(),
                )
            })
            .collect(),
        in_flight: in_flight(changes),
        ..CitationIndex::default()
    };
    let mut sightings = Vec::new();

    let literal = Grammar::new(None);
    let sighting = |file: &PathBuf, found: &super::Found| Sighting {
        file: file.clone(),
        citation: found.citation.clone(),
        ends_at_line_break: found.ends_at_line_break,
    };
    for spec in specs {
        for found in literal.found(&spec.text) {
            index
                .citers
                .entry(found.citation.clone())
                .or_default()
                .push(Citer {
                    file: spec.path.clone(),
                    citing_capability: Some(spec.capability.clone()),
                });
            sightings.push(sighting(&spec.path, &found));
        }
    }
    for source in sources {
        for found in grammar.found(&source.text) {
            index
                .citers
                .entry(found.citation.clone())
                .or_default()
                .push(Citer {
                    file: source.path.clone(),
                    citing_capability: None,
                });
            sightings.push(sighting(&source.path, &found));
        }
    }
    for delta in changes.iter().flat_map(|c| c.delta_files.iter()) {
        for found in literal.found(&delta.text) {
            sightings.push(sighting(&delta.path, &found));
        }
    }
    Scanned { index, sightings }
}
