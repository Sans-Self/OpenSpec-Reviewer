# semantic-diff (delta)

## RENAMED Requirements

- FROM: `### Requirement: Every delta entry pairs with canon by name`
- TO: `### Requirement: Every delta entry matches canon by name`

## MODIFIED Requirements

### Requirement: Every delta entry matches canon by name

The tool MUST match each delta entry to the canon requirement of the same
name in the same capability. A MODIFIED, REMOVED or RENAMED entry matches
the canon requirement it names. An ADDED entry has no canon side.

#### Scenario: Modified requirement found in canon

- **GIVEN** a MODIFIED entry naming a requirement canon has in that
  capability
- **WHEN** the tool matches the entry
- **THEN** the pairing's before side is the canon text
- **AND** its after side is the delta text

#### Scenario: Same name in a different capability

- **GIVEN** a MODIFIED entry naming a requirement that exists only in
  another capability
- **WHEN** the tool matches the entry
- **THEN** the pairing has no before side
- **AND** the review reports it as modified without canon

### Requirement: A rename shows both names and the body diff

For a RENAMED entry, the tool MUST show the old name marked `-`, the new
name marked `+`, and then the body diff between the canon requirement
under the old name and the after side.

#### Scenario: Rename with wording change

- **GIVEN** A renamed to B
- **AND** B's body differing from A's by one word
- **WHEN** the tool renders the pairing
- **THEN** it shows both names
- **AND** a body diff marking that one word
