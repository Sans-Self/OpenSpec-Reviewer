# Tasks: reviewer-foundation

## 1. Model

- [ ] 1.1 Line scanner that turns a spec file into requirements and
      scenarios, for canon and delta alike.
- [ ] 1.2 Delta sections: ADDED, MODIFIED, REMOVED, RENAMED, with FROM/TO
      pairs; unknown sections and orphan FROM/TO are typed parse errors.
- [ ] 1.3 Canon loader over `openspec/specs/*/spec.md`; change loader over
      a snapshot's files with artefacts and deltas; archive is refused.
- [ ] 1.4 Rename joining: RENAMED + MODIFIED of the new name become one
      entry.
- [ ] 1.5 Fixtures under `tests/fixtures/` lifted from rel-monorepo: the
      sitemap-labels MODIFIED, a RENAMED + MODIFIED pair, a pure re-wrap.

## 2. Sources

- [ ] 2.1 `Source` trait, `Snapshot`, `FileChange`, `SourceError`.
- [ ] 2.2 Clap surface: global flags (`--plain`, `--format`,
      `--findings-only`, `--color`, `--no-state`) and the subcommands
      `change`, `diff`, `git`, `gh`; no subcommand prints help.
- [ ] 2.3 `change` source over the working tree.
- [ ] 2.4 `diff` source: split on `diff --git`, filter to `openspec/`,
      apply hunks with `diffy`, pre-image mismatch is a hard error naming
      file and hunk.
- [ ] 2.5 `git` source: `git diff --name-status base...ref` plus
      `git show` per side; base defaults to `main`, then `master`, then
      an error; git stderr passes through.
- [ ] 2.6 `gh` source: `gh pr diff` into the diff source; gh stderr
      passes through.
- [ ] 2.7 Canon files in a snapshot listed as plain line diffs.
- [ ] 2.8 Source tests against a temp git repository.

## 3. Semantic diff

- [ ] 3.1 Paragraph normalization: join wrapped lines, collapse spaces,
      keep list items and blank lines as boundaries.
- [ ] 3.2 Pairing by capability and name for every kind.
- [ ] 3.3 Paragraph diff with word-level inline marks via `similar`.
- [ ] 3.4 Scenario matching by name; added, removed and reordered
      scenarios handled per spec.
- [ ] 3.5 Rendering rules per kind: added full, removed canon, renamed
      name pair plus body.

## 4. Findings

- [ ] 4.1 Finding enum with fixed severities and a location type.
- [ ] 4.2 Checks: modified without canon, added already exists, removed
      or rename source missing, rename target taken, scenario dropped,
      no scenarios, no change.
- [ ] 4.3 Cross-change collision scan over other open changes in the
      working tree.
- [ ] 4.4 Exit status from the worst finding.

## 5. State

- [ ] 5.1 `ItemState` and `Approval` types; store keyed by
      `capability/requirement` and artefact filename.
- [ ] 5.2 XDG state path from origin URL or top-level path; `--no-state`
      bypass.
- [ ] 5.3 Stale detection by normalized-text hash.
- [ ] 5.4 Note editing through `$EDITOR` with TUI suspend and resume;
      empty file removes the note.
- [ ] 5.5 Markdown export of notes.

## 6. History

- [ ] 6.1 Archive walk sorted by directory name, entries by name and
      capability.
- [ ] 6.2 Rename chasing backwards through RENAMED entries.
- [ ] 6.3 Unparseable archive becomes a note finding, not an error.

## 7. Output

- [ ] 7.1 Plain text renderer mirroring the inline view, with state,
      findings and history summary; no escapes by default, `--color`
      opt-in.
- [ ] 7.2 JSON renderer over the review model with serde.
- [ ] 7.3 `--findings-only` line format.
- [ ] 7.4 Terminal detection picks TUI or plain.

## 8. TUI

- [ ] 8.1 App state and event loop on ratatui + crossterm; terminal
      restore on quit, panic and signal.
- [ ] 8.2 List pane: artefact rows, capability headings, requirement rows
      with approval mark, kind glyph, finding and note markers.
- [ ] 8.3 Detail pane: inline, side-by-side, raw; findings, note and
      history line under the diff; scrolling.
- [ ] 8.4 Status line with change name, approved count, finding counts,
      mode.
- [ ] 8.5 Key bindings and help overlay per spec; `n`/`p` finding jumps;
      `a` approve; `e` note.
- [ ] 8.6 History view: entries left, version or diff-to-previous right,
      `Esc` returns.
- [ ] 8.7 `NO_COLOR` and no-colour terminal handling.

## 9. Wrap-up

- [ ] 9.1 README with the four subcommands and a plain-text sample.
- [ ] 9.2 Every requirement cited from at least one test title.
- [ ] 9.3 `nix build` produces the binary; `Cargo.lock` committed.
- [ ] 9.4 Review this change with the tool itself, then archive it.
