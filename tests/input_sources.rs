//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::build::build_review;
use openspec_reviewer::source::{
    parse_diff, ChangeSource, DiffSource, GitSource, Source, SourceError,
};
use std::path::Path;

const DELTA: &str = "\
## MODIFIED Requirements

### Requirement: Index rows are ordered by path

Rows MUST be ordered alphabetically by path, orphan pages last.

#### Scenario: Paths sort alphabetically

- **WHEN** the index renders routes
- **THEN** the rows appear in path order
";

fn branch_repo() -> Repo {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON).init_git("main");
    repo.git(&["checkout", "-q", "-b", "feature/foo"]);
    repo.delta("foo", "alpha", DELTA)
        .write("openspec/changes/foo/proposal.md", "# foo\n");
    repo.commit("add change foo");
    repo.git(&["checkout", "-q", "main"]);
    repo
}

#[test]
fn every_source_yields_the_same_snapshot() {
    let repo = branch_repo();
    let via_git = GitSource {
        root: repo.root().into(),
        reference: "feature/foo".into(),
        base: None,
    }
    .fetch()
    .unwrap();
    let patch = repo.git(&["diff", "main...feature/foo"]);
    let via_diff = parse_diff(repo.root(), &patch, "diff".into()).unwrap();
    let mut a = via_git.files.clone();
    let mut b = via_diff.files.clone();
    a.sort_by(|x, y| x.path.cmp(&y.path));
    b.sort_by(|x, y| x.path.cmp(&y.path));
    assert_eq!(a, b);

    let r1 = build_review(repo.root(), &via_git).unwrap();
    let r2 = build_review(repo.root(), &via_diff).unwrap();
    let names = |r: &openspec_reviewer::review::Review| {
        r.pairings()
            .map(|p| (p.key(), p.findings.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&r1), names(&r2));
}

#[test]
fn each_source_is_a_subcommand() {
    let repo = Repo::new();
    let out = run_in(repo.root(), &[]);
    let text = format!("{}{}", stdout(&out), stderr(&out));
    for sub in ["change", "diff", "git", "gh"] {
        assert!(text.contains(sub), "help lacks {sub}: {text}");
    }
    assert_eq!(out.status.code(), Some(2), "usage status");
}

#[test]
fn canon_always_comes_from_the_working_directory() {
    let repo = branch_repo();
    repo.git(&["checkout", "-q", "feature/foo"]);
    repo.canon(
        "alpha",
        &ALPHA_CANON.replace("every route", "every single route"),
    );
    repo.commit("edit canon on the branch");
    repo.git(&["checkout", "-q", "main"]);

    let snap = GitSource {
        root: repo.root().into(),
        reference: "feature/foo".into(),
        base: None,
    }
    .fetch()
    .unwrap();
    let review = build_review(repo.root(), &snap).unwrap();
    let p = review.pairings().next().unwrap();
    assert_eq!(
        p.before.as_ref().unwrap().body,
        "Rows MUST be ordered alphabetically by path.",
        "working-directory canon"
    );
    assert_eq!(review.canon_edits.len(), 1);
    assert_eq!(review.canon_edits[0].path, "openspec/specs/alpha/spec.md");
    assert!(
        review.canon_edits[0]
            .lines
            .iter()
            .any(|l| l.text().contains("every single route")),
        "branch canon edit shown as a line diff"
    );
}

#[test]
fn the_change_source_reads_the_working_tree() {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("sweep-gate", "alpha", DELTA)
        .write("openspec/changes/sweep-gate/tasks.md", "- [ ] 1\n");
    let snap = ChangeSource {
        root: repo.root().into(),
        name: "sweep-gate".into(),
    }
    .fetch()
    .unwrap();
    assert_eq!(snap.files.len(), 2);
    assert!(snap
        .files
        .iter()
        .all(|f| f.before.is_none() && f.after.is_some()));
    let review = build_review(repo.root(), &snap).unwrap();
    assert_eq!(review.changes[0].artefacts[0].artefact.name, "tasks.md");
    assert_eq!(review.pairings().count(), 1);
}

#[test]
fn the_change_source_reads_the_working_tree__unknown_change() {
    let repo = Repo::new();
    repo.delta("other", "alpha", DELTA);
    let err = ChangeSource {
        root: repo.root().into(),
        name: "nope".into(),
    }
    .fetch()
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("openspec/changes/nope"), "{msg}");
    assert!(msg.contains("other"), "{msg}");
}

#[test]
fn the_diff_source_applies_a_unified_diff_to_the_working_tree() {
    let repo = branch_repo();
    let patch = repo.git(&["diff", "main...feature/foo"]);
    let snap = parse_diff(repo.root(), &patch, "diff".into()).unwrap();
    let spec = snap
        .files
        .iter()
        .find(|f| f.path == Path::new("openspec/changes/foo/specs/alpha/spec.md"))
        .expect("created delta in snapshot");
    assert_eq!(spec.before, None);
    assert_eq!(spec.after.as_deref(), Some(DELTA));
    let review = build_review(repo.root(), &snap).unwrap();
    assert_eq!(review.changes[0].name, "foo");
}

#[test]
fn the_diff_source_applies_a_unified_diff_to_the_working_tree__pre_image_mismatch() {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON)
        .delta("foo", "alpha", DELTA)
        .init_git("main");
    repo.write(
        "openspec/changes/foo/specs/alpha/spec.md",
        &DELTA.replace("orphan pages last", "orphans last"),
    );
    let patch = repo.git(&["diff"]);
    repo.write(
        "openspec/changes/foo/specs/alpha/spec.md",
        "## MODIFIED Requirements\n\nsomething else entirely\n",
    );
    let err = parse_diff(repo.root(), &patch, "diff".into()).unwrap_err();
    match &err {
        SourceError::HunkMismatch { file, hunk } => {
            assert_eq!(file, "openspec/changes/foo/specs/alpha/spec.md");
            assert!(hunk.starts_with("@@ -"), "{hunk}");
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn the_diff_source_applies_a_unified_diff_to_the_working_tree__no_openspec_content() {
    let repo = Repo::new();
    repo.write("README.md", "hello\n").init_git("main");
    repo.write("README.md", "hello world\n");
    let patch = repo.git(&["diff"]);
    let err = parse_diff(repo.root(), &patch, "diff".into()).unwrap_err();
    assert!(matches!(err, SourceError::NoOpenSpecContent));
}

#[test]
fn the_diff_source_reads_stdin_when_the_path_is_dash() {
    let repo = branch_repo();
    let patch = repo.git(&["diff", "main...feature/foo"]);
    let path = repo.root().join("pr.patch");
    std::fs::write(&path, &patch).unwrap();
    let snap = DiffSource {
        root: repo.root().into(),
        path: Some(path),
    }
    .fetch()
    .unwrap();
    assert!(snap.origin.starts_with("diff "));
    assert_eq!(snap.files.len(), 2);
}

#[test]
fn the_git_source_reads_two_refs() {
    let repo = branch_repo();
    let snap = GitSource {
        root: repo.root().into(),
        reference: "feature/foo".into(),
        base: None,
    }
    .fetch()
    .unwrap();
    assert_eq!(snap.origin, "git feature/foo against main");
    let spec = snap
        .files
        .iter()
        .find(|f| f.path.ends_with("specs/alpha/spec.md"))
        .unwrap();
    assert_eq!(spec.before, None, "added on the branch: no main side");
    assert_eq!(spec.after.as_deref(), Some(DELTA));
}

#[test]
fn the_git_source_reads_two_refs__repository_with_master() {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON).init_git("master");
    repo.git(&["checkout", "-q", "-b", "feature/foo"]);
    repo.delta("foo", "alpha", DELTA).commit("foo");
    let snap = GitSource {
        root: repo.root().into(),
        reference: "feature/foo".into(),
        base: None,
    }
    .fetch()
    .unwrap();
    assert!(snap.origin.ends_with("against master"));
}

#[test]
fn the_git_source_reads_two_refs__neither_default_base_exists() {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON).init_git("trunk");
    let err = GitSource {
        root: repo.root().into(),
        reference: "trunk".into(),
        base: None,
    }
    .fetch()
    .unwrap_err();
    assert!(matches!(err, SourceError::NoBase));
    assert!(err.to_string().contains("--base"));
}

#[test]
fn the_git_source_reads_two_refs__unknown_ref() {
    let repo = branch_repo();
    let err = GitSource {
        root: repo.root().into(),
        reference: "nope".into(),
        base: None,
    }
    .fetch()
    .unwrap_err();
    match err {
        SourceError::Git { stderr } => assert!(!stderr.is_empty()),
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn the_gh_source_reads_a_pull_request__gh_missing() {
    let repo = Repo::new();
    let out = std::process::Command::new(exe())
        .args(["gh", "1"])
        .current_dir(repo.root())
        .env("PATH", repo.root())
        .output()
        .unwrap();
    assert!(
        stderr(&out).contains("needs the `gh` CLI"),
        "{}",
        stderr(&out)
    );
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn canon_files_in_a_snapshot_are_shown_as_plain_diffs() {
    let repo = Repo::new();
    repo.canon("alpha", ALPHA_CANON);
    let edited = ALPHA_CANON.replace("Rows MUST be ordered", "Rows MUST always be ordered");
    let snap = snapshot(vec![
        (
            "openspec/specs/alpha/spec.md",
            Some(ALPHA_CANON),
            Some(&edited),
        ),
        (
            "openspec/changes/foo/specs/alpha/spec.md",
            None,
            Some(DELTA),
        ),
    ]);
    let review = build_review(repo.root(), &snap).unwrap();
    assert_eq!(review.canon_edits.len(), 1);
    assert!(review.canon_edits[0].lines.iter().any(|l| l.kind
        == openspec_reviewer::review::ParaKind::Added
        && l.text().contains("always")));
    assert_eq!(
        review.pairings().count(),
        1,
        "no pairing is made from the canon file"
    );
}
