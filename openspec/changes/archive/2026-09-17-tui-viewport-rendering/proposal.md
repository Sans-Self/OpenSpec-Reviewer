# tui-viewport-rendering

## Why

The view stalls on a change with a large spec. Every frame, and the
event loop draws a frame every 250 ms whether or not anything happened,
rebuilds the whole detail pane from scratch: it recompiles the
glossary's matchers, recomputes the semantic diff of the selected
pairing or the line diff of the selected artefact, and runs every
matcher over every line of the result, including the lines that are
scrolled off the screen. Underneath, `Grammar::blank` compiles two
regexes on each call, and the matchers call it once per line each. A
frame over a 2,000-line artefact with twenty glossary names is tens of
thousands of regex compilations, four times a second, while idle.

## What Changes

- The regexes of the citation grammar compile once per process.
- The glossary's compiled matchers are built once when the view opens
  and reused for every frame; a line is blanked once and read by every
  matcher, not blanked once per matcher.
- The diff the detail pane shows is computed when the selection changes
  and kept until it changes again.
- The detail pane styles only the lines that can reach the screen. Lines
  above the scroll offset become empty placeholders that keep the offset
  honest; lines below the last visible row are not built. With wrapping
  on, no line above the offset is skipped, because a wrapped line only
  grows.
- The event loop redraws after a key or a resize, not on every poll
  timeout.

## Capabilities

### Modified Capabilities

- `review-tui`: one new requirement, "Rendering cost follows the screen,
  not the spec", stating the cost bound and that scrolling past a long
  diff still reaches the findings under it.

## Impact

- `src/citations/grammar.rs`: the two grammar regexes become statics.
- `src/glossary/synonyms.rs`, `src/glossary/marks.rs`: `Marks` owns its
  terms so the view can hold it; `occurrences` blanks once.
- `src/render/tui/app.rs`: `App` holds the compiled marks and the
  cached detail lines.
- `src/render/tui/ui.rs`: a viewport window around the styling of the
  detail and history panes.
- `src/render/tui/mod.rs`: the dirty flag in the event loop.
- No new dependency.

## Non-goals

- Windowing the list pane. Its rows are one line each and ratatui's
  `List` already clips them; the cost there is linear in rows, not in
  spec text.
- Incremental diffing. The diff runs once per selection, which is
  enough.
- Any change to what the panes show.
