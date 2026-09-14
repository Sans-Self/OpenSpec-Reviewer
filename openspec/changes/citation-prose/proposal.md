# citation-prose

## Why

Citations live in prose as well as test titles: a moduledoc, a `#`
comment, a paragraph in a spec. Prose wraps at 80 columns and ends
sentences with a full stop, and the grammar treats both as the end of
the name. Opake's Elixir sources produce five dangling citations that
way, every one of them pointing at a requirement that exists.

## What Changes

- A citation opened with a backtick runs to the closing backtick, across
  line breaks; runs of whitespace, newlines included, compare as one
  space. A comment marker at the start of a continuation line (`#`,
  `//`, `///`, `//!`, `*`, `--`) is not part of the name, so a wrapped
  citation in a comment resolves like one in a docstring.
- Trailing sentence punctuation (`.`, `,`, `;`, `:`) is not part of a
  requirement name, on either side of the comparison.
- A dangling citation whose name ends at a line break, and which does
  not resolve, says so: the error adds that the name looks cut off by a
  line break and that wrapping it in backticks lets it continue.

Bare wrapped prose stays an error. Without a closing delimiter there is
no fact that says the next line belongs to the name, and a linter that
guesses produces false negatives.

## Capabilities

### Modified Capabilities

- `citations`: "A citation names a capability and a requirement" gains
  the backtick-span and punctuation rules; "A citation must resolve"
  gains the line-break hint.

## Impact

- `src/citations/grammar.rs`, `src/citations/mod.rs` (normalization),
  `src/citations/scan.rs` (the hint), tests.
