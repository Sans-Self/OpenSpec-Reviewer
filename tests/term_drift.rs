//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::drift::{drift_findings, removed_terms, RemovedTerm, Tier};
use openspec_reviewer::model::{Canon, DeltaSpec};
use openspec_reviewer::review::{pair_change, Finding, FindingKind, Pairing};

fn drift_canon() -> Canon {
    let mut canon = Canon::default();
    canon.specs.insert(
        "key-rotation".into(),
        openspec_reviewer::model::parse_canon_spec(&fixture("drift", "key-rotation.md")),
    );
    canon.specs.insert(
        "keyring-tombstones".into(),
        openspec_reviewer::model::parse_canon_spec(&fixture("drift", "keyring-tombstones.md")),
    );
    canon
}

fn pairing_of(canon: &Canon, deltas: Vec<DeltaSpec>) -> (Pairing, Vec<DeltaSpec>) {
    let review = pair_change(&change_of("epoch-retire", deltas.clone()), canon, &[]);
    (review.capabilities[0].pairings[0].clone(), deltas)
}

fn drift(canon: &Canon, deltas: Vec<DeltaSpec>) -> Vec<Finding> {
    let (p, deltas) = pairing_of(canon, deltas);
    drift_findings(&p, canon, &deltas, 5)
}

fn terms_of(before: &str, after: Option<&str>) -> Vec<RemovedTerm> {
    let b = req("R", before, &[]);
    let a = after.map(|t| req("R", t, &[]));
    removed_terms(Some(&b), a.as_ref())
}

fn fixture_delta() -> DeltaSpec {
    delta_of("key-rotation", &fixture("drift", "delta.md"))
}

#[test]
fn a_pairing_yields_the_terms_it_removes__identifier_dropped() {
    let terms = terms_of(
        "Carry `mountType: 'feature'` on the row.",
        Some("Carry nothing on the row."),
    );
    assert!(
        terms.contains(&RemovedTerm {
            tier: Tier::Backticked,
            text: "mountType: 'feature'".into()
        }),
        "{terms:?}"
    );
}

#[test]
fn a_pairing_yields_the_terms_it_removes__phrase_moved_to_a_scenario() {
    let before = req("R", "The row shows the status badge for the mount.", &[]);
    let after = req(
        "R",
        "The row shows the mount.",
        &[("Badge", "- **THEN** the status badge is visible")],
    );
    let terms = removed_terms(Some(&before), Some(&after));
    assert!(
        !terms.iter().any(|t| t.text.contains("status badge")),
        "survives in a scenario: {terms:?}"
    );
}

#[test]
fn a_pairing_yields_the_terms_it_removes__removed_requirement() {
    let terms = terms_of(
        "Set `epoch: 'sealed'` and log the \"rotation ledger\" entry.",
        None,
    );
    assert!(terms.contains(&RemovedTerm {
        tier: Tier::Backticked,
        text: "epoch: 'sealed'".into()
    }));
    assert!(terms.contains(&RemovedTerm {
        tier: Tier::Quoted,
        text: "rotation ledger".into()
    }));
}

#[test]
fn a_pairing_yields_the_terms_it_removes__a_dropped_phrase_is_reported_once() {
    let terms = terms_of("Shows the status badge here.", Some("Shows nothing here."));
    let phrases: Vec<&str> = terms
        .iter()
        .filter(|t| t.tier == Tier::Phrase)
        .map(|t| t.text.as_str())
        .collect();
    assert_eq!(
        phrases,
        vec!["shows the status badge"],
        "one run, edge stop words trimmed"
    );
}

#[test]
fn a_removed_term_found_in_a_sibling_is_a_warning__sibling_still_says_feature() {
    let canon = drift_canon();
    let findings = drift(&canon, vec![fixture_delta()]);
    let hit = findings
        .iter()
        .find(|f| matches!(&f.kind, FindingKind::SiblingUsesRemoved { term, .. } if term == "epoch: 'sealed'"))
        .unwrap_or_else(|| panic!("{findings:#?}"));
    assert_eq!(hit.severity, openspec_reviewer::review::Severity::Warning);
    assert!(hit.message.contains("sibling mentions removed term"));
    assert!(hit
        .message
        .contains("keyring-tombstones § A tombstone names the epoch it closes"));
    assert!(findings.iter().any(|f| matches!(&f.kind, FindingKind::SiblingUsesRemoved { term, .. } if term == "rotation ledger")));
    assert!(findings.iter().any(|f| matches!(&f.kind, FindingKind::SiblingUsesRemoved { term, .. } if term.contains("grace window"))), "{findings:#?}");
}

#[test]
fn a_removed_term_found_in_a_sibling_is_a_warning__same_capability_is_not_a_sibling() {
    let mut canon = drift_canon();
    canon.specs.get_mut("key-rotation").unwrap().push(req(
        "Another rule",
        "Also uses `epoch: 'sealed'`.",
        &[],
    ));
    let findings = drift(&canon, vec![fixture_delta()]);
    assert!(
        findings
            .iter()
            .all(|f| !f.message.contains("key-rotation § Another rule")),
        "{findings:#?}"
    );
}

#[test]
fn a_sibling_the_change_already_touches_is_not_a_finding__change_updates_the_sibling_too() {
    let canon = drift_canon();
    let sibling_fixed = delta_of(
        "keyring-tombstones",
        "## MODIFIED Requirements\n\n### Requirement: A tombstone names the epoch it closes\n\nA tombstone MUST carry the epoch number it retires. Records still marked `epoch: 'retired'` are readable until the purge runs; the audit log lists them.\n\n#### Scenario: Epoch on the tombstone\n\n- **WHEN** an epoch is retired\n- **THEN** its tombstone names that epoch\n",
    );
    let findings = drift(&canon, vec![fixture_delta(), sibling_fixed]);
    assert!(
        findings
            .iter()
            .all(|f| !f.message.contains("A tombstone names the epoch it closes")),
        "{findings:#?}"
    );
}

#[test]
fn a_sibling_the_change_already_touches_is_not_a_finding__change_touches_the_sibling_but_keeps_the_term(
) {
    let canon = drift_canon();
    let sibling_kept = delta_of(
        "keyring-tombstones",
        "## MODIFIED Requirements\n\n### Requirement: A tombstone names the epoch it closes\n\nA tombstone MUST carry the epoch. Records still marked `epoch: 'sealed'` stay readable.\n\n#### Scenario: Epoch on the tombstone\n\n- **WHEN** an epoch is retired\n- **THEN** its tombstone names that epoch\n",
    );
    let findings = drift(&canon, vec![fixture_delta(), sibling_kept]);
    let hit = findings
        .iter()
        .find(|f| matches!(&f.kind, FindingKind::SiblingUsesRemoved { term, kept_in_delta: true, .. } if term == "epoch: 'sealed'"))
        .unwrap_or_else(|| panic!("{findings:#?}"));
    assert!(hit
        .message
        .contains("the delta for keyring-tombstones still contains it"));
}

#[test]
fn an_old_requirement_name_in_prose_is_a_warning__old_name_in_a_sibling_body() {
    let canon = drift_canon();
    let rename = delta_of(
        "key-rotation",
        "## RENAMED Requirements\n\n- FROM: `### Requirement: Rotation produces a new epoch key`\n- TO: `### Requirement: Rotation mints a new epoch key`\n",
    );
    let findings = drift(&canon, vec![rename]);
    let hit = findings
        .iter()
        .find(|f| matches!(f.kind, FindingKind::SiblingUsesOldName { .. }))
        .unwrap_or_else(|| panic!("{findings:#?}"));
    assert!(hit
        .message
        .contains("keyring-tombstones § Clients act on the outcome, never on URI matching"));
    assert_eq!(
        hit.details,
        vec!["openspec/specs/keyring-tombstones/spec.md"]
    );
}

#[test]
fn an_old_requirement_name_in_prose_is_a_warning__old_name_only_as_a_citation() {
    let mut canon = drift_canon();
    let tomb = canon.specs.get_mut("keyring-tombstones").unwrap();
    tomb.retain(|r| r.name.starts_with("A tombstone"));
    tomb.push(req(
        "Formal link",
        "See `spec:key-rotation § Rotation produces a new epoch key`.",
        &[],
    ));
    let rename = delta_of(
        "key-rotation",
        "## RENAMED Requirements\n\n- FROM: `### Requirement: Rotation produces a new epoch key`\n- TO: `### Requirement: Rotation mints a new epoch key`\n",
    );
    let findings = drift(&canon, vec![rename]);
    assert!(
        !findings
            .iter()
            .any(|f| matches!(f.kind, FindingKind::SiblingUsesOldName { .. })),
        "{findings:#?}"
    );
}

#[test]
fn common_phrases_do_not_produce_findings__ubiquitous_phrase() {
    let terms = terms_of("The tool MUST do it.", Some("Something else entirely."));
    let filtered =
        openspec_reviewer::drift::filter::filter_terms(terms, || Box::new(std::iter::empty()), 5);
    assert!(
        filtered.iter().all(|t| t.text != "the tool must"),
        "{filtered:?}"
    );
    let canon = drift_canon();
    let mut deltas = vec![fixture_delta()];
    let mut wide = canon.clone();
    for i in 0..6 {
        wide.specs.insert(
            format!("cap-{i}"),
            vec![req("R", "Until the grace window closes, wait.", &[])],
        );
    }
    let (p, _) = pairing_of(&wide, deltas.clone());
    let findings = drift_findings(&p, &wide, &deltas, 5);
    assert!(
        !findings.iter().any(|f| matches!(&f.kind, FindingKind::SiblingUsesRemoved { term, .. } if term.contains("grace window"))),
        "a phrase in more than max_common requirements is dropped: {findings:#?}"
    );
    deltas.clear();
}

#[test]
fn common_phrases_do_not_produce_findings__backticked_term_appears_everywhere() {
    let canon = drift_canon();
    let mut wide = canon.clone();
    for i in 0..10 {
        wide.specs.insert(
            format!("cap-{i}"),
            vec![req("R", "Carries `epoch: 'sealed'` too.", &[])],
        );
    }
    let deltas = vec![fixture_delta()];
    let (p, _) = pairing_of(&wide, deltas.clone());
    let findings = drift_findings(&p, &wide, &deltas, 5);
    let hits = findings
        .iter()
        .filter(|f| matches!(&f.kind, FindingKind::SiblingUsesRemoved { term, .. } if term == "epoch: 'sealed'"))
        .count();
    assert_eq!(hits, 11, "ten new siblings plus the fixture one");
}

#[test]
fn drift_findings_carry_their_locations__detail_pane() {
    let repo = Repo::new();
    repo.canon("key-rotation", &fixture("drift", "key-rotation.md"))
        .canon(
            "keyring-tombstones",
            &fixture("drift", "keyring-tombstones.md"),
        )
        .delta(
            "epoch-retire",
            "key-rotation",
            &fixture("drift", "delta.md"),
        );
    let out = run_in(repo.root(), &["change", "epoch-retire"]);
    let text = stdout(&out);
    assert!(text.contains("warning: sibling mentions removed term `epoch: 'sealed'`: keyring-tombstones § A tombstone names the epoch it closes"), "{text}");
    assert!(
        text.contains("        openspec/specs/keyring-tombstones/spec.md"),
        "{text}"
    );
    let json = run_in(repo.root(), &["--format", "json", "change", "epoch-retire"]);
    let doc: serde_json::Value = serde_json::from_str(&stdout(&json)).unwrap();
    let findings = &doc["changes"][0]["capabilities"][0]["pairings"][0]["findings"];
    let drift = findings
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["kind"] == "sibling_uses_removed")
        .unwrap();
    assert_eq!(drift["sibling"]["capability"], "keyring-tombstones");
    assert_eq!(
        drift["sibling"]["path"],
        "openspec/specs/keyring-tombstones/spec.md"
    );

    let snapshot = openspec_reviewer::source::ChangeSource {
        root: repo.root().to_path_buf(),
        name: "epoch-retire".into(),
    };
    use openspec_reviewer::source::Source;
    let review =
        openspec_reviewer::build::build_review(repo.root(), &snapshot.fetch().unwrap()).unwrap();
    let mut app = openspec_reviewer::render::tui::App::new(review, Default::default());
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
    terminal
        .draw(|f| openspec_reviewer::render::tui::draw(f, &mut app))
        .unwrap();
    let screen = format!("{:?}", terminal.backend().buffer());
    assert!(screen.contains("sibling mentions removed term"), "{screen}");
    assert!(
        screen.contains("openspec/specs/keyring-tombstones/spec.md"),
        "{screen}"
    );
}

#[test]
fn a_pairing_yields_the_terms_it_removes__word_dropped_from_a_deprecated_line() {
    let terms = terms_of(
        "The key that wraps content keys.\n\n- **Deprecated:** workspace key, rotation key",
        Some("The key that wraps content keys.\n\n- **Deprecated:** workspace key"),
    );
    assert!(
        terms.iter().all(|t| t.tier != Tier::Phrase),
        "a marker line is a list, not a sentence: {terms:?}"
    );
}

#[test]
fn a_removed_term_found_in_a_sibling_is_a_warning__sibling_names_the_phrase_on_a_deprecated_line() {
    let mut canon = drift_canon();
    canon.specs.insert(
        "definitions".into(),
        vec![req(
            "group key",
            "A spec MUST use `group key` to mean:\nThe key that wraps content keys.\n\n- **Deprecated:** grace window closes",
            &[],
        )],
    );
    let findings = drift(&canon, vec![fixture_delta()]);
    assert!(
        findings
            .iter()
            .all(|f| !f.message.contains("definitions § group key")),
        "a term that retires a word is not a sibling using it: {findings:#?}"
    );
}

#[test]
fn a_pairing_yields_the_terms_it_removes__word_dropped_from_a_deprecated_line_end_to_end() {
    let repo = Repo::new();
    repo.canon("definitions", &fixture("shielding", "definitions.md"))
        .canon("key-rotation", &fixture("shielding", "key-rotation.md"))
        .delta(
            "retire-rotation-key",
            "definitions",
            &fixture("shielding", "delta.md"),
        )
        .write(
            "openspec/changes/retire-rotation-key/proposal.md",
            "# retire-rotation-key\n",
        )
        .write("openspec/reviewer.toml", "[lint]\n");
    let text = stdout(&run_in(
        repo.root(),
        &["--findings-only", "change", "retire-rotation-key"],
    ));
    assert!(
        !text.contains("sibling mentions removed term"),
        "retiring a word from a Deprecated line is not a sentence rewrite: {text}"
    );
}
