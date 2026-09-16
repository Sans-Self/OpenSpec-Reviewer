# glossary (delta)

## ADDED Requirements

### Requirement: The glossary is a capability named definitions

The tool MUST treat the canon spec `openspec/specs/definitions/spec.md`
as the project's glossary. Each requirement in it is a term. The
requirement name is the term, the body is its meaning, and its scenarios
are usage examples. Deltas for `definitions` in a change are term
additions, changes, removals and renames, reviewed like any other
pairing. The capability name is configurable as `definitions.capability`
in `openspec/reviewer.toml`.

#### Scenario: Two terms in canon

- **GIVEN** `openspec/specs/definitions/spec.md` with requirements
  `group key` and `manager`
- **WHEN** the tool loads canon
- **THEN** the glossary has two terms
- **AND** each term has the body of its requirement as meaning

#### Scenario: No glossary

- **GIVEN** a repository without a `definitions` capability
- **WHEN** the tool runs
- **THEN** every definitions check is skipped
- **AND** the summary line says the project has no glossary

### Requirement: A term lists the words not to use for it

A `- **Deprecated:**` line in a term's body MUST be read as a comma-separated
list of deprecated synonyms. The line is part of the meaning text for
diffing but is not shown as prose in the definitions panel; the panel
shows it as a list.

#### Scenario: Two synonyms

- **GIVEN** a term whose body has `- **Deprecated:** workspace key, rotation key`
- **WHEN** the tool parses the term
- **THEN** its deprecated synonyms are `workspace key` and `rotation key`

#### Scenario: No Deprecated line

- **GIVEN** a term whose body has no `- **Deprecated:**` line
- **WHEN** the tool parses the term
- **THEN** its deprecated synonyms are empty

### Requirement: A deprecated synonym in a spec is a warning

For every deprecated synonym of every term, the tool MUST search the
normalized text of every canon requirement outside `definitions` and
every delta of the change under review, as a case-insensitive whole-word
match, ignoring `spec:` citations. Each hit MUST produce a warning
finding "uses deprecated synonym" naming the synonym, the term, and the
requirement or scenario containing it. A hit in a delta is reported on
that pairing; a hit in canon is reported on the term's pairing when the
change touches the term, and in `lint` otherwise.

#### Scenario: Delta says workspace key

- **GIVEN** a glossary term `group key` with deprecated synonym
  `workspace key`
- **AND** a delta scenario saying "when the workspace key rotates"
- **WHEN** the tool reviews the change
- **THEN** that pairing has a warning naming `workspace key` and `group key`
- **AND** naming the scenario

#### Scenario: Synonym is a substring

- **GIVEN** deprecated synonym `admin`
- **AND** a delta containing the word `administrative`
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for that word

#### Scenario: Synonym inside a citation

- **GIVEN** deprecated synonym `admin`
- **AND** a delta containing `` `spec:keyring-tombstones § Admin API mints invites` ``
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for that citation

### Requirement: A term nobody uses is a note

For every term, the tool MUST check whether the term appears, as a
case-insensitive whole-word match, in any canon requirement outside
`definitions` or in any delta of an open change. A term that appears
nowhere MUST produce a note finding "defined but unused" in `lint` and
on the term's pairing when a change touches it.

#### Scenario: Orphan term

- **GIVEN** a term `loket` used in no canon requirement
- **AND** no open change using it
- **WHEN** the lint runs
- **THEN** it reports a note naming `loket`

### Requirement: A recurring undefined term is a note

The tool MUST collect every backticked span and double-quoted string in
canon requirements outside `definitions`, count the requirements and the
capabilities each appears in, and report a note finding "recurring term
without definition" for each that appears in at least
`definitions.min_recurrence` requirements, default `3`, across at least
two capabilities, and is not a term or a deprecated synonym. The note
names the term and where it appears. This check runs in `lint`.

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

### Requirement: A change that introduces an undefined term is a warning

For a change under review, the tool MUST collect the backticked spans
and double-quoted strings present on any pairing's after side and absent
from every before side of the change. Each such term that is not a term
in the glossary, not a deprecated synonym, and not added to `definitions`
by the same change, and that appears in at least two pairings or twice
in one, MUST produce a warning finding "new term without definition" on
the pairing that introduces it first.

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

### Requirement: Changing a term lists the requirements that use it

For a MODIFIED or RENAMED pairing in `definitions`, the tool MUST report
a note finding "term in use" listing every canon requirement outside
`definitions` that contains the term, so the reviewer can judge whether
the new meaning still fits each use.

#### Scenario: Meaning changes

- **GIVEN** a change that modifies the term `group key`
- **AND** two canon requirements using the word
- **WHEN** the tool reviews the change
- **THEN** the pairing has a note listing the two requirements

### Requirement: The detail pane can show the terms a pairing uses

Pressing `D` on a requirement row MUST toggle a definitions panel under
the diff listing each glossary term that appears in the pairing's after
text, with its meaning and its deprecated synonyms. Terms are listed in
order of first appearance. When no term appears the panel says so.

#### Scenario: Pairing uses two terms

- **GIVEN** a pairing whose after text contains `group key` and `manager`
- **AND** both are glossary terms
- **WHEN** the reviewer presses `D`
- **THEN** the panel lists `group key` then `manager`
- **AND** shows each meaning

#### Scenario: Toggle off

- **GIVEN** the definitions panel is open
- **WHEN** the reviewer presses `D`
- **THEN** the panel closes

### Requirement: Definitions findings reach plain output and JSON

Every definitions finding MUST appear in plain text, `--findings-only`
and JSON like any other finding. JSON MUST also carry the glossary as a
top-level `definitions` array of term, meaning and deprecated synonyms.

#### Scenario: Agent reads the glossary

- **WHEN** an agent runs the tool with `--format json`
- **THEN** the document has a `definitions` array
- **AND** each entry has `term`, `meaning` and `deprecated`
