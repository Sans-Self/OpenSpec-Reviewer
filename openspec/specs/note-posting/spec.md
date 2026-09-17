# note-posting Specification

## Purpose
TBD - created by archiving change post-notes-to-pr. Update Purpose after archive.
## Requirements
### Requirement: Quitting a pull request review offers to post the notes

When the review came from a pull request and at least one note is
unposted, every quit key in the browsing view, `q`, `Esc` and `Ctrl-C`,
MUST open a prompt naming the number of unposted notes and the pull
request number instead of quitting. In the prompt, `y` MUST post the
notes and quit, `n` MUST quit without posting, escape MUST return to
the review, and the interrupt key MUST quit without posting. With no
pull request or no unposted note, the quit keys MUST quit as before.

#### Scenario: Unposted notes on a pull request

- **GIVEN** a review opened through `gh 224`
- **AND** two unposted notes
- **WHEN** the user presses a quit key
- **THEN** the view shows a prompt naming 2 notes and pull request 224
- **AND** the view has not quit

#### Scenario: Decline

- **GIVEN** the prompt open
- **WHEN** the user declines
- **THEN** the view quits
- **AND** nothing is posted

#### Scenario: Back to the review

- **GIVEN** the prompt open
- **WHEN** the user presses escape
- **THEN** the prompt closes
- **AND** the view is browsing again

#### Scenario: Not a pull request

- **GIVEN** a review opened through the change source
- **AND** an unposted note
- **WHEN** the user presses a quit key
- **THEN** the view quits

#### Scenario: Everything already posted

- **GIVEN** a pull request review
- **AND** every note posted
- **WHEN** the user presses a quit key
- **THEN** the view quits

### Requirement: Notes post as one review of comments

Posting MUST submit one review to the pull request with the comment
event and the head commit the source resolved. Each unposted note MUST
be one review comment on the line of its anchor: the requirement
heading in `openspec/changes/<change>/specs/<capability>/spec.md` for a
requirement note, the scenario heading under that requirement for a
scenario note, line 1 of `openspec/changes/<change>/<artefact>` for an
artefact note. An outdated note MUST end with the line the detail pane
shows for an outdated note. A note whose anchor is not found in the
file MUST go into the review body under a heading naming the anchor as
plain output does, capability, section sign and requirement, or the
artefact's name. When every note is a comment, the body MUST be one
line naming the tool.

#### Scenario: Requirement and scenario notes

- **GIVEN** a note on a requirement
- **AND** a note on one of its scenarios
- **WHEN** the user posts
- **THEN** the review has two comments
- **AND** the first is on the requirement's heading line in the delta
  spec file of its capability
- **AND** the second is on the scenario's heading line of the same file

#### Scenario: Artefact note

- **GIVEN** a note on the artefact `design.md`
- **WHEN** the user posts
- **THEN** the review has a comment on line 1 of that artefact's file
  under the change

#### Scenario: Anchor not found

- **GIVEN** a note on a requirement whose heading is not in the head's
  delta file
- **WHEN** the user posts
- **THEN** the review body has a section headed with the requirement's
  capability and name
- **AND** the section holds the note's text

#### Scenario: Outdated note

- **GIVEN** a note whose anchor text changed since it was written
- **WHEN** the user posts
- **THEN** its comment ends with the outdated line

### Requirement: A rejected review is retried in the body

When GitHub rejects the review with status `422`, posting MUST submit
it once more with every comment moved into the body as sections and no
line comments. A second rejection is a failure.

#### Scenario: Line outside the diff

- **GIVEN** a comment on a line GitHub does not accept
- **WHEN** the user posts
- **THEN** a second review is sent with no comments
- **AND** its body holds every note as a section

### Requirement: A failed post keeps the view open

When posting fails, the view MUST stay open with gh's stderr in the
status line, and no note MUST become posted. When posting succeeds,
every sent note MUST become posted with the review's URL, and the view
MUST quit.

#### Scenario: gh fails

- **GIVEN** `gh api` exiting non-zero with a message
- **WHEN** the user posts
- **THEN** the status line shows the message
- **AND** the view has not quit
- **AND** no note is posted

#### Scenario: Posted

- **GIVEN** gh answering with a review URL
- **WHEN** the user posts
- **THEN** each sent note is posted with that URL
- **AND** the view quits

