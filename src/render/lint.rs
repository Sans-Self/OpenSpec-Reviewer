//! Output of the `lint` subcommand: notes to stdout, errors to stderr, one
//! summary line; or one JSON document.

use crate::citations::coverage::Coverage;
use crate::citations::LintReport;
use crate::review::Severity;
use serde::Serialize;
use std::fmt::Write;

pub struct LintOutput {
    pub stdout: String,
    pub stderr: String,
}

fn write_finding(out: &mut String, f: &crate::citations::LintFinding) {
    let _ = writeln!(out, "{f}");
    for d in &f.details {
        let _ = writeln!(out, "    {d}");
    }
}

pub fn render_text(report: &LintReport, coverage: Option<&Coverage>) -> LintOutput {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for f in &report.findings {
        match f.severity {
            Severity::Error => write_finding(&mut stderr, f),
            Severity::Warning | Severity::Note | Severity::Hint => write_finding(&mut stdout, f),
        }
    }
    let _ = writeln!(stdout, "{}", report.summary_line());
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
