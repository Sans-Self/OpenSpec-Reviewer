//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use openspec_reviewer::build::build_review;
use openspec_reviewer::render::colour::{Background, Palette, PaletteChoice};
use openspec_reviewer::render::terminal::{
    background_from_colorfgbg, background_from_reply, take_reply,
};
use openspec_reviewer::render::tui::{App, Row};
use openspec_reviewer::source::{ChangeSource, Source};
use ratatui::backend::TestBackend;
use ratatui::buffer::{Buffer, Cell};
use ratatui::style::Color;
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

#### Scenario: Nested page inherits the parent path

- **WHEN** a page sits under a nested route
- **THEN** the row shows the joined path

### Requirement: Flat index of all routes

Typo in the name: no canon target.

#### Scenario: S

- a
";

const WIDTH: u16 = 160;
const HEIGHT: u16 = 40;

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn app() -> App {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA)
        .write("openspec/changes/foo/proposal.md", "# foo\n");
    let snap = ChangeSource {
        root: repo.root().into(),
        name: "foo".into(),
    }
    .fetch()
    .unwrap();
    let mut app = App::new(build_review(repo.root(), &snap).unwrap(), BTreeMap::new());
    app.palette = Palette::new(PaletteChoice::Default, Background::Dark);
    app
}

fn on_requirement(app: &mut App, name: &str) {
    while app.current_pairing().map(|p| p.name.as_str()) != Some(name)
        || !matches!(app.current_row(), Some(Row::Requirement { .. }))
    {
        app.handle_key(key('j'));
    }
}

fn buffer(app: &mut App) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT)).unwrap();
    terminal
        .draw(|f| openspec_reviewer::render::tui::draw(f, app))
        .unwrap();
    terminal.backend().buffer().clone()
}

fn screen(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The row and first column of `needle` on the screen, searching from
/// column `from`, so the detail pane can be told apart from the list.
fn locate(buf: &Buffer, needle: &str, from: u16) -> (u16, u16) {
    for y in 0..buf.area.height {
        let row: Vec<&str> = (from..buf.area.width)
            .map(|x| buf[(x, y)].symbol())
            .collect();
        let text = row.concat();
        if let Some(byte) = text.find(needle) {
            let mut seen = 0;
            for (i, sym) in row.iter().enumerate() {
                if seen == byte {
                    return (from + i as u16, y);
                }
                seen += sym.len();
            }
        }
    }
    panic!("no {needle:?} on screen:\n{}", screen(buf));
}

fn cell<'a>(buf: &'a Buffer, needle: &str, from: u16) -> &'a Cell {
    let (x, y) = locate(buf, needle, from);
    &buf[(x, y)]
}

/// The detail pane starts at 40% of the width; its first text column
/// is one past the border.
const DETAIL: u16 = WIDTH * 2 / 5;

#[test]
fn colour_mode_paints_the_diff_by_background__added_line() {
    let mut app = app();
    on_requirement(&mut app, "Flat index of all routes and pages");
    let buf = buffer(&mut app);
    let (x, y) = locate(&buf, "a page sits under a nested route", DETAIL);
    assert_eq!(buf[(x, y)].bg, Color::Green, "{}", screen(&buf));
    assert_eq!(
        buf[(DETAIL + 1, y)].symbol(),
        "+",
        "the line starts with the glyph"
    );
    assert_eq!(
        buf[(WIDTH - 2, y)].bg,
        Color::Green,
        "the tint reaches the pane's edge"
    );
    assert_eq!(
        buf[(x, y)].fg,
        Color::Reset,
        "default foreground inside a tint"
    );
}

#[test]
fn colour_mode_paints_the_diff_by_background__changed_words() {
    let mut app = app();
    on_requirement(&mut app, "Flat index of all routes and pages");
    let buf = buffer(&mut app);
    assert_eq!(cell(&buf, "inherited", DETAIL).bg, Color::Green);
    assert_eq!(cell(&buf, "website.", DETAIL).bg, Color::Red);
    assert_eq!(cell(&buf, "dashboard", DETAIL).bg, Color::Reset);
    let text = screen(&buf);
    assert!(!text.contains("[-") && !text.contains("{+"), "{text}");
}

#[test]
fn colour_mode_paints_the_diff_by_background__non_colour_keeps_the_marks() {
    let mut app = app();
    app.palette = Palette::none();
    on_requirement(&mut app, "Flat index of all routes and pages");
    let text = screen(&buffer(&mut app));
    assert!(text.contains("[-") && text.contains("{+"), "{text}");
}

#[test]
fn colour_is_never_the_only_signal__tinted_span_keeps_its_modifier() {
    let mut app = app();
    on_requirement(&mut app, "Flat index of all routes and pages");
    let buf = buffer(&mut app);
    let removed = cell(&buf, "website.", DETAIL);
    assert_eq!(removed.bg, Color::Red);
    assert!(removed
        .modifier
        .contains(ratatui::style::Modifier::CROSSED_OUT));
}

#[test]
fn the_list_pane_colours_its_markers__error_marker() {
    let mut app = app();
    let buf = buffer(&mut app);
    let (x, y) = locate(&buf, "Flat index of all routes !", 0);
    let bang = x + "Flat index of all routes ".chars().count() as u16;
    assert_eq!(buf[(bang, y)].symbol(), "!");
    assert_eq!(buf[(bang, y)].fg, Color::Red);
}

#[test]
fn the_list_pane_colours_its_markers__approved_mark() {
    let mut app = app();
    on_requirement(&mut app, "Flat index of all routes and pages");
    app.handle_key(key('a'));
    let buf = buffer(&mut app);
    assert_eq!(cell(&buf, "√", 0).fg, Color::Green);
}

#[test]
fn the_focused_pane_has_a_coloured_border__tab_moves_the_accent() {
    let mut app = app();
    let buf = buffer(&mut app);
    let list_border = &buf[(0, 0)];
    let detail_border = &buf[(DETAIL, 0)];
    assert_eq!(list_border.fg, Color::Cyan, "the list starts focused");
    assert_eq!(detail_border.fg, Color::DarkGray);

    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let buf = buffer(&mut app);
    assert_eq!(buf[(DETAIL, 0)].fg, Color::Cyan, "Tab moved the accent");
    assert_eq!(buf[(0, 0)].fg, Color::DarkGray);
}

#[test]
fn the_status_bar_colours_non_zero_counts__one_error_no_warnings() {
    let mut app = app();
    let buf = buffer(&mut app);
    assert_eq!(cell(&buf, "1 errors", 0).fg, Color::Red);
    assert_eq!(cell(&buf, "0 warnings", 0).fg, Color::Reset);
}

#[test]
fn lint_output_is_coloured_on_a_terminal__piped_lint() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/stale.test.ts",
        "test(`spec:nowhere § Anything`, () => {})\n",
    );
    let out = run_in(repo.root(), &["lint"]);
    assert!(!out.stdout.contains(&0x1b) && !out.stderr.contains(&0x1b));
}

#[test]
fn lint_output_is_coloured_on_a_terminal__forced_colour() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/stale.test.ts",
        "test(`spec:nowhere § Anything`, () => {})\n",
    );
    let out = run_in(repo.root(), &["lint", "--color"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("\x1b[1;31merror\x1b[0m"),
        "the severity word is painted: {err}"
    );
}

#[test]
fn the_palette_adapts_to_the_terminal_background__colorfgbg_says_light() {
    assert_eq!(background_from_colorfgbg("0;15"), Some(Background::Light));
    assert_eq!(background_from_colorfgbg("15;0"), Some(Background::Dark));
    assert_eq!(
        background_from_colorfgbg("15;default;7"),
        Some(Background::Light)
    );
    let light = Palette::new(PaletteChoice::Default, Background::Light);
    assert_eq!(light.warning().fg, Some(Color::Magenta));
}

#[test]
fn the_palette_adapts_to_the_terminal_background__no_answer() {
    assert_eq!(take_reply(b""), None);
    assert_eq!(take_reply(b"\x1b]11;rgb:0000/0000"), None, "unterminated");
    let dark = Palette::new(PaletteChoice::Default, Background::Dark);
    assert_eq!(dark.warning().fg, Some(Color::Yellow));
}

#[test]
fn the_palette_adapts_to_the_terminal_background__reply_is_classified() {
    assert_eq!(
        background_from_reply("rgb:ffff/ffff/ffff"),
        Some(Background::Light)
    );
    assert_eq!(
        background_from_reply("rgb:1e1e/1e1e/1e1e"),
        Some(Background::Dark)
    );
    let reply = take_reply(b"\x1b]11;rgb:fdfd/f6f6/e3e3\x07").unwrap();
    assert_eq!(background_from_reply(reply), Some(Background::Light));
    let reply = take_reply(b"junk\x1b]11;rgb:00/00/00\x1b\\").unwrap();
    assert_eq!(background_from_reply(reply), Some(Background::Dark));
}

#[test]
fn three_palettes__accessible_added_line() {
    let mut app = app();
    app.palette = Palette::new(PaletteChoice::Accessible, Background::Dark);
    on_requirement(&mut app, "Flat index of all routes and pages");
    let buf = buffer(&mut app);
    assert_eq!(
        cell(&buf, "a page sits under a nested route", DETAIL).bg,
        Color::Blue
    );
    assert_eq!(cell(&buf, "website.", DETAIL).bg, Color::Yellow);
    let light = Palette::new(PaletteChoice::Accessible, Background::Light);
    assert_eq!(light.removed_span().bg, Some(Color::Magenta));
}

#[test]
fn three_palettes__none_is_non_colour() {
    let mut app = app();
    app.palette = Palette::none();
    on_requirement(&mut app, "Flat index of all routes and pages");
    let buf = buffer(&mut app);
    let text = screen(&buf);
    assert!(text.contains("[-") && text.contains("{+"), "{text}");
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let c = &buf[(x, y)];
            assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset), "cell {x},{y}");
        }
    }
}

#[test]
fn three_palettes__every_role_carries_a_modifier() {
    for choice in [
        PaletteChoice::Default,
        PaletteChoice::Accessible,
        PaletteChoice::None,
    ] {
        for background in [Background::Dark, Background::Light] {
            let p = Palette::new(choice, background);
            let roles = [
                p.added(),
                p.removed(),
                p.changed(),
                p.added_span(),
                p.removed_span(),
                p.error(),
                p.warning(),
                p.note(),
                p.approved(),
                p.stale(),
                p.pending(),
                p.accent(),
                p.muted(),
                p.heading(),
                p.term(),
                p.synonym(),
                p.focus(),
                p.unfocus(),
                p.selected(),
            ];
            for role in roles {
                assert!(
                    !role.add_modifier.is_empty(),
                    "{choice:?} {background:?} {role:?}"
                );
            }
        }
    }
}
