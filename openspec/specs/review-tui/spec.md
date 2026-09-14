# review-tui Specification

## Purpose
TBD - created by archiving change reviewer-foundation. Update Purpose after archive.
## Requirements
### Requirement: The interactive view opens when stdout is a terminal

The tool MUST open the interactive view when stdout is a terminal and no
output format flag is given. `--plain` or `--format` selects plain output
instead.

#### Scenario: Run from a shell

- **GIVEN** stdout is a terminal
- **WHEN** the user runs a subcommand with no format flag
- **THEN** the interactive view opens

#### Scenario: Forced plain

- **GIVEN** stdout is a terminal
- **WHEN** the user passes `--plain`
- **THEN** the tool prints plain output
- **AND** exits

### Requirement: The view is a list and a detail pane

The interactive view MUST show a list on the left and a detail pane on
the right. The list has one row per review item: each artefact of the
change, then each requirement pairing grouped under a capability heading
row. Heading rows are not selectable. A requirement row shows, in order:
approval mark (`[ ]`, `[√]` or `[~]`), kind glyph (`+` added, `~`
modified, `-` removed, `>` renamed), name, then `!` for an error finding
or `?` for a warning, and `✎` when it carries a note. When the snapshot
holds more than one change, each change gets its own heading row above
its capabilities.

#### Scenario: List for one change

- **GIVEN** a change with two artefacts
- **AND** one capability with three requirements
- **WHEN** the view opens
- **THEN** the list shows two artefact rows
- **AND** one capability heading
- **AND** three requirement rows with their marks and glyphs

#### Scenario: Requirement with an error

- **GIVEN** a requirement with an error finding
- **WHEN** the view renders its row
- **THEN** the row shows `!` after the name

### Requirement: The detail pane follows the list selection

Selecting a requirement row MUST show its semantic diff, its findings,
its note, and its one-line history summary in the detail pane. Selecting
an artefact row shows its line diff. The detail pane scrolls with
`PgUp`/`PgDn` and `Ctrl-u`/`Ctrl-d`.

#### Scenario: Select a modified requirement

- **GIVEN** the cursor on a modified requirement
- **WHEN** the view renders
- **THEN** the detail pane shows its word-level diff
- **AND** its findings below the diff
- **AND** its note, when it has one

### Requirement: The detail pane has three display modes

The detail pane MUST offer inline (word diff in one column),
side-by-side (before left, after right), and raw (the snapshot's before
and after text as-is). `m` cycles them and the status line names the
current mode.

#### Scenario: Cycle modes

- **GIVEN** the detail pane in inline mode
- **WHEN** the user presses `m` three times
- **THEN** the pane shows side-by-side, then raw, then inline again

### Requirement: The status line shows progress

The status line MUST show the change name, the count of approved items
over the total, the count of findings per severity, and the current
display mode.

#### Scenario: Two of five approved

- **GIVEN** five requirement rows with two approved
- **WHEN** the view renders
- **THEN** the status line reads `2/5 approved`

### Requirement: Keys follow vi and arrow conventions

The tool MUST bind: `j`/`k` and arrows to move in the list, `Tab` to
switch pane focus, `n`/`p` to jump to the next and previous row with a
finding, `a` to toggle approval, `e` to edit the note, `H` to open
history, `m` to cycle display mode, `?` for a help overlay, `q` and `Esc`
to quit. The help overlay lists every binding.

#### Scenario: Jump to next finding

- **GIVEN** a later row with a finding
- **WHEN** the user presses `n`
- **THEN** the cursor moves to that row
- **AND** the detail pane shows it

#### Scenario: Help

- **WHEN** the user presses `?`
- **THEN** an overlay lists every binding
- **AND** any key closes it

### Requirement: Colour is never the only signal

Every marked change MUST carry a glyph as well as a colour: `-` for
removed, `+` for added, `~` for changed. When `NO_COLOR` is set or the
terminal reports no colour, the tool MUST render with glyphs and bold or
reverse video only.

#### Scenario: NO_COLOR

- **GIVEN** `NO_COLOR` is set
- **WHEN** the view renders
- **THEN** it uses no colour
- **AND** every change is still marked by its glyph

### Requirement: The view restores the terminal on exit

On quit, panic or signal, the tool MUST leave the alternate screen and
restore the terminal mode so the shell is usable.

#### Scenario: Panic inside the view

- **WHEN** the tool panics while drawing
- **THEN** the terminal is restored
- **AND** the panic message prints afterwards

