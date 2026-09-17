# change-model (delta)

## RENAMED Requirements

- FROM: `### Requirement: A rename is a pair of names`
- TO: `### Requirement: A rename reads a FROM line and a TO line`

## MODIFIED Requirements

### Requirement: A rename reads a FROM line and a TO line

Under `## RENAMED Requirements` the tool MUST read `- FROM:` and `- TO:`
lines in twos. Each line carries a backticked `### Requirement: <name>`
heading. The two together yield a RENAMED entry with the old and new
name. A `FROM` without a following `TO`, or the reverse, is a parse
error.

#### Scenario: One rename

- **GIVEN** a `- FROM:` line naming A
- **AND** a `- TO:` line naming B
- **WHEN** the tool parses the section
- **THEN** it yields one RENAMED entry from A to B

#### Scenario: Orphan FROM

- **GIVEN** a `- FROM:` line with no `- TO:` after it
- **WHEN** the tool parses the section
- **THEN** it fails
- **AND** the error names the file and the line
