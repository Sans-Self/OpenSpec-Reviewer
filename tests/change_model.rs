//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::model::{
    load_change, parse_canon_spec, parse_delta_spec, ChangeError, DeltaKind, ParseError,
};
use openspec_reviewer::source::{load_canon, ChangeSource, Source, SourceError};

const TWO_SCENARIOS: &str = "\
# alpha

## Requirements

### Requirement: One

Body of one.

#### Scenario: First

- **WHEN** a
- **THEN** b

#### Scenario: Second

- **WHEN** c
- **THEN** d

### Requirement: Two

Body of two.

#### Scenario: Only

- **WHEN** e
- **THEN** f
";

#[test]
fn a_spec_file_parses_into_requirements_and_scenarios() {
    let reqs = parse_canon_spec(TWO_SCENARIOS);
    assert_eq!(reqs.len(), 2);
    assert_eq!(reqs[0].name, "One");
    assert_eq!(reqs[0].body, "Body of one.");
    assert_eq!(reqs[0].scenarios.len(), 2);
    assert_eq!(reqs[0].scenarios[0].name, "First");
    assert_eq!(reqs[0].scenarios[0].body, "- **WHEN** a\n- **THEN** b");
    assert_eq!(reqs[0].scenarios[1].name, "Second");
}

#[test]
fn a_spec_file_parses_into_requirements_and_scenarios__body_text_stops_at_the_next_requirement() {
    let reqs = parse_canon_spec(TWO_SCENARIOS);
    assert!(!reqs[0].body.contains("two"));
    assert!(reqs[0].scenarios.iter().all(|s| !s.body.contains("e\n")));
    assert_eq!(reqs[1].body, "Body of two.");
    assert_eq!(reqs[1].scenarios.len(), 1);
}

#[test]
fn canon_is_every_spec_under_the_specs_directory() {
    let repo = Repo::new();
    repo.canon("alpha", TWO_SCENARIOS)
        .canon("beta", "### Requirement: Beta one\n\nBody.\n");
    let canon = load_canon(repo.root()).unwrap();
    assert_eq!(
        canon.specs.keys().cloned().collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
    assert_eq!(canon.specs["alpha"].len(), 2);
    assert_eq!(canon.specs["beta"][0].name, "Beta one");
}

#[test]
fn canon_is_every_spec_under_the_specs_directory__no_specs_directory() {
    let dir = tempfile::tempdir().unwrap();
    let err = load_canon(dir.path()).unwrap_err();
    assert!(
        err.to_string().contains("found no OpenSpec canon here"),
        "{err}"
    );
}

#[test]
fn a_delta_groups_requirements_by_kind() {
    let text = "\
## ADDED Requirements

### Requirement: New

Body.

#### Scenario: S

- **WHEN** x
- **THEN** y

## MODIFIED Requirements

### Requirement: Changed one

Body.

### Requirement: Changed two

Body.
";
    let delta = parse_delta_spec("alpha", "alpha/spec.md", text).unwrap();
    let kinds: Vec<&DeltaKind> = delta.entries.iter().map(|e| &e.kind).collect();
    assert_eq!(
        kinds,
        [
            &DeltaKind::Added,
            &DeltaKind::Modified,
            &DeltaKind::Modified
        ]
    );
    assert_eq!(delta.entries[0].requirement.scenarios.len(), 1);
}

#[test]
fn a_delta_groups_requirements_by_kind__unknown_section() {
    let text = "## Changed Requirements\n\n### Requirement: X\n\nBody.\n";
    let err =
        parse_delta_spec("alpha", "openspec/changes/foo/specs/alpha/spec.md", text).unwrap_err();
    match &err {
        ParseError::UnknownSection { file, heading, .. } => {
            assert_eq!(file, "openspec/changes/foo/specs/alpha/spec.md");
            assert_eq!(heading, "Changed Requirements");
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(err.to_string().contains("Changed Requirements"));
}

#[test]
fn a_rename_is_a_pair_of_names() {
    let text = "\
## RENAMED Requirements

- FROM: `### Requirement: A`
- TO: `### Requirement: B`
";
    let delta = parse_delta_spec("alpha", "f", text).unwrap();
    assert_eq!(delta.entries.len(), 1);
    assert_eq!(
        delta.entries[0].kind,
        DeltaKind::Renamed { from: "A".into() }
    );
    assert_eq!(delta.entries[0].requirement.name, "B");
}

#[test]
fn a_rename_is_a_pair_of_names__orphan_from() {
    let text = "## RENAMED Requirements\n\n- FROM: `### Requirement: A`\n\n## ADDED Requirements\n";
    let err = parse_delta_spec("alpha", "f.md", text).unwrap_err();
    assert_eq!(
        err,
        ParseError::OrphanFrom {
            file: "f.md".into(),
            line: 3
        }
    );
    let err = parse_delta_spec(
        "alpha",
        "f.md",
        "## RENAMED Requirements\n- TO: `### Requirement: B`\n",
    )
    .unwrap_err();
    assert!(matches!(err, ParseError::OrphanTo { line: 2, .. }));
}

#[test]
fn a_rename_joins_the_modified_entry_with_the_new_name() {
    let review = review_fixture("rename-modified", "keyring-tombstones");
    let pairings = &review.capabilities[0].pairings;
    assert_eq!(pairings.len(), 1, "RENAMED and MODIFIED become one entry");
    let p = &pairings[0];
    assert_eq!(
        p.name,
        "Clients dispatch on the outcome, never on URI matching"
    );
    assert_eq!(
        p.kind,
        DeltaKind::Renamed {
            from: "Clients act on the outcome, never on URI matching".into()
        }
    );
    assert!(p.after.as_ref().unwrap().body.contains("state untouched"));
    assert!(p.before.as_ref().unwrap().body.contains("state alone"));
}

#[test]
fn a_rename_joins_the_modified_entry_with_the_new_name__rename_only() {
    let canon = canon_of("alpha", ALPHA_CANON);
    let delta = delta_of(
        "alpha",
        "## RENAMED Requirements\n\n- FROM: `### Requirement: Index rows are ordered by path`\n- TO: `### Requirement: Rows are ordered by path`\n",
    );
    let review = openspec_reviewer::review::pair_change(&change_of("c", vec![delta]), &canon, &[]);
    let p = &review.capabilities[0].pairings[0];
    assert_eq!(p.name, "Rows are ordered by path");
    let after = p.after.as_ref().unwrap();
    let before = p.before.as_ref().unwrap();
    assert_eq!(after.body, before.body);
    assert_eq!(after.scenarios, before.scenarios);
    assert!(!p.diff.changed);
}

#[test]
fn a_change_is_its_artefacts_plus_its_deltas() {
    let snap = snapshot(vec![
        ("openspec/changes/foo/proposal.md", None, Some("p")),
        ("openspec/changes/foo/design.md", None, Some("d")),
        ("openspec/changes/foo/tasks.md", None, Some("t")),
        (
            "openspec/changes/foo/.openspec.yaml",
            None,
            Some("schema: x"),
        ),
        (
            "openspec/changes/foo/specs/alpha/spec.md",
            None,
            Some("## ADDED Requirements\n\n### Requirement: A\n\nB.\n"),
        ),
        (
            "openspec/changes/foo/specs/beta/spec.md",
            None,
            Some("## ADDED Requirements\n\n### Requirement: C\n\nD.\n"),
        ),
    ]);
    let changes = load_change(&snap).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].artefacts.len(), 3);
    assert_eq!(changes[0].deltas.len(), 2);
    assert_eq!(changes[0].deltas[1].capability, "beta");

    let empty = snapshot(vec![(
        "openspec/changes/foo/.openspec.yaml",
        None,
        Some(""),
    )]);
    assert!(matches!(load_change(&empty), Err(ChangeError::Empty(_))));
}

#[test]
fn a_change_is_its_artefacts_plus_its_deltas__change_name_under_archive() {
    let repo = Repo::new();
    repo.archive("2026-01-01-old-change", "alpha", "## ADDED Requirements\n");
    let err = ChangeSource {
        root: repo.root().to_path_buf(),
        name: "old-change".into(),
    }
    .fetch()
    .unwrap_err();
    assert!(
        matches!(err, SourceError::ArchivedChange(ref n) if n == "old-change"),
        "{err}"
    );
    assert!(err.to_string().contains("archived"));
}
