# Tasks: term-highlighting

## 1. Occurrences

- [ ] 1.1 `Matcher` reports hits as ranges into the text it was given,
      not into the citation-stripped copy.
- [ ] 1.2 A glossary-level function returns the live occurrences of a
      term in a text: every hit of every admitted name, and every hit of
      a deprecated synonym that no longer name contains.
- [ ] 1.3 `deprecated_in_pairings` and `deprecated_in_canon` answer from
      that function rather than their own `shields`/`covered`/`used`.

## 2. The detail pane

- [ ] 2.1 Detail spans split on occurrence boundaries, and an admitted
      name is underlined across the whole name.
- [ ] 2.2 A live deprecated synonym takes the warning style on top of
      the underline.
- [ ] 2.3 Marking composes: a term inside an added paragraph keeps the
      added style and the `+` glyph, and under `NO_COLOR` the underline
      remains.

## 3. Wrap-up

- [ ] 3.1 Tests titled by requirement, asserting cell styles through
      `TestBackend`.
- [ ] 3.2 `tui-color` drops "Glossary terms are highlighted in the
      detail text", its task and its proposal bullet.
