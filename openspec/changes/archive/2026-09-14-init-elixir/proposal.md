# init-elixir

## Why

`lint init` recognizes a fixed list of file extensions. Elixir is not on
it, so a repository whose Phoenix tests cite requirements in `# spec:`
comments gets a configuration that never reads them. Opake is exactly
that repository, and its `_build` and `deps` directories are not in the
default skip list either.

## What Changes

- `ex`, `exs` and `heex` join the candidate extensions.
- `_build` and `deps` join the default skip directories.

## Capabilities

### Modified Capabilities

- `citations`: "`lint init` writes a starting configuration" lists the
  new extensions and skip directories.

## Impact

- Two constant lists in `src/citations/config.rs`, one test.
