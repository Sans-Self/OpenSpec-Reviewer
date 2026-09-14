# reviewer-assist

## Why

The reviewer's mechanical checks stop where judgment starts. They can
say a scenario has a WHEN and a THEN; they cannot say the WHEN hides two
events. They can list every requirement that uses *module*; they cannot
say one of them uses it to mean a code module. They can find a sibling
that still says *feature mount*; they cannot say an archived design
already rejected the approach a new change takes. Those are reading
tasks, and a reader is one shell command away.

openclaw shows the shape: when it errors, it offers to open a Claude,
Codex or opencode session on the error with the context already loaded.
The reviewer can do the same on a pairing, and go one step further by
asking the agent for structured suggestions the reviewer can list, jump
to and dismiss.

## What changes

An `Assistant` abstraction with one adapter per agent CLI, and two ways
to use it from the review.

- **Handoff.** On a pairing, one key suspends the interactive view,
  writes a prompt file holding the pairing, its findings, drift hits,
  sibling texts, glossary terms and the project's spec rules, and opens
  the configured agent interactively on it. The view resumes when the
  agent exits.
- **Batch review.** One key runs the agent non-interactively on a
  pairing or on the whole change and turns its JSON reply into hints:
  a fourth severity that never affects the exit status, shown with its
  own glyph, cached by text hash, and dismissable.
- **Prompts** ship in the binary and can be overridden per project
  under `openspec/reviewer/prompts/`. The default prompt quotes the
  project's `openspec/config.yaml` rules, so the reviewing agent checks
  the same style the proposing agent was told to follow.

The agent never approves anything. Hints inform; the human decides.

## Capabilities

| Capability | Covers |
| --- | --- |
| `assist` | The assistant abstraction and adapters, the handoff, batch hints, caching and dismissal, prompt overrides, and configuration |

## Non-goals

- Talking to a model API directly. The tool shells out to agent CLIs
  the reviewer already has and pays for; no API keys live in the tool.
- Running the agent unprompted. Every agent call follows a key press or
  an explicit flag; nothing runs on open.
- Letting hints change the exit status. CI stays deterministic.
- Auto-applying agent suggestions to spec files. The tool reads; the
  editor writes.
