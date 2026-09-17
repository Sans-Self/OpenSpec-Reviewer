# Design: notes-panel

## Context

The view has one modal at a time, a flat `Focus` enum with `History`
and `Note` modals and a `Help` transient. Notes live in the store keyed
by `<capability>/<requirement>`, `<capability>/<requirement>#<scenario>`
or an artefact's name, and every pairing already carries its notes in
`Pairing::notes` with an `outdated` flag, filled by the caller that
owns the store. The list pane's rows are built by `visible_rows` and
can be rebuilt with a different fold set.

## Goals / Non-Goals

**Goals:** every note of the change on one screen; one key from a note
to its row; removing notes without opening each.

**Non-Goals:** editing inside the panel; touching approvals; a panel
across changes.

## Decisions

**A modal, like history.** The panel is `Modal::Notes(NotesState {
selected })`. It swallows keys until it closes, which is what a list
you navigate needs, and it reuses the shape `History` already has: a
state struct with a selection, a drawer that reads it, a key handler
that moves it.

**Rows are derived, not stored.** The panel's rows are computed from
the review each draw: every artefact with a note, then every pairing's
notes in the order the main list shows them. A row holds the `Row` it
belongs to in the main list, the anchor label, the first line of the
text and the outdated flag. Nothing is cached, because deleting a note
changes the list and a derived list is right by construction. There
are never enough notes for this to cost anything.

**Anchor labels read as the list does.** A requirement note is
`<capability> § <requirement>`, a scenario note adds ` › <scenario>`,
an artefact note is the artefact's name. The same words appear in
plain output and in findings, so the panel introduces no vocabulary.

**`Enter` reuses the jump.** Jumping to a row is what `n` and `p`
already do, including unfolding a folded requirement to land on a
scenario. `Enter` closes the modal and calls the same move with the
row's anchor.

**`d` deletes one, `X` asks first.** Deleting the selected note is
`set_note(key, None)`, the existing path for an empty note, so the
detail pane, the marker and the file all follow. Clearing is
`Store::clear_notes`, which drops the `note` field of every item and
removes items left empty, then saves once. Clearing is the one
destructive key in the view, so it opens a `y`/`n` confirmation naming
the count; `y` clears, any other key returns to the panel. A panel
emptied by `d` or `X` stays open and reads "no notes".

**Panel over both panes.** The panel takes the area of both panes,
because a note's first line needs the width and the main list is not
useful behind it. The status line stays.

## Risks / Trade-offs

- [A note deleted by mistake] → `d` acts without asking, since one note
  is quick to rewrite and a prompt on every deletion would make the
  panel slower than the detail pane it replaces. `X` asks.
- [Two changes in one review] → rows carry their change and the label
  is prefixed with the change name when the review has more than one,
  the rule the main list already follows.
