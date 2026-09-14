# keyring-tombstones

## Requirements

### Requirement: A tombstone names the epoch it closes

A tombstone MUST carry the epoch number it retires, so a reader can tell
whether a record still needs the lazy path in
`spec:key-rotation § Members re-encrypt on their next write`.

#### Scenario: Epoch on the tombstone

- **WHEN** an epoch is retired
- **THEN** its tombstone names that epoch

### Requirement: Clients act on the outcome, never on URI matching

A client MUST branch on the tombstone lookup result, not on the shape of
the record URI.

#### Scenario: Lookup drives the branch

- **WHEN** a client resolves a record
- **THEN** it consults the tombstone table before parsing the URI
