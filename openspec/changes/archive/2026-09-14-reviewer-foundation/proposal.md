# reviewer-foundation

## Why

An OpenSpec change restates every requirement it touches. A delta file
under `## MODIFIED Requirements` carries the whole new text of the
requirement, scenarios included. Git sees a new file, so a pull request
shows forty added lines where the real change is three words and one new
scenario. The reviewer has to open the canonical spec in another tab and
compare by eye. That is slow, and it misses things: a scenario that
quietly disappears, a requirement added under a name that already exists,
a second open change that modifies the same requirement.

The OpenSpec CLI validates structure but does not show the difference. Git
shows the difference but does not understand the structure. This tool sits
between the two.

## What changes

A new Rust binary, `openspec-reviewer`, that reviews an OpenSpec change
against the canonical specs of the repository it runs in.

- Sources are pluggable and each is a subcommand: `change` reads the
  working tree, `diff` applies a unified diff, `git` reads two refs, `gh`
  reads a pull request. Every source yields the same snapshot, so a
  forge such as tangled is one more module.
- The tool pairs every requirement in the delta with its counterpart in
  canon and shows the semantic difference: word-level for MODIFIED, the
  full text for ADDED, the vanishing text for REMOVED, and the name pair
  for RENAMED. Scenarios are matched by name so an added or dropped
  scenario is a finding, not a wall of green.
- It reports findings the CLI does not: a MODIFIED requirement with no
  canon target, an ADDED requirement whose name already exists, a lost
  scenario, and a collision with another open change.
- The reviewer approves items one by one and attaches notes. State
  persists per repository and change, and an approval goes stale when
  the text it approved changes. Notes export as Markdown for a pull
  request review.
- Each requirement has a history: every archived change that touched it,
  renames followed backwards, each version readable and diffable.
- It is a terminal UI by default and plain text or JSON when the output
  is not a terminal or when asked. The plain output exists for CI and for
  agents, which cannot drive a TUI.

## Capabilities

| Capability | Covers |
| --- | --- |
| `change-model` | Parsing canon and delta specs into requirements, scenarios and delta kinds |
| `input-sources` | The source abstraction, the four subcommands, and how a snapshot is built |
| `semantic-diff` | What difference is shown for each delta kind, and what counts as no change |
| `review-findings` | The checks the tool runs and how it reports them |
| `review-state` | Approvals, notes, persistence, stale approvals, Markdown export |
| `requirement-history` | Collecting a requirement's past from archived changes |
| `review-tui` | The list-and-detail view, keys and display rules |
| `plain-output` | Text and JSON output for pipes, CI and agents |

## Non-goals

- Editing specs. The tool reads; the editor writes.
- Replacing `openspec validate`. Structural validation stays with the CLI.
  This tool assumes a change that parses and reports what it finds when
  one does not.
- Diffing proposal, design and tasks semantically. Those are prose; the
  tool shows them as line diffs.
- Posting review comments to a forge. Notes export as Markdown; the
  reviewer pastes them.
- Collision detection across pull requests. Only changes present in the
  working tree are checked.
