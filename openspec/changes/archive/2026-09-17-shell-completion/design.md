# Design: shell-completion

## Context

`clap_complete` offers two mechanisms. Static generation writes a
script from the command tree once; it cannot know the change names in a
repository. Dynamic completion makes the binary the completer: the
shell calls it with the current command line and clap computes the
candidates, running any `ArgValueCandidates` an argument carries. In
`clap_complete` 4.6.0 the dynamic mode still sits behind the
`unstable-dynamic` feature and its types have moved between minor
versions.

## Goals / Non-Goals

**Goals:** change names complete; every fixed word completes; pull
requests complete with their titles; tab never hangs.

**Non-Goals:** shells nobody here runs; writing into the user's shell
configuration.

## Decisions

**Dynamic, and only dynamic.** Completing change names is the reason to
do this at all, and only the dynamic mode can. Shipping the static
scripts beside it would be a second mechanism with a subset of the
behaviour; the stub the dynamic mode prints is installed the same way a
static script would be, so nothing is lost by having one.

**The clap convention for the entry point.** `COMPLETE=<shell>
openspec-reviewer` prints the stub, as every tool built on
`CompleteEnv` does, so a user who knows one knows this one. It runs
before `Cli::parse` and exits when the variable is set; otherwise the
program continues untouched. `CompleteEnv::shells` narrows the set to
bash, zsh and fish, the shells in use here; any other value of
`COMPLETE` is an error naming the three. No `completions` subcommand: it would be a
second way to do the same thing and would appear in the command tree
the completions themselves list.

**Candidates come from `source/`.** Reading `openspec/changes/` and
asking git for refs are filesystem and process I/O, which this crate
keeps in `source/` and `render/`. `source::complete` exposes three
functions, `change_names(root)`, `git_refs(root)` and
`pull_requests(root)`, each returning a list of candidates and an empty
list on any error. A completer that fails offers nothing; it never
prints an error into the user's command line.

**`archive` is not a change.** It is the directory the archived changes
move into, so it is filtered from the candidates. Any other directory
under `openspec/changes/` is offered, whether or not it validates: the
user may be about to review a broken one.

**Refs, not commits.** `git <REF>` completes from
`git for-each-ref --format=%(refname:short)` over heads and tags. A
commit hash is typed, not completed.

**`gh` under a deadline.** `gh pr list --json number,title` is the one
network call, and fish and zsh block the prompt until a completer
returns, so it runs under a one-second deadline: the child is spawned,
polled with `try_wait` every twenty milliseconds, and killed when the
deadline passes, with no candidates offered. Inside the deadline each
pull request becomes a candidate whose value is the number and whose
help text is the title, so `gh <TAB>` reads as a list of pull requests
rather than of integers. `gh` has no timeout flag of its own, which is
why the bound lives here.

**Pinned.** `clap_complete = "=4.6.0"`. The API under `unstable-dynamic`
is the part of the dependency that has changed most, and a `cargo
update` should not be able to break the build.

## Risks / Trade-offs

- [The `unstable-dynamic` API changes again] → the pin holds the build;
  an upgrade is a deliberate task.
- [The stub needs the binary on `PATH` under the name the shell calls]
  → `CompleteEnv` uses the invoked path; the README says to install the
  binary before sourcing the stub.
- [`gh` is slow on a large repository] → the deadline caps the pause
  at one second; the candidates are the loss, not the prompt.
- [Killing `gh` leaves its output half-written] → the completer reads
  stdout only after a clean exit; a killed child yields nothing.
