//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::{attach_state, build_review};
use openspec_reviewer::render::tui::{App, Row};
use openspec_reviewer::source::{repo_key, ChangeSource, Source};
use openspec_reviewer::state::{state_path, Store};

const DELTA: &str = "\
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

### Requirement: Index rows are ordered by path

Rows MUST be ordered alphabetically by path, orphans last.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order
";

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ch(c: char) -> KeyEvent {
    key(KeyCode::Char(c))
}

fn panel_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA)
        .write("openspec/changes/foo/design.md", "# design\n\nThe shape.\n");
    repo
}

fn app_for(repo: &Repo) -> App {
    let snap = ChangeSource {
        root: repo.root().into(),
        name: "foo".into(),
    }
    .fetch()
    .unwrap();
    let review = build_review(repo.root(), &snap).unwrap();
    let built = attach_state(repo.root(), review, Some(&repo.state_home())).unwrap();
    App::new(built.review, built.stores)
}

fn stored(repo: &Repo) -> Store {
    Store::open(state_path(
        &repo.state_home(),
        &repo_key(repo.root()),
        "foo",
    ))
    .unwrap()
}

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

/// Put the cursor on the row of the named artefact.
fn select_artefact(app: &mut App, name: &str) {
    app.cursor = app
        .rows
        .iter()
        .position(|r| app.artefact_at(r).is_some_and(|a| a.artefact.name == name))
        .expect("the artefact has a row");
}

fn select_requirement(app: &mut App, name: &str) {
    app.cursor = app
        .rows
        .iter()
        .position(|r| app.pairing_at(r).is_some_and(|p| p.name == name))
        .expect("the requirement has a row");
}

/// Put the cursor on the named scenario, unfolding its requirement.
fn select_scenario(app: &mut App, name: &str) {
    let row_of = |app: &App| {
        app.rows
            .iter()
            .position(|r| app.scenario_at(r).is_some_and(|(_, m)| m.name() == name))
    };
    if row_of(app).is_none() {
        let holder = app
            .rows
            .iter()
            .find_map(|r| {
                app.pairing_at(r)
                    .filter(|p| p.diff.scenarios.iter().any(|m| m.name() == name))
                    .map(|p| p.name.clone())
            })
            .expect("a requirement holds that scenario");
        select_requirement(app, &holder);
        app.handle_key(ch(' '));
    }
    app.cursor = row_of(app).expect("the scenario has a row");
}

/// The three notes the first requirement covers, written on the three
/// kinds of anchor the panel has to list.
fn three_notes(app: &mut App) {
    select_artefact(app, "design.md");
    app.set_current_note(Some("the design is thin here".into()));
    select_requirement(app, "Flat index of all routes and pages");
    app.set_current_note(Some("inherited paths need a definition".into()));
    select_scenario(app, "Feature mount appears");
    app.set_current_note(Some(
        "which feature name?\nthe mount's or the page's".into(),
    ));
}

#[test]
fn the_notes_panel_lists_every_note_of_the_change() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    app.handle_key(ch('N'));
    assert!(app.notes_state().is_some(), "`N` opens the panel");
    let screen = screen_of(&mut app);
    assert!(screen.contains("notes (3)"), "the title counts: {screen}");
    assert!(
        openspec_reviewer::render::tui::status_text(&app).contains("mode: notes"),
        "the mode is named in text"
    );
}

#[test]
fn the_notes_panel_lists_every_note_of_the_change__three_kinds_of_note() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    app.handle_key(ch('N'));
    let rows = app.note_rows();
    assert_eq!(
        rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(),
        vec![
            "design.md",
            "alpha § Flat index of all routes and pages",
            "alpha § Flat index of all routes and pages › Feature mount appears",
        ],
        "the artefact first, then the main list's order"
    );
    assert_eq!(
        rows.iter()
            .map(|r| r.first_line.as_str())
            .collect::<Vec<_>>(),
        vec![
            "the design is thin here",
            "inherited paths need a definition",
            "which feature name?",
        ],
        "each row shows the first line of its note"
    );
    let screen = screen_of(&mut app);
    assert!(
        screen.contains("alpha § Flat index of all routes and pages › Feature mount appears"),
        "{screen}"
    );
}

#[test]
fn the_notes_panel_lists_every_note_of_the_change__outdated_note() {
    let repo = panel_repo();
    {
        let mut app = app_for(&repo);
        select_scenario(&mut app, "Feature mount appears");
        app.set_current_note(Some("which feature name?".into()));
    }
    repo.delta(
        "foo",
        "alpha",
        &DELTA.replace(
            "- **THEN** the row shows the feature name",
            "- **THEN** the row shows the mounted feature's name",
        ),
    );
    let mut app = app_for(&repo);
    app.handle_key(ch('N'));
    assert!(
        app.note_rows()[0].outdated,
        "the words the note was written about moved"
    );
    let screen = screen_of(&mut app);
    assert!(screen.contains("(outdated)"), "{screen}");
}

#[test]
fn the_notes_panel_lists_every_note_of_the_change__no_notes() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    app.handle_key(ch('N'));
    assert!(app.note_rows().is_empty());
    let screen = screen_of(&mut app);
    assert!(screen.contains("notes (0)"), "{screen}");
    assert!(screen.contains("no notes"), "{screen}");
}

#[test]
fn the_notes_panel_lists_every_note_of_the_change__close() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    let cursor = app.cursor;
    app.handle_key(ch('N'));
    app.handle_key(key(KeyCode::Down));
    app.handle_key(key(KeyCode::Esc));
    assert!(app.notes_state().is_none(), "Esc closes the panel");
    assert!(!app.quit, "and does not quit the view");
    assert_eq!(app.cursor, cursor, "the main list is where it was");
}

#[test]
fn enter_jumps_to_the_notes_row() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    select_artefact(&mut app, "design.md");
    app.handle_key(ch('N'));
    app.handle_key(ch('j'));
    app.handle_key(key(KeyCode::Enter));
    assert!(app.notes_state().is_none(), "⏎ closes the panel");
    assert_eq!(
        app.current_pairing().map(|p| p.name.as_str()),
        Some("Flat index of all routes and pages"),
        "the cursor is on the note's requirement"
    );
}

#[test]
fn enter_jumps_to_the_notes_row__scenario_note_under_a_folded_requirement() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    select_scenario(&mut app, "Feature mount appears");
    app.set_current_note(Some("which feature name?".into()));
    // Fold it again, so the row the note hangs on is not in the list.
    select_requirement(&mut app, "Flat index of all routes and pages");
    app.handle_key(ch(' '));
    assert!(
        !app.rows.iter().any(|r| matches!(r, Row::Scenario { .. })),
        "the requirement is folded"
    );

    app.handle_key(ch('N'));
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.rows.iter().any(|r| matches!(r, Row::Scenario { .. })),
        "the requirement unfolds"
    );
    assert_eq!(
        app.scenario_at(app.current_row().unwrap())
            .map(|(_, m)| m.name().to_string()),
        Some("Feature mount appears".to_string()),
        "the cursor is on the scenario row"
    );
    let screen = screen_of(&mut app);
    assert!(
        screen.contains("which feature name?"),
        "the detail pane shows the note under the diff: {screen}"
    );
}

#[test]
fn a_note_can_be_deleted_from_the_panel() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    app.handle_key(ch('N'));
    app.handle_key(ch('j'));
    app.handle_key(ch('d'));
    assert!(app.notes_state().is_some(), "the panel stays open");
    assert_eq!(app.note_rows().len(), 2, "the row is gone");
    assert!(
        !stored(&repo)
            .state
            .items
            .contains_key("alpha/Flat index of all routes and pages"),
        "and so is the item that held it"
    );
}

#[test]
fn a_note_can_be_deleted_from_the_panel__delete_one_of_three() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    select_requirement(&mut app, "Index rows are ordered by path");
    app.handle_key(ch('a'));
    app.handle_key(ch('N'));
    app.handle_key(ch('j'));
    app.handle_key(ch('d'));
    let rows = app.note_rows();
    assert_eq!(
        rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(),
        vec![
            "design.md",
            "alpha § Flat index of all routes and pages › Feature mount appears",
        ]
    );
    assert_eq!(
        app.notes_state().map(|s| s.selected),
        Some(1),
        "the selection is on the note that followed"
    );
    let store = stored(&repo);
    assert!(
        store
            .get("alpha/Flat index of all routes and pages")
            .note
            .is_none(),
        "the state file no longer holds the deleted note"
    );
    assert!(
        store
            .get("alpha/Index rows are ordered by path")
            .approved
            .is_some(),
        "the approval it did not touch is still there"
    );

    app.handle_key(ch('d'));
    assert_eq!(
        app.notes_state().map(|s| s.selected),
        Some(0),
        "deleting the last row selects the one before it"
    );
}

#[test]
fn every_note_can_be_cleared_after_confirmation__confirm() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    select_requirement(&mut app, "Index rows are ordered by path");
    app.handle_key(ch('a'));
    select_artefact(&mut app, "design.md");
    app.handle_key(ch('a'));
    app.handle_key(ch('N'));
    app.handle_key(ch('X'));
    let screen = screen_of(&mut app);
    assert!(
        screen.contains("delete all 3 notes"),
        "the confirmation names the count: {screen}"
    );
    app.handle_key(ch('y'));
    assert!(app.notes_state().is_some(), "the panel stays open");
    assert!(app.note_rows().is_empty());
    let screen = screen_of(&mut app);
    assert!(screen.contains("no notes"), "{screen}");

    let store = stored(&repo);
    assert!(
        store.state.items.values().all(|i| i.note.is_none()),
        "the state file holds no note"
    );
    assert_eq!(
        store
            .state
            .items
            .values()
            .filter(|i| i.approved.is_some())
            .count(),
        2,
        "and still holds both approvals"
    );
}

#[test]
fn every_note_can_be_cleared_after_confirmation__decline() {
    let repo = panel_repo();
    let mut app = app_for(&repo);
    three_notes(&mut app);
    let before = stored(&repo).state;
    app.handle_key(ch('N'));
    app.handle_key(ch('X'));
    app.handle_key(ch('n'));
    assert!(
        app.notes_state().is_some_and(|s| !s.confirming),
        "the confirmation closes"
    );
    assert_eq!(app.note_rows().len(), 3, "the panel lists the same notes");
    assert_eq!(stored(&repo).state, before, "the state file is unchanged");
}
