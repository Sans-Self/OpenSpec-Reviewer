# key-rotation (delta)

## MODIFIED Requirements

### Requirement: Rotation produces a new group key

Rotating a workspace MUST mint a fresh group key and wrap it for every
current member; only a manager may rotate. The rotation record carries
`epochNumber`, the `chainHead` it was authored against and its
`chainParent`.

#### Scenario: Fresh key

- **WHEN** an admin rotates the workspace
- **THEN** the new group key wraps for every member
- **AND** the record names its `chainParent`

## ADDED Requirements

### Requirement: Rotation keeps a grace period

Wrapped keys of a retired rotation MUST stay readable for the grace
period. The `chainParent` of the rotation record points at the head it
retired; see `spec:keyring-tombstones § Admin API mints invites` for provisioning.

#### Scenario: Offline member catches up

- **GIVEN** a member offline through a rotation
- **WHEN** they reconnect inside the grace period
- **THEN** they can still unwrap the retired group key
