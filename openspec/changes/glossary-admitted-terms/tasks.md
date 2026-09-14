# Tasks: glossary-admitted-terms

## 1. Parsing

- [ ] 1.1 `parse_deprecated` becomes a marker-keyed parser over a fixed
      set of `- **<Marker>:**` lines, returning the meaning with every
      recognized line removed plus the words each line listed. An
      unrecognized bold line stays in the meaning.
- [ ] 1.2 `Term` gains `admitted: Vec<String>`, populated from
      `- **Admitted:**` and serialized beside `deprecated`.

## 2. Checks

- [ ] 2.1 `Glossary::knows` is true for an admitted synonym, so both
      undefined-term checks skip it with no change of their own.
- [ ] 2.2 `unused_terms` treats a term as used when its name or any
      admitted synonym matches.
- [ ] 2.3 `Glossary::terms_in` matches on the name and the admitted
      synonyms, ordering each term by its earliest hit.
- [ ] 2.4 A word listed as both admitted and deprecated still raises the
      deprecated warning.

## 3. Surfaces

- [ ] 3.1 Definitions panel lists admitted synonyms between the meaning
      and the deprecated synonyms.
- [ ] 3.2 JSON `definitions` entries carry `admitted`.

## 4. Skill

- [ ] 4.1 `src/skills/define.md` drafts an `- **Admitted:**` line for a
      second acceptable word, a `- **Deprecated:**` line for a word to
      stop saying, and never the same word on both.

## 5. Wrap-up

- [ ] 5.1 Fixture glossary gains a term with an `- **Admitted:**` line
      and canon that uses only the admitted synonym.
- [ ] 5.2 Every requirement of this change cited from at least one test
      title.
