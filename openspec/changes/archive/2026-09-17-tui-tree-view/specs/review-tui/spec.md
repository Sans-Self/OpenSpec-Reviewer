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
