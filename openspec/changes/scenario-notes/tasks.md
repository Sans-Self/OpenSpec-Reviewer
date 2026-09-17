# Tasks: scenario-notes

## 1. The store

- [ ] 1.1 Item keys accept `capability/requirement#scenario`, and a
      requirement's own note keeps the key it has today.
- [ ] 1.2 A note is `{ text, text_hash, at }`; a bare string in an
      existing file parses as a note with no hash.
- [ ] 1.3 A note whose stored hash differs from its anchor's current
      normalized text reports as outdated.

## 2. The list

- [ ] 2.1 A requirement's scenarios are rows beneath it, taking `+`,
      `-`, `~` or no glyph from `ScenarioMatch`, with no approval mark.
- [ ] 2.2 Scenario rows are folded when the view opens; `Space` toggles
      a requirement's fold; `n` and `p` expand what they land in.
- [ ] 2.3 A folded requirement shows `✎` when it or any scenario under
      it holds a note.
- [ ] 2.4 `a` on a scenario row toggles its parent requirement.

## 3. The popup

- [ ] 3.1 Focus is `Browsing`, `Transient` or `Modal`, one modal at a
      time, and `Esc` cancels the modal instead of quitting.
- [ ] 3.2 The popup quotes the anchor above a single wrapped buffer;
      `⏎` saves, `Esc` cancels, an empty buffer removes the note.
- [ ] 3.3 `^E` suspends the view, opens `$EDITOR` on the buffer, and
      returns to the popup with what the editor saved.

## 4. Output

- [ ] 4.1 Plain and JSON carry a note's anchor and whether it is
      outdated.
- [ ] 4.2 `--format markdown` nests a scenario's note under its
      requirement and marks an outdated one.
