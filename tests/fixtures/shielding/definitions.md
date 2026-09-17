# definitions

## Purpose

Four terms in which one word sits inside another. `rotation key` is
deprecated for `group key` and is also the tail of the term
`PLC rotation key`; `anchor` is deprecated for `verification method` and
is also the tail of `lineage`'s admitted `lineage anchor`.

## Requirements

### Requirement: group key

A spec MUST use `group key` to mean:

The symmetric key that wraps a workspace's document content keys for the
current rotation. Every member holds a wrap of it; rotation replaces it.

- **Deprecated:** workspace key, rotation key

#### Scenario: In a sentence

- **WHEN** a member is removed
- **THEN** the group key rotates

### Requirement: PLC rotation key

A spec MUST use `PLC rotation key` to mean:

The key that signs operations on an account's PLC directory entry. It
belongs to the DID method, not to the workspace.

#### Scenario: In a sentence

- **WHEN** an account publishes a directory operation
- **THEN** its PLC rotation key signs that operation

### Requirement: verification method

A spec MUST use `verification method` to mean:

The entry an account publishes in its DID document to vouch for the
signing key it also publishes on its own PDS.

- **Deprecated:** anchor

#### Scenario: In a sentence

- **WHEN** an account has published no verification method
- **THEN** it resolves as unverified

### Requirement: lineage

A spec MUST use `lineage` to mean:

The field carrying the chain's genesis URI on every record after
genesis.

- **Admitted:** lineage anchor

#### Scenario: In a sentence

- **WHEN** a record is read
- **THEN** its lineage is the genesis URI
