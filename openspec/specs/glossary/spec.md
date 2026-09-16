# glossary Specification

## Purpose
TBD - created by archiving change reviewer-definitions. Update Purpose after archive.
## Requirements
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

For every term, the tool MUST check whether the term or any of its
admitted synonyms appears, as a case-insensitive whole-word match, in
any canon requirement outside `definitions` or in any delta of an open
change. A term whose name and admitted synonyms all appear nowhere MUST
produce a note finding "defined but unused" in `lint` and on the term's
pairing when a change touches it.

#### Scenario: Orphan term

- **GIVEN** a term `loket` used in no canon requirement
- **AND** no open change using it
- **WHEN** the lint runs
- **THEN** it reports a note naming `loket`

#### Scenario: Used only under an admitted synonym

- **GIVEN** a term `manager` with admitted synonym `steward`
- **AND** canon requirements that say `steward` and never say `manager`
- **WHEN** the lint runs
- **THEN** it reports no note for `manager`

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
the diff listing each glossary term whose name or admitted synonym
appears in the pairing's after text, with its meaning, its admitted
synonyms and its deprecated synonyms. Admitted synonyms are listed above
deprecated ones. Terms are listed in order of first appearance, where a
term's first appearance is the earliest match of its name or any
admitted synonym. When no term appears the panel says so.

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

#### Scenario: Pairing names a term only by an admitted synonym

- **GIVEN** a pairing whose after text contains `steward` and not `manager`
- **AND** a term `manager` with admitted synonym `steward`
- **WHEN** the reviewer presses `D`
- **THEN** the panel lists `manager`
- **AND** shows `steward` under its admitted synonyms

### Requirement: Definitions findings reach plain output and JSON

Every definitions finding MUST appear in plain text, `--findings-only`
and JSON like any other finding. JSON MUST also carry the glossary as a
top-level `definitions` array of term, meaning, admitted synonyms and
deprecated synonyms.

#### Scenario: Agent reads the glossary

- **WHEN** an agent runs the tool with `--format json`
- **THEN** the document has a `definitions` array
- **AND** each entry has `term`, `meaning`, `admitted` and `deprecated`

### Requirement: A term lists the words that are acceptable for it

A `- **Admitted:**` line in a term's body MUST be read as a
comma-separated list of admitted synonyms. The line is part of the
meaning text for diffing but is not shown as prose in the definitions
panel; the panel shows it as a list. An admitted synonym MUST NOT
produce a finding of its own.

#### Scenario: Two admitted synonyms

- **GIVEN** a term whose body has `- **Admitted:** steward, custodian`
- **WHEN** the tool parses the term
- **THEN** its admitted synonyms are `steward` and `custodian`

#### Scenario: No Admitted line

- **GIVEN** a term whose body has no `- **Admitted:**` line
- **WHEN** the tool parses the term
- **THEN** its admitted synonyms are empty

#### Scenario: Both lines

- **GIVEN** a term whose body has an `- **Admitted:**` line
- **AND** a `- **Deprecated:**` line
- **WHEN** the tool parses the term
- **THEN** the meaning text holds neither line

#### Scenario: Using an admitted synonym

- **GIVEN** a term `manager` with admitted synonym `steward`
- **AND** a delta scenario saying "the steward approves the invite"
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for that word

#### Scenario: Word on both lines

- **GIVEN** a term whose `- **Admitted:**` line and `- **Deprecated:**`
  line both list `steward`
- **AND** a delta saying "the steward approves the invite"
- **WHEN** the tool reviews the change
- **THEN** it reports the deprecated synonym warning

#### Scenario: Unrecognized bold line

- **GIVEN** a term whose body has a `- **Example:**` line
- **WHEN** the tool parses the term
- **THEN** the meaning text still holds that line

### Requirement: A term opens with a binding line

A term's body MUST open with `A spec MUST use <term> to mean:`, where
`<term>` is the requirement name, backticked or bare. The tool MUST strip
that line from the meaning text, as it strips `- **Admitted:**` and
`- **Deprecated:**`. The full body, binding line included, stays the text
a pairing diffs. A term whose body has no binding line, or whose binding
line names something other than the requirement name, MUST produce a
warning finding "term without a binding line" naming the term, in `lint`
and on the term's pairing when a change touches it.

#### Scenario: Meaning excludes the line

- **GIVEN** a term `register` whose body opens with
  ``A spec MUST use `register` to mean:``
- **WHEN** the tool parses the term
- **THEN** the meaning text does not hold that line

#### Scenario: Panel shows the meaning

- **GIVEN** a pairing whose after text contains `register`
- **WHEN** the reviewer presses `D`
- **THEN** the panel shows the meaning without the binding line

#### Scenario: No binding line

- **GIVEN** a term whose body opens with its meaning
- **WHEN** the lint runs
- **THEN** it reports a warning naming that term

#### Scenario: Binding line names another term

- **GIVEN** a term `register` whose body opens with
  ``A spec MUST use `snapshot` to mean:``
- **WHEN** the lint runs
- **THEN** it reports a warning naming `register`

#### Scenario: Editing the line is a change to the term

- **GIVEN** a change whose delta edits only a term's binding line
- **WHEN** the tool reviews the change
- **THEN** the pairing shows a diff

