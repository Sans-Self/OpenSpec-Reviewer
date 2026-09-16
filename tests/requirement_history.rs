//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::build_review;
use openspec_reviewer::model::DeltaKind;
use openspec_reviewer::render::tui::{App, View};
use openspec_reviewer::render::{json, text};
use openspec_reviewer::review::{collect_history, FindingKind, Location};
use openspec_reviewer::source::{load_archives, ChangeSource, Source};
use std::collections::BTreeMap;

const CURRENT: &str = "\
## MODIFIED Requirements

### Requirement: Index rows are ordered by path

Rows MUST be ordered alphabetically by path, orphans last.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order
";

fn modified(body: &str) -> String {
    format!("## MODIFIED Requirements\n\n### Requirement: Index rows are ordered by path\n\n{body}\n\n#### Scenario: Paths sort alphabetically\n\n- **WHEN** x\n- **THEN** y\n")
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn location() -> Location {
    Location {
        change: "c".into(),
        capability: "alpha".into(),
        requirement: "Index rows are ordered by path".into(),
        scenario: None,
    }
}

fn history_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", CURRENT)
        .archive("2026-07-01-first", "alpha", &modified("First wording."))
        .archive("2026-08-01-second", "alpha", &modified("Second wording."));
    repo
}

fn app_for(repo: &Repo) -> App {
    let snap = ChangeSource {
        root: repo.root().into(),
        name: "foo".into(),
    }
    .fetch()
    .unwrap();
    App::new(build_review(repo.root(), &snap).unwrap(), BTreeMap::new())
}

#[test]
fn history_comes_from_archived_changes() {
    let repo = history_repo();
    let archives = load_archives(repo.root());
    let (entries, findings) = collect_history(
        &archives,
        "alpha",
        "Index rows are ordered by path",
        &location(),
    );
    assert!(findings.is_empty());
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].change, "2026-07-01-first");
    assert_eq!(entries[0].kind, DeltaKind::Modified);
    assert_eq!(entries[0].text.body, "First wording.");
    assert_eq!(entries[1].change, "2026-08-01-second");
}

#[test]
fn history_comes_from_archived_changes__never_touched() {
    let repo = history_repo();
    repo.delta("foo", "alpha", "## MODIFIED Requirements\n\n### Requirement: Flat index of all routes and pages\n\nNew body.\n\n#### Scenario: S\n\n- a\n");
    let mut app = app_for(&repo);
    assert!(app.current_pairing().unwrap().history.is_empty());
    app.handle_key(key(KeyCode::Char('H')));
    assert!(matches!(app.view, View::History(_)));
    let lines = app.history_lines();
    assert!(
        lines.iter().any(|l| l.text().contains("New body.")),
        "the only version is the one under review"
    );
    let plain = text::render(
        &app.review,
        text::TextOptions {
            colour: false,
            findings_only: false,
            hints: false,
        },
    );
    assert!(plain.contains("history: none"));
}

#[test]
fn history_follows_renames_backwards() {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", CURRENT)
        .archive("2026-06-01-added", "alpha", "## ADDED Requirements\n\n### Requirement: Rows are sorted\n\nOldest body.\n\n#### Scenario: S\n\n- a\n")
        .archive("2026-07-01-renamed", "alpha", "## RENAMED Requirements\n\n- FROM: `### Requirement: Rows are sorted`\n- TO: `### Requirement: Index rows are ordered by path`\n")
        .archive("2026-08-01-modified", "alpha", &modified("Later body."));
    let archives = load_archives(repo.root());
    let (entries, _) = collect_history(
        &archives,
        "alpha",
        "Index rows are ordered by path",
        &location(),
    );
    let kinds: Vec<&DeltaKind> = entries.iter().map(|e| &e.kind).collect();
    assert_eq!(entries.len(), 3);
    assert_eq!(kinds[0], &DeltaKind::Added);
    assert_eq!(entries[0].name, "Rows are sorted");
    assert_eq!(
        kinds[1],
        &DeltaKind::Renamed {
            from: "Rows are sorted".into()
        }
    );
    assert!(entries[1]
        .label()
        .contains("Rows are sorted → Index rows are ordered by path"));
    assert_eq!(kinds[2], &DeltaKind::Modified);
}

#[test]
fn unparseable_archive_becomes_a_note_finding() {
    let repo = history_repo();
    repo.archive(
        "2026-05-01-broken",
        "alpha",
        "## Changed Requirements\n\n### Requirement: Index rows are ordered by path\n\nx\n",
    );
    let archives = load_archives(repo.root());
    let (entries, findings) = collect_history(
        &archives,
        "alpha",
        "Index rows are ordered by path",
        &location(),
    );
    assert_eq!(entries.len(), 2, "the readable archives still count");
    assert_eq!(findings.len(), 1);
    assert!(
        matches!(&findings[0].kind, FindingKind::HistoryUnreadable { archive, .. } if archive == "2026-05-01-broken")
    );
    assert_eq!(
        findings[0].severity,
        openspec_reviewer::review::Severity::Note
    );
}

#[test]
fn the_history_view_is_a_list_and_a_version() {
    let repo = history_repo();
    let mut app = app_for(&repo);
    let cursor = app.cursor;
    app.handle_key(key(KeyCode::Char('H')));
    let View::History(h) = &app.view else {
        panic!("history view opens")
    };
    assert_eq!(h.selected, 2, "newest entry (current) selected");
    assert!(app
        .history_lines()
        .iter()
        .any(|l| l.text().contains("orphans last")));

    app.handle_key(key(KeyCode::Char('k')));
    app.handle_key(key(KeyCode::Char('m')));
    let lines = app.history_lines();
    let changed = lines
        .iter()
        .find(|l| l.kind == openspec_reviewer::review::ParaKind::Changed)
        .expect("word diff to previous");
    assert!(changed.spans.iter().any(
        |s| s.mark == openspec_reviewer::review::SpanMark::Removed && s.text.contains("First")
    ));
    assert!(
        changed
            .spans
            .iter()
            .any(|s| s.mark == openspec_reviewer::review::SpanMark::Added
                && s.text.contains("Second"))
    );

    app.handle_key(key(KeyCode::Esc));
    assert_eq!(app.view, View::Main);
    assert_eq!(app.cursor, cursor);
    assert!(!app.quit);
}

#[test]
fn history_is_available_in_plain_output() {
    let repo = history_repo();
    let app = app_for(&repo);
    let v: serde_json::Value = serde_json::from_str(&json::render(&app.review)).unwrap();
    let history = &v["changes"][0]["capabilities"][0]["pairings"][0]["history"];
    assert_eq!(history.as_array().unwrap().len(), 2);
    assert_eq!(history[0]["change"], "2026-07-01-first");
    assert_eq!(history[0]["kind"], "modified");
    assert_eq!(history[0]["text"]["body"], "First wording.");

    let plain = text::render(
        &app.review,
        text::TextOptions {
            colour: false,
            findings_only: false,
            hints: false,
        },
    );
    assert!(
        plain.contains("history: 2026-07-01-first modified → 2026-08-01-second modified → current"),
        "{plain}"
    );
}
