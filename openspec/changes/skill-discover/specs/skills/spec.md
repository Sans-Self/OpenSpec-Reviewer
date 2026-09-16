# skills (delta)

## MODIFIED Requirements

### Requirement: Skills install into the repository

`openspec-reviewer skills install` MUST write each shipped skill to
`.claude/skills/opsx-reviewer-<name>/SKILL.md` under the working
directory, with frontmatter naming the skill `opsx-reviewer-<name>`, the
tool version that wrote it, and a checksum of the body. When `.agents/`
exists the same file MUST also be written under
`.agents/skills/opsx-reviewer-<name>/`; the command MUST NOT create
`.agents/`. A file whose body still matches its recorded checksum MUST
be overwritten; a file that does not MUST be left unchanged and named
in the output. Without a `.claude/` directory the command MUST refuse
with an error naming it and exit `2`. `skills list` MUST print each
skill's name, one line of description, and whether it is installed, up
to date or edited.

#### Scenario: Fresh install

- **GIVEN** a repository with a `.claude/` directory
- **AND** no `.agents/` directory
- **AND** no reviewer skills installed
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** `.claude/skills/opsx-reviewer-workflow/SKILL.md` exists
- **AND** five more skills exist beside it
- **AND** no `.agents/` directory exists
- **AND** stdout names the files written

#### Scenario: Other agents' directory present

- **GIVEN** a repository with `.claude/` and `.agents/`
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** `.agents/skills/opsx-reviewer-workflow/SKILL.md` exists
- **AND** its body equals the one under `.claude/skills/`

#### Scenario: Hand-edited skill is kept

- **GIVEN** an installed skill whose body was edited
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** that file is unchanged
- **AND** stdout says it was kept because it differs from what the tool wrote

#### Scenario: No .claude directory

- **GIVEN** a directory without `.claude/`
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** stderr names `.claude/`
- **AND** the exit status is `2`

### Requirement: User-callable skills get a command alias

For each of `define`, `discover`, `cite`, `crossref` and `triage`,
install MUST write `.claude/commands/opsx-reviewer/<name>.md` whose body
instructs the agent to load the `opsx-reviewer-<name>` skill and pass
the command's arguments to it. The command file MUST carry the same
version and checksum frontmatter as a skill and follow the same
overwrite rule. No command MUST be written for `workflow`.

#### Scenario: Five commands, not six

- **GIVEN** a repository with a `.claude/` directory
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** `.claude/commands/opsx-reviewer/` holds `define.md`, `discover.md`, `cite.md`, `crossref.md` and `triage.md`
- **AND** holds no `workflow.md`

#### Scenario: Command defers to the skill

- **WHEN** `.claude/commands/opsx-reviewer/triage.md` is read
- **THEN** its body names the `opsx-reviewer-triage` skill
- **AND** carries the arguments placeholder
- **AND** carries none of the skill's own instructions

### Requirement: Every skill writes a change, never canon

Each shipped task skill MUST state in its first paragraph that it
writes under `openspec/changes/`, or for `cite` into test files, and
never under `openspec/specs/`, and MUST end by running the reviewer on
what it wrote. The workflow skill MUST state that it writes nothing and
name the skill that does for each kind of edit.

#### Scenario: Shape of every task skill

- **WHEN** the shipped `define`, `discover`, `cite`, `crossref` and `triage` skills are read
- **THEN** each first paragraph names `openspec/changes/` or test files as its output
- **AND** each names `openspec/specs/` as out of bounds
- **AND** each ends with an `openspec-reviewer` command

#### Scenario: Shape of the workflow skill

- **WHEN** the shipped `workflow` skill is read
- **THEN** its first paragraph says it writes nothing
- **AND** it names `openspec/specs/` as out of bounds

## ADDED Requirements

### Requirement: The discover skill finds the vocabulary the spans miss

`/opsx-reviewer:discover` MUST instruct the agent to read every
requirement name and body under `openspec/specs/`, collect the noun
phrases of one to three words that appear in two or more capabilities,
drop every phrase the glossary already names as a term or a synonym,
merge the rest with the lint's "recurring term without definition"
notes, and hand the merged list to `define` as candidates with the
`capability § requirement` uses of each. The skill MUST tell the agent
to drop a phrase found in one capability only, to ask for a batch size
before handing off, and MUST end by running `openspec-reviewer change
<name>` on the change `define` wrote.

#### Scenario: Prose term in three capabilities

- **GIVEN** canon where three capabilities say "chain head" in plain prose
- **AND** no span check note names it
- **WHEN** the agent follows the skill
- **THEN** `chain head` is a candidate handed to `define`
- **AND** its uses list the three requirements

#### Scenario: Phrase in one capability only

- **GIVEN** canon where one capability says "retry budget" in four requirements
- **WHEN** the agent follows the skill
- **THEN** `retry budget` is not a candidate

#### Scenario: Phrase the glossary knows

- **GIVEN** a glossary term `ledger` with an admitted synonym `log`
- **AND** canon that says "log" in two capabilities
- **WHEN** the agent follows the skill
- **THEN** `log` is not a candidate
