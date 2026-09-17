# review-tui

## MODIFIED Requirements

### Requirement: The detail pane has three display modes

The detail pane MUST offer inline (word diff in one column),
side-by-side (before left, after right), and raw (the snapshot's before
and after text as-is). `m` cycles them and the status line names the
current mode. No mode may drop text that does not fit its column: in
every mode a line wider than its column wraps. Side-by-side wraps each
side within its own column and pads the shorter side so the gutter stays
straight. Raw keeps the snapshot's own line breaks and wraps only the
lines that exceed the column.

#### Scenario: Cycle modes

- **GIVEN** the detail pane in inline mode
- **WHEN** the user presses `m` three times
- **THEN** the pane shows side-by-side, then raw, then inline again

#### Scenario: Side-by-side wider than the column

- **GIVEN** the detail pane in side-by-side mode
- **AND** a paragraph wider than half the pane
- **WHEN** the pane renders
- **THEN** the whole paragraph is on screen across several rows
- **AND** the gutter is in the same column on every row

#### Scenario: Uneven sides

- **GIVEN** the detail pane in side-by-side mode
- **AND** a before side that wraps to two rows
- **AND** an after side that wraps to four rows
- **WHEN** the pane renders
- **THEN** the pair occupies four rows
- **AND** the last two rows of the before side are blank

### Requirement: The detail pane follows the list selection

Selecting a requirement row MUST show its semantic diff, its findings,
its note, and its one-line history summary in the detail pane. Selecting
an artefact row shows its line diff. The detail pane scrolls with
`PgUp`/`PgDn` and `Ctrl-u`/`Ctrl-d`. The scroll offset MUST NOT exceed
the rendered line count less the pane height, and MUST be zero when the
content is shorter than the pane.

#### Scenario: Select a modified requirement

- **GIVEN** the cursor on a modified requirement
- **WHEN** the view renders
- **THEN** the detail pane shows its word-level diff
- **AND** its findings below the diff
- **AND** its note, when it has one

#### Scenario: Scroll past the end

- **GIVEN** a detail pane whose content is one row taller than the pane
- **WHEN** the user presses `Ctrl-d` twice
- **THEN** the last row is at the bottom of the pane
- **AND** the first row has scrolled off by one

#### Scenario: Content shorter than the pane

- **GIVEN** a detail pane whose content is shorter than the pane
- **WHEN** the user presses `PgDn`
- **THEN** the first row is still at the top of the pane

## ADDED Requirements

### Requirement: A line too wide for its column wraps

The view MUST measure text in display columns rather than characters. A
line wider than its column MUST break at the last whitespace that fits,
and a single word wider than the column MUST break at the column. A
continuation row MUST carry blank cells the width of the kind glyph, and
MUST NOT repeat the glyph.

#### Scenario: Break on whitespace

- **GIVEN** an added paragraph wider than the detail pane
- **WHEN** the pane renders it
- **THEN** the first row ends at a word boundary
- **AND** the first row starts with `+`
- **AND** the second row starts with two blank cells

#### Scenario: A word wider than the column

- **GIVEN** a paragraph holding one word wider than the detail pane
- **WHEN** the pane renders it
- **THEN** the word breaks at the pane's last column
- **AND** no row is wider than the pane

#### Scenario: A wide character

- **GIVEN** the detail pane in side-by-side mode
- **AND** a before side holding characters that draw two cells each
- **WHEN** the pane renders
- **THEN** the gutter is in the same column as on a row without them

### Requirement: A long list row ends in an ellipsis

A list row whose text is wider than the list pane MUST be cut at the
pane's last column and MUST end in `…`.

#### Scenario: A long requirement name

- **GIVEN** a requirement whose name is wider than the list pane
- **WHEN** the list renders its row
- **THEN** the row ends in `…`
- **AND** the row is no wider than the pane
