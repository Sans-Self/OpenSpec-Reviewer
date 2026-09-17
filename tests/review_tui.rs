//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::build_review;
use openspec_reviewer::render::colour::Palette;
use openspec_reviewer::render::tui::{
    styled_line, App, DetailMode, Effect, Focus, Pane, Row, Transient,
};
use openspec_reviewer::review::DiffLine;
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

    let screen = render(&mut app, 160, 30);
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
    let screen = render(&mut app, 160, 30);
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
    assert_eq!(app.pane, Pane::Detail);
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
    next_requirement(&mut app);
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
    assert_eq!(app.focus, Focus::Transient(Transient::Help));
    let screen = render(&mut app, 160, 30);
    for (k, _) in openspec_reviewer::render::tui::BINDINGS {
        assert!(screen.contains(k), "help lacks {k}: {screen}");
    }
    app.handle_key(key(KeyCode::Char('x')));
    assert_eq!(app.focus, Focus::Browsing);
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
    let styled = styled_line(&line, palette, None);
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

/// The same requirement with one scenario gone: a dropped scenario keeps
/// its row, and its finding belongs to that row.
const DROPS_SCENARIO: &str = "\
## MODIFIED Requirements

### Requirement: Flat index of all routes and pages

The dashboard MUST offer an index view listing every route and every
page of the active website.

#### Scenario: Route-mounted page appears as a row

- **WHEN** a route mounts a page
- **THEN** the index shows one row

#### Scenario: Orphan page appears without a path

- **WHEN** a page is not mounted by any route
- **THEN** the page appears as a row without a path
";

fn drop_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DROPS_SCENARIO);
    repo
}

/// `j` past the open requirement's scenarios onto the next requirement.
fn next_requirement(app: &mut App) {
    let from = app.current_pairing().map(|p| p.name.clone());
    while app.current_pairing().map(|p| p.name.clone()) == from {
        app.handle_key(key(KeyCode::Char('j')));
    }
}

/// Move to the first requirement row and pin it open.
fn unfold_first(app: &mut App) {
    while !matches!(app.current_row(), Some(Row::Requirement { .. })) {
        app.handle_key(key(KeyCode::Char('j')));
    }
    app.handle_key(key(KeyCode::Char(' ')));
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__scenarios_folded_on_open() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    assert!(
        !app.rows.iter().any(|r| matches!(r, Row::Scenario { .. })),
        "no scenario row is in the list when the view opens"
    );
    let screen = render(&mut app, 160, 30);
    assert!(
        !screen.contains("Feature mount appears"),
        "a folded requirement hides its scenarios: {screen}"
    );

    hover_first(&mut app);
    let scenarios = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Scenario { .. }))
        .count();
    assert_eq!(scenarios, 3, "the requirement under the cursor is open");
    let screen = render(&mut app, 160, 30);
    assert!(screen.contains("Feature mount appears"), "{screen}");
    app.handle_key(key(KeyCode::Char('k')));
    assert!(
        !app.rows.iter().any(|r| matches!(r, Row::Scenario { .. })),
        "leaving an unpinned requirement folds it"
    );
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__a_removed_scenario_has_a_row() {
    let repo = drop_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    let dropped = app
        .rows
        .iter()
        .find(|r| {
            app.scenario_at(r)
                .is_some_and(|(_, m)| m.name() == "Feature mount appears")
        })
        .cloned()
        .expect("the dropped scenario has a row");
    let (_, m) = app.scenario_at(&dropped).unwrap();
    assert_eq!(m.heading_kind().glyph(), '-', "its glyph is `-`");
    let screen = render(&mut app, 160, 30);
    assert!(screen.contains("- Feature mount appears"), "{screen}");
}

#[test]
fn keys_follow_vi_and_arrow_conventions__jump_into_a_folded_requirement() {
    let repo = drop_repo();
    let mut app = app_for(&repo);
    app.handle_key(key(KeyCode::Char('n')));
    let (_, m) = app
        .current_row()
        .and_then(|r| app.scenario_at(r))
        .expect("n lands on the scenario that carries the finding");
    assert_eq!(m.name(), "Feature mount appears");
    assert!(
        app.rows.iter().any(|r| matches!(r, Row::Scenario { .. })),
        "the requirement it jumped into unfolded"
    );
}

#[test]
fn keys_follow_vi_and_arrow_conventions__approve_from_a_scenario_row() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    app.handle_key(key(KeyCode::Char('j')));
    assert!(matches!(app.current_row(), Some(Row::Scenario { .. })));
    app.handle_key(key(KeyCode::Char('a')));
    let p = app.current_pairing().expect("the scenario's parent");
    assert_eq!(
        openspec_reviewer::render::approval(p),
        openspec_reviewer::state::ApprovalStatus::Approved,
        "`a` on a scenario row toggles its parent requirement"
    );
    let screen = render(&mut app, 160, 30);
    assert!(
        screen.contains("[√] ~ Flat index of all routes and pages"),
        "{screen}"
    );
}

#[test]
fn a_note_is_written_in_a_popup() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    app.handle_key(key(KeyCode::Char('j')));
    app.handle_key(key(KeyCode::Char('e')));
    let edit = app.note_edit().expect("`e` opens the popup");
    assert_eq!(edit.title, "Scenario: Route-mounted page appears as a row");
    assert!(
        edit.quote
            .iter()
            .any(|l| l.contains("a route mounts a page")),
        "the popup quotes the anchor's body: {:?}",
        edit.quote
    );
    let screen = render(&mut app, 160, 30);
    assert!(screen.contains("writing a note"), "{screen}");
    assert!(
        openspec_reviewer::render::tui::status_text(&app).contains("mode: note"),
        "the mode is named in text"
    );
    app.handle_key(key(KeyCode::Char('?')));
    assert!(
        app.note_edit().is_some(),
        "no second overlay opens over the popup"
    );
}

#[test]
fn a_note_is_written_in_a_popup__write_and_save() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    app.handle_key(key(KeyCode::Char('j')));
    app.handle_key(key(KeyCode::Char('e')));
    for c in "no".chars() {
        app.handle_key(key(KeyCode::Char(c)));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(app.note_edit().is_none(), "⏎ closes the popup");
    let p = app.current_pairing().unwrap();
    assert_eq!(
        p.scenario_note("Route-mounted page appears as a row")
            .map(|n| n.text.as_str()),
        Some("no")
    );
    let screen = render(&mut app, 200, 30);
    assert!(
        screen.contains("Route-mounted page appears as a row ✎"),
        "the row shows `✎`: {screen}"
    );
    assert!(
        screen.contains("Flat index of all routes and pages ✎"),
        "and so does the requirement above it: {screen}"
    );

    app.handle_key(key(KeyCode::Char('e')));
    for _ in 0..2 {
        app.handle_key(key(KeyCode::Backspace));
    }
    app.handle_key(key(KeyCode::Enter));
    assert!(
        app.current_pairing()
            .unwrap()
            .scenario_note("Route-mounted page appears as a row")
            .is_none(),
        "an empty buffer removes the note"
    );
}

#[test]
fn a_note_is_written_in_a_popup__cancel() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    app.handle_key(key(KeyCode::Char('j')));
    app.handle_key(key(KeyCode::Char('e')));
    for c in "draft".chars() {
        app.handle_key(key(KeyCode::Char(c)));
    }
    app.handle_key(key(KeyCode::Esc));
    assert!(app.note_edit().is_none(), "the popup closes");
    assert!(!app.quit, "Esc in the popup does not quit the view");
    assert!(
        app.current_pairing()
            .unwrap()
            .scenario_note("Route-mounted page appears as a row")
            .is_none(),
        "the note is unchanged"
    );
}

#[test]
fn a_note_is_written_in_a_popup__escalate_to_the_editor() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    app.handle_key(key(KeyCode::Char('e')));
    for c in "short".chars() {
        app.handle_key(key(KeyCode::Char(c)));
    }
    assert_eq!(
        app.handle_key(ctrl('e')),
        Some(Effect::EscalateNote),
        "^E hands the buffer to the editor"
    );
    assert_eq!(
        app.note_edit().map(|e| e.buffer.as_str()),
        Some("short"),
        "the buffer is what the editor is seeded with"
    );
    let seen = openspec_reviewer::state::edit_note_with(
        &["sh".to_string(), "-c".to_string(), "cat \"$0\"".to_string()],
        Some("short"),
    )
    .unwrap();
    assert_eq!(seen.as_deref(), Some("short"), "$EDITOR opens on that text");
    app.set_note_buffer("what the\neditor saved");
    assert_eq!(
        app.note_edit().map(|e| e.buffer.as_str()),
        Some("what the editor saved"),
        "the popup returns holding it, with no line breaks"
    );
}

#[test]
fn a_note_is_written_in_a_popup__keys_reach_the_popup() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    app.handle_key(key(KeyCode::Char('e')));
    app.handle_key(key(KeyCode::Char('q')));
    assert_eq!(app.note_edit().map(|e| e.buffer.as_str()), Some("q"));
    assert!(!app.quit, "the view does not quit");
    app.handle_key(key(KeyCode::Backspace));
    assert_eq!(app.note_edit().map(|e| e.buffer.as_str()), Some(""));
}

// The shielding fixture already holds what these scenarios need: `group
// key` with `rotation key` deprecated for it, the longer `PLC rotation
// key` that contains the synonym, and `anchor`, deprecated and shielded
// only by `lineage anchor`.
fn marking_glossary() -> openspec_reviewer::glossary::Glossary {
    let mut canon = openspec_reviewer::model::Canon::default();
    canon.specs.insert(
        "definitions".into(),
        openspec_reviewer::model::parse_canon_spec(&fixture("shielding", "definitions.md")),
    );
    openspec_reviewer::glossary::Glossary::build(&canon, &[], "definitions")
}

fn body_line(kind: openspec_reviewer::review::ParaKind, text: &str) -> DiffLine {
    DiffLine::plain(kind, openspec_reviewer::review::LineRole::Body, text)
}

/// The style of the span drawing exactly `text`, which is what marking a
/// term produces: a span of its own, split out of the paragraph.
fn span_style(line: &ratatui::text::Line<'static>, text: &str) -> ratatui::style::Style {
    line.spans
        .iter()
        .find(|s| s.content.as_ref() == text)
        .unwrap_or_else(|| {
            let drawn: Vec<&str> = line.spans.iter().map(|s| s.content.as_ref()).collect();
            panic!("no span drawing {text:?}: {drawn:?}")
        })
        .style
}

fn underlined(style: ratatui::style::Style) -> bool {
    style
        .add_modifier
        .contains(ratatui::style::Modifier::UNDERLINED)
}

#[test]
fn glossary_terms_are_marked_where_they_appear__term_in_an_unchanged_paragraph() {
    let glossary = marking_glossary();
    let marks = glossary.marks();
    let line = body_line(
        openspec_reviewer::review::ParaKind::Equal,
        "The holder signs with the group key before publishing.",
    );
    let styled = styled_line(&line, Palette::from_env(), Some(&marks));
    assert!(
        underlined(span_style(&styled, "group key")),
        "an admitted name is underlined where it stands"
    );
}

#[test]
fn glossary_terms_are_marked_where_they_appear__term_inside_an_added_paragraph() {
    let glossary = marking_glossary();
    let marks = glossary.marks();
    let palette = Palette::from_env();
    let line = body_line(
        openspec_reviewer::review::ParaKind::Added,
        "A new holder receives the group key.",
    );
    let styled = styled_line(&line, palette, Some(&marks));
    let drawn: String = styled.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(drawn.starts_with("+ "), "the line keeps its glyph: {drawn}");
    let style = span_style(&styled, "group key");
    assert!(underlined(style), "marking composes with the diff");
    assert_eq!(
        style.fg,
        palette.added().fg,
        "the term keeps the added style"
    );
}

#[test]
fn glossary_terms_are_marked_where_they_appear__deprecated_synonym_in_the_text() {
    let glossary = marking_glossary();
    let marks = glossary.marks();
    let palette = Palette::from_env();
    let line = body_line(
        openspec_reviewer::review::ParaKind::Equal,
        "The reader resolves the anchor before trusting it.",
    );
    let styled = styled_line(&line, palette, Some(&marks));
    let style = span_style(&styled, "anchor");
    assert!(underlined(style), "a synonym is marked at least as a term");
    assert_eq!(
        style.fg,
        palette.warning().fg,
        "a deprecated synonym takes the warning style"
    );
}

#[test]
fn glossary_terms_are_marked_where_they_appear__synonym_inside_a_longer_term() {
    let glossary = marking_glossary();
    let marks = glossary.marks();
    let palette = Palette::from_env();
    let line = body_line(
        openspec_reviewer::review::ParaKind::Equal,
        "Rotation uses the PLC rotation key and nothing else.",
    );
    let styled = styled_line(&line, palette, Some(&marks));
    assert!(
        underlined(span_style(&styled, "PLC rotation key")),
        "the whole name is one occurrence"
    );
    assert!(
        styled
            .spans
            .iter()
            .all(|s| s.style.fg != palette.warning().fg || s.content.as_ref().trim().is_empty()),
        "the synonym inside a longer name is not marked as deprecated"
    );
}

#[test]
fn glossary_terms_are_marked_where_they_appear__marking_without_colour() {
    let glossary = marking_glossary();
    let marks = glossary.marks();
    let line = body_line(
        openspec_reviewer::review::ParaKind::Equal,
        "The holder signs with the group key.",
    );
    let styled = styled_line(&line, Palette::none(), Some(&marks));
    let style = span_style(&styled, "group key");
    assert_eq!(style.fg, None, "no colour without colour");
    assert!(underlined(style), "the underline is not a colour");
}

/// The list pane's part of the screen line holding `text`, cut where the
/// two pane borders meet. The detail pane repeats the names.
fn line_with(screen: &str, text: &str) -> String {
    let top = screen.lines().next().unwrap_or("");
    let split = top
        .find("┐┌")
        .map_or(top.chars().count(), |b| top[..b].chars().count() + 1);
    screen
        .lines()
        .map(|l| l.chars().take(split).collect::<String>())
        .find(|l| l.contains(text))
        .unwrap_or_else(|| panic!("no list line holds {text:?}: {screen}"))
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__the_last_requirement_closes_the_branch() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    let screen = render(&mut app, 160, 30);
    assert!(
        line_with(&screen, "Flat index of all routes and pages").contains("├─ ▸ [ ] ~"),
        "{screen}"
    );
    assert!(
        line_with(&screen, "Flat index of all routes !").contains("└─ ▸ [ ] ~"),
        "{screen}"
    );
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__roots_hang_from_the_change() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    let screen = render(&mut app, 160, 30);
    assert!(
        line_with(&screen, "proposal.md").starts_with("│├─ "),
        "{screen}"
    );
    assert!(
        line_with(&screen, "tasks.md").starts_with("│├─ "),
        "{screen}"
    );
    assert!(line_with(&screen, "alpha").starts_with("│└─ "), "{screen}");
}

/// Move the cursor onto the first requirement row without pinning it.
fn hover_first(app: &mut App) {
    while !matches!(app.current_row(), Some(Row::Requirement { .. })) {
        app.handle_key(key(KeyCode::Char('j')));
    }
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__the_cursor_opens_a_requirement() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    assert!(matches!(app.current_row(), Some(Row::Artefact { .. })));
    hover_first(&mut app);
    let scenarios = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Scenario { .. }))
        .count();
    assert_eq!(scenarios, 3);
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__leaving_folds_an_unpinned_requirement() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    hover_first(&mut app);
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('j')));
    }
    assert!(
        matches!(app.current_row(), Some(Row::Requirement { index: 1, .. })),
        "{:?}",
        app.current_row()
    );
    assert!(
        !app.rows
            .iter()
            .any(|r| matches!(r, Row::Scenario { index: 0, .. })),
        "the first requirement folded"
    );
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__space_pins_a_requirement_open() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    hover_first(&mut app);
    app.handle_key(key(KeyCode::Char(' ')));
    for _ in 0..4 {
        app.handle_key(key(KeyCode::Char('j')));
    }
    assert!(matches!(
        app.current_row(),
        Some(Row::Requirement { index: 1, .. })
    ));
    let pinned = app
        .rows
        .iter()
        .filter(|r| matches!(r, Row::Scenario { index: 0, .. }))
        .count();
    assert_eq!(pinned, 3, "the pinned requirement keeps its scenarios");
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__a_scenario_under_a_requirement_with_a_later_sibling() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    unfold_first(&mut app);
    let screen = render(&mut app, 160, 30);
    assert!(
        line_with(&screen, "Route-mounted page appears").contains("│  ├─ "),
        "{screen}"
    );
    assert!(
        line_with(&screen, "Orphan page appears").contains("│  └─ "),
        "{screen}"
    );
}

#[test]
fn the_view_is_a_list_and_a_detail_pane__fold_marker_follows_the_cursor() {
    let repo = tui_repo();
    let mut app = app_for(&repo);
    let screen = render(&mut app, 160, 30);
    assert!(
        line_with(&screen, "Flat index of all routes and pages").contains("▸ [ ]"),
        "{screen}"
    );
    hover_first(&mut app);
    let screen = render(&mut app, 160, 30);
    assert!(
        line_with(&screen, "Flat index of all routes and pages").contains("▾ [ ]"),
        "{screen}"
    );
}
