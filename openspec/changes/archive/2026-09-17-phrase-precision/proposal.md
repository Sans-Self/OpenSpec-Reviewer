# phrase-precision

## Why

Reviewing Opake's `define-protocol-vocabulary` — forty terms proposed at
once — surfaced two checks that fire on the glossary doing its job.

A deprecated synonym is matched on its own, so a synonym that is the tail
of a longer accepted phrase fires on every use of the longer phrase.
`rotation key`, deprecated under `group key`, warns on every "PLC rotation
key". `anchor`, deprecated under `verification method`, warns sixteen
times on "lineage anchor", which `lineage` lists as admitted. Opake
worked around both by writing `> Note:` prose instead of a Deprecated
line, which switches the check off for those words.

Term drift reads a term's body as prose, marker lines included. Retiring
a word from `- **Deprecated:** workspace key, rotation key` reads as a
sentence losing a phrase: the bigram run walks across the comma and
reports a removed term `key rotation` that no requirement ever said.

## What Changes

- The longest phrase wins. Every multi-word term name and admitted
  synonym shields the text it covers, and a deprecated synonym found
  inside a shielded span is that longer phrase, not a violation.
- The phrase tier of term drift reads a requirement's meaning, not its
  marker lines. Backticked and quoted tiers are unchanged, on both the
  removed-term side and the sibling side.

## Capabilities

### Modified Capabilities

- `glossary`: "A deprecated synonym in a spec is a warning" gains the
  shielding rule.
- `term-drift`: "A pairing yields the terms it removes" and "A removed
  term found in a sibling is a warning" read marker lines as lists
  rather than prose.

## Impact

- `src/glossary/synonyms.rs`: `Matcher` gains `ranges`.
- `src/glossary/checks.rs`: both deprecated checks iterate hits and drop
  the shielded ones.
- `src/drift/terms.rs`: the phrase tier's text comes through
  `parse_markers`.
- `src/drift/siblings.rs`: sibling phrase matching uses the same text.

## Non-goals

- Single-word shields. A one-word term cannot contain a synonym, and
  admitting it as a shield would silence the synonym everywhere.
- Deciding which of two terms owns an overlapping phrase. The longer
  match wins by length, not by ownership.
