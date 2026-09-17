# Tasks: phrase-precision

## 1. Matching

- [x] 1.1 `Matcher::ranges` returns the byte range of every hit in the
      citation-stripped text, comparable across matchers over the same
      text.
- [x] 1.2 Both deprecated checks build the shield set from the glossary's
      multi-word term names and admitted synonyms, and drop a hit whose
      range lies inside a shield's range. Scenario names take the same
      path as bodies.

## 2. Drift

- [x] 2.1 The phrase tier of `removed_terms` reads the body through
      `parse_markers`; the backticked and quoted tiers read the raw text.
- [x] 2.2 Sibling phrase matching reads the same marker-stripped text.

## 3. Tests

- [x] 3.1 `glossary § A deprecated synonym in a spec is a warning`: a
      body saying "PLC rotation key" yields no finding for `rotation key`
      when `PLC rotation key` is a term; "the rotation key wraps" yields
      one.
- [x] 3.2 `term-drift § A pairing yields the terms it removes`: a
      before/after pair differing only in a Deprecated line yields no
      removed terms.
