## Why

`define` drafts glossary terms from the lint's "recurring term without
definition" notes, and the lint only sees backticked and double-quoted
spans. A specification written in plain prose, where the chain head is
"the chain head" and nobody marks it up, produces no notes, so `define`
has nothing to draft and the lint reports the vocabulary is fine when it
is undefined. Large repositories written before the glossary existed
are mostly that kind of prose.

## What Changes

- A sixth shipped skill, `discover`, with a `/opsx-reviewer:discover`
  command. It reads every requirement name and body in canon, collects
  the noun phrases that recur across two or more capabilities, drops
  what the glossary already knows, merges the list with the lint's span
  notes, and hands the result to `define` with the requirements each
  phrase appears in. It writes nothing itself; `define` writes the
  change.
- The command alias requirement counts five commands, and the shape
  requirement for task skills covers `discover`.
- The workflow skill routes to `discover` when a repository has a
  glossary and the lint is quiet.

Not in this change: a phrase-recurrence check in the tool. That is a
change to the `glossary` capability with its own design, and the skill
is what a project can use today.

## Capabilities

### Modified Capabilities

- `skills`: one more user-callable skill, and its shape.
