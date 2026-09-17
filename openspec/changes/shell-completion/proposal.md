# shell-completion

## Why

`openspec-reviewer change <name>` is the command typed most, and the
name is a directory under `openspec/changes/` the user has to remember
or look up. The shell can offer it. Every other argument of the tool is
a fixed word too: subcommands, `--format` values, the global flags.
None of it completes today.

## What Changes

- The binary answers the shell's completion requests itself, through
  `clap_complete`'s dynamic mode: `COMPLETE=<shell> openspec-reviewer`
  prints a stub for bash, zsh or fish, and the stub calls the binary
  back on every tab press.
- Subcommands, their flags, the `--format` values and the
  `lint` and `skills` actions complete from the command tree.
- `change <name>` completes from the directories under
  `openspec/changes/` in the current directory, `archive` excluded.
- `diff <path>` completes as a path. `git <REF>` and `--base <REF>`
  complete from `git for-each-ref` names.
- `gh <pr>` completes from `gh pr list`, number and title, with a
  one-second bound: past it the completer kills `gh` and offers
  nothing, so tab pauses at most a second and never hangs.
- README gets a "Shell completion" section with the one line per shell.

## Capabilities

### New Capabilities

- `shell-completion`: the completion entry point, which shells it
  serves, and where each argument's candidates come from.

### Modified Capabilities

None.

## Impact

- `Cargo.toml`: `clap_complete` with `unstable-dynamic`, pinned to
  `=4.6.0` because the dynamic API has moved between minors.
- `src/main.rs`: `CompleteEnv` runs before `Cli::parse`; the `change`
  and `git` arguments carry completers.
- New `src/source/complete.rs`: the candidate lookups, which read the
  filesystem, git and gh and so belong in `source/`.
- `README.md`: the new section.

## Non-goals

- Elvish and PowerShell. Nobody here uses them; a shell is a change.
- Installing the stub into a shell's configuration for the user. The
  line is printed, and where it goes is the shell's business.
- Static generated scripts. One mechanism, not two.
