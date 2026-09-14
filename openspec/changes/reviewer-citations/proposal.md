# reviewer-citations

## Why

Specs are only as good as what points at them. In `rel-monorepo`, specs
cite repository paths, `bug__` regression tests and commit hashes as
evidence, and tests cite the requirement they exercise with
`spec:<capability> § <requirement>`. A TypeScript script,
`bin/spec-lint.mts`, keeps both directions honest: a renamed test, a moved
file, a fabricated hash or a requirement renamed without its tests all
fail CI. It also computes blast radius for a change: a requirement
removed while something still cites it is an error, a modified one with
outside citers is a note.

That script is welded to one repository. Its path prefixes, source roots,
change-name scopes and helper name are constants. And its blast-radius
output is a line on stderr, while the reviewer this tool provides is
exactly where a reviewer wants to see "this requirement you are looking
at is cited by these four tests".

## What changes

A `lint` subcommand that rewrites the script in Rust on top of the same
requirement parser the reviewer uses, and a join that turns the lint's
per-change results into findings on the review.

- Citations resolve against canon plus requirements ADDED by open
  changes, so tests can cite spec-first.
- Specs cite evidence: paths must exist, test names must appear in
  source, hashes must be commits.
- A REMOVED requirement with an outside citer is an error; a MODIFIED one
  lists its citers.
- Change directories must be changes, and their names may be required to
  start with a configured scope.
- `--coverage` prints the per-requirement citing-test ledger.
- Repository-specific choices live in `openspec/reviewer.toml`. The
  lint refuses to run without it and prints a template.
- When the reviewer shows a change, the lint's results for that change
  appear as findings on the matching pairings, with the citing files in
  the detail pane.
- Term drift: a term a pairing removes, an identifier, a quoted string
  or a phrase, that still appears in a canon requirement of another
  capability is a warning on that pairing, naming the sibling. A renamed
  requirement's old name in sibling prose is the same warning. This is
  the mechanical half of cross-spec review.

## Capabilities

| Capability | Covers |
| --- | --- |
| `citations` | Citation grammar, resolution, evidence checks, blast radius, coverage, configuration, the `lint` subcommand, and the join into review findings |
| `term-drift` | Terms a pairing removes, siblings that still use them, old names in prose, the common-phrase filter |

## Non-goals

- Judging whether a sibling's reasoning still holds. Term drift finds
  siblings that still use words a change removed; whether an archived
  design rejected the approach a new change takes needs a reader. The
  JSON output hands that reader the pairings and the drift hits.
- Fixing citations. The lint reports; the editor edits.
- Replacing the repository's CI script on day one. The two can run side
  by side until the outputs match.
