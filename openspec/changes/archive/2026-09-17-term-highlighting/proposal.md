# term-highlighting

## Why

The glossary knows where every term sits. `Matcher::ranges` returns the
byte range of every hit, and `deprecated_in_canon` already decides which
of those hits are live and which are shielded by a longer admitted name.
None of it reaches the reader. The detail pane styles spans by diff
verdict and nothing else, so a term reads like any other word, and the
only glossary surface in the view is the `D` overlay, which appends a
list of definitions to the bottom of the pane with no link back to where
the word stood. A finding that says `deprecated synonym "admin"` sends
the reviewer hunting by eye for a word the tool has an offset for.

Marking terms in place needs no colour model, no configuration file and
no terminal probing. It needs the offsets the matcher already returns
and a modifier the palette already has. `tui-color` specifies it as one
requirement among eight, behind palette tables, an XDG configuration
file and `OSC 11` background detection. This change takes that
requirement out so it can ship on its own; `tui-color` drops it.

## What Changes

- Every admitted name of a glossary term in the detail text is
  underlined where it appears.
- A deprecated synonym is marked in the warning style on top of the
  underline, so the word the reviewer has to change is the one that
  stands out.
- An occurrence inside a longer admitted name belongs to that name. The
  `rotation key` inside `PLC rotation key` is not a synonym occurrence,
  matching the shielding rule the checks already apply.
- Marking composes with the diff: an underlined term inside an added
  paragraph keeps the added style and the `+` glyph.
- `tui-color` loses "Glossary terms are highlighted in the detail text"
  and its task, and keeps the rest.

## Capabilities

### Modified Capabilities

- `review-tui`: a new requirement for term marking in the detail text.

## Impact

- `src/render/tui/ui.rs` splits detail spans on term offsets.
- `src/glossary/` exposes the shielded-occurrence decision that
  `checks.rs` makes privately today, so the view and the check answer
  "is this a live synonym" the same way.
- `Matcher::ranges` reports offsets in citation-stripped text, so the
  view needs those offsets mapped back onto the text it draws.
- No new dependency. `Modifier::UNDERLINED` and `Palette::warning` both
  exist.

## Non-goals

- Colour roles, palettes, background detection and the user
  configuration file. They stay in `tui-color`.
- Marking terms in plain or markdown output.
- A per-occurrence definition popup. `D` still answers "what does this
  mean".
