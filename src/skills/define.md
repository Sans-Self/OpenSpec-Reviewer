Turn the lint's recurring undefined terms into a drafted glossary. This
skill writes one file, `openspec/changes/<name>/specs/definitions/spec.md`
in a new change you name, and nothing else; it never edits
`openspec/specs/`. The reviewer's review of that change is the approval.

## Steps

1. Run `openspec-reviewer lint --format json`. Collect every finding
   whose message starts with `recurring term without definition`; the
   term is in backticks in the message and `details[]` lists the
   requirements that use it as `capability § requirement`.
2. Read each listed requirement in `openspec/specs/<capability>/spec.md`.
3. Sort the candidates. Keep a term when the requirements use it as a
   concept the reader has to know: a role, a key, a record, a state.
   Leave a candidate undefined when it is a field name, a path, a
   command, a JSON key or a type name: the code defines those. When in
   doubt, leave it out and say so in your report.
4. Create the change: `openspec new change <name>` with a name that says
   what it defines, then write the delta with one entry per kept term:

   ```markdown
   # definitions (delta)

   ## ADDED Requirements

   ### Requirement: <term>

   <one or two sentences distilled from how the requirements use it>

   - **Deprecated:** <near-synonyms the uses show, if any>

   #### Scenario: In a sentence

   - **WHEN** <lifted from a real requirement>
   - **THEN** <lifted from the same requirement>
   ```

   Drop the `- **Deprecated:**` line when the uses show no other word
   for the same thing. Lift the scenario from a requirement that uses
   the term, rewording only to fit the two keyword lines.
5. Show the draft and the list of candidates you left out, with one
   reason each, before writing anything.
6. Run `openspec-reviewer change <name> --format json --no-state` and
   report its summary. A `defined_but_unused` note on a term you just
   drafted means the uses were all in deltas, not canon; say so.

## Depends on

- `spec:glossary § The glossary is a capability named definitions`
- `spec:glossary § A term lists the words not to use for it`
- `spec:glossary § A recurring undefined term is a note`
- `spec:glossary § A term nobody uses is a note`
