//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::glossary::{parse_markers, Glossary, Matcher};
use openspec_reviewer::model::Canon;

fn glossary_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("definitions", &fixture("glossary", "definitions.md"))
        .canon("key-rotation", &fixture("glossary", "key-rotation.md"))
        .canon(
            "keyring-tombstones",
            &fixture("glossary", "keyring-tombstones.md"),
        )
        .canon("sharing-grants", &fixture("glossary", "sharing-grants.md"))
        .delta(
            "epoch-retire",
            "key-rotation",
            &fixture("glossary", "delta.md"),
        )
        .write(
            "openspec/changes/epoch-retire/proposal.md",
            "# epoch-retire\n",
        )
        .write("openspec/reviewer.toml", "[lint]\n");
    repo
}

fn glossary_canon() -> Canon {
    let mut canon = Canon::default();
    for cap in [
        "definitions",
        "key-rotation",
        "keyring-tombstones",
        "sharing-grants",
    ] {
        canon.specs.insert(
            cap.into(),
            openspec_reviewer::model::parse_canon_spec(&fixture("glossary", &format!("{cap}.md"))),
        );
    }
    canon
}

fn review_json(repo: &Repo, change: &str) -> serde_json::Value {
    let out = run_in(
        repo.root(),
        &["--format", "json", "--no-state", "change", change],
    );
    serde_json::from_str(&stdout(&out)).unwrap_or_else(|e| panic!("{e}: {}", stdout(&out)))
}

fn findings_of<'a>(doc: &'a serde_json::Value, requirement: &str) -> Vec<&'a serde_json::Value> {
    doc["changes"][0]["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["pairings"].as_array().unwrap())
        .filter(|p| p["name"] == requirement)
        .flat_map(|p| p["findings"].as_array().unwrap())
        .collect()
}

#[test]
fn the_glossary_is_a_capability_named_definitions__two_terms_in_canon() {
    let canon = glossary_canon();
    let g = Glossary::build(&canon, &[], "definitions");
    assert_eq!(g.terms.len(), 4);
    let key = g.get("group key").unwrap();
    assert!(key.meaning.starts_with("The symmetric key that wraps"));
    assert!(
        !key.meaning.contains("Deprecated"),
        "the line is not prose: {}",
        key.meaning
    );
    assert_eq!(key.deprecated, vec!["workspace key", "rotation key"]);
    let doc = review_json(&glossary_repo(), "epoch-retire");
    let defs = doc["definitions"].as_array().expect("definitions array");
    assert_eq!(defs.len(), 4);
    assert_eq!(defs[0]["term"], "group key");
    assert!(defs[0]["meaning"].is_string());
    assert_eq!(defs[1]["deprecated"], serde_json::json!(["admin", "owner"]));
}

#[test]
fn the_glossary_is_a_capability_named_definitions__no_glossary() {
    let repo = Repo::from_fixture("lint");
    let review = run_in(
        repo.root(),
        &["--findings-only", "change", "rotation-grace-periods"],
    );
    assert!(
        stdout(&review).contains("; no glossary"),
        "{}",
        stdout(&review)
    );
    assert!(
        !stdout(&review).contains("deprecated"),
        "{}",
        stdout(&review)
    );
    let lint = run_in(repo.root(), &["lint"]);
    assert!(stdout(&lint).contains("; no glossary"), "{}", stdout(&lint));
}

#[test]
fn the_glossary_is_a_capability_named_definitions__configurable_capability() {
    let repo = glossary_repo();
    repo.write(
        "openspec/reviewer.toml",
        "[lint]\n\n[definitions]\ncapability = \"terms\"\n",
    );
    let out = run_in(repo.root(), &["--findings-only", "change", "epoch-retire"]);
    assert!(stdout(&out).contains("; no glossary"), "{}", stdout(&out));
    repo.write(
        "openspec/reviewer.toml",
        "[lint]\n\n[definitions]\ncapability = \"\"\n",
    );
    let off = run_in(repo.root(), &["--findings-only", "change", "epoch-retire"]);
    assert!(
        stdout(&off).contains("; no glossary"),
        "an empty capability switches it off: {}",
        stdout(&off)
    );
    assert!(!stdout(&off).contains("synonym"), "{}", stdout(&off));
}

#[test]
fn a_term_lists_the_words_not_to_use_for_it__two_synonyms() {
    let m = parse_markers("The key.\n\n- **Deprecated:** workspace key, rotation key\n");
    assert_eq!(m.meaning, "The key.");
    assert_eq!(m.deprecated, vec!["workspace key", "rotation key"]);
}

#[test]
fn a_term_lists_the_words_not_to_use_for_it__no_deprecated_line() {
    let m = parse_markers("Just a meaning.\n");
    assert_eq!(m.meaning, "Just a meaning.");
    assert!(m.deprecated.is_empty());
}

#[test]
fn a_term_lists_the_words_that_are_acceptable_for_it__two_admitted_synonyms() {
    let m = parse_markers("The role.\n\n- **Admitted:** steward, custodian\n");
    assert_eq!(m.meaning, "The role.");
    assert_eq!(m.admitted, vec!["steward", "custodian"]);
}

#[test]
fn a_term_lists_the_words_that_are_acceptable_for_it__no_admitted_line() {
    assert!(parse_markers("Just a meaning.\n").admitted.is_empty());
}

#[test]
fn a_term_lists_the_words_that_are_acceptable_for_it__both_lines() {
    let m = parse_markers(
        "The role.\n\n- **Admitted:** steward\n- **Deprecated:** admin\n\nMore prose.\n",
    );
    assert_eq!(m.meaning, "The role.\n\nMore prose.");
    assert_eq!(m.admitted, vec!["steward"]);
    assert_eq!(m.deprecated, vec!["admin"]);
}

#[test]
fn a_term_lists_the_words_that_are_acceptable_for_it__unrecognized_bold_line() {
    let m = parse_markers("The role.\n\n- **Example:** a steward of the keyring\n");
    assert!(
        m.meaning
            .contains("- **Example:** a steward of the keyring"),
        "an author's own bullet is prose: {}",
        m.meaning
    );
    assert!(m.admitted.is_empty());
}

#[test]
fn a_term_lists_the_words_that_are_acceptable_for_it__using_an_admitted_synonym() {
    let repo = glossary_repo();
    repo.delta(
        "epoch-retire",
        "sharing-grants",
        "## MODIFIED Requirements\n\n### Requirement: A grant names its chain head\n\nA grant MUST name the chain head it was issued under.\n\n#### Scenario: Steward issues\n\n- **WHEN** a steward issues a grant\n- **THEN** the grant names the chain head\n",
    );
    let doc = review_json(&repo, "epoch-retire");
    let fs = findings_of(&doc, "A grant names its chain head");
    assert!(
        fs.iter()
            .all(|f| f["synonym"] != "steward" && f["term"] != "steward"),
        "an admitted synonym is never a finding: {fs:#?}"
    );
}

#[test]
fn a_term_lists_the_words_that_are_acceptable_for_it__word_on_both_lines() {
    let repo = glossary_repo();
    let mut defs = fixture("glossary", "definitions.md");
    defs = defs.replace(
        "- **Deprecated:** admin, owner",
        "- **Deprecated:** admin, steward",
    );
    repo.canon("definitions", &defs);
    repo.delta(
        "epoch-retire",
        "sharing-grants",
        "## MODIFIED Requirements\n\n### Requirement: A grant names its chain head\n\nA grant MUST name the chain head it was issued under.\n\n#### Scenario: Steward issues\n\n- **WHEN** a steward issues a grant\n- **THEN** the grant names the chain head\n",
    );
    let doc = review_json(&repo, "epoch-retire");
    let fs = findings_of(&doc, "A grant names its chain head");
    let hit = fs
        .iter()
        .find(|f| f["kind"] == "uses_deprecated_synonym" && f["synonym"] == "steward")
        .unwrap_or_else(|| panic!("the contradiction is shown, not resolved: {fs:#?}"));
    assert_eq!(hit["term"], "manager");
}

#[test]
fn a_deprecated_synonym_in_a_spec_is_a_warning__delta_says_workspace_key() {
    let doc = review_json(&glossary_repo(), "epoch-retire");
    let fs = findings_of(&doc, "Rotation produces a new group key");
    let hit = fs
        .iter()
        .find(|f| f["kind"] == "uses_deprecated_synonym")
        .unwrap_or_else(|| panic!("{fs:#?}"));
    assert_eq!(hit["synonym"], "admin");
    assert_eq!(hit["term"], "manager");
    assert_eq!(hit["severity"], "warning");
    assert_eq!(hit["location"]["scenario"], "Fresh key");
    let text = stdout(&run_in(
        glossary_repo().root(),
        &["--findings-only", "change", "epoch-retire"],
    ));
    assert!(
        text.contains("#Fresh key: uses deprecated synonym `admin`, the term is `manager`"),
        "{text}"
    );
}

#[test]
fn a_deprecated_synonym_in_a_spec_is_a_warning__synonym_is_a_substring() {
    assert!(!Matcher::new("admin").is_match("An administrative script may bypass this."));
    assert!(Matcher::new("admin").is_match("An Admin may bypass this."));
    assert!(
        Matcher::new("manager").is_match("Two managers"),
        "plural forms are the same word"
    );
    assert!(Matcher::new("workspace key").is_match("the workspace   keys rotate"));
    let repo = glossary_repo();
    let lint = run_in(repo.root(), &["lint"]);
    assert!(
        !stdout(&lint).contains("Clients act on the outcome"),
        "administrative: {}",
        stdout(&lint)
    );
}

#[test]
fn a_deprecated_synonym_in_a_spec_is_a_warning__synonym_inside_a_citation() {
    assert!(!Matcher::new("admin")
        .is_match("see `spec:keyring-tombstones § Admin API mints invites` for it"));
    let doc = review_json(&glossary_repo(), "epoch-retire");
    let fs = findings_of(&doc, "Rotation keeps a grace period");
    assert!(
        fs.iter().all(|f| f["kind"] != "uses_deprecated_synonym"),
        "{fs:#?}"
    );
}

#[test]
fn a_deprecated_synonym_in_a_spec_is_a_warning__canon_hit_in_lint_or_on_the_term() {
    let repo = glossary_repo();
    let lint = run_in(repo.root(), &["lint"]);
    let line = stdout(&lint)
        .lines()
        .find(|l| l.contains("uses deprecated synonym `workspace key`"))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("{}", stdout(&lint)));
    assert!(
        line.starts_with("warning: openspec/specs/key-rotation/spec.md"),
        "{line}"
    );
    assert!(
        line.contains("Members re-encrypt on their next write"),
        "{line}"
    );
    assert_eq!(lint.status.code(), Some(1), "a warning sets exit 1");

    repo.delta(
        "epoch-retire",
        "definitions",
        "## MODIFIED Requirements\n\n### Requirement: group key\n\nThe key, reworded.\n\n- **Deprecated:** workspace key, rotation key\n\n#### Scenario: In a sentence\n\n- **WHEN** x\n- **THEN** y\n",
    );
    let doc = review_json(&repo, "epoch-retire");
    let fs = findings_of(&doc, "group key");
    let hit = fs
        .iter()
        .find(|f| f["kind"] == "uses_deprecated_synonym")
        .unwrap_or_else(|| panic!("{fs:#?}"));
    assert_eq!(
        hit["details"][0],
        "key-rotation § Members re-encrypt on their next write"
    );
}

#[test]
fn a_term_nobody_uses_is_a_note__orphan_term() {
    let repo = glossary_repo();
    let lint = run_in(repo.root(), &["lint"]);
    assert!(
        stdout(&lint).contains("note: openspec/specs/definitions/spec.md: defined but unused: no requirement outside the glossary uses `loket`"),
        "{}",
        stdout(&lint)
    );
    assert!(
        !stdout(&lint).contains("uses `manager`"),
        "{}",
        stdout(&lint)
    );
}

#[test]
fn a_term_nobody_uses_is_a_note__used_only_under_an_admitted_synonym() {
    let repo = glossary_repo();
    let lint = stdout(&run_in(repo.root(), &["lint"]));
    assert!(
        !lint.contains("uses `ledger`"),
        "canon says `log`, never `ledger`: {lint}"
    );
}

#[test]
fn a_recurring_undefined_term_is_a_note__identifier_in_three_capabilities() {
    let repo = glossary_repo();
    let text = stdout(&run_in(repo.root(), &["lint"]));
    let idx = text
        .find("recurring term without definition: `chainHead`")
        .unwrap_or_else(|| panic!("{text}"));
    let after = &text[idx..];
    assert!(
        after.contains("    key-rotation § Rotation produces a new group key"),
        "{after}"
    );
    assert!(
        after.contains("    keyring-tombstones § A tombstone names the rotation it closes"),
        "{after}"
    );
    assert!(
        after.contains("    sharing-grants § A grant names its chain head"),
        "{after}"
    );
    assert!(
        !text.contains("definition: `spec:x"),
        "citations are not terms: {text}"
    );
}

#[test]
fn a_recurring_undefined_term_is_a_note__recurring_inside_one_capability() {
    let repo = glossary_repo();
    let mut spec = String::from("# alpha\n\n## Requirements\n");
    for i in 0..5 {
        spec.push_str(&format!("\n### Requirement: Rule {i}\n\nUses `localOnly` here.\n\n#### Scenario: S\n\n- **WHEN** x\n- **THEN** y\n"));
    }
    repo.canon("alpha", &spec);
    let text = stdout(&run_in(repo.root(), &["lint"]));
    assert!(!text.contains("`localOnly`"), "{text}");
}

#[test]
fn a_recurring_undefined_term_is_a_note__recurring_admitted_synonym() {
    let repo = glossary_repo();
    for cap in ["a", "b", "c"] {
        repo.canon(cap, &format!("# {cap}\n\n## Requirements\n\n### Requirement: R {cap}\n\nThe `steward` decides.\n\n#### Scenario: S\n\n- **WHEN** x\n- **THEN** y\n"));
    }
    let text = stdout(&run_in(repo.root(), &["lint"]));
    assert!(
        !text.contains("definition: `steward`"),
        "an admitted synonym is known: {text}"
    );
}

#[test]
fn a_change_that_introduces_an_undefined_term_is_a_warning__new_identifier_used_twice() {
    let doc = review_json(&glossary_repo(), "epoch-retire");
    let first = findings_of(&doc, "Rotation produces a new group key");
    let hit = first
        .iter()
        .find(|f| f["kind"] == "new_term_undefined")
        .unwrap_or_else(|| panic!("{first:#?}"));
    assert_eq!(hit["term"], "chainParent");
    assert_eq!(hit["severity"], "warning");
    let second = findings_of(&doc, "Rotation keeps a grace period");
    assert!(
        second.iter().all(|f| f["kind"] != "new_term_undefined"),
        "reported once: {second:#?}"
    );
    assert!(
        first
            .iter()
            .all(|f| f["term"] != "epochNumber" && f["term"] != "chainHead"),
        "present on a before side: {first:#?}"
    );
}

#[test]
fn a_change_that_introduces_an_undefined_term_is_a_warning__new_term_defined_in_the_same_change() {
    let repo = glossary_repo();
    repo.delta(
        "epoch-retire",
        "definitions",
        "## ADDED Requirements\n\n### Requirement: chainParent\n\nThe head a rotation record retired.\n\n#### Scenario: In a sentence\n\n- **WHEN** x\n- **THEN** y\n",
    );
    let doc = review_json(&repo, "epoch-retire");
    let fs = findings_of(&doc, "Rotation produces a new group key");
    assert!(
        fs.iter().all(|f| f["kind"] != "new_term_undefined"),
        "{fs:#?}"
    );
    assert_eq!(
        doc["definitions"].as_array().unwrap().len(),
        5,
        "the new term is in the glossary"
    );
}

#[test]
fn a_change_that_introduces_an_undefined_term_is_a_warning__used_once() {
    let doc = review_json(&glossary_repo(), "epoch-retire");
    let all: Vec<_> = doc["changes"][0]["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["pairings"].as_array().unwrap())
        .flat_map(|p| p["findings"].as_array().unwrap())
        .filter(|f| f["kind"] == "new_term_undefined")
        .collect();
    assert_eq!(all.len(), 1, "only chainParent: {all:#?}");
}

#[test]
fn a_change_that_introduces_an_undefined_term_is_a_warning__new_span_is_an_admitted_synonym() {
    let repo = glossary_repo();
    repo.delta(
        "epoch-retire",
        "sharing-grants",
        "## MODIFIED Requirements\n\n### Requirement: A grant names its chain head\n\nA grant MUST name the `steward` that issued it and the chain head.\n\n#### Scenario: Steward issues\n\n- **WHEN** the `steward` issues a grant\n- **THEN** the grant names the chain head\n",
    );
    let doc = review_json(&repo, "epoch-retire");
    let fs = findings_of(&doc, "A grant names its chain head");
    assert!(
        fs.iter().all(|f| f["term"] != "steward"),
        "the glossary knows it: {fs:#?}"
    );
}

#[test]
fn changing_a_term_lists_the_requirements_that_use_it__meaning_changes() {
    let repo = glossary_repo();
    repo.delta(
        "epoch-retire",
        "definitions",
        "## MODIFIED Requirements\n\n### Requirement: group key\n\nThe key, reworded.\n\n#### Scenario: In a sentence\n\n- **WHEN** x\n- **THEN** y\n",
    );
    let doc = review_json(&repo, "epoch-retire");
    let fs = findings_of(&doc, "group key");
    let hit = fs
        .iter()
        .find(|f| f["kind"] == "term_in_use")
        .unwrap_or_else(|| panic!("{fs:#?}"));
    assert_eq!(hit["severity"], "note");
    let uses = hit["uses"].as_array().unwrap();
    assert_eq!(uses.len(), 2, "{uses:?}");
    assert!(uses.contains(&serde_json::json!(
        "key-rotation § Members re-encrypt on their next write"
    )));
    assert_eq!(hit["details"].as_array().unwrap().len(), 2);
}

#[test]
fn the_detail_pane_can_show_the_terms_a_pairing_uses__pairing_uses_two_terms() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use openspec_reviewer::render::tui::{draw, App};
    use openspec_reviewer::source::Source;
    let repo = glossary_repo();
    let snapshot = openspec_reviewer::source::ChangeSource {
        root: repo.root().to_path_buf(),
        name: "epoch-retire".into(),
    }
    .fetch()
    .unwrap();
    let review = openspec_reviewer::build::build_review(repo.root(), &snapshot).unwrap();
    let mut app = App::new(review, Default::default());
    // The first selectable row is the proposal artefact; step onto the requirement.
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert_eq!(
        app.current_pairing().unwrap().name,
        "Rotation produces a new group key"
    );
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 60)).unwrap();
    terminal.draw(|f| draw(f, &mut app)).unwrap();
    let before = format!("{:?}", terminal.backend().buffer());
    assert!(
        !before.contains("The symmetric key that wraps"),
        "closed by default"
    );

    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT));
    terminal.draw(|f| draw(f, &mut app)).unwrap();
    let screen = format!("{:?}", terminal.backend().buffer());
    let key = screen
        .find("The symmetric key that wraps")
        .unwrap_or_else(|| panic!("{screen}"));
    let manager = screen
        .find("The membership role")
        .unwrap_or_else(|| panic!("{screen}"));
    assert!(key < manager, "group key first, then manager");
    assert!(
        screen.contains("workspace key"),
        "deprecated list: {screen}"
    );

    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    assert!(!app.definitions_shown(), "per pairing, not global");
    app.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    assert!(app.definitions_shown(), "still open on the first pairing");
}

#[test]
fn the_detail_pane_can_show_the_terms_a_pairing_uses__toggle_off() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use openspec_reviewer::render::tui::App;
    use openspec_reviewer::source::Source;
    let repo = glossary_repo();
    let snapshot = openspec_reviewer::source::ChangeSource {
        root: repo.root().to_path_buf(),
        name: "epoch-retire".into(),
    }
    .fetch()
    .unwrap();
    let review = openspec_reviewer::build::build_review(repo.root(), &snapshot).unwrap();
    let mut app = App::new(review, Default::default());
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT));
    assert!(app.definitions_shown());
    app.handle_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT));
    assert!(!app.definitions_shown());
    assert!(openspec_reviewer::render::tui::BINDINGS
        .iter()
        .any(|(k, _)| *k == "D"));
}

#[test]
fn definitions_findings_reach_plain_output_and_json__agent_reads_the_glossary() {
    let repo = glossary_repo();
    let doc = review_json(&repo, "epoch-retire");
    let defs = doc["definitions"].as_array().unwrap();
    for d in defs {
        assert!(
            d["term"].is_string()
                && d["meaning"].is_string()
                && d["admitted"].is_array()
                && d["deprecated"].is_array(),
            "{d}"
        );
    }
    let plain = stdout(&run_in(
        repo.root(),
        &["--no-state", "change", "epoch-retire"],
    ));
    assert!(
        plain.contains("warning: uses deprecated synonym `admin`, the term is `manager`"),
        "{plain}"
    );
    assert!(
        plain.contains("warning: new term without definition: `chainParent`"),
        "{plain}"
    );
    let only = stdout(&run_in(
        repo.root(),
        &["--findings-only", "--no-state", "change", "epoch-retire"],
    ));
    assert!(only.contains("new term without definition"), "{only}");
    assert!(only.contains("summary: 0 errors, 2 warnings"), "{only}");
    assert!(only.ends_with("glossary: 4 terms\n"), "{only}");
}

/// Backticked `spec:` citations were extracted as terms and reported as
/// recurring or new terms without definition.
#[test]
fn bug__citations_counted_as_terms() {
    let spans = openspec_reviewer::drift::terms::quoted_spans(
        "See `spec:alpha § Some rule` and `realTerm`, also \"spec:beta § Other\".",
    );
    assert_eq!(spans.into_iter().collect::<Vec<_>>(), vec!["realTerm"]);
    let repo = glossary_repo();
    for cap in ["a", "b", "c"] {
        repo.canon(cap, &format!("# {cap}\n\n## Requirements\n\n### Requirement: R\n\nSee `spec:key-rotation § Rotation produces a new group key`.\n\n#### Scenario: S\n\n- **WHEN** x\n- **THEN** y\n"));
    }
    let text = stdout(&run_in(repo.root(), &["lint"]));
    assert!(!text.contains("definition: `spec:"), "{text}");
}
