//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;

const DELTA: &str = "\
## MODIFIED Requirements

### Requirement: Flat index of all routes and pages

The dashboard MUST offer an index view listing every route and every
page of the active website, plus inherited paths.

#### Scenario: Route-mounted page appears as a row

- **WHEN** a route mounts a page
- **THEN** the index shows one row

#### Scenario: Orphan page appears without a path

- **WHEN** a page is not mounted by any route
- **THEN** the page appears as a row without a path
";

fn repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("sweep-gate", "alpha", DELTA);
    repo
}

#[test]
fn plain_text_is_the_default_outside_a_terminal() {
    let repo = repo();
    let out = run_in(repo.root(), &["--no-state", "change", "sweep-gate"]);
    assert!(
        stdout(&out).contains("# change sweep-gate"),
        "piped stdout gets plain text"
    );
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn plain_text_mirrors_the_inline_view() {
    let repo = repo();
    let text = stdout(&run_in(
        repo.root(),
        &["--no-state", "--plain", "change", "sweep-gate"],
    ));
    assert!(
        text.contains("[ ] ~ Flat index of all routes and pages ?"),
        "{text}"
    );
    assert!(text.contains("{+"), "word diff with + marks");
    assert!(
        text.contains("    warning: scenario dropped: `Feature mount appears`"),
        "{text}"
    );
    assert!(
        text.contains("summary: 0 errors, 1 warning, 0 notes"),
        "{text}"
    );
    assert!(
        text.contains("- Scenario: Feature mount appears"),
        "removed scenario marked -"
    );
}

#[test]
fn plain_text_uses_no_escape_codes_unless_asked() {
    let repo = repo();
    let out = run_in(
        repo.root(),
        &["--no-state", "--plain", "change", "sweep-gate"],
    );
    assert!(
        !out.stdout.contains(&0x1b),
        "no ESC byte in default plain output"
    );
    let out = run_in(
        repo.root(),
        &["--no-state", "--plain", "--color", "change", "sweep-gate"],
    );
    assert!(out.stdout.contains(&0x1b));
}

#[test]
fn json_output_is_the_review_model() {
    let repo = repo();
    let out = run_in(
        repo.root(),
        &["--no-state", "--format", "json", "change", "sweep-gate"],
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("one JSON document");
    let p = &v["changes"][0]["capabilities"][0]["pairings"][0];
    assert_eq!(p["kind"], "modified");
    assert_eq!(p["name"], "Flat index of all routes and pages");
    assert!(p["before"]["body"]
        .as_str()
        .unwrap()
        .contains("every route"));
    assert!(p["after"]["body"].as_str().unwrap().contains("inherited"));
    assert_eq!(p["diff"]["scenarios"][2]["status"], "removed");
    assert_eq!(p["findings"][0]["kind"], "scenario_dropped");
    assert_eq!(p["findings"][0]["severity"], "warning");
    assert!(p["state"].is_object());
    assert_eq!(v["summary"]["warnings"], 1);
    assert_eq!(v["changes"][0]["capabilities"][0]["name"], "alpha");
}

#[test]
fn plain_output_can_be_limited_to_findings() {
    let repo = repo();
    let out = run_in(
        repo.root(),
        &["--no-state", "--findings-only", "change", "sweep-gate"],
    );
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert!(lines[0].starts_with("warning  sweep-gate/alpha/Flat index of all routes and pages#Feature mount appears: scenario dropped"), "{}", lines[0]);
    assert!(lines[1].starts_with("summary:"));
    assert_eq!(out.status.code(), Some(1));
}
