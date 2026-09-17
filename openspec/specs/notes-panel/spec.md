# notes-panel Specification

## Purpose
TBD - created by archiving change notes-panel. Update Purpose after archive.
## Requirements
### Requirement: The notes panel lists every note of the change

`N` in the browsing view MUST open a panel over both panes listing every
note of the change: artefact notes first, then requirement and scenario
notes in the order of the main list. Each row MUST show the note's
anchor as plain output names it, capability, section sign and
requirement, followed by a chevron and the scenario for a scenario
note, or the artefact's name for an artefact note; then the first line
of the note's text; then a mark when the text it was written about has
changed. The title MUST name the count. A change with no note MUST
read "no notes". Escape MUST close the panel with the main list
unchanged.

#### Scenario: Three kinds of note

- **GIVEN** a note on the artefact `design.md`
- **AND** a note on requirement `Rotation` of capability `keys`
- **AND** a note on its scenario `Expired key`
- **WHEN** the reviewer opens the panel
- **THEN** the panel lists the artefact, then `keys § Rotation`, then
  `keys § Rotation › Expired key`
- **AND** each row shows the first line of its note

#### Scenario: Outdated note

- **GIVEN** a note whose anchor text changed since it was written
- **WHEN** the panel opens
- **THEN** that row carries the outdated mark

#### Scenario: No notes

- **GIVEN** a change with no note
- **WHEN** the reviewer opens the panel
- **THEN** the panel says it has no notes

#### Scenario: Close

- **GIVEN** the panel open
- **WHEN** the reviewer presses escape
- **THEN** the main list returns with the same cursor position

### Requirement: Enter jumps to the note's row

`Enter` on a panel row MUST close the panel and move the main list's
cursor to the note's anchor, unfolding the requirement when the anchor
is one of its scenarios. The detail pane MUST show that row.

#### Scenario: Scenario note under a folded requirement

- **GIVEN** the panel row of a scenario note
- **AND** its requirement folded in the main list
- **WHEN** the reviewer presses enter
- **THEN** the requirement unfolds
- **AND** the cursor is on the scenario row
- **AND** the detail pane shows the note under the diff

### Requirement: A note can be deleted from the panel

`d` on a panel row MUST remove that note from the local state as saving
it empty would. The panel MUST stay open with the row gone and the
selection on the next row, or the last when the deleted row was last.
Approvals MUST NOT change.

#### Scenario: Delete one of three

- **GIVEN** the panel with three notes and the second selected
- **WHEN** the reviewer deletes it
- **THEN** the panel lists two notes
- **AND** the selection is on the note that followed
- **AND** the state file no longer holds the deleted note

### Requirement: Every note can be cleared after confirmation

`X` in the panel MUST open a confirmation naming the number of notes.
`y` MUST remove every note of the change from the local state, keep
every approval, and leave the panel open saying it has no notes. Any
other key MUST return to the panel with nothing removed.

#### Scenario: Confirm

- **GIVEN** a change with four notes and two approvals
- **AND** the panel open
- **WHEN** the reviewer asks to clear and confirms
- **THEN** the panel says it has no notes
- **AND** the state file holds no note
- **AND** the state file still holds both approvals

#### Scenario: Decline

- **GIVEN** the panel open with notes
- **WHEN** the reviewer asks to clear and declines
- **THEN** the panel lists the same notes
- **AND** the state file is unchanged

