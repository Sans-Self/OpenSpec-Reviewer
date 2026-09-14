# tui-color

## Why

The view has a palette of eight roles and uses it for the diff and the
selection bar, and nowhere else. The list pane, the pane you scan, is
bold or nothing: approval marks, severity markers and delta glyphs all
draw in the default colour. Word-level changes wrap in `[-old-]{+new+}`,
the piped-output convention, on a screen that could paint them instead.
The focused pane has no visible border. The lint prints black on black.
And the palette is ANSI 16 with no idea whether the terminal behind it
is dark or light, so `Yellow` warnings disappear on white.

## What Changes

- In colour mode the diff paints backgrounds: an added line green, a
  removed line red, a changed word red or green with strikethrough or
  bold underneath it, so the glyph rule still holds when colour does not.
  Non-colour mode keeps the glyphs, the modifiers and the wdiff marks.
- The list pane colours what it already draws: severity markers,
  approval marks, delta glyphs, the note marker.
- The focused pane's border takes the accent colour; the other pane's
  border is dim.
- Glossary terms in the detail text are underlined; a deprecated synonym
  is painted in the warning colour where it stands.
- The status bar colours a count only when it is not zero.
- `lint` output is coloured when stdout is a terminal: severity word,
  dim path, bold `capability § requirement`, coloured non-zero counts.
  Plain review output follows the same rule, so `--color` becomes the
  override for pagers rather than the only way to get colour.
- The tool detects a light or dark terminal background from `COLORFGBG`
  or an `OSC 11` query with a short timeout, and picks the variant of the
  palette that reads on it. No answer means dark.
- A user configuration file at `$XDG_CONFIG_HOME/openspec-reviewer/config.toml`
  holds settings that belong to the person, not the project. Its first
  key is `palette`: `default`, `accessible` or `none`. `accessible`
  swaps the red and green pair for blue and orange. `none` is a standing
  `NO_COLOR`.

## Capabilities

### New Capabilities

- `tui-color`: colour-mode rendering of the diff, list, borders, terms,
  status bar and lint output; background detection; the palette roles
  and their variants.
- `user-config`: the user configuration file, its location and
  precedence, and the `palette` key.

### Modified Capabilities

- `review-tui`: "Colour is never the only signal" names the rule that a
  tinted span keeps a modifier underneath it.
- `plain-output`: "Plain text uses no escape codes unless asked" becomes
  no escape codes when stdout is not a terminal, colour when it is,
  `--color` forcing it either way.

## Impact

- `src/render/colour.rs` grows roles and variants and becomes the one
  place that answers "which style for this role".
- `src/render/tui/ui.rs`, `src/render/text.rs`, `src/render/lint.rs`
  ask the palette instead of hard-coding.
- New `src/render/terminal.rs` for background detection.
- New `src/config/user.rs` for the XDG file.
- `directories` already gives the config directory; no new dependency for
  the file. The `OSC 11` query uses crossterm's raw mode, already a
  dependency.

## Non-goals

- Truecolour. Sixteen colours inherit the terminal theme; truecolour
  fights it.
- Line numbers in the diff. Paragraphs have none worth showing.
- User-defined palettes. Three named ones; a fourth is a change.
