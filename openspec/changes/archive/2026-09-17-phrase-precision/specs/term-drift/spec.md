# term-drift (delta)

## MODIFIED Requirements

### Requirement: A pairing yields the terms it removes

For every MODIFIED, REMOVED or RENAMED pairing, the tool MUST collect the
terms present on the before side and absent from the after side, in
three tiers: backticked spans, double-quoted strings, and phrases of two
or more words. The phrase tier MUST read the body without its
`- **Admitted:**` and `- **Deprecated:**` lines and without its binding
line, so a word retired from a marker line is not a phrase lost from a
sentence. The backticked and quoted tiers MUST read the whole body.
Comparison is case-insensitive on normalized text. A term that survives
anywhere on the after side, in the body or any scenario, is not removed.

#### Scenario: Identifier dropped

- **GIVEN** a before side containing `` `mountType: 'feature'` ``
- **AND** an after side that does not contain it
- **WHEN** the tool collects removed terms
- **THEN** the list holds `mountType: 'feature'` as a backticked term

#### Scenario: Phrase moved to a scenario

- **GIVEN** a before side with "status badge" in the body
- **AND** an after side with "status badge" only in a scenario
- **WHEN** the tool collects removed terms
- **THEN** "status badge" is not in the list

#### Scenario: Removed requirement

- **GIVEN** a REMOVED pairing
- **WHEN** the tool collects removed terms
- **THEN** every backticked span and quoted string of the canon text is
  in the list

#### Scenario: Word dropped from a Deprecated line

- **GIVEN** a before side with `- **Deprecated:** workspace key, rotation key`
- **AND** an after side identical but for `- **Deprecated:** workspace key`
- **WHEN** the tool collects removed terms
- **THEN** the list holds no phrase

### Requirement: A removed term found in a sibling is a warning

For every removed term, the tool MUST search the text of every canon
requirement outside the pairing's capability. A phrase-tier term MUST be
searched in the sibling's text without its `- **Admitted:**` and
`- **Deprecated:**` lines, so a term that lists a word as deprecated does
not count as a sibling using it. Each requirement that contains the term
MUST produce a warning finding "sibling mentions removed term" on the
pairing, naming the term, the sibling capability and the sibling
requirement. One finding per term per sibling requirement.

#### Scenario: Sibling still says feature

- **GIVEN** a pairing in `sitemap-index` that removes
  `` `mountType: 'feature'` ``
- **AND** canon `sitemap-tree § Node shows its mount` containing that
  span
- **WHEN** the tool reviews the change
- **THEN** the pairing has a warning naming `mountType: 'feature'`
- **AND** naming `sitemap-tree § Node shows its mount`

#### Scenario: Same capability is not a sibling

- **GIVEN** a removed term that appears in another requirement of the
  same capability
- **WHEN** the tool reviews the change
- **THEN** it reports no sibling finding for that hit

#### Scenario: Sibling names the phrase on a Deprecated line

- **GIVEN** a pairing that removes the phrase "workspace key"
- **AND** canon `definitions § group key` whose only "workspace key" is
  on its `- **Deprecated:**` line
- **WHEN** the tool reviews the change
- **THEN** it reports no sibling finding for that term
