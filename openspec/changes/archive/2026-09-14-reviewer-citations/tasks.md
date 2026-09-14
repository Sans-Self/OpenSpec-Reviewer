# Tasks: reviewer-citations

## 1. Index

- [x] 1.1 `Citation`, `Citer`, `CitationIndex` types; whitespace
      normalization of requirement names.
- [x] 1.2 Literal and call-form grammars as regexes, with the apostrophe
      and trailing-comma cases as tests.
- [x] 1.3 Scan canon specs, open-change deltas and configured source
      roots into the index; skipped directories honoured.
- [x] 1.4 Resolution against canon plus in-flight ADDED requirements.

## 2. Evidence

- [x] 2.1 Path citations from configured prefixes and extensions, with
      the dotfile-prefix boundary check.
- [x] 2.2 Test-name citations looked up as fixed strings in the scanned
      source files.
- [x] 2.3 Hash citations via `git cat-file -e`.

## 3. Blast radius and structure

- [x] 3.1 REMOVED with outside citers is an error unless the change
      carries a delta for the citing capability.
- [x] 3.2 MODIFIED with outside citers is a note listing them.
- [x] 3.3 Change directory marker check; scope-prefix check;
      grandfathered list with stale-entry error.

## 4. Configuration

- [x] 4.1 `openspec/reviewer.toml` schema with every field optional and
      unknown keys rejected.
- [x] 4.2 Missing file is a refusal that prints a template; fields left
      out of a present file are named in the summary line.

## 5. Output

- [x] 5.1 `lint` subcommand: notes to stdout, errors to stderr, summary
      line, exit status from the worst finding.
- [x] 5.2 `--coverage` ledger.
- [x] 5.3 `--format json` for the lint.

## 6. Review join

- [x] 6.1 Three citation finding kinds with citing-file lists.
- [x] 6.2 Findings for the change under review attached to their
      pairings; citing files shown in the detail pane, plain text and
      JSON.

## 7. Term drift

- [x] 7.1 Removed-term extraction in three tiers with the containment
      rule for phrases.
- [x] 7.2 Stop list, spec keywords, `term_drift.max_common` threshold.
- [x] 7.3 Sibling search over canon outside the capability; hits
      resolved against the change's own deltas.
- [x] 7.4 Old-name search for RENAMED pairings with citations stripped.
- [x] 7.5 Two drift finding kinds with sibling lists, shown in the
      detail pane, plain text and JSON.

## 8. Wrap-up

- [x] 8.1 Fixture repository under `tests/fixtures/lint/` initialized
      as git at test time.
- [x] 8.2 Every requirement cited from at least one test title.
- [x] 8.3 A sample `openspec/reviewer.toml` in the README, and the lint
      run against a real repository with `spec:` citations, `bug__`
      tests, path and hash citations across specs and source.
