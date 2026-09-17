# definitions (delta)

## MODIFIED Requirements

### Requirement: pairing

A spec MUST use `pairing` to mean:

One delta entry matched with its canon counterpart: the requirement
before, the requirement after, their diff, and the findings about them.
A review is a list of pairings grouped by capability. `match` is the verb
for the act that produces one.

- **Deprecated:** pair

#### Scenario: In a sentence

- **WHEN** a RENAMED entry names a requirement canon has
- **THEN** the pairing shows the old name struck through and the new one added
