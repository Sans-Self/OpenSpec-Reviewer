//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::{attach_state, build_review};
use openspec_reviewer::render::tui::App;
use openspec_reviewer::render::{markdown, text};
use openspec_reviewer::state::{state_path, ApprovalStatus, ItemState, Store};
use std::collections::BTreeMap;

const DELTA: &str = "\
## MODIFIED Requirements

### Requirement: Index rows are ordered by path

Rows MUST be ordered alphabetically by path, orphans last.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order

## ADDED Requirements

### Requirement: Rows can be filtered

The index MUST offer a filter box.

#### Scenario: Filter by title

- **WHEN** the reviewer types a title
- **THEN** only matching rows remain
";

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn app_for(repo: &Repo, change: &str) -> App {
    let snap = openspec_reviewer::source::ChangeSource {
        root: repo.root().into(),
        name: change.into(),
    }
    .fetch_snapshot();
    let review = build_review(repo.root(), &snap).unwrap();
    let built = attach_state(repo.root(), review, Some(&repo.state_home())).unwrap();
    App::new(built.review, built.stores)
}

trait FetchSnapshot {
    fn fetch_snapshot(&self) -> openspec_reviewer::source::Snapshot;
}
impl FetchSnapshot for openspec_reviewer::source::ChangeSource {
    fn fetch_snapshot(&self) -> openspec_reviewer::source::Snapshot {
        openspec_reviewer::source::Source::fetch(self).unwrap()
    }
}

fn state_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA);
    repo
}

#[test]
fn a_requirement_can_be_approved() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    let before = app.approval_counts();
    app.handle_key(key('a'));
    let p = app.current_pairing().unwrap();
    assert_eq!(
        openspec_reviewer::render::approval(p),
        ApprovalStatus::Approved
    );
    assert_eq!(app.approval_counts().0, before.0 + 1);
    app.handle_key(key('a'));
    assert_eq!(
        openspec_reviewer::render::approval(app.current_pairing().unwrap()),
        ApprovalStatus::Pending
    );
    assert_eq!(app.approval_counts().0, before.0);
}

#[test]
fn a_requirement_can_carry_a_note() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    assert_eq!(
        app.handle_key(key('e')),
        Some(openspec_reviewer::render::tui::Effect::EditNote)
    );
    app.set_current_note(Some("Looks fine, but check the orphan rule.".into()));
    let p = app.current_pairing().unwrap();
    assert_eq!(
        p.state.note.as_deref(),
        Some("Looks fine, but check the orphan rule.")
    );
    assert_eq!(
        openspec_reviewer::render::note_marker(p.state.note.is_some()),
        "✎"
    );
    let plain = text::render(
        &app.review,
        text::TextOptions {
            colour: false,
            findings_only: false,
            hints: false,
        },
    );
    assert!(plain.contains("✎ note: Looks fine"));
}

#[test]
fn a_requirement_can_carry_a_note__empty_note() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    app.set_current_note(Some("draft".into()));
    app.set_current_note(None);
    assert!(app.current_pairing().unwrap().state.note.is_none());
    let keep =
        openspec_reviewer::state::edit_note_with(&["true".to_string()], Some("old text")).unwrap();
    assert_eq!(
        keep,
        Some("old text".into()),
        "an untouched file keeps the note"
    );
    let truncate = vec!["sh".to_string(), "-c".to_string(), ": > \"$0\"".to_string()];
    let cleared = openspec_reviewer::state::edit_note_with(&truncate, Some("old text")).unwrap();
    assert_eq!(cleared, None, "an emptied file removes the note");
}

#[test]
fn state_persists_between_runs() {
    let repo = state_repo();
    {
        let mut app = app_for(&repo, "foo");
        app.handle_key(key('a'));
        app.handle_key(key('j'));
        app.handle_key(key('a'));
    }
    let app = app_for(&repo, "foo");
    assert_eq!(app.approval_counts(), (2, 2));
    let path = state_path(
        &repo.state_home(),
        &openspec_reviewer::source::repo_key(repo.root()),
        "foo",
    );
    assert!(
        path.starts_with(repo.root().join(".state")),
        "{}",
        path.display()
    );
    assert!(path.ends_with("foo.json"));
    assert!(path.exists());
}

#[test]
fn state_persists_between_runs__different_repository_same_change_name() {
    let a = state_repo();
    let b = Repo::new();
    b.canon("alpha", ALPHA_CANON).delta("foo", "alpha", DELTA);
    let mut app_a = app_for(&a, "foo");
    app_a.handle_key(key('a'));
    let app_b = app_for(&b, "foo");
    assert_eq!(app_b.approval_counts().0, 0);
}

#[test]
fn state_persists_between_runs__no_state_flag() {
    let repo = state_repo();
    let snap = openspec_reviewer::source::ChangeSource {
        root: repo.root().into(),
        name: "foo".into(),
    }
    .fetch_snapshot();
    let review = build_review(repo.root(), &snap).unwrap();
    let built = attach_state(repo.root(), review, None).unwrap();
    assert!(built.stores.values().all(|s| s.path.is_none()));
}

#[test]
fn an_approval_is_tied_to_the_text_it_approved() {
    let repo = state_repo();
    {
        let mut app = app_for(&repo, "foo");
        app.handle_key(key('a'));
    }
    repo.delta(
        "foo",
        "alpha",
        &DELTA.replace("orphans last", "orphan pages last"),
    );
    let mut app = app_for(&repo, "foo");
    let p = app.current_pairing().unwrap();
    assert_eq!(
        openspec_reviewer::render::approval(p),
        ApprovalStatus::Stale
    );
    assert_eq!(ApprovalStatus::Stale.mark(), "[~]");
    assert_eq!(app.approval_counts().0, 0, "stale counts as not approved");
    let plain = text::render(
        &app.review,
        text::TextOptions {
            colour: false,
            findings_only: false,
            hints: false,
        },
    );
    assert!(plain.contains("text changed since approval"));
    app.handle_key(key('a'));
    assert_eq!(
        openspec_reviewer::render::approval(app.current_pairing().unwrap()),
        ApprovalStatus::Approved
    );
}

#[test]
fn plain_output_includes_state() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    app.handle_key(key('a'));
    app.set_current_note(Some("first note".into()));
    app.handle_key(key('j'));
    app.set_current_note(Some("second note\nwith two lines".into()));

    let json: serde_json::Value =
        serde_json::from_str(&openspec_reviewer::render::json::render(&app.review)).unwrap();
    let pairings = &json["changes"][0]["capabilities"][0]["pairings"];
    assert!(pairings[0]["state"]["approved"]["text_hash"].is_u64());
    assert_eq!(pairings[0]["state"]["note"], "first note");

    let md = markdown::render(&app.review);
    assert!(md.contains("## alpha"));
    assert!(md.contains("- **Index rows are ordered by path**: first note"));
    assert!(md.contains("- **Rows can be filtered**: second note\n  with two lines"));
    assert!(!md.contains("MUST"), "markdown holds notes only");
}

#[test]
fn store_round_trips_through_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("repo").join("foo.json");
    let mut store = Store::open(path.clone()).unwrap();
    store.toggle_approval("alpha/One", 42).unwrap();
    store.set_note("alpha/One", Some("hi".into())).unwrap();
    let again = Store::open(path).unwrap();
    let item: ItemState = again.get("alpha/One");
    assert_eq!(item.approved.unwrap().text_hash, 42);
    assert_eq!(item.note.as_deref(), Some("hi"));
    assert!(!again.state.items.contains_key("missing"));
    let _ = BTreeMap::<String, Store>::new();
}
