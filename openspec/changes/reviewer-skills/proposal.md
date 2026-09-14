# reviewer-skills

## Why

The reviewer speaks JSON, and an agent that reads JSON is one command
away, but today every session that wants the reviewer's help has to be
told from scratch which command to run, what the finding kinds mean,
and where the agent may write. Assist points the reviewer at an agent;
nothing yet points an agent at the reviewer. A skill is that pointer: a
`SKILL.md` an agent CLI loads when the agent reaches for it or the user
calls it, telling it which reviewer command to run, how to read the
reply, and that it writes change deltas, never canon.

## What Changes

- Five skills ship in the binary. One is for the agent, four are for
  the user and the agent both:
  - `opsx-reviewer-workflow` tells an agent how the reviewer fits its
    work: run the review after editing a change's deltas and before
    archiving, run the lint after touching cited source or tests, what
    the summary line and each finding kind mean, how to write a `spec:`
    citation, and which of the four skills below to reach for when a
    finding needs more than a mechanical fix. Nobody types it; the
    agent loads it from its description.
  - `define` turns the lint's recurring undefined terms into a drafted
    `definitions` delta, one term per candidate the agent judges to be
    vocabulary rather than a field name.
  - `cite` reads the coverage ledger and adds `spec:` citations to the
    tests that exercise uncited requirements.
  - `crossref` reads a change's drift and blast-radius findings, judges
    each sibling as consistent, contradicting or stale, quotes any
    archived design that decided the question before, and drafts
    sibling deltas into the same change on confirmation.
  - `triage` walks a change's findings in severity order, explains each,
    proposes the usual fix, applies it to the change's deltas on
    confirmation, routes judgment findings to crossref, and re-runs the
    review.
- Skills are the primary artefact, commands are aliases. The skill body
  is the same file for every agent CLI that reads `SKILL.md`; only the
  directory differs. `openspec-reviewer skills install` writes each
  skill to `.claude/skills/opsx-reviewer-<name>/` and, when `.agents/`
  exists, to `.agents/skills/opsx-reviewer-<name>/`, the directory
  Codex, OpenCode and omp read. The four user-callable skills also get a
  command at `.claude/commands/opsx-reviewer/<name>.md` whose body is
  one line invoking the skill, so `/opsx-reviewer:<name>` works in
  Claude Code and in any CLI that reads Claude's command directory. The
  workflow skill has no command. Install overwrites only files it wrote
  before; `skills list` names every file and its state.
- One prompt store, `openspec/reviewer/`, with `prompts/` for assist and
  `skills/` for agents. A project file under `skills/<name>.md` replaces
  the shipped skill body at install time.
- Every shipped skill cites the reviewer requirements it depends on with
  `spec:` citations, so `lint` checks the skills when `.claude` is a
  source root.

## Capabilities

### New Capabilities

- `skills`: the install and list subcommands, the command aliases, the
  store seam, the citation rule, and the five skills' behaviour.

### Modified Capabilities

None in canon. `reviewer-assist`, still open, places its templates under
`openspec/reviewer/prompts/`; this change fixes the parent directory as
the shared store, and assist's design should say so when it is applied.

## Impact

- `src/skills/`: the embedded `SKILL.md` files, the command alias
  template, the install and list logic, the override merge.
- `src/main.rs`: `skills` subcommand with `install` and `list`.
- `lint init` adds `.claude` and `.agents` to `source_roots` when the
  directories exist.
- No new dependency.

## Non-goals

- Testing the skills against agents other than Claude Code. The
  `SKILL.md` format is shared by Claude Code, Codex, OpenCode and omp,
  so the install writes where they all look, but only Claude Code's
  reading of the bodies is exercised here.
- Command aliases for other CLIs. Every CLI keeps its own command
  directory and syntax, Codex has retired commands in favour of skills,
  and OpenCode has no namespaces. Users there call the skill by name.
- Running skills from the reviewer. That is assist's handoff.
- Skills that write canon. Every skill writes a change, and the review
  path is the approval.
