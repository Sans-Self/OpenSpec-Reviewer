# pairing-vocabulary

## Why

`pairing` deprecates `pair` and `matching`, and nine requirements across
four capabilities trip the check. None of them means a pairing.

`matching` is flagged in "files matching the configured globs", "text
matching the configured test pattern" and "findings on the matching
pairings" — globs, regular expressions and an adjective in front of the
word it supposedly replaces. The term's own meaning reads "One delta
entry matched with its canon counterpart", so the glossary deprecates a
word its own definition uses.

`pair` is flagged as the verb that forms a pairing — "the tool MUST pair
each delta entry with the canon requirement" — and as the ordinary noun
for two of a thing: two names in a rename, two lines under `## RENAMED
Requirements`. The verb reading is the one worth keeping out: a reader
three lines from a RENAMED entry cannot tell "the pair" from "the
pairing".

## What Changes

- `pairing` no longer deprecates `matching`. It keeps `pair`, and names
  `match` as the verb, which is the word its meaning already uses.
- The four capabilities say `match` where they said `pair` as a verb, and
  name the two things plainly where they said "a pair".
- Two requirements are renamed, because the word is in their names.

## Capabilities

### Modified Capabilities

- `definitions`: `pairing` drops one deprecated synonym.
- `change-model`: "A rename is a pair of names" is renamed and reworded.
- `semantic-diff`: "Every delta entry pairs with canon by name" is
  renamed and reworded; one scenario of "A rename shows both names and
  the body diff" drops "name pair".
- `input-sources`: "Canon files in a snapshot are shown as plain diffs"
  says `match`.

## Impact

- Four canon specs and the four test titles that quote the two renamed
  requirements. No source changes: the vocabulary is prose.

## Non-goals

- Deprecating `match`. The verb has to be available for the act that
  produces a pairing, and for globs and patterns.
