# input-sources (delta)

## MODIFIED Requirements

### Requirement: The gh source reads a pull request

`gh <pr>` MUST run `gh pr view <pr> --json number,url,headRefOid` in
the working directory and record the pull request's number, URL and
head commit on the snapshot, then run `gh pr diff <pr>` and hand its
output to the diff source. When `gh` is not on the path or either
command exits non-zero, the tool MUST stop and show gh's stderr.

#### Scenario: Open pull request

- **GIVEN** a checkout where `gh` is authenticated
- **WHEN** the user runs `openspec-reviewer gh 224`
- **THEN** the review equals piping `gh pr diff 224` into `diff -`
- **AND** the review knows pull request 224, its URL and its head commit

#### Scenario: gh missing

- **GIVEN** no `gh` on the path
- **WHEN** the user runs the `gh` subcommand
- **THEN** the tool stops with an error saying the `gh` source needs the
  `gh` CLI
