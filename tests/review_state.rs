//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::{attach_state, build_review};
use openspec_reviewer::render::tui::App;
use openspec_reviewer::render::{markdown, text};
use openspec_reviewer::state::{item_key, split_key, state_path, ApprovalStatus, ItemState, Store};
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

/// `j` past the open requirement's scenarios onto the next requirement.
fn next_requirement(app: &mut App) {
    let from = app.current_pairing().map(|p| p.name.clone());
    while app.current_pairing().map(|p| p.name.clone()) == from {
        app.handle_key(key('j'));
    }
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
fn a_requirement_or_a_scenario_can_carry_a_note() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    app.set_current_note(Some("Looks fine, but check the orphan rule.".into()));
    let p = app.current_pairing().unwrap();
    assert_eq!(
        p.note().map(|n| n.text.as_str()),
        Some("Looks fine, but check the orphan rule.")
    );
    assert_eq!(p.key(), "alpha/Index rows are ordered by path");
    assert_eq!(openspec_reviewer::render::note_marker(p.has_note()), "✎");
    let plain = text::render(
        &app.review,
        text::TextOptions {
            palette: openspec_reviewer::render::colour::Palette::none(),
            findings_only: false,
        },
    );
    assert!(plain.contains("✎ note: Looks fine"));
}

#[test]
fn a_requirement_or_a_scenario_can_carry_a_note__empty_note() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    app.set_current_note(Some("draft".into()));
    app.set_current_note(None);
    assert!(!app.current_pairing().unwrap().has_note());
    assert_eq!(openspec_reviewer::render::note_marker(false), "");
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
        next_requirement(&mut app);
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
            palette: openspec_reviewer::render::colour::Palette::none(),
            findings_only: false,
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
    next_requirement(&mut app);
    app.set_current_note(Some("second note\nwith two lines".into()));

    let json: serde_json::Value =
        serde_json::from_str(&openspec_reviewer::render::json::render(&app.review)).unwrap();
    let pairings = &json["changes"][0]["capabilities"][0]["pairings"];
    assert!(pairings[0]["state"]["approved"]["text_hash"].is_u64());
    assert_eq!(pairings[0]["notes"][0]["text"], "first note");
    assert_eq!(pairings[0]["notes"][0]["outdated"], false);
    assert!(pairings[0]["notes"][0].get("scenario").is_none());

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
    store.set_note("alpha/One", Some("hi".into()), 42).unwrap();
    store
        .set_note("alpha/One#A scenario", Some("there".into()), 7)
        .unwrap();
    let again = Store::open(path).unwrap();
    let item: ItemState = again.get("alpha/One");
    assert_eq!(item.approved.unwrap().text_hash, 42);
    let note = item.note.unwrap();
    assert_eq!(note.text, "hi");
    assert_eq!(note.text_hash, Some(42));
    assert!(note.at.is_some(), "a note records when it was written");
    assert_eq!(
        again.get("alpha/One#A scenario").note.map(|n| n.text),
        Some("there".to_string()),
        "a scenario's note is a key of its own"
    );
    assert_eq!(
        item_key("alpha", "One", Some("A scenario")),
        "alpha/One#A scenario"
    );
    assert_eq!(
        split_key("alpha/One#A scenario"),
        ("alpha/One", Some("A scenario"))
    );
    assert_eq!(split_key("alpha/One"), ("alpha/One", None));
    assert!(!again.state.items.contains_key("missing"));
    let _ = BTreeMap::<String, Store>::new();
}

/// A requirement with three scenarios, so notes on siblings can be told
/// apart.
const MANY_SCENARIOS: &str = "\
## MODIFIED Requirements

### Requirement: Flat index of all routes and pages

The dashboard MUST offer an index view listing every route and every
page of the active website, plus inherited paths.

#### Scenario: Route-mounted page appears as a row

- **WHEN** a route mounts a page
- **THEN** the index shows one row

#### Scenario: Feature mount appears

- **WHEN** a route has a feature mount
- **THEN** the row shows the feature name

#### Scenario: Orphan page appears without a path

- **WHEN** a page is not mounted by any route
- **THEN** the page appears as a row without a path
";

fn many_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", MANY_SCENARIOS);
    repo
}

/// The whole view as text, for the rules about what the panes say.
fn screen_of(app: &mut App) -> String {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(160, 40)).unwrap();
    terminal
        .draw(|f| openspec_reviewer::render::tui::draw(f, app))
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Put the cursor on the row of the named scenario, unfolding the
/// requirement that holds it.
fn select_scenario(app: &mut App, name: &str) {
    let row_of = |app: &App| {
        app.rows
            .iter()
            .position(|r| app.scenario_at(r).is_some_and(|(_, m)| m.name() == name))
    };
    if row_of(app).is_none() {
        app.cursor = app
            .rows
            .iter()
            .position(|r| {
                app.pairing_at(r)
                    .is_some_and(|p| p.diff.scenarios.iter().any(|m| m.name() == name))
            })
            .expect("a requirement holds that scenario");
        app.handle_key(key(' '));
    }
    app.cursor = row_of(app).expect("the scenario has a row");
}

/// Put the cursor on the named requirement's row.
fn select_requirement(app: &mut App, name: &str) {
    app.cursor = app
        .rows
        .iter()
        .position(|r| app.pairing_at(r).is_some_and(|p| p.name == name))
        .expect("that requirement has a row");
}

#[test]
fn a_requirement_or_a_scenario_can_carry_a_note__write_a_note_on_a_scenario() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    select_scenario(&mut app, "Paths sort alphabetically");
    app.set_current_note(Some("This THEN is the one I object to.".into()));
    let p = app.current_pairing().unwrap();
    assert_eq!(
        p.scenario_note("Paths sort alphabetically")
            .map(|n| n.text.as_str()),
        Some("This THEN is the one I object to.")
    );
    assert!(p.has_note(), "the requirement above it shows `✎` too");
    assert_eq!(
        p.scenario_key("Paths sort alphabetically"),
        "alpha/Index rows are ordered by path#Paths sort alphabetically"
    );

    let reopened = app_for(&repo, "foo");
    let p = reopened
        .review
        .pairings()
        .find(|p| p.name == "Index rows are ordered by path")
        .unwrap();
    assert_eq!(
        p.scenario_note("Paths sort alphabetically")
            .map(|n| n.text.as_str()),
        Some("This THEN is the one I object to."),
        "the note is stored under that scenario's key"
    );
}

#[test]
fn a_requirement_or_a_scenario_can_carry_a_note__two_scenarios_of_one_requirement() {
    let repo = many_repo();
    let mut app = app_for(&repo, "foo");
    select_scenario(&mut app, "Route-mounted page appears as a row");
    app.set_current_note(Some("about the first".into()));
    select_scenario(&mut app, "Feature mount appears");
    app.set_current_note(Some("about the second".into()));
    let p = app.current_pairing().unwrap();
    assert_eq!(
        p.scenario_note("Route-mounted page appears as a row")
            .map(|n| n.text.as_str()),
        Some("about the first"),
        "neither note replaces the other"
    );
    assert_eq!(
        p.scenario_note("Feature mount appears")
            .map(|n| n.text.as_str()),
        Some("about the second")
    );
    assert_eq!(p.notes.len(), 2, "both notes are kept");
}

#[test]
fn a_note_records_the_text_it_was_written_about() {
    let repo = state_repo();
    let mut app = app_for(&repo, "foo");
    app.set_current_note(Some("on the requirement".into()));
    let p = app.current_pairing().unwrap();
    let note = p.note().unwrap();
    assert!(!note.outdated, "the words have not moved yet");
    assert!(note.at.is_some(), "and it records when it was written");

    let store = app.stores.get("foo").unwrap();
    let stored = store.get(&p.key()).note.unwrap();
    assert_eq!(
        stored.text_hash,
        Some(p.anchor_hash(None)),
        "a note hashes its own anchor"
    );
    assert_ne!(
        p.anchor_hash(Some("Paths sort alphabetically")),
        p.anchor_hash(None),
        "and a scenario's anchor is not the whole requirement's"
    );
}

#[test]
fn a_note_records_the_text_it_was_written_about__scenario_reworded_after_a_note() {
    let repo = many_repo();
    {
        let mut app = app_for(&repo, "foo");
        select_scenario(&mut app, "Feature mount appears");
        app.set_current_note(Some("the feature name is not the mount".into()));
        select_scenario(&mut app, "Route-mounted page appears as a row");
        app.set_current_note(Some("this one is fine".into()));
    }
    repo.delta(
        "foo",
        "alpha",
        &MANY_SCENARIOS.replace(
            "- **THEN** the row shows the feature name",
            "- **THEN** the row shows the mounted feature's name",
        ),
    );
    let app = app_for(&repo, "foo");
    let p = app
        .review
        .pairings()
        .find(|p| p.name == "Flat index of all routes and pages")
        .unwrap();
    assert!(
        p.scenario_note("Feature mount appears").unwrap().outdated,
        "the note's anchor moved"
    );
    assert!(
        !p.scenario_note("Route-mounted page appears as a row")
            .unwrap()
            .outdated,
        "its sibling's note did not"
    );

    let mut app = app;
    select_scenario(&mut app, "Feature mount appears");
    let screen = screen_of(&mut app);
    assert!(
        screen.contains("text changed since the note was"),
        "the detail pane says so: {screen}"
    );
}

#[test]
fn a_note_records_the_text_it_was_written_about__note_from_an_older_state_file() {
    let repo = state_repo();
    let path = state_path(
        &repo.state_home(),
        &openspec_reviewer::source::repo_key(repo.root()),
        "foo",
    );
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        r#"{"items":{"alpha/Index rows are ordered by path":{"note":"written before notes had a hash"}}}"#,
    )
    .unwrap();
    let store = Store::open(path).unwrap();
    let note = store
        .get("alpha/Index rows are ordered by path")
        .note
        .unwrap();
    assert_eq!(note.text, "written before notes had a hash");
    assert_eq!(note.text_hash, None);
    assert!(
        !note.is_outdated(12345),
        "a note with no hash never reads as outdated"
    );

    let app = app_for(&repo, "foo");
    let p = app
        .review
        .pairings()
        .find(|p| p.name == "Index rows are ordered by path")
        .unwrap();
    let note = p.note().expect("the note loads with its text");
    assert_eq!(note.text, "written before notes had a hash");
    assert!(!note.outdated);
}

#[test]
fn plain_output_includes_state__export_a_scenario_s_note() {
    let repo = many_repo();
    let mut app = app_for(&repo, "foo");
    select_requirement(&mut app, "Flat index of all routes and pages");
    app.set_current_note(Some("on the requirement".into()));
    select_scenario(&mut app, "Feature mount appears");
    app.set_current_note(Some("on the scenario".into()));

    let plain = text::render(
        &app.review,
        text::TextOptions {
            palette: openspec_reviewer::render::colour::Palette::none(),
            findings_only: false,
        },
    );
    assert!(plain.contains("✎ note: on the requirement"), "{plain}");
    assert!(
        plain.contains("✎ note # Feature mount appears: on the scenario"),
        "plain names the anchor: {plain}"
    );

    let json: serde_json::Value =
        serde_json::from_str(&openspec_reviewer::render::json::render(&app.review)).unwrap();
    let notes = &json["changes"][0]["capabilities"][0]["pairings"][0]["notes"];
    assert_eq!(notes[1]["scenario"], "Feature mount appears");
    assert_eq!(notes[1]["outdated"], false);

    let md = markdown::render(&app.review);
    assert!(
        md.contains("- **Flat index of all routes and pages**: on the requirement\n  - **Feature mount appears**: on the scenario"),
        "the scenario's note nests under its requirement and names it: {md}"
    );
}

#[test]
fn plain_output_includes_state__an_outdated_note_is_marked() {
    let repo = many_repo();
    {
        let mut app = app_for(&repo, "foo");
        select_scenario(&mut app, "Feature mount appears");
        app.set_current_note(Some("about the old wording".into()));
    }
    repo.delta(
        "foo",
        "alpha",
        &MANY_SCENARIOS.replace(
            "- **THEN** the row shows the feature name",
            "- **THEN** the row shows the mounted feature's name",
        ),
    );
    let app = app_for(&repo, "foo");
    let plain = text::render(
        &app.review,
        text::TextOptions {
            palette: openspec_reviewer::render::colour::Palette::none(),
            findings_only: false,
        },
    );
    assert!(
        plain.contains("text changed since the note was written"),
        "{plain}"
    );
    let md = markdown::render(&app.review);
    assert!(
        md.contains("- **Feature mount appears**: about the old wording _(outdated)_"),
        "{md}"
    );
    let json: serde_json::Value =
        serde_json::from_str(&openspec_reviewer::render::json::render(&app.review)).unwrap();
    let notes = &json["changes"][0]["capabilities"][0]["pairings"][0]["notes"];
    assert_eq!(notes[0]["outdated"], true);
}
