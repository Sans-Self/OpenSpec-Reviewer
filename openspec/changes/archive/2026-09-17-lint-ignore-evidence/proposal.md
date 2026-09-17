# lint-ignore-evidence

## Why

This repository's own specs quote evidence that does not resolve, because
quoting it is the point: `citations` shows the grammar parsing
`spec:alpha § Some rule`, a made-up capability; `src/a.rs`, `deadbeef`
and `bug__float_not_rounding` stand in for a path, a commit and a
regression test; `term-drift` and `glossary` name Opake capabilities this
repository does not have. The lint reads all ten as broken evidence and
exits `1`, so `openspec-reviewer lint` has never been able to gate this
repository's own tree.

Two ignore lists already exist, for undefined terms and for uncited
requirements. Evidence has none, so there is nowhere to write down that a
citation is an example.

## What Changes

- `openspec/reviewer.toml` accepts `[[lint.ignore_evidence]]`. Each entry
  names exactly one of `citation`, `path`, `test` or `commit`, and says
  why. The lint drops the matching finding.
- An entry that silences nothing warns, like the other two lists.
- This repository's configuration gains the ten entries its own specs
  need, and its lint reaches zero errors.

## Capabilities

### Modified Capabilities

- `citations`: "An ignore entry names one finding and its reason" covers
  the third list; "A dangling ignore entry is a warning" covers an
  evidence entry that matches nothing.

## Impact

- `src/citations/config.rs`: the entry type, and validation that an entry
  names exactly one kind and a reason.
- `src/citations/lint.rs`: the four evidence families filter through the
  list, and unused entries become warnings.
- `openspec/reviewer.toml`: ten entries.

## Non-goals

- Globs or patterns. The other two lists match exact strings and say so;
  a third that did not would be a trap.
- Confining an entry with `in`. Evidence findings name a file, not a
  requirement, and the strings are distinctive enough to stand alone.
