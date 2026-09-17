# Design: scenario-notes

## Context

Three structures already carry the scenario, and one does not.
`Location` has `scenario: Option<String>`. `ScenarioMatch` is a four-way
union over the matched pair. The plain renderer prints
`capability § requirement # scenario`. The state store is
`BTreeMap<String, ItemState>` keyed `capability/requirement`.

## Goals / Non-Goals

**Goals:** a note that anchors where the objection is; a note that is
cheap enough to write that the anchor is worth having.

**Non-Goals:** threads, note kinds, per-scenario approval, a second word
for a note. Accessibility of the popup is deferred, not dropped.

## Decisions

**The key gains a scenario, with the separator already in use.**
`capability/requirement#scenario`. Plain output and the lint already
write `#` before a scenario name, so the store reads the way the output
reads. A note on the requirement keeps today's key exactly, so existing
state files keep resolving.

**Approval stays per requirement.** The `text_hash` is over the whole
normalized after text, `approved/total` counts requirements and
artefacts, and a requirement is the unit a reviewer signs off. Scenario
rows therefore show no mark, and the blank column says so without a
legend. `a` on a scenario row toggles the parent rather than doing
nothing, because a key that silently no-ops is a bug report waiting to
happen, and the parent row is on screen directly above with its mark
flipping.

This leaves two hash scopes on purpose: an approval hashes the
requirement, a note hashes its own anchor. Editing one scenario moves
the requirement hash, so the parent goes `[~]` and the note on that
scenario reads as written about text that has changed. Both are true and
they are about different questions.

**Rows fold, and jumping expands.** Twenty requirements with three
scenarios each is eighty rows. Scenario rows are folded when the view
opens — always, rather than by some rule about which scenarios changed,
because a predictable list is worth more than a clever one. `Space`
toggles the fold. `n` and `p` may land on a scenario row inside a folded
requirement, so jumping expands what it jumps into: the cursor has to be
able to reach where it is going.

A folded requirement shows `✎` when it or any of its scenarios holds a
note. The marker stays one character and stays a boolean. Its job is
"there is something here", and expanding is how you find out where; a
count would widen a column that plain output also prints.

**The popup carries its quote.** It is drawn over the detail pane, so
depending on what is behind it would be circular. Quoting the target
inside the popup makes placement, scrolling and the fold state all
irrelevant to whether the reviewer can read what they are writing about.

**One buffer, no line model.** The popup holds a flat `String` that
wraps at render width. `⏎` saves, `Esc` cancels, `^E` suspends the view
and opens `$EDITOR` seeded with the buffer. There is no line array to
fall out of step with the text, and the foundation design's refusal to
build a multi-line editor in ratatui stands: anything that wants line
structure leaves for the real editor and comes back.

**Focus becomes a union.** `Esc` quits today, and inside the popup it
has to cancel instead; `q` has to type a `q`. So the view holds
`Browsing`, `Transient(Help)` or `Modal(Note { .. })` rather than an
editing flag, and only one modal is open at a time — a flat enum, not a
stack. The help overlay is transient because any key closes it; the
popup is modal because it swallows keys. That difference is why one
overlay concept cannot serve both.

**A note records what it was written about.** `Note { text, text_hash,
at }`, mirroring `Approval`. A note whose anchor has since changed is
not wrong, but the reviewer needs to know the words moved, which is what
GitHub calls an outdated review comment. It is reported in the detail
pane rather than in a row marker: the mark column belongs to approval,
and `✎` already means what it means.

Old state files hold `note` as a bare string. That parses as a note with
no hash, which reads as never outdated until it is next edited. No
migration step, no format version.
