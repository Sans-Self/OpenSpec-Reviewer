# Design: lint-ignore-evidence

## Context

The lint reports four families of missing evidence — a cited path, a
cited regression test, a cited commit, a citation that resolves against
nothing — each built in `lint()` from a checker's output. All four are
errors, and none can be dismissed.

## Goals / Non-Goals

**Goals:** a written, audited reason for every piece of evidence the lint
must not chase; a clean `lint` on this repository.

**Non-Goals:** patterns, scoping by capability, or silencing a family
wholesale.

## Decisions

**One list, four keys.** `[[lint.ignore_evidence]]` takes exactly one of
`citation`, `path`, `test` or `commit`. Four separate lists would repeat
the reason handling four times, and a single `evidence` key would make
`src/a.rs` and `alpha § Some rule` the same kind of string, so a typo in
one would silence the other.

Naming the kind also keeps the match cheap and exact: an entry is
compared only against findings of its own family, as an equal string. The
existing lists match exact strings and the specification says so; a third
that quietly accepted globs would be the surprise.

**Staleness is judged from the run.** An evidence entry silences nothing
when no finding of its kind carried its string. That is a property of the
repository's specs rather than of the invocation — `lint` and `change`
read the same specs — so the warning is stable, and it is computed where
the findings are, rather than threaded into `dangling_ignores` with the
register it does not need.

**No `in`.** `[[definitions.ignore]]` confines a term because a word
recurs across capabilities and means different things. A path or a commit
is one string with one meaning, and the citation strings that need
ignoring are made-up capabilities that appear nowhere else.
