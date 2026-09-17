# shell-completion

## ADDED Requirements

### Requirement: The binary prints a completion stub for its shell

When the environment variable `COMPLETE` names a shell, running
`openspec-reviewer` MUST print that shell's completion stub to stdout
and exit `0` without parsing the command line. The shells MUST be
`bash`, `zsh` and `fish`; any other value is an error naming those
three. The stub MUST call the binary back to compute candidates, so
completions reflect the repository the shell is in.

#### Scenario: Fish stub

- **GIVEN** `COMPLETE=fish` in the environment
- **WHEN** `openspec-reviewer` runs with no arguments
- **THEN** stdout is a fish script that names `openspec-reviewer`
- **AND** the exit status is `0`

#### Scenario: Unsupported shell

- **GIVEN** `COMPLETE=elvish` in the environment
- **WHEN** `openspec-reviewer` runs with no arguments
- **THEN** stderr names `bash`, `zsh` and `fish`
- **AND** the exit status is not `0`

#### Scenario: Unset variable

- **GIVEN** `COMPLETE` is unset
- **WHEN** `openspec-reviewer change foo` runs
- **THEN** the command runs as it does today

### Requirement: Fixed words complete from the command tree

Completion MUST offer the subcommands, each subcommand's flags, the
`--format` values and the `lint` and `skills` actions.

#### Scenario: Format values

- **GIVEN** the command line `openspec-reviewer --format `
- **WHEN** the shell asks for candidates
- **THEN** the candidates are `text`, `json` and `markdown`

### Requirement: A change name completes from the repository

`change <name>` MUST complete from the directories under
`openspec/changes/` in the current directory, sorted by name, with
`archive` excluded. A missing or unreadable `openspec/changes/` MUST
yield no candidates and no error.

#### Scenario: Two changes and the archive

- **GIVEN** `openspec/changes/` containing `alpha`, `beta` and `archive`
- **WHEN** the shell asks for candidates after `change `
- **THEN** the candidates are `alpha` and `beta`

#### Scenario: Not in a repository

- **GIVEN** a current directory with no `openspec/` in it
- **WHEN** the shell asks for candidates after `change `
- **THEN** there are no candidates
- **AND** nothing is written to stderr

### Requirement: A git reference completes from git

`git <REF>` and `--base <REF>` MUST complete from the short names of
the repository's heads and tags. A failing `git` MUST yield no
candidates and no error.

#### Scenario: A branch and a tag

- **GIVEN** a repository with branch `main` and tag `v0.2.0`
- **WHEN** the shell asks for candidates after `git `
- **THEN** the candidates include `main` and `v0.2.0`

### Requirement: A pull request completes from gh within a second

`gh <pr>` MUST complete from `gh pr list --json number,title`, each
candidate being the number with the title as its help text. The
completer MUST stop waiting for `gh` after one second, kill it, and
offer no candidates. A failing or absent `gh` MUST yield no candidates
and no error.

#### Scenario: Open pull requests

- **GIVEN** `gh pr list` answering with pull requests `12` and `23`
- **WHEN** the shell asks for candidates after `gh `
- **THEN** the candidates are `12` and `23`
- **AND** each carries its title as help text

#### Scenario: Slow gh

- **GIVEN** a `gh` on `PATH` that does not answer for five seconds
- **WHEN** the shell asks for candidates after `gh `
- **THEN** there are no candidates
- **AND** the completer returns within about one second

#### Scenario: No gh

- **GIVEN** no `gh` on `PATH`
- **WHEN** the shell asks for candidates after `gh `
- **THEN** there are no candidates
- **AND** nothing is written to stderr
