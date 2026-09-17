# review-tui (delta)

## MODIFIED Requirements

### Requirement: The view is a list and a detail pane

The interactive view MUST show a list on the left and a detail pane on
the right. The list has one row per review item: each artefact of the
change, then each requirement pairing grouped under a capability heading
row, then that requirement's scenarios beneath it. Heading rows are not
selectable. A requirement row shows, in order: approval mark (`[ ]`,
`[√]` or `[~]`), kind glyph (`+` added, `~` modified, `-` removed, `>`
renamed), name, then `!` for an error finding or `?` for a warning, and
`✎` when it or any scenario under it carries a note. A scenario row is
indented, shows no approval mark, and takes its glyph from the scenario
match: `+` added, `-` removed, `~` changed, none when unchanged. Scenario
rows are folded when the view opens. When the snapshot holds more than
one change, each change gets its own heading row above its capabilities.

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

- **GIVEN** a requirement with three scenarios
- **WHEN** the view opens
- **THEN** the list shows the requirement row
- **AND** it shows no scenario rows under it

#### Scenario: A removed scenario has a row

- **GIVEN** a requirement whose delta drops one scenario
- **WHEN** the reviewer unfolds that requirement
- **THEN** the dropped scenario has a row
- **AND** the row's glyph is `-`

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

## ADDED Requirements

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
