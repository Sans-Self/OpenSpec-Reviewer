# beta

## Requirements

### Requirement: A mount names its feature

A feature mount MUST name the feature it serves and the `mountType` it
was mounted under.

#### Scenario: Feature named

- **WHEN** a route has a feature mount
- **THEN** the row shows the feature name

### Requirement: A custodian approves a mount

Mounting a feature MUST be approved by the workspace `custodian`.

#### Scenario: Approval recorded

- **WHEN** a feature is mounted
- **THEN** the record names who approved it
