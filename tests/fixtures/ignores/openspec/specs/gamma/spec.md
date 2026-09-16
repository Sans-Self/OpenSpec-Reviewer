# gamma

## Requirements

### Requirement: The ledger prints one line per entry

The ledger MUST print one line per entry, naming the `mountType` of the
route the entry came from.

#### Scenario: One line per entry

- **WHEN** the ledger renders
- **THEN** every entry has its own line

### Requirement: Entries are appended to the ledger

An entry MUST be appended, never inserted ahead of an older one.

#### Scenario: Append only

- **WHEN** an entry arrives
- **THEN** it lands after the entries already there
