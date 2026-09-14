# Design: citation-prose

## Decisions

**Two literal passes, backticked first.** The grammar runs
`` `spec:([a-z0-9-]+)\s*§\s*([^`]+)` `` over the text and records each
match with its byte range, then runs the existing line-bound literal
over the same text and drops any match that starts inside a recorded
range. The backticked form is tried first because it is the more
specific one; when the closing backtick is missing it simply does not
match and the line-bound form catches what it can. A name captured
across lines is split at newlines, each continuation line loses its
leading whitespace and one leading comment marker matched by
`^(?:#+|/{2,}!?|\*|--)\s*`, and the lines join with a space before the
ordinary whitespace normalization runs. The marker list is fixed and
small: the languages a spec-citing repository is written in all use one
of these, and the grammar still knows nothing about where a comment
begins or ends, only that a line inside a span it already matched cannot
start with one. Bare prose gets none of this, because there is no span
to be inside.

**Punctuation is normalization, not grammar.** `normalize_name` strips
trailing `.`, `,`, `;` and `:` after collapsing whitespace. It runs on
canon headings and citations alike, so a requirement whose heading ends
in a full stop still matches a citation that does too. The characters
are exactly the ones a sentence can end with; a name ending in `?` or
`!` is kept, since those carry meaning.

**The hint is a property of the sighting.** `Sighting` records whether
the literal match ended at a newline. The dangling error appends the
hint when that flag is set and the citation did not resolve. Resolution
itself is unchanged; the hint costs nothing when the wrapped name
happens to be complete.

## Risks

- A backticked span that contains `spec:` but is not a citation, code
  that mentions the tag for instance, now matches across lines where it
  previously stopped at the newline. → The old grammar matched it too,
  only shorter; both dangle. No new false positive, a longer name in the
  message.
