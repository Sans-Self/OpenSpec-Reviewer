//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::build_review;
use openspec_reviewer::render::colour::Palette;
use openspec_reviewer::render::tui::{App, DetailMode, Pane, Row, View};
use openspec_reviewer::source::{ChangeSource, Source};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::collections::BTreeMap;

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

#### Scenario: Orphan page appears without a path

- **WHEN** a page is not mounted by any route
- **THEN** the page appears as a row without a path

### Requirement: Index rows are ordered by path

Rows MUST be ordered alphabetically by path.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order

### Requirement: Flat index of all routes

Typo in the name: no canon target.

#### Scenario: S

- a
";

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn tui_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA)
        .write("openspec/changes/foo/proposal.md", "# foo\n")
        .write("openspec/changes/foo/tasks.md", "- [ ] 1\n");
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
fn the_interactive_view_opens_when_stdout_is_a_terminal__forced_plain() {
    let repo = tui_repo();
    let out = run_in(repo.root(), &["--plain", "--no-state", "change", "foo"]);
    assert!(stdout(&out).contains("# change foo"));
    assert!(
        out.status.code().is_some(),
        "the tool exits instead of waiting for input"
    );
}

#[test]
fn the_view_is_a_list_and_a_detail_pane() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    let artefacts = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Artefact { .. }))
        .count();
    let capabilities = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Capability { .. }))
        .count();
    let requirements = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Requirement { .. }))
        .count();
    assert_eq!((artefacts, capabilities, requirements), (2, 1, 3));
    assert!(app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Capability { .. }))
        .all(|r| !r.selectable()));

    let screen = render(&mut app, 120, 30);
    assert!(
        screen.contains("[ ] ~ Flat index of all routes and pages"),
        "{screen}"
    );
    assert!(
        screen.contains("[ ] ~ Index rows are ordered by path"),
        "{screen}"
    );
    assert!(screen.contains("proposal.md"));
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__requirement_with_an_error() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    let screen = render(&mut app, 120, 30);
    assert!(screen.contains("~ Flat index of all routes !"), "{screen}");
}

#[test]
fn the_detail_pane_follows_the_list_selection() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    while !matches!(app.current_row(), Some(Row::Requirement { .. })) {
        app.handle_key(key(KeyCode::Char('j')));
    }
    let screen = render(&mut app, 140, 40);
    assert!(screen.contains("{+"), "word diff marks visible: {screen}");
    assert!(screen.contains("inherited"), "{screen}");
    assert!(screen.contains("history: none"));

    let before = app.scroll;
    app.handle_key(key(KeyCode::PageDown));
    assert!(app.scroll > before);
    app.handle_key(ctrl('u'));
    assert_eq!(app.scroll, before);
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.focus, Pane::Detail);
    app.handle_key(key(KeyCode::Char('j')));
    assert_eq!(
        app.scroll,
        before + 1,
        "j scrolls when the detail pane has focus"
    );
}

#[test]
fn the_detail_pane_has_three_display_modes() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    assert_eq!(app.mode, DetailMode::Inline);
    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(app.mode, DetailMode::SideBySide);
    let screen = render(&mut app, 140, 40);
    assert!(screen.contains("mode: side-by-side"), "{screen}");
    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(app.mode, DetailMode::Raw);
    app.handle_key(key(KeyCode::Char('m')));
    assert_eq!(app.mode, DetailMode::Inline);
}

#[test]
fn the_status_line_shows_progress() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    while !matches!(app.current_row(), Some(Row::Requirement { .. })) {
        app.handle_key(key(KeyCode::Char('j')));
    }
    app.handle_key(key(KeyCode::Char('a')));
    app.handle_key(key(KeyCode::Char('j')));
    app.handle_key(key(KeyCode::Char('a')));
    let status = openspec_reviewer::render::tui::status_text(&app);
    assert!(status.contains("2/5 approved"), "{status}");
    assert!(status.contains("foo"));
    assert!(status.contains("1 errors"), "{status}");
    assert!(status.contains("mode: inline"));
}

#[test]
fn keys_follow_vi_and_arrow_conventions() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    let start = app.cursor;
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.cursor, start + 1);
    app.handle_key(key(KeyCode::Char('k')));
    assert_eq!(app.cursor, start);

    app.handle_key(key(KeyCode::Char('n')));
    let p = app.current_pairing().expect("n lands on a requirement");
    assert!(!p.findings.is_empty());
    assert_eq!(
        p.name, "Index rows are ordered by path",
        "restated verbatim: a note is a finding too"
    );
    app.handle_key(key(KeyCode::Char('n')));
    let p = app.current_pairing().unwrap();
    assert_eq!(p.name, "Flat index of all routes");
    let screen = render(&mut app, 140, 40);
    assert!(
        screen.contains("modified without canon"),
        "detail shows the finding: {screen}"
    );
    app.handle_key(key(KeyCode::Char('p')));
    assert!(!app.current_pairing().unwrap().findings.is_empty());

    app.handle_key(key(KeyCode::Char('q')));
    assert!(app.quit);
}

#[test]
fn keys_follow_vi_and_arrow_conventions__help() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    app.handle_key(key(KeyCode::Char('?')));
    assert_eq!(app.view, View::Help);
    let screen = render(&mut app, 120, 30);
    for (k, _) in openspec_reviewer::render::tui::BINDINGS {
        assert!(screen.contains(k), "help lacks {k}: {screen}");
    }
    app.handle_key(key(KeyCode::Char('x')));
    assert_eq!(app.view, View::Main);
    app.handle_key(key(KeyCode::Esc));
    assert!(app.quit, "Esc quits from the main view");
}

#[test]
fn colour_is_never_the_only_signal() {
    let palette = Palette::none();
    assert_eq!(palette.added().fg, None);
    assert_eq!(palette.removed().fg, None);
    assert!(!palette.added().add_modifier.is_empty() || !palette.removed().add_modifier.is_empty());
    let line = openspec_reviewer::review::DiffLine::plain(
        openspec_reviewer::review::ParaKind::Added,
        openspec_reviewer::review::LineRole::Body,
        "text",
    );
    let styled = openspec_reviewer::render::tui::styled_line(&line, palette);
    let rendered: String = styled.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(
        rendered.starts_with("+ "),
        "glyph survives without colour: {rendered}"
    );
}

#[test]
fn colour_is_never_the_only_signal__no_color() {
    let repo = tui_repo();
    let out = std::process::Command::new(exe())
        .args(["--plain", "--no-state", "--color", "change", "foo"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(stdout(&out).contains("\x1b["), "--color opts in");
    let out = std::process::Command::new(exe())
        .args(["--plain", "--no-state", "change", "foo"])
        .env("NO_COLOR", "1")
        .current_dir(repo.root())
        .output()
        .unwrap();
    let text = stdout(&out);
    assert!(!text.contains("\x1b["));
    assert!(text.contains("{+"), "marks still carry glyphs");
}

#[test]
fn the_view_restores_the_terminal_on_exit() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render/tui/mod.rs"
    ))
    .unwrap();
    assert!(
        src.contains("ratatui::try_init()"),
        "init installs the panic hook that restores"
    );
    assert!(src.contains("ratatui::restore()"), "quit path restores");
    assert!(src.contains("SIGTERM"), "signals set the quit flag");
}
