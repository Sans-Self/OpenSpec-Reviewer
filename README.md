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
openspec-reviewer change sweep-gate      # the change as it is on disk
openspec-reviewer diff pr.patch                # a unified diff, applied against this checkout
gh pr diff 224 | openspec-reviewer diff        # same, from stdin
openspec-reviewer git feature/foo --base main  # two refs, no patching
openspec-reviewer gh 224                       # a pull request through the gh CLI
```

`lint --coverage`, the citation lint, is proposed in the
`reviewer-citations` change and not built yet.

Interactive when stdout is a terminal. Plain text when piped, or with
`--plain`; `--format json` for tooling and agents; `--format markdown`
for the notes you wrote; `--findings-only` for CI. Exit status is `2` on
errors, `1` on warnings, `0` otherwise.

Approvals and notes live under `$XDG_STATE_HOME/openspec-reviewer/`, per
repository and change. `--no-state` ignores them.

## What a review looks like

A delta that restates a requirement to change one sentence, add two
paragraphs and two scenarios is, to git, a new file of added lines. The
reviewer pairs it with canon and shows this:

```
# change sweep-gate  (change sweep-gate)

## key-rotation

[ ] ~ The re-wrap sweep is hygiene under the background-work contract
      Requirement: The re-wrap sweep is hygiene under the background-work contract

    ~ Migrating existing documents' content-key wraps from historical group
      keys to the current one SHALL be a background task [...] The sweep
      [-bounds key-history walks; it has no security effect — a re-wrap does
      not and cannot revoke-]{+reduces use of historical wraps; it has no
      revocation effect — a re-wrap cannot revoke+} anything a former member
      could already unwrap.
    + Before replacing a document's sole content-key wrap, the runner SHALL
      check that every currently admitted member has a usable wrap [...]
    + The runner SHALL re-evaluate the current head and eligibility for each
      item [...]

    ~ Scenario: interrupted sweep needs no recovery
    ~ - **WHEN** a sweep is interrupted with half a workspace's {+eligible +}wraps migrated
    ~ - **THEN** [...]

    + Scenario: pending member retains an old document
    + - **GIVEN** Carol remains admitted, holds group key 7, and lacks the current key 8
    + - **WHEN** maintenance encounters a document whose sole content-key wrap uses key 7
    + - **THEN** it leaves that wrap unchanged [...]

    + Scenario: canonical removal releases the sweep gate
    + [...]

    history: none

summary: 0 errors, 0 warnings, 0 notes
```

Each requirement row carries its approval mark (`[ ]`, `[√]`, or `[~]`
when the text changed since approval), the delta kind (`+` added, `~`
modified, `-` removed, `>` renamed), then `!` for an error finding, `?`
for a warning and `✎` for a note. Changed paragraphs mark removed words
`[-like this-]` and added words `{+like this+}`; paragraphs and scenarios
equal on both sides print once, unmarked. Findings, the note and a
one-line history follow each requirement. In the interactive view the
same rows sit in a list on the left with the diff on the right; `?` lists
the keys.

## Development

```sh
direnv allow      # or: nix develop
cargo test
cargo run -- --help
```

Specs live under `openspec/`. The tool reviews its own changes.
