# key-rotation

## Purpose

Epoch keys wrap the workspace secret; rotation retires one epoch and
starts the next.

## Requirements

### Requirement: Rotation produces a new epoch key

Rotating a workspace MUST mint a fresh epoch key and wrap it for every
current member. The rotate path is `packages/crypto/src/rotate.ts`;
`bug__epoch_skipped_on_rotate` covers the epoch counter.

#### Scenario: Fresh epoch

- **WHEN** an admin rotates the workspace
- **THEN** the new epoch number is the old one plus one

### Requirement: Members re-encrypt on their next write

A member MUST re-encrypt records under the current epoch the next time
they write, never in a background sweep.

#### Scenario: Lazy re-encryption

- **GIVEN** a record sealed under an old epoch
- **WHEN** the member edits it
- **THEN** the record is sealed under the current epoch

### Requirement: Old epochs are purged immediately

Wrapped keys of a retired epoch MUST be deleted as soon as the rotation
commits.

#### Scenario: No grace

- **WHEN** rotation commits
- **THEN** the retired epoch's wrapped keys are gone

### Requirement: A rotation records the operator's device

The rotation record MUST carry the device id that performed it.

#### Scenario: Device recorded

- **WHEN** an admin rotates from a device
- **THEN** the rotation record names that device
