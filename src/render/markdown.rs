//! Notes only, grouped by capability, shaped for a pull request review.

use crate::review::{AnchoredNote, Pairing, Review};
use std::fmt::Write;

fn indent_continuation(note: &str, depth: usize) -> String {
    let pad = "  ".repeat(depth);
    note.lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 0 {
                l.to_string()
            } else {
                format!("{pad}{l}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ` _(outdated)_` when the words the note is about have moved since.
fn outdated_mark(note: &AnchoredNote) -> &'static str {
    if note.outdated {
        " _(outdated)_"
    } else {
        ""
    }
}

/// One requirement's notes: its own on the bullet, each scenario's nested
/// under it. A requirement whose only note is on a scenario still gets a
/// bullet, because that is what the scenario's note hangs from.
fn write_pairing(out: &mut String, p: &Pairing) {
    let own = p.note();
    match own {
        Some(n) => {
            let _ = writeln!(
                out,
                "- **{}**: {}{}",
                p.name,
                indent_continuation(&n.text, 1),
                outdated_mark(n)
            );
        }
        None => {
            let _ = writeln!(out, "- **{}**", p.name);
        }
    }
    for n in p.notes.iter().filter(|n| n.scenario.is_some()) {
        let scenario = n.scenario.as_deref().unwrap_or_default();
        let _ = writeln!(
            out,
            "  - **{scenario}**: {}{}",
            indent_continuation(&n.text, 2),
            outdated_mark(n)
        );
    }
}

pub fn render(review: &Review) -> String {
    let mut out = String::new();
    for change in &review.changes {
        let noted_artefacts: Vec<_> = change
            .artefacts
            .iter()
            .filter_map(|a| a.state.note.as_ref().map(|n| (&a.artefact.name, n, a)))
            .collect();
        if !noted_artefacts.is_empty() {
            let _ = writeln!(out, "## artefacts\n");
            for (name, note, a) in noted_artefacts {
                let _ = writeln!(
                    out,
                    "- **{name}**: {}{}",
                    indent_continuation(&note.text, 1),
                    if a.note_outdated() {
                        " _(outdated)_"
                    } else {
                        ""
                    }
                );
            }
            out.push('\n');
        }
        for cap in &change.capabilities {
            let noted: Vec<&crate::review::Pairing> =
                cap.pairings.iter().filter(|p| p.has_note()).collect();
            if noted.is_empty() {
                continue;
            }
            let _ = writeln!(out, "## {}\n", cap.name);
            for p in noted {
                write_pairing(&mut out, p);
            }
            out.push('\n');
        }
    }
    if out.is_empty() {
        out.push_str("No notes.\n");
    }
    out
}
