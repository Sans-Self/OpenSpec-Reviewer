## MODIFIED Requirements

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
when unchanged. Scenario rows are folded when the view opens. When the
snapshot holds more than one change, each change gets its own heading
row above its capabilities.

Rows nest as a tree. Each row after the roots MUST begin with a guide
prefix in the muted style: `├─` when a later sibling follows, `└─` when
it is the last sibling, and for every ancestor above it `│` when that
ancestor has a later sibling, spaces when not. The roots are the change
heading rows when there are several changes, otherwise the artefacts and
capability headings; roots draw no connector. Guides are glyphs and MUST
draw the same without colour.

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

#### Scenario: Fold marker follows Space

- **GIVEN** a folded requirement with scenarios
- **WHEN** the user presses `Space`
- **THEN** the row's marker changes from `▸` to `▾`

#### Scenario: Two changes are the roots

- **GIVEN** a snapshot with two changes
- **WHEN** the view renders
- **THEN** each change heading row draws no connector
- **AND** its artefacts and capabilities begin with `├─` or `└─`
