//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::build::build_review;
use openspec_reviewer::model::DeltaSpec;
use openspec_reviewer::review::{pair_change, FindingKind, Severity, Summary};

fn findings(
    canon_text: &str,
    delta_text: &str,
    others: &[(String, Vec<DeltaSpec>)],
) -> Vec<openspec_reviewer::review::Finding> {
    let canon = canon_of("alpha", canon_text);
    let delta = delta_of("alpha", delta_text);
    let review = pair_change(&change_of("under-review", vec![delta]), &canon, others);
    review.findings().cloned().collect()
}

const WITH_SCENARIO: &str = "\n\n#### Scenario: S\n\n- **WHEN** x\n- **THEN** y\n";

#[test]
fn a_finding_has_a_severity_a_location_and_a_message() {
    let fs = findings(
        ALPHA_CANON,
        &format!("## MODIFIED Requirements\n\n### Requirement: Flat index of all routes\n\nBody.{WITH_SCENARIO}"),
        &[],
    );
    let f = &fs[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(f.location.change, "under-review");
    assert_eq!(f.location.capability, "alpha");
    assert_eq!(f.location.requirement, "Flat index of all routes");
    assert!(!f.message.contains('\n'));
    assert_eq!(
        f.kind.severity(),
        FindingKind::ModifiedWithoutCanon.severity(),
        "one kind, one severity"
    );
}

#[test]
fn a_modified_requirement_must_exist_in_canon() {
    let canon = canon_of("alpha", ALPHA_CANON);
    let delta = delta_of(
        "alpha",
        &format!("## MODIFIED Requirements\n\n### Requirement: Flat index of all routes\n\nBody.{WITH_SCENARIO}"),
    );
    let review = pair_change(&change_of("c", vec![delta]), &canon, &[]);
    let p = &review.capabilities[0].pairings[0];
    let f = p
        .findings
        .iter()
        .find(|f| f.kind == FindingKind::ModifiedWithoutCanon)
        .unwrap();
    assert!(f.message.contains("Flat index of all routes"));
    assert!(openspec_reviewer::review::inline_view(p)
        .iter()
        .filter(|l| !l.spans.is_empty())
        .all(|l| l.kind == openspec_reviewer::review::ParaKind::Added));
}

#[test]
fn an_added_requirement_must_be_new() {
    let canon = canon_of("alpha", ALPHA_CANON);
    let delta = delta_of(
        "alpha",
        &format!("## ADDED Requirements\n\n### Requirement: Index rows are ordered by path\n\nRows MUST be ordered by title.{WITH_SCENARIO}"),
    );
    let review = pair_change(&change_of("c", vec![delta]), &canon, &[]);
    let p = &review.capabilities[0].pairings[0];
    assert!(p
        .findings
        .iter()
        .any(|f| f.kind == FindingKind::AddedAlreadyExists && f.severity == Severity::Error));
    assert!(
        p.diff
            .body
            .iter()
            .any(|l| l.kind == openspec_reviewer::review::ParaKind::Changed),
        "word diff against existing text"
    );
}

#[test]
fn a_removed_or_renamed_requirement_must_exist_in_canon() {
    let fs = findings(
        ALPHA_CANON,
        "## REMOVED Requirements\n\n### Requirement: Nope\n\nReason.\n",
        &[],
    );
    assert!(fs
        .iter()
        .any(|f| f.kind == FindingKind::RemovedWithoutCanon));

    let fs = findings(
        ALPHA_CANON,
        "## RENAMED Requirements\n\n- FROM: `### Requirement: Missing`\n- TO: `### Requirement: Whatever`\n",
        &[],
    );
    let f = fs
        .iter()
        .find(|f| matches!(f.kind, FindingKind::RenameSourceMissing { .. }))
        .unwrap();
    assert!(f.message.contains("Missing"), "{}", f.message);
}

#[test]
fn a_removed_or_renamed_requirement_must_exist_in_canon__rename_onto_an_existing_name() {
    let fs = findings(
        ALPHA_CANON,
        "## RENAMED Requirements\n\n- FROM: `### Requirement: Index rows are ordered by path`\n- TO: `### Requirement: Flat index of all routes and pages`\n",
        &[],
    );
    let f = fs
        .iter()
        .find(|f| f.kind == FindingKind::RenameTargetTaken)
        .unwrap();
    assert!(f.message.contains("Flat index of all routes and pages"));
    assert_eq!(f.severity, Severity::Error);
}

#[test]
fn a_dropped_scenario_is_a_warning() {
    let fs = findings(
        ALPHA_CANON,
        "## MODIFIED Requirements\n\n### Requirement: Flat index of all routes and pages\n\nThe dashboard MUST offer an index view.\n\n#### Scenario: Route-mounted page appears as a row\n\n- a\n\n#### Scenario: Orphan page appears without a path\n\n- b\n",
        &[],
    );
    let f = fs
        .iter()
        .find(|f| matches!(f.kind, FindingKind::ScenarioDropped { .. }))
        .unwrap();
    assert_eq!(f.severity, Severity::Warning);
    assert!(f.message.contains("Feature mount appears"), "{}", f.message);
    assert_eq!(
        f.location.scenario.as_deref(),
        Some("Feature mount appears")
    );
}

#[test]
fn a_requirement_without_scenarios_is_a_warning() {
    let fs = findings(
        ALPHA_CANON,
        "## ADDED Requirements\n\n### Requirement: Bare\n\nJust a body.\n",
        &[],
    );
    let f = fs
        .iter()
        .find(|f| f.kind == FindingKind::RequirementWithoutScenario)
        .unwrap();
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location.requirement, "Bare");
}

#[test]
fn a_collision_with_another_open_change_is_a_warning() {
    let other = delta_of(
        "alpha",
        &format!("## MODIFIED Requirements\n\n### Requirement: Index rows are ordered by path\n\nOther body.{WITH_SCENARIO}"),
    );
    let fs = findings(
        ALPHA_CANON,
        &format!("## MODIFIED Requirements\n\n### Requirement: Index rows are ordered by path\n\nMy body.{WITH_SCENARIO}"),
        &[("other-change".to_string(), vec![other])],
    );
    let f = fs
        .iter()
        .find(|f| matches!(f.kind, FindingKind::CrossChangeCollision { .. }))
        .unwrap();
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.message, "also touched by other-change");
}

#[test]
fn a_collision_with_another_open_change_is_a_warning__other_change_is_archived() {
    let repo = Repo::new();
    let delta = format!("## MODIFIED Requirements\n\n### Requirement: Index rows are ordered by path\n\nMy body.{WITH_SCENARIO}");
    repo.canon("alpha", ALPHA_CANON)
        .delta("mine", "alpha", &delta)
        .archive("2026-01-01-old", "alpha", &delta);
    let snap = snapshot(vec![(
        "openspec/changes/mine/specs/alpha/spec.md",
        None,
        Some(&delta),
    )]);
    let review = build_review(repo.root(), &snap).unwrap();
    assert!(!review.pairings().any(|p| p
        .findings
        .iter()
        .any(|f| matches!(f.kind, FindingKind::CrossChangeCollision { .. }))));
}

#[test]
fn an_unchanged_modification_is_a_note() {
    let review = review_fixture("rewrap", "key-rotation");
    let p = &review.capabilities[0].pairings[0];
    let f = p
        .findings
        .iter()
        .find(|f| f.kind == FindingKind::UnchangedModified)
        .unwrap();
    assert_eq!(f.severity, Severity::Note);
    assert_eq!(f.message, "no change");
    assert!(!p.diff.changed);
}

#[test]
fn exit_status_reflects_the_worst_finding() {
    assert_eq!(
        Summary {
            errors: 0,
            warnings: 2,
            notes: 0
        }
        .exit_code(),
        1
    );
    assert_eq!(
        Summary {
            errors: 1,
            warnings: 2,
            notes: 0
        }
        .exit_code(),
        2
    );
    assert_eq!(
        Summary {
            errors: 0,
            warnings: 0,
            notes: 3
        }
        .exit_code(),
        0
    );

    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("bare", "alpha", "## ADDED Requirements\n\n### Requirement: Bare\n\nBody.\n\n### Requirement: Also bare\n\nBody.\n");
    let out = run_in(repo.root(), &["--plain", "--no-state", "change", "bare"]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
}
