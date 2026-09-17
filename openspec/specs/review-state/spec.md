# review-state Specification

## Purpose
TBD - created by archiving change reviewer-foundation. Update Purpose after archive.
## Requirements
### Requirement: A requirement can be approved

The reviewer MUST be able to mark each requirement pairing as approved
and unmark it again. The list shows `[√]` for approved and `[ ]` for not
yet approved. Artefact rows can be approved the same way.

#### Scenario: Approve a requirement

- **GIVEN** the cursor on an unapproved requirement
- **WHEN** the reviewer presses `a`
- **THEN** the row shows `[√]`
- **AND** the status line's approved count rises by one

#### Scenario: Unapprove

- **GIVEN** the cursor on an approved requirement
- **WHEN** the reviewer presses `a`
- **THEN** the row shows `[ ]`

### Requirement: A requirement or a scenario can carry a note

The reviewer MUST be able to attach one free-text note to a requirement
pairing and one to each of its scenarios. A note on a requirement is
keyed `<capability>/<requirement>`; a note on a scenario is keyed
`<capability>/<requirement>#<scenario>`. A row with a note shows a `✎`
marker, and a folded requirement shows it when any scenario under it
holds one. Saving an empty note removes it.

#### Scenario: Write a note

- **GIVEN** the cursor on a requirement
- **WHEN** the reviewer writes a note and saves it
- **THEN** the row shows `✎`
- **AND** the detail pane shows the note under the diff
- **AND** the note is stored under `<capability>/<requirement>`

#### Scenario: Write a note on a scenario

- **GIVEN** the cursor on a scenario row
- **WHEN** the reviewer writes a note and saves it
- **THEN** the row shows `✎`
- **AND** the note is stored under that scenario's key

#### Scenario: Empty note

- **WHEN** the reviewer saves an empty note
- **THEN** the note is removed
- **AND** the `✎` marker is gone

#### Scenario: Two scenarios of one requirement

- **GIVEN** a requirement with two scenarios
- **WHEN** the reviewer notes each of them
- **THEN** both notes are kept
- **AND** neither replaces the other

### Requirement: State persists between runs

Approvals and notes MUST be stored under
`$XDG_STATE_HOME/openspec-reviewer/<repo>/<change>.json`, where `<repo>`
is the origin remote URL made path-safe, or the absolute path of the git
top level when there is no origin. `$XDG_STATE_HOME` defaults to
`~/.local/state`. The file is written on every state change. `--no-state`
disables both reading and writing.

#### Scenario: Reopen the same change

- **GIVEN** a previous run approved two requirements
- **WHEN** the reviewer runs the tool on the same change in the same
  repository
- **THEN** those two rows show `[√]`

#### Scenario: Different repository, same change name

- **GIVEN** two repositories that both have a change named `foo`
- **WHEN** the reviewer approves a requirement in one
- **THEN** the other shows nothing approved

### Requirement: An approval is tied to the text it approved

Each approval MUST store a hash of the normalized after text of the
pairing. When the text differs at a later run, the row shows `[~]` and
the detail pane says the text changed since approval. Pressing `a`
re-approves against the new text.

#### Scenario: Requirement edited after approval

- **GIVEN** an approved requirement
- **WHEN** a later run finds the after text changed
- **THEN** the row shows `[~]`
- **AND** the status line counts it as not approved

### Requirement: Plain output includes state

Plain text and JSON MUST include each pairing's approval state and
notes, each note naming its anchor and whether it is outdated.
`--format markdown` MUST print only the notes, grouped by capability and
requirement with a scenario's note nested under its requirement, in a
shape suited to pasting into a pull request review. An outdated note
MUST be marked as such in that output.

#### Scenario: Export notes for a PR

- **GIVEN** three requirements with notes
- **WHEN** the reviewer runs the tool with `--format markdown`
- **THEN** the output has one heading per capability
- **AND** one bullet per noted requirement with its note text

#### Scenario: Export a scenario's note

- **GIVEN** a note on a scenario of a noted requirement
- **WHEN** the reviewer runs the tool with `--format markdown`
- **THEN** that note is nested under its requirement's bullet
- **AND** it names the scenario

### Requirement: A note records the text it was written about

Each note MUST store a hash of the normalized text of its anchor at the
time it was written, and the time it was written. When the anchor's text
differs at a later run, the detail pane MUST say the text changed since
the note was written. The row marker MUST stay `✎`. A note read from a
state file that holds it as a bare string MUST load as a note with no
hash, and MUST NOT report as outdated until it is next saved.

#### Scenario: Scenario reworded after a note

- **GIVEN** a note on a scenario
- **WHEN** a later run finds that scenario's text changed
- **THEN** the detail pane says the text changed since the note was
  written

#### Scenario: Note from an older state file

- **GIVEN** a state file holding a note as a bare string
- **WHEN** the tool reads it
- **THEN** the note loads with its text
- **AND** it does not report as outdated

