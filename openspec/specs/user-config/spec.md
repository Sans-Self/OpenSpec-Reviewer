# user-config Specification

## Purpose
TBD - created by archiving change tui-color. Update Purpose after archive.
## Requirements
### Requirement: User settings live in the XDG config directory

The tool MUST read `$XDG_CONFIG_HOME/openspec-reviewer/config.toml`,
falling back to `~/.config/openspec-reviewer/config.toml` when
`XDG_CONFIG_HOME` is unset. The file holds settings that belong to the
person running the tool, not to the repository. An absent file means
defaults. A file with an unknown key or a malformed value is an error
naming the key and the path, with exit status `2`.

#### Scenario: Absent file

- **GIVEN** no config file under `XDG_CONFIG_HOME`
- **WHEN** the tool runs
- **THEN** it uses the default for every setting

#### Scenario: Unknown key

- **GIVEN** a config file containing `pallete = "default"`
- **WHEN** the tool runs
- **THEN** it stops with an error naming `pallete` and the file path
- **AND** the exit status is `2`

### Requirement: The palette is a user setting

`palette` in the user configuration MUST be one of `default`,
`accessible` or `none`, default `default`. It selects the palette for
the interactive view, plain output and `lint`. `none` has the same
effect as `NO_COLOR`. `NO_COLOR` set in the environment wins over any
value in the file.

#### Scenario: Accessible chosen

- **GIVEN** a config file with `palette = "accessible"`
- **WHEN** the view renders an added line
- **THEN** the line has a blue background

#### Scenario: NO_COLOR wins

- **GIVEN** a config file with `palette = "default"`
- **AND** `NO_COLOR` set
- **WHEN** the view renders
- **THEN** it uses no colour

#### Scenario: Bad value

- **GIVEN** a config file with `palette = "solarized"`
- **WHEN** the tool runs
- **THEN** it stops with an error naming `palette` and the three allowed values

