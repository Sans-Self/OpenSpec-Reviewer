# plain-output (delta)

## MODIFIED Requirements

### Requirement: Plain text uses no escape codes unless asked

Plain text MUST contain no ANSI escape codes when stdout is not a
terminal, and MUST use the palette's colours when stdout is a terminal,
unless `NO_COLOR` is set or the palette is `none`. `--color` turns
escape codes on regardless of where stdout goes, for terminals that pipe
through a pager.

#### Scenario: Default plain

- **WHEN** the tool prints plain text to a pipe without `--color`
- **THEN** the bytes contain no escape sequences

#### Scenario: Forced colour

- **WHEN** the tool prints plain text to a pipe with `--color`
- **THEN** the bytes contain escape sequences

#### Scenario: NO_COLOR on a terminal

- **GIVEN** `NO_COLOR` is set
- **WHEN** the tool prints plain text to a terminal
- **THEN** the bytes contain no escape sequences
