# Design: phrase-precision

## Context

Both checks read a requirement body as one flat string. The glossary's
own bodies are not flat: they carry marker lines that list words, and
they name longer phrases that contain shorter ones. A check that reads
the body as prose reports the list as a sentence and the containing
phrase as its part.

## Goals / Non-Goals

**Goals:** a deprecated synonym warns where the word is used and nowhere
else; retiring a word from a marker line is not a sentence rewrite.

**Non-Goals:** a term ownership model for overlapping phrases, or a
per-capability switch for the phrase tier.

## Decisions

**Longest phrase wins, by span containment.** The shield set is every
term name and admitted synonym of two or more words. A deprecated
synonym's hit is dropped when some shield's match contains it. This is
containment on byte ranges over the same citation-stripped text, so
`Matcher` grows `ranges` and the checks compare offsets rather than
asking `is_match`.

Single-word phrases stay out of the shield set. `anchor` is admitted for
`lineage` and deprecated for `verification method`; admitting one-word
shields would let the admission silence the deprecation everywhere,
while `lineage anchor` shields exactly the uses that belong to `lineage`.

The shield set is built from the glossary under review, so terms a change
adds shield the same run that adds them.

**The phrase tier reads the meaning.** `parse_markers` already separates a
body into meaning, admitted, deprecated and the binding line. The phrase
tier takes `meaning`; the backticked and quoted tiers keep the raw text,
where a span inside a marker line is still a span worth tracking. The
same text serves the sibling side, so a term that lists a word as
deprecated does not read as a sibling still using it.

Marker lines only exist in glossary bodies, so stripping them everywhere
costs nothing and saves threading the capability into the drift module,
which knows nothing else about the glossary.
