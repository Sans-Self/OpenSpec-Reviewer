# alpha

## Requirements

### Requirement: A route records its mountType

Every route MUST record a `mountType`, and the workspace `custodian`
that mounted it.

#### Scenario: Mount recorded

- **WHEN** a route mounts a page
- **THEN** the row shows its `mountType`

### Requirement: Rows are ordered by path

Rows MUST be ordered alphabetically by path.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order
