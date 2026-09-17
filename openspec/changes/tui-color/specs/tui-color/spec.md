# tui-color (delta)

## ADDED Requirements

### Requirement: Colour mode paints the diff by background

In colour mode the view MUST render an added line with a green
background across the detail pane and a removed line with a red
background across the pane, and MUST render a changed line with no line
background and its differing spans tinted: red for removed spans, green
for added spans. Text inside a tint keeps the terminal's default
foreground. The wdiff marks `[-`, `-]`, `{+`, `+}` MUST NOT appear in
colour mode. In non-colour mode rendering is unchanged.

#### Scenario: Added line

- **GIVEN** colour mode
- **AND** a pairing with an added paragraph
- **WHEN** the view renders
- **THEN** every cell of that line has a green background
- **AND** the line starts with the `+` glyph

#### Scenario: Changed words

- **GIVEN** colour mode
- **AND** a paragraph where one word changed
- **WHEN** the view renders
- **THEN** the removed word has a red background
- **AND** the added word has a green background
- **AND** the rest of the line has no background
- **AND** the line contains no `[-` or `{+`

#### Scenario: Non-colour keeps the marks

- **GIVEN** `NO_COLOR` is set
- **AND** a paragraph where one word changed
- **WHEN** the view renders
- **THEN** the line contains `[-` and `{+`

### Requirement: The list pane colours its markers

In colour mode the list MUST paint the `!` marker in the error colour,
the `?` marker in the warning colour, the `✎` marker in the accent
colour, `[√]` in the approved colour, `[~]` in the stale colour, and
each delta glyph in the colour of the change it stands for: `+` added,
`-` removed, `~` changed, `>` accent.

#### Scenario: Error marker

- **GIVEN** colour mode
- **AND** a pairing with an error finding
- **WHEN** the view renders
- **THEN** the `!` cell has the error colour

#### Scenario: Approved mark

- **GIVEN** colour mode
- **AND** an approved pairing
- **WHEN** the view renders
- **THEN** the `√` cell has the approved colour

### Requirement: The focused pane has a coloured border

In colour mode the border of the focused pane MUST use the accent colour
and the border of the other pane MUST be dim. In non-colour mode the
focused border is bold and the other is plain.

#### Scenario: Tab moves the accent

- **GIVEN** colour mode
- **AND** the list pane is focused
- **WHEN** the reviewer presses `Tab`
- **THEN** the detail pane's border has the accent colour
- **AND** the list pane's border is dim

### Requirement: The status bar colours non-zero counts

In colour mode the status bar MUST paint the errors count in the error
colour, the warnings count in the warning colour and the notes count in
the note colour, each only when the count is not zero.

#### Scenario: One error, no warnings

- **GIVEN** colour mode
- **AND** a review with one error and no warnings
- **WHEN** the view renders
- **THEN** `1 errors` has the error colour
- **AND** `0 warnings` has the default colour

### Requirement: Lint output is coloured on a terminal

When stdout is a terminal and colour is not disabled, `lint` MUST paint
the severity word in its colour, the file path dim, the
`capability § requirement` in bold, and each non-zero count in the
summary line in its colour. When stdout is not a terminal, or `NO_COLOR`
is set, or the palette is `none`, the output MUST contain no escape
codes unless `--color` is given.

#### Scenario: Piped lint

- **WHEN** `lint` runs with stdout piped
- **THEN** the bytes contain no escape sequences

#### Scenario: Forced colour

- **WHEN** `lint --color` runs with stdout piped
- **THEN** the severity words carry escape sequences

### Requirement: The palette adapts to the terminal background

The tool MUST determine whether the terminal background is light or
dark, first from `COLORFGBG` when set, where a background index of `7`
or `15` means light, otherwise by an `OSC 11` query answered within
100 ms whose `rgb:` reply has a relative luminance above one half.
Without an answer the background is dark. Each palette has a light and
a dark variant; the tool MUST use the variant for the detected
background. On a light background warnings use magenta rather than
yellow.

#### Scenario: COLORFGBG says light

- **GIVEN** `COLORFGBG=0;15`
- **WHEN** the view renders a warning
- **THEN** the warning colour is magenta

#### Scenario: No answer

- **GIVEN** no `COLORFGBG`
- **AND** a terminal that does not answer `OSC 11`
- **WHEN** the view starts
- **THEN** it renders within 100 ms of the query
- **AND** it uses the dark variant

#### Scenario: Reply is classified

- **GIVEN** an `OSC 11` reply of `rgb:ffff/ffff/ffff`
- **WHEN** the classifier reads it
- **THEN** the background is light

### Requirement: Three palettes

The tool MUST offer palettes `default`, `accessible` and `none`.
`default` uses green for added and red for removed. `accessible` uses
blue for added and orange for removed, ANSI yellow on a dark background
and magenta on a light one. `none` uses no colour and renders as
non-colour mode does. Every role of every palette MUST differ from the
default style in at least a modifier, so no role is colour only.

#### Scenario: Accessible added line

- **GIVEN** palette `accessible`
- **AND** a pairing with an added paragraph
- **WHEN** the view renders
- **THEN** the line has a blue background

#### Scenario: None is non-colour

- **GIVEN** palette `none`
- **WHEN** the view renders a changed word
- **THEN** the line contains `[-` and `{+`
- **AND** no cell has a foreground or background colour
