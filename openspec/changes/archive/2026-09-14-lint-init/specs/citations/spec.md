# citations (delta)

## ADDED Requirements

### Requirement: `lint init` writes a starting configuration

`openspec-reviewer lint init` MUST create `openspec/reviewer.toml` with
`source_roots` set to the directories that exist in the working directory
among `src`, `lib`, `apps`, `packages`, `crates`, `services`, `tests` and
`test`; `source_globs` and `path_extensions` set to the extensions found
in files under those roots, drawn from `rs`, `ts`, `tsx`, `js`, `mjs`,
`mts`, `py`, `go`, `md`, `json`, `yaml`, `yml`, `toml` and `css`;
`skip_dirs` set to `node_modules`, `target`, `dist` and `build`;
`path_prefixes` set to the source roots plus `docs` when it exists;
`test_pattern` set to `bug__\w+`; and `cite_helper`, `change_scopes` and
`grandfathered` present as commented examples. It MUST print the path it
wrote to stdout. When the file exists it MUST refuse with an error naming
the path and exit `2`, leaving the file untouched. When `openspec/` does
not exist it MUST refuse with an error naming that directory and exit
`2`.

#### Scenario: Fresh repository

- **GIVEN** a repository with `openspec/`, `crates/a/src/x.rs` and `apps/web/y.tsx`
- **AND** no `openspec/reviewer.toml`
- **WHEN** the user runs `openspec-reviewer lint init`
- **THEN** `openspec/reviewer.toml` exists
- **AND** it parses with `source_roots = ["apps", "crates"]`
- **AND** `path_extensions` contains `rs` and `tsx`
- **AND** `path_extensions` does not contain `py`
- **AND** the exit status is `0`

#### Scenario: Config already exists

- **GIVEN** a repository with `openspec/reviewer.toml`
- **WHEN** the user runs `openspec-reviewer lint init`
- **THEN** the file is unchanged
- **AND** stderr names `openspec/reviewer.toml`
- **AND** the exit status is `2`

#### Scenario: No openspec directory

- **GIVEN** a directory without `openspec/`
- **WHEN** the user runs `openspec-reviewer lint init`
- **THEN** stderr names `openspec/`
- **AND** the exit status is `2`

#### Scenario: Written file lints

- **GIVEN** a repository where `lint init` has run
- **WHEN** the user runs `openspec-reviewer lint`
- **THEN** the lint reads the file without a configuration error

## MODIFIED Requirements

### Requirement: Configuration lives beside the specs

The lint MUST read `openspec/reviewer.toml`. It holds the source roots,
source globs, skipped directories, path prefixes, path extensions, test
pattern, call helper name, change scopes, and grandfathered changes.
Without the file the lint MUST refuse to run, with an error naming the
path, naming `openspec-reviewer lint init` as the way to create it, and
showing a minimal example of the file. A malformed file is an error
naming the key.

#### Scenario: No config

- **GIVEN** a repository without `openspec/reviewer.toml`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that path
- **AND** the error names `lint init`
- **AND** the error shows a minimal example of the file
- **AND** the exit status is `2`

#### Scenario: Bad key

- **GIVEN** a config with `source_root = "apps"` instead of `source_roots`
- **WHEN** the lint runs
- **THEN** it stops with an error naming the unknown key
