# citations (delta)

## MODIFIED Requirements

### Requirement: An ignore entry names one finding and its reason

`openspec/reviewer.toml` MUST accept three arrays of tables.
`[[definitions.ignore]]` entries have `term`, an optional `in`, and
`reason`. `[[lint.ignore_uncited]]` entries have `requirement`, written
`<capability> § <requirement name>`, and `reason`.
`[[lint.ignore_evidence]]` entries have exactly one of `citation`,
written `<capability> § <requirement name>`, `path`, `test` or `commit`,
and `reason`; neither `lint` nor a review MUST report the missing
evidence that entry names. Every value is an exact string; the tool MUST NOT read any of them
as a glob or a pattern. An entry missing `reason` MUST be a configuration
error naming the entry. An `[[lint.ignore_evidence]]` entry naming no
kind, or more than one, MUST be a configuration error naming the entry.

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

#### Scenario: Ignored example citation

- **GIVEN** a spec quoting `spec:alpha § Some rule`
- **AND** a `[[lint.ignore_evidence]]` entry with
  `citation = "alpha § Some rule"`
- **WHEN** the lint runs
- **THEN** it reports no dangling citation for `alpha § Some rule`

#### Scenario: Ignored example path

- **GIVEN** a spec citing the path `src/a.rs`, which does not exist
- **AND** a `[[lint.ignore_evidence]]` entry with `path = "src/a.rs"`
- **WHEN** the lint runs
- **THEN** it reports no missing path for `src/a.rs`

#### Scenario: Ignored citation during a review

- **GIVEN** a delta quoting `spec:alpha § Some rule`
- **AND** a `[[lint.ignore_evidence]]` entry with
  `citation = "alpha § Some rule"`
- **WHEN** the tool reviews that change
- **THEN** it reports no dangling citation on that pairing

#### Scenario: Evidence entry names no kind

- **GIVEN** a `[[lint.ignore_evidence]]` entry with only `reason`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

#### Scenario: Evidence entry names two kinds

- **GIVEN** a `[[lint.ignore_evidence]]` entry with `path` and `commit`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

### Requirement: A dangling ignore entry is a warning

An ignore entry is dangling when a scope in its `in` does not resolve
against the register, when its `requirement` does not resolve against the
register, when its `term` is a glossary term, an admitted synonym or a
deprecated synonym, when the evidence it names is not missing, or when it
silenced no finding in the run. Each MUST produce a warning naming the
entry, the scope at fault and `openspec/reviewer.toml`, asking for it to
be removed. Scopes MUST be judged one at a time, so a live scope does not
mask a dead one. `lint` and `change` MUST reach the same verdict for the
same entry.

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

#### Scenario: Unknown capability in a scope

- **GIVEN** an ignore with `in = ["sitemap-index"]`
- **AND** no such capability in the register
- **WHEN** the lint runs
- **THEN** it reports a warning naming that scope

#### Scenario: Same verdict during a review

- **GIVEN** an ignore entry the lint reports as dangling
- **WHEN** the tool reviews any open change
- **THEN** it reports the same warning

#### Scenario: Ignored path now exists

- **GIVEN** a `[[lint.ignore_evidence]]` entry with `path = "src/a.rs"`
- **AND** `src/a.rs` present in the repository
- **WHEN** the lint runs
- **THEN** it reports a warning naming that entry
- **AND** naming `openspec/reviewer.toml`
