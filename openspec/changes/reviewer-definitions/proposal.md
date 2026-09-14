# reviewer-definitions

## Why

A project's specs drift apart in vocabulary before they drift apart in
meaning. In rel-monorepo, *module* means a set of pages a customer buys
as one, switched on per website. For a year the same thing was a
*feature mount* in some specs and a *module* in others, and the pull
request that unified them was forty lines of restated requirements
whose only real change was the word. Nothing in the toolchain knew the
two words were the same thing, so nothing could have said so earlier.

A glossary fixes this only if it lives where the specs live, changes
the way specs change, and is checked the way citations are checked. A
wiki page does none of that.

## What changes

The reviewer treats one capability, `definitions`, as the project's
glossary. Each requirement in it is a term: the requirement name is the
word, the body is its meaning, the scenarios are usage examples, and a
`- **Deprecated:**` line lists the words the project does not use for it.
Because a term is an ordinary requirement, everything the reviewer
already does applies: terms are cited with `spec:definitions § module`,
renamed with RENAMED, tracked through history, and changed in the same
change as the specs that need the new meaning.

On top of that shape, four mechanical checks:

- A deprecated synonym used in any spec or delta is a warning.
- A term nobody uses is a note.
- A recurring identifier or quoted term with no definition is a note.
- A change that introduces a new term without defining it is a warning.

And one affordance: the detail pane can show the definitions of the
terms a pairing uses, so a reviewer reads *module* with its meaning one
key away.

## Capabilities

| Capability | Covers |
| --- | --- |
| `definitions` | The glossary shape, the four checks, the definitions panel, and configuration |

## Non-goals

- Flagging synonyms in source code or tests. That is a linter's job and
  needs the source roots; specs are the scope for now.
- Judging whether a spec uses a defined term against its meaning. That
  is a reading task and belongs to `reviewer-assist`.
- More than one glossary capability. One `definitions` spec per project
  for now; splitting by domain is a later change if a project outgrows
  it.
