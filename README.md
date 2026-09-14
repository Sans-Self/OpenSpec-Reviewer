# openspec-reviewer

Review an [OpenSpec](https://github.com/Fission-AI/OpenSpec) change as the
semantic diff it is, not the file diff git shows.

A delta spec restates every requirement it modifies. To git that is a new
file full of added lines; to a reviewer the actual change is a few words
and maybe a scenario. `openspec-reviewer` pairs each delta requirement
with its canonical counterpart in the repository you run it from and
shows the difference at word level, with findings for the things git
cannot see: a dropped scenario, a MODIFIED requirement that has no canon
target, a second open change touching the same requirement. You approve
items one by one, leave notes, and export them for the pull request.

## Usage

Run from the root of a repository that has an `openspec/` directory.
Canon is always read from that working directory.

```sh
openspec-reviewer change energiehuis-fork      # the change as it is on disk
openspec-reviewer diff pr.patch                # a unified diff, applied against this checkout
gh pr diff 224 | openspec-reviewer diff        # same, from stdin
openspec-reviewer git feature/foo --base main  # two refs, no patching
openspec-reviewer gh 224                       # a pull request through the gh CLI
openspec-reviewer lint --coverage              # citation lint, both directions
```

Interactive when stdout is a terminal. Plain text when piped, or with
`--plain`; `--format json` for tooling and agents; `--format markdown`
for the notes you wrote; `--findings-only` for CI. Exit status is `2` on
errors, `1` on warnings, `0` otherwise.

Approvals and notes live under `$XDG_STATE_HOME/openspec-reviewer/`, per
repository and change. `--no-state` ignores them.

## Development

```sh
direnv allow      # or: nix develop
cargo test
cargo run -- --help
```

Specs live under `openspec/`. The tool reviews its own changes.
