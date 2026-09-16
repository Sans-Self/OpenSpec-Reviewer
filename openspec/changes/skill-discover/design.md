## Context

The glossary checks work on spans because a span is unambiguous: what
sits between the backticks is the candidate, whole. Prose has no such
marker. Finding "chain head" in "the chain head a rotation retires" is a
reading task, and `reviewer-assist` established that reading tasks go
to an agent while the tool keeps the mechanical part.

## Decisions

### A skill, not a check

The skill is the version of this that exists today. It costs a markdown
file and a registry entry, it can be replaced per project through
`openspec/reviewer/skills/`, and running it on a large repository is
how the shape of a later mechanical check gets learned. Requirement
names are the first source it reads because a requirement title is the
author naming the nouns that matter, and one that recurs across
capabilities is a shared contract.

### Cross-capability recurrence, not frequency

A spec repeats its own subject constantly, so recurrence inside one
capability is noise. A phrase that appears in two or more capabilities
is the same signal the span check approximates: two authors relying on
one word meaning one thing. The skill drops single-capability phrases
before it does anything else.

### It hands off to define

`define` already knows how to sort a concept from a field name, draft
the binding line, sort synonyms onto the admitted and deprecated lines
and lift a scenario. Teaching `discover` to draft would be a second copy
of those rules. The skill produces a candidate list in the shape
`define` reads, term plus the `capability § requirement` uses, and
`define` takes it from step 3 onward.

## Risks / Trade-offs

- **The list is long on a big repository.** → The skill caps candidates
  at phrases seen in two or more capabilities and asks the user for a
  batch size before handing off; `define` drafts in changes of that
  size.
- **An agent's phrase extraction is not reproducible.** → The output is
  a draft under `openspec/changes/` reviewed like any other. The
  mechanical check, when it exists, makes the candidate list a property
  of the tool.
