# definitions (delta)

## ADDED Requirements

### Requirement: anchor

A spec MUST use `anchor` to mean:

The thing a note or a finding hangs on: a requirement, one of its
scenarios, or an artefact. A note records the hash of its anchor's text
so it can say when that text has changed.

- **Deprecated:** target, subject

#### Scenario: In a sentence

- **WHEN** a note is written on a scenario row
- **THEN** that scenario is the note's anchor

### Requirement: posted

A spec MUST use `posted` to mean:

The state of a note that stands as a comment in a pull request review.
A posted note records when it was posted and the review's URL; a note
with no such record is unposted, and saving a new text makes a note
unposted again.

- **Deprecated:** published, submitted, synced

#### Scenario: In a sentence

- **WHEN** the reviewer quits with an unposted note on a pull request
  review
- **THEN** the view offers to post it
