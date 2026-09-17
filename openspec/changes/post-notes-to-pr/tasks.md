# Tasks: post-notes-to-pr

## 1. The pull request on the snapshot

- [x] 1.1 `source::PullRequest { number, url, head }` and
      `Snapshot.pull_request: Option<PullRequest>`; `Review` carries it
      and keeps the snapshot's `FileChange`s.
- [x] 1.2 `GhSource::fetch` runs `gh pr view <pr> --json
      number,url,headRefOid` before `gh pr diff` and fills the field; a
      failing `gh pr view` is the same error as a failing diff.

## 2. The state

- [x] 2.1 `state::Posted { at, url }` and `Note.posted`, serialized
      only when present; older files load with `None`.
- [x] 2.2 `Store::mark_posted(keys, url)` stamps the given notes.

## 3. The planner

- [x] 3.1 `review::post::plan(review, stores) -> Option<ReviewPost>`:
      the unposted notes, each located to a path and line from the
      after texts where a heading is found, the rest in the body;
      `None` when there is nothing to post.
- [x] 3.2 `ReviewPost::payload(head) -> serde_json::Value` in GitHub's
      shape, and `ReviewPost::into_body_only()` for the retry.

## 4. Posting

- [x] 4.1 `source::gh::post_review(root, pr, payload) -> Result<String,
      SourceError>` running `gh api repos/<owner>/<repo>/pulls/<n>/reviews
      --input -` and returning the review's `html_url`; a `422` triggers
      the body-only retry once.

## 5. The view

- [x] 5.1 `Modal::Quit` with the count and the pull request number;
      `y`, `n`, `Esc` and `Ctrl-C` as designed; the quit keys open it
      only when the review has a pull request and unposted notes exist.
- [x] 5.2 `Effect::PostNotes` handled in the event loop: post, stamp
      the notes, quit; or set the status message and stay.
- [x] 5.3 The detail pane shows `posted <date>` under a posted note.

## 6. Tests and docs

- [ ] 6.1 Tests titled by requirement with a stub `gh` on `PATH` that
      records the request it received: the prompt appears and does not
      appear, the payload anchors a requirement, a scenario and an
      artefact note, the body-only retry, the failure message, and the
      posted stamp round-trips through the state file.
- [ ] 6.2 README: the quit prompt under the `gh` source.
- [ ] 6.3 `clippy` and `rustfmt` clean.
