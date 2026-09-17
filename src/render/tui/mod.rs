//! The interactive view: a list on the left, a detail pane on the right.

mod app;
mod tree;
mod ui;

pub use app::{
    App, DetailMode, Effect, Focus, HistoryMode, Modal, NoteEdit, Pane, Row, Transient, BINDINGS,
};
pub use ui::{draw, status_text, styled_line};

use crate::review::Review;
use crate::state::Store;
use crossterm::event::{self, Event, KeyEventKind};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use std::collections::BTreeMap;
use std::io;
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
pub fn run(review: Review, stores: BTreeMap<String, Store>) -> io::Result<()> {
    let mut app = App::new(review, stores);
    let interrupted = install_signal_flag()?;
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &mut app, &interrupted);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    interrupted: &AtomicBool,
) -> io::Result<()> {
    while !app.quit && !interrupted.load(Ordering::Relaxed) {
        terminal.draw(|f| ui::draw(f, app))?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if let Some(app::Effect::EscalateNote) = app.handle_key(key) {
                    escalate_to_editor(terminal, app)?;
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
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
