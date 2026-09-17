# change-model Specification

## Purpose
TBD - created by archiving change reviewer-foundation. Update Purpose after archive.
## Requirements
### Requirement: A spec file parses into requirements and scenarios

The tool MUST parse a spec file into requirements and their scenarios. A
requirement starts at a `### Requirement:` heading and ends at the next
`###` or `##` heading. A scenario starts at a `#### Scenario:` heading and
ends at the next `####`, `###` or `##` heading. Text between a requirement
heading and its first scenario is the requirement body. The name of a
requirement or scenario is the heading text after the colon, trimmed.

#### Scenario: Requirement with two scenarios

- **GIVEN** a file with one `### Requirement:` heading
- **AND** two `#### Scenario:` headings under it
- **WHEN** the tool parses the file
- **THEN** it yields one requirement with two scenarios
- **AND** each scenario has its own name and body

#### Scenario: Body text stops at the next requirement

- **GIVEN** two requirements in one file
- **WHEN** the tool parses the file
- **THEN** the first requirement's body holds no text of the second
- **AND** the first requirement's scenarios hold no text of the second

### Requirement: Canon is every spec under the specs directory

The tool MUST read canon from `openspec/specs/<capability>/spec.md` in the
working directory, one capability per directory. The capability name is
the directory name.

#### Scenario: Two capabilities in canon

- **GIVEN** `openspec/specs/alpha/spec.md`
- **AND** `openspec/specs/beta/spec.md`
- **WHEN** the tool loads canon
- **THEN** canon has the capabilities `alpha` and `beta`
- **AND** each holds the requirements its file declares

#### Scenario: No specs directory

- **GIVEN** a working directory without `openspec/specs/`
- **WHEN** the tool loads canon
- **THEN** it stops with an error saying it found no OpenSpec canon here

### Requirement: A delta groups requirements by kind

The tool MUST read a delta spec at
`openspec/changes/<change>/specs/<capability>/spec.md` and group its
requirements under the `##` section they appear in: `ADDED`, `MODIFIED`,
`REMOVED` or `RENAMED`. A requirement under any other `##` heading is a
parse error that names the file and the heading.

#### Scenario: Mixed delta

- **GIVEN** a delta with one requirement under `## ADDED Requirements`
- **AND** two under `## MODIFIED Requirements`
- **WHEN** the tool parses the delta
- **THEN** it yields one ADDED entry
- **AND** two MODIFIED entries

#### Scenario: Unknown section

- **GIVEN** a delta with a requirement under `## Changed Requirements`
- **WHEN** the tool parses the delta
- **THEN** it fails
- **AND** the error names the file and the heading

### Requirement: A rename reads a FROM line and a TO line

Under `## RENAMED Requirements` the tool MUST read `- FROM:` and `- TO:`
lines in twos. Each line carries a backticked `### Requirement: <name>`
heading. The two together yield a RENAMED entry with the old and new
name. A `FROM` without a following `TO`, or the reverse, is a parse
error.

#### Scenario: One rename

- **GIVEN** a `- FROM:` line naming A
- **AND** a `- TO:` line naming B
- **WHEN** the tool parses the section
- **THEN** it yields one RENAMED entry from A to B

#### Scenario: Orphan FROM

- **GIVEN** a `- FROM:` line with no `- TO:` after it
- **WHEN** the tool parses the section
- **THEN** it fails
- **AND** the error names the file and the line

### Requirement: A rename joins the modified entry with the new name

When a delta has a RENAMED entry from A to B and a MODIFIED entry named B,
the tool MUST treat them as one entry: kind RENAMED from A, body from the
MODIFIED entry. When no MODIFIED entry named B exists, the entry keeps the
canon body of A.

#### Scenario: Rename with body change

- **GIVEN** a delta that renames A to B
- **AND** lists B under MODIFIED
- **WHEN** the tool builds the review
- **THEN** it shows one entry named B, renamed from A
- **AND** the MODIFIED body is its after side

#### Scenario: Rename only

- **GIVEN** a delta that renames A to B
- **AND** does not list B under MODIFIED
- **WHEN** the tool builds the review
- **THEN** it shows one entry named B, renamed from A
- **AND** the after side equals the canon body of A

### Requirement: A change is its artefacts plus its deltas

The tool MUST read a change at `openspec/changes/<change>/` as its
artefacts (`proposal.md`, `design.md`, `tasks.md`, each optional) and its
delta specs. A change directory with no `specs/` subdirectory and no
artefacts is an error. Archived changes under `openspec/changes/archive/`
are not changes the tool reviews; they are history.

#### Scenario: Change with three artefacts and two deltas

- **GIVEN** a change directory with `proposal.md`, `design.md` and
  `tasks.md`
- **AND** two capability directories under `specs/`
- **WHEN** the tool loads the change
- **THEN** it has three artefacts
- **AND** two delta specs

#### Scenario: Change name under archive

- **GIVEN** a name that resolves only under `openspec/changes/archive/`
- **WHEN** the tool loads the change
- **THEN** it stops with an error saying the change is archived

### Requirement: The register is every requirement canon and open changes assert

The tool MUST build a register once per run, holding every requirement of
canon and every requirement any open change adds, renames or modifies,
each identified as `<capability> § <requirement name>`. The register MUST
be ordered by capability name, then by requirement name. `lint` and
`change` MUST build the same register from the same working directory and
differ only in what they report from it.

#### Scenario: Canon and an open change

- **GIVEN** canon with `alpha § One` and `beta § Two`
- **AND** an open change adding `alpha § Three`
- **WHEN** the tool builds the register
- **THEN** the register holds all three

#### Scenario: Stable order

- **GIVEN** a register holding `beta § Two` and `alpha § One`
- **WHEN** the tool reports it
- **THEN** `alpha § One` comes before `beta § Two`

#### Scenario: Same register in both commands

- **GIVEN** a working directory with canon and two open changes
- **WHEN** the lint builds the register
- **AND** a review of one change builds the register
- **THEN** the two registers hold the same entries
