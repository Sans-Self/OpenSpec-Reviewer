# definitions (delta)

## ADDED Requirements

### Requirement: palette

A spec MUST use `palette` to mean:

The table of styles the tool draws with, one per role such as added,
removed, error or accent. The person chooses a palette in the user
configuration, and each palette has a light and a dark variant for the
terminal background it is read on.

- **Deprecated:** theme, colour scheme, color scheme

#### Scenario: In a sentence

- **WHEN** the user configuration says `accessible`
- **THEN** the view draws with the accessible palette
