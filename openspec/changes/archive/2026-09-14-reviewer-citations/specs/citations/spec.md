# citations (delta)

## ADDED Requirements

### Requirement: A citation names a capability and a requirement

The lint MUST read `spec:<capability> § <requirement name>` inside a
backtick span or a double-quoted string as a citation. The capability is lowercase
letters, digits and hyphens. The requirement name runs to the closing
delimiter, so an apostrophe does not end it. Runs of whitespace in the
name compare as one space. When `openspec/reviewer.toml` names a call
helper, `<helper>("<capability>", "<requirement>")` with either quote
style is also a citation.

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

### Requirement: A citation must resolve

Every citation MUST name a capability that exists in canon or that an
open change adds to, and a requirement that exists in that capability's
canon or is listed under ADDED in an open change. Otherwise the lint
reports an error naming the citing file and the citation.

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

With `--coverage`, the lint MUST print, per capability, each canon or
in-flight requirement with the number of source files citing it, marking
those with zero, and a closing line `coverage: <cited>/<total>`. Spec-to-
spec citations do not count. The ledger never changes the exit status.

#### Scenario: One uncited requirement

- **GIVEN** a capability with three requirements
- **AND** tests citing two of them
- **WHEN** the lint runs with `--coverage`
- **THEN** the ledger marks the third as uncited
- **AND** the closing line reads `coverage: 2/3`

### Requirement: Configuration lives beside the specs

The lint MUST read `openspec/reviewer.toml`. It holds the source roots,
source globs, skipped directories, path prefixes, path extensions, test
pattern, call helper name, change scopes, and grandfathered changes.
Without the file the lint MUST refuse to run, with an error naming the
path and a minimal example of the file. A malformed file is an error
naming the key.

#### Scenario: No config

- **GIVEN** a repository without `openspec/reviewer.toml`
- **WHEN** the lint runs
- **THEN** it stops with an error naming that path
- **AND** the error shows a minimal example of the file
- **AND** the exit status is `2`

#### Scenario: Bad key

- **GIVEN** a config with `source_root = "apps"` instead of `source_roots`
- **WHEN** the lint runs
- **THEN** it stops with an error naming the unknown key

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
