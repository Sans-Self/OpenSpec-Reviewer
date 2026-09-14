# lint-init

## Why

The lint refuses to run without `openspec/reviewer.toml` and prints a
template in the refusal. That is correct and unhelpful in the same
breath: the template names `src`, `tests` and `**/*.rs`, which is wrong
for every repository that is not this one, and the reader has to copy it
out of stderr, create the file, then edit half the values. Opake needed
four of the six fields changed by hand before the first useful run.

## What Changes

- `openspec-reviewer lint init` writes `openspec/reviewer.toml` and
  fills it from what the repository already shows: the source roots that
  exist among the usual candidates, the file extensions actually found
  under them, and path prefixes drawn from the same roots plus `docs`.
  Fields the tool cannot guess, the call helper and the change scopes,
  are written as commented examples.
- `init` refuses to overwrite an existing file and refuses to run outside
  an `openspec/` tree, both with exit status `2`.
- The no-config refusal names `lint init` as the fix, alongside the
  minimal example it already shows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `citations`: adds the requirement that `lint init` writes a starting
  configuration derived from the repository; the requirement
  "Configuration lives beside the specs" gains the pointer to `lint init`
  in its refusal.

## Impact

- `src/main.rs`: `lint` grows an `init` subcommand.
- `src/citations/config.rs`: the template becomes a function of what the
  repository contains; the refusal message changes.
- `src/source/`: a small probe that lists top-level directories and the
  extensions under them.
- Tests under `tests/citations.rs` and the README's Citations section.

## Non-goals

- Editing or merging an existing file. `init` creates; the editor edits.
- Guessing change scopes. A naming convention is a decision, not a
  measurement.
