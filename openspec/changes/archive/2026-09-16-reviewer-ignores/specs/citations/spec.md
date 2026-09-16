## ADDED Requirements

### Requirement: An ignore entry names one finding and its reason

`openspec/reviewer.toml` MUST accept two arrays of tables.
`[[definitions.ignore]]` entries have `term`, an optional `in`, and
`reason`. `[[lint.ignore_uncited]]` entries have `requirement`, written
`<capability> § <requirement name>`, and `reason`. Every value is an
exact string; the tool MUST NOT read any of them as a glob or a pattern.
An entry missing `reason` MUST be a configuration error naming the entry.

#### Scenario: Ignored term

- **GIVEN** a config with a `[[definitions.ignore]]` entry for `steward`
- **WHEN** the lint runs
- **THEN** it reports no undefined-term finding for `steward`

#### Scenario: Entry without a reason

- **GIVEN** a `[[definitions.ignore]]` entry with only `term`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

#### Scenario: Entry is not a pattern

- **GIVEN** a `[[definitions.ignore]]` entry for `steward`
- **AND** an undefined term `stewardship`
- **WHEN** the lint runs
- **THEN** it reports the finding for `stewardship`

### Requirement: An ignore entry can be confined to scopes

A `[[definitions.ignore]]` entry MAY carry `in`, an array of scopes. Each
scope is a capability name or a `<capability> § <requirement name>`. The
entry MUST apply only to text inside those scopes. An entry without `in`
MUST apply everywhere. A bare string in `in` MUST be a configuration
error naming the entry.

#### Scenario: Scoped to one capability

- **GIVEN** an ignore for `reason` with `in = ["citations"]`
- **AND** the term recurring in `citations` and in `glossary`
- **WHEN** the lint runs
- **THEN** it reports no finding in `citations`
- **AND** it reports the finding in `glossary`

#### Scenario: Scoped to two capabilities

- **GIVEN** an ignore for `reason` with `in = ["citations", "glossary"]`
- **AND** the term recurring in both
- **WHEN** the lint runs
- **THEN** it reports no finding for `reason`

#### Scenario: Scoped to one requirement

- **GIVEN** an ignore with `in = ["glossary § A term lists the words that are acceptable for it"]`
- **AND** the term appearing in another requirement of `glossary`
- **WHEN** the lint runs
- **THEN** it reports the finding for that other requirement

#### Scenario: Scope is a bare string

- **GIVEN** an ignore entry with `in = "citations"`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

### Requirement: A dangling ignore entry is a warning

An ignore entry is dangling when a scope in its `in` does not resolve
against the register, when its `requirement` does not resolve against the
register, when its `term` is a glossary term, an admitted synonym or a
deprecated synonym, or when it silenced no finding in the run. Each MUST
produce a warning naming the entry, the scope at fault and
`openspec/reviewer.toml`, asking for it to be removed. Scopes MUST be
judged one at a time, so a live scope does not mask a dead one. `lint`
and `change` MUST reach the same verdict for the same entry.

#### Scenario: Term now defined

- **GIVEN** an ignore entry for `steward`
- **AND** a glossary term `steward`
- **WHEN** the lint runs
- **THEN** it reports a warning naming `steward`
- **AND** naming `openspec/reviewer.toml`

#### Scenario: Ignored requirement is gone

- **GIVEN** an `ignore_uncited` entry naming a requirement the register
  does not hold
- **WHEN** the lint runs
- **THEN** it reports a warning naming that entry

#### Scenario: One scope of two is dead

- **GIVEN** an ignore with `in = ["citations", "glossary"]`
- **AND** the term appearing only in `citations`
- **WHEN** the lint runs
- **THEN** it reports a warning naming the `glossary` scope
- **AND** it reports no finding for the term in `citations`

#### Scenario: Unknown capability in a scope

- **GIVEN** an ignore with `in = ["sitemap-index"]`
- **AND** no such capability in the register
- **WHEN** the lint runs
- **THEN** it reports a warning naming that scope

#### Scenario: Same verdict during a review

- **GIVEN** an ignore entry the lint reports as dangling
- **WHEN** the tool reviews any open change
- **THEN** it reports the same warning

## MODIFIED Requirements

### Requirement: Coverage lists citing tests per requirement

With `--coverage`, the lint MUST print, per capability, each requirement
of the register with the number of source files citing it, marking those
with zero, and a closing line
`coverage: <cited>/<total> (<ignored> ignored)`. A requirement named by a
`[[lint.ignore_uncited]]` entry MUST NOT be marked uncited and MUST still
be counted in `<total>`. The `(<ignored> ignored)` part is omitted when no
entry matched. Spec-to-spec citations do not count. The ledger never
changes the exit status.

#### Scenario: One uncited requirement

- **GIVEN** a capability with three requirements
- **AND** tests citing two of them
- **WHEN** the lint runs with `--coverage`
- **THEN** the ledger marks the third as uncited
- **AND** the closing line reads `coverage: 2/3`

#### Scenario: The uncited requirement is ignored

- **GIVEN** a capability with three requirements
- **AND** tests citing two of them
- **AND** an `ignore_uncited` entry naming the third
- **WHEN** the lint runs with `--coverage`
- **THEN** the ledger does not mark the third as uncited
- **AND** the closing line reads `coverage: 2/3 (1 ignored)`

### Requirement: Configuration lives beside the specs

The lint MUST read `openspec/reviewer.toml`. It holds the source roots,
source globs, skipped directories, path prefixes, path extensions, test
pattern, call helper name, change scopes, grandfathered changes, ignored
terms and ignored uncited requirements. Without the file the lint MUST
refuse to run, with an error naming the path, naming
`openspec-reviewer lint init` as the way to create it, and showing a
minimal example of the file. A malformed file is an error naming the key.
The file `lint init` writes MUST show both ignore sections as commented
examples.

#### Scenario: No config

- **GIVEN** a repository without `openspec/reviewer.toml`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that path
- **AND** the error names `lint init`
- **AND** the error shows a minimal example of the file
- **AND** the exit status is `2`

#### Scenario: Bad key

- **GIVEN** a config with `source_root = "apps"` instead of `source_roots`
- **WHEN** the lint runs
- **THEN** it stops with an error naming the unknown key

#### Scenario: Init documents the ignore sections

- **WHEN** `openspec-reviewer lint init` writes the file
- **THEN** it holds a commented `[[definitions.ignore]]` example
- **AND** a commented `[[lint.ignore_uncited]]` example
