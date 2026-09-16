## Why

A glossary term cannot be added through a change. `openspec validate`
errors on any ADDED or MODIFIED requirement whose body holds no SHALL or
MUST, and a definition is not a normative rule, so every drafted term
fails:

```
✗ definitions/spec.md: ADDED "register" must contain SHALL or MUST
```

The check is unconditional. `dist/core/validation/validator.js` reads no
project configuration, and `openspec/config.yaml` accepts only `schema`,
`context` and `rules`, where `rules` is prompt text keyed by artefact and
never reaches a validator. There is no flag; `--strict` only adds checks.

This repository's own terms escaped by being written straight into
`openspec/specs/definitions/spec.md` as canon. Every other project has to
go through a change, and the `skills` spec requires it: the define skill
MUST draft terms under `## ADDED Requirements` in
`openspec/changes/<name>/specs/definitions/spec.md`. As it stands that
skill emits changes that do not validate.

## What Changes

- A term's body MUST open with a binding line: `A spec MUST use
  <term> to mean:` followed by the meaning. The line carries the
  keyword the validator wants and states what a glossary entry actually
  asserts — that this word has one meaning across the specs.
- The parser strips the binding line from the meaning text, as it already
  strips `- **Deprecated:**` and `- **Admitted:**`. The panel and the
  JSON `definitions` array are unchanged.
- A term whose body has no binding line is a warning naming the term, so
  the glossary does not silently split into two shapes.
- The five terms in `openspec/specs/definitions/spec.md` gain the line.
- `/opsx-reviewer:define` drafts the line.

Not in this change: moving the glossary out of the spec model. Terms are
requirements so that a changed meaning is a pairing with a diff and a
`term in use` finding. A separate file would validate cleanly and lose
all of that.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `glossary`: a term's body opens with a binding line, stripped from the
  meaning; a missing line is a warning.
- `definitions`: the five existing terms gain the line.
- `skills`: the define skill drafts the line.

## Impact

- `src/glossary/synonyms.rs`: the marker parser also recognizes the
  binding line.
- `src/glossary/mod.rs`: `Term::from_requirement` reports a term with no
  binding line.
- `src/glossary/checks.rs`: the warning.
- `src/skills/define.md`: the drafting guidance.
- `openspec/specs/definitions/spec.md`: five terms.
- `tests/fixtures/`: the fixture glossary gains the line, and one term
  without it.

## Sequencing

`reviewer-ignores` adds the term `register` and had its `definitions`
delta removed because of this validator. That delta can return once this
change lands.
