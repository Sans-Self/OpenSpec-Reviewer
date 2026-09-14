# assist (delta)

## ADDED Requirements

### Requirement: An assistant is an agent CLI behind one interface

The tool MUST expose every agent through one interface with two
operations: open an interactive session on a prompt file, and run a
prompt non-interactively returning text. Adapters ship for `claude`,
`codex` and `opencode`. Adding an agent MUST NOT touch code outside its
adapter and the configuration.

#### Scenario: Two agents, same handoff

- **GIVEN** a pairing under review
- **WHEN** the reviewer hands off with `claude` configured
- **AND** hands off with `codex` configured
- **THEN** both sessions receive the same prompt file

### Requirement: The agent is chosen in configuration

`assist.agent` in `openspec/reviewer.toml` MUST name the adapter, one of
`claude`, `codex`, `opencode` or `custom`. For `custom`,
`assist.handoff_command` and `assist.review_command` are shell templates
in which `{prompt}` is replaced by the prompt file path. Without an
`[assist]` section every assist key MUST show a message saying assist
is not configured, and the tool runs no agent.

#### Scenario: Not configured

- **GIVEN** a `reviewer.toml` without `[assist]`
- **WHEN** the reviewer presses an assist key
- **THEN** the status line says assist is not configured
- **AND** no process is started

#### Scenario: Custom agent

- **GIVEN** `assist.agent = "custom"`
- **AND** `assist.handoff_command = "aichat -f {prompt}"`
- **WHEN** the reviewer hands off
- **THEN** the tool runs that command with the prompt path substituted

#### Scenario: Agent binary missing

- **GIVEN** `assist.agent = "codex"`
- **AND** no `codex` on the path
- **WHEN** the reviewer presses an assist key
- **THEN** the status line says the `codex` CLI was not found

### Requirement: A prompt file carries the pairing's full context

For a pairing, the prompt file MUST contain, in this order: the task
text from the prompt template; the project's spec rules from
`openspec/config.yaml`; the pairing's kind, capability and name; the
before text and after text; every finding on the pairing with its
message; each drift hit with the sibling's text; each glossary term the
after text uses with its meaning and deprecated terms; the reviewer's
note when there is one; and the history summary. For a whole change the
file holds the proposal, then each pairing's section.

#### Scenario: Pairing prompt

- **GIVEN** a modified pairing with one warning
- **AND** one drift hit
- **AND** one glossary term in its text
- **WHEN** the tool builds the prompt
- **THEN** the file has the rules, both texts, the warning, the sibling's
  text and the term's meaning

#### Scenario: No glossary

- **GIVEN** a project without a `definitions` capability
- **WHEN** the tool builds the prompt
- **THEN** the glossary section is absent
- **AND** the rest is unchanged

### Requirement: Handoff opens the agent on the current pairing

Pressing `i` on a requirement row MUST write the prompt file, leave the
alternate screen, run the agent's interactive session on it, and resume
the view with the same selection when the session exits. The prompt
file is written under the state directory and named after the pairing.

#### Scenario: Open and return

- **GIVEN** the cursor on a pairing
- **WHEN** the reviewer presses `i`
- **AND** the agent session exits
- **THEN** the view is back
- **AND** the same row is selected

#### Scenario: Handoff on the change

- **GIVEN** the cursor on a change heading row
- **WHEN** the reviewer presses `i`
- **THEN** the prompt file holds the whole change

### Requirement: Batch review turns the agent's reply into hints

Pressing `A` on a requirement row MUST run the agent non-interactively
on the pairing's prompt with the hint schema appended, parse the reply
as a JSON array of hints, and attach each hint to the pairing. `Shift-A`
on a change heading does the same per pairing of the change. A hint has
a kind, a one-line message, an optional quote of the text it concerns,
and an optional scenario name. A reply that does not parse MUST produce
one hint of kind `unparsed` carrying the raw reply's first line.

#### Scenario: Two hints

- **GIVEN** an agent reply with two valid hints
- **WHEN** the batch run finishes
- **THEN** the pairing shows two hints
- **AND** each has its message

#### Scenario: Malformed reply

- **GIVEN** an agent reply that is not JSON
- **WHEN** the batch run finishes
- **THEN** the pairing shows one `unparsed` hint
- **AND** the hint quotes the reply's first line

#### Scenario: Agent exits non-zero

- **GIVEN** an agent that exits with an error
- **WHEN** the batch run finishes
- **THEN** the status line shows the agent's stderr
- **AND** no hint is attached

### Requirement: Hints are a severity that never affects the exit status

A hint MUST be a fourth severity below `note`. Rows with a hint show `✦`
after the finding markers. Hints appear in the detail pane under the
findings, in plain text and in JSON, and `--findings-only` includes them
only with `--hints`. The exit status MUST ignore hints.

#### Scenario: Only hints

- **GIVEN** a review whose only findings are hints
- **WHEN** the tool prints plain output
- **THEN** the exit status is `0`

#### Scenario: Findings-only

- **GIVEN** two hints and one warning
- **WHEN** the tool runs with `--findings-only`
- **THEN** the output has the warning
- **AND** no hint

### Requirement: Hints are cached by the text they were made for

Hints MUST be stored in the change's state file with the hash of the
pairing's normalized after text. On a later run the tool MUST show the
stored hints when the hash matches and drop them when it does not.
Pressing `A` on a pairing with cached hints MUST re-run the agent and
replace them.

#### Scenario: Text unchanged

- **GIVEN** a pairing with two cached hints
- **AND** unchanged after text
- **WHEN** the reviewer opens the change
- **THEN** the two hints show without running the agent

#### Scenario: Text changed

- **GIVEN** a pairing with cached hints
- **AND** after text that differs from the cached hash
- **WHEN** the reviewer opens the change
- **THEN** the pairing shows no hints

### Requirement: A hint can be dismissed

Pressing `x` with a hint selected in the detail pane MUST mark it
dismissed in the state file. Dismissed hints are hidden in the view and
in plain text, carry `dismissed: true` in JSON, and MUST stay dismissed
when a later batch run returns a hint with the same kind and message.

#### Scenario: Dismiss

- **GIVEN** a hint selected in the detail pane
- **WHEN** the reviewer presses `x`
- **THEN** the hint disappears from the pane
- **AND** the row's `✦` disappears when no hint remains

#### Scenario: Same hint returns

- **GIVEN** a dismissed hint
- **WHEN** a later batch run returns a hint with the same kind and
  message
- **THEN** it stays hidden

### Requirement: Prompt templates ship and can be overridden

The tool MUST ship a template for the pairing prompt, the change prompt
and the hint schema. A file at `openspec/reviewer/prompts/<name>.md`
MUST replace the built-in of the same name. `openspec-reviewer assist
prompts` MUST write the built-in templates to that directory without
overwriting existing files, so a project starts from the defaults.

#### Scenario: Override

- **GIVEN** `openspec/reviewer/prompts/pairing.md` exists
- **WHEN** the tool builds a pairing prompt
- **THEN** it uses that file's text as the task section

#### Scenario: Export defaults

- **GIVEN** an empty `openspec/reviewer/prompts/`
- **WHEN** the user runs `openspec-reviewer assist prompts`
- **THEN** the directory holds `pairing.md`, `change.md` and `hints.md`

#### Scenario: Export does not overwrite

- **GIVEN** an edited `openspec/reviewer/prompts/pairing.md`
- **WHEN** the user runs `openspec-reviewer assist prompts`
- **THEN** the edited file is unchanged

### Requirement: The default prompt asks for judgment, not repetition

The built-in pairing prompt MUST ask the agent for hints of these kinds
only: a keyword line that holds more than one condition; a MUST clause
no scenario exercises; a sentence that fails the project's plain
language rules; a glossary term used against its meaning; a sibling
requirement whose reasoning the change invalidates; and an archived
change that decided against the approach. It MUST tell the agent that
the mechanical findings listed in the prompt are already known and
must not be repeated.

#### Scenario: Known finding not repeated

- **GIVEN** a pairing with a mechanical "scenario dropped" warning
- **WHEN** the agent replies
- **THEN** no hint restates the dropped scenario

### Requirement: Batch review is available without the view

`--assist` on a plain-output run MUST perform the batch review for every
pairing before printing, honouring the cache, and include hints in text
and JSON. Without the flag plain output never runs an agent.

#### Scenario: Agent-driven review

- **WHEN** a script runs `openspec-reviewer change foo --format json --assist`
- **THEN** each pairing in the output has a `hints` array
- **AND** the agent ran only for pairings without a valid cache
