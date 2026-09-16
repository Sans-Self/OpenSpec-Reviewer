# definitions

## Purpose

The words this project uses, one requirement per term. The body opens
with the binding line, the rest is the meaning, the scenarios show the
word in a sentence, and the Deprecated line lists the words not to use
for it. `loket` is the term written without a binding line.

## Requirements

### Requirement: group key

A spec MUST use `group key` to mean:

The symmetric key that wraps a workspace's document content keys for the
current rotation. Every member holds a wrap of it; rotation replaces it.

- **Deprecated:** workspace key, rotation key

#### Scenario: In a sentence

- **WHEN** a member is removed
- **THEN** the group key rotates

### Requirement: manager

A spec MUST use `manager` to mean:

The membership role that may add and remove members and author keyring
supersedes. The other roles are editor and viewer; there is no owner
role.

- **Admitted:** steward
- **Deprecated:** admin, owner

#### Scenario: In a sentence

- **WHEN** a manager removes a member
- **THEN** the keyring head changes

### Requirement: ledger

A spec MUST use `ledger` to mean:

The append-only record of every keyring supersede, read to reconstruct
who held a wrap at any rotation.

- **Admitted:** log

#### Scenario: In a sentence

- **WHEN** a supersede lands
- **THEN** the ledger gains an entry

### Requirement: loket

A counter where residents ask questions. Defined, used nowhere.

#### Scenario: In a sentence

- **WHEN** a resident visits the loket
- **THEN** they are helped
