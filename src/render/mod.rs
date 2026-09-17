//! Output. Plain text and JSON for pipes, Markdown for notes, and the TUI.

pub mod colour;
pub mod json;
pub mod lint;
pub mod markdown;
pub mod terminal;
pub mod text;
pub mod tui;

use crate::review::Pairing;
use crate::state::ApprovalStatus;

pub fn approval(p: &Pairing) -> ApprovalStatus {
    p.state.status(p.text_hash())
}

/// `!` for an error, `?` for a warning, nothing otherwise.
pub fn finding_marker(p: &Pairing) -> &'static str {
    match p.worst_severity() {
        Some(crate::review::Severity::Error) => "!",
        Some(crate::review::Severity::Warning) => "?",
        _ => "",
    }
}

pub fn note_marker(has_note: bool) -> &'static str {
    if has_note {
        "✎"
    } else {
        ""
    }
}

/// One line naming the archived changes that touched a requirement.
pub fn history_summary(p: &Pairing) -> String {
    if p.history.is_empty() {
        return "history: none".to_string();
    }
    let names: Vec<String> = p.history.iter().map(|h| h.label()).collect();
    format!("history: {} → current", names.join(" → "))
}
