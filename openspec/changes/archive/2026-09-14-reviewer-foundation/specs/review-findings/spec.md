# review-findings (delta)

## ADDED Requirements

### Requirement: A finding has a severity, a location and a message

Every finding the tool reports MUST carry a severity (`error`, `warning`
or `note`), a location (change, capability, requirement, and scenario
when it applies) and a one-line message. Findings of one kind always have
the same severity.

#### Scenario: Finding shape

- **WHEN** the tool reports any finding
- **THEN** it has a severity
- **AND** a location
- **AND** a message that fits on one line

### Requirement: A modified requirement must exist in canon

When a MODIFIED entry names a requirement canon does not have in that
capability, the tool MUST report an error finding "modified without
canon".

#### Scenario: Typo in requirement name

- **GIVEN** a delta that modifies "Flat index of all routes"
- **AND** canon that has "Flat index of all routes and pages"
- **WHEN** the tool reviews the delta
- **THEN** it reports an error naming the delta's requirement
- **AND** shows the entry as added text, not a diff

### Requirement: An added requirement must be new

When an ADDED entry names a requirement canon already has in that
capability, the tool MUST report an error finding "added already exists"
and show the entry as a word diff against the existing text.

#### Scenario: Added over existing

- **GIVEN** a delta that adds a requirement under a name canon uses
- **WHEN** the tool reviews the delta
- **THEN** it reports an error
- **AND** shows a word diff against the existing text

### Requirement: A removed or renamed requirement must exist in canon

When a REMOVED entry, or the `FROM` side of a RENAMED entry, names a
requirement canon does not have, the tool MUST report an error finding.
When the `TO` side of a RENAMED entry names a requirement canon already
has, the tool MUST report an error finding "rename target taken".

#### Scenario: Rename from a missing name

- **GIVEN** a RENAMED entry whose `FROM` is not in canon
- **WHEN** the tool reviews the delta
- **THEN** it reports an error naming the old name

#### Scenario: Rename onto an existing name

- **GIVEN** a RENAMED entry whose `TO` matches another canon requirement
- **WHEN** the tool reviews the delta
- **THEN** it reports an error naming the new name

### Requirement: A dropped scenario is a warning

When a MODIFIED or RENAMED entry's after side lacks a scenario the canon
side has, the tool MUST report a warning finding "scenario dropped" that
names the scenario.

#### Scenario: One scenario vanishes

- **GIVEN** canon with three scenarios
- **AND** a delta that restates two of them
- **WHEN** the tool reviews the delta
- **THEN** it warns about the third by name

### Requirement: A requirement without scenarios is a warning

When any ADDED, MODIFIED or RENAMED entry's after side has no scenario,
the tool MUST report a warning finding "no scenarios".

#### Scenario: Bare requirement

- **GIVEN** an ADDED entry with a body and no `#### Scenario:` heading
- **WHEN** the tool reviews the delta
- **THEN** it warns about that requirement

### Requirement: A collision with another open change is a warning

When another change under `openspec/changes/`, not archived and not the
one under review, has an entry for the same capability and requirement
name, the tool MUST report a warning finding "also touched by <change>".

#### Scenario: Two changes modify one requirement

- **GIVEN** the change under review lists a requirement under MODIFIED
- **AND** another open change lists the same requirement under MODIFIED
- **WHEN** the tool reviews the change
- **THEN** it warns
- **AND** names the other change

#### Scenario: Other change is archived

- **GIVEN** the only other change touching the requirement is under
  `archive/`
- **WHEN** the tool reviews the change
- **THEN** it reports no collision

### Requirement: An unchanged modification is a note

When a MODIFIED entry's after side equals the canon side after
normalization, scenarios included, the tool MUST report a note finding
"no change".

#### Scenario: Restated verbatim

- **GIVEN** a delta that restates a requirement identically apart from
  line wrapping
- **WHEN** the tool reviews the delta
- **THEN** it reports a note
- **AND** shows no diff marks

### Requirement: Exit status reflects the worst finding

In plain output the tool MUST exit `2` when any error finding exists, `1`
when only warnings exist, and `0` otherwise. In the interactive view the
exit status is `0` when the user quits.

#### Scenario: Warnings only

- **GIVEN** a review with two warnings
- **AND** no errors
- **WHEN** the tool prints plain output
- **THEN** it exits with status `1`
