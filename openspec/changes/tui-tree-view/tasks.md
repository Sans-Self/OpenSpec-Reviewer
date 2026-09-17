# Tasks: tui-tree-view

## 1. Guides

- [x] 1.1 `render/tui/tree.rs`: `depth(row)` and `guides(rows) ->
      Vec<String>` building the prefix from ancestor continuation and the
      row's own connector; only depth 0 draws none.

## 2. The list

- [x] 2.1 `row_line` takes the prefix in the muted style instead of the
      fixed indents; `draw_list` computes the guides once per frame.
- [x] 2.2 A requirement row with scenarios shows `▸` or `▾` from
      `App::is_open` before its approval mark; one without shows a space.
- [x] 2.3 `visible_rows` takes the pinned set and the cursor's anchor;
      every cursor move ends in `rebuild_rows`; `Space` toggles the pin;
      `App::new` opens the requirement the cursor starts on.

## 3. Tests and docs

- [x] 3.1 Tests titled by requirement: artefact rows begin with `├─`; a
      capability's last requirement draws `└─`; a scenario under an open
      requirement that has a later sibling draws `│` in the requirement
      column; two changes put the change rows at the root; the marker
      follows the cursor; moving onto a requirement opens it; leaving
      folds it; `Space` pins it.
- [x] 3.2 README: the fold marker.
- [x] 3.3 `clippy` and `rustfmt` clean.
