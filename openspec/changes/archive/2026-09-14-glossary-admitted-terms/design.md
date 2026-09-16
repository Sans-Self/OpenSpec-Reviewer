## Context

`src/glossary/` reads the `definitions` capability as a glossary. A term
is a requirement: the name is the term, the body its meaning, and
`parse_deprecated` lifts a `- **Deprecated:**` line out of the body into
`Term::deprecated`. Two consumers read that list. `Glossary::knows`
answers "does the glossary account for this word?" and gates both
undefined-term checks. `deprecated_in_pairings` and `deprecated_in_canon`
walk the list to raise warnings.

The list is doing one job with two meanings folded into it: the words
this project accounts for, and the words this project forbids. Admitted
terms need the first without the second.

## Goals / Non-Goals

**Goals:**

- A term can name more than one acceptable word for its concept.
- A concept used under an admitted synonym is not "unused", and is not
  proposed as undefined.
- An admitted synonym is never itself a finding.

**Non-Goals:**

- Term drift. `src/drift/` compares before and after text and consults
  nothing else; it keeps that property here.
- Citations. A `spec:` citation names a requirement, not a term.
- Ranking admitted synonyms. ISO 704 allows several with no order; the list
  is a set, rendered in written order.

## Decisions

### The line is `- **Admitted:**`

ISO 704 §7.3 rates a term preferred, admitted or deprecated. The
requirement name is the preferred term by construction and
`- **Deprecated:**` is already in the file, so `- **Admitted:**` is the
rung that completes the scale rather than a new vocabulary.

The rejection is the point for one alternative: `- **Synonyms:**` reads
well to someone who has not met ISO 704, but the deprecated line also
lists synonyms, so the two lines would claim the same name for opposite
verdicts.

### One parser, keyed by marker

`parse_deprecated` becomes `parse_markers`, which walks the body once and
returns the meaning with every recognized marker line removed, plus a map
from marker to its comma-separated words. `Term` gains `admitted:
Vec<String>` beside `deprecated`. Both lines stay part of the diffed text
for pairings, as the deprecated line already does, so editing either one
shows up as a change to the term.

An unrecognized `- **Foo:**` line stays in the meaning. The parser
recognizes a fixed set, not any bold-prefixed line, so an author's own
bullet is not silently eaten.

### `knows` widens, the warning does not

| consumer | change |
| --- | --- |
| `Glossary::knows` | also true for an admitted synonym |
| `recurring_undefined`, `new_undefined` | none; they call `knows` |
| `unused_terms` | a term is used when the term or any admitted synonym matches |
| `deprecated_in_pairings`, `deprecated_in_canon` | none |
| `terms_in` | an admitted synonym's offset counts as the term's first appearance |

No new `FindingKind`. The whole value of an admitted synonym is that saying
it is fine.

`terms_in` currently matches on the term name alone, so the definitions
panel would miss a pairing that only ever says the admitted word. It
takes the earliest offset among the term and its admitted synonyms.

### Rendering

The definitions panel lists admitted synonyms under the meaning, above
deprecated ones, so the reader sees what they may say before what they
may not. `Term` serializes `admitted` as a field beside `deprecated`,
which carries it into the JSON `definitions` array with no work in
`src/render/json.rs`.

## Risks / Trade-offs

- **An author writes a word under both lines.** → The parser keeps both
  lists as written; `knows` is true either way, and the deprecated
  warning still fires on every use. The panel shows the word under both
  headings, which is the contradiction made visible rather than a
  precedence rule invented to hide it.
- **Admitted terms mask a real drift.** → A change that drops `manager`
  for `owner` still produces sibling findings, because term drift is out
  of scope here. That is noise, not a wrong answer, and it is the
  evidence the term-drift decision needs.
- **`unused_terms` and `terms_in` compile one `Matcher` per admitted
  synonym.** → Same order as the deprecated checks already pay, over a
  glossary of tens of terms.

## Open Questions

None.
