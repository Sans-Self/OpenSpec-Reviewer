//! Paragraph diff with word-level marks, scenario matching by name, and
//! plain line diffs for artefacts.

use super::normalize::paragraphs;
use crate::model::{Requirement, Scenario};
use serde::Serialize;
use similar::{ChangeTag, DiffOp, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanMark {
    Equal,
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub mark: SpanMark,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParaKind {
    Equal,
    Added,
    Removed,
    Changed,
    /// A `@@` style separator between hunks of a line diff.
    Separator,
}

impl ParaKind {
    pub fn glyph(self) -> char {
        match self {
            ParaKind::Equal => ' ',
            ParaKind::Added => '+',
            ParaKind::Removed => '-',
            ParaKind::Changed => '~',
            ParaKind::Separator => '@',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineRole {
    Name,
    Body,
    Scenario,
    Blank,
}

/// One rendered line of a diff view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffLine {
    pub kind: ParaKind,
    pub role: LineRole,
    pub spans: Vec<Span>,
}

impl DiffLine {
    pub fn plain(kind: ParaKind, role: LineRole, text: impl Into<String>) -> DiffLine {
        let mark = match kind {
            ParaKind::Added => SpanMark::Added,
            ParaKind::Removed => SpanMark::Removed,
            _ => SpanMark::Equal,
        };
        DiffLine {
            kind,
            role,
            spans: vec![Span {
                mark,
                text: text.into(),
            }],
        }
    }

    pub fn blank() -> DiffLine {
        DiffLine {
            kind: ParaKind::Equal,
            role: LineRole::Blank,
            spans: Vec::new(),
        }
    }

    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    /// The line as it reads on one side of a side-by-side view.
    pub fn side(&self, mark_to_drop: SpanMark) -> Option<String> {
        match (self.kind, mark_to_drop) {
            (ParaKind::Added, SpanMark::Added) | (ParaKind::Removed, SpanMark::Removed) => None,
            _ => Some(
                self.spans
                    .iter()
                    .filter(|s| s.mark != mark_to_drop)
                    .map(|s| s.text.as_str())
                    .collect(),
            ),
        }
    }
}

/// Word-level diff of two normalized paragraphs.
pub fn diff_words(before: &str, after: &str) -> Vec<Span> {
    let diff = TextDiff::from_words(before, after);
    let mut spans: Vec<Span> = Vec::new();
    for change in diff.iter_all_changes() {
        let mark = match change.tag() {
            ChangeTag::Equal => SpanMark::Equal,
            ChangeTag::Delete => SpanMark::Removed,
            ChangeTag::Insert => SpanMark::Added,
        };
        let text = change.value();
        match spans.last_mut() {
            Some(last) if last.mark == mark => last.text.push_str(text),
            _ => spans.push(Span {
                mark,
                text: text.to_string(),
            }),
        }
    }
    spans
}

/// Paragraph-level diff of two bodies; changed paragraphs carry word marks.
pub fn diff_paragraphs(before: &str, after: &str, role: LineRole) -> Vec<DiffLine> {
    let b = paragraphs(before);
    let a = paragraphs(after);
    let b_refs: Vec<&str> = b.iter().map(String::as_str).collect();
    let a_refs: Vec<&str> = a.iter().map(String::as_str).collect();
    let diff = TextDiff::from_slices(&b_refs, &a_refs);
    let mut out = Vec::new();
    for op in diff.ops() {
        match *op {
            DiffOp::Equal { old_index, len, .. } => out.extend(
                b[old_index..old_index + len]
                    .iter()
                    .map(|p| DiffLine::plain(ParaKind::Equal, role, p)),
            ),
            DiffOp::Delete {
                old_index, old_len, ..
            } => out.extend(
                b[old_index..old_index + old_len]
                    .iter()
                    .map(|p| DiffLine::plain(ParaKind::Removed, role, p)),
            ),
            DiffOp::Insert {
                new_index, new_len, ..
            } => out.extend(
                a[new_index..new_index + new_len]
                    .iter()
                    .map(|p| DiffLine::plain(ParaKind::Added, role, p)),
            ),
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let olds = &b[old_index..old_index + old_len];
                let news = &a[new_index..new_index + new_len];
                let paired = olds.len().min(news.len());
                for (o, n) in olds[..paired].iter().zip(&news[..paired]) {
                    out.push(DiffLine {
                        kind: ParaKind::Changed,
                        role,
                        spans: diff_words(o, n),
                    });
                }
                out.extend(
                    olds[paired..]
                        .iter()
                        .map(|p| DiffLine::plain(ParaKind::Removed, role, p)),
                );
                out.extend(
                    news[paired..]
                        .iter()
                        .map(|p| DiffLine::plain(ParaKind::Added, role, p)),
                );
            }
        }
    }
    out
}

/// A scenario as seen from both sides, matched by name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum ScenarioMatch {
    Same {
        name: String,
        #[serde(skip)]
        lines: Vec<DiffLine>,
    },
    Changed {
        name: String,
        #[serde(skip)]
        lines: Vec<DiffLine>,
    },
    Added {
        name: String,
        #[serde(skip)]
        lines: Vec<DiffLine>,
    },
    Removed {
        name: String,
        #[serde(skip)]
        lines: Vec<DiffLine>,
    },
}

impl ScenarioMatch {
    pub fn name(&self) -> &str {
        match self {
            ScenarioMatch::Same { name, .. }
            | ScenarioMatch::Changed { name, .. }
            | ScenarioMatch::Added { name, .. }
            | ScenarioMatch::Removed { name, .. } => name,
        }
    }

    pub fn lines(&self) -> &[DiffLine] {
        match self {
            ScenarioMatch::Same { lines, .. }
            | ScenarioMatch::Changed { lines, .. }
            | ScenarioMatch::Added { lines, .. }
            | ScenarioMatch::Removed { lines, .. } => lines,
        }
    }

    pub fn heading_kind(&self) -> ParaKind {
        match self {
            ScenarioMatch::Same { .. } => ParaKind::Equal,
            ScenarioMatch::Changed { .. } => ParaKind::Changed,
            ScenarioMatch::Added { .. } => ParaKind::Added,
            ScenarioMatch::Removed { .. } => ParaKind::Removed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementDiff {
    #[serde(skip)]
    pub body: Vec<DiffLine>,
    pub scenarios: Vec<ScenarioMatch>,
    pub changed: bool,
}

fn all_equal(lines: &[DiffLine]) -> bool {
    lines.iter().all(|l| l.kind == ParaKind::Equal)
}

pub fn marked_lines(text: &str, kind: ParaKind) -> Vec<DiffLine> {
    text.lines()
        .map(|l| DiffLine::plain(kind, LineRole::Body, l))
        .collect()
}

pub fn plain_lines(text: &str) -> Vec<DiffLine> {
    marked_lines(text, ParaKind::Equal)
}

fn marked_paragraphs(text: &str, kind: ParaKind) -> Vec<DiffLine> {
    paragraphs(text)
        .into_iter()
        .map(|p| DiffLine::plain(kind, LineRole::Body, p))
        .collect()
}

fn one_sided(scenario: &Scenario, kind: ParaKind) -> ScenarioMatch {
    let lines = marked_paragraphs(&scenario.body, kind);
    let name = scenario.name.clone();
    match kind {
        ParaKind::Removed => ScenarioMatch::Removed { name, lines },
        _ => ScenarioMatch::Added { name, lines },
    }
}

/// Scenarios matched by name: after side in order, then canon-only ones.
pub fn match_scenarios(before: &[Scenario], after: &[Scenario]) -> Vec<ScenarioMatch> {
    let mut out: Vec<ScenarioMatch> = after
        .iter()
        .map(|a| match before.iter().find(|b| b.name == a.name) {
            Some(b) => {
                let lines = diff_paragraphs(&b.body, &a.body, LineRole::Body);
                if all_equal(&lines) {
                    ScenarioMatch::Same {
                        name: a.name.clone(),
                        lines,
                    }
                } else {
                    ScenarioMatch::Changed {
                        name: a.name.clone(),
                        lines,
                    }
                }
            }
            None => one_sided(a, ParaKind::Added),
        })
        .collect();
    out.extend(
        before
            .iter()
            .filter(|b| !after.iter().any(|a| a.name == b.name))
            .map(|b| one_sided(b, ParaKind::Removed)),
    );
    out
}

pub fn diff_requirements(before: &Requirement, after: &Requirement) -> RequirementDiff {
    let body = diff_paragraphs(&before.body, &after.body, LineRole::Body);
    let scenarios = match_scenarios(&before.scenarios, &after.scenarios);
    let changed = !all_equal(&body)
        || scenarios
            .iter()
            .any(|s| !matches!(s, ScenarioMatch::Same { .. }));
    RequirementDiff {
        body,
        scenarios,
        changed,
    }
}

/// A requirement seen from one side only, every line marked `kind`.
pub fn one_sided_requirement(req: &Requirement, kind: ParaKind) -> RequirementDiff {
    RequirementDiff {
        body: marked_paragraphs(&req.body, kind),
        scenarios: req.scenarios.iter().map(|s| one_sided(s, kind)).collect(),
        changed: true,
    }
}

/// Plain line diff with three lines of context, for artefacts and canon.
pub fn diff_lines(before: &str, after: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(before, after);
    let groups = diff.grouped_ops(3);
    let mut out = Vec::new();
    for (n, group) in groups.iter().enumerate() {
        let (Some(first), Some(last)) = (group.first(), group.last()) else {
            continue;
        };
        let old_start = first.old_range().start + 1;
        let new_start = first.new_range().start + 1;
        let old_len = last.old_range().end - first.old_range().start;
        let new_len = last.new_range().end - first.new_range().start;
        if n > 0 || old_start > 1 || new_start > 1 {
            out.push(DiffLine::plain(
                ParaKind::Separator,
                LineRole::Body,
                format!("@@ -{old_start},{old_len} +{new_start},{new_len} @@"),
            ));
        }
        for op in group {
            for change in diff.iter_changes(op) {
                let kind = match change.tag() {
                    ChangeTag::Equal => ParaKind::Equal,
                    ChangeTag::Delete => ParaKind::Removed,
                    ChangeTag::Insert => ParaKind::Added,
                };
                let text = change.value().trim_end_matches(['\n', '\r']);
                out.push(DiffLine::plain(kind, LineRole::Body, text));
            }
        }
    }
    out
}
