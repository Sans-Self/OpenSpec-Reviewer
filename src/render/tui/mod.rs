//! The interactive view: a list on the left, a detail pane on the right.

mod app;
mod tree;
mod ui;

pub use app::{
    App, DetailMode, Effect, Focus, HistoryMode, Modal, NoteEdit, NoteRow, NotesState, Pane,
    QuitPrompt, Row, Transient, BINDINGS,
};
pub use ui::{draw, status_text, styled_line, styled_line_wide};

use crate::render::colour::{Background, Palette, PaletteChoice};
use crate::render::terminal::detect_background;
use crate::review::Review;
use crate::state::Store;
use crossterm::event::{self, Event, KeyEventKind};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// SIGTERM and SIGHUP set a flag the poll loop reads; the loop then exits
/// and the terminal is restored on the main thread. Raw mode swallows
/// Ctrl-C into a key event, so SIGINT here only comes from outside.
fn install_signal_flag() -> io::Result<Arc<AtomicBool>> {
    let flag = Arc::new(AtomicBool::new(false));
    for sig in [SIGINT, SIGTERM, SIGHUP] {
        signal_hook::flag::register(sig, Arc::clone(&flag))?;
    }
    Ok(flag)
}

/// Run the view until the user quits. `ratatui::init` installs a panic
/// hook that restores the terminal before the message prints.
pub fn run(
    root: &Path,
    review: Review,
    stores: BTreeMap<String, Store>,
    palette: PaletteChoice,
) -> io::Result<()> {
    let mut app = App::new(review, stores);
    let interrupted = install_signal_flag()?;
    let mut terminal = ratatui::try_init()?;
    // Raw mode is on and nothing has been drawn: the one moment the
    // terminal can be asked about its background without a keystroke
    // getting mixed into the answer.
    let choice = palette.resolve(true, false);
    let background = match choice {
        PaletteChoice::None => Background::Dark,
        _ => detect_background(true),
    };
    app.palette = Palette::new(choice, background);
    let result = event_loop(root, &mut terminal, &mut app, &interrupted);
    ratatui::restore();
    result
}

fn event_loop(
    root: &Path,
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    interrupted: &AtomicBool,
) -> io::Result<()> {
    // Nothing in the view moves on its own, so a frame drawn on a poll
    // timeout is the frame already on the screen. The poll stays at 250 ms
    // so a signal is still noticed that soon.
    let mut dirty = true;
    while !app.quit && !interrupted.load(Ordering::Relaxed) {
        if dirty {
            terminal.draw(|f| ui::draw(f, app))?;
            dirty = false;
        }
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                match app.handle_key(key) {
                    Some(app::Effect::EscalateNote) => escalate_to_editor(terminal, app)?,
                    Some(app::Effect::PostNotes) => post_notes(root, app),
                    None => {}
                }
                dirty = true;
            }
            Event::Resize(_, _) => dirty = true,
            _ => {}
        }
    }
    Ok(())
}

/// `y` in the quit prompt: send the unposted notes, stamp them and leave.
/// A failure keeps the view open with gh's own words in the status line,
/// because the notes are safe locally and a retry costs nothing.
pub fn post_notes(root: &Path, app: &mut App) {
    let (Some(pr), Some(post)) = (app.review.pull_request.clone(), app.unposted_notes()) else {
        app.quit = true;
        return;
    };
    match crate::source::post_review(root, &pr, &post) {
        Ok(url) => {
            stamp(app, &post, &url);
            app.quit = true;
        }
        Err(e) => {
            app.focus = Focus::Browsing;
            app.message = Some(e.to_string());
        }
    }
}

/// Record the review every sent note now stands in, one store at a time.
fn stamp(app: &mut App, post: &crate::review::post::ReviewPost, url: &str) {
    let mut by_change: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (change, key) in &post.keys {
        by_change
            .entry(change.clone())
            .or_default()
            .push(key.clone());
    }
    for (change, keys) in by_change {
        let Some(store) = app.stores.get_mut(&change) else {
            continue;
        };
        if let Err(e) = store.mark_posted(&keys, url) {
            app.message = Some(e.to_string());
        }
    }
}

/// `^E`: leave the alternate screen, run the editor on the popup's buffer,
/// come back holding what it saved. Anything that wants line structure
/// leaves for the real editor and comes back.
fn escalate_to_editor(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> io::Result<()> {
    let current = app.note_edit().map(|e| e.buffer.clone());
    ratatui::restore();
    let edited = crate::state::edit_note(current.as_deref());
    *terminal = ratatui::try_init()?;
    terminal.clear()?;
    match edited {
        Ok(text) => app.set_note_buffer(text.as_deref().unwrap_or("")),
        Err(e) => app.message = Some(format!("editor failed: {e}")),
    }
    Ok(())
}
