# sharing-grants

## Requirements

### Requirement: A grant names its chain head

A grant MUST record the `chainHead` of the keyring it was issued under.

#### Scenario: Head recorded

- **WHEN** a manager issues a grant
- **THEN** the grant names the `chainHead`
