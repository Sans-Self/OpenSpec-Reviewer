# review-tui (delta)

## MODIFIED Requirements

### Requirement: Colour is never the only signal

Every marked change MUST carry a glyph as well as a colour: `-` for
removed, `+` for added, `~` for changed. Every span painted with a
background tint MUST also carry a modifier: strikethrough for a removed
span, bold for an added span, so the span reads without its colour.
When `NO_COLOR` is set, the palette is `none`, or the terminal reports
no colour, the tool MUST render with glyphs and modifiers only.

#### Scenario: NO_COLOR

- **GIVEN** `NO_COLOR` is set
- **WHEN** the view renders
- **THEN** it uses no colour
- **AND** every change is still marked by its glyph

#### Scenario: Tinted span keeps its modifier

- **GIVEN** colour mode
- **AND** a paragraph where one word was removed
- **WHEN** the view renders
- **THEN** the removed word's cells have a red background
- **AND** those cells are struck through
