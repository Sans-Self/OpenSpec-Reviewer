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
row, then that requirement's scenarios beneath it. Heading rows are not
selectable. A requirement row shows, in order: fold marker (`▸` folded,
`▾` unfolded, a space when the requirement has no scenarios), approval
mark (`[ ]`, `[√]` or `[~]`), kind glyph (`+` added, `~` modified, `-`
removed, `>` renamed), name, then `!` for an error finding or `?` for a
warning, and `✎` when it or any scenario under it carries a note. A
scenario row shows no approval mark and no fold marker, and takes its
glyph from the scenario match: `+` added, `-` removed, `~` changed, none
when unchanged. When the snapshot holds more than one change, each
change gets its own heading row above its capabilities.

Scenario rows are shown only under a requirement that is open. A
requirement is open while the cursor is on it or on one of its
scenarios, and while `Space` has pinned it open; leaving an unpinned
requirement folds it again. `Space` on a requirement or its scenario
toggles the pin.

Rows nest as a tree hanging from the change. Each row below a change
MUST begin with a guide prefix in the muted style: `├─` when a later
sibling follows, `└─` when it is the last sibling, and for every
ancestor above it `│` when that ancestor has a later sibling, spaces
when not. The change heading row, when present, draws no connector.
Guides are glyphs and MUST draw the same without colour.

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

#### Scenario: Scenarios folded on open

- **GIVEN** a change with an artefact
- **AND** a requirement with three scenarios
- **WHEN** the view opens
- **THEN** the cursor is on the artefact row
- **AND** the list shows no scenario rows

#### Scenario: The cursor opens a requirement

- **GIVEN** the cursor on an artefact row
- **AND** a requirement with three scenarios below it
- **WHEN** the user moves the cursor onto the requirement
- **THEN** its three scenario rows appear under it

#### Scenario: Leaving folds an unpinned requirement

- **GIVEN** the cursor on an open requirement that is not pinned
- **WHEN** the user moves the cursor past its last scenario
- **THEN** its scenario rows disappear
- **AND** the cursor is on the next requirement

#### Scenario: Space pins a requirement open

- **GIVEN** the cursor on a requirement
- **WHEN** the user presses `Space`
- **AND** moves the cursor onto another requirement
- **THEN** the pinned requirement keeps its scenario rows

#### Scenario: A removed scenario has a row

- **GIVEN** a requirement whose delta drops one scenario
- **WHEN** the reviewer unfolds that requirement
- **THEN** the dropped scenario has a row
- **AND** the row's glyph is `-`

#### Scenario: Roots hang from the change

- **GIVEN** one change with two artefacts
- **AND** one capability
- **WHEN** the view renders
- **THEN** both artefact rows begin with `├─`
- **AND** the capability heading begins with `└─`

#### Scenario: The last requirement closes the branch

- **GIVEN** a capability with two requirements
- **WHEN** the view renders
- **THEN** the first requirement row begins with `├─`
- **AND** the second begins with `└─`

#### Scenario: A scenario under a requirement with a later sibling

- **GIVEN** a capability with two requirements
- **AND** the first unfolded with one scenario
- **WHEN** the view renders
- **THEN** the scenario row shows `│` in the requirement column
- **AND** begins its own cell with `└─`

#### Scenario: Fold marker follows the cursor

- **GIVEN** a folded requirement with scenarios
- **WHEN** the user moves the cursor onto it
- **THEN** the row's marker changes from `▸` to `▾`

#### Scenario: Two changes are the roots

- **GIVEN** a snapshot with two changes
- **WHEN** the view renders
- **THEN** each change heading row draws no connector
- **AND** its artefacts and capabilities begin with `├─` or `└─`
- **AND** their requirements begin one column further in

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
finding, `Space` to fold and unfold a requirement's scenarios, `a` to
toggle approval, `e` to write a note, `H` to open history, `m` to cycle
display mode, `?` for a help overlay, `q` and `Esc` to quit. `a` on a
scenario row MUST toggle its parent requirement. `n` and `p` MUST unfold
a requirement they land inside. The help overlay lists every binding.

#### Scenario: Jump to next finding

- **GIVEN** a later row with a finding
- **WHEN** the user presses `n`
- **THEN** the cursor moves to that row
- **AND** the detail pane shows it

#### Scenario: Help

- **WHEN** the user presses `?`
- **THEN** an overlay lists every binding
- **AND** any key closes it

#### Scenario: Jump into a folded requirement

- **GIVEN** a folded requirement whose scenario carries a finding
- **WHEN** the user presses `n`
- **THEN** that requirement unfolds
- **AND** the cursor is on the scenario row

#### Scenario: Approve from a scenario row

- **GIVEN** the cursor on a scenario row of an unapproved requirement
- **WHEN** the user presses `a`
- **THEN** the parent requirement row shows `[√]`

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

### Requirement: A note is written in a popup

Pressing `e` MUST open a popup over the detail pane holding the name and
body of the row's anchor above a single editable buffer seeded with the
note's current text. The buffer MUST wrap at the popup's width and MUST
NOT hold line breaks. `⏎` saves, `Esc` cancels, and `^E` suspends the
view and opens `$EDITOR` on the buffer, returning to the popup with what
the editor saved. Without `$EDITOR` the tool falls back to `vi`. While
the popup is open every key MUST go to it, and no second popup or
overlay may open over it.

#### Scenario: Write and save

- **GIVEN** the cursor on a scenario row
- **WHEN** the user presses `e`
- **THEN** the popup quotes that scenario
- **AND** typing and pressing `⏎` stores the note

#### Scenario: Cancel

- **GIVEN** the popup open with typed text
- **WHEN** the user presses `Esc`
- **THEN** the popup closes
- **AND** the note is unchanged
- **AND** the view does not quit

#### Scenario: Escalate to the editor

- **GIVEN** the popup holding typed text
- **WHEN** the user presses `^E`
- **THEN** `$EDITOR` opens on that text
- **AND** the popup returns holding what the editor saved

#### Scenario: Keys reach the popup

- **GIVEN** the popup open
- **WHEN** the user presses `q`
- **THEN** a `q` is typed into the buffer
- **AND** the view does not quit

### Requirement: Glossary terms are marked where they appear

The detail pane MUST underline every admitted name of a glossary term
where it appears in the text it draws, across the whole name. A
deprecated synonym MUST additionally take the warning style. An
occurrence that falls inside a longer admitted name is part of that
name and MUST NOT be marked as a synonym. Marking MUST compose with the
diff: a marked span keeps its kind glyph and its added, removed or
changed style.

#### Scenario: Term in an unchanged paragraph

- **GIVEN** a glossary defining `group key`
- **AND** a requirement whose body says `group key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `group key` are underlined

#### Scenario: Term inside an added paragraph

- **GIVEN** a glossary defining `group key`
- **AND** an added paragraph containing `group key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `group key` are underlined
- **AND** the line keeps its `+` glyph
- **AND** those cells keep the added style

#### Scenario: Deprecated synonym in the text

- **GIVEN** a glossary in which `admin` is deprecated for `manager`
- **AND** a requirement whose body says `admin`
- **WHEN** the detail pane renders it
- **THEN** the cells of `admin` have the warning style

#### Scenario: Synonym inside a longer term

- **GIVEN** a glossary in which `rotation key` is deprecated for
  `group key`
- **AND** a term named `PLC rotation key`
- **AND** a requirement whose body says `PLC rotation key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `PLC rotation key` are underlined
- **AND** no cell has the warning style

#### Scenario: Marking without colour

- **GIVEN** `NO_COLOR` is set
- **AND** a requirement whose body says `group key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `group key` are underlined

### Requirement: Rendering cost follows the screen, not the spec

A frame of the detail pane MUST style only the lines that can reach the
screen at the current scroll offset. The glossary's matchers MUST be
compiled once per session and the selected row's diff once per
selection. The view MUST NOT redraw while no key or resize event has
arrived. Scrolling MUST still reach every line, including the findings,
notes and history summary under a diff longer than the pane.

#### Scenario: Findings under a long diff

- **GIVEN** the cursor on a requirement whose diff is longer than the
  detail pane
- **AND** the requirement has a finding
- **WHEN** the user scrolls to the end of the pane
- **THEN** the finding's line is visible

#### Scenario: Findings under a long diff in every mode

- **GIVEN** the cursor on a requirement whose diff is longer than the
  detail pane
- **AND** the requirement has a finding
- **WHEN** the user cycles through side-by-side and raw mode
- **AND** scrolls to the end of the pane in each
- **THEN** the finding's line is visible in each

#### Scenario: A large artefact with a glossary

- **GIVEN** an artefact of 5,000 changed lines
- **AND** a glossary of twenty terms
- **WHEN** the view draws one frame with the artefact selected
- **THEN** the frame finishes within one second in a debug build

