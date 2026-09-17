# Design: term-highlighting

## Context

`styled_line` in `src/render/tui/ui.rs` turns a `DiffLine` into spans
styled by `ParaKind` and `LineRole`. Word-level change styling already
splits a paragraph into spans, so the pane draws several spans per line
and adding more is not a new shape. `Matcher::ranges` returns every hit
in `Grammar::strip`ped text. `checks.rs` holds `shields`, `covered` and
`used`, which together decide whether a synonym hit is live or sits
inside a longer admitted name.

## Goals / Non-Goals

**Goals:** a term is visible where it stands; the word a finding names
is findable without reading the paragraph twice.

**Non-Goals:** colour roles, palette choice, background detection, plain
and markdown output.

## Decisions

**Underline, not colour.** The diff owns foreground colour at word
level: added green, removed red, changed yellow. A term that took a
colour would either lose the diff verdict or fight it. `UNDERLINED` is
orthogonal, composes with a foreground and a background, and survives
`NO_COLOR` unchanged, so one rule covers both modes instead of two. It
also keeps "Colour is never the only signal" true without a new glyph:
the mark is not a colour to begin with.

A deprecated synonym takes `Palette::warning` on top of the underline.
In colour mode that is a foreground, which does collide with the diff's
own foreground — and the synonym wins, because a word that must change
outranks how it changed. In non-colour mode `warning` is already `BOLD`,
which composes, so nothing collides. The underline stays either way, so
a synonym is never less marked than a plain term.

**The shielded decision moves out of `checks.rs`.** The view must not
underline `rotation key` as a synonym when it is the tail of the
admitted `PLC rotation key`, and the check already refuses to report it
there. Two implementations of one rule drift, so `used` and its
supporting `shields`/`covered` become a glossary-level function that
returns live occurrences, and both callers use it. The check keeps its
boolean by asking whether the list is empty.

**Offsets are the hard part.** `Matcher::ranges` measures
citation-stripped text; the pane draws the text with citations in it.
Stripping changes lengths, so an offset taken on one is meaningless on
the other. Rather than reconstruct a mapping, the glossary returns
occurrences against the text it was handed, keeping the strip internal
and the offsets in the caller's coordinates. This is a change to the
matcher, not to the view, and it is the reason this change is not a
half-hour of span splitting.

**The whole name is one occurrence.** `PLC rotation key` underlines as
three words, not as an underlined `PLC` beside a separately underlined
`rotation key`. The unit is the term the glossary defines, which is also
the unit `D` lists and the unit a finding names.
