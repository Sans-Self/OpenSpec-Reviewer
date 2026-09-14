# citations (delta)

## MODIFIED Requirements

### Requirement: A citation names a capability and a requirement

The lint MUST read `spec:<capability> § <requirement name>` inside a
backtick span or a double-quoted string as a citation. The capability is
lowercase letters, digits and hyphens. The requirement name runs to the
closing delimiter, so an apostrophe does not end it. Inside a backtick
span the name MAY continue across line breaks until the closing backtick,
and a comment marker (`#`, `//`, `///`, `//!`, `*` or `--`) at the start
of a continuation line is not part of the name. Runs of whitespace in the name, line breaks included, compare as one
space, and trailing `.`, `,`, `;` or `:` is not part of the name. When
`openspec/reviewer.toml` names a call helper,
`<helper>("<capability>", "<requirement>")` with either quote style is
also a citation.

#### Scenario: Literal citation

- **GIVEN** a test title containing `` `spec:sitemap-index § Flat index of all routes and pages` ``
- **WHEN** the lint scans the file
- **THEN** it records one citation of that capability and requirement

#### Scenario: Apostrophe in the name

- **GIVEN** a citation in double quotes whose name contains `page's`
- **WHEN** the lint scans the file
- **THEN** the recorded name includes the apostrophe and the words after
  it

#### Scenario: Call helper

- **GIVEN** a config naming the helper `cite`
- **AND** source containing `cite('alpha', 'Some rule')`
- **WHEN** the lint scans the file
- **THEN** it records one citation of `alpha § Some rule`

#### Scenario: Backticked citation wraps

- **GIVEN** a comment containing `` `spec:lineage § Lineage is`` at the end of one line
- **AND** ``the chain's genesis URI` `` at the start of the next
- **WHEN** the lint scans the file
- **THEN** it records one citation of `lineage § Lineage is the chain's genesis URI`

#### Scenario: Backticked citation wraps inside a comment

- **GIVEN** a `#` comment containing `` `spec:lineage § Lineage is`` at the end of one line
- **AND** ``# the chain's genesis URI` `` on the next line
- **WHEN** the lint scans the file
- **THEN** it records one citation of `lineage § Lineage is the chain's genesis URI`

#### Scenario: Sentence-final punctuation

- **GIVEN** prose containing `See spec:alpha § Some rule.`
- **WHEN** the lint scans the file
- **THEN** it records one citation of `alpha § Some rule`

### Requirement: A citation must resolve

Every citation MUST name a capability that exists in canon or that an
open change adds to, and a requirement that exists in that capability's
canon or is listed under ADDED in an open change. Otherwise the lint
reports an error naming the citing file and the citation. When the
unresolved name ended at a line break, the error MUST add that the name
looks cut off by a line break and that a backtick span may wrap.

#### Scenario: Spec-first test

- **GIVEN** an open change that adds requirement R to capability C
- **AND** canon that does not have R
- **WHEN** a test cites `C § R`
- **THEN** the lint reports no error for it

#### Scenario: Renamed requirement

- **GIVEN** canon where R was renamed to S
- **AND** a test that still cites R
- **WHEN** the lint runs
- **THEN** it reports an error naming the test file and R

#### Scenario: Unknown capability

- **WHEN** a citation names a capability no spec and no change has
- **THEN** the lint reports an error saying the capability does not
  exist

#### Scenario: Bare prose wraps

- **GIVEN** a comment containing `See spec:alpha § Some rule that` at the end of one line
- **AND** `continues here.` on the next line
- **AND** canon `alpha § Some rule that continues here`
- **WHEN** the lint runs
- **THEN** it reports an error naming `alpha § Some rule that`
- **AND** the error says the name looks cut off by a line break
- **AND** the error says a backtick span may wrap
