//! Notes only, grouped by capability, shaped for a pull request review.

use crate::review::Review;
use std::fmt::Write;

fn indent_continuation(note: &str) -> String {
    note.lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 0 {
                l.to_string()
            } else {
                format!("  {l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render(review: &Review) -> String {
    let mut out = String::new();
    for change in &review.changes {
        let noted_artefacts: Vec<_> = change
            .artefacts
            .iter()
            .filter_map(|a| a.state.note.as_ref().map(|n| (&a.artefact.name, n)))
            .collect();
        if !noted_artefacts.is_empty() {
            let _ = writeln!(out, "## artefacts\n");
            for (name, note) in noted_artefacts {
                let _ = writeln!(out, "- **{name}**: {}", indent_continuation(note));
            }
            out.push('\n');
        }
        for cap in &change.capabilities {
            let noted: Vec<_> = cap
                .pairings
                .iter()
                .filter_map(|p| p.state.note.as_ref().map(|n| (&p.name, n)))
                .collect();
            if noted.is_empty() {
                continue;
            }
            let _ = writeln!(out, "## {}\n", cap.name);
            for (name, note) in noted {
                let _ = writeln!(out, "- **{name}**: {}", indent_continuation(note));
            }
            out.push('\n');
        }
    }
    if out.is_empty() {
        out.push_str("No notes.\n");
    }
    out
}
