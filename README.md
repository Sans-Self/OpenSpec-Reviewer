<h1 align="center">openspec-reviewer</h1>

<p align="center">
  Review an <a href="https://github.com/Fission-AI/OpenSpec">OpenSpec</a> change as the semantic diff it is,<br>
  not the file diff git shows.
</p>

<p align="center">
  <a href="https://github.com/Sans-Self/OpenSpec-Reviewer/actions/workflows/nix.yml"><img alt="nix build" src="https://github.com/Sans-Self/OpenSpec-Reviewer/actions/workflows/nix.yml/badge.svg"></a>
  <a href="https://github.com/Sans-Self/OpenSpec-Reviewer/tags"><img alt="release" src="https://img.shields.io/github/v/tag/Sans-Self/OpenSpec-Reviewer?label=release&sort=semver"></a>
  <a href="https://sans-self.cachix.org"><img alt="cachix" src="https://img.shields.io/badge/cachix-sans--self-5c6bc0"></a>
  <a href="LICENSE"><img alt="MIT" src="https://img.shields.io/badge/licence-MIT-green"></a>
</p>

A delta spec restates every requirement it modifies. To git that is a new
file full of added lines; to a reviewer the actual change is a few words
and maybe a scenario. `openspec-reviewer` pairs each delta requirement
with its canonical counterpart in the repository you run it from and
shows the difference at word level, with findings for the things git
cannot see: a dropped scenario, a MODIFIED requirement that has no canon
target, a second open change touching the same requirement. You approve
items one by one, leave notes, and export them for the pull request.

- [Installation](#installation)
- [Usage](#usage)
- [What a review looks like](#what-a-review-looks-like)
- [Citations](#citations)
- [Glossary](#glossary)
- [Ignoring a finding](#ignoring-a-finding)
- [Skills](#skills)
- [Development](#development)

## Installation

The repository is a nix flake with a package output, so nothing needs
cloning:

```sh
nix run github:Sans-Self/OpenSpec-Reviewer/v0.1.0 -- lint
nix profile install github:Sans-Self/OpenSpec-Reviewer/v0.1.0
```

Or pin it in your own flake and put it in the dev shell:

```nix
inputs.openspec-reviewer = {
  url = "github:Sans-Self/OpenSpec-Reviewer/v0.1.0";
  inputs.nixpkgs.follows = "nixpkgs";
};

devShells.default = pkgs.mkShell {
  packages = [ openspec-reviewer.packages.${system}.default ];
};
```

CI builds every tag for `x86_64-linux` and `aarch64-darwin` and pushes
the results to the `sans-self` Cachix cache. Trust it once to skip the
compile:

```sh
cachix use sans-self
```

The `openspec` CLI is a separate npm package the skills call for
`openspec validate` and `openspec new change`; the reviewer itself does
not need it. `npm install -g @fission-ai/openspec` or the dev shell's
wrapper provide it.

## Usage

Run from the root of a repository that has an `openspec/` directory.
Canon is always read from that working directory.

```sh
openspec-reviewer change sweep-gate            # the change as it is on disk
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

| mark | meaning |
| --- | --- |
| `[ ]` `[√]` `[~]` | not approved, approved, text changed since approval |
| `+` `~` `-` `>` | requirement added, modified, removed, renamed |
| `!` `?` `✎` | an error finding, a warning, a note |
| `[-word-]` `{+word+}` | removed and added words inside a changed paragraph |

Paragraphs and scenarios equal on both sides print once, unmarked.
Findings, the notes and a one-line history follow each requirement. A
note anchors to the requirement or to one of its scenarios, prints its
anchor, and says so when the words it was written about have changed
since. In the interactive view the same rows sit in a list on the left
with the diff on the right; `Space` folds a requirement's scenarios open,
`e` writes a note in a popup over the diff, and `?` lists the keys.

`N` puts every note of the change on one screen, each row naming its
anchor, quoting the note's first line and marking the ones whose words
have moved since. `Enter` goes to the note's row, unfolding the
requirement when the note hangs on one of its scenarios. `d` deletes the
selected note and `X` clears every note of the change, after a `y`/`n`
confirmation. Both touch the state file only: approvals stay.

## Citations

Specs cite evidence and tests cite specs. A test title, a comment or a
`cite()` call carrying `spec:<capability> § <requirement name>` must name
a requirement that exists in canon or that an open change adds. Inside
backticks a citation may wrap across lines, comment markers included, so
a moduledoc can cite at 80 columns. A canon spec that names
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
## Ignoring a finding

Two checks report things a project sometimes cannot act on: a word
invented as an example inside a scenario will never have a definition,
and a requirement about an output format is asserted by tests that
cannot name it. Both can be dismissed in writing, with the reason next
to the entry.

```toml
[[definitions.ignore]]
term   = "mountType"
in     = ["citations", "glossary § A term lists the words that are acceptable for it"]
reason = "a configuration key quoted in prose, not a concept"

[[lint.ignore_uncited]]
requirement = "citations § Coverage lists citing tests per requirement"
reason      = "asserted by the ledger snapshot, which cannot cite itself"
```

Every value is an exact string; nothing is read as a pattern. `in` is
optional and always an array, each element a capability or a
`<capability> § <requirement name>`, and without it the term is ignored
everywhere. An entry with no `reason` is a configuration error.

An ignored requirement still counts in the coverage total, which closes
with `coverage: <cited>/<total> (<n> ignored)`; a figure that improves
when somebody edits a configuration is a figure that lies. When an entry
stops matching anything — the term got defined, the requirement was
removed, one scope of several went quiet — the tool warns and asks for
it back out. The verdict comes from the register, every requirement
canon and the open changes assert, so `lint` and a review say the same
thing about the same entry.
## Skills

`openspec-reviewer skills install` writes six skills an agent loads to
work with the reviewer. `opsx-reviewer-workflow` is for the agent: it
loads on its own in a repository with `openspec/` and the binary, and
says when to run the review and the lint, what every finding kind means,
how to write a citation, and which skill to reach for next. The other
five are for you and the agent both, each with a `/opsx-reviewer:<name>`
command:

- `define` drafts glossary terms from the lint's recurring undefined
  terms, into a new change.
- `discover` reads canon for the concepts written in plain prose that
  recur across capabilities, which the span check cannot see, and hands
  them to `define`.
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

Specs live under `openspec/`. The tool reviews its own changes, and
`.github/workflows/nix.yml` builds every push on Linux and macOS.
