# Design: tui-tree-view

## Guides are derived from the visible rows, not stored

A row's prefix depends on whether it is the last of its siblings and on
whether each ancestor is the last of its own siblings. Both facts change
when a requirement folds or unfolds, so the prefixes are recomputed from
`App.rows` after every fold, not carried on the `Row`. `tree::guides(rows,
multi) -> Vec<String>` does it in one pass: a row is the last sibling when
no later row at the same depth comes before a row at a shallower depth.
Its prefix is, for every ancestor from the root down, `│  ` when that
ancestor has later siblings and three spaces when not, then its own
connector `├─ ` or `└─ `. Roots draw no connector.

Depth is a function of the `Row` variant alone: `Change` 0, `Artefact`
and `Capability` 1, `Requirement` 2, `Scenario` 3. With a single change
no `Change` row exists and the artefacts hang from an implicit root, so
they still draw a connector; only depth 0 draws none.

## Hover is derived, pins are stored

`App.unfolded` keeps only what `Space` pinned. The requirement the
cursor is on or inside is open by virtue of the cursor, so
`visible_rows` takes the pinned set and the cursor's anchor and shows a
scenario when either admits it. Every cursor move ends in
`rebuild_rows`, which recomputes the rows for the new position and finds
the cursor's row again by identity. Leaving an unpinned requirement
therefore folds it with no bookkeeping, and `n`/`p` landing on a
scenario open its requirement without pinning it.

## The fold marker sits in the row, not the guide

`▸` and `▾` say what `Space` will do, so they belong to the requirement's
own cells, right after the connector and before the approval mark. A
requirement with no scenarios draws a space there. The marker comes from
`App::is_open`, which is the pin or the cursor, and the scenario count
on the pairing. Scenario rows carry none.

## Style

The guide prefix is one `Span` in `palette.muted()`. The rest of the row
draws exactly as today. Without colour the guides are plain glyphs and
the muted style falls back to dim or nothing, which satisfies "Colour is
never the only signal" with no extra rule.

## Testing

`tree::guides` is tested against small row vectors: a single change with
two capabilities, a folded and an unfolded requirement, and two changes.
The row rendering is tested through `TestBackend`: the connectors on
the artefact rows, the `│` column, the fold marker following the
cursor, and pinning with `Space`. Test titles quote the requirement.
