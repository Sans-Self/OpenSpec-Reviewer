# Design: reviewer-assist

## Shape

```
src/
  assist/
    mod.rs        Assistant trait, Hint, AssistError
    prompt.rs     prompt assembly from Pairing + context, template loading
    claude.rs     `claude` adapter
    codex.rs      `codex` adapter
    opencode.rs   `opencode` adapter
    custom.rs     shell-template adapter
    prompts/      built-in templates, include_str!'d
  state/          gains hints + dismissals per pairing
  render/tui/     `i`, `A`, `Shift-A`, `x`; hint rendering
```

## The trait

```rust
pub trait Assistant {
    fn handoff(&self, prompt: &Path) -> Result<(), AssistError>;
    fn review(&self, prompt: &Path) -> Result<String, AssistError>;
}
```

`handoff` inherits the terminal: the TUI has already left the alternate
screen, the adapter spawns the CLI with stdin, stdout and stderr
attached, and waits. `review` captures stdout and returns it as text;
parsing into hints happens outside the adapter so every agent's raw
reply goes through one parser.

Per adapter the difference is flags only:

| Adapter | handoff | review |
| --- | --- | --- |
| `claude` | `claude "$(cat prompt)"` | `claude -p --output-format json` with the prompt on stdin, `.result` extracted |
| `codex` | `codex "$(cat prompt)"` | `codex exec --json` with the prompt on stdin |
| `opencode` | `opencode run` with the prompt | `opencode run --format json` |
| `custom` | `handoff_command` template | `review_command` template |

The exact flags are pinned in each adapter's test against a fake binary
on `PATH` that records its argv. When a CLI changes its flags, one
adapter and one test change.

## Prompt assembly

```
{template: pairing.md}
## Project rules
{openspec/config.yaml → rules.specs, verbatim}
## Pairing
kind, capability, name
### Before
### After
## Known findings
- severity  message
## Sibling texts
### {capability} § {requirement}
## Glossary
### {term}   Deprecated: …
## Reviewer note
## History
{hints.md, batch only}
```

The template owns the task text; the assembler owns the data sections.
Sections with no data are omitted rather than left as empty headings, so
the agent is never told "Glossary:" followed by nothing.

Quoting `rules.specs` verbatim closes a loop: the same sentences the
OpenSpec CLI shows to an agent that writes a proposal are the sentences
the reviewing agent grades against. A project changes its style in one
file.

## Hints

```rust
struct Hint {
    kind: HintKind,        // CompoundCondition, UncoveredMust, PlainLanguage,
                           // TermMisuse, SiblingInvalidated, RejectedApproach, Unparsed
    message: String,
    quote: Option<String>,
    scenario: Option<String>,
    dismissed: bool,
}
```

The kind list is closed and mirrors the six judgment tasks the default
prompt asks for, plus `Unparsed`. The hint schema in `hints.md` lists
the same kinds, so an agent replying with an unknown kind produces an
`Unparsed` hint rather than a silent drop. `Unparsed` is deliberately
visible: a prompt that stops working should look broken, not quiet.

Severity ordering becomes `error > warning > note > hint`. Exit status
computation stops at `note`.

## Cache and dismissal

The state file per change gains, per pairing:

```json
"hints": { "text_hash": 1234, "items": [ { …Hint } ] },
"dismissed": [ { "kind": "PlainLanguage", "message": "…" } ]
```

`text_hash` is the same normalized-after-text hash approvals use, so one
hash function serves both. Dismissals are keyed by kind and message
rather than by index so a re-run that returns the same hint in a
different position stays hidden. A dismissal survives a text change on
purpose: if the reviewer said "not this one", the same wording coming
back after an edit is still not wanted.

## Concurrency

Batch review of a whole change runs pairings sequentially on a worker
thread while the TUI shows a spinner and a counter in the status line.
Sequential because the agent CLIs are the bottleneck, not the tool, and
because parallel calls multiply the reviewer's bill. Blocking I/O
throughout, consistent with the foundation.

## Prompt overrides

`openspec/reviewer/` is the one store for agent-facing text: `prompts/`
here, `skills/` for the skill overrides `reviewer-skills` defines.
`openspec/reviewer/prompts/{pairing,change,hints}.md` override the
built-ins by name. `assist prompts` writes the built-ins there without
clobbering, printing which files it wrote and which it skipped. The
directory sits under `openspec/` so it travels with the specs and lands
in the same pull requests.

## Testing

Adapters are tested against fake CLIs on `PATH` that echo their argv and
return canned replies: a valid two-hint JSON, a malformed reply, a
non-zero exit. Prompt assembly is tested as a pure function over a
fixture pairing with and without glossary and drift data. Caching and
dismissal are tested through the state module with a temp XDG dir. No
test calls a real agent.
