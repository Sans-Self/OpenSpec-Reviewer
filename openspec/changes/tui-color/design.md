# Design: tui-color

## Context

`render::colour::Palette` is a struct with one field, `colour: bool`,
and eight methods returning a `Style`. Every caller already goes through
it, which is what makes this change mostly additive: the palette grows,
the callers ask for more roles, nothing outside `render/` learns about
colour. The one exception is the user config, which is read before the
first draw and hands the palette its choice.

## Goals / Non-Goals

**Goals:** colour that carries meaning in every pane; the glyph rule
intact under colour; legible on light and dark terminals without being
told which; a home for user-bound settings.

**Non-Goals:** truecolour, user-defined palettes, per-project colour
settings. Colour belongs to the person reading, so it is not in
`reviewer.toml`.

## Decisions

**Background tints, modifiers underneath.** In colour mode a whole added
or removed line takes a background colour across the pane width, and a
changed line stays untinted with only its differing spans tinted. Every
tinted span also carries the modifier the non-colour mode would have
used, strikethrough for removed and bold for added, so `review-tui`'s
rule that colour is never the only signal is satisfied by construction
rather than by the gutter glyph alone. Foreground stays the terminal's
default inside a tint, because a red foreground on a red tint is the
first thing a light theme breaks.

**Roles, not colours.** `Palette` gains roles for every thing the view
distinguishes: `added`, `removed`, `changed`, `added_span`,
`removed_span`, `error`, `warning`, `note`, `approved`, `stale`,
`pending`, `accent`, `muted`, `heading`, `term`, `synonym`, `focus`,
`unfocus`, `selected`. A role resolves to a `Style` through
`Palette::for(choice, background)`, where `choice` is the user's palette
and `background` is `Dark` or `Light`. The three palettes are tables of
roles to ANSI 16 colours, each with a light and a dark column; `none`
is the table of modifiers the non-colour mode uses today.

**The accessible pair.** `default` keeps red and green because it is
what every diff tool has taught readers. `accessible` uses blue for
added and orange, ANSI `Yellow` on dark and `Magenta` on light, for
removed, the pair colour-vision research settles on. Warnings move to
magenta in both palettes on light backgrounds, where yellow is
unreadable.

**Background detection is two probes and a default.** `COLORFGBG`, when
set, is `fg;bg` and a background of 7 or 15 means light. Otherwise the
terminal is asked with `OSC 11 ; ?` on the tty while already in raw
mode, before the first draw, and the reply is read with a 100 ms
timeout; luminance of the `rgb:` triple above half means light. No
reply, or not a tty, means dark, which is what every terminal was
assumed to be before this change. The probe runs once per process. A
reply that arrives late is drained by the key loop's existing filter for
non-key events, so it cannot become a keystroke.

**Colour follows the terminal, `--color` forces it.** Plain review
output and `lint` share one rule: escape codes when stdout is a
terminal, none when it is not, `--color` turning them on regardless for
pagers, `NO_COLOR` or `palette = "none"` turning them off regardless.
The old opt-in behaviour is a special case of this rule, so the
`--color` flag keeps its meaning.

**Term highlighting reuses the matcher.** The glossary's `Matcher::first`
already yields byte offsets in citation-stripped text. Highlighting
needs every offset, so the matcher gains an iterator, and the inline
view splits its spans at term boundaries before styling. Deprecated
synonyms use the same path with the `synonym` role. Highlighting is a
rendering concern and touches no finding.

**The user file is `directories` plus `toml`.** `ProjectDirs` gives
`$XDG_CONFIG_HOME/openspec-reviewer/` on Linux and the equivalent
elsewhere; `config.toml` inside it is parsed with `deny_unknown_fields`
like `reviewer.toml`. Absent is fine. Malformed is an error naming the
key, because a silent fallback would hide a typo in `palette` behind a
default that looks like the tool ignoring the user.

**Precedence, top wins.**

```
NO_COLOR set                     → none
config palette = "none"          → none
stdout not a tty, no --color     → none
TERM=dumb                        → none
config palette                   → that palette
otherwise                        → default
detected background              → light or dark column of the palette
```

## Risks / Trade-offs

- A terminal that swallows `OSC 11` and answers nothing costs 100 ms at
  startup. → Acceptable once per process; `COLORFGBG` skips the probe
  when present, and `palette = "none"` skips it entirely.
- Background tints across the pane width can make a dense removed block
  loud. → It is the convention every graphical diff uses, and the
  reader chose colour mode; `none` is one key away.
- Highlighting terms inside a changed span composes two styles. →
  Underline is orthogonal to tint and to bold or strikethrough, so the
  composition is a union of modifiers, never a conflict.

## Testing

`TestBackend` buffers expose cell styles, so every role can be asserted
where it lands: the tint on an added line, the strikethrough under a
removed span, the accent on the focused border, the underline on a term.
Background detection is tested through the pure classifier on sample
`COLORFGBG` values and `rgb:` replies; the probe itself is one function
that reads from a `Read` with a deadline and is tested with a cursor.
The user file is tested through `XDG_CONFIG_HOME` pointed at a temp dir.
