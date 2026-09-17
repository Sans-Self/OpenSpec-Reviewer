# review-tui (delta)

## MODIFIED Requirements

### Requirement: Keys follow vi and arrow conventions

The tool MUST bind: `j`/`k` and arrows to move in the list, `Tab` to
switch pane focus, `n`/`p` to jump to the next and previous row with a
finding, `Space` to fold and unfold a requirement's scenarios, `a` to
toggle approval, `e` to write a note, `N` to open the notes panel, `H`
to open history, `m` to cycle display mode, `?` for a help overlay, `q`
and `Esc` to quit. `a` on a scenario row MUST toggle its parent
requirement. `n` and `p` MUST unfold a requirement they land inside.
The help overlay lists every binding.

#### Scenario: Jump to next finding

- **GIVEN** a later row with a finding
- **WHEN** the user presses `n`
- **THEN** the cursor moves to that row
- **AND** the detail pane shows it

#### Scenario: Help

- **WHEN** the user presses `?`
- **THEN** an overlay lists every binding
- **AND** any key closes it

#### Scenario: Jump into a folded requirement

- **GIVEN** a folded requirement whose scenario carries a finding
- **WHEN** the user presses `n`
- **THEN** that requirement unfolds
- **AND** the cursor is on the scenario row

#### Scenario: Approve from a scenario row

- **GIVEN** the cursor on a scenario row of an unapproved requirement
- **WHEN** the user presses `a`
- **THEN** the parent requirement row shows `[√]`
