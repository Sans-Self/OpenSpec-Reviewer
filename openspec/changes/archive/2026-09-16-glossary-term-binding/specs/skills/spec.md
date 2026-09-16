## ADDED Requirements

### Requirement: A skill's drafted output validates

For every shipped skill that writes an OpenSpec artefact, the repository
MUST hold a test that produces the artefact shape the skill instructs,
runs `openspec validate` over the change containing it, and asserts no
error. A skill whose instructed shape cannot validate MUST fail that
test.

#### Scenario: Drafted term

- **GIVEN** the shape `/opsx-reviewer:define` instructs for a term
- **WHEN** the test writes it into a change and validates
- **THEN** the validation reports no error

#### Scenario: Instructed shape stops validating

- **GIVEN** a skill whose instructed shape omits SHALL and MUST
- **WHEN** the test runs
- **THEN** it fails naming that skill

## MODIFIED Requirements

### Requirement: The define skill drafts glossary terms

`/opsx-reviewer:define` MUST instruct the agent to run
`openspec-reviewer lint --format json`, take every "recurring term
without definition" note with its uses, read the requirements listed,
separate vocabulary from field names and paths, and for each term it
keeps draft a `### Requirement: <term>` whose body opens with
``A spec MUST use `<term>` to mean:``, followed by a meaning distilled
from the uses, an `- **Admitted:**` line when the uses show a second word
for the same concept the project is content to keep, a
`- **Deprecated:**` line when the uses show a word the project should
stop saying, and one usage scenario lifted from a real requirement. The
skill MUST tell the agent to put a word on one line or the other, never
both. The draft MUST go under `## ADDED Requirements` in
`openspec/changes/<name>/specs/definitions/spec.md` of a new change the
agent names, and the skill MUST end by running
`openspec-reviewer change <name>`.

#### Scenario: Three candidates, one kept

- **GIVEN** lint notes for `ledger`, `createdAt` and `at.example.record`
- **AND** `ledger` is used in three requirements as a concept
- **WHEN** the agent follows the skill
- **THEN** it drafts one term `ledger`
- **AND** it leaves `createdAt` and `at.example.record` undefined
- **AND** the draft is under `openspec/changes/`

#### Scenario: Uses show a second acceptable word

- **GIVEN** a lint note for `ledger`
- **AND** the requirements listed say `ledger` and `log` for one concept
- **AND** the project uses both
- **WHEN** the agent follows the skill
- **THEN** the drafted term has an `- **Admitted:**` line naming `log`

#### Scenario: The draft validates

- **GIVEN** a drafted term
- **WHEN** `openspec validate <name>` runs
- **THEN** it reports no error for that requirement
