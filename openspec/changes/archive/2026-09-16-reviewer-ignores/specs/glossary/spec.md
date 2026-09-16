## MODIFIED Requirements

### Requirement: A recurring undefined term is a note

The tool MUST collect every backticked span and double-quoted string in
canon requirements outside `definitions`, count the requirements and the
capabilities each appears in, and report a note finding "recurring term
without definition" for each that appears in at least
`definitions.min_recurrence` requirements, default `3`, across at least
two capabilities, and is not a term, an admitted synonym or a deprecated
synonym. A requirement whose `<capability> § <requirement name>` falls in
the scopes of a `[[definitions.ignore]]` entry for that term MUST NOT be
counted, and the note MUST NOT be reported when no requirement is left to
count. The note names the term and where it appears. This check runs in
`lint`.

#### Scenario: Identifier in three capabilities

- **GIVEN** `` `mountType` `` in one requirement of each of three
  capabilities
- **AND** no term `mountType`
- **WHEN** the lint runs
- **THEN** it reports a note naming `mountType`
- **AND** listing the three requirements

#### Scenario: Recurring inside one capability

- **GIVEN** a backticked span in five requirements of one capability
- **AND** none elsewhere
- **WHEN** the lint runs
- **THEN** it reports no note for it

#### Scenario: Recurring admitted synonym

- **GIVEN** `` `steward` `` in one requirement of each of three
  capabilities
- **AND** a term `manager` with admitted synonym `steward`
- **WHEN** the lint runs
- **THEN** it reports no note for `steward`

#### Scenario: Recurring ignored term

- **GIVEN** `` `custodian` `` in one requirement of each of three
  capabilities
- **AND** an unscoped `[[definitions.ignore]]` entry for `custodian`
- **WHEN** the lint runs
- **THEN** it reports no note for `custodian`

#### Scenario: Ignored in one capability of three

- **GIVEN** `` `custodian` `` in one requirement of each of three
  capabilities
- **AND** a `[[definitions.ignore]]` entry for `custodian` with `in`
  naming one of them
- **AND** `definitions.min_recurrence` of `3`
- **WHEN** the lint runs
- **THEN** it reports no note for `custodian`

### Requirement: A change that introduces an undefined term is a warning

For a change under review, the tool MUST collect the backticked spans
and double-quoted strings present on any pairing's after side and absent
from every before side of the change. Each such term that is not a term
in the glossary, not an admitted synonym, not a deprecated synonym, and
not added to `definitions` by the same change, and that appears in at
least two pairings or twice in one, MUST produce a warning finding "new
term without definition" on the pairing that introduces it first. A
pairing whose `<capability> § <requirement name>` falls in the scopes of
a `[[definitions.ignore]]` entry for that term MUST NOT be counted.

#### Scenario: New identifier used twice

- **GIVEN** a change whose deltas introduce `` `chainParent` `` in two
  requirements
- **AND** no glossary term for it
- **WHEN** the tool reviews the change
- **THEN** the first pairing using it has a warning naming `chainParent`

#### Scenario: New term defined in the same change

- **GIVEN** a change that introduces `` `chainParent` `` in two
  requirements
- **AND** adds a term `chainParent` to `definitions` in the same change
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for it

#### Scenario: Used once

- **GIVEN** a change that introduces a backticked span in exactly one
  place
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for it

#### Scenario: New span is an admitted synonym

- **GIVEN** a change that introduces `` `steward` `` in two requirements
- **AND** a term `manager` with admitted synonym `steward`
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for it

#### Scenario: New span is ignored

- **GIVEN** a change that introduces `` `custodian` `` in two
  requirements
- **AND** an unscoped `[[definitions.ignore]]` entry for `custodian`
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for it

#### Scenario: Ignored in one pairing of two

- **GIVEN** a change that introduces `` `custodian` `` in two pairings
- **AND** a `[[definitions.ignore]]` entry for `custodian` with `in`
  naming one of those pairings
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for it
