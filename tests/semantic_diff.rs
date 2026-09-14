//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::model::DeltaKind;
use openspec_reviewer::review::normalize::paragraphs;
use openspec_reviewer::review::{
    diff_lines, diff_requirements, inline_view, pair_change, FindingKind, LineRole, ParaKind,
    ScenarioMatch, SpanMark,
};

fn pair(canon_text: &str, delta_text: &str) -> openspec_reviewer::review::Pairing {
    let canon = canon_of("alpha", canon_text);
    let delta = delta_of("alpha", delta_text);
    let review = pair_change(&change_of("c", vec![delta]), &canon, &[]);
    review.capabilities[0].pairings[0].clone()
}

#[test]
fn every_delta_entry_pairs_with_canon_by_name() {
    let p = pair(
        ALPHA_CANON,
        "## MODIFIED Requirements\n\n### Requirement: Index rows are ordered by path\n\nRows MUST be ordered by path.\n\n#### Scenario: Paths sort alphabetically\n\n- **WHEN** x\n- **THEN** y\n",
    );
    assert!(p.before.as_ref().unwrap().body.contains("alphabetically"));
    assert_eq!(
        p.after.as_ref().unwrap().body,
        "Rows MUST be ordered by path."
    );
}

#[test]
fn every_delta_entry_pairs_with_canon_by_name__same_name_in_a_different_capability() {
    let mut canon = canon_of("beta", ALPHA_CANON);
    canon.specs.insert("alpha".into(), Vec::new());
    let delta = delta_of(
        "alpha",
        "## MODIFIED Requirements\n\n### Requirement: Index rows are ordered by path\n\nBody.\n\n#### Scenario: S\n\n- **WHEN** x\n- **THEN** y\n",
    );
    let review = pair_change(&change_of("c", vec![delta]), &canon, &[]);
    let p = &review.capabilities[0].pairings[0];
    assert!(p.before.is_none());
    assert!(p
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::ModifiedWithoutCanon));
}

#[test]
fn text_is_normalized_before_comparison() {
    let review = review_fixture("rewrap", "key-rotation");
    let p = &review.capabilities[0].pairings[0];
    assert!(!p.diff.changed, "a pure re-wrap shows as unchanged");
    assert!(inline_view(p).iter().all(|l| l.kind == ParaKind::Equal));
}

#[test]
fn text_is_normalized_before_comparison__list_items_stay_separate() {
    let paras =
        paragraphs("- **WHEN** a route\n  mounts a page\n- **THEN** the index\n  shows one row");
    assert_eq!(
        paras,
        [
            "- **WHEN** a route mounts a page",
            "- **THEN** the index shows one row"
        ]
    );
    let a = req("R", "", &[("S", "- **WHEN** one\n  two\n- **THEN** three")]);
    let b = req(
        "R",
        "",
        &[("S", "- **WHEN** one two\n- **THEN** three\n  changed")],
    );
    let d = diff_requirements(&a, &b);
    let lines = d.scenarios[0].lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].kind, ParaKind::Equal);
    assert_eq!(lines[1].kind, ParaKind::Changed);
}

#[test]
fn a_modified_requirement_shows_a_word_level_diff() {
    let review = review_fixture("sweep-gate", "key-rotation");
    let p = review.capabilities[0]
        .pairings
        .iter()
        .find(|p| p.kind == DeltaKind::Modified)
        .expect("the MODIFIED sweep requirement");
    let changed: Vec<_> = p
        .diff
        .body
        .iter()
        .filter(|l| l.kind == ParaKind::Changed)
        .collect();
    assert_eq!(changed.len(), 1, "one paragraph changes by a few words");
    assert_eq!(
        p.diff
            .body
            .iter()
            .filter(|l| l.kind == ParaKind::Added)
            .count(),
        2,
        "two paragraphs join it"
    );
    let added: String = changed[0]
        .spans
        .iter()
        .filter(|s| s.mark == SpanMark::Added)
        .map(|s| s.text.as_str())
        .collect();
    assert!(added.contains("revocation"), "{added}");
    assert!(changed[0]
        .spans
        .iter()
        .any(|s| s.mark == SpanMark::Equal && s.text.contains("Migrating existing documents")));
}

#[test]
fn a_modified_requirement_shows_a_word_level_diff__paragraph_inserted() {
    let before = req("R", "First paragraph.\n\nThird paragraph.", &[]);
    let after = req(
        "R",
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.",
        &[],
    );
    let d = diff_requirements(&before, &after);
    let kinds: Vec<ParaKind> = d.body.iter().map(|l| l.kind).collect();
    assert_eq!(kinds, [ParaKind::Equal, ParaKind::Added, ParaKind::Equal]);
    assert_eq!(d.body[1].text(), "Second paragraph.");
}

#[test]
fn scenarios_are_matched_by_name() {
    let before = req(
        "R",
        "B",
        &[("Feature mount appears", "- **WHEN** a\n- **THEN** b")],
    );
    let after = req(
        "R",
        "B",
        &[("Module mount appears", "- **WHEN** a\n- **THEN** b")],
    );
    let d = diff_requirements(&before, &after);
    assert!(
        matches!(&d.scenarios[0], ScenarioMatch::Added { name, .. } if name == "Module mount appears")
    );
    assert!(
        matches!(&d.scenarios[1], ScenarioMatch::Removed { name, .. } if name == "Feature mount appears")
    );
    assert!(d
        .scenarios
        .iter()
        .all(|s| s.lines().iter().all(|l| l.kind != ParaKind::Changed)));
}

#[test]
fn scenarios_are_matched_by_name__scenarios_reordered() {
    let before = req("R", "B", &[("One", "- a"), ("Two", "- b")]);
    let after = req("R", "B", &[("Two", "- b"), ("One", "- a")]);
    assert!(!diff_requirements(&before, &after).changed);
}

#[test]
fn an_added_requirement_shows_its_full_text() {
    let p = pair(
        ALPHA_CANON,
        "## ADDED Requirements\n\n### Requirement: Brand new\n\nBody here.\n\n#### Scenario: One\n\n- a\n\n#### Scenario: Two\n\n- b\n",
    );
    let lines = inline_view(&p);
    assert!(lines
        .iter()
        .filter(|l| !l.spans.is_empty())
        .all(|l| l.kind == ParaKind::Added));
    assert!(lines
        .iter()
        .any(|l| l.role == LineRole::Scenario && l.text().contains("Two")));
}

#[test]
fn a_removed_requirement_shows_the_canon_text() {
    let p = pair(
        ALPHA_CANON,
        "## REMOVED Requirements\n\n### Requirement: Index rows are ordered by path\n\n**Reason**: gone.\n",
    );
    assert_eq!(p.kind, DeltaKind::Removed);
    let lines = inline_view(&p);
    assert!(lines
        .iter()
        .filter(|l| !l.spans.is_empty())
        .all(|l| l.kind == ParaKind::Removed));
    assert!(
        lines
            .iter()
            .any(|l| l.text().contains("alphabetically by path")),
        "canon body shown"
    );
    assert!(
        !lines.iter().any(|l| l.text().contains("Reason")),
        "delta's own text is not shown"
    );
}

#[test]
fn a_rename_shows_both_names_and_the_body_diff() {
    let review = review_fixture("rename-modified", "keyring-tombstones");
    let p = &review.capabilities[0].pairings[0];
    let lines = inline_view(p);
    assert_eq!(lines[0].kind, ParaKind::Removed);
    assert!(lines[0].text().contains("Clients act on the outcome"));
    assert_eq!(lines[1].kind, ParaKind::Added);
    assert!(lines[1].text().contains("Clients dispatch on the outcome"));
    let changed = lines
        .iter()
        .find(|l| l.kind == ParaKind::Changed)
        .expect("body diff");
    assert!(changed
        .spans
        .iter()
        .any(|s| s.mark == SpanMark::Removed && s.text.contains("alone")));
    assert!(changed
        .spans
        .iter()
        .any(|s| s.mark == SpanMark::Added && s.text.contains("untouched")));
}

#[test]
fn artefacts_are_shown_as_line_diffs() {
    let before = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\n";
    let after =
        "line 1\nline 2\nline 3\nline 4 changed\nline 5\nline 6\nline 7\nline 8 changed\nline 9\n";
    let lines = diff_lines(before, after);
    assert_eq!(
        lines.iter().filter(|l| l.kind == ParaKind::Removed).count(),
        2
    );
    assert_eq!(
        lines.iter().filter(|l| l.kind == ParaKind::Added).count(),
        2
    );
    assert!(
        lines.iter().any(|l| l.kind == ParaKind::Equal),
        "context lines present"
    );
}

#[test]
fn artefacts_are_shown_as_line_diffs__review_from_the_working_tree() {
    let a = openspec_reviewer::review::ArtefactReview {
        artefact: openspec_reviewer::model::Artefact {
            name: "tasks.md".into(),
            before: None,
            after: Some("- [ ] 1\n- [ ] 2\n".into()),
        },
        state: Default::default(),
    };
    let lines = a.lines();
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().all(|l| l.kind == ParaKind::Equal));
}
