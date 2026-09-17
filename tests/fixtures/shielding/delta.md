## MODIFIED Requirements

### Requirement: group key

A spec MUST use `group key` to mean:

The symmetric key that wraps a workspace's document content keys for the
current rotation. Every member holds a wrap of it; rotation replaces it.

- **Deprecated:** workspace key
> Note: a rotation key is a `PLC rotation key`, never the group key.

#### Scenario: In a sentence

- **WHEN** a member is removed
- **THEN** the group key rotates
