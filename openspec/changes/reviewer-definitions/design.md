# Design: reviewer-definitions

## Shape

```
src/
  glossary/
    mod.rs        Term, Glossary, load from canon + change deltas
    synonyms.rs   `- **Deprecated:**` parsing, whole-word matcher
    checks.rs     the four checks as pure functions over Glossary + canon + pairings
  render/
    tui/defs.rs   the `D` panel
```

`Glossary` is built from canon's `definitions` capability plus any
`definitions` delta in the change under review, so a term added in the
same change counts as defined. Every check is a function from
`(&Glossary, &Canon, &[Pairing]) -> Vec<Finding>`; no I/O.

## Term

```rust
struct Term {
    name: String,            // requirement name, e.g. "module"
    meaning: String,         // body with the Deprecated: line removed
    deprecated: Vec<String>, // ISO 704 deprecated terms
    examples: Vec<Scenario>, // the requirement's scenarios
}
```

The `- **Deprecated:**` line follows the FROM/TO precedent: one parsable
convention inside ordinary prose. The word is ISO 704's: a glossary entry
has a preferred term and may list deprecated terms, and this is the
latter, so the one keyword the reviewer adds to the spec grammar borrows
an existing vocabulary rather than inventing one. The CLI, the lint
and the diff all see a normal requirement; only the glossary loader
gives the line meaning. The line stays in the diffed text so a change to the synonym
list shows up as a change.

## Matching

Whole-word, case-insensitive. A synonym `feature` must not fire on
`featured`, and a term `module` must not fire on `modules`? It should:
plural forms are the same word. The matcher therefore treats a trailing
`s` or `es` on the candidate as the same word, and nothing else. No
stemming library; the two suffixes cover the cases the fixtures show and
anything cleverer is guessing.

Multi-word synonyms (`feature mount`) match across a single space in
normalized text, which is what normalization guarantees.

Citations are stripped before matching: any `spec:…§…` span up to its
closing delimiter is blanked, so a citation of a requirement named
"Feature mount appears" does not count as using the synonym.

## The four checks

| Check | Runs in | Where reported |
| --- | --- | --- |
| deprecated synonym | review + lint | the pairing that contains it; canon hits in lint, or on the term's pairing when the change touches the term |
| defined but unused | review + lint | the term's pairing when touched; otherwise lint |
| recurring undefined | lint | lint |
| new term undefined | review | the pairing that introduces it first, in list order |
| term in use | review | the term's pairing |

Recurring-undefined shares the backticked and quoted extraction with
term-drift; it lives in `glossary::checks` but calls the same tier
extractor. Counting is per requirement, not per occurrence, and the
"two capabilities" floor keeps a capability's own local jargon out of
the glossary suggestions.

New-term-undefined needs "absent from every before side of the change".
That is a set difference over the change, not per pairing, which is why
it is reported once, on the first introducer.

## Panel

`D` toggles a section under the diff. Terms present in the after text
are found with the same matcher, ordered by first byte offset. Each entry
renders the term bold, the meaning as a paragraph, and `Deprecated:` as an
indented list. The panel is per-pairing state, not global, so it stays
open while moving between rows.

## Configuration

```toml
[definitions]
capability     = "definitions"   # which capability is the glossary
min_recurrence = 3               # recurring-undefined threshold
```

Both optional. The section is in the same `openspec/reviewer.toml` as
`[lint]` and `[term_drift]`, and unlike `[lint]` its absence is not a
refusal: a project without a glossary simply has no glossary checks, and
the summary line says so.

## Testing

A fixture glossary with `module` (`Deprecated: feature, feature mount`) and
`website` (`Deprecated: tenant`), lifted from rel-monorepo's actual renames, and
a fixture canon that uses both words plus one synonym. One test per
requirement; the substring and citation exclusions as regression tests.
