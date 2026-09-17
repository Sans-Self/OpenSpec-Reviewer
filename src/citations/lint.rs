//! The lint as one pure function over what the workspace collected.

use super::config::{Config, Evidence, IgnoreEvidence};
use super::evidence::{check_hashes, check_paths, check_tests, Probe};
use super::radius::blast_radius;
use super::scan::{reason_for, scan, OpenChange, SourceFile, SpecFile};
use super::structure::{check_changes, ignore_warnings, ChangeDir};
use super::{CitationIndex, Grammar};
use crate::glossary::{self, Glossary};
use crate::model::{Canon, DeltaSpec, Register};
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
    pub fn new(
        severity: Severity,
        file: impl Into<String>,
        message: impl Into<String>,
    ) -> LintFinding {
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
    /// `glossary: N terms` or `no glossary`.
    pub glossary: String,
    #[serde(skip)]
    pub index: CitationIndex,
    /// Every requirement the repository asserts, for the ledger.
    #[serde(skip)]
    pub register: Register,
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
        line.push_str("; ");
        line.push_str(&self.glossary);
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

/// The `[[lint.ignore_evidence]]` entries, and which of them silenced
/// something. An entry is matched only against its own family and only as
/// an equal string, so a path entry never swallows a citation.
struct EvidenceIgnores<'a> {
    entries: Vec<(Evidence, &'a str)>,
    used: Vec<bool>,
}

impl<'a> EvidenceIgnores<'a> {
    fn new(config: &'a Config) -> EvidenceIgnores<'a> {
        let entries: Vec<_> = config
            .lint
            .ignore_evidence
            .iter()
            .filter_map(IgnoreEvidence::names)
            .collect();
        let used = vec![false; entries.len()];
        EvidenceIgnores { entries, used }
    }

    /// Whether this finding is dismissed, marking the entry that dismissed it.
    fn dismisses(&mut self, kind: Evidence, value: &str) -> bool {
        let mut hit = false;
        for (i, (k, v)) in self.entries.iter().enumerate() {
            if *k == kind && *v == value {
                self.used[i] = true;
                hit = true;
            }
        }
        hit
    }

    /// Entries that silenced nothing, as warnings for the summary.
    fn stale(&self) -> Vec<LintFinding> {
        self.entries
            .iter()
            .zip(&self.used)
            .filter(|(_, used)| !**used)
            .map(|((kind, value), _)| {
                LintFinding::new(
                    Severity::Warning,
                    super::config::CONFIG_PATH,
                    format!(
                        "ignored evidence `{value}` silences nothing; remove the {} entry from `[[lint.ignore_evidence]]` in {}",
                        kind.key(),
                        super::config::CONFIG_PATH
                    ),
                )
            })
            .collect()
    }
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
    let mut ignores = EvidenceIgnores::new(config);
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
        let missing = checked
            .missing
            .into_iter()
            .filter(|(_, path)| !ignores.dismisses(Evidence::Path, path))
            .collect::<Vec<_>>();
        findings.extend(missing.into_iter().map(|(spec, path)| {
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
        let missing = checked
            .missing
            .into_iter()
            .filter(|(_, name)| !ignores.dismisses(Evidence::Test, name))
            .collect::<Vec<_>>();
        findings.extend(missing.into_iter().map(|(spec, name)| {
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
        let missing = checked
            .missing
            .into_iter()
            .filter(|(_, hash)| !ignores.dismisses(Evidence::Commit, hash))
            .collect::<Vec<_>>();
        findings.extend(missing.into_iter().map(|(spec, hash)| {
            LintFinding::new(
                Severity::Error,
                display(&spec),
                format!("cited commit not found: {hash}"),
            )
        }));
    }

    let dangling: Vec<_> = scanned
        .dangling()
        .filter(|(s, _)| !ignores.dismisses(Evidence::Citation, &s.citation.to_string()))
        .collect();
    findings.extend(dangling.into_iter().map(|(s, r)| {
        LintFinding::new(
            Severity::Error,
            display(&s.file),
            format!("dangling citation `{}`: {}", s.citation, reason_for(s, &r)),
        )
    }));

    for change in input.changes {
        findings.extend(
            blast_radius(&scanned.index, change)
                .iter()
                .map(LintFinding::from_review),
        );
    }

    let open_deltas: Vec<DeltaSpec> = input
        .changes
        .iter()
        .flat_map(|c| c.deltas.iter().cloned())
        .collect();
    let register = Register::build(input.canon, &open_deltas);
    findings.extend(ignores.stale());
    for d in ignore_warnings(config, &register, input.canon, &open_deltas) {
        findings.push(LintFinding::new(
            Severity::Warning,
            super::config::CONFIG_PATH,
            d.message(),
        ));
    }

    let glossary = Glossary::build(input.canon, &[], &config.definitions.capability);
    let glossary_line = match glossary.terms.len() {
        0 => "no glossary".to_string(),
        1 => "glossary: 1 term".to_string(),
        n => format!("glossary: {n} terms"),
    };
    if !glossary.is_empty() {
        let spec_path = |cap: &str| format!("openspec/specs/{cap}/spec.md");
        for hit in glossary::deprecated_in_canon(&glossary, input.canon) {
            findings.push(LintFinding::new(
                Severity::Warning,
                spec_path(&hit.capability),
                format!(
                    "{} § {}{}: uses deprecated synonym `{}`, the term is `{}`",
                    hit.capability,
                    hit.requirement,
                    hit.scenario
                        .as_ref()
                        .map(|s| format!(" # {s}"))
                        .unwrap_or_default(),
                    hit.synonym,
                    hit.term
                ),
            ));
        }
        for term in glossary::unused_terms(&glossary, input.canon, &open_deltas) {
            findings.push(LintFinding::new(
                Severity::Note,
                spec_path(&glossary.capability),
                format!(
                    "defined but unused: no requirement outside the glossary uses `{}`",
                    term.name
                ),
            ));
        }
        for term in glossary::unbound_terms(&glossary) {
            findings.push(LintFinding::new(
                Severity::Warning,
                spec_path(&glossary.capability),
                match term.binding.names_another() {
                    Some(other) => format!(
                        "term without a binding line: `{}` opens by binding `{other}`",
                        term.name
                    ),
                    None => format!("term without a binding line: `{}`", term.name),
                },
            ));
        }
        for r in glossary::recurring_undefined(
            &glossary,
            input.canon,
            config.definitions.min_recurrence,
            &config.definitions.ignores(),
        ) {
            let mut f = LintFinding::new(
                Severity::Note,
                spec_path(&glossary.capability),
                format!("recurring term without definition: `{}`", r.term),
            );
            f.details = r.uses;
            findings.push(f);
        }
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
        glossary: glossary_line,
        index: scanned.index,
        register,
    })
}

fn display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
