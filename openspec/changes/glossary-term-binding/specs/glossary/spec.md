## ADDED Requirements

### Requirement: A term opens with a binding line

A term's body MUST open with `A spec MUST use <term> to mean:`, where
`<term>` is the requirement name, backticked or bare. The tool MUST strip
that line from the meaning text, as it strips `- **Admitted:**` and
`- **Deprecated:**`. The full body, binding line included, stays the text
a pairing diffs. A term whose body has no binding line, or whose binding
line names something other than the requirement name, MUST produce a
warning finding "term without a binding line" naming the term, in `lint`
and on the term's pairing when a change touches it.

#### Scenario: Meaning excludes the line

- **GIVEN** a term `register` whose body opens with
  ``A spec MUST use `register` to mean:``
- **WHEN** the tool parses the term
- **THEN** the meaning text does not hold that line

#### Scenario: Panel shows the meaning

- **GIVEN** a pairing whose after text contains `register`
- **WHEN** the reviewer presses `D`
- **THEN** the panel shows the meaning without the binding line

#### Scenario: No binding line

- **GIVEN** a term whose body opens with its meaning
- **WHEN** the lint runs
- **THEN** it reports a warning naming that term

#### Scenario: Binding line names another term

- **GIVEN** a term `register` whose body opens with
  ``A spec MUST use `snapshot` to mean:``
- **WHEN** the lint runs
- **THEN** it reports a warning naming `register`

#### Scenario: Editing the line is a change to the term

- **GIVEN** a change whose delta edits only a term's binding line
- **WHEN** the tool reviews the change
- **THEN** the pairing shows a diff
