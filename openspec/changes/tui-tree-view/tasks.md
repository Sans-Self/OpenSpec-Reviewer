# Tasks: tui-tree-view

## 1. Guides

- [x] 1.1 `render/tui/tree.rs`: `depth(row, multi)` and `guides(rows,
      multi) -> Vec<String>` building the prefix from ancestor
      continuation and the row's own connector.

## 2. The list

- [x] 2.1 `row_line` takes the prefix in the muted style instead of the
      fixed indents; `draw_list` computes the guides once per frame.
- [x] 2.2 A requirement row with scenarios shows `▸` or `▾` from
      `App.unfolded` before its approval mark; one without shows a space.

## 3. Tests and docs

- [x] 3.1 Tests titled by requirement: a capability's last requirement
      draws `└─`; a scenario under an unfolded requirement that has a
      later sibling draws `│` in the requirement column; two changes put
      the change rows at the root; `Space` flips the marker.
- [x] 3.2 README: the fold marker.
- [x] 3.3 `clippy` and `rustfmt` clean.
