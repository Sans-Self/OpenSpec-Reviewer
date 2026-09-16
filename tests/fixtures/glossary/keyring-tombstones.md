# keyring-tombstones

## Requirements

### Requirement: A tombstone names the rotation it closes

A tombstone MUST carry the `epochNumber` it retires and the `chainHead`
that retired it, so a reader can tell whether a record still needs the
lazy path.

#### Scenario: Epoch on the tombstone

- **WHEN** a rotation is retired
- **THEN** its tombstone names that `epochNumber`
- **AND** the supersede log gains an entry

### Requirement: Clients act on the outcome, never on URI matching

A client MUST branch on the tombstone lookup result, not on the shape of
the record URI. An administrative script may bypass this.

#### Scenario: Lookup drives the branch

- **WHEN** a client resolves a record
- **THEN** it consults the tombstone table

### Requirement: Admin API mints invites

The indexer's provisioning endpoint MUST mint invite codes only when the
caller is a manager of the workspace.

#### Scenario: Manager mints

- **WHEN** a manager calls the endpoint
- **THEN** an invite code is returned
