# Tasks: shell-completion

## 1. Dependency and entry point

- [ ] 1.1 `clap_complete = { version = "=4.6.0", features = ["unstable-dynamic"] }`
      in `Cargo.toml`.
- [ ] 1.2 `CompleteEnv::with_factory(Cli::command)` narrowed to bash,
      zsh and fish runs before `Cli::parse` in `main`.

## 2. Candidates

- [ ] 2.1 `source/complete.rs`: `change_names(root)` lists the
      directories under `openspec/changes/` without `archive`, sorted;
      an unreadable directory yields an empty list.
- [ ] 2.2 `source/complete.rs`: `git_refs(root)` lists heads and tags
      by short name; a failing `git` yields an empty list.
- [ ] 2.3 `source/complete.rs`: `pull_requests(root)` runs `gh pr list
      --json number,title` under a one-second deadline, killing the child
      past it; each row is a candidate with the number as value and the
      title as help; any failure yields an empty list.
- [ ] 2.4 `change <name>` carries an `ArgValueCandidates` over
      `change_names`; `git <REF>` and `--base` carry one over
      `git_refs`; `gh <pr>` carries one over `pull_requests`; `diff
      <path>` carries `ValueHint::FilePath`.

## 3. Documentation and tests

- [ ] 3.1 README section "Shell completion" with the source line for
      bash, zsh and fish.
- [ ] 3.2 Tests titled by requirement: `COMPLETE=fish` prints a stub
      that names the binary; `change_names` on a fixture repo lists the
      changes and not `archive`; `git_refs` on a temp repo lists its
      branch; `pull_requests` with a stub `gh` on `PATH` that sleeps
      returns an empty list within the deadline.
- [ ] 3.3 `clippy` and `rustfmt` clean.
