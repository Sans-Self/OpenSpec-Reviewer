# definitions

## Purpose

The words this project uses, one requirement per term. The requirement
name is the term, the body opens with the line binding that word to one
meaning across every spec, the rest of the body is that meaning, the
scenarios show it in a sentence, and a `- **Deprecated:**` line lists the
words not to use for it. The reviewer reads this file as the glossary: it finds defined terms
and their synonyms anywhere in prose, and proposes new terms only from
what specs already mark with backticks or double quotes.
## Requirements
### Requirement: canon

A spec MUST use `canon` to mean:

The specs under `openspec/specs/`, as they are in the working directory.
Every review compares a change against canon.

- **Deprecated:** main specs, baseline

#### Scenario: In a sentence

- **WHEN** a delta modifies a requirement
- **THEN** the review pairs it with the same requirement in canon

### Requirement: snapshot

A spec MUST use `snapshot` to mean:

The files under `openspec/` a source yields, each with a before and an
after side. Every source produces one; everything downstream reads only
the snapshot and canon.

- **Deprecated:** patch set, changeset

#### Scenario: In a sentence

- **WHEN** the `gh` source fetches a pull request
- **THEN** it returns a snapshot of the files the pull request touches

### Requirement: source

A spec MUST use `source` to mean:

Where a change comes from: the working tree, a unified diff, two git
refs or a pull request. A source yields a snapshot.

- **Deprecated:** provider, adapter

#### Scenario: In a sentence

- **WHEN** the user runs `openspec-reviewer diff pr.patch`
- **THEN** the diff source reads the patch

### Requirement: pairing

A spec MUST use `pairing` to mean:

One delta entry matched with its canon counterpart: the requirement
before, the requirement after, their diff, and the findings about them.
A review is a list of pairings grouped by capability.

- **Deprecated:** pair, matching

#### Scenario: In a sentence

- **WHEN** a RENAMED entry names a requirement canon has
- **THEN** the pairing shows the old name struck through and the new one added

### Requirement: finding

A spec MUST use `finding` to mean:

One thing the review noticed about a pairing, with a fixed severity of
error, warning or note. Errors set the exit status to 2, warnings to 1.

- **Deprecated:** issue, diagnostic, violation

#### Scenario: In a sentence

- **WHEN** a MODIFIED requirement has no canon counterpart
- **THEN** the pairing carries an error finding

