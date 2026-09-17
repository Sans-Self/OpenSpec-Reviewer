//! Output of the `lint` subcommand: notes to stdout, errors to stderr, one
//! summary line; or one JSON document.

use super::colour::{ansi, Palette};
use crate::citations::coverage::Coverage;
use crate::citations::{LintFinding, LintReport};
use crate::review::Severity;
use serde::Serialize;
use std::fmt::Write;

pub struct LintOutput {
    pub stdout: String,
    pub stderr: String,
}

fn severity_style(s: Severity, palette: Palette) -> ratatui::style::Style {
    match s {
        Severity::Error => palette.error(),
        Severity::Warning => palette.warning(),
        Severity::Note => palette.note(),
    }
}

/// A message with every backticked `capability § requirement` in bold: the
/// anchor is what the reader looks for, the rest says what is wrong with it.
fn emphasize_anchors(message: &str, palette: Palette) -> String {
    message
        .split('`')
        .enumerate()
        .map(|(i, part)| {
            if i % 2 == 1 && part.contains(" § ") {
                format!("`{}`", ansi::paint(palette, palette.heading(), part))
            } else if i % 2 == 1 {
                format!("`{part}`")
            } else {
                part.to_string()
            }
        })
        .collect()
}

fn write_finding(out: &mut String, f: &LintFinding, palette: Palette) {
    let severity = ansi::paint(
        palette,
        severity_style(f.severity, palette),
        &f.severity.to_string(),
    );
    let message = emphasize_anchors(&f.message, palette);
    if f.file.is_empty() {
        let _ = writeln!(out, "{severity}: {message}");
    } else {
        let file = ansi::paint(palette, palette.muted(), &f.file);
        let _ = writeln!(out, "{severity}: {file}: {message}");
    }
    for d in &f.details {
        let _ = writeln!(out, "    {d}");
    }
}

/// The summary line with its error count in the error colour when there
/// is one; a zero stays quiet.
fn summary(report: &LintReport, palette: Palette) -> String {
    let line = report.summary_line();
    let errors = report.summary.errors;
    if errors == 0 {
        return line;
    }
    let plain = format!("{errors} error{}", if errors == 1 { "" } else { "s" });
    line.replacen(&plain, &ansi::paint(palette, palette.error(), &plain), 1)
}

pub fn render_text(
    report: &LintReport,
    coverage: Option<&Coverage>,
    palette: Palette,
) -> LintOutput {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for f in &report.findings {
        match f.severity {
            Severity::Error => write_finding(&mut stderr, f, palette),
            Severity::Warning | Severity::Note => write_finding(&mut stdout, f, palette),
        }
    }
    let _ = writeln!(stdout, "{}", summary(report, palette));
    if let Some(c) = coverage {
        stdout.push('\n');
        stdout.push_str(&c.render());
    }
    LintOutput { stdout, stderr }
}

#[derive(Serialize)]
struct Document<'a> {
    #[serde(flatten)]
    report: &'a LintReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage: Option<&'a Coverage>,
}

pub fn render_json(report: &LintReport, coverage: Option<&Coverage>) -> String {
    serde_json::to_string_pretty(&Document { report, coverage }).expect("report serializes")
}
