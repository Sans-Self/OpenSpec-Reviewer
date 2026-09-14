# Tasks: citation-prose

## 1. Grammar

- [x] 1.1 Backticked literal pass across line breaks, then the line-bound
      pass skipping recorded ranges; tests for a wrapped span and for a
      span with no closing backtick.
- [x] 1.3 Continuation lines inside a span lose a leading comment marker
      (`#`, `//`, `///`, `//!`, `*`, `--`); tests for `#` and `*`.
- [x] 1.2 `normalize_name` strips trailing `.`, `,`, `;`, `:`; applied to
      canon headings and citations.

## 2. Hint

- [x] 2.1 `Sighting` carries `ends_at_line_break`; the dangling error
      appends the wrap hint when set.

## 3. Wrap-up

- [x] 3.1 Tests titled by requirement for the four new scenarios.
- [x] 3.2 README Citations section mentions that backticked citations
      may wrap.
