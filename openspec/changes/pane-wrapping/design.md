# Design: pane-wrapping

## Context

`detail_text` builds a `Text` of `Line`s and `draw_detail` hands it to a
`Paragraph`, with `Wrap { trim: false }` in inline mode and no wrap in
the other two. The unwrapped modes pre-fit every column through `fit`,
which is `chars().take(width)` followed by space padding. `app.scroll`
is a `u16` incremented by `scroll_by`, and `app.detail_height` is set
from the pane rectangle each frame.

## Goals / Non-Goals

**Goals:** no text that cannot be reached; a scroll offset that means
the same thing as what is on screen.

**Non-Goals:** horizontal scrolling, reflowing raw text, wrapping the
piped outputs.

## Decisions

**The view wraps, the widget does not.** Delegating to `Wrap` is what
splits the three problems apart: the widget knows the rendered line
count and the app does not, so the app's scroll offset, its half-page
step and its clamp are all computed against a number that is not the
one on screen. Wrapping in `detail_text` gives one line list that is
both what gets drawn and what gets counted. It also makes the gutter
possible, which `Wrap` cannot do at all, and makes side-by-side and
inline the same mechanism at two widths rather than two mechanisms.

The cost is that the view must re-wrap on every resize. It already
rebuilds `detail_text` every frame, so this is work per frame, not new
state.

**Continuation rows carry a blank gutter.** `styled_line` prefixes two
cells for the kind glyph. A continuation that starts at column zero puts
prose under the glyph column, which breaks the alignment the glyph
exists to provide and makes the second row of an added paragraph read as
unchanged. Two spaces, styled as the line is styled, keep the text in
one column. The glyph is not repeated: repeating it would claim each row
is a separate change.

**Columns, not characters.** `unicode-width` measures what the terminal
draws. `char` counting is wrong in both directions — a CJK character
occupies two cells and is counted as one, a combining mark occupies none
and is counted as one — and the error compounds across a padded column,
so the right-hand side of a side-by-side view drifts out of alignment
by however many such characters the left side held. This is a direct
dependency rather than an internal helper because the correct table is
Unicode's and it changes with Unicode versions.

**A pair wraps to the taller side.** Side-by-side wraps each side
independently, then pads the shorter one with blank rows so the gutter
stays a straight line down the middle. Aligning the two sides row by row
would mean wrapping them together, which forces a shared break point
that suits neither.

**Breaks fall on whitespace, and a word longer than the column breaks
anyway.** The alternative is a line that overflows the pane, which is
the bug being fixed. A long identifier or URL is the case that hits
this, and it is better cut at the border than drawn over the neighbour.

**The clamp is against rendered height.** `scroll` may not exceed the
rendered line count less the pane height, so the last line cannot leave
the top of the pane and `Ctrl-d` at the end does nothing visible rather
than scrolling into blank. Content shorter than the pane clamps to zero.
