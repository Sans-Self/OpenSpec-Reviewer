# init-source-globs

## Why

`lint init` writes every extension it finds into both `source_globs` and
`path_extensions`. Those fields mean different things: `path_extensions`
is what a spec may cite, `source_globs` is what the lint reads looking for
citations. A JSON file has no comment syntax and no place for a
`spec:` tag, so scanning it is a wasted read on every run.

## What Changes

- `lint init` leaves `json` out of `source_globs`. Every other extension
  found stays: a stylesheet or a YAML file can carry a citation in a
  comment.
- `path_extensions` is unchanged: a spec citing a JSON fixture is a
  legitimate path citation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `citations`: "`lint init` writes a starting configuration" separates
  the two extension lists.

## Impact

- `src/citations/config.rs`: the renderer filters the globs.
- One test.
