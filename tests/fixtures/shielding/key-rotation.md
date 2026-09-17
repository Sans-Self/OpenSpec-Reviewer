# key-rotation

## Purpose

Canon that uses the long phrases the glossary defines, one bare
deprecated synonym, and the words "key rotation" in prose.

## Requirements

### Requirement: Directory operations are signed by their own key

Publication requires a PLC rotation key, and the directory accepts a
signed operation from any holder of one.

#### Scenario: An account holds no PLC rotation key

- **WHEN** an account holding no PLC rotation key publishes
- **THEN** the holder it delegates to signs on its behalf

### Requirement: Rotation replaces the wrap

The rotation key wraps every content key in the workspace, so retiring a
member replaces it.

#### Scenario: A member is removed

- **WHEN** a manager removes a member
- **THEN** the wrap is replaced

### Requirement: Records resolve through their chain

The lineage anchor resolves to the genesis URI for every record in a
supersede chain.

#### Scenario: A record is read

- **WHEN** a component derives an object's identity
- **THEN** the lineage anchor answers

### Requirement: Background work never gates correctness

Protocol correctness never depends on a key rotation completing in the
background.

#### Scenario: The sweep is interrupted

- **WHEN** a re-wrap sweep stops halfway
- **THEN** every reader still resolves its key
