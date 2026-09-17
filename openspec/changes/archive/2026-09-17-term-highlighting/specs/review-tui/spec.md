# review-tui

## ADDED Requirements

### Requirement: Glossary terms are marked where they appear

The detail pane MUST underline every admitted name of a glossary term
where it appears in the text it draws, across the whole name. A
deprecated synonym MUST additionally take the warning style. An
occurrence that falls inside a longer admitted name is part of that
name and MUST NOT be marked as a synonym. Marking MUST compose with the
diff: a marked span keeps its kind glyph and its added, removed or
changed style.

#### Scenario: Term in an unchanged paragraph

- **GIVEN** a glossary defining `group key`
- **AND** a requirement whose body says `group key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `group key` are underlined

#### Scenario: Term inside an added paragraph

- **GIVEN** a glossary defining `group key`
- **AND** an added paragraph containing `group key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `group key` are underlined
- **AND** the line keeps its `+` glyph
- **AND** those cells keep the added style

#### Scenario: Deprecated synonym in the text

- **GIVEN** a glossary in which `admin` is deprecated for `manager`
- **AND** a requirement whose body says `admin`
- **WHEN** the detail pane renders it
- **THEN** the cells of `admin` have the warning style

#### Scenario: Synonym inside a longer term

- **GIVEN** a glossary in which `rotation key` is deprecated for
  `group key`
- **AND** a term named `PLC rotation key`
- **AND** a requirement whose body says `PLC rotation key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `PLC rotation key` are underlined
- **AND** no cell has the warning style

#### Scenario: Marking without colour

- **GIVEN** `NO_COLOR` is set
- **AND** a requirement whose body says `group key`
- **WHEN** the detail pane renders it
- **THEN** the cells of `group key` are underlined
