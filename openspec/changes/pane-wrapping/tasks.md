# Tasks: pane-wrapping

## 1. Measurement

- [ ] 1.1 `unicode-width` is a direct dependency and the view measures
      text in display columns.
- [ ] 1.2 A wrap function takes a styled line and a width and returns
      the rows it draws, breaking on whitespace and cutting a word
      longer than the width.

## 2. The modes

- [ ] 2.1 Inline mode wraps in `detail_text`; `Paragraph::wrap` is no
      longer used for the detail pane.
- [ ] 2.2 A continuation row carries a blank gutter the width of the
      kind glyph, and the glyph is not repeated.
- [ ] 2.3 `side_by_side` wraps each side within its column and pads the
      shorter side to the taller one; `fit` is gone.
- [ ] 2.4 Raw mode keeps the snapshot's own line breaks and wraps only
      the lines that exceed the column.

## 3. Scrolling

- [ ] 3.1 The view reports the rendered line count alongside
      `detail_height`.
- [ ] 3.2 `scroll_by` clamps to the rendered count less the pane
      height, and to zero when the content is shorter than the pane.

## 4. The list

- [ ] 4.1 A row wider than the list pane ends in `…` at the border.

## 5. Wrap-up

- [ ] 5.1 Tests titled by requirement, asserting rendered cells through
      `TestBackend` at a narrow width, including a wide-character line.
