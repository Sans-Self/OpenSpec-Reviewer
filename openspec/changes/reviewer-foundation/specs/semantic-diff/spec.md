# semantic-diff (delta)

## ADDED Requirements

### Requirement: Every delta entry pairs with canon by name

The tool MUST pair each delta entry with the canon requirement of the same
name in the same capability. A MODIFIED, REMOVED or RENAMED entry pairs
with the canon requirement it names. An ADDED entry has no canon side.

#### Scenario: Modified requirement found in canon

- **GIVEN** a MODIFIED entry naming a requirement canon has in that
  capability
- **WHEN** the tool pairs the entry
- **THEN** the pairing's before side is the canon text
- **AND** its after side is the delta text

#### Scenario: Same name in a different capability

- **GIVEN** a MODIFIED entry naming a requirement that exists only in
  another capability
- **WHEN** the tool pairs the entry
- **THEN** the pairing has no before side
- **AND** the review reports it as modified without canon

### Requirement: Text is normalized before comparison

Before comparing, the tool MUST join the lines of each paragraph into one
line and collapse runs of spaces to one. Blank lines, headings and list
items start a new paragraph. The tool compares and displays the
normalized text, so what the reviewer sees is what was compared.

#### Scenario: Re-wrap only

- **GIVEN** delta text equal to canon except for where lines break inside
  paragraphs
- **WHEN** the tool compares them
- **THEN** it shows the requirement as unchanged

#### Scenario: List items stay separate

- **GIVEN** two list items that were re-wrapped
- **WHEN** the tool normalizes them
- **THEN** they remain two items
- **AND** are compared item by item

### Requirement: A modified requirement shows a word-level diff

For a MODIFIED entry with a canon side, the tool MUST show the body as a
paragraph diff, with changed paragraphs marked at word level: removed
words marked `-`, added words marked `+`, unchanged words plain.
Paragraphs equal on both sides are shown once, unmarked.

#### Scenario: Three words change

- **GIVEN** one paragraph that differs by three words
- **WHEN** the tool renders the pairing
- **THEN** those three words are marked
- **AND** the rest of the paragraph is plain

#### Scenario: Paragraph inserted

- **GIVEN** a delta that adds a paragraph the canon body lacks
- **WHEN** the tool renders the pairing
- **THEN** that whole paragraph is marked `+`

### Requirement: Scenarios are matched by name

Within a paired requirement, the tool MUST match scenarios by name before
diffing. A scenario with the same name on both sides is diffed like a
body. A scenario only in the delta is shown as added. A scenario only in
canon is shown as removed. Order of scenarios is not a difference.

#### Scenario: Scenario renamed

- **GIVEN** canon with scenario "Feature mount appears"
- **AND** a delta with scenario "Module mount appears" of similar body
- **WHEN** the tool renders the pairing
- **THEN** it shows one removed scenario
- **AND** one added scenario
- **AND** no word diff between the two

#### Scenario: Scenarios reordered

- **GIVEN** a delta listing the same scenarios as canon in another order
- **WHEN** the tool renders the pairing
- **THEN** it shows the requirement as unchanged

### Requirement: An added requirement shows its full text

For an ADDED entry, the tool MUST show the whole requirement, body and
scenarios, marked as added.

#### Scenario: New requirement

- **GIVEN** an ADDED entry with a body and two scenarios
- **WHEN** the tool renders the pairing
- **THEN** all of it is marked `+`

### Requirement: A removed requirement shows the canon text

For a REMOVED entry, the tool MUST show the canon requirement in full,
marked as removed. The delta's own text under REMOVED, if any, is not
shown.

#### Scenario: Requirement removed

- **GIVEN** a REMOVED entry naming a canon requirement
- **WHEN** the tool renders the pairing
- **THEN** it shows the canon body and scenarios marked `-`

### Requirement: A rename shows both names and the body diff

For a RENAMED entry, the tool MUST show the old name marked `-`, the new
name marked `+`, and then the body diff between the canon requirement
under the old name and the after side.

#### Scenario: Rename with wording change

- **GIVEN** A renamed to B
- **AND** B's body differing from A's by one word
- **WHEN** the tool renders the pairing
- **THEN** it shows the name pair
- **AND** a body diff marking that one word

### Requirement: Artefacts are shown as line diffs

For `proposal.md`, `design.md` and `tasks.md` the tool MUST show a plain
line diff between the before side and the after side of the snapshot.
When the snapshot has no before side, the artefact is shown in full,
unmarked.

#### Scenario: Diff edits tasks

- **GIVEN** a snapshot that changes two lines of `tasks.md`
- **WHEN** the tool renders the artefact
- **THEN** it shows those two lines as a line diff with context

#### Scenario: Review from the working tree

- **GIVEN** a snapshot from the `change` source
- **WHEN** the tool renders an artefact
- **THEN** it shows the file in full, unmarked
