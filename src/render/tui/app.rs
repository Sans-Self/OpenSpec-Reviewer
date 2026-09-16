//! App state and key handling. No terminal here, so every rule about keys
//! and rows is a unit test over an `App`.

use crate::assist::{AssistError, Hint, Session};
use crate::render::approval;
use crate::review::normalize::text_hash;
use crate::review::pair::{diff_versions, version_lines};
use crate::review::{inline_view, DiffLine, Pairing, Review};
use crate::state::{ApprovalStatus, Store};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    Change {
        change: usize,
    },
    Artefact {
        change: usize,
        index: usize,
    },
    Capability {
        change: usize,
        capability: usize,
    },
    Requirement {
        change: usize,
        capability: usize,
        index: usize,
    },
}

impl Row {
    pub fn selectable(&self) -> bool {
        matches!(self, Row::Artefact { .. } | Row::Requirement { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailMode {
    Inline,
    SideBySide,
    Raw,
}

impl DetailMode {
    pub fn next(self) -> DetailMode {
        match self {
            DetailMode::Inline => DetailMode::SideBySide,
            DetailMode::SideBySide => DetailMode::Raw,
            DetailMode::Raw => DetailMode::Inline,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DetailMode::Inline => "inline",
            DetailMode::SideBySide => "side-by-side",
            DetailMode::Raw => "raw",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryMode {
    Version,
    DiffToPrevious,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryState {
    pub selected: usize,
    pub mode: HistoryMode,
    pub scroll: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Main,
    History(HistoryState),
    Help,
}

/// One pairing's place in a batch run: what to send, and where the reply
/// belongs when it comes back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub change: String,
    pub key: String,
    pub hash: u64,
    pub prompt: std::path::PathBuf,
}

/// A batch run in flight. The worker sends one reply per pairing; the poll
/// loop drains them as they arrive.
pub struct Batch {
    pub done: usize,
    pub total: usize,
    pub replies: Receiver<(Job, Result<Vec<Hint>, String>)>,
    /// Advanced once per draw, for the spinner.
    pub tick: usize,
}

impl Batch {
    const SPINNER: [char; 4] = ['|', '/', '-', '\\'];

    pub fn spinner(&self) -> char {
        Batch::SPINNER[self.tick % Batch::SPINNER.len()]
    }
}

/// What the event loop has to do outside the app: the editor, an agent
/// session that wants the terminal, and the worker a batch run needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    EditNote,
    /// Suspend the view and open the agent on this prompt file.
    Handoff(std::path::PathBuf),
    /// Run these pairings through the agent on a worker thread.
    Batch(Vec<Job>),
}

pub struct App {
    pub review: Review,
    pub rows: Vec<Row>,
    pub cursor: usize,
    pub focus: Pane,
    pub mode: DetailMode,
    pub scroll: u16,
    pub view: View,
    pub quit: bool,
    pub message: Option<String>,
    pub stores: BTreeMap<String, Store>,
    /// Lines the detail pane can show at once; set by the drawer.
    pub detail_height: u16,
    /// Pairings whose definitions panel is open, by key.
    pub definitions_open: std::collections::BTreeSet<String>,
    /// Everything the agent needs; `None` without an `openspec/` tree the
    /// session could read.
    pub assist: Option<Session>,
    pub batch: Option<Batch>,
    /// Which hint of the current pairing `x` would dismiss.
    pub hint_cursor: usize,
}

pub const BINDINGS: &[(&str, &str)] = &[
    ("j / ↓", "next row"),
    ("k / ↑", "previous row"),
    ("Tab", "switch pane focus"),
    ("n / p", "next / previous row with a finding"),
    ("a", "toggle approval"),
    ("e", "edit note in $EDITOR"),
    ("H", "history of this requirement"),
    ("m", "cycle display mode (inline, side-by-side, raw)"),
    ("D", "definitions of the terms this requirement uses"),
    ("i", "open the agent on this pairing, or on the change"),
    (
        "A",
        "ask the agent for hints on this pairing, or on the change",
    ),
    ("x", "dismiss the selected hint"),
    ("PgUp / PgDn, Ctrl-u / Ctrl-d", "scroll the detail pane"),
    ("?", "this help"),
    ("q / Esc", "quit (Esc closes history first)"),
];

fn build_rows(review: &Review) -> Vec<Row> {
    let mut rows = Vec::new();
    let multi = review.changes.len() > 1;
    for (ci, change) in review.changes.iter().enumerate() {
        if multi {
            rows.push(Row::Change { change: ci });
        }
        for ai in 0..change.artefacts.len() {
            rows.push(Row::Artefact {
                change: ci,
                index: ai,
            });
        }
        for (capi, cap) in change.capabilities.iter().enumerate() {
            rows.push(Row::Capability {
                change: ci,
                capability: capi,
            });
            for pi in 0..cap.pairings.len() {
                rows.push(Row::Requirement {
                    change: ci,
                    capability: capi,
                    index: pi,
                });
            }
        }
    }
    rows
}

impl App {
    pub fn new(review: Review, stores: BTreeMap<String, Store>) -> App {
        let rows = build_rows(&review);
        let cursor = rows.iter().position(Row::selectable).unwrap_or(0);
        App {
            review,
            rows,
            cursor,
            focus: Pane::List,
            mode: DetailMode::Inline,
            scroll: 0,
            view: View::Main,
            quit: false,
            message: None,
            stores,
            detail_height: 20,
            definitions_open: std::collections::BTreeSet::new(),
            assist: None,
            batch: None,
            hint_cursor: 0,
        }
    }

    pub fn with_assist(mut self, session: Session) -> App {
        self.assist = Some(session);
        self
    }

    /// The change the cursor sits in, with its proposal text.
    fn current_change(&self) -> Option<(&crate::review::ChangeReview, Option<&str>)> {
        let index = match self.current_row()? {
            Row::Change { change }
            | Row::Artefact { change, .. }
            | Row::Capability { change, .. }
            | Row::Requirement { change, .. } => *change,
        };
        let change = self.review.changes.get(index)?;
        let proposal = change
            .artefacts
            .iter()
            .find(|a| a.artefact.name == "proposal.md")
            .and_then(|a| a.artefact.after.as_deref());
        Some((change, proposal))
    }

    /// The session, or the message saying why no agent will run. Checked
    /// before anything is written, so a key press on an unconfigured
    /// project costs nothing.
    fn ready_session(&mut self) -> Option<&Session> {
        let Some(session) = &self.assist else {
            self.message = Some(AssistError::NotConfigured.to_string());
            return None;
        };
        if let Err(e) = session.check() {
            self.message = Some(e.to_string());
            return None;
        }
        self.assist.as_ref()
    }

    /// `i`: the prompt file to open the agent on. A requirement row hands
    /// over its pairing, any other row the whole change.
    fn handoff(&mut self) -> Option<Effect> {
        self.ready_session()?;
        let session = self.assist.as_ref()?;
        let written = match self.current_pairing() {
            Some(p) => session.write_pairing_prompt(p, false),
            None => {
                let (change, proposal) = self.current_change()?;
                let pairings: Vec<&Pairing> = change.pairings().collect();
                session.write_change_prompt(&change.name, proposal, &pairings, false)
            }
        };
        match written {
            Ok(path) => Some(Effect::Handoff(path)),
            Err(e) => {
                self.message = Some(e.to_string());
                None
            }
        }
    }

    /// `A`: the pairings a batch run covers, each with its prompt written.
    /// A requirement row runs its own pairing, any other row every pairing
    /// of the change.
    fn start_batch(&mut self) -> Option<Effect> {
        if self.batch.is_some() {
            self.message = Some("a batch run is already going".to_string());
            return None;
        }
        self.ready_session()?;
        let session = self.assist.as_ref()?;
        let targets: Vec<&Pairing> = match self.current_pairing() {
            Some(p) => vec![p],
            None => self.current_change()?.0.pairings().collect(),
        };
        let mut jobs = Vec::new();
        for p in targets {
            match session.write_pairing_prompt(p, true) {
                Ok(prompt) => jobs.push(Job {
                    change: p.change.clone(),
                    key: p.key(),
                    hash: p.text_hash(),
                    prompt,
                }),
                Err(e) => {
                    self.message = Some(e.to_string());
                    return None;
                }
            }
        }
        if jobs.is_empty() {
            self.message = Some("nothing to review here".to_string());
            return None;
        }
        Some(Effect::Batch(jobs))
    }

    /// Hand the worker's channel to the app, which drains it as it draws.
    pub fn batch_started(
        &mut self,
        total: usize,
        replies: Receiver<(Job, Result<Vec<Hint>, String>)>,
    ) {
        self.batch = Some(Batch {
            done: 0,
            total,
            replies,
            tick: 0,
        });
    }

    /// Every reply that has arrived since the last draw, stored and shown.
    /// A pairing whose text changed while the agent was reading keeps the
    /// hints under the hash they were made for; the cache drops them on
    /// the next run.
    pub fn poll_batch(&mut self) {
        let Some(batch) = &mut self.batch else {
            return;
        };
        batch.tick += 1;
        let mut arrived = Vec::new();
        while let Ok(reply) = batch.replies.try_recv() {
            arrived.push(reply);
        }
        let disconnected = matches!(
            batch.replies.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        );
        batch.done += arrived.len();
        let finished = batch.done >= batch.total || (arrived.is_empty() && disconnected);
        for (job, result) in arrived {
            match result {
                Ok(hints) => self.store_hints(&job, hints),
                Err(e) => self.message = Some(e),
            }
        }
        if finished {
            self.batch = None;
        }
        self.review.recount();
    }

    fn store_hints(&mut self, job: &Job, hints: Vec<Hint>) {
        let stored = self
            .store_for(&job.change)
            .set_hints(&job.key, job.hash, hints);
        match stored {
            Ok(state) => {
                let shown = state.hints_for(job.hash);
                if let Some(p) = self
                    .review
                    .pairings_mut()
                    .find(|p| p.change == job.change && p.key() == job.key)
                {
                    p.state = state;
                    p.hints = shown;
                }
            }
            Err(e) => self.message = Some(e.to_string()),
        }
    }

    /// Whether `j` and `k` move between hints rather than scroll: only in
    /// the detail pane, and only when there is more than nothing to move
    /// between. Without hints the pane scrolls as it always did.
    fn hints_selectable(&self) -> bool {
        self.focus == Pane::Detail && !self.current_hints().is_empty()
    }

    fn move_hint_cursor(&mut self, forward: bool) {
        let last = self.current_hints().len().saturating_sub(1);
        self.hint_cursor = if forward {
            (self.hint_cursor + 1).min(last)
        } else {
            self.hint_cursor.saturating_sub(1)
        };
    }

    /// The hints `x` can reach: the current pairing's, undismissed.
    pub fn current_hints(&self) -> Vec<Hint> {
        self.current_pairing()
            .map(|p| p.visible_hints().cloned().collect())
            .unwrap_or_default()
    }

    /// `x`: dismiss the selected hint. It stays dismissed when a later run
    /// returns the same kind and message.
    fn dismiss_hint(&mut self) {
        let hints = self.current_hints();
        let Some(hint) = hints.get(self.hint_cursor.min(hints.len().saturating_sub(1))) else {
            return;
        };
        let Some((change, key)) = self.current_pairing().map(|p| (p.change.clone(), p.key()))
        else {
            return;
        };
        let hash = self.current_pairing().map(Pairing::text_hash).unwrap_or(0);
        let dismissed = self
            .store_for(&change)
            .dismiss_hint(&key, hint.kind, &hint.message);
        match dismissed {
            Ok(state) => {
                let shown = state.hints_for(hash);
                if let Some(p) = self.current_pairing_mut() {
                    p.state = state;
                    p.hints = shown;
                }
                self.hint_cursor = 0;
                self.review.recount();
            }
            Err(e) => self.message = Some(e.to_string()),
        }
    }

    /// `D`: toggle the definitions panel for the current pairing.
    pub fn toggle_definitions(&mut self) {
        let Some(key) = self.current_pairing().map(Pairing::key) else {
            return;
        };
        if !self.definitions_open.remove(&key) {
            self.definitions_open.insert(key);
        }
    }

    pub fn definitions_shown(&self) -> bool {
        self.current_pairing()
            .is_some_and(|p| self.definitions_open.contains(&p.key()))
    }

    pub fn current_row(&self) -> Option<&Row> {
        self.rows.get(self.cursor)
    }

    pub fn current_pairing(&self) -> Option<&Pairing> {
        match self.current_row()? {
            Row::Requirement {
                change,
                capability,
                index,
            } => self
                .review
                .changes
                .get(*change)?
                .capabilities
                .get(*capability)?
                .pairings
                .get(*index),
            _ => None,
        }
    }

    fn current_pairing_mut(&mut self) -> Option<&mut Pairing> {
        match self.rows.get(self.cursor)?.clone() {
            Row::Requirement {
                change,
                capability,
                index,
            } => self
                .review
                .changes
                .get_mut(change)?
                .capabilities
                .get_mut(capability)?
                .pairings
                .get_mut(index),
            _ => None,
        }
    }

    pub fn pairing_at(&self, row: &Row) -> Option<&Pairing> {
        match row {
            Row::Requirement {
                change,
                capability,
                index,
            } => self
                .review
                .changes
                .get(*change)?
                .capabilities
                .get(*capability)?
                .pairings
                .get(*index),
            _ => None,
        }
    }

    pub fn artefact_at(&self, row: &Row) -> Option<&crate::review::ArtefactReview> {
        match row {
            Row::Artefact { change, index } => {
                self.review.changes.get(*change)?.artefacts.get(*index)
            }
            _ => None,
        }
    }

    pub fn current_change_name(&self) -> String {
        let index = match self.current_row() {
            Some(Row::Change { change })
            | Some(Row::Artefact { change, .. })
            | Some(Row::Capability { change, .. })
            | Some(Row::Requirement { change, .. }) => *change,
            None => 0,
        };
        self.review
            .changes
            .get(index)
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }

    /// `approved / total` over every selectable row; stale counts as not
    /// approved.
    pub fn approval_counts(&self) -> (usize, usize) {
        let mut approved = 0;
        let mut total = 0;
        for row in &self.rows {
            if let Some(p) = self.pairing_at(row) {
                total += 1;
                if approval(p) == ApprovalStatus::Approved {
                    approved += 1;
                }
            } else if let Some(a) = self.artefact_at(row) {
                total += 1;
                if a.state.status(artefact_hash(a)) == ApprovalStatus::Approved {
                    approved += 1;
                }
            }
        }
        (approved, total)
    }

    fn move_cursor(&mut self, forward: bool) {
        let len = self.rows.len();
        if len == 0 {
            return;
        }
        let mut i = self.cursor;
        loop {
            i = if forward {
                (i + 1).min(len - 1)
            } else {
                i.saturating_sub(1)
            };
            if self.rows[i].selectable() || i == 0 || i == len - 1 {
                break;
            }
        }
        if self.rows[i].selectable() {
            self.cursor = i;
            self.scroll = 0;
            self.hint_cursor = 0;
        }
    }

    fn jump_finding(&mut self, forward: bool) {
        let len = self.rows.len();
        let has_finding = |app: &App, i: usize| {
            app.pairing_at(&app.rows[i])
                .is_some_and(|p| !p.findings.is_empty())
        };
        let candidates: Box<dyn Iterator<Item = usize>> = if forward {
            Box::new((self.cursor + 1..len).chain(0..self.cursor))
        } else {
            Box::new((0..self.cursor).rev().chain((self.cursor + 1..len).rev()))
        };
        for i in candidates {
            if has_finding(self, i) {
                self.cursor = i;
                self.scroll = 0;
                self.hint_cursor = 0;
                return;
            }
        }
    }

    fn scroll_by(&mut self, delta: i32) {
        let apply = |scroll: u16| -> u16 {
            if delta < 0 {
                scroll.saturating_sub(delta.unsigned_abs() as u16)
            } else {
                scroll.saturating_add(delta as u16)
            }
        };
        match &mut self.view {
            View::History(h) => h.scroll = apply(h.scroll),
            _ => self.scroll = apply(self.scroll),
        }
    }

    fn store_for(&mut self, change: &str) -> &mut Store {
        self.stores.entry(change.to_string()).or_default()
    }

    fn toggle_approval(&mut self) {
        let Some(row) = self.current_row().cloned() else {
            return;
        };
        match row {
            Row::Requirement { .. } => {
                let (change, key, hash) = {
                    let p = self.current_pairing().expect("requirement row");
                    (p.change.clone(), p.key(), p.text_hash())
                };
                let result = self.store_for(&change).toggle_approval(&key, hash);
                match result {
                    Ok(state) => {
                        if let Some(p) = self.current_pairing_mut() {
                            p.state = state;
                        }
                    }
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            Row::Artefact { change, index } => {
                let (name, key, hash) = {
                    let c = &self.review.changes[change];
                    let a = &c.artefacts[index];
                    (c.name.clone(), a.key(), artefact_hash(a))
                };
                let result = self.store_for(&name).toggle_approval(&key, hash);
                match result {
                    Ok(state) => self.review.changes[change].artefacts[index].state = state,
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            _ => {}
        }
    }

    pub fn current_note(&self) -> Option<String> {
        let row = self.current_row()?;
        self.pairing_at(row)
            .and_then(|p| p.state.note.clone())
            .or_else(|| self.artefact_at(row).and_then(|a| a.state.note.clone()))
    }

    pub fn set_current_note(&mut self, note: Option<String>) {
        let Some(row) = self.current_row().cloned() else {
            return;
        };
        match row {
            Row::Requirement { .. } => {
                let (change, key) = {
                    let p = self.current_pairing().expect("requirement row");
                    (p.change.clone(), p.key())
                };
                match self.store_for(&change).set_note(&key, note) {
                    Ok(state) => {
                        if let Some(p) = self.current_pairing_mut() {
                            p.state = state;
                        }
                    }
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            Row::Artefact { change, index } => {
                let (name, key) = {
                    let c = &self.review.changes[change];
                    (c.name.clone(), c.artefacts[index].key())
                };
                match self.store_for(&name).set_note(&key, note) {
                    Ok(state) => self.review.changes[change].artefacts[index].state = state,
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            _ => {}
        }
    }

    fn open_history(&mut self) {
        if let Some(p) = self.current_pairing() {
            let entries = history_entries(p).len();
            self.view = View::History(HistoryState {
                selected: entries.saturating_sub(1),
                mode: HistoryMode::Version,
                scroll: 0,
            });
        }
    }

    /// The right pane of the history view for the selected entry.
    pub fn history_lines(&self) -> Vec<DiffLine> {
        let (Some(p), View::History(h)) = (self.current_pairing(), &self.view) else {
            return Vec::new();
        };
        let versions = history_entries(p);
        let Some(current) = versions.get(h.selected) else {
            return Vec::new();
        };
        match (
            h.mode,
            h.selected.checked_sub(1).and_then(|i| versions.get(i)),
        ) {
            (HistoryMode::DiffToPrevious, Some(previous)) => diff_versions(&previous.1, &current.1),
            _ => version_lines(&current.1),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Effect> {
        self.message = None;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match &self.view {
            View::Help => {
                self.view = View::Main;
                return None;
            }
            View::History(_) => return self.handle_history_key(key),
            View::Main => {}
        }
        match (key.code, ctrl) {
            (KeyCode::Char('c'), true) | (KeyCode::Char('q'), false) | (KeyCode::Esc, false) => {
                self.quit = true
            }
            (KeyCode::Char('j'), false) | (KeyCode::Down, false) => {
                if self.focus != Pane::Detail {
                    self.move_cursor(true);
                } else if self.hints_selectable() {
                    self.move_hint_cursor(true);
                } else {
                    self.scroll_by(1);
                }
            }
            (KeyCode::Char('k'), false) | (KeyCode::Up, false) => {
                if self.focus != Pane::Detail {
                    self.move_cursor(false);
                } else if self.hints_selectable() {
                    self.move_hint_cursor(false);
                } else {
                    self.scroll_by(-1);
                }
            }
            (KeyCode::Tab, _) | (KeyCode::BackTab, _) => {
                self.focus = match self.focus {
                    Pane::List => Pane::Detail,
                    Pane::Detail => Pane::List,
                }
            }
            (KeyCode::Char('n'), false) => self.jump_finding(true),
            (KeyCode::Char('p'), false) => self.jump_finding(false),
            (KeyCode::Char('a'), false) => self.toggle_approval(),
            (KeyCode::Char('e'), false) => {
                if self.current_row().is_some_and(Row::selectable) {
                    return Some(Effect::EditNote);
                }
            }
            (KeyCode::Char('i'), false) => return self.handoff(),
            (KeyCode::Char('A'), false) => return self.start_batch(),
            (KeyCode::Char('x'), false) => self.dismiss_hint(),
            (KeyCode::Char('H'), false) => self.open_history(),
            (KeyCode::Char('m'), false) => self.mode = self.mode.next(),
            (KeyCode::Char('D'), false) => self.toggle_definitions(),
            (KeyCode::Char('?'), false) => self.view = View::Help,
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), true) => {
                self.scroll_by(i32::from(self.detail_height / 2).max(1))
            }
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), true) => {
                self.scroll_by(-i32::from(self.detail_height / 2).max(1))
            }
            _ => {}
        }
        None
    }

    fn handle_history_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let entries = self
            .current_pairing()
            .map(|p| history_entries(p).len())
            .unwrap_or(0);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let View::History(h) = &mut self.view else {
            return None;
        };
        match (key.code, ctrl) {
            (KeyCode::Esc, _) | (KeyCode::Char('H'), false) => self.view = View::Main,
            (KeyCode::Char('q'), false) | (KeyCode::Char('c'), true) => self.quit = true,
            (KeyCode::Char('j'), false) | (KeyCode::Down, false) => {
                h.selected = (h.selected + 1).min(entries.saturating_sub(1));
                h.scroll = 0;
            }
            (KeyCode::Char('k'), false) | (KeyCode::Up, false) => {
                h.selected = h.selected.saturating_sub(1);
                h.scroll = 0;
            }
            (KeyCode::Char('m'), false) => {
                h.mode = match h.mode {
                    HistoryMode::Version => HistoryMode::DiffToPrevious,
                    HistoryMode::DiffToPrevious => HistoryMode::Version,
                }
            }
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), true) => {
                let half = i32::from(self.detail_height / 2).max(1);
                self.scroll_by(half)
            }
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), true) => {
                let half = i32::from(self.detail_height / 2).max(1);
                self.scroll_by(-half)
            }
            (KeyCode::Char('?'), false) => self.view = View::Help,
            _ => {}
        }
        None
    }

    pub fn finding_counts(&self) -> (usize, usize, usize) {
        let s = &self.review.summary;
        (s.errors, s.warnings, s.notes)
    }

    pub fn detail_lines(&self) -> Vec<DiffLine> {
        match self.current_row() {
            Some(row) => match (self.pairing_at(row), self.artefact_at(row)) {
                (Some(p), _) => inline_view(p),
                (_, Some(a)) => a.lines(),
                _ => Vec::new(),
            },
            None => Vec::new(),
        }
    }
}

pub fn artefact_hash(a: &crate::review::ArtefactReview) -> u64 {
    text_hash(a.artefact.after.as_deref().unwrap_or(""))
}

/// Archived versions oldest first, then the version under review as
/// `current`. A requirement no archive mentions has one version.
pub fn history_entries(p: &Pairing) -> Vec<(String, crate::model::Requirement)> {
    let mut out: Vec<(String, crate::model::Requirement)> = p
        .history
        .iter()
        .map(|h| (h.label(), h.text.clone()))
        .collect();
    if let Some(current) = p.after.as_ref().or(p.before.as_ref()) {
        out.push(("current".to_string(), current.clone()));
    }
    out
}
