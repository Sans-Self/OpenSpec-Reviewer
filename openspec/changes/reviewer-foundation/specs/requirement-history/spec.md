# requirement-history (delta)

## ADDED Requirements

### Requirement: History comes from archived changes

For a requirement in a capability, the tool MUST collect every archived
change under `openspec/changes/archive/` whose delta for that capability
has an ADDED, MODIFIED, REMOVED or RENAMED entry with that name. Entries
are ordered by the archive directory name, which starts with the archive
date. Each entry carries the archived change name, the kind, and the
text of the requirement in that delta.

#### Scenario: Requirement changed twice before

- **GIVEN** two archived changes that both list the requirement under
  MODIFIED
- **WHEN** the reviewer opens history for it
- **THEN** the list shows two entries in archive order
- **AND** each entry names its archived change and kind

#### Scenario: Never touched

- **GIVEN** a requirement no archived change mentions
- **WHEN** the reviewer opens history for it
- **THEN** the view says there is no history
- **AND** shows the canon text as the only version

### Requirement: History follows renames backwards

When an archived change renamed a requirement to the current name, the
tool MUST continue collecting under the old name in earlier archives.
The rename entry itself appears in the list with both names.

#### Scenario: Renamed once

- **GIVEN** an archived change that renamed A to B
- **AND** an older archived change that added A
- **WHEN** the reviewer opens history for B
- **THEN** the list shows the ADDED entry for A
- **AND** the RENAMED entry from A to B
- **AND** any later entries for B

### Requirement: The history view is a list and a version

The history view MUST show the entries as a list on the left and the
selected version's text on the right. The version under review appears
as the last entry, marked `current`. Pressing `m` toggles the right pane
between the version's text and its diff against the previous entry.

#### Scenario: Open history

- **GIVEN** the cursor on a requirement in the main list
- **WHEN** the reviewer presses `H`
- **THEN** the history view opens with the newest entry selected

#### Scenario: Diff against previous

- **GIVEN** the history view on the second of three entries
- **WHEN** the reviewer presses `m`
- **THEN** the right pane shows the word diff from the first entry to the
  second

#### Scenario: Close history

- **WHEN** the reviewer presses `Esc` in the history view
- **THEN** the main list returns with the same cursor position

### Requirement: History is available in plain output

`--format json` MUST include each pairing's history entries with change
name, kind and text. Plain text MUST include a one-line history summary
per requirement, naming the archived changes in order.

#### Scenario: JSON history

- **WHEN** an agent reads the JSON output
- **THEN** each pairing has a `history` array with one object per
  archived entry
