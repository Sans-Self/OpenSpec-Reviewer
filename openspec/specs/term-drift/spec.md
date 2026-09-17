# term-drift Specification

## Purpose
TBD - created by archiving change reviewer-citations. Update Purpose after archive.
## Requirements
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

### Requirement: A sibling the change already touches is not a finding

When the change carries a delta for the sibling's capability, and that
delta's after text for the sibling requirement no longer contains the
term, the tool MUST report no finding for that hit. When the delta's
after text still contains the term, the finding is reported and says the
delta keeps it.

#### Scenario: Change updates the sibling too

- **GIVEN** a change that removes a term in capability A
- **AND** the same change modifies the sibling requirement in B so the
  term is gone
- **WHEN** the tool reviews the change
- **THEN** it reports no sibling finding for B

#### Scenario: Change touches the sibling but keeps the term

- **GIVEN** a change that removes a term in capability A
- **AND** the same change modifies the sibling requirement in B and
  keeps the term
- **WHEN** the tool reviews the change
- **THEN** it reports the finding
- **AND** the message says the delta for B still contains the term

### Requirement: An old requirement name in prose is a warning

For every RENAMED pairing, the tool MUST search canon requirements
outside the capability for the old name as plain text, ignoring formal
`spec:` citations, which the citation lint owns. Each hit MUST produce a
warning finding "sibling mentions old name" on the pairing, naming the
sibling.

#### Scenario: Old name in a sibling body

- **GIVEN** a pairing that renames "A tenant imports the document" to
  "A website imports the document"
- **AND** a canon requirement in another capability whose body says
  "see A tenant imports the document"
- **WHEN** the tool reviews the change
- **THEN** the pairing has a warning naming that sibling

#### Scenario: Old name only as a citation

- **GIVEN** the only sibling mention is `` `spec:x § A tenant imports the document` ``
- **WHEN** the tool reviews the change
- **THEN** term drift reports nothing for it

### Requirement: Common phrases do not produce findings

Phrase-tier terms MUST be dropped when they consist only of words from
the tool's stop list or of keywords `MUST`, `MUST NOT`, `GIVEN`, `WHEN`,
`THEN`, `AND`, or when they appear in more than a configurable number of
canon requirements, default `5`. Backticked and quoted tiers are never
dropped. The threshold key is `term_drift.max_common` in
`openspec/reviewer.toml`.

#### Scenario: Ubiquitous phrase

- **GIVEN** the removed phrase "the tool MUST"
- **WHEN** the tool filters removed terms
- **THEN** the phrase is not searched

#### Scenario: Backticked term appears everywhere

- **GIVEN** a removed backticked span present in ten canon requirements
- **WHEN** the tool filters removed terms
- **THEN** the span is still searched
- **AND** ten findings are reported

### Requirement: Drift findings carry their locations

Each term-drift finding MUST carry the term, the sibling capability, the
sibling requirement name and the sibling file path. The detail pane and
plain text list them under the finding; JSON has them as fields. `lint`
does not report term drift; it belongs to the review of one change.

#### Scenario: Detail pane

- **GIVEN** a pairing with two drift findings
- **WHEN** the reviewer selects it
- **THEN** the detail pane lists both terms
- **AND** each sibling's capability, requirement and path

