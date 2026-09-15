## Why

The glossary models two of ISO 704's three term acceptability ratings.
The requirement name is the preferred term and `- **Deprecated:**` lists
the terms not to use, but there is no way to say that a second word is a
fine name for the same concept. Every project has these: `pairing` and
`match`, `canon` and `the specs`. Today the reviewer treats them as
strangers, and two checks give wrong answers because of it. A term used
everywhere under its second name is reported "defined but unused". A
word that already has a definition under another name is proposed as a
"recurring term without definition", so `/opsx-reviewer:define` drafts a
duplicate of a term the glossary already holds.

## What Changes

- A `- **Admitted:**` line in a term's body lists terms that are
  acceptable for the concept. It parses like `- **Deprecated:**`: a
  comma-separated list, removed from the meaning text, shown as a list in
  the definitions panel.
- An admitted synonym counts as a use of its preferred term. "Defined but
  unused" fires only when neither the term nor any admitted synonym appears.
- An admitted synonym is never proposed as undefined. Both "recurring term
  without definition" and "new term without definition" skip it, as they
  already skip deprecated terms.
- An admitted synonym is never a finding by itself. There is no "uses
  admitted synonym" warning; that is what deprecated terms are for.
- The definitions panel and the JSON `definitions` array carry admitted
  synonyms alongside deprecated ones. JSON gains an `admitted` field per
  entry.
- `/opsx-reviewer:define` drafts an `- **Admitted:**` line when the uses
  show a word that names the same concept, and keeps drafting
  `- **Deprecated:**` when the uses show a word the project should stop
  saying.

Not in this change: term drift. A removed term that survives on the
after side under an admitted name is still reported as removed, and a
sibling using an admitted synonym is still not a hit on the preferred term.
Making term drift consult the glossary couples a pure text comparison to
the definitions capability and changes the finding count in both
directions. That decision is worth making from real misfires once
projects have admitted synonyms, not before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `glossary`: a term lists the terms that are acceptable for it;
  admitted synonyms count as usage for "defined but unused"; admitted synonyms
  are excluded from the undefined-term checks; the definitions panel and
  the JSON `definitions` array carry them.
- `skills`: the define skill drafts an `- **Admitted:**` line as well as
  a `- **Deprecated:**` one.

## Impact

- `src/glossary/synonyms.rs`: the `- **Deprecated:**` parser generalizes
  to a marker-keyed line parser returning meaning, admitted and
  deprecated. The `Matcher` is unchanged.
- `src/glossary/mod.rs`: `Term` gains an `admitted` field.
- `src/glossary/checks.rs`: the unused check and both undefined-term
  checks consult admitted synonyms.
- `src/build.rs`, `src/review/findings.rs`: no new finding kind; the JSON
  document's `definitions` entries gain `admitted`.
- `src/render/`: the definitions panel lists admitted synonyms under the
  meaning.
- `src/skills/define.md`: drafting guidance for the new line.
- `tests/fixtures/`: the glossary fixture gains a term with an
  `- **Admitted:**` line.
