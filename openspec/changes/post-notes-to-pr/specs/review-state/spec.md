# review-state (delta)

## ADDED Requirements

### Requirement: A note remembers where it was posted

A note MUST be able to record when it was posted and the URL of the
review it was posted in. A note with no record is unposted. Saving a
new text for a note MUST produce an unposted note. A state file written
before this record MUST load with every note unposted. The detail pane
MUST show the word posted and the date under a posted note.

#### Scenario: Posted and reopened

- **GIVEN** a posted note with a URL
- **WHEN** the tool runs again on the same change
- **THEN** the note is still posted
- **AND** the detail pane shows the word posted with the date under it

#### Scenario: Edited after posting

- **GIVEN** a posted note
- **WHEN** the reviewer saves a new text for it
- **THEN** the note is unposted

#### Scenario: Older state file

- **GIVEN** a state file whose notes carry no posted record
- **WHEN** the tool reads it
- **THEN** every note loads as unposted
