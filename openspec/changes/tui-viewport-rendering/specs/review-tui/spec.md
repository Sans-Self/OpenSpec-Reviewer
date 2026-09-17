# review-tui (delta)

## ADDED Requirements

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
