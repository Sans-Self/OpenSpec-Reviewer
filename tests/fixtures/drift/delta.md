# key-rotation (delta)

## MODIFIED Requirements

### Requirement: Rotation produces a new epoch key

Rotating a workspace MUST mint a fresh epoch key. Records sealed under
the old epoch carry `epoch: 'retired'` until the purge runs, and the
audit log records every step.

#### Scenario: Fresh epoch

- **WHEN** an admin rotates the workspace
- **THEN** the new epoch number is the old one plus one
