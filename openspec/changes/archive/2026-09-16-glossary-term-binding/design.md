## Context

The `definitions` capability stores terms as requirements: the name is
the term, the body its meaning, the scenarios usage examples. That choice
is what makes a changed meaning a pairing with a diff, a `term in use`
finding listing every requirement affected, and a review like any other.

OpenSpec's validator does not know about it. Every ADDED or MODIFIED
requirement must hold SHALL or MUST in its body, checked in
`dist/core/validation/validator.js` with no capability filter, no project
configuration and no opt-out. This repository's terms were written
directly into canon and so were never validated. Any project following
the `skills` spec drafts terms into a change, and every one of those
changes fails.

## Goals / Non-Goals

**Goals:**

- A drafted term validates.
- The panel, the JSON and the diff show the meaning, not the boilerplate.
- One shape for every term, enforced.

**Non-Goals:**

- Moving the glossary out of the spec model.
- Patching or vendoring the OpenSpec validator.
- Making the line optional.

## Decisions

### The binding line says what a glossary entry asserts

```markdown
### Requirement: register

A spec MUST use `register` to mean:

Every requirement the repository asserts: canon plus every open change,
each as `<capability> § <requirement name>`, ordered by capability then
requirement name.

- **Deprecated:** index, roll
```

The obvious move is a throwaway sentence carrying the keyword. This is
not that. A glossary entry does assert something normative — that one
word has one meaning across every spec — and that assertion has simply
been implicit in the file's shape until now. Writing it down satisfies
the validator because the requirement genuinely is a requirement, not
because a keyword was smuggled in.

It also gives the term's name a second, machine-checkable appearance,
which is what lets the parser verify the line belongs to this term rather
than accepting any sentence with a MUST in it.

### The parser strips it

The binding line joins `- **Deprecated:**` and `- **Admitted:**` as a
recognized line lifted out of the body. `meaning` stays what the panel
and the JSON show; the full body, binding line included, stays what
pairings diff, so editing the line is a visible change to the term.

### A missing line is a warning, not an error

The tool reports it; `openspec validate` is the thing that refuses. Two
tools erroring on the same condition means a project with a legacy
glossary cannot run the reviewer to find out what else is wrong.

### The line is required, not optional

An optional convention produces two shapes of term and a parser that
must handle both forever. The five terms in this repository are edited by
this change, and the warning covers everyone else's.

## Risks / Trade-offs

- **A line of boilerplate per term.** → It is one line, it is checkable,
  and it states something true that the file previously left implicit.
- **OpenSpec relaxes the rule later.** → The binding line stays valid
  under any weaker rule; nothing has to be unwound.
- **The term name in the line drifts from the requirement name.** → The
  parser compares them and the mismatch is the missing-line warning.

## Open Questions

None.
