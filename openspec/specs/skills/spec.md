# skills Specification

## Purpose
TBD - created by archiving change reviewer-skills. Update Purpose after archive.
## Requirements
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
- **AND** four more skills exist beside it
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

For each of `define`, `cite`, `crossref` and `triage`, install MUST
write `.claude/commands/opsx-reviewer/<name>.md` whose body instructs
the agent to load the `opsx-reviewer-<name>` skill and pass the
command's arguments to it. The command file MUST carry the same version
and checksum frontmatter as a skill and follow the same overwrite rule.
No command MUST be written for `workflow`.

#### Scenario: Four commands, not five

- **GIVEN** a repository with a `.claude/` directory
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** `.claude/commands/opsx-reviewer/` holds `define.md`, `cite.md`, `crossref.md` and `triage.md`
- **AND** holds no `workflow.md`

#### Scenario: Command defers to the skill

- **WHEN** `.claude/commands/opsx-reviewer/triage.md` is read
- **THEN** its body names the `opsx-reviewer-triage` skill
- **AND** carries the arguments placeholder
- **AND** carries none of the skill's own instructions

### Requirement: The workflow skill tells an agent when to run the reviewer

`opsx-reviewer-workflow` MUST have a description naming a repository
with an `openspec/` directory and the `openspec-reviewer` binary as the
condition for loading it, and MUST NOT opt out of model invocation. Its
body MUST tell the agent to run `openspec-reviewer change <name>
--format json` after editing a change's deltas and before archiving, to
run `openspec-reviewer lint` after touching cited source or tests, how
to read the summary line, what each finding kind and severity means,
how to write a `spec:<capability> § <requirement>` citation, and which
of `define`, `cite`, `crossref` and `triage` to load for each finding
kind that is not a mechanical fix. It MUST state that the agent never
edits `openspec/specs/`.

#### Scenario: Every finding kind is explained

- **WHEN** the workflow body is read
- **THEN** every finding kind the review can emit appears in it
- **AND** each has one sentence of meaning
- **AND** each judgment kind names the skill that handles it

#### Scenario: Agent finishes a delta

- **GIVEN** an agent has edited `openspec/changes/<name>/specs/alpha/spec.md`
- **AND** the workflow skill is loaded
- **WHEN** the agent follows the skill
- **THEN** it runs the review on `<name>` before proposing to archive
- **AND** it runs no command that writes under `openspec/specs/`

### Requirement: A project can replace a skill's body

A file at `openspec/reviewer/skills/<name>.md` MUST replace the shipped
body of skill `<name>` at install time, keeping the shipped frontmatter
and recording the override path in it. `openspec/reviewer/` is the one
store for agent-facing text; `prompts/` beside `skills/` is reserved for
the assist templates.

#### Scenario: Override present

- **GIVEN** `openspec/reviewer/skills/triage.md` containing "Ask before every edit."
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** the installed triage skill's body is that text
- **AND** its frontmatter still names `opsx-reviewer-triage`
- **AND** its frontmatter records the override path

### Requirement: Skills cite the requirements they depend on

Each shipped skill body MUST end with a list of `spec:` citations into
this tool's canon naming the requirements the skill relies on. When the
skills are installed into this repository the citations MUST be kept,
so `lint` checks them; when installed elsewhere they MUST be rewritten
as plain `capability § requirement` text, so a foreign lint does not
report them as dangling.

#### Scenario: Installed here

- **GIVEN** this repository
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** every installed skill contains at least one `spec:` citation
- **AND** `openspec-reviewer lint` reports no dangling citation in `.claude/skills/`

#### Scenario: Installed elsewhere

- **GIVEN** a repository whose canon has no `glossary` capability
- **WHEN** the user runs `openspec-reviewer skills install`
- **THEN** no installed skill contains a citation the lint would check
- **AND** the "Depends on" names are still there as plain text

### Requirement: Every skill writes a change, never canon

Each shipped task skill MUST state in its first paragraph that it
writes under `openspec/changes/`, or for `cite` into test files, and
never under `openspec/specs/`, and MUST end by running the reviewer on
what it wrote. The workflow skill MUST state that it writes nothing and
name the skill that does for each kind of edit.

#### Scenario: Shape of every task skill

- **WHEN** the shipped `define`, `cite`, `crossref` and `triage` skills are read
- **THEN** each first paragraph names `openspec/changes/` or test files as its output
- **AND** each names `openspec/specs/` as out of bounds
- **AND** each ends with an `openspec-reviewer` command

#### Scenario: Shape of the workflow skill

- **WHEN** the shipped `workflow` skill is read
- **THEN** its first paragraph says it writes nothing
- **AND** it names `openspec/specs/` as out of bounds

### Requirement: The define skill drafts glossary terms

`/opsx-reviewer:define` MUST instruct the agent to run
`openspec-reviewer lint --format json`, take every "recurring term
without definition" note with its uses, read the requirements listed,
separate vocabulary from field names and paths, and for each term it
keeps draft a `### Requirement: <term>` with a meaning distilled from
the uses, an `- **Admitted:**` line when the uses show a second word for
the same concept the project is content to keep, a `- **Deprecated:**`
line when the uses show a word the project should stop saying, and one
usage scenario lifted from a real requirement. The skill MUST tell the
agent to put a word on one line or the other, never both. The draft MUST
go under `## ADDED Requirements` in
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

### Requirement: The cite skill adds citations to uncited requirements

`/opsx-reviewer:cite` MUST instruct the agent to run
`openspec-reviewer lint --coverage --format json`, take every
requirement with zero citing files, search the configured source roots
for the test that exercises it, and add a `spec:<capability> § <name>`
citation to that test's title or a comment, in double quotes or
backticks, without changing the test. A requirement with no test MUST
be listed, not cited. The skill MUST end by running the lint again and
reporting the coverage line before and after.

#### Scenario: Two uncited, one test found

- **GIVEN** a coverage ledger with two uncited requirements
- **AND** a test that exercises one of them
- **WHEN** the agent follows the skill
- **THEN** that test's title carries the citation
- **AND** the other requirement is listed as having no test
- **AND** the report shows the coverage count risen by one

### Requirement: The crossref skill judges siblings

`/opsx-reviewer:crossref <change>` MUST instruct the agent to run
`openspec-reviewer change <change> --format json`, take every pairing
with a `sibling_uses_removed`, `sibling_uses_old_name`, `term_in_use` or
`modified_has_citers` finding, read the pairing's before and after text
and the full text of each sibling named, and give each sibling one
verdict: `consistent` when its claim still holds, `contradicts` when it
asserts something the change removes, `stale term` when only the word
is old. It MUST search `openspec/changes/archive/*/design.md` and
`proposal.md` for the pairing's capability and each removed term and
quote any passage that decided the question, with the archive's name.
For `contradicts` and `stale term` it MUST draft a MODIFIED entry for
the sibling into the change's own delta for that capability, on
confirmation. It MUST end by running the review again and reporting
drift warnings before and after, naming the ones that remain.

#### Scenario: One of each verdict

- **GIVEN** a change in `alpha` that removes the term `ledger`
- **AND** `beta § Entries are appended to the ledger` asserting appends
- **AND** `gamma § The genesis record is frozen` mentioning the ledger only as frozen state
- **AND** `delta § Readers fall back to the ledger` naming it in passing
- **WHEN** the agent follows the skill
- **THEN** it reports `beta` as contradicting
- **AND** `gamma` as consistent
- **AND** `delta` as a stale term
- **AND** it drafts MODIFIED entries for `beta` and `delta` only

#### Scenario: The archive decided this before

- **GIVEN** an archived design saying "unbounded ledger growth is the accepted cost"
- **AND** a change that bounds the ledger
- **WHEN** the agent follows the skill
- **THEN** the report quotes that sentence
- **AND** names the archive it came from
- **AND** says the change reverses a recorded decision

#### Scenario: Warnings after

- **GIVEN** six drift warnings before the skill runs
- **AND** the agent drafts deltas for four siblings
- **WHEN** the review runs again
- **THEN** the report shows two warnings remaining
- **AND** names both

### Requirement: The triage skill walks findings in order

`/opsx-reviewer:triage <change>` MUST instruct the agent to run
`openspec-reviewer change <change> --format json`, present findings
grouped by severity, errors first, and for each finding of a mechanical
kind explain what it means and propose the usual fix: a dangling
citation gets the nearest canon name, a requirement without scenarios
gets one drafted from its body, a removed requirement still cited gets a
choice between the test and the requirement. Every proposed edit MUST be
shown before it is applied, applied only on confirmation, and only to
the change's delta files. Findings of kind `sibling_uses_removed`,
`sibling_uses_old_name`, `term_in_use` and `modified_has_citers` MUST be
listed and handed to `/opsx-reviewer:crossref`, not fixed. The skill
MUST NOT dismiss a finding. It MUST end by running the review again and
showing the summary line before and after.

#### Scenario: Typo in a citation

- **GIVEN** a dangling citation `beta § Entries are apended to the ledger`
- **AND** canon `beta § Entries are appended to the ledger`
- **WHEN** the agent follows the skill
- **THEN** it proposes the canon name
- **AND** applies it only after confirmation
- **AND** the edit is in the change's delta file

#### Scenario: Judgment is routed

- **GIVEN** a pairing with a `sibling_uses_removed` warning
- **WHEN** the agent follows the skill
- **THEN** it lists the warning
- **AND** names `/opsx-reviewer:crossref` as the next step
- **AND** proposes no edit for it

