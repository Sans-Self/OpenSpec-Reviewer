//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::source::complete::{change_names, git_refs};
use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// What the shell does on a tab press: hand the binary the command line so
/// far and read the candidates off stdout.
fn candidates_for(root: &Path, path: &str, line: &[&str]) -> Output {
    Command::new(exe())
        .env("COMPLETE", "fish")
        .env("PATH", path)
        .args(["--", "openspec-reviewer"])
        .args(line)
        .current_dir(root)
        .output()
        .expect("binary runs")
}

fn values(out: &Output) -> Vec<String> {
    stdout(out)
        .lines()
        .map(|line| line.split('\t').next().unwrap_or("").to_string())
        .collect()
}

/// A `PATH` whose `gh` is this script. The system directories come after it,
/// so the script still has `sleep` and the real `gh` stays shadowed.
fn stub_gh(repo: &Repo, body: &str) -> String {
    let bin = repo.root().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let path = bin.join("gh");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    format!("{}:/bin:/usr/bin", bin.display())
}

#[test]
fn the_binary_prints_a_completion_stub_for_its_shell__fish_stub() {
    let repo = Repo::new();
    let out = Command::new(exe())
        .env("COMPLETE", "fish")
        .current_dir(repo.root())
        .output()
        .expect("binary runs");
    assert_eq!(out.status.code(), Some(0));
    assert!(
        stdout(&out).contains("openspec-reviewer"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("complete"), "{}", stdout(&out));
}

#[test]
fn the_binary_prints_a_completion_stub_for_its_shell__unsupported_shell() {
    let repo = Repo::new();
    let out = Command::new(exe())
        .env("COMPLETE", "elvish")
        .current_dir(repo.root())
        .output()
        .expect("binary runs");
    assert_ne!(out.status.code(), Some(0));
    let err = stderr(&out);
    for shell in ["bash", "zsh", "fish"] {
        assert!(err.contains(shell), "{err}");
    }
}

#[test]
fn the_binary_prints_a_completion_stub_for_its_shell__unset_variable() {
    let repo = Repo::new();
    let out = run_in(repo.root(), &["change", "foo"]);
    assert!(stderr(&out).contains("no change at"), "{}", stderr(&out));
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn fixed_words_complete_from_the_command_tree__format_values() {
    let repo = Repo::new();
    let out = candidates_for(repo.root(), "", &["--format", ""]);
    assert_eq!(values(&out), vec!["text", "json", "markdown"]);
}

#[test]
fn a_change_name_completes_from_the_repository__two_changes_and_the_archive() {
    let repo = Repo::new();
    repo.delta("beta", "alpha", "## ADDED Requirements\n");
    repo.delta("alpha", "alpha", "## ADDED Requirements\n");
    repo.archive("gamma", "alpha", "## ADDED Requirements\n");
    let names = change_names(repo.root())
        .into_iter()
        .map(|c| c.get_value().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["alpha", "beta"]);
}

#[test]
fn a_change_name_completes_from_the_repository__not_in_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    assert!(change_names(dir.path()).is_empty());

    let out = candidates_for(dir.path(), "", &["change", ""]);
    assert!(stderr(&out).is_empty(), "{}", stderr(&out));
    assert!(
        values(&out).iter().all(|v| v.starts_with('-')),
        "{}",
        stdout(&out)
    );
}

#[test]
fn a_git_reference_completes_from_git__a_branch_and_a_tag() {
    let repo = Repo::new();
    // Signing is off: the machine running the tests may have it on globally,
    // and a test must not reach for a key.
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .current_dir(repo.root())
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    };
    git(&["init", "--initial-branch=main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["commit", "--allow-empty", "-m", "root"]);
    git(&["tag", "v0.2.0"]);

    let refs = git_refs(repo.root())
        .into_iter()
        .map(|c| c.get_value().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(refs.contains(&"main".to_string()), "{refs:?}");
    assert!(refs.contains(&"v0.2.0".to_string()), "{refs:?}");
}

#[test]
fn a_git_reference_completes_from_git__no_repository() {
    let dir = tempfile::tempdir().unwrap();
    assert!(git_refs(dir.path()).is_empty());
}

#[test]
#[cfg(unix)]
fn a_pull_request_completes_from_gh_within_a_second__open_pull_requests() {
    let repo = Repo::new();
    let path = stub_gh(
        &repo,
        r#"echo '[{"number":12,"title":"Widen the gate"},{"number":23,"title":"Tighten it again"}]'"#,
    );
    let out = candidates_for(repo.root(), &path, &["gh", ""]);
    let lines = stdout(&out);
    let pulls = lines
        .lines()
        .filter(|l| !l.starts_with('-'))
        .collect::<Vec<_>>();
    assert_eq!(
        pulls,
        vec!["12\tWiden the gate", "23\tTighten it again"],
        "{lines}"
    );
}

#[test]
#[cfg(unix)]
fn a_pull_request_completes_from_gh_within_a_second__slow_gh() {
    let repo = Repo::new();
    let path = stub_gh(
        &repo,
        "sleep 5\necho '[{\"number\":12,\"title\":\"Late\"}]'",
    );
    let start = Instant::now();
    let out = candidates_for(repo.root(), &path, &["gh", ""]);
    let elapsed = start.elapsed();
    assert!(
        values(&out).iter().all(|v| v.starts_with('-')),
        "{}",
        stdout(&out)
    );
    assert!(
        elapsed < Duration::from_secs(3),
        "the completer waited {elapsed:?} on a five-second gh"
    );
}

#[test]
fn a_pull_request_completes_from_gh_within_a_second__no_gh() {
    let repo = Repo::new();
    let empty = repo.root().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let out = candidates_for(repo.root(), &empty.to_string_lossy(), &["gh", ""]);
    assert!(stderr(&out).is_empty(), "{}", stderr(&out));
    assert!(
        values(&out).iter().all(|v| v.starts_with('-')),
        "{}",
        stdout(&out)
    );
}
