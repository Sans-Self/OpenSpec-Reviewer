//! App state and key handling. No terminal here, so every rule about keys
//! and rows is a unit test over an `App`.

use crate::glossary::Marks;
use crate::render::approval;
use crate::render::colour::Palette;
use crate::review::pair::{diff_versions, version_lines};
use crate::review::{inline_view, scenario_view, DiffLine, Pairing, Review, ScenarioMatch};
use crate::state::{ApprovalStatus, ItemState, Store};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::{BTreeMap, BTreeSet};

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
    Scenario {
        change: usize,
        capability: usize,
        index: usize,
        scenario: usize,
    },
}

/// Which requirement a row belongs to, which is also what folds.
type Anchor = (usize, usize, usize);

impl Row {
    pub fn selectable(&self) -> bool {
        matches!(
            self,
            Row::Artefact { .. } | Row::Requirement { .. } | Row::Scenario { .. }
        )
    }

    /// The requirement a row is or hangs under.
    pub fn anchor(&self) -> Option<Anchor> {
        match *self {
            Row::Requirement {
                change,
                capability,
                index,
            }
            | Row::Scenario {
                change,
                capability,
                index,
                ..
            } => Some((change, capability, index)),
            _ => None,
        }
    }

    fn requirement_row(anchor: Anchor) -> Row {
        Row::Requirement {
            change: anchor.0,
            capability: anchor.1,
            index: anchor.2,
        }
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

/// A note being written: the anchor it hangs on, quoted so the popup does
/// not depend on what the detail pane happens to be showing, and one flat
/// buffer that wraps at render width.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteEdit {
    pub row: Row,
    pub title: String,
    pub quote: Vec<String>,
    pub buffer: String,
}

/// An overlay any key closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transient {
    Help,
}

/// One row of the notes panel: the note, how its anchor reads, and where
/// in the main list `Enter` has to land.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRow {
    pub row: Row,
    /// The anchor as plain output names it: `<capability> § <requirement>`,
    /// a chevron and the scenario for a scenario note, or the artefact's
    /// name.
    pub label: String,
    pub first_line: String,
    pub outdated: bool,
}

/// The notes panel. `confirming` is `X` waiting for its answer: it lives
/// here rather than in a transient, because the answer returns to the
/// panel and a transient returns to browsing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotesState {
    pub selected: usize,
    pub confirming: bool,
}

/// What the quit prompt says: how many notes are unposted, and which pull
/// request they would go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuitPrompt {
    pub notes: usize,
    pub pull_request: u64,
}

/// An overlay that swallows every key until it closes itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    History(HistoryState),
    Note(NoteEdit),
    Notes(NotesState),
    Quit(QuitPrompt),
}

/// Where the keys go. One modal at a time: a flat enum, not a stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Focus {
    Browsing,
    Transient(Transient),
    Modal(Modal),
}

/// What the event loop has to do outside the app: only the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// `^E`: suspend the view and hand the popup's buffer to `$EDITOR`.
    EscalateNote,
    /// `y` in the quit prompt: send the unposted notes to the pull request.
    PostNotes,
}

pub struct App {
    pub review: Review,
    pub rows: Vec<Row>,
    pub cursor: usize,
    pub pane: Pane,
    pub mode: DetailMode,
    pub scroll: u16,
    pub focus: Focus,
    pub quit: bool,
    pub message: Option<String>,
    pub stores: BTreeMap<String, Store>,
    /// Lines the detail pane can show at once; set by the drawer.
    pub detail_height: u16,
    /// Pairings whose definitions panel is open, by key.
    pub definitions_open: BTreeSet<String>,
    /// Requirements whose scenarios are showing. Folded is the default, so
    /// an empty set is a view that has just opened.
    pub unfolded: BTreeSet<Anchor>,
    /// Resolved once before the first draw; tests set it directly.
    pub palette: Palette,
    /// The glossary's matchers, compiled once: the glossary cannot change
    /// while the view is open.
    marks: Marks,
    /// The diff of the row the cache was filled from. A diff costs more
    /// than a frame does, and it only changes when the selection does.
    detail: DetailCache,
}

/// The lines of one row's diff, and which row they belong to.
#[derive(Default)]
struct DetailCache {
    row: Option<Row>,
    lines: Vec<DiffLine>,
}

pub const BINDINGS: &[(&str, &str)] = &[
    ("j / ↓", "next row"),
    ("k / ↑", "previous row"),
    ("Tab", "switch pane focus"),
    ("n / p", "next / previous row with a finding"),
    ("Space", "fold / unfold a requirement's scenarios"),
    ("a", "toggle approval (on a scenario: its requirement)"),
    ("e", "write a note on this row"),
    ("⏎ / Esc", "in the note popup: save / cancel"),
    ("^E", "in the note popup: hand the text to $EDITOR"),
    ("N", "every note of the change, in a panel"),
    (
        "⏎ / d / X",
        "in the notes panel: go to the row / delete / clear every note",
    ),
    ("H", "history of this requirement"),
    ("m", "cycle display mode (inline, side-by-side, raw)"),
    ("D", "definitions of the terms this requirement uses"),
    ("PgUp / PgDn, Ctrl-u / Ctrl-d", "scroll the detail pane"),
    ("?", "this help"),
    ("q / Esc", "quit (Esc closes an overlay first)"),
    ("y / n", "in the quit prompt: post the notes / quit without"),
];

/// Every row the list can hold, folds ignored.
fn all_rows(review: &Review) -> Vec<Row> {
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
            for (pi, p) in cap.pairings.iter().enumerate() {
                rows.push(Row::Requirement {
                    change: ci,
                    capability: capi,
                    index: pi,
                });
                for si in 0..p.diff.scenarios.len() {
                    rows.push(Row::Scenario {
                        change: ci,
                        capability: capi,
                        index: pi,
                        scenario: si,
                    });
                }
            }
        }
    }
    rows
}

/// The rows the list shows: every row but the scenarios of requirements
/// that are neither pinned nor under the cursor.
fn visible_rows(review: &Review, pinned: &BTreeSet<Anchor>, hovered: Option<Anchor>) -> Vec<Row> {
    all_rows(review)
        .into_iter()
        .filter(|row| match row {
            Row::Scenario { .. } => row
                .anchor()
                .is_some_and(|a| pinned.contains(&a) || hovered == Some(a)),
            _ => true,
        })
        .collect()
}

impl App {
    pub fn new(review: Review, stores: BTreeMap<String, Store>) -> App {
        let rows = visible_rows(&review, &BTreeSet::new(), None);
        let cursor = rows.iter().position(Row::selectable).unwrap_or(0);
        let marks = review.glossary.marks();
        let mut app = App {
            review,
            rows,
            cursor,
            pane: Pane::List,
            mode: DetailMode::Inline,
            scroll: 0,
            focus: Focus::Browsing,
            quit: false,
            message: None,
            stores,
            detail_height: 20,
            definitions_open: BTreeSet::new(),
            unfolded: BTreeSet::new(),
            palette: Palette::from_env(),
            marks,
            detail: DetailCache::default(),
        };
        app.rebuild_rows();
        app
    }

    /// The glossary's compiled matchers, or nothing when the project has no
    /// glossary: a `Marks` with no terms would mark nothing anyway, and the
    /// drawer's `None` skips the work of asking.
    pub fn marks(&self) -> Option<&Marks> {
        (!self.review.glossary.is_empty()).then_some(&self.marks)
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

    /// The pairing of the row under the cursor, scenario rows included.
    pub fn current_pairing(&self) -> Option<&Pairing> {
        self.pairing_under(self.current_row()?)
    }

    fn pairing_mut(&mut self, anchor: Anchor) -> Option<&mut Pairing> {
        self.review
            .changes
            .get_mut(anchor.0)?
            .capabilities
            .get_mut(anchor.1)?
            .pairings
            .get_mut(anchor.2)
    }

    /// The pairing a row shows: only a requirement row has one of its own.
    pub fn pairing_at(&self, row: &Row) -> Option<&Pairing> {
        match row {
            Row::Requirement { .. } => self.pairing_under(row),
            _ => None,
        }
    }

    /// The pairing a row belongs to, a scenario row's parent included.
    pub fn pairing_under(&self, row: &Row) -> Option<&Pairing> {
        let (change, capability, index) = row.anchor()?;
        self.review
            .changes
            .get(change)?
            .capabilities
            .get(capability)?
            .pairings
            .get(index)
    }

    /// A scenario row's pairing and the scenario it names.
    pub fn scenario_at(&self, row: &Row) -> Option<(&Pairing, &ScenarioMatch)> {
        match row {
            Row::Scenario { scenario, .. } => {
                let p = self.pairing_under(row)?;
                Some((p, p.diff.scenarios.get(*scenario)?))
            }
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
            | Some(Row::Requirement { change, .. })
            | Some(Row::Scenario { change, .. }) => *change,
            None => 0,
        };
        self.review
            .changes
            .get(index)
            .map(|c| c.name.clone())
            .unwrap_or_default()
    }

    /// `approved / total` over every approvable row; stale counts as not
    /// approved. Scenario rows carry no approval of their own, so folding
    /// never moves the total.
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
                if a.state.status(a.text_hash()) == ApprovalStatus::Approved {
                    approved += 1;
                }
            }
        }
        (approved, total)
    }

    fn index_of(&self, row: &Row) -> Option<usize> {
        self.rows.iter().position(|r| r == row)
    }

    /// Whether a requirement shows its scenarios: pinned with `Space`, or
    /// the cursor is on it or inside it.
    pub fn is_open(&self, anchor: Anchor) -> bool {
        self.unfolded.contains(&anchor) || self.hovered() == Some(anchor)
    }

    fn hovered(&self) -> Option<Anchor> {
        self.current_row().and_then(Row::anchor)
    }

    /// Rebuild the list after a fold or a cursor move, keeping the cursor
    /// on the row it was on, or on that row's requirement when the fold
    /// hid it.
    fn rebuild_rows(&mut self) {
        let keep = self.current_row().cloned();
        self.rows = visible_rows(&self.review, &self.unfolded, self.hovered());
        self.cursor = keep
            .and_then(|row| {
                self.index_of(&row).or_else(|| {
                    row.anchor()
                        .and_then(|a| self.index_of(&Row::requirement_row(a)))
                })
            })
            .unwrap_or_else(|| self.rows.iter().position(Row::selectable).unwrap_or(0));
    }

    /// `Space`: pin or unpin the scenarios of the requirement the cursor
    /// is on or inside.
    fn toggle_fold(&mut self) {
        let Some(anchor) = self.current_row().and_then(Row::anchor) else {
            return;
        };
        if !self.unfolded.remove(&anchor) {
            self.unfolded.insert(anchor);
        }
        self.rebuild_rows();
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
            self.rebuild_rows();
        }
    }

    /// A finding belongs to the scenario row it names, and to the
    /// requirement row otherwise — including when it names a scenario that
    /// has no row, so no finding becomes unreachable.
    fn row_has_finding(&self, row: &Row) -> bool {
        match row {
            Row::Requirement { .. } => {
                let Some(p) = self.pairing_at(row) else {
                    return false;
                };
                p.findings.iter().any(|f| match &f.location.scenario {
                    None => true,
                    Some(s) => !p.diff.scenarios.iter().any(|m| m.name() == s),
                })
            }
            Row::Scenario { .. } => {
                let Some((p, m)) = self.scenario_at(row) else {
                    return false;
                };
                p.findings
                    .iter()
                    .any(|f| f.location.scenario.as_deref() == Some(m.name()))
            }
            _ => false,
        }
    }

    /// Put the cursor on `target`. The target's requirement opens by
    /// virtue of the cursor landing on or inside it, so the row exists
    /// before the cursor looks for it.
    fn jump_to(&mut self, target: &Row) {
        self.rows = visible_rows(&self.review, &self.unfolded, target.anchor());
        if let Some(i) = self.index_of(target) {
            self.cursor = i;
            self.scroll = 0;
        }
    }

    /// `n` and `p`. The search runs over every row, folded or not: the
    /// cursor has to be able to reach where it is going, so a jump into a
    /// folded requirement unfolds it.
    fn jump_finding(&mut self, forward: bool) {
        let rows = all_rows(&self.review);
        let start = self
            .current_row()
            .and_then(|current| rows.iter().position(|r| r == current))
            .unwrap_or(0);
        let len = rows.len();
        let order: Vec<usize> = if forward {
            (start + 1..len).chain(0..start).collect()
        } else {
            (0..start).rev().chain((start + 1..len).rev()).collect()
        };
        let Some(target) = order
            .into_iter()
            .map(|i| rows[i].clone())
            .find(|row| self.row_has_finding(row))
        else {
            return;
        };
        self.jump_to(&target);
    }

    fn scroll_by(&mut self, delta: i32) {
        let apply = |scroll: u16| -> u16 {
            if delta < 0 {
                scroll.saturating_sub(delta.unsigned_abs() as u16)
            } else {
                scroll.saturating_add(delta as u16)
            }
        };
        match &mut self.focus {
            Focus::Modal(Modal::History(h)) => h.scroll = apply(h.scroll),
            _ => self.scroll = apply(self.scroll),
        }
    }

    fn store_for(&mut self, change: &str) -> &mut Store {
        self.stores.entry(change.to_string()).or_default()
    }

    /// Read a pairing's approval and notes back out of its store.
    fn sync_pairing(&mut self, anchor: Anchor, change: &str) {
        let items = self
            .stores
            .get(change)
            .map(|s| s.state.items.clone())
            .unwrap_or_default();
        if let Some(p) = self.pairing_mut(anchor) {
            p.state = items.get(&p.key()).cloned().unwrap_or_default();
            p.refresh_notes(&items);
        }
    }

    /// Read every row's approval and notes back out of the stores, for the
    /// changes that touch all of them at once.
    fn sync_all(&mut self) {
        let items: BTreeMap<String, BTreeMap<String, ItemState>> = self
            .stores
            .iter()
            .map(|(name, store)| (name.clone(), store.state.items.clone()))
            .collect();
        for change in &mut self.review.changes {
            let Some(items) = items.get(&change.name) else {
                continue;
            };
            for a in &mut change.artefacts {
                let key = a.key();
                a.state = items.get(&key).cloned().unwrap_or_default();
            }
            for cap in &mut change.capabilities {
                for p in &mut cap.pairings {
                    let key = p.key();
                    p.state = items.get(&key).cloned().unwrap_or_default();
                    p.refresh_notes(items);
                }
            }
        }
    }

    /// `a`. On a scenario row this toggles the parent requirement: approval
    /// is per requirement, and a key that silently does nothing is a bug
    /// report waiting to happen.
    fn toggle_approval(&mut self) {
        let Some(row) = self.current_row().cloned() else {
            return;
        };
        match row {
            Row::Requirement { .. } | Row::Scenario { .. } => {
                let Some(anchor) = row.anchor() else { return };
                let (change, key, hash) = {
                    let Some(p) = self.pairing_under(&row) else {
                        return;
                    };
                    (p.change.clone(), p.key(), p.text_hash())
                };
                match self.store_for(&change).toggle_approval(&key, hash) {
                    Ok(_) => self.sync_pairing(anchor, &change),
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            Row::Artefact { change, index } => {
                let (name, key, hash) = {
                    let c = &self.review.changes[change];
                    let a = &c.artefacts[index];
                    (c.name.clone(), a.key(), a.text_hash())
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

    /// The text of the note on the row under the cursor.
    pub fn current_note(&self) -> Option<String> {
        let row = self.current_row()?;
        match row {
            Row::Requirement { .. } => self
                .pairing_at(row)
                .and_then(Pairing::note)
                .map(|n| n.text.clone()),
            Row::Scenario { .. } => self
                .scenario_at(row)
                .and_then(|(p, m)| p.scenario_note(m.name()))
                .map(|n| n.text.clone()),
            Row::Artefact { .. } => self
                .artefact_at(row)
                .and_then(|a| a.state.note.as_ref())
                .map(|n| n.text.clone()),
            _ => None,
        }
    }

    /// Store a note against the row's own anchor, hashed to the text it is
    /// written about. An empty note removes it.
    pub fn set_note_for(&mut self, row: &Row, note: Option<String>) {
        match row {
            Row::Requirement { .. } | Row::Scenario { .. } => {
                let Some(anchor) = row.anchor() else { return };
                let scenario = self.scenario_at(row).map(|(_, m)| m.name().to_string());
                let (change, key, hash) = {
                    let Some(p) = self.pairing_under(row) else {
                        return;
                    };
                    match &scenario {
                        None => (p.change.clone(), p.key(), p.anchor_hash(None)),
                        Some(s) => (
                            p.change.clone(),
                            p.scenario_key(s),
                            p.anchor_hash(Some(s.as_str())),
                        ),
                    }
                };
                match self.store_for(&change).set_note(&key, note, hash) {
                    Ok(_) => self.sync_pairing(anchor, &change),
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            Row::Artefact { change, index } => {
                let (name, key, hash) = {
                    let c = &self.review.changes[*change];
                    let a = &c.artefacts[*index];
                    (c.name.clone(), a.key(), a.text_hash())
                };
                let result = self.store_for(&name).set_note(&key, note, hash);
                match result {
                    Ok(state) => self.review.changes[*change].artefacts[*index].state = state,
                    Err(e) => self.message = Some(e.to_string()),
                }
            }
            _ => {}
        }
    }

    pub fn set_current_note(&mut self, note: Option<String>) {
        let Some(row) = self.current_row().cloned() else {
            return;
        };
        self.set_note_for(&row, note);
    }

    /// `e`: open the popup on the row under the cursor, quoting what the
    /// note is about so the text stays legible while it is written.
    fn open_note(&mut self) {
        let Some(row) = self.current_row().cloned() else {
            return;
        };
        let (title, quote) = match &row {
            Row::Requirement { .. } => {
                let Some(p) = self.pairing_at(&row) else {
                    return;
                };
                let body = p
                    .after
                    .as_ref()
                    .or(p.before.as_ref())
                    .map(|r| crate::review::normalize::paragraphs(&r.body))
                    .unwrap_or_default();
                (format!("Requirement: {}", p.name), body)
            }
            Row::Scenario { .. } => {
                let Some((p, m)) = self.scenario_at(&row) else {
                    return;
                };
                let body = p
                    .scenario_text(m.name())
                    .map(|t| t.lines().skip(1).map(str::to_string).collect())
                    .unwrap_or_default();
                (format!("Scenario: {}", m.name()), body)
            }
            Row::Artefact { .. } => {
                let Some(a) = self.artefact_at(&row) else {
                    return;
                };
                (a.artefact.name.clone(), Vec::new())
            }
            _ => return,
        };
        let buffer = self.current_note().unwrap_or_default();
        self.focus = Focus::Modal(Modal::Note(NoteEdit {
            row,
            title,
            quote,
            buffer,
        }));
    }

    pub fn note_edit(&self) -> Option<&NoteEdit> {
        match &self.focus {
            Focus::Modal(Modal::Note(edit)) => Some(edit),
            _ => None,
        }
    }

    /// What `$EDITOR` saved, back into the popup. The buffer holds no line
    /// breaks, so the editor's lines come back joined.
    pub fn set_note_buffer(&mut self, text: &str) {
        if let Focus::Modal(Modal::Note(edit)) = &mut self.focus {
            edit.buffer = text.split_whitespace().collect::<Vec<_>>().join(" ");
        }
    }

    fn save_note(&mut self) {
        let Some(edit) = self.note_edit() else { return };
        let row = edit.row.clone();
        let text = edit.buffer.trim().to_string();
        self.focus = Focus::Browsing;
        self.set_note_for(&row, (!text.is_empty()).then_some(text));
    }

    fn open_history(&mut self) {
        if let Some(p) = self.current_pairing() {
            let entries = history_entries(p).len();
            self.focus = Focus::Modal(Modal::History(HistoryState {
                selected: entries.saturating_sub(1),
                mode: HistoryMode::Version,
                scroll: 0,
            }));
        }
    }

    pub fn history_state(&self) -> Option<&HistoryState> {
        match &self.focus {
            Focus::Modal(Modal::History(h)) => Some(h),
            _ => None,
        }
    }

    /// The right pane of the history view for the selected entry.
    pub fn history_lines(&self) -> Vec<DiffLine> {
        let (Some(p), Some(h)) = (self.current_pairing(), self.history_state()) else {
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

    /// `N`: every note of the review, artefacts first and then the main
    /// list's order. Derived on the spot rather than cached, so deleting a
    /// note cannot leave a stale row behind.
    pub fn note_rows(&self) -> Vec<NoteRow> {
        let multi = self.review.changes.len() > 1;
        all_rows(&self.review)
            .into_iter()
            .filter_map(|row| self.note_row(row, multi))
            .collect()
    }

    fn note_row(&self, row: Row, multi: bool) -> Option<NoteRow> {
        let (anchor, text, outdated) = match &row {
            Row::Artefact { .. } => {
                let a = self.artefact_at(&row)?;
                let note = a.state.note.as_ref()?;
                (
                    a.artefact.name.clone(),
                    note.text.clone(),
                    a.note_outdated(),
                )
            }
            Row::Requirement { .. } => {
                let p = self.pairing_at(&row)?;
                let note = p.note()?;
                (
                    format!("{} § {}", p.capability, p.name),
                    note.text.clone(),
                    note.outdated,
                )
            }
            Row::Scenario { .. } => {
                let (p, m) = self.scenario_at(&row)?;
                let note = p.scenario_note(m.name())?;
                (
                    format!("{} § {} › {}", p.capability, p.name, m.name()),
                    note.text.clone(),
                    note.outdated,
                )
            }
            _ => return None,
        };
        let label = match (multi, self.change_name_of(&row)) {
            (true, Some(change)) => format!("{change} · {anchor}"),
            _ => anchor,
        };
        Some(NoteRow {
            row,
            label,
            first_line: text.lines().next().unwrap_or_default().to_string(),
            outdated,
        })
    }

    fn change_name_of(&self, row: &Row) -> Option<String> {
        let index = match *row {
            Row::Change { change }
            | Row::Artefact { change, .. }
            | Row::Capability { change, .. }
            | Row::Requirement { change, .. }
            | Row::Scenario { change, .. } => change,
        };
        self.review.changes.get(index).map(|c| c.name.clone())
    }

    pub fn notes_state(&self) -> Option<&NotesState> {
        match &self.focus {
            Focus::Modal(Modal::Notes(state)) => Some(state),
            _ => None,
        }
    }

    fn notes_state_mut(&mut self) -> Option<&mut NotesState> {
        match &mut self.focus {
            Focus::Modal(Modal::Notes(state)) => Some(state),
            _ => None,
        }
    }

    /// `d`: remove the selected note, as saving it empty would. The panel
    /// stays open on the note that followed, or on the last one.
    fn delete_selected_note(&mut self, rows: &[NoteRow], selected: usize) {
        let Some(target) = rows.get(selected).map(|r| r.row.clone()) else {
            return;
        };
        self.set_note_for(&target, None);
        let remaining = self.note_rows().len();
        if let Some(state) = self.notes_state_mut() {
            state.selected = selected.min(remaining.saturating_sub(1));
        }
    }

    /// `y` to the clear confirmation. Every store the review reads is
    /// cleared, since the panel lists every one of their notes; a store
    /// that refuses to write says so in the status line and the rest are
    /// still cleared.
    fn clear_notes(&mut self) {
        let names: Vec<String> = self.stores.keys().cloned().collect();
        for name in names {
            if let Err(e) = self.store_for(&name).clear_notes() {
                self.message = Some(e.to_string());
            }
        }
        self.sync_all();
    }

    fn handle_notes_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let rows = self.note_rows();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let state = self.notes_state()?;
        if state.confirming {
            if matches!(key.code, KeyCode::Char('y')) {
                self.clear_notes();
            }
            if let Some(state) = self.notes_state_mut() {
                state.confirming = false;
                state.selected = 0;
            }
            return None;
        }
        let selected = state.selected;
        let last = rows.len().saturating_sub(1);
        match (key.code, ctrl) {
            (KeyCode::Esc, _) | (KeyCode::Char('N'), false) => self.focus = Focus::Browsing,
            (KeyCode::Char('q'), false) | (KeyCode::Char('c'), true) => self.quit = true,
            (KeyCode::Char('j'), false) | (KeyCode::Down, false) => {
                if let Some(state) = self.notes_state_mut() {
                    state.selected = (selected + 1).min(last);
                }
            }
            (KeyCode::Char('k'), false) | (KeyCode::Up, false) => {
                if let Some(state) = self.notes_state_mut() {
                    state.selected = selected.saturating_sub(1);
                }
            }
            (KeyCode::Enter, _) => {
                if let Some(target) = rows.get(selected).map(|r| r.row.clone()) {
                    self.focus = Focus::Browsing;
                    self.pane = Pane::List;
                    self.jump_to(&target);
                }
            }
            (KeyCode::Char('d'), false) => self.delete_selected_note(&rows, selected),
            (KeyCode::Char('X'), false) if !rows.is_empty() => {
                if let Some(state) = self.notes_state_mut() {
                    state.confirming = true;
                }
            }
            _ => {}
        }
        None
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Effect> {
        self.message = None;
        match &self.focus {
            Focus::Transient(_) => {
                self.focus = Focus::Browsing;
                None
            }
            Focus::Modal(Modal::History(_)) => self.handle_history_key(key),
            Focus::Modal(Modal::Note(_)) => self.handle_note_key(key),
            Focus::Modal(Modal::Notes(_)) => self.handle_notes_key(key),
            Focus::Modal(Modal::Quit(_)) => self.handle_quit_key(key),
            Focus::Browsing => self.handle_browsing_key(key),
        }
    }

    fn handle_browsing_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            (KeyCode::Char('c'), true) | (KeyCode::Char('q'), false) | (KeyCode::Esc, false) => {
                self.quit_or_prompt()
            }
            (KeyCode::Char('j'), false) | (KeyCode::Down, false) => {
                if self.pane == Pane::Detail {
                    self.scroll_by(1);
                } else {
                    self.move_cursor(true);
                }
            }
            (KeyCode::Char('k'), false) | (KeyCode::Up, false) => {
                if self.pane == Pane::Detail {
                    self.scroll_by(-1);
                } else {
                    self.move_cursor(false);
                }
            }
            (KeyCode::Tab, _) | (KeyCode::BackTab, _) => {
                self.pane = match self.pane {
                    Pane::List => Pane::Detail,
                    Pane::Detail => Pane::List,
                }
            }
            (KeyCode::Char('n'), false) => self.jump_finding(true),
            (KeyCode::Char('p'), false) => self.jump_finding(false),
            (KeyCode::Char(' '), false) => self.toggle_fold(),
            (KeyCode::Char('a'), false) => self.toggle_approval(),
            (KeyCode::Char('e'), false) => self.open_note(),
            (KeyCode::Char('H'), false) => self.open_history(),
            (KeyCode::Char('N'), false) => {
                self.focus = Focus::Modal(Modal::Notes(NotesState::default()))
            }
            (KeyCode::Char('m'), false) => self.mode = self.mode.next(),
            (KeyCode::Char('D'), false) => self.toggle_definitions(),
            (KeyCode::Char('?'), false) => self.focus = Focus::Transient(Transient::Help),
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

    /// The notes this review would post, if it has anywhere to post them.
    pub fn unposted_notes(&self) -> Option<crate::review::post::ReviewPost> {
        self.review.pull_request.as_ref()?;
        crate::review::post::plan(&self.review, &self.stores)
    }

    /// A quit key with unposted notes on a pull request asks first; with
    /// nothing to post it quits, which is every other source's only path.
    fn quit_or_prompt(&mut self) {
        match (self.review.pull_request.as_ref(), self.unposted_notes()) {
            (Some(pr), Some(post)) => {
                self.focus = Focus::Modal(Modal::Quit(QuitPrompt {
                    notes: post.len(),
                    pull_request: pr.number,
                }))
            }
            _ => self.quit = true,
        }
    }

    pub fn quit_prompt(&self) -> Option<&QuitPrompt> {
        match &self.focus {
            Focus::Modal(Modal::Quit(prompt)) => Some(prompt),
            _ => None,
        }
    }

    /// The prompt asked twice already, so the interrupt key is a decline
    /// rather than a third question.
    fn handle_quit_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            (KeyCode::Char('y'), false) => return Some(Effect::PostNotes),
            (KeyCode::Char('n'), false) | (KeyCode::Char('c'), true) => self.quit = true,
            (KeyCode::Esc, _) => self.focus = Focus::Browsing,
            _ => {}
        }
        None
    }

    /// Every key belongs to the popup: `q` types a `q`, and `Esc` cancels
    /// rather than quitting.
    fn handle_note_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            (KeyCode::Char('e'), true) => return Some(Effect::EscalateNote),
            (KeyCode::Esc, _) => self.focus = Focus::Browsing,
            (KeyCode::Enter, _) => self.save_note(),
            (KeyCode::Backspace, _) => {
                if let Focus::Modal(Modal::Note(edit)) = &mut self.focus {
                    edit.buffer.pop();
                }
            }
            (KeyCode::Char(c), false) => {
                if let Focus::Modal(Modal::Note(edit)) = &mut self.focus {
                    edit.buffer.push(c);
                }
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
        let Focus::Modal(Modal::History(h)) = &mut self.focus else {
            return None;
        };
        match (key.code, ctrl) {
            (KeyCode::Esc, _) | (KeyCode::Char('H'), false) => self.focus = Focus::Browsing,
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
            (KeyCode::Char('?'), false) => self.focus = Focus::Transient(Transient::Help),
            _ => {}
        }
        None
    }

    pub fn finding_counts(&self) -> (usize, usize, usize) {
        let s = &self.review.summary;
        (s.errors, s.warnings, s.notes)
    }

    /// The diff of the row under the cursor, computed once per selection.
    /// The drawer calls `refresh_detail` before it reads this.
    pub fn detail_lines(&self) -> &[DiffLine] {
        &self.detail.lines
    }

    /// Recompute the cached diff when the cursor has moved to another row.
    pub fn refresh_detail(&mut self) {
        let row = self.current_row().cloned();
        if self.detail.row == row {
            return;
        }
        self.detail = DetailCache {
            lines: row.as_ref().map(|r| self.diff_of(r)).unwrap_or_default(),
            row,
        };
    }

    fn diff_of(&self, row: &Row) -> Vec<DiffLine> {
        if let Some((_, m)) = self.scenario_at(row) {
            return scenario_view(m);
        }
        match (self.pairing_at(row), self.artefact_at(row)) {
            (Some(p), _) => inline_view(p),
            (_, Some(a)) => a.lines(),
            _ => Vec::new(),
        }
    }
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
