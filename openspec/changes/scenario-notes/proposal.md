# scenario-notes

## Why

A finding can name a scenario. `Location` carries
`scenario: Option<String>`, the deprecated-synonym check fills it, and
plain output renders `capability § requirement # scenario`. A note
cannot: the state store is keyed `capability/requirement` and stops
there. The machine can point at the sentence it objects to; the reviewer
writing about the same sentence has to describe it in prose.

The diff already knows the structure. `ScenarioMatch` is
`Same | Changed | Added | Removed`, each with a name and its diff lines,
computed for every pairing and rendered only inside the detail pane.
Nothing in the model is missing. The list and the store are one level
coarser than everything around them.

Editing is the second half. A note opens `$EDITOR` with the view
suspended, which the foundation design chose because a line editor
"is not worth building for multi-line prose". A note about one THEN line
is not multi-line prose, it is one sentence written five times per
requirement, and suspending the view hides the text being written about.

## What Changes

- A note anchors to a requirement or to one of its scenarios, keyed
  `capability/requirement#scenario`.
- Scenarios become selectable rows under their requirement, folded by
  default, taking their glyph from `ScenarioMatch`.
- Approval stays per requirement. Scenario rows carry no approval mark
  and `a` on one toggles the parent.
- A note is written in a popup that quotes what it is about, so the text
  is legible while it is being written. `^E` escalates the buffer to
  `$EDITOR` for anything longer than the popup suits.
- A note records a hash of the text it was written about, and the detail
  pane says when that text has changed since.

## Capabilities

### Modified Capabilities

- `review-state`: a note anchors to a scenario, records what it was
  written about, and markdown export groups by scenario.
- `review-tui`: scenario rows and folding, the popup and its keys.

## Impact

- `src/state/`: the item key gains a scenario, the note gains a hash and
  a timestamp. Old state files parse: a bare string note is a note with
  no hash.
- `src/render/tui/`: a row kind, a fold, a modal focus state.
- `src/review/`: a `Pairing` carries its notes, resolved from the store
  and anchored, because JSON has to report each note's anchor and
  whether it is outdated, and nothing else reaches a scenario note.
- No change to `model` or `citations`. The data is there.

## Non-goals

- Threads. One note per anchor, as today. The finer anchor is what gives
  several notes per requirement; replies and resolution are a different
  feature for a tool that has one author and a local state file.
- Typed notes. Free text. `reviewer-assist` owns what an agent reads,
  and a taxonomy invented here would pre-empt that design.
- A second word. These are notes, as the store, the `✎` marker and
  `--format markdown` already call them. Coining "comment" beside "note"
  is the drift this tool exists to catch.
- Per-scenario approval. Separable, and the store can grow it later.
- Accessibility of the popup, deferred by agreement. It needs settling
  before implementation: the caret must be the terminal cursor and the
  mode must be announced in text.
