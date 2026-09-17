# Design: tui-viewport-rendering

## Context

`ui::draw` is a pure function of `&App`: it builds every widget from the
review each frame and hands them to ratatui, which clips to the screen.
That shape is right for a small review and the point of the change is to
keep it. The cost is not in ratatui's clipping but in building the
`Text` it clips: glossary matching per line, and per frame the diff and
the matchers themselves.

## Decisions

**Compile once, at the edges of change.** Three things were rebuilt per
frame that change less often than that. The grammar regexes never
change: statics. The glossary does not change during a session: `App`
builds `Marks` once in `App::new`. The diff of a pairing or artefact
changes only when the selection does: `App` keeps the lines of the row
it last computed and rebuilds them when `draw` finds the row has moved.
`Marks` therefore owns clones of its terms instead of borrowing the
glossary, because a struct borrowing a sibling field cannot live in the
same struct; a few cloned terms are cheaper than a frame of regexes.

**Blank once per text.** `Matcher::ranges` blanks citations out of the
text before matching so a `spec:` tag is never marked as a term. Every
matcher of a `Marks` did that to the same line. `occurrences` blanks
once and hands the cleaned text to each matcher through
`ranges_cleaned`; `ranges` stays for callers with one matcher.

**Window the styling, not the text.** The detail pane keeps passing a
`Paragraph` with a scroll offset, so ratatui's model of the text is
unchanged. What changes is which lines are built with styling. A
`Viewport` of `first..end` rows, `end = scroll + height`, maps each line
index to one of three outcomes: an empty placeholder below `first`, a
styled line inside the window, nothing at or beyond `end`. A line at
index at or beyond `end` starts at a row at or beyond `end` in every
mode, so dropping it removes nothing visible. With wrapping on, a line
above the offset may occupy several rows and push later lines down, so
`first` is zero there and only the tail is cut. The findings, notes and
history summary under the diff are appended only when the diff ends
before `end`; otherwise they are beyond the window too.

**Draw on change.** The loop polls at 250 ms so a `SIGINT` is noticed,
and drew on every timeout because the draw sat above the poll. It now
draws once, then again after each key or resize. Nothing in the view
moves on its own, so a frame with no event behind it is identical to the
previous one.

## Risks

- A future line kind whose wrapped height can shrink below one row
  would break the window's invariant. ratatui has none.
- The detail cache is keyed on the `Row`, which indexes the review.
  Rows are rebuilt on fold and unfold but their indexes stay valid, so
  the key stays correct.
