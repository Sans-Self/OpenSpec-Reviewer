# tui-tree-view

## Why

The list pane is a tree drawn as an indented list. Artefacts, capability
headings, requirements and scenarios sit at four indents of plain spaces,
so on a long change the eye loses which capability a requirement belongs
to and where one requirement's scenarios end and the next begins. A
folded requirement looks the same as one with no scenarios at all, so the
reviewer presses `Space` to find out.

## What Changes

- The list draws tree guides: `├─` before a child, `└─` before the last
  child, and `│` continuing down the column of every open ancestor. The
  guides take the muted style and are glyphs, so they read without colour.
- A requirement row with scenarios carries a fold marker before its
  approval mark: `▸` folded, `▾` unfolded. A requirement without
  scenarios carries a space there, so the marks stay aligned.
- The tree hangs from the change: artefacts and capabilities are its
  children, requirements theirs, scenarios theirs. With one change the
  root is implicit and the artefacts start with a connector; with
  several, each change heading is a bare root.
- The cursor opens the requirement it rests on. Its scenarios appear
  when the cursor lands on it and fold when the cursor leaves, unless
  `Space` pinned them open. The marker shows `▾` for either reason.

## Capabilities

### Modified Capabilities

- `review-tui`: "The view is a list and a detail pane" gains the guides,
  the fold marker and the open-on-cursor rule.

## Impact

- New `src/render/tui/tree.rs`: a pure function from the visible rows to
  one guide prefix per row, tested on its own.
- `src/render/tui/ui.rs` prepends the prefix in `row_line` instead of the
  fixed indents.
- README: one sentence on the fold marker.

## Non-goals

- Folding capabilities or artefacts. `Space` keeps its one meaning,
  now a pin.
- A configurable guide set. Box drawing is what the borders already use.
