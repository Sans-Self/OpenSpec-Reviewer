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

Depth is a function of the `Row` variant: `Change` 0, `Artefact` and
`Capability` 1, `Requirement` 2, `Scenario` 3, all lowered by one when
the snapshot holds a single change and no `Change` rows exist. `guides`
takes `multi` so it does not have to look at the review.

## The fold marker sits in the row, not the guide

`▸` and `▾` say what `Space` will do, so they belong to the requirement's
own cells, right after the connector and before the approval mark. A
requirement with no scenarios draws a space there. The marker comes from
`App.unfolded` and the scenario count on the pairing; both are already
at hand in `row_line`. Scenario rows carry none.

## Style

The guide prefix is one `Span` in `palette.muted()`. The rest of the row
draws exactly as today. Without colour the guides are plain glyphs and
the muted style falls back to dim or nothing, which satisfies "Colour is
never the only signal" with no extra rule.

## Testing

`tree::guides` is tested against small row vectors: a single change with
two capabilities, a folded and an unfolded requirement, and two changes.
The row rendering is tested through `TestBackend` on the fold marker
flipping with `Space`. Test titles quote the requirement.
