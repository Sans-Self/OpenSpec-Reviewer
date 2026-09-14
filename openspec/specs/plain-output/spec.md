# plain-output Specification

## Purpose
TBD - created by archiving change reviewer-foundation. Update Purpose after archive.
## Requirements
### Requirement: Plain text is the default outside a terminal

When stdout is not a terminal, or `--plain` is given, the tool MUST print
the review as plain text and exit. It MUST NOT open the interactive view
or wait for input.

#### Scenario: Piped to a file

- **WHEN** the user runs `openspec-reviewer gh 224 > review.txt`
- **THEN** the file holds the plain-text review
- **AND** the tool exits

### Requirement: Plain text mirrors the inline view

Plain text MUST print, per change: the artefact diffs, then per
capability each requirement with its approval mark, kind, name and inline
word diff using the same glyphs as the interactive view (`+`, `-`, `~`),
then its findings, note and history summary indented under it. A findings
summary with counts per severity closes the output.

#### Scenario: One modified requirement with a warning

- **GIVEN** a review with one modified requirement
- **AND** a dropped-scenario warning on it
- **WHEN** the tool prints plain text
- **THEN** the output shows the requirement heading
- **AND** its word diff
- **AND** an indented warning line
- **AND** a summary line counting one warning

### Requirement: Plain text uses no escape codes unless asked

Plain text MUST contain no ANSI escape codes by default. `--color` turns
them on for terminals that pipe through a pager.

#### Scenario: Default plain

- **WHEN** the tool prints plain text without `--color`
- **THEN** the bytes contain no escape sequences

### Requirement: JSON output is the review model

`--format json` MUST print one JSON document holding every change, its
capabilities, each pairing with kind, name, before text, after text,
scenario matches, findings, approval state, note and history, plus the
findings summary. Field names are stable within a major version.

#### Scenario: Agent reads the review

- **WHEN** an agent runs `openspec-reviewer change sweep-gate --format json`
- **THEN** it can read each pairing's kind, before and after text
- **AND** the findings
- **AND** the approval state, without parsing prose

### Requirement: Plain output can be limited to findings

`--findings-only` MUST print only the findings and the summary, one
finding per line as `severity  change/capability/requirement: message`.

#### Scenario: CI step

- **WHEN** CI runs `openspec-reviewer diff --findings-only < pr.diff`
- **THEN** the output is one line per finding plus the summary
- **AND** the exit status follows the worst finding

