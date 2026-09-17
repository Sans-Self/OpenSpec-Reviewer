# input-sources Specification

## Purpose
TBD - created by archiving change reviewer-foundation. Update Purpose after archive.
## Requirements
### Requirement: Every source yields the same snapshot

A source MUST produce a snapshot: for each file under `openspec/` it
touches, the path, the text before, and the text after. Either side may
be absent for a created or deleted file. Everything after the source,
from pairing to rendering, reads only the snapshot and canon. Adding a
source MUST NOT touch code outside the source module and the command
surface.

#### Scenario: Two sources, same change

- **GIVEN** a change that exists on a branch
- **AND** the same change in an open pull request
- **WHEN** the tool reviews it through the `git` source
- **AND** reviews it through the `gh` source
- **THEN** both reviews show the same pairings
- **AND** the same findings

### Requirement: Each source is a subcommand

The tool MUST expose one subcommand per source:

| Subcommand | Reads |
| --- | --- |
| `change <name>` | the change as it is in the working tree |
| `diff [<path>]` | a unified diff from a file, or stdin when the path is `-` or absent |
| `git <ref> [--base <ref>]` | the files of `<ref>` against `--base`, which defaults to `main`, then `master` |
| `gh <pr>` | a GitHub pull request by number or URL through the `gh` CLI |

Global flags come before the subcommand. Running with no subcommand
prints the help text.

#### Scenario: No subcommand

- **WHEN** the user runs the tool with no arguments
- **THEN** it prints the help text listing the four subcommands
- **AND** exits with a usage status

### Requirement: Canon always comes from the working directory

Whatever the source, canon MUST be read from `openspec/specs/` in the
working directory. The tool does not read canon from a branch or a pull
request.

#### Scenario: Branch with newer canon

- **GIVEN** a branch that also edits `openspec/specs/alpha/spec.md`
- **WHEN** the tool reviews that branch through the `git` source
- **THEN** pairings use the working-directory canon
- **AND** the branch's canon edit is listed as a plain diff under canon

### Requirement: The change source reads the working tree

`change <name>` MUST read `openspec/changes/<name>/` from the working
directory. The snapshot has an after side only.

#### Scenario: Existing change

- **GIVEN** a repository with `openspec/changes/sweep-gate/`
- **WHEN** the user runs `openspec-reviewer change sweep-gate`
- **THEN** the review covers that change's deltas and artefacts as they
  are on disk

#### Scenario: Unknown change

- **WHEN** the named change directory does not exist
- **THEN** the tool stops with an error that names the path it looked for
- **AND** lists the changes it did find

### Requirement: The diff source applies a unified diff to the working tree

`diff` MUST read a unified diff and keep every file under `openspec/`.
For a created file the after side is the added lines. For a modified
file the before side is the working-tree file and the after side is that
file with the hunks applied. For a deleted file the after side is absent.
When a hunk does not apply, the tool MUST stop with an error that names
the file and the hunk header. It MUST NOT fall back to the working-tree
version.

#### Scenario: Diff creates a whole change

- **WHEN** the diff creates every file of `openspec/changes/foo/`
- **THEN** the review covers change `foo`
- **AND** the files' content is taken from the diff

#### Scenario: Pre-image mismatch

- **GIVEN** a working-tree file that differs from a hunk's context lines
- **WHEN** the tool applies the diff
- **THEN** it stops
- **AND** the error names the file and the hunk header

#### Scenario: Diff has no OpenSpec content

- **WHEN** the diff touches no file under `openspec/`
- **THEN** the tool stops with an error saying so

### Requirement: The git source reads two refs

`git <ref>` MUST list files under `openspec/` that differ between
`--base` and `<ref>` using `git diff --name-status base...ref`, then read
each side with `git show <ref>:<path>`. No patch is applied. When
`--base` is omitted the tool MUST use `main` if it resolves, else
`master` if it resolves, else stop with an error asking for `--base`.
When git exits non-zero the tool MUST stop and show git's stderr.

#### Scenario: Feature branch against main

- **GIVEN** a branch `feature/foo` that adds a change
- **AND** a `main` branch
- **WHEN** the user runs `openspec-reviewer git feature/foo`
- **THEN** the snapshot holds each touched file with its `main` side and
  its branch side

#### Scenario: Repository with master

- **GIVEN** a repository with `master` and no `main`
- **WHEN** the user runs `openspec-reviewer git feature/foo`
- **THEN** the base is `master`

#### Scenario: Neither default base exists

- **GIVEN** a repository with neither `main` nor `master`
- **WHEN** the user runs `openspec-reviewer git feature/foo` without
  `--base`
- **THEN** the tool stops with an error asking for `--base`

#### Scenario: Unknown ref

- **WHEN** `<ref>` does not resolve
- **THEN** the tool stops
- **AND** shows git's error text

### Requirement: The gh source reads a pull request

`gh <pr>` MUST run `gh pr diff <pr>` in the working directory and hand
its output to the diff source. When `gh` is not on the path or exits
non-zero, the tool MUST stop and show gh's stderr.

#### Scenario: Open pull request

- **GIVEN** a checkout where `gh` is authenticated
- **WHEN** the user runs `openspec-reviewer gh 224`
- **THEN** the review equals piping `gh pr diff 224` into `diff -`

#### Scenario: gh missing

- **GIVEN** no `gh` on the path
- **WHEN** the user runs the `gh` subcommand
- **THEN** the tool stops with an error saying the `gh` source needs the
  `gh` CLI

### Requirement: Canon files in a snapshot are shown as plain diffs

When a snapshot holds a file under `openspec/specs/`, the tool MUST list
it under a canon heading as a plain line diff. It MUST NOT match those
lines to a change.

#### Scenario: Direct canon edit

- **WHEN** the snapshot modifies `openspec/specs/alpha/spec.md`
- **THEN** the review shows that file's line diff under canon
- **AND** no requirement pairing is made from it

