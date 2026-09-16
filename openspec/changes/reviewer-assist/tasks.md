# Tasks: reviewer-assist

## 1. Abstraction

- [x] 1.1 `Assistant` trait, `Hint`, `HintKind`, `AssistError`.
- [x] 1.2 `[assist]` config: `agent`, `handoff_command`,
      `review_command`; unconfigured and missing-binary messages.
- [x] 1.3 Adapters for `claude`, `codex`, `opencode` and `custom`, each
      pinned by an argv-recording fake-binary test.

## 2. Prompts

- [ ] 2.1 Built-in `pairing.md`, `change.md`, `hints.md` templates
      embedded in the binary; the pairing template asks for the six
      judgment kinds and forbids restating known findings.
- [ ] 2.2 Prompt assembler over a pairing and its context; empty
      sections omitted; rules quoted from `openspec/config.yaml`.
- [ ] 2.3 Override loading from `openspec/reviewer/prompts/`;
      `assist prompts` subcommand writing defaults without overwriting.

## 3. Handoff

- [ ] 3.1 `i` on a requirement row and on a change heading: write prompt
      file under the state directory, suspend, run, resume with the
      selection kept.

## 4. Batch

- [ ] 4.1 `A` on a pairing, `Shift-A` on a change: worker thread,
      spinner and counter in the status line, sequential agent calls.
- [ ] 4.2 Reply parser to hints; `Unparsed` on malformed replies;
      stderr to the status line on non-zero exit.
- [ ] 4.3 Hint severity below note; `✦` row marker; detail pane section;
      exit status ignores hints.
- [ ] 4.4 `--assist` on plain runs; `--hints` to include hints in
      `--findings-only`.

## 5. State

- [ ] 5.1 Hint cache keyed by the normalized-after-text hash; re-run
      replaces.
- [ ] 5.2 Dismissal by kind and message with `x`; hidden in view and
      text, `dismissed: true` in JSON; survives re-runs.

## 6. Wrap-up

- [ ] 6.1 Fake-CLI fixtures: valid reply, malformed reply, non-zero
      exit.
- [ ] 6.2 Every requirement cited from at least one test title.
- [ ] 6.3 README section on configuring an agent and exporting prompts.
