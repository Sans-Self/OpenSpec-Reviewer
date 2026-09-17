# pane-wrapping

## Why

Two of the three display modes throw text away. `side_by_side` and the
raw mode pass every line through `fit`, which takes the first `half`
characters and pads to width. Anything past that column is gone: no
ellipsis, no horizontal scroll, no indication it was ever there. On an
eighty-column terminal `half` is thirty-eight, which is narrower than
most paragraphs in this repository's own specs, so the side-by-side view
of a real requirement is mostly invisible.

`fit` also counts `char`s where the terminal counts columns. A CJK
character takes two cells and a combining mark takes none, so a line
holding either misaligns the gutter and overflows the pane.

Inline mode does wrap, by handing `Wrap` to ratatui, and that buys three
smaller problems. A wrapped line's continuation starts at column zero,
under the `+` or `-` glyph rather than beside it, so the second row of
an added paragraph is indistinguishable from an unchanged one. The
scroll offset counts source lines while the widget scrolls rendered
lines, so `Ctrl-d` moves a different distance depending on how much the
text wrapped. And `scroll_by` only saturates at `u16::MAX`, so scrolling
past the end walks into blank space with no way to tell how far.

The list pane truncates long requirement names at the border with
nothing to say it did.

## What Changes

- Wrapping happens in the view, not in the widget. Each display mode
  produces the lines it will actually draw, so the scroll offset, the
  glyph gutter and the side-by-side alignment all agree on what a line
  is.
- Side-by-side and raw wrap within their column instead of truncating.
  A pair of sides wraps to the taller of the two, and the shorter side
  pads.
- A wrapped line's continuation rows carry a blank gutter the width of
  the glyph, so the text stays in one column and only the first row is
  marked.
- Width is measured in display columns, so wide characters and
  combining marks keep the columns aligned.
- The detail scroll offset is clamped to the rendered height, so the
  last line cannot scroll off the top.
- A list row too long for the pane ends in `…`.

## Capabilities

### Modified Capabilities

- `review-tui`: the display modes wrap rather than truncate; the detail
  pane's scroll is bounded; a new requirement for how a line wraps and
  how width is measured.

## Impact

- `src/render/tui/ui.rs`: `fit` is replaced by a wrap that returns rows;
  `side_by_side` and `two_columns` wrap both sides; `Paragraph::wrap` is
  no longer used for the detail pane.
- `src/render/tui/app.rs`: `scroll_by` clamps against the rendered line
  count, which the view now reports alongside `detail_height`.
- New direct dependency on `unicode-width`. It is already in the lock
  file through ratatui, which measures cells with it; taking it directly
  means the view measures the way the terminal does rather than the way
  `char` counting guesses.

## Non-goals

- Horizontal scrolling. Wrapping is the answer to a long line.
- Reflowing the raw mode's text. Raw shows the snapshot as-is, so its
  own line breaks stay and only overlong lines wrap.
- Wrapping in plain and markdown output, which is piped and wraps where
  the reader's pager says.
