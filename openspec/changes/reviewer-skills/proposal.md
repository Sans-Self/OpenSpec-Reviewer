# reviewer-skills

## Why

The reviewer speaks JSON, and an agent that reads JSON is one command
away, but today every session that wants the reviewer's help has to be
told from scratch which command to run, what the finding kinds mean,
and where the agent may write. Assist points the reviewer at an agent;
nothing yet points an agent at the reviewer. A skill is that pointer: a
`SKILL.md` an agent CLI loads on `/opsx-reviewer:<name>`, telling it
which reviewer command to run, how to read the reply, and that it writes
change deltas, never canon.

## What Changes

- Four skills ship in the binary, Claude Code flavour only for now:
  - `/opsx-reviewer:define` turns the lint's recurring undefined terms
    into a drafted `definitions` delta, one term per candidate the agent
    judges to be vocabulary rather than a field name.
  - `/opsx-reviewer:cite` reads the coverage ledger and adds `spec:`
    citations to the tests that exercise uncited requirements.
  - `/opsx-reviewer:crossref` reads a change's drift and blast-radius
    findings, judges each sibling as consistent, contradicting or stale,
    quotes any archived design that decided the question before, and
    drafts sibling deltas into the same change on confirmation.
  - `/opsx-reviewer:triage` walks a change's findings in severity
    order, explains each, proposes the usual fix, applies it to the
    change's deltas on confirmation, routes judgment findings to
    crossref, and re-runs the review.
- `openspec-reviewer skills install` writes the skills into the
  repository's `.claude/skills/`, overwriting only files it wrote
  before; `skills list` names them and their versions.
- One prompt store, `openspec/reviewer/`, with `prompts/` for assist and
  `skills/` for agents. A project file under `skills/<name>.md` replaces
  the shipped skill body at install time.
- Every shipped skill cites the reviewer requirements it depends on with
  `spec:` citations, so `lint` checks the skills when `.claude` is a
  source root.

## Capabilities

### New Capabilities

- `skills`: the install and list subcommands, the store seam, the
  citation rule, and the four skills' behaviour.

### Modified Capabilities

None in canon. `reviewer-assist`, still open, places its templates under
`openspec/reviewer/prompts/`; this change fixes the parent directory as
the shared store, and assist's design should say so when it is applied.

## Impact

- `src/skills/`: the embedded `SKILL.md` files, the install and list
  logic, the override merge.
- `src/main.rs`: `skills` subcommand with `install` and `list`.
- `lint init` adds `.claude` to `source_roots` when the directory exists.
- No new dependency.

## Non-goals

- Flavours for other agent CLIs. The skill bodies are agent-neutral
  prose; the frontmatter and directory layout are Claude Code's, and a
  second flavour is a change that adds a layout, not a rewrite.
- Running skills from the reviewer. That is assist's handoff.
- Skills that write canon. Every skill writes a change, and the review
  path is the approval.
