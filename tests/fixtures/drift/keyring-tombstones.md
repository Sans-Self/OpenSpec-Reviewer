# keyring-tombstones

## Requirements

### Requirement: A tombstone names the epoch it closes

A tombstone MUST carry the epoch number it retires. Records still marked
`epoch: 'sealed'` are readable until the grace window closes; the
"rotation ledger" lists them.

#### Scenario: Epoch on the tombstone

- **WHEN** an epoch is retired
- **THEN** its tombstone names that epoch

### Requirement: Clients act on the outcome, never on URI matching

A client MUST branch on the tombstone lookup result, not on the shape of
the record URI. See also Rotation produces a new epoch key.

#### Scenario: Lookup drives the branch

- **WHEN** a client resolves a record
- **THEN** it consults the tombstone table
