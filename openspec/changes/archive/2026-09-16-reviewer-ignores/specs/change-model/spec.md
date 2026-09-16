## ADDED Requirements

### Requirement: The register is every requirement canon and open changes assert

The tool MUST build a register once per run, holding every requirement of
canon and every requirement any open change adds, renames or modifies,
each identified as `<capability> § <requirement name>`. The register MUST
be ordered by capability name, then by requirement name. `lint` and
`change` MUST build the same register from the same working directory and
differ only in what they report from it.

#### Scenario: Canon and an open change

- **GIVEN** canon with `alpha § One` and `beta § Two`
- **AND** an open change adding `alpha § Three`
- **WHEN** the tool builds the register
- **THEN** the register holds all three

#### Scenario: Stable order

- **GIVEN** a register holding `beta § Two` and `alpha § One`
- **WHEN** the tool reports it
- **THEN** `alpha § One` comes before `beta § Two`

#### Scenario: Same register in both commands

- **GIVEN** a working directory with canon and two open changes
- **WHEN** the lint builds the register
- **AND** a review of one change builds the register
- **THEN** the two registers hold the same entries
