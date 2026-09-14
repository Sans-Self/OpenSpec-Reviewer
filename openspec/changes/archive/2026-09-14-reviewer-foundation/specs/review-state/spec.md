# review-state (delta)

## ADDED Requirements

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

### Requirement: A requirement can carry a note

The reviewer MUST be able to attach one free-text note per requirement
pairing. Editing opens `$EDITOR` on the note with the interactive view
suspended, and resumes the view on exit. A row with a note shows a `✎`
marker. Without `$EDITOR` the tool falls back to `vi`.

#### Scenario: Write a note

- **GIVEN** the cursor on a requirement
- **WHEN** the reviewer presses `e`
- **AND** saves text in the editor
- **AND** the editor exits
- **THEN** the view resumes
- **AND** the row shows `✎`
- **AND** the detail pane shows the note under the diff

#### Scenario: Empty note

- **WHEN** the reviewer saves an empty file in the editor
- **THEN** the note is removed
- **AND** the `✎` marker is gone

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

Plain text and JSON MUST include each pairing's approval state and note.
`--format markdown` MUST print only the notes, grouped by capability and
requirement, in a shape suited to pasting into a pull request review.

#### Scenario: Export notes for a PR

- **GIVEN** three requirements with notes
- **WHEN** the reviewer runs the tool with `--format markdown`
- **THEN** the output has one heading per capability
- **AND** one bullet per noted requirement with its note text
