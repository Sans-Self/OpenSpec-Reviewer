## Why

Two checks produce findings a project cannot act on. "Recurring term
without definition" and "new term without definition" fire on example
vocabulary invented inside scenarios and on configuration keys quoted in
a spec about configuration, none of which will ever have a definition.
The coverage ledger marks requirements uncited that no test can
reasonably cite, such as the ledger's own output format. With no way to
record that a finding was read and dismissed, a project either runs with
permanent noise or stops reading the output. Both defeat the exit status.

An exemption list only stays honest if the tool can say when an entry has
outlived its reason, and that answer must not depend on which subcommand
was typed. Today every consumer assembles its own view of which
requirements exist: `build_review` loads canon and every open change,
the coverage ledger rebuilds the same set inline, and the undefined-term
checks see only the pairings handed to them. Naming that set once is what
makes a dangling ignore decidable.

## What Changes

- The **register** is every requirement the repository asserts: canon
  plus every open change, each as `<capability> § <requirement>`, ordered
  by capability then requirement name. It is built once per run. `lint`
  and `change` hold the same register and differ only in what they show.
- The coverage ledger and the undefined-term checks read the register
  instead of assembling their own set.
- `openspec/reviewer.toml` gains `[[definitions.ignore]]` and
  `[[lint.ignore_uncited]]`. Each entry names an exact string and carries
  a `reason`. No globs, no patterns on the thing being ignored.
- A `[[definitions.ignore]]` entry may carry `in`, an array of scopes,
  each a capability or a `<capability> § <requirement>`. The entry
  applies only inside those scopes. Without `in` it is repo-wide.
- An ignore is dangling when a scope does not resolve against the
  register, when the term is already a glossary term or synonym, or when
  it silenced nothing. Each is a warning naming the entry, the scope and
  `openspec/reviewer.toml`. Scopes are judged one by one, so an array
  hides nothing that separate entries would have shown.
- An ignored requirement is still counted. The ledger's closing line
  reads `coverage: <cited>/<total> (<n> ignored)`.
- An entry with no `reason` is a configuration error.

Not in this change: ignores for any other finding kind. Deprecated
synonyms, sibling term drift, unresolved citations and the structural
errors are actionable by construction, and a project that wants them
silenced wants a different rule, not an exemption.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `change-model`: the register is every requirement canon and the open
  changes assert, in a stable order.
- `citations`: the configuration holds the two ignore lists; the
  coverage ledger reads the register, honours `ignore_uncited` and
  reports the ignored count; a dangling entry is a warning.
- `glossary`: both undefined-term checks skip an ignored term within its
  scopes.

## Impact

- `src/model/`: the register, built from canon plus the open changes
  `build_review` and the lint already load.
- `src/citations/config.rs`: two arrays of tables; `render` gains
  commented examples so `lint init` documents them.
- `src/citations/lint.rs`: the ledger filters and counts against the
  register.
- `src/citations/structure.rs`: the stale-entry check generalizes from
  `grandfathered` to any configured list of names.
- `src/glossary/checks.rs`: `recurring_undefined` and `new_undefined`
  take the ignore list and the scope of the text they are checking.
- `tests/fixtures/`: a config with one entry of each kind, one scoped to
  two capabilities, one dangling.

## Sequencing

This change modifies the two undefined-term requirements that
`glossary-admitted-terms` also modifies. Its delta text is written
against that change's version, so it applies after it.
