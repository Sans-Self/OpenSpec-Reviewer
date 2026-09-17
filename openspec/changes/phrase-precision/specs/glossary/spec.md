# glossary (delta)

## MODIFIED Requirements

### Requirement: A deprecated synonym in a spec is a warning

For every deprecated synonym of every term, the tool MUST search the
normalized text of every canon requirement outside `definitions` and
every delta of the change under review, as a case-insensitive whole-word
match, ignoring `spec:` citations. The longest phrase wins: every term
name and admitted synonym of two or more words shields the text it
matches, and a hit that lies inside a shielded span MUST NOT produce a
finding. A term name or admitted synonym of one word MUST NOT shield.
Each remaining hit MUST produce a warning finding "uses deprecated
synonym" naming the synonym, the term, and the requirement or scenario
containing it. A hit in a delta is reported on that pairing; a hit in
canon is reported on the term's pairing when the change touches the term,
and in `lint` otherwise.

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

#### Scenario: Synonym is the tail of a term name

- **GIVEN** a term `group key` with deprecated synonym `rotation key`
- **AND** a term named `PLC rotation key`
- **AND** a delta saying "the PLC rotation key signs the operation"
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for that sentence

#### Scenario: Synonym outside the term name that contains it

- **GIVEN** a term `group key` with deprecated synonym `rotation key`
- **AND** a term named `PLC rotation key`
- **AND** a delta saying "the rotation key wraps the content key"
- **WHEN** the tool reviews the change
- **THEN** it reports one finding naming `rotation key` and `group key`

#### Scenario: Synonym shielded by an admitted phrase

- **GIVEN** a term `verification method` with deprecated synonym `anchor`
- **AND** a term `lineage` with admitted synonym `lineage anchor`
- **AND** a delta saying "the lineage anchor resolves to the genesis URI"
- **WHEN** the tool reviews the change
- **THEN** it reports no finding for that sentence

#### Scenario: A one-word admitted synonym does not shield

- **GIVEN** a term `verification method` with deprecated synonym `anchor`
- **AND** a term `lineage` with admitted synonym `anchor`
- **AND** a delta saying "the anchor is checked before the wrap"
- **WHEN** the tool reviews the change
- **THEN** it reports one finding naming `anchor` and `verification method`
