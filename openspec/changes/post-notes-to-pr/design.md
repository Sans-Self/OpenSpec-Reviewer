# Design: post-notes-to-pr

## Context

Notes live in the state store keyed `<capability>/<requirement>` or
`<capability>/<requirement>#<scenario>`, and on artefacts by name.
The `gh` source is a thin wrapper that pipes `gh pr diff` into the diff
source and records `gh pr <n>` in a display string. The view quits on
`q`, `Esc` or `Ctrl-C` by setting a flag the event loop reads. Nothing
downstream of the source knows it was a pull request.

## Goals / Non-Goals

**Goals:** every note reaches the pull request, on its line when the
line is there; nothing is posted twice; a failed post loses nothing.

**Non-Goals:** approvals on GitHub; a headless posting flag; editing
what was posted.

## Decisions

**A typed pull request on the snapshot.** `Snapshot` gains
`pull_request: Option<PullRequest { number, url, head }>`, filled by
the `gh` source from `gh pr view <pr> --json number,url,headRefOid`
before it runs `gh pr diff`. The owner and repository come from the
URL, so a pull request given as a URL to another repository posts to
that repository, the one the diff came from. `Review` carries the same
field. The other sources leave it `None`, and `None` is what makes the
quit prompt not appear.

**One review, `COMMENT`.** A pull request review is the unit GitHub
offers for a batch of comments with one notification, and `COMMENT` is
the event that neither approves nor blocks. The payload is
`POST /repos/{owner}/{repo}/pulls/{n}/reviews` with `commit_id` the
head, `event` `COMMENT`, `body` the fallback text and `comments` the
anchored notes, sent through `gh api --input -` so authentication stays
gh's problem.

**Anchoring by heading.** A requirement note anchors to the line of
`### Requirement: <name>` in
`openspec/changes/<change>/specs/<capability>/spec.md` as the pull
request's head has it; a scenario note to its `#### Scenario: <name>`
line under that requirement; an artefact note to line 1 of
`openspec/changes/<change>/<artefact>`. The line numbers come from the
after text the snapshot already holds for each file, so `Review` keeps
the `FileChange`s. GitHub accepts a review comment only on a line the
pull request's diff touches or shows as context, and a heading in a
delta file that the pull request did not add or change may not be.
The planner cannot see the hunks after the diff source has applied
them, so it anchors every note it can locate and, when GitHub rejects
the review, retries once with every comment moved into the body. Two
requests in the worst case, no per-comment guessing.

**The body is the fallback, not a summary.** Notes with no anchor line
go into the body as `### <capability> § <requirement>` sections, or
`### <artefact>` for artefacts, followed by the note text. When every
note anchors, the body is one line naming the tool, because GitHub
rejects an empty body with no comments and a bare line is the cheapest
body that always passes.

**Posted is a record on the note.** `Note` gains
`posted: Option<Posted { at, url }>`, the time and the review's
`html_url`. Only notes with `posted` `None` are offered and sent.
`Store::set_note` builds a fresh `Note`, so editing a posted note
clears `posted` without a rule for it. A posted note reads as posted in
the detail pane: `posted <date>` after the note.

**The prompt is a modal.** Quitting with unposted notes on a pull
request review sets `Focus::Modal(Modal::Quit)` instead of the quit
flag. `y` returns `Effect::PostNotes`; `n` sets the quit flag; `Esc`
returns to browsing. The event loop handles `PostNotes` like
`EscalateNote`: it calls the source-side function, then either sets the
quit flag on success or writes gh's stderr to `app.message` and stays.
The planner is pure and lives in `review::post`; the two `gh` calls
live in `source::gh`.

## Risks / Trade-offs

- [GitHub rejects a line outside the diff] → the retry with everything
  in the body; the notes still land, less precisely.
- [The head moves between `gh pr view` and posting] → the review is
  pinned to the `commit_id` fetched at open; GitHub shows it as
  outdated on the new head, which is true.
- [A note posted then edited is posted again] → intended: the new text
  is a new note, and the old comment stays as history.
- [`Ctrl-C` while the prompt is open] → treated as `n`: quit without
  posting, since the reviewer asked twice.
