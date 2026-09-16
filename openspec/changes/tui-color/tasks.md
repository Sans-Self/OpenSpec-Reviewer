# Tasks: tui-color

## 1. Palette

- [ ] 1.1 `Palette` becomes roles resolved through `Palette::for(choice,
      background)`; the three palettes as tables with light and dark
      columns; every role of every palette carries a modifier.
- [ ] 1.2 Background classifier over `COLORFGBG` values and `OSC 11`
      replies, and the probe with a 100 ms deadline over a `Read`.

## 2. User configuration

- [ ] 2.1 `config/user.rs`: the XDG path, `deny_unknown_fields`, the
      `palette` enum, absent means defaults, malformed is an error with
      exit `2`.
- [ ] 2.2 Precedence: `NO_COLOR`, `none`, not a tty, `TERM=dumb`, the
      configured palette, the detected background.

## 3. The view

- [ ] 3.1 Colour-mode diff: line tints, span tints with modifiers, no
      wdiff marks; non-colour unchanged.
- [ ] 3.2 List markers, approval marks and delta glyphs in their roles.
- [ ] 3.3 Focused border in the accent colour, the other dim.
- [ ] 3.4 Term underline and synonym colour in the detail text through
      the matcher's offsets.
- [ ] 3.5 Status bar counts coloured when non-zero.

## 4. Text outputs

- [ ] 4.1 Plain review output and `lint` coloured on a terminal, none on
      a pipe, `--color` forcing; the ANSI wrapper takes its colours from
      the palette.

## 5. Wrap-up

- [ ] 5.1 Tests titled by requirement for every scenario, using
      `TestBackend` cell styles and `XDG_CONFIG_HOME` in a temp dir.
- [ ] 5.2 README: the palette setting, its location, `--color`.
