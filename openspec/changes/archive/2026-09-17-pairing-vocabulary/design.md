# Design: pairing-vocabulary

## Context

The glossary's job is to stop one concept being written two ways. A
deprecated synonym is matched as a whole word wherever it appears, so a
word with a strong life outside the concept produces findings that are
noise, and the only way to answer them is to change the word or change
the glossary.

## Goals / Non-Goals

**Goals:** no deprecated synonym in canon; a vocabulary a spec can be
written in without fighting it.

**Non-Goals:** a mechanism for dismissing a deprecated-synonym finding.
Where the word is wrong the prose changes, and where the glossary is
wrong the glossary changes.

## Decisions

**`matching` stops being deprecated.** Glob matching, pattern matching
and "the matching pairings" are not a delta entry beside its canon
counterpart, and the definition of `pairing` says "matched" itself. A
deprecation that fires on its own meaning is one word too wide.

**`pair` stays deprecated, and `match` is the verb.** The confusion the
deprecation exists for is a reader meeting "the pair" beside "the
pairing" and taking them for the same thing, which is likeliest in
exactly the requirements that trip it. So the verb becomes `match`,
which is the definition's own word for that act, and the noun becomes
whatever the two things are: two names, two lines.

**Two requirements are renamed.** `change-model § A rename is a pair of
names` and `semantic-diff § Every delta entry pairs with canon by name`
carry the word in their names, so a reworded body would leave the name
saying it. Only test titles quote them, and the rename is what a RENAMED
entry is for.
