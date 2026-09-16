//! The interactive view: a list on the left, a detail pane on the right.

mod app;
mod ui;

pub use app::{App, Batch, DetailMode, Effect, HistoryMode, Job, Pane, Row, View, BINDINGS};
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
pub fn run(
    review: Review,
    stores: BTreeMap<String, Store>,
    assist: Option<crate::assist::Session>,
) -> io::Result<()> {
    let mut app = App::new(review, stores);
    if let Some(session) = assist {
        app = app.with_assist(session);
    }
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
        app.poll_batch();
        terminal.draw(|f| ui::draw(f, app))?;
        // Short while an agent is working, so the spinner turns and replies
        // land as they arrive; a quarter second otherwise.
        let wait = if app.batch.is_some() { 100 } else { 250 };
        if !event::poll(Duration::from_millis(wait))? {
            continue;
        }
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => match app.handle_key(key) {
                Some(app::Effect::EditNote) => edit_note_suspended(terminal, app)?,
                Some(app::Effect::Handoff(prompt)) => handoff_suspended(terminal, app, &prompt)?,
                Some(app::Effect::Batch(jobs)) => start_worker(app, jobs),
                None => {}
            },
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}

/// Leave the alternate screen, hand the terminal to the agent, come back
/// with the same row selected: the cursor is never touched.
fn handoff_suspended(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    prompt: &std::path::Path,
) -> io::Result<()> {
    let agent = match app.assist.as_ref() {
        Some(session) => session.assistant(),
        None => Err(crate::assist::AssistError::NotConfigured),
    };
    ratatui::restore();
    let result = agent.and_then(|agent| agent.handoff(prompt));
    *terminal = ratatui::try_init()?;
    terminal.clear()?;
    if let Err(e) = result {
        app.message = Some(e.to_string());
    }
    Ok(())
}

/// One thread, one pairing at a time: the agent CLI is the bottleneck, and
/// parallel calls multiply the reviewer's bill.
fn start_worker(app: &mut App, jobs: Vec<app::Job>) {
    let agent = match app.assist.as_ref().map(|s| s.assistant()) {
        Some(Ok(agent)) => agent,
        Some(Err(e)) => {
            app.message = Some(e.to_string());
            return;
        }
        None => {
            app.message = Some(crate::assist::AssistError::NotConfigured.to_string());
            return;
        }
    };
    let (tx, rx) = std::sync::mpsc::channel();
    let total = jobs.len();
    std::thread::spawn(move || {
        for job in jobs {
            let reply = agent
                .review(&job.prompt)
                .map(|text| crate::assist::parse_hints(&text))
                .map_err(|e| e.to_string());
            if tx.send((job, reply)).is_err() {
                return;
            }
        }
    });
    app.batch_started(total, rx);
}

/// Leave the alternate screen, run the editor, come back.
fn edit_note_suspended(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> io::Result<()> {
    ratatui::restore();
    let current = app.current_note();
    let edited = crate::state::edit_note(current.as_deref());
    *terminal = ratatui::try_init()?;
    terminal.clear()?;
    match edited {
        Ok(note) => app.set_current_note(note),
        Err(e) => app.message = Some(format!("editor failed: {e}")),
    }
    Ok(())
}
