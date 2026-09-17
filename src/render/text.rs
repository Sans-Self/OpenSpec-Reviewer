//! Plain text mirroring the inline view. Escape codes only on a terminal
//! or when asked, and then from the same palette the view draws with.

use super::colour::{ansi, Palette};
use super::{approval, history_summary};
use crate::review::{
    inline_view, DiffLine, Pairing, ParaKind, Review, Severity, SpanMark, OUTDATED_NOTE,
};
use std::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub struct TextOptions {
    pub palette: Palette,
    pub findings_only: bool,
}

fn paint(palette: Palette, style: ratatui::style::Style, text: &str) -> String {
    ansi::paint(palette, style, text)
}

/// wdiff conventions: `[-removed-]` and `{+added+}`. The marks stay in
/// colour too: piped output is read without the codes as often as with.
pub fn render_line(line: &DiffLine, palette: Palette) -> String {
    let body: String = match line.kind {
        ParaKind::Changed => line
            .spans
            .iter()
            .map(|s| match s.mark {
                SpanMark::Equal => s.text.clone(),
                SpanMark::Removed => {
                    paint(palette, palette.removed_span(), &format!("[-{}-]", s.text))
                }
                SpanMark::Added => {
                    paint(palette, palette.added_span(), &format!("{{+{}+}}", s.text))
                }
            })
            .collect(),
        ParaKind::Added => paint(palette, palette.added_span(), &line.text()),
        ParaKind::Removed => paint(palette, palette.removed_span(), &line.text()),
        ParaKind::Separator => paint(palette, palette.muted(), &line.text()),
        ParaKind::Equal => line.text(),
    };
    if line.spans.is_empty() {
        String::new()
    } else {
        format!("{} {}", line.kind.glyph(), body)
    }
}

fn severity_style(s: Severity, palette: Palette) -> ratatui::style::Style {
    match s {
        Severity::Error => palette.error(),
        Severity::Warning => palette.warning(),
        Severity::Note => palette.note(),
    }
}

fn write_pairing(out: &mut String, p: &Pairing, palette: Palette) {
    let mark = approval(p).mark();
    let markers = format!(
        "{}{}",
        super::finding_marker(p),
        super::note_marker(p.has_note())
    );
    let heading = format!("{mark} {} {}", p.kind.glyph(), p.name);
    let _ = writeln!(
        out,
        "{}{}",
        paint(palette, palette.heading(), &heading),
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
        let rendered = render_line(&line, palette);
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
            paint(
                palette,
                severity_style(f.severity, palette),
                &f.severity.to_string()
            ),
            f.message
        );
        for d in &f.details {
            let _ = writeln!(out, "        {d}");
        }
    }
    for note in &p.notes {
        let head = format!("✎ note{}:", note.anchor_suffix());
        for (i, l) in note.text.lines().enumerate() {
            let _ = match i {
                0 => writeln!(out, "    {head} {l}"),
                _ => writeln!(out, "    {:width$} {l}", "", width = head.chars().count()),
            };
        }
        if note.outdated {
            let _ = writeln!(out, "        {OUTDATED_NOTE}");
        }
    }
    let _ = writeln!(out, "    {}", history_summary(p));
    out.push('\n');
}

pub fn render(review: &Review, options: TextOptions) -> String {
    let palette = options.palette;
    let mut out = String::new();
    if options.findings_only {
        return render_findings_only(review);
    }
    for change in &review.changes {
        let _ = writeln!(
            out,
            "{}  ({})",
            paint(
                palette,
                palette.heading(),
                &format!("# change {}", change.name)
            ),
            review.origin
        );
        out.push('\n');
        if !change.artefacts.is_empty() {
            let _ = writeln!(out, "{}", paint(palette, palette.heading(), "## artefacts"));
            for a in &change.artefacts {
                let status = a.state.status(a.text_hash());
                let _ = writeln!(out, "{} {}", status.mark(), a.artefact.name);
                for line in a.lines() {
                    let _ = writeln!(out, "    {}", render_line(&line, palette));
                }
                if let Some(note) = &a.state.note {
                    let _ = writeln!(out, "    ✎ note: {}", note.text);
                    if a.note_outdated() {
                        let _ = writeln!(out, "        {OUTDATED_NOTE}");
                    }
                }
                out.push('\n');
            }
        }
        for cap in &change.capabilities {
            let _ = writeln!(
                out,
                "{}",
                paint(palette, palette.heading(), &format!("## {}", cap.name))
            );
            out.push('\n');
            for p in &cap.pairings {
                write_pairing(&mut out, p, palette);
            }
        }
    }
    if !review.canon_edits.is_empty() {
        let _ = writeln!(out, "{}", paint(palette, palette.heading(), "## canon"));
        for edit in &review.canon_edits {
            let _ = writeln!(out, "### {}", edit.path);
            for line in &edit.lines {
                let _ = writeln!(out, "    {}", render_line(line, palette));
            }
            out.push('\n');
        }
    }
    write_notices(&mut out, review, palette);
    let _ = writeln!(
        out,
        "summary: {}; {}",
        review.summary,
        review.glossary_summary()
    );
    out
}

/// Findings about the configuration: the same lines the lint prints.
fn write_notices(out: &mut String, review: &Review, palette: Palette) {
    for n in &review.notices {
        let _ = writeln!(
            out,
            "{}: {}: {}",
            paint(
                palette,
                severity_style(n.severity, palette),
                &n.severity.to_string()
            ),
            n.file,
            n.message
        );
    }
}

/// One line per finding: `severity  change/capability/requirement: message`.
pub fn render_findings_only(review: &Review) -> String {
    let mut out = String::new();
    for p in review.pairings() {
        for f in &p.findings {
            let _ = writeln!(out, "{:<7}  {}: {}", f.severity, f.location, f.message);
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
