//! The line scanner. One grammar covers canon and delta files:
//!
//! ```text
//! ## <section>
//! ### Requirement: <name>
//! #### Scenario: <name>
//! - FROM: `### Requirement: <name>`      (RENAMED only)
//! - TO:   `### Requirement: <name>`      (RENAMED only)
//! ```
//!
//! Everything else is body text attached to the nearest requirement or
//! scenario above it.

use super::{DeltaEntry, DeltaKind, DeltaSpec, Requirement, Scenario};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("{file}:{line}: requirement under unknown section `{heading}`; expected ADDED, MODIFIED, REMOVED or RENAMED")]
    UnknownSection {
        file: String,
        line: usize,
        heading: String,
    },
    #[error("{file}:{line}: `- FROM:` without a following `- TO:`")]
    OrphanFrom { file: String, line: usize },
    #[error("{file}:{line}: `- TO:` without a preceding `- FROM:`")]
    OrphanTo { file: String, line: usize },
    #[error("{file}:{line}: rename line does not carry a `### Requirement:` heading")]
    MalformedRename { file: String, line: usize },
}

const REQUIREMENT: &str = "### Requirement:";
const SCENARIO: &str = "#### Scenario:";

/// A requirement or scenario under construction; the body is finished when
/// the next heading arrives.
#[derive(Default)]
struct Builder {
    requirements: Vec<Requirement>,
    current: Option<Requirement>,
    scenario: Option<Scenario>,
    body: Vec<String>,
}

impl Builder {
    fn push_line(&mut self, line: &str) {
        if self.current.is_some() {
            self.body.push(line.trim_end().to_string());
        }
    }

    fn flush_body(&mut self) -> String {
        let text = trim_blank_lines(&self.body);
        self.body.clear();
        text
    }

    fn close_scenario(&mut self) {
        let body = self.flush_body();
        if let Some(mut scenario) = self.scenario.take() {
            scenario.body = body;
            if let Some(req) = self.current.as_mut() {
                req.scenarios.push(scenario);
            }
        } else if let Some(req) = self.current.as_mut() {
            req.body = body;
        }
    }

    fn close_requirement(&mut self) {
        self.close_scenario();
        if let Some(req) = self.current.take() {
            self.requirements.push(req);
        }
    }

    fn open_requirement(&mut self, name: &str) {
        self.close_requirement();
        self.current = Some(Requirement {
            name: name.trim().to_string(),
            body: String::new(),
            scenarios: Vec::new(),
        });
    }

    fn open_scenario(&mut self, name: &str) {
        self.close_scenario();
        if self.current.is_some() {
            self.scenario = Some(Scenario {
                name: name.trim().to_string(),
                body: String::new(),
            });
        }
    }

    fn finish(mut self) -> Vec<Requirement> {
        self.close_requirement();
        self.requirements
    }
}

fn trim_blank_lines(lines: &[String]) -> String {
    let start = lines.iter().position(|l| !l.trim().is_empty());
    let end = lines.iter().rposition(|l| !l.trim().is_empty());
    match (start, end) {
        (Some(s), Some(e)) => lines[s..=e].join("\n"),
        _ => String::new(),
    }
}

/// Parse a canonical spec: every `### Requirement:` in the file, whatever
/// `##` section it sits under.
pub fn parse_canon_spec(text: &str) -> Vec<Requirement> {
    let mut b = Builder::default();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix(REQUIREMENT) {
            b.open_requirement(name);
        } else if let Some(name) = line.strip_prefix(SCENARIO) {
            b.open_scenario(name);
        } else if line.starts_with("## ") || line.starts_with("# ") {
            b.close_requirement();
        } else {
            b.push_line(line);
        }
    }
    b.finish()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Added,
    Modified,
    Removed,
    Renamed,
}

fn section_of(heading: &str) -> Option<Section> {
    let word = heading.split_whitespace().next()?;
    match word {
        "ADDED" => Some(Section::Added),
        "MODIFIED" => Some(Section::Modified),
        "REMOVED" => Some(Section::Removed),
        "RENAMED" => Some(Section::Renamed),
        _ => None,
    }
}

fn rename_name(rest: &str, file: &str, line: usize) -> Result<String, ParseError> {
    let inner = rest.trim().trim_matches('`').trim();
    inner
        .strip_prefix(REQUIREMENT)
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or_else(|| ParseError::MalformedRename {
            file: file.to_string(),
            line,
        })
}

/// Parse a delta spec. `file` is only used in error messages.
pub fn parse_delta_spec(capability: &str, file: &str, text: &str) -> Result<DeltaSpec, ParseError> {
    let mut entries: Vec<DeltaEntry> = Vec::new();
    let mut section: Option<Section> = None;
    let mut heading = String::new();
    let mut b = Builder::default();
    let mut pending_from: Option<(String, usize)> = None;

    let flush = |b: &mut Builder, section: Option<Section>, entries: &mut Vec<DeltaEntry>| {
        let kind = match section {
            Some(Section::Added) => DeltaKind::Added,
            Some(Section::Modified) => DeltaKind::Modified,
            Some(Section::Removed) => DeltaKind::Removed,
            // RENAMED sections carry FROM/TO pairs, not requirement bodies.
            // A requirement heading there was already refused.
            Some(Section::Renamed) | None => return,
        };
        let reqs = std::mem::take(b).finish();
        entries.extend(reqs.into_iter().map(|requirement| DeltaEntry {
            kind: kind.clone(),
            requirement,
        }));
    };

    for (index, line) in text.lines().enumerate() {
        let lineno = index + 1;
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some((_, from_line)) = pending_from.take() {
                return Err(ParseError::OrphanFrom {
                    file: file.to_string(),
                    line: from_line,
                });
            }
            flush(&mut b, section, &mut entries);
            heading = rest.trim().to_string();
            section = section_of(rest);
            continue;
        }
        if let Some(name) = line.strip_prefix(REQUIREMENT) {
            match section {
                Some(Section::Added) | Some(Section::Modified) | Some(Section::Removed) => {
                    b.open_requirement(name)
                }
                _ => {
                    return Err(ParseError::UnknownSection {
                        file: file.to_string(),
                        line: lineno,
                        heading: heading.clone(),
                    })
                }
            }
            continue;
        }
        if let Some(name) = line.strip_prefix(SCENARIO) {
            b.open_scenario(name);
            continue;
        }
        if section == Some(Section::Renamed) {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("- FROM:") {
                if let Some((_, from_line)) = pending_from.take() {
                    return Err(ParseError::OrphanFrom {
                        file: file.to_string(),
                        line: from_line,
                    });
                }
                pending_from = Some((rename_name(rest, file, lineno)?, lineno));
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("- TO:") {
                let (from, _) = pending_from.take().ok_or(ParseError::OrphanTo {
                    file: file.to_string(),
                    line: lineno,
                })?;
                let to = rename_name(rest, file, lineno)?;
                entries.push(DeltaEntry {
                    kind: DeltaKind::Renamed { from },
                    requirement: Requirement {
                        name: to,
                        body: String::new(),
                        scenarios: Vec::new(),
                    },
                });
                continue;
            }
            continue;
        }
        b.push_line(line);
    }
    if let Some((_, from_line)) = pending_from {
        return Err(ParseError::OrphanFrom {
            file: file.to_string(),
            line: from_line,
        });
    }
    flush(&mut b, section, &mut entries);

    Ok(DeltaSpec {
        capability: capability.to_string(),
        entries,
    })
}
