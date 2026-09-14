# openspec-reviewer — Agent Guide

Rust CLI and TUI that reviews OpenSpec changes against canon. One crate,
binary `openspec-reviewer`. Read `README.md` for what it does and
`openspec/changes/*/design.md` for how.

## Toolchain

Everything comes from the nix flake: `direnv allow` once, or
`nix develop`. Non-interactive shells don't fire the direnv hook, so
prefix commands with `direnv exec .` from the repo root
(`direnv exec . cargo test`, `direnv exec . openspec validate <change>`).
The shell provides `cargo`, `rustfmt`, `clippy`, `rust-analyzer`, `git`,
`gh` and `openspec` (a pinned dlx wrapper; first call downloads).

## Conventions

- Specs first. Work is proposed as an OpenSpec change under
  `openspec/changes/` and implemented from its `tasks.md`. Artefact style
  rules live in `openspec/config.yaml` under `rules`; the scenario rule
  is one condition per keyword line with `- **AND**`.
- Domain code is pure. `model`, `review`, `state` and `citations` take
  values in and return values out; filesystem, git, gh and the terminal
  live in `source/` and `render/`.
- Failures are values: typed error enums with `thiserror`, `Result` up
  to `main`. No panics for expected conditions.
- Blocking I/O, no async runtime. The reasoning is in the foundation
  design under "Why blocking I/O".
- Tests cite the requirement they cover by name in the test title, one
  per requirement at minimum. Fixtures live under `tests/fixtures/`.
- Prose in Oxford spelling; `rustfmt` and `clippy` clean before commit.
- Commits are imperative and typed (`feat:`, `fix:`, `chore:`, `docs:`,
  `test:`), no attribution trailers.

## Self-hosting

`openspec-reviewer change <name>` reviews this repo's own changes once
the binary exists. Until then, `openspec validate <name>`.
