## Context

`openspec/reviewer.toml` is the one place a repository tells the reviewer
about itself. `grandfathered` is the existing precedent for an exemption
list: a flat array of change names, with `src/citations/structure.rs`
reporting an entry whose directory is gone.

Two checks need the same treatment. Both undefined-term checks fire on
anything `Glossary::knows` does not account for, which includes words
invented as examples inside scenarios and configuration keys quoted in
prose. The coverage ledger marks every requirement with zero citing
source files, including requirements about output formats that a test
asserts without naming.

The obstacle is knowing when an entry has gone dead. Every consumer
currently builds its own view of which requirements exist.
`build_review` loads canon and every open change at `src/build.rs:47`,
then hands `new_undefined` only the pairings of the change under review.
The coverage ledger assembles "each canon or in-flight requirement"
inline. `unused_terms` takes `open_deltas` as a third argument. The data
is the same in every path; only the slice passed along differs.

## Goals / Non-Goals

**Goals:**

- One named, ordered set of requirements that every consumer reads.
- A project can record that a finding was read and dismissed, with the
  reason next to the entry.
- A dangling entry says so, and says the same thing in `lint` and in
  `change`.
- Ignoring a requirement never changes the coverage number.

**Non-Goals:**

- Ignores for other finding kinds.
- Patterns on the ignored term.
- A separate ignore file.

## Decisions

### The register is a domain concept, not a helper

The register is every requirement the repository asserts: canon plus
every open change, each as `<capability> § <requirement>`. It is built
once per run, before any check. `lint` and `change` hold the same
register; they differ in what they display, not in what they know.

This is not a refactor for tidiness. An ignore is dangling or not
depending on whether anything in the repository still matches it, and
with each consumer holding a different slice, that question gets
different answers from different subcommands. A `change` run would warn
about ignores that exist to silence a finding in some other change. The
register makes the answer a property of the repository.

The concept is already in the specs without a name — the coverage ledger
says "each canon or in-flight requirement" — so this names something the
tool already computes twice.

### One order, everywhere

Capability, then requirement name, both stable. `load_open_changes`
returns arrival order and `checks.rs` reaches for `BTreeMap` and
`BTreeSet` to recover determinism piecemeal. Ordering the register once
means the ledger, the dangling report and the JSON enumerate identically,
so a diff of two runs shows real change rather than iteration order.

### Every entry carries a reason

`grandfathered` is a flat array of names. Ignores are arrays of tables:

```toml
[[definitions.ignore]]
term   = "reason"
in     = ["citations", "glossary"]
reason = "TOML key in the config specs, not a concept"

[[lint.ignore_uncited]]
requirement = "citations § Coverage lists citing tests per requirement"
reason      = "asserted by the ledger snapshot, which cannot cite itself"
```

A missing `reason` is a configuration error. The reason is what makes the
list auditable a year later, and an optional field would be absent from
every entry within a month.

### `in` is always an array, and scopes are judged one by one

`source_roots`, `path_prefixes` and `skip_dirs` are arrays in that file
even when they hold one element, so `in = ["citations"]` is the shape the
reader already has. Accepting a bare string too would buy a nicer single
case and cost a second parse path in a file whose errors work by naming
keys.

One term often has one justification across several capabilities.
Splitting that into two entries duplicates the reason, and the copies
drift the first time somebody sharpens the wording.

The cost is that an array could hide a dead scope behind a live one, so
staleness is per element: the warning names the entry and the scope that
matched nothing. An array then hides nothing separate entries would have
shown.

### Exact strings on the term, scopes on the location

A glob on the term silences findings nobody has read. A scope narrows an
ignore so it cannot drift into a capability nobody considered, which is
the opposite risk. So the term is exact and `in` is exact, and there are
no patterns anywhere — a scope that does not resolve against the register
is dangling rather than matching nothing quietly.

### A dangling entry is a warning, not an error

`structure.rs` reports a stale `grandfathered` entry as an error, because
a missing change directory means the configuration disagrees with the
repository. A dangling ignore usually means the opposite: the noise it
covered is gone, which is good news. Good news should not set the exit
status to 2.

### The coverage number does not move

The ledger counts an ignored requirement in `<total>` and keeps it out of
the marked lines, closing with `coverage: <cited>/<total> (<n> ignored)`.
A coverage figure that improves when somebody edits a config is a figure
that lies, and the ledger's only job is to be read.

## Risks / Trade-offs

- **An ignore hides a term that later deserves a definition.** → Writing
  the definition makes the entry redundant, which the dangling check
  reports, so the ignore is removed by the same work that obsoletes it.
- **The register is built even for a small review.** → It is the set the
  review already loads; the change is that it is built once and ordered
  rather than sliced per consumer.
- **Two open changes modify the same two requirements.** → This change's
  delta text is written against `glossary-admitted-terms`, and the
  proposal records the order.

## Open Questions

None.
