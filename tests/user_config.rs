//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::config::user::{parse, read_user_config_at, UserConfig};
use openspec_reviewer::render::colour::{Background, Palette, PaletteChoice};
use std::path::Path;

const DELTA: &str = "\
## MODIFIED Requirements

### Requirement: Flat index of all routes and pages

The dashboard MUST offer an index view listing every route and every
page of the active website, plus inherited paths.

#### Scenario: Route-mounted page appears as a row

- **WHEN** a route mounts a page
- **THEN** the index shows one row
";

fn repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA);
    repo
}

/// The tool with `XDG_CONFIG_HOME` inside the repo, where `config` was
/// written as `openspec-reviewer/config.toml`.
fn run_with_config(repo: &Repo, config: Option<&str>) -> std::process::Output {
    if let Some(text) = config {
        repo.write(".config/openspec-reviewer/config.toml", text);
    }
    run_in(repo.root(), &["--plain", "--no-state", "change", "foo"])
}

fn config_path(repo: &Repo) -> String {
    repo.root()
        .join(".config/openspec-reviewer/config.toml")
        .display()
        .to_string()
}

#[test]
fn user_settings_live_in_the_xdg_config_directory__absent_file() {
    let repo = repo();
    let out = run_with_config(&repo, None);
    assert_ne!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(stdout(&out).contains("# change foo"));
    assert_eq!(
        read_user_config_at(Path::new("/nonexistent/config.toml")).unwrap(),
        UserConfig::default()
    );
}

#[test]
fn user_settings_live_in_the_xdg_config_directory__unknown_key() {
    let repo = repo();
    let out = run_with_config(&repo, Some("pallete = \"default\"\n"));
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(err.contains("pallete"), "{err}");
    assert!(err.contains(&config_path(&repo)), "{err}");
}

#[test]
fn the_palette_is_a_user_setting__accessible_chosen() {
    let config = parse("palette = \"accessible\"\n").unwrap();
    assert_eq!(config.palette, PaletteChoice::Accessible);
    let palette = Palette::new(config.palette, Background::Dark);
    assert_eq!(palette.added_span().bg, Some(ratatui::style::Color::Blue));
    assert_eq!(parse("").unwrap().palette, PaletteChoice::Default);
}

#[test]
fn the_palette_is_a_user_setting__no_color_wins() {
    let resolved = PaletteChoice::Default.resolve_with(true, false, true, None);
    assert_eq!(resolved, PaletteChoice::None);
    let piped = PaletteChoice::Default.resolve_with(false, false, false, None);
    assert_eq!(piped, PaletteChoice::None, "a pipe gets no colour");
    let forced = PaletteChoice::Accessible.resolve_with(false, true, false, None);
    assert_eq!(
        forced,
        PaletteChoice::Accessible,
        "--color keeps the choice"
    );
    let dumb = PaletteChoice::Default.resolve_with(true, false, false, Some("dumb"));
    assert_eq!(dumb, PaletteChoice::None);
    let none = PaletteChoice::None.resolve_with(true, true, false, None);
    assert_eq!(none, PaletteChoice::None, "none is a standing NO_COLOR");
}

#[test]
fn the_palette_is_a_user_setting__bad_value() {
    let repo = repo();
    let out = run_with_config(&repo, Some("palette = \"solarized\"\n"));
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(err.contains("palette"), "{err}");
    for allowed in ["default", "accessible", "none"] {
        assert!(err.contains(allowed), "{err}");
    }
}
