# post-notes-to-pr

## Why

A review of a pull request through `gh <pr>` ends with the notes in a
local state file and nothing on the pull request. The reviewer then
retypes them into GitHub, requirement by requirement, or does not. The
tool knows the pull request, the change, the requirement and the
scenario each note hangs on, and where in the delta file that heading
stands, which is everything a review comment needs.

## What Changes

- When the view quits and the review came from a pull request, and at
  least one note has not been posted, the view asks before leaving:
  post the notes to the pull request, quit without posting, or stay.
- Posting submits one pull request review with the event `COMMENT`.
  Each note becomes a review comment on the line of its anchor in the
  pull request's head: the `### Requirement:` heading of a requirement
  note, the `#### Scenario:` heading of a scenario note, the first line
  of an artefact note's file. A note whose anchor line is not in the
  pull request's diff goes into the review body under a heading naming
  its requirement or file. An outdated note carries the line "text
  changed since the note was written".
- A posted note remembers when and to which review it was posted. Only
  unposted notes are offered; editing a posted note makes it unposted.
- The `gh` source resolves the pull request first, through
  `gh pr view <pr> --json number,url,headRefOid`, and the snapshot
  carries it, so posting knows the repository, the number and the head
  commit without a second guess.
- A failed post leaves the view open with gh's stderr in the status
  line. The notes stay local either way.

## Capabilities

### New Capabilities

- `note-posting`: the quit prompt, what a posted review contains, how a
  note is anchored, and how a failure is shown.

### Modified Capabilities

- `input-sources`: "The gh source reads a pull request" also resolves
  the pull request's number, URL and head commit.
- `review-state`: a new requirement, "A note remembers where it was
  posted".
- `definitions`: the terms `anchor` and `posted`.

## Impact

- `src/source/gh.rs`: `gh pr view` before `gh pr diff`; a `post_review`
  function running `gh api` with a JSON body on stdin.
- `src/source/mod.rs`: `Snapshot` and `Review` carry
  `Option<PullRequest>` and the after text of the changed files, which
  anchoring reads.
- New `src/review/post.rs`: the pure planner from notes to a review
  payload.
- `src/state/mod.rs`: `Note.posted`.
- `src/render/tui/`: the quit prompt as a modal, `Effect::PostNotes`.
- No new dependency; `serde_json` already builds the payload.

## Non-goals

- Posting approvals, or a review event other than `COMMENT`. An
  approval in this tool is the reviewer's own bookkeeping.
- A flag to post from plain output. The prompt is the view's.
- Editing or deleting a comment already posted.
- Posting from the `git`, `diff` or `change` sources. Only a pull
  request has somewhere to post to.
