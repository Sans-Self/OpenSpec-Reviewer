//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::{attach_state, build_review};
use openspec_reviewer::render::tui::{post_notes, App, Focus, Modal, Row};
use openspec_reviewer::review::post::plan;
use openspec_reviewer::source::{ChangeSource, PullRequest, Source};
use openspec_reviewer::state::{state_path, Store};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

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

const REVIEW_URL: &str = "https://github.com/acme/widgets/pull/224#pullrequestreview-77";

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn pr_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA)
        .write("openspec/changes/foo/proposal.md", "# foo\n\nWhy.\n")
        .write("openspec/changes/foo/design.md", "# Design: foo\n")
        .gh(
            "response.json",
            &format!("{{\"html_url\": \"{REVIEW_URL}\"}}\n"),
        );
    repo
}

fn pull_request() -> PullRequest {
    PullRequest {
        number: 224,
        url: "https://github.com/acme/widgets/pull/224".to_string(),
        head: "d34db33f".to_string(),
    }
}

/// The view as `gh 224` leaves it: the change's own files, and the pull
/// request the `gh` source resolved.
fn app_for(repo: &Repo) -> App {
    let mut app = plain_app_for(repo);
    app.review.pull_request = Some(pull_request());
    app
}

/// The same review with no pull request: what every other source yields.
fn plain_app_for(repo: &Repo) -> App {
    let snapshot = ChangeSource {
        root: repo.root().into(),
        name: "foo".into(),
    }
    .fetch()
    .unwrap();
    let review = build_review(repo.root(), &snapshot).unwrap();
    let built = attach_state(repo.root(), review, Some(&repo.state_home())).unwrap();
    App::new(built.review, built.stores)
}

fn requirement_row(app: &App, name: &str) -> Row {
    app.rows
        .iter()
        .find(|row| app.pairing_at(row).is_some_and(|p| p.name == name))
        .cloned()
        .unwrap_or_else(|| panic!("no row for requirement {name}"))
}

fn artefact_row(app: &App, name: &str) -> Row {
    app.rows
        .iter()
        .find(|row| {
            app.artefact_at(row)
                .is_some_and(|a| a.artefact.name == name)
        })
        .cloned()
        .unwrap_or_else(|| panic!("no row for artefact {name}"))
}

/// The scenario row under a requirement, which exists once it is unfolded.
fn scenario_row(app: &mut App, requirement: &str, scenario: &str) -> Row {
    let row = requirement_row(app, requirement);
    app.cursor = app.rows.iter().position(|r| *r == row).unwrap();
    app.handle_key(key(KeyCode::Char(' ')));
    app.rows
        .iter()
        .find(|r| {
            app.scenario_at(r)
                .is_some_and(|(p, m)| p.name == requirement && m.name() == scenario)
        })
        .cloned()
        .unwrap_or_else(|| panic!("no row for scenario {scenario}"))
}

fn note_on(app: &mut App, row: &Row, text: &str) {
    app.set_note_for(row, Some(text.to_string()));
}

fn store_of(repo: &Repo, change: &str) -> Store {
    Store::open(state_path(
        &repo.state_home(),
        &openspec_reviewer::source::repo_key(repo.root()),
        change,
    ))
    .unwrap()
}

fn render(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| openspec_reviewer::render::tui::draw(f, app))
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__unposted_notes_on_a_pull_request() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let first = requirement_row(&app, "Index rows are ordered by path");
    let second = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &first, "the ordering rule needs a tie-break");
    note_on(&mut app, &second, "name the filter's field");

    app.handle_key(key(KeyCode::Char('q')));
    assert!(!app.quit, "the view has not quit");
    let prompt = app.quit_prompt().expect("the prompt is open");
    assert_eq!((prompt.notes, prompt.pull_request), (2, 224));
    let screen = render(&mut app, 100, 24);
    assert!(screen.contains("2 notes are not posted"), "{screen}");
    assert!(screen.contains("pull request 224"), "{screen}");
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__every_quit_key_asks() {
    let repo = pr_repo();
    for quit in [key(KeyCode::Char('q')), key(KeyCode::Esc), ctrl('c')] {
        let mut app = app_for(&repo);
        let row = requirement_row(&app, "Rows can be filtered");
        note_on(&mut app, &row, "a note");
        app.handle_key(quit);
        assert!(!app.quit, "{quit:?} quit instead of asking");
        assert!(matches!(app.focus, Focus::Modal(Modal::Quit(_))));
    }
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__decline() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "a note");
    app.handle_key(key(KeyCode::Char('q')));
    assert_eq!(
        app.handle_key(key(KeyCode::Char('n'))),
        None,
        "nothing sent"
    );
    assert!(app.quit);
    assert_eq!(repo.gh_requests(), 0, "nothing was posted");
    assert!(store_of(&repo, "foo")
        .get("alpha/Rows can be filtered")
        .note
        .is_some_and(|n| n.posted.is_none()));
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__interrupt_declines() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "a note");
    app.handle_key(key(KeyCode::Char('q')));
    app.handle_key(ctrl('c'));
    assert!(app.quit);
    assert_eq!(repo.gh_requests(), 0, "nothing was posted");
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__back_to_the_review() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "a note");
    app.handle_key(key(KeyCode::Char('q')));
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.quit);
    assert_eq!(app.focus, Focus::Browsing);
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__not_a_pull_request() {
    let repo = pr_repo();
    let mut app = plain_app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "a note");
    app.handle_key(key(KeyCode::Char('q')));
    assert!(app.quit, "no pull request, no prompt");
}

#[test]
fn quitting_a_pull_request_review_offers_to_post_the_notes__everything_already_posted() {
    gh_stub();
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "a note");
    app.handle_key(key(KeyCode::Char('q')));
    app.handle_key(key(KeyCode::Char('y')));
    post_notes(repo.root(), &mut app);
    assert!(app.quit);

    let mut app = app_for(&repo);
    app.handle_key(key(KeyCode::Char('q')));
    assert!(app.quit, "every note posted, so the quit key quits");
}

#[test]
fn notes_post_as_one_review_of_comments__requirement_and_scenario_notes() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let requirement = requirement_row(&app, "Index rows are ordered by path");
    note_on(&mut app, &requirement, "say what happens on a tie");
    let scenario = scenario_row(
        &mut app,
        "Index rows are ordered by path",
        "Paths sort alphabetically",
    );
    note_on(&mut app, &scenario, "orphans are not covered here");

    let post = plan(&app.review, &app.stores).expect("something to post");
    assert_eq!(post.comments.len(), 2);
    assert!(post.sections.is_empty(), "{:?}", post.sections);
    let spec = "openspec/changes/foo/specs/alpha/spec.md";
    let text = std::fs::read_to_string(repo.root().join(spec)).unwrap();
    let line_of = |needle: &str| text.lines().position(|l| l == needle).unwrap() + 1;
    assert_eq!(
        (post.comments[0].path.as_str(), post.comments[0].line),
        (
            spec,
            line_of("### Requirement: Index rows are ordered by path")
        )
    );
    assert_eq!(
        (post.comments[1].path.as_str(), post.comments[1].line),
        (spec, line_of("#### Scenario: Paths sort alphabetically"))
    );
    assert_eq!(post.comments[0].body, "say what happens on a tie");

    let payload = post.payload("d34db33f");
    assert_eq!(payload["event"], "COMMENT");
    assert_eq!(payload["commit_id"], "d34db33f");
    assert_eq!(payload["body"], "Notes from openspec-reviewer.");
    assert_eq!(payload["comments"].as_array().unwrap().len(), 2);
}

#[test]
fn notes_post_as_one_review_of_comments__artefact_note() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = artefact_row(&app, "design.md");
    note_on(&mut app, &row, "the trade-off table is missing");

    let post = plan(&app.review, &app.stores).expect("something to post");
    assert_eq!(post.comments.len(), 1);
    assert_eq!(post.comments[0].path, "openspec/changes/foo/design.md");
    assert_eq!(post.comments[0].line, 1);
}

#[test]
fn notes_post_as_one_review_of_comments__anchor_not_found() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");
    // A head whose diff does not carry the delta file: the heading is
    // nowhere to be found, so the note has no line to hang on.
    app.review
        .files
        .retain(|f| !f.path.ends_with("specs/alpha/spec.md"));

    let post = plan(&app.review, &app.stores).expect("something to post");
    assert!(post.comments.is_empty());
    assert_eq!(post.sections.len(), 1);
    assert_eq!(post.sections[0].heading, "alpha § Rows can be filtered");
    assert_eq!(post.sections[0].body, "name the filter's field");
    let payload = post.payload("d34db33f");
    assert_eq!(payload.get("comments"), None);
    assert!(
        payload["body"]
            .as_str()
            .unwrap()
            .contains("### alpha § Rows can be filtered"),
        "{payload}"
    );
}

#[test]
fn notes_post_as_one_review_of_comments__outdated_note() {
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");
    repo.delta(
        "foo",
        "alpha",
        &DELTA.replace(
            "The index MUST offer a filter box.",
            "The index MUST offer a filter box by title.",
        ),
    );

    let app = app_for(&repo);
    let post = plan(&app.review, &app.stores).expect("something to post");
    assert_eq!(
        post.comments[0].body,
        "name the filter's field\n\ntext changed since the note was written"
    );
}

#[test]
fn a_rejected_review_is_retried_in_the_body__line_outside_the_diff() {
    gh_stub();
    let repo = pr_repo();
    repo.gh("reject-first", "");
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");

    post_notes(repo.root(), &mut app);
    assert!(app.quit, "{:?}", app.message);
    assert_eq!(repo.gh_requests(), 2, "one retry");
    let first = repo.gh_request(1);
    assert_eq!(first["comments"].as_array().unwrap().len(), 1);
    let second = repo.gh_request(2);
    assert_eq!(
        second.get("comments"),
        None,
        "no line comments in the retry"
    );
    assert!(
        second["body"]
            .as_str()
            .unwrap()
            .contains("### alpha § Rows can be filtered"),
        "{second}"
    );
}

#[test]
fn a_failed_post_keeps_the_view_open__gh_fails() {
    gh_stub();
    let repo = pr_repo();
    repo.gh("fail", "gh: could not reach github.com\n");
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");

    app.handle_key(key(KeyCode::Char('q')));
    assert_eq!(
        app.handle_key(key(KeyCode::Char('y'))),
        Some(openspec_reviewer::render::tui::Effect::PostNotes)
    );
    post_notes(repo.root(), &mut app);
    assert!(!app.quit, "the view stays open");
    assert!(
        app.message
            .as_deref()
            .unwrap_or_default()
            .contains("could not reach github.com"),
        "{:?}",
        app.message
    );
    assert!(
        store_of(&repo, "foo")
            .get("alpha/Rows can be filtered")
            .note
            .is_some_and(|n| n.posted.is_none()),
        "no note was stamped"
    );
}

#[test]
fn a_failed_post_keeps_the_view_open__posted() {
    gh_stub();
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");

    post_notes(repo.root(), &mut app);
    assert!(app.quit, "{:?}", app.message);
    let request = repo.gh_request(1);
    assert_eq!(request["event"], "COMMENT");
    let args = std::fs::read_to_string(repo.root().join("gh/args")).unwrap();
    assert!(
        args.contains("api repos/acme/widgets/pulls/224/reviews"),
        "{args}"
    );
    let note = store_of(&repo, "foo")
        .get("alpha/Rows can be filtered")
        .note
        .expect("the note is still there");
    assert_eq!(note.posted.expect("posted").url, REVIEW_URL);
}

#[test]
fn a_note_remembers_where_it_was_posted__posted_and_reopened() {
    gh_stub();
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");
    post_notes(repo.root(), &mut app);

    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    app.cursor = app.rows.iter().position(|r| *r == row).unwrap();
    let screen = render(&mut app, 140, 40);
    assert!(screen.contains("posted 20"), "{screen}");
}

#[test]
fn a_note_remembers_where_it_was_posted__edited_after_posting() {
    gh_stub();
    let repo = pr_repo();
    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field");
    post_notes(repo.root(), &mut app);

    let mut app = app_for(&repo);
    let row = requirement_row(&app, "Rows can be filtered");
    note_on(&mut app, &row, "name the filter's field, and its default");
    let note = store_of(&repo, "foo")
        .get("alpha/Rows can be filtered")
        .note
        .expect("the note is still there");
    assert_eq!(note.posted, None, "a new text is an unposted note");
    assert!(
        plan(&app.review, &app.stores).is_some(),
        "it is offered again"
    );
}

#[test]
fn a_note_remembers_where_it_was_posted__older_state_file() {
    let repo = pr_repo();
    let path = state_path(
        &repo.state_home(),
        &openspec_reviewer::source::repo_key(repo.root()),
        "foo",
    );
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        r#"{"items": {
             "alpha/Rows can be filtered": {"note": {"text": "a note", "at": "2025-01-01T00:00:00Z"}},
             "alpha/Index rows are ordered by path": {"note": "an older bare note"}
           }}"#,
    )
    .unwrap();

    let store = Store::open(path).unwrap();
    for key in [
        "alpha/Rows can be filtered",
        "alpha/Index rows are ordered by path",
    ] {
        let note = store.get(key).note.expect("the note loads");
        assert_eq!(note.posted, None, "{key} loads unposted");
    }
    let app = app_for(&repo);
    assert_eq!(
        plan(&app.review, &app.stores).map(|p| p.len()),
        Some(2),
        "both notes are offered"
    );
}
