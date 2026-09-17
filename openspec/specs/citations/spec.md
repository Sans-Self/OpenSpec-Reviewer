# citations Specification

## Purpose
TBD - created by archiving change reviewer-citations. Update Purpose after archive.
## Requirements
### Requirement: A citation names a capability and a requirement

The lint MUST read `spec:<capability> § <requirement name>` inside a
backtick span or a double-quoted string as a citation. The capability is
lowercase letters, digits and hyphens. The requirement name runs to the
closing delimiter, so an apostrophe does not end it. Inside a backtick
span the name MAY continue across line breaks until the closing backtick,
and a comment marker (`#`, `//`, `///`, `//!`, `*` or `--`) at the start
of a continuation line is not part of the name. Runs of whitespace in the name, line breaks included, compare as one
space, and trailing `.`, `,`, `;` or `:` is not part of the name. When
`openspec/reviewer.toml` names a call helper,
`<helper>("<capability>", "<requirement>")` with either quote style is
also a citation.

#### Scenario: Literal citation

- **GIVEN** a test title containing `` `spec:sitemap-index § Flat index of all routes and pages` ``
- **WHEN** the lint scans the file
- **THEN** it records one citation of that capability and requirement

#### Scenario: Apostrophe in the name

- **GIVEN** a citation in double quotes whose name contains `page's`
- **WHEN** the lint scans the file
- **THEN** the recorded name includes the apostrophe and the words after
  it

#### Scenario: Call helper

- **GIVEN** a config naming the helper `cite`
- **AND** source containing `cite('alpha', 'Some rule')`
- **WHEN** the lint scans the file
- **THEN** it records one citation of `alpha § Some rule`

#### Scenario: Backticked citation wraps

- **GIVEN** a comment containing `` `spec:lineage § Lineage is`` at the end of one line
- **AND** ``the chain's genesis URI` `` at the start of the next
- **WHEN** the lint scans the file
- **THEN** it records one citation of `lineage § Lineage is the chain's genesis URI`

#### Scenario: Backticked citation wraps inside a comment

- **GIVEN** a `#` comment containing `` `spec:lineage § Lineage is`` at the end of one line
- **AND** ``# the chain's genesis URI` `` on the next line
- **WHEN** the lint scans the file
- **THEN** it records one citation of `lineage § Lineage is the chain's genesis URI`

#### Scenario: Sentence-final punctuation

- **GIVEN** prose containing `See spec:alpha § Some rule.`
- **WHEN** the lint scans the file
- **THEN** it records one citation of `alpha § Some rule`

### Requirement: A citation must resolve

Every citation MUST name a capability that exists in canon or that an
open change adds to, and a requirement that exists in that capability's
canon or is listed under ADDED in an open change. Otherwise the lint
reports an error naming the citing file and the citation. When the
unresolved name ended at a line break, the error MUST add that the name
looks cut off by a line break and that a backtick span may wrap.

#### Scenario: Spec-first test

- **GIVEN** an open change that adds requirement R to capability C
- **AND** canon that does not have R
- **WHEN** a test cites `C § R`
- **THEN** the lint reports no error for it

#### Scenario: Renamed requirement

- **GIVEN** canon where R was renamed to S
- **AND** a test that still cites R
- **WHEN** the lint runs
- **THEN** it reports an error naming the test file and R

#### Scenario: Unknown capability

- **WHEN** a citation names a capability no spec and no change has
- **THEN** the lint reports an error saying the capability does not
  exist

#### Scenario: Bare prose wraps

- **GIVEN** a comment containing `See spec:alpha § Some rule that` at the end of one line
- **AND** `continues here.` on the next line
- **AND** canon `alpha § Some rule that continues here`
- **WHEN** the lint runs
- **THEN** it reports an error naming `alpha § Some rule that`
- **AND** the error says the name looks cut off by a line break
- **AND** the error says a backtick span may wrap

### Requirement: Citations are scanned in specs, deltas and source

The lint MUST scan canon specs, delta specs of open changes, and source
files under the configured roots matching the configured globs. It MUST
skip the directories listed in the configuration. A citation in a spec
or delta records the citing capability; a citation in source records
none.

#### Scenario: Source under two roots

- **GIVEN** a config with roots `apps` and `packages`
- **AND** a citation in a file under each
- **WHEN** the lint runs
- **THEN** both citations are recorded

#### Scenario: Skipped directory

- **GIVEN** a config that skips `node_modules`
- **AND** a citation inside `packages/x/node_modules/y.ts`
- **WHEN** the lint runs
- **THEN** that citation is not recorded

### Requirement: A cited path must exist

In canon specs, text that starts with a configured path prefix and ends
with a configured extension is a path citation. The lint MUST report an
error for every cited path that does not exist in the working tree.

#### Scenario: Moved file

- **GIVEN** a spec citing `packages/ui/src/old.ts`
- **AND** no such file
- **WHEN** the lint runs
- **THEN** it reports an error naming the spec and the path

#### Scenario: Dotfile prefix

- **GIVEN** a config with prefix `.github`
- **AND** a spec citing `.github/workflows/ci.yml`
- **WHEN** the lint runs
- **THEN** it checks that path like any other

### Requirement: A cited test must exist in source

In canon specs, text matching the configured test pattern is a test
citation. The lint MUST report an error when the text appears in no
source file under the configured roots.

#### Scenario: Renamed regression test

- **GIVEN** a spec citing `bug__float_not_rounding`
- **AND** source where that name no longer appears
- **WHEN** the lint runs
- **THEN** it reports an error naming the spec and the test

### Requirement: A cited hash must be a commit

In canon specs, a backticked run of seven to forty hex digits is a commit
citation. The lint MUST report an error when `git cat-file -e
<hash>^{commit}` fails in the working directory.

#### Scenario: Fabricated hash

- **GIVEN** a spec citing `` `deadbeef` ``
- **AND** no such commit
- **WHEN** the lint runs
- **THEN** it reports an error naming the spec and the hash

### Requirement: Removing a cited requirement is an error

For every requirement under REMOVED in an open change, the lint MUST
report an error for each citer outside the requirement's own capability,
unless the change also carries a delta for the citing capability. The
error names the change, the requirement and the citing file.

#### Scenario: Test still cites

- **GIVEN** a change that removes `C § R`
- **AND** a test that cites `C § R`
- **WHEN** the lint runs
- **THEN** it reports an error naming the change, `C § R` and the test
  file

#### Scenario: Citing spec updated in the same change

- **GIVEN** a change that removes `C § R`
- **AND** spec D cites `C § R`
- **AND** the change carries a delta for D
- **WHEN** the lint runs
- **THEN** it reports no error for that citer

### Requirement: Modifying a cited requirement lists its citers

For every requirement under MODIFIED in an open change, the lint MUST
report a note listing each citing file outside the requirement's own
capability. Notes do not change the exit status.

#### Scenario: Two outside citers

- **GIVEN** a change that modifies `C § R`
- **AND** two files outside C cite it
- **WHEN** the lint runs
- **THEN** it prints one note naming the change, `C § R` and both files

### Requirement: Change directories must be changes

Every directory under `openspec/changes/` other than `archive/` MUST
contain `proposal.md` or `.openspec.yaml`; otherwise the lint reports an
error saying the directory is not a change. When the configuration lists
change scopes, a change name MUST start with one of them followed by a
hyphen and at least one character, unless the name is listed as
grandfathered. A grandfathered name that no longer exists is an error.

#### Scenario: Nested grouping directory

- **GIVEN** `openspec/changes/ui/menu/` with no marker file in `ui/`
- **WHEN** the lint runs
- **THEN** it reports an error saying `ui` is not a change directory

#### Scenario: Name outside the scopes

- **GIVEN** scopes `ui` and `billing`
- **AND** a change named `menu-fix`
- **WHEN** the lint runs
- **THEN** it reports an error saying the name does not start with a
  scope

#### Scenario: Stale grandfather entry

- **GIVEN** a grandfathered name whose directory is gone
- **WHEN** the lint runs
- **THEN** it reports an error asking to remove the entry

### Requirement: Coverage lists citing tests per requirement

With `--coverage`, the lint MUST print, per capability, each requirement
of the register with the number of source files citing it, marking those
with zero, and a closing line
`coverage: <cited>/<total> (<ignored> ignored)`. A requirement named by a
`[[lint.ignore_uncited]]` entry MUST NOT be marked uncited and MUST still
be counted in `<total>`. The `(<ignored> ignored)` part is omitted when no
entry matched. Spec-to-spec citations do not count. The ledger never
changes the exit status.

#### Scenario: One uncited requirement

- **GIVEN** a capability with three requirements
- **AND** tests citing two of them
- **WHEN** the lint runs with `--coverage`
- **THEN** the ledger marks the third as uncited
- **AND** the closing line reads `coverage: 2/3`

#### Scenario: The uncited requirement is ignored

- **GIVEN** a capability with three requirements
- **AND** tests citing two of them
- **AND** an `ignore_uncited` entry naming the third
- **WHEN** the lint runs with `--coverage`
- **THEN** the ledger does not mark the third as uncited
- **AND** the closing line reads `coverage: 2/3 (1 ignored)`

### Requirement: Configuration lives beside the specs

The lint MUST read `openspec/reviewer.toml`. It holds the source roots,
source globs, skipped directories, path prefixes, path extensions, test
pattern, call helper name, change scopes, grandfathered changes, ignored
terms and ignored uncited requirements. Without the file the lint MUST
refuse to run, with an error naming the path, naming
`openspec-reviewer lint init` as the way to create it, and showing a
minimal example of the file. A malformed file is an error naming the key.
The file `lint init` writes MUST show both ignore sections as commented
examples.

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

#### Scenario: Init documents the ignore sections

- **WHEN** `openspec-reviewer lint init` writes the file
- **THEN** it holds a commented `[[definitions.ignore]]` example
- **AND** a commented `[[lint.ignore_uncited]]` example

### Requirement: The lint is a subcommand with a summary

`openspec-reviewer lint` MUST print notes to stdout, errors to stderr,
then one summary line with the counts of specs, changes, paths, tests,
hashes and citations checked and the number of errors. `--format json`
prints the findings and counts as one document instead. Exit status
follows the worst finding as in the review.

#### Scenario: Clean repository

- **GIVEN** a repository where every citation resolves
- **WHEN** the user runs `openspec-reviewer lint`
- **THEN** the summary line reports zero dangling
- **AND** the exit status is `0`

#### Scenario: One dangling citation

- **GIVEN** one citation that does not resolve
- **WHEN** the user runs `openspec-reviewer lint`
- **THEN** stderr has one error line
- **AND** the exit status is `2`

### Requirement: Citation findings join the review

When the reviewer shows a change, the lint's results for that change
MUST appear as findings on the matching pairings: a dangling citation in
a delta is an error on the pairing whose text holds it, a removed
requirement with outside citers is an error on that pairing, and a
modified requirement with outside citers is a note on that pairing. The
detail pane lists the citing files under the finding.

#### Scenario: Modified requirement with citers

- **GIVEN** a change under review that modifies `C § R`
- **AND** three test files citing `C § R`
- **WHEN** the reviewer selects that pairing
- **THEN** the detail pane shows a note
- **AND** lists the three files

#### Scenario: Removed requirement still cited

- **GIVEN** a change under review that removes `C § R`
- **AND** a test citing it
- **WHEN** the review is built
- **THEN** the pairing has an error finding
- **AND** the list row shows `!`

### Requirement: `lint init` writes a starting configuration

`openspec-reviewer lint init` MUST create `openspec/reviewer.toml` with
`source_roots` set to the directories that exist in the working directory
among `src`, `lib`, `apps`, `packages`, `crates`, `services`, `tests` and
`test`; `path_extensions` set to the extensions found in files under those
roots, drawn from `rs`, `ts`, `tsx`, `js`, `mjs`, `mts`, `py`, `go`,
`ex`, `exs`, `heex`, `md`, `json`, `yaml`, `yml`, `toml` and `css`; `source_globs` set to the
same extensions except `json`, which has no comment syntax and so no
place for a citation;
`skip_dirs` set to `node_modules`, `target`, `dist`, `build`, `_build`
and `deps`;
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

#### Scenario: Elixir tests are scanned

- **GIVEN** a repository with `openspec/` and `test/keyring_test.exs`
- **WHEN** the user runs `openspec-reviewer lint init`
- **THEN** `source_globs` contains `**/*.exs`

#### Scenario: JSON is cited but not scanned

- **GIVEN** a repository with `openspec/`, `src/a.rs` and `src/fixtures/b.json`
- **WHEN** the user runs `openspec-reviewer lint init`
- **THEN** `path_extensions` contains `json`
- **AND** `source_globs` does not contain `**/*.json`

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

### Requirement: An ignore entry names one finding and its reason

`openspec/reviewer.toml` MUST accept three arrays of tables.
`[[definitions.ignore]]` entries have `term`, an optional `in`, and
`reason`. `[[lint.ignore_uncited]]` entries have `requirement`, written
`<capability> § <requirement name>`, and `reason`.
`[[lint.ignore_evidence]]` entries have exactly one of `citation`,
written `<capability> § <requirement name>`, `path`, `test` or `commit`,
and `reason`; neither `lint` nor a review MUST report the missing
evidence that entry names. Every value is an exact string; the tool MUST NOT read any of them
as a glob or a pattern. An entry missing `reason` MUST be a configuration
error naming the entry. An `[[lint.ignore_evidence]]` entry naming no
kind, or more than one, MUST be a configuration error naming the entry.

#### Scenario: Ignored term

- **GIVEN** a config with a `[[definitions.ignore]]` entry for `steward`
- **WHEN** the lint runs
- **THEN** it reports no undefined-term finding for `steward`

#### Scenario: Entry without a reason

- **GIVEN** a `[[definitions.ignore]]` entry with only `term`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

#### Scenario: Entry is not a pattern

- **GIVEN** a `[[definitions.ignore]]` entry for `steward`
- **AND** an undefined term `stewardship`
- **WHEN** the lint runs
- **THEN** it reports the finding for `stewardship`

#### Scenario: Ignored example citation

- **GIVEN** a spec quoting `spec:alpha § Some rule`
- **AND** a `[[lint.ignore_evidence]]` entry with
  `citation = "alpha § Some rule"`
- **WHEN** the lint runs
- **THEN** it reports no dangling citation for `alpha § Some rule`

#### Scenario: Ignored example path

- **GIVEN** a spec citing the path `src/a.rs`, which does not exist
- **AND** a `[[lint.ignore_evidence]]` entry with `path = "src/a.rs"`
- **WHEN** the lint runs
- **THEN** it reports no missing path for `src/a.rs`

#### Scenario: Ignored citation during a review

- **GIVEN** a delta quoting `spec:alpha § Some rule`
- **AND** a `[[lint.ignore_evidence]]` entry with
  `citation = "alpha § Some rule"`
- **WHEN** the tool reviews that change
- **THEN** it reports no dangling citation on that pairing

#### Scenario: Evidence entry names no kind

- **GIVEN** a `[[lint.ignore_evidence]]` entry with only `reason`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

#### Scenario: Evidence entry names two kinds

- **GIVEN** a `[[lint.ignore_evidence]]` entry with `path` and `commit`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

### Requirement: An ignore entry can be confined to scopes

A `[[definitions.ignore]]` entry MAY carry `in`, an array of scopes. Each
scope is a capability name or a `<capability> § <requirement name>`. The
entry MUST apply only to text inside those scopes. An entry without `in`
MUST apply everywhere. A bare string in `in` MUST be a configuration
error naming the entry.

#### Scenario: Scoped to one capability

- **GIVEN** an ignore for `reason` with `in = ["citations"]`
- **AND** the term recurring in `citations` and in `glossary`
- **WHEN** the lint runs
- **THEN** it reports no finding in `citations`
- **AND** it reports the finding in `glossary`

#### Scenario: Scoped to two capabilities

- **GIVEN** an ignore for `reason` with `in = ["citations", "glossary"]`
- **AND** the term recurring in both
- **WHEN** the lint runs
- **THEN** it reports no finding for `reason`

#### Scenario: Scoped to one requirement

- **GIVEN** an ignore with `in = ["glossary § A term lists the words that are acceptable for it"]`
- **AND** the term appearing in another requirement of `glossary`
- **WHEN** the lint runs
- **THEN** it reports the finding for that other requirement

#### Scenario: Scope is a bare string

- **GIVEN** an ignore entry with `in = "citations"`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that entry

### Requirement: A dangling ignore entry is a warning

An ignore entry is dangling when a scope in its `in` does not resolve
against the register, when its `requirement` does not resolve against the
register, when its `term` is a glossary term, an admitted synonym or a
deprecated synonym, when the evidence it names is not missing, or when it
silenced no finding in the run. Each MUST produce a warning naming the
entry, the scope at fault and `openspec/reviewer.toml`, asking for it to
be removed. Scopes MUST be judged one at a time, so a live scope does not
mask a dead one. `lint` and `change` MUST reach the same verdict for the
same entry.

#### Scenario: Term now defined

- **GIVEN** an ignore entry for `steward`
- **AND** a glossary term `steward`
- **WHEN** the lint runs
- **THEN** it reports a warning naming `steward`
- **AND** naming `openspec/reviewer.toml`

#### Scenario: Ignored requirement is gone

- **GIVEN** an `ignore_uncited` entry naming a requirement the register
  does not hold
- **WHEN** the lint runs
- **THEN** it reports a warning naming that entry

#### Scenario: One scope of two is dead

- **GIVEN** an ignore with `in = ["citations", "glossary"]`
- **AND** the term appearing only in `citations`
- **WHEN** the lint runs
- **THEN** it reports a warning naming the `glossary` scope

#### Scenario: Unknown capability in a scope

- **GIVEN** an ignore with `in = ["sitemap-index"]`
- **AND** no such capability in the register
- **WHEN** the lint runs
- **THEN** it reports a warning naming that scope

#### Scenario: Same verdict during a review

- **GIVEN** an ignore entry the lint reports as dangling
- **WHEN** the tool reviews any open change
- **THEN** it reports the same warning

#### Scenario: Ignored path now exists

- **GIVEN** a `[[lint.ignore_evidence]]` entry with `path = "src/a.rs"`
- **AND** `src/a.rs` present in the repository
- **WHEN** the lint runs
- **THEN** it reports a warning naming that entry
- **AND** naming `openspec/reviewer.toml`

