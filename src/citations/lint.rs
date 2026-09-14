//! The lint as one pure function over what the workspace collected.

use super::config::Config;
use super::evidence::{check_hashes, check_paths, check_tests, Probe};
use super::radius::blast_radius;
use super::scan::{reason, scan, OpenChange, SourceFile, SpecFile};
use super::structure::{check_changes, ChangeDir};
use super::{CitationIndex, Grammar};
use crate::model::Canon;
use crate::review::{Finding, Severity, Summary};
use regex::Regex;
use serde::Serialize;
use std::fmt;
use std::path::Path;
use thiserror::Error;

/// A finding anchored to a file rather than a pairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LintFinding {
    pub severity: Severity,
    pub file: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
}

impl LintFinding {
    fn new(severity: Severity, file: impl Into<String>, message: impl Into<String>) -> LintFinding {
        LintFinding {
            severity,
            file: file.into(),
            message: message.into(),
            details: Vec::new(),
        }
    }

    fn from_review(f: &Finding) -> LintFinding {
        LintFinding {
            severity: f.severity,
            file: format!(
                "openspec/changes/{}/specs/{}/spec.md",
                f.location.change, f.location.capability
            ),
            message: format!(
                "{} § {}: {}",
                f.location.capability, f.location.requirement, f.message
            ),
            details: f.details.clone(),
        }
    }
}

impl fmt::Display for LintFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.file.is_empty() {
            write!(f, "{}: {}", self.severity, self.message)
        } else {
            write!(f, "{}: {}: {}", self.severity, self.file, self.message)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    pub specs: usize,
    pub changes: usize,
    pub paths: usize,
    pub tests: usize,
    pub hashes: usize,
    pub citations: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LintReport {
    pub findings: Vec<LintFinding>,
    pub counts: Counts,
    pub summary: Summary,
    /// Config fields left out, named in the summary line.
    pub unconfigured: Vec<&'static str>,
    #[serde(skip)]
    pub index: CitationIndex,
}

impl LintReport {
    pub fn summary_line(&self) -> String {
        let c = self.counts;
        let mut line = format!(
            "lint: {} specs, {} changes, {} paths, {} tests, {} hashes, {} citations checked, {} error{}",
            c.specs,
            c.changes,
            c.paths,
            c.tests,
            c.hashes,
            c.citations,
            self.summary.errors,
            if self.summary.errors == 1 { "" } else { "s" }
        );
        if !self.unconfigured.is_empty() {
            line.push_str(&format!(
                "; not configured: {}",
                self.unconfigured.join(", ")
            ));
        }
        line
    }

    pub fn exit_code(&self) -> i32 {
        self.summary.exit_code()
    }
}

#[derive(Debug, Error)]
pub enum LintError {
    #[error("openspec/reviewer.toml: test_pattern is not a valid regex: {0}")]
    TestPattern(regex::Error),
}

/// Everything the lint reads, gathered by `source::lint::Workspace`.
pub struct Input<'a> {
    pub canon: &'a Canon,
    pub specs: &'a [SpecFile],
    pub changes: &'a [OpenChange],
    pub change_dirs: &'a [ChangeDir],
    pub sources: &'a [SourceFile],
    pub probe: &'a dyn Probe,
}

pub fn lint(input: &Input<'_>, config: &Config) -> Result<LintReport, LintError> {
    let lint = &config.lint;
    let grammar = Grammar::new(lint.cite_helper.as_deref());
    let scanned = scan(
        input.canon,
        input.specs,
        input.changes,
        input.sources,
        &grammar,
    );
    let mut findings: Vec<LintFinding> = Vec::new();
    let mut counts = Counts {
        specs: input.specs.len(),
        changes: input.change_dirs.len(),
        citations: scanned.sightings.len(),
        ..Counts::default()
    };

    for (name, message) in check_changes(
        input.change_dirs,
        lint.change_scopes.as_deref(),
        &lint.grandfathered,
    ) {
        let file = if name.is_empty() {
            super::config::CONFIG_PATH.to_string()
        } else {
            format!("openspec/changes/{name}")
        };
        findings.push(LintFinding::new(Severity::Error, file, message));
    }

    if let (Some(prefixes), Some(extensions)) = (&lint.path_prefixes, &lint.path_extensions) {
        let checked = check_paths(input.specs, prefixes, extensions, input.probe);
        counts.paths = checked.count;
        findings.extend(checked.missing.into_iter().map(|(spec, path)| {
            LintFinding::new(
                Severity::Error,
                display(&spec),
                format!("cited path does not exist: {path}"),
            )
        }));
    }
    if let (Some(pattern), Some(_)) = (&lint.test_pattern, &lint.source_roots) {
        let re = Regex::new(pattern).map_err(LintError::TestPattern)?;
        let checked = check_tests(input.specs, &re, input.sources);
        counts.tests = checked.count;
        findings.extend(checked.missing.into_iter().map(|(spec, name)| {
            LintFinding::new(
                Severity::Error,
                display(&spec),
                format!("cited regression test not found in source: {name}"),
            )
        }));
    }
    {
        let checked = check_hashes(input.specs, input.probe);
        counts.hashes = checked.count;
        findings.extend(checked.missing.into_iter().map(|(spec, hash)| {
            LintFinding::new(
                Severity::Error,
                display(&spec),
                format!("cited commit not found: {hash}"),
            )
        }));
    }

    findings.extend(scanned.dangling().map(|(s, r)| {
        LintFinding::new(
            Severity::Error,
            display(&s.file),
            format!(
                "dangling citation `{}`: {}",
                s.citation,
                reason(&r, &s.citation)
            ),
        )
    }));

    for change in input.changes {
        findings.extend(
            blast_radius(&scanned.index, change)
                .iter()
                .map(LintFinding::from_review),
        );
    }

    let summary = findings.iter().fold(Summary::default(), |mut s, f| {
        match f.severity {
            Severity::Error => s.errors += 1,
            Severity::Warning => s.warnings += 1,
            Severity::Note => s.notes += 1,
        }
        s
    });
    let mut unconfigured = lint.missing();
    if lint.source_roots.is_some() {
        unconfigured.retain(|f| *f != "source_roots");
    }
    Ok(LintReport {
        findings,
        counts,
        summary,
        unconfigured,
        index: scanned.index,
    })
}

fn display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
