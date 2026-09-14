# key-rotation (delta)

## ADDED Requirements

### Requirement: Rotation keeps a grace period

Wrapped keys of a retired epoch MUST stay readable for the configured
grace period before they are purged.

#### Scenario: Offline member catches up

- **GIVEN** a member offline through a rotation
- **WHEN** they reconnect inside the grace period
- **THEN** they can still unwrap the retired epoch

## MODIFIED Requirements

### Requirement: Members re-encrypt on their next write

A member MUST re-encrypt records under the current epoch the next time
they write, never in a background sweep, and MUST finish before the
grace period ends.

#### Scenario: Lazy re-encryption

- **GIVEN** a record sealed under an old epoch
- **WHEN** the member edits it
- **THEN** the record is sealed under the current epoch

## REMOVED Requirements

### Requirement: Old epochs are purged immediately

**Reason**: replaced by the grace period.
