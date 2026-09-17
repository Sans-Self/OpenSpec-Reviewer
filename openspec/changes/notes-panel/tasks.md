# Tasks: notes-panel

## 1. State

- [x] 1.1 `Store::clear_notes() -> Result<usize, StateError>` drops
      every note, removes emptied items, saves once and returns the
      count.

## 2. The panel

- [x] 2.1 `Modal::Notes(NotesState { selected })` and
      `App::note_rows()` deriving the rows from artefacts and pairings
      in list order, with anchor label, first line, outdated flag and
      the main-list `Row`.
- [x] 2.2 `N` opens the panel from browsing; `j`/`k` and arrows move;
      `Esc` closes; `Enter` closes and jumps to the row, unfolding when
      the note is on a scenario.
- [x] 2.3 `d` deletes the selected note through `set_note_for`; the
      selection clamps; an empty panel reads "no notes".
- [x] 2.4 `X` opens a confirmation naming the count; `y` clears through
      `Store::clear_notes` and resyncs every pairing and artefact; any
      other key returns to the panel.
- [x] 2.5 Drawing: the panel over both panes with a title naming the
      count, the confirmation as a centred box; `BINDINGS` lists `N`
      and the panel keys.

## 3. Tests and docs

- [x] 3.1 Tests titled by requirement: the panel lists requirement,
      scenario and artefact notes in list order with the outdated mark;
      `Enter` lands on a folded scenario; `d` removes one and the file
      follows; `X` then `y` empties the file's notes and keeps its
      approvals; `X` then `n` keeps everything; `N` is in the help.
- [x] 3.2 README: the panel and the two removal keys.
- [x] 3.3 `clippy` and `rustfmt` clean.
