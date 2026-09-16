//! Plain text mirroring the inline view. No escape codes unless asked.

use super::colour::ansi;
use super::{approval, history_summary};
use crate::review::{inline_view, DiffLine, Pairing, ParaKind, Review, Severity, SpanMark};
use std::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub struct TextOptions {
    pub colour: bool,
    pub findings_only: bool,
    /// Hints in --findings-only output: off unless --hints asks for them.
    pub hints: bool,
}

fn paint(colour: bool, code: &str, text: &str) -> String {
    if colour && !text.is_empty() {
        format!("{code}{text}{}", ansi::RESET)
    } else {
        text.to_string()
    }
}

/// wdiff conventions: `[-removed-]` and `{+added+}`.
pub fn render_line(line: &DiffLine, colour: bool) -> String {
    let body: String = match line.kind {
        ParaKind::Changed => line
            .spans
            .iter()
            .map(|s| match s.mark {
                SpanMark::Equal => s.text.clone(),
                SpanMark::Removed => paint(colour, ansi::RED, &format!("[-{}-]", s.text)),
                SpanMark::Added => paint(colour, ansi::GREEN, &format!("{{+{}+}}", s.text)),
            })
            .collect(),
        ParaKind::Added => paint(colour, ansi::GREEN, &line.text()),
        ParaKind::Removed => paint(colour, ansi::RED, &line.text()),
        ParaKind::Separator => paint(colour, ansi::DIM, &line.text()),
        ParaKind::Equal => line.text(),
    };
    if line.spans.is_empty() {
        String::new()
    } else {
        format!("{} {}", line.kind.glyph(), body)
    }
}

fn severity_code(s: Severity) -> &'static str {
    match s {
        Severity::Error => ansi::RED,
        Severity::Warning => ansi::YELLOW,
        Severity::Note | Severity::Hint => ansi::DIM,
    }
}

fn write_pairing(out: &mut String, p: &Pairing, colour: bool) {
    let mark = approval(p).mark();
    let markers = format!(
        "{}{}{}",
        super::finding_marker(p),
        super::hint_marker(p),
        super::note_marker(p.state.note.is_some())
    );
    let heading = format!("{mark} {} {}", p.kind.glyph(), p.name);
    let _ = writeln!(
        out,
        "{}{}",
        paint(colour, ansi::BOLD, &heading),
        if markers.is_empty() {
            String::new()
        } else {
            format!(" {markers}")
        }
    );
    if approval(p) == crate::state::ApprovalStatus::Stale {
        let _ = writeln!(out, "    text changed since approval");
    }
    for line in inline_view(p) {
        let rendered = render_line(&line, colour);
        if rendered.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "    {rendered}");
        }
    }
    out.push('\n');
    for f in &p.findings {
        let _ = writeln!(
            out,
            "    {}: {}",
            paint(colour, severity_code(f.severity), &f.severity.to_string()),
            f.message
        );
        for d in &f.details {
            let _ = writeln!(out, "        {d}");
        }
    }
    for h in p.visible_hints() {
        let _ = writeln!(
            out,
            "    {}: {}: {}",
            paint(colour, severity_code(Severity::Hint), "hint"),
            h.kind,
            h.message
        );
        if let Some(scenario) = &h.scenario {
            let _ = writeln!(out, "        in scenario: {scenario}");
        }
        if let Some(quote) = &h.quote {
            let _ = writeln!(out, "        {quote}");
        }
    }
    if let Some(note) = &p.state.note {
        for (i, l) in note.lines().enumerate() {
            let _ = writeln!(
                out,
                "    {} {l}",
                if i == 0 { "✎ note:" } else { "       " }
            );
        }
    }
    let _ = writeln!(out, "    {}", history_summary(p));
    out.push('\n');
}

pub fn render(review: &Review, options: TextOptions) -> String {
    let colour = options.colour;
    let mut out = String::new();
    if options.findings_only {
        return render_findings_only(review, options.hints);
    }
    for change in &review.changes {
        let _ = writeln!(
            out,
            "{}  ({})",
            paint(colour, ansi::BOLD, &format!("# change {}", change.name)),
            review.origin
        );
        out.push('\n');
        if !change.artefacts.is_empty() {
            let _ = writeln!(out, "{}", paint(colour, ansi::BOLD, "## artefacts"));
            for a in &change.artefacts {
                let status = a.state.status(crate::review::normalize::text_hash(
                    a.artefact.after.as_deref().unwrap_or(""),
                ));
                let _ = writeln!(out, "{} {}", status.mark(), a.artefact.name);
                for line in a.lines() {
                    let _ = writeln!(out, "    {}", render_line(&line, colour));
                }
                if let Some(note) = &a.state.note {
                    let _ = writeln!(out, "    ✎ note: {note}");
                }
                out.push('\n');
            }
        }
        for cap in &change.capabilities {
            let _ = writeln!(
                out,
                "{}",
                paint(colour, ansi::BOLD, &format!("## {}", cap.name))
            );
            out.push('\n');
            for p in &cap.pairings {
                write_pairing(&mut out, p, colour);
            }
        }
    }
    if !review.canon_edits.is_empty() {
        let _ = writeln!(out, "{}", paint(colour, ansi::BOLD, "## canon"));
        for edit in &review.canon_edits {
            let _ = writeln!(out, "### {}", edit.path);
            for line in &edit.lines {
                let _ = writeln!(out, "    {}", render_line(line, colour));
            }
            out.push('\n');
        }
    }
    write_notices(&mut out, review, colour);
    let _ = writeln!(
        out,
        "summary: {}; {}",
        review.summary,
        review.glossary_summary()
    );
    out
}

/// Findings about the configuration: the same lines the lint prints.
fn write_notices(out: &mut String, review: &Review, colour: bool) {
    for n in &review.notices {
        let _ = writeln!(
            out,
            "{}: {}: {}",
            paint(colour, severity_code(n.severity), &n.severity.to_string()),
            n.file,
            n.message
        );
    }
}

/// One line per finding: `severity  change/capability/requirement: message`.
pub fn render_findings_only(review: &Review, hints: bool) -> String {
    let mut out = String::new();
    for p in review.pairings() {
        for f in &p.findings {
            let _ = writeln!(out, "{:<7}  {}: {}", f.severity, f.location, f.message);
        }
        if !hints {
            continue;
        }
        for h in p.visible_hints() {
            let _ = writeln!(
                out,
                "{:<7}  {}: {}: {}",
                Severity::Hint,
                p.location(),
                h.kind,
                h.message
            );
        }
    }
    for n in &review.notices {
        let _ = writeln!(out, "{:<7}  {}: {}", n.severity, n.file, n.message);
    }
    let _ = writeln!(
        out,
        "summary: {}; {}",
        review.summary,
        review.glossary_summary()
    );
    out
}
