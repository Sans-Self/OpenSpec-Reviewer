# Tasks: tui-viewport-rendering

## 1. Compile once

- [x] 1.1 `Grammar`'s literal and span regexes are `LazyLock` statics
      used by `new`, `strip` and `blank`.
- [x] 1.2 `Marks` owns its terms; `Glossary::marks` returns it without a
      borrow of the glossary; `occurrences` blanks the text once and
      matches through `Matcher::ranges_cleaned`.
- [x] 1.3 `App` holds the compiled `Marks` from `App::new` and the
      detail lines of the row last drawn, refreshed when the row changes.

## 2. Window the panes

- [x] 2.1 A `Viewport` in `render/tui/ui.rs` maps a line index to a
      placeholder, a styled line or nothing; the inline, side-by-side
      and raw views and the history pane style through it.
- [x] 2.2 The findings, notes and history summary under the diff are
      appended only when the diff ends inside the window, and are
      reachable by scrolling in every mode.

## 3. Draw on change

- [x] 3.1 The event loop draws once, then only after a key or resize
      event.

## 4. Wrap-up

- [ ] 4.1 Tests titled by requirement: the trailer is visible after
      scrolling past a long diff in each mode, and a frame over a large
      artefact with a glossary finishes inside the budget.
- [ ] 4.2 `clippy` and `rustfmt` clean.
