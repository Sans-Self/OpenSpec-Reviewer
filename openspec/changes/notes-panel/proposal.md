# notes-panel

## Why

Notes are visible one row at a time, in the detail pane of the row they
hang on. A reviewer who has written eight notes across a change has no
way to read them together, find the one to revisit, or see which are
stale. And there is no way to remove notes except opening each and
saving it empty, so the state file under `~/.local/state` keeps every
note of every review until it is deleted by hand.

## What Changes

- `N` opens a notes panel: a modal listing every note of the change,
  requirement notes, scenario notes and artefact notes, in list order,
  each with its anchor, its first line, and a mark when the text it was
  written about has changed.
- In the panel `j`/`k` move, `Enter` jumps to the note's row in the main
  list, unfolding a requirement when the note is on one of its
  scenarios, `d` deletes the selected note, `X` clears every note of the
  change after a `y`/`n` confirmation, `Esc` closes.
- Deleting and clearing touch the local state file only. Approvals
  stay. The file stays.
- The help overlay lists the new bindings.

## Capabilities

### New Capabilities

- `notes-panel`: the panel, its rows, its keys, deletion and clearing.

### Modified Capabilities

- `review-tui`: "Keys follow vi and arrow conventions" binds `N` to the
  notes panel.

## Impact

- `src/render/tui/app.rs`: `Modal::Notes(NotesState)`, the collected
  note rows, the key handling, `Transient::ConfirmClear`.
- `src/render/tui/ui.rs`: drawing the panel and the confirmation.
- `src/state/mod.rs`: `Store::clear_notes`.
- `README.md`: the panel under the notes section.
- No new dependency.

## Non-goals

- Editing a note from the panel. `Enter` takes the reviewer to the row,
  where `e` already edits.
- Clearing approvals. An approval is undone with `a` on its row.
- Notes of other changes. The panel is the change under review.
