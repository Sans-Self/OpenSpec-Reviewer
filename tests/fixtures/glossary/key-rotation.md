# key-rotation

## Requirements

### Requirement: Rotation produces a new group key

Rotating a workspace MUST mint a fresh group key and wrap it for every
current member. The rotation record carries `epochNumber` and the
`chainHead` it was authored against.

#### Scenario: Fresh key

- **WHEN** a manager rotates the workspace
- **THEN** the new group key wraps for every member

### Requirement: Members re-encrypt on their next write

A member MUST re-encrypt records under the current workspace key the
next time they write, never in a background sweep. The record carries
`epochNumber`.

#### Scenario: Lazy re-encryption

- **GIVEN** a record sealed under an old group key
- **WHEN** the member edits it
- **THEN** the record is sealed under the current one
