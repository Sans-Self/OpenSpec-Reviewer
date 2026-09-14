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

```sh
openspec-reviewer lint init                    # write openspec/reviewer.toml from the tree
openspec-reviewer lint                         # every citation against the repository
openspec-reviewer lint --coverage              # plus the per-requirement ledger of citing tests
openspec-reviewer lint --format json
```

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

## Citations

Specs cite evidence and tests cite specs. A test title or a `cite()` call
carrying `spec:<capability> § <requirement name>` must name a requirement
that exists in canon or that an open change adds. A canon spec that names
a path, a `bug__` regression test or a commit hash must name one that
exists. `lint` checks both directions and, per open change, the blast
radius: a REMOVED requirement something outside its capability still
cites is an error, a MODIFIED one lists its citers as a note. The same
results appear as findings on the matching rows when you review the
change, with the citing files listed under the finding.

The lint reads `openspec/reviewer.toml` and refuses to run without it.
`openspec-reviewer lint init` writes one from what the repository
contains: the source directories that exist among the usual names, the
file extensions found under them, and commented examples for the fields
it cannot measure. Every field is optional; a field left out switches
that check off and is named in the summary line.

```toml
[lint]
source_roots    = ["apps", "crates", "packages", "tests"]
source_globs    = ["**/*.rs", "**/*.ts", "**/*.tsx"]
skip_dirs       = ["node_modules", "target", "dist"]
path_prefixes   = ["apps", "crates", "packages", "docs"]
path_extensions = ["rs", "ts", "tsx", "md", "json", "yaml", "toml"]
test_pattern    = "bug__\\w+"
cite_helper     = "cite"          # cite('capability', 'Requirement name')
change_scopes   = ["auth", "tree", "keyring"]
grandfathered   = []

[term_drift]
max_common = 5
```

Term drift needs no configuration beyond `max_common`. When a change
removes a backticked identifier, a quoted string or a phrase from a
requirement and a canon requirement in another capability still uses it,
the pairing gets a warning naming the sibling. A renamed requirement
whose old name still appears in sibling prose gets the same.

## Glossary

`openspec/specs/definitions/spec.md` is the project's glossary when it
exists. Each requirement is a term: the name is the word, the body its
meaning, the scenarios usage examples, and a `- **Deprecated:** old word,
other word` line lists the words not to use for it. Because a term is an
ordinary requirement it is cited, renamed, diffed and tracked like any
other.

The review warns when a delta uses a deprecated synonym or introduces a
backticked or quoted term twice without defining it, and notes the
requirements that use a term whose meaning a change edits. The tool
proposes only what specs already mark with backticks or double quotes,
and finds defined terms and their synonyms anywhere in prose. `lint` warns
on deprecated synonyms in canon, notes terms nobody uses, and notes
spans that recur across capabilities without a definition. In the TUI,
`D` opens the definitions of the terms the selected requirement uses.

```toml
[definitions]
capability     = "definitions"   # which capability is the glossary; "" switches it off
min_recurrence = 3               # how often an undefined span must recur
```

## Skills

`openspec-reviewer skills install` writes five skills an agent loads to
work with the reviewer. `opsx-reviewer-workflow` is for the agent: it
loads on its own in a repository with `openspec/` and the binary, and
says when to run the review and the lint, what every finding kind means,
how to write a citation, and which skill to reach for next. The other
four are for you and the agent both, each with a `/opsx-reviewer:<name>`
command:

- `define` drafts glossary terms from the lint's recurring undefined
  terms, into a new change.
- `cite` adds `spec:` citations to the tests that already exercise
  uncited requirements.
- `crossref` judges the siblings a change puts in question, quotes the
  archived decision if there was one, and drafts sibling deltas into the
  change.
- `triage` walks a change's findings in severity order and applies the
  usual fix to the change's deltas, on confirmation, routing judgment to
  crossref.

The skills go to `.claude/skills/` and, when `.agents/` exists, to
`.agents/skills/` for Codex, OpenCode and omp; the commands go to
`.claude/commands/opsx-reviewer/`. Install never creates `.agents/` and
refuses without `.claude/`. A file you edit by hand is kept on the next
install and named; `skills list` shows each file's state. To rewrite a
skill's instructions for one project, put the body at
`openspec/reviewer/skills/<name>.md`; the shipped frontmatter stays.
Every skill writes into a change, never into `openspec/specs/`.

## Development

```sh
direnv allow      # or: nix develop
cargo test
cargo run -- --help
```

Specs live under `openspec/`. The tool reviews its own changes.
