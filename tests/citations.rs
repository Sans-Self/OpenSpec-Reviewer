//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::citations::{Citation, Grammar};

fn lint_in(repo: &Repo, args: &[&str]) -> std::process::Output {
    let mut all = vec!["lint"];
    all.extend_from_slice(args);
    run_in(repo.root(), &all)
}

fn errors(out: &std::process::Output) -> Vec<String> {
    stderr(out)
        .lines()
        .filter(|l| l.starts_with("error:"))
        .map(str::to_string)
        .collect()
}

const DELTA: &str = "openspec/changes/rotation-grace-periods/specs/key-rotation/spec.md";

#[test]
fn a_citation_names_a_capability_and_a_requirement__literal_citation() {
    let cites = Grammar::literal("it(`spec:sitemap-index § Flat index of all routes and pages`)");
    assert_eq!(
        cites,
        vec![Citation::new(
            "sitemap-index",
            "Flat index of all routes and pages"
        )]
    );
}

#[test]
fn a_citation_names_a_capability_and_a_requirement__apostrophe_in_the_name() {
    let cites = Grammar::literal(r#"test("spec:pages § The page's route is stable", f)"#);
    assert_eq!(cites[0].requirement, "The page's route is stable");
}

#[test]
fn a_citation_names_a_capability_and_a_requirement__call_helper() {
    let grammar = Grammar::new(Some("cite"));
    let cites =
        grammar.citations("cite('alpha', 'Some rule')\ncite(\n  \"beta\",\n  \"Other's rule\",\n)");
    assert_eq!(
        cites,
        vec![
            Citation::new("alpha", "Some rule"),
            Citation::new("beta", "Other's rule")
        ]
    );
    assert_eq!(
        Citation::new("a", "Two   spaced\n words").requirement,
        "Two spaced words",
        "runs of whitespace compare as one space"
    );
}

#[test]
fn a_citation_must_resolve__spec_first_test() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &[]);
    assert!(
        !stderr(&out).contains("Rotation keeps a grace period"),
        "an in-flight ADDED requirement resolves: {}",
        stderr(&out)
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
}

#[test]
fn a_citation_must_resolve__renamed_requirement() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/stale.test.ts",
        "test(`spec:key-rotation § Rotation mints a new epoch key`, () => {})\n",
    );
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("apps/web/test/stale.test.ts"));
    assert!(errs[0].contains("no requirement `Rotation mints a new epoch key` in key-rotation"));
}

#[test]
fn a_citation_must_resolve__unknown_capability() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/stale.test.ts",
        "test(`spec:nowhere § Anything`, () => {})\n",
    );
    let out = lint_in(&repo, &[]);
    assert!(
        stderr(&out).contains("capability `nowhere` does not exist"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn citations_are_scanned_in_specs_deltas_and_source__source_under_two_roots() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &["--coverage"]);
    let text = stdout(&out);
    assert!(
        text.contains("[1] A tombstone names the epoch it closes"),
        "{text}"
    );
    assert!(
        text.contains("[1] Members re-encrypt on their next write"),
        "{text}"
    );
    assert!(text.contains("6 citations checked"), "{text}");
}

#[test]
fn citations_are_scanned_in_specs_deltas_and_source__skipped_directory() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &[]);
    assert!(!stderr(&out).contains("nowhere"), "{}", stderr(&out));
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn a_cited_path_must_exist__moved_file() {
    let repo = Repo::from_fixture("lint");
    repo.append(
        "openspec/specs/key-rotation/spec.md",
        "\nThe old path was `packages/crypto/src/old.ts`.\n",
    );
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("openspec/specs/key-rotation/spec.md"));
    assert!(errs[0].contains("cited path does not exist: packages/crypto/src/old.ts"));
}

#[test]
fn a_cited_path_must_exist__dotfile_prefix() {
    let repo = Repo::from_fixture("lint");
    repo.append(
        "openspec/specs/key-rotation/spec.md",
        "\nCI runs from `.github/workflows/ci.yml`.\n",
    );
    let missing = lint_in(&repo, &[]);
    assert!(
        errors(&missing)[0].contains(".github/workflows/ci.yml"),
        "{:?}",
        errors(&missing)
    );
    repo.write(".github/workflows/ci.yml", "on: push\n");
    let present = lint_in(&repo, &[]);
    assert_eq!(present.status.code(), Some(0), "{}", stderr(&present));
    assert!(stdout(&present).contains("2 paths"), "{}", stdout(&present));
}

#[test]
fn a_cited_test_must_exist_in_source__renamed_regression_test() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "packages/crypto/src/rotate.ts",
        "export const rotate = (epoch: number): number => epoch + 1\n",
    );
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(
        errs[0].contains("cited regression test not found in source: bug__epoch_skipped_on_rotate")
    );
}

#[test]
fn a_cited_hash_must_be_a_commit__fabricated_hash() {
    let repo = Repo::from_fixture("lint");
    let head = repo.head();
    repo.append(
        "openspec/specs/key-rotation/spec.md",
        &format!(
            "\nLanded in `{}`; the bad one was `deadbeef`.\n",
            &head[..12]
        ),
    );
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("cited commit not found: deadbeef"));
    assert!(stdout(&out).contains("2 hashes"), "{}", stdout(&out));
}

#[test]
fn removing_a_cited_requirement_is_an_error__test_still_cites() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/purge.test.ts",
        "test(`spec:key-rotation § Old epochs are purged immediately`, () => {})\n",
    );
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("rotation-grace-periods"));
    assert!(errs[0].contains("key-rotation § Old epochs are purged immediately"));
    assert!(errs[0].contains("apps/web/test/purge.test.ts"));
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn removing_a_cited_requirement_is_an_error__citing_spec_updated_in_the_same_change() {
    let repo = Repo::from_fixture("lint");
    repo.append(
        "openspec/specs/keyring-tombstones/spec.md",
        "\nPurging follows `spec:key-rotation § Old epochs are purged immediately`.\n",
    );
    let alone = lint_in(&repo, &[]);
    assert_eq!(errors(&alone).len(), 1, "{:?}", errors(&alone));
    assert!(errors(&alone)[0].contains("no delta for keyring-tombstones in this change"));

    repo.delta(
        "rotation-grace-periods",
        "keyring-tombstones",
        "## MODIFIED Requirements\n\n### Requirement: A tombstone names the epoch it closes\n\nUpdated.\n\n#### Scenario: Epoch on the tombstone\n\n- **WHEN** x\n- **THEN** y\n",
    );
    let handled = lint_in(&repo, &[]);
    assert_eq!(errors(&handled).len(), 0, "{:?}", errors(&handled));
}

#[test]
fn modifying_a_cited_requirement_lists_its_citers__two_outside_citers() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &[]);
    let text = stdout(&out);
    let note = text
        .lines()
        .find(|l| l.starts_with("note:"))
        .unwrap_or_else(|| panic!("no note in {text}"));
    assert!(note.contains("rotation-grace-periods"));
    assert!(note.contains("key-rotation § Members re-encrypt on their next write"));
    assert!(text.contains("    openspec/specs/keyring-tombstones/spec.md"));
    assert!(text.contains("    packages/crypto/test/rotate.test.ts"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "notes do not change the exit status"
    );
}

#[test]
fn change_directories_must_be_changes__nested_grouping_directory() {
    let repo = Repo::from_fixture("lint");
    repo.write("openspec/changes/ui/menu/proposal.md", "# menu\n");
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("openspec/changes/ui is not a change directory"));
}

#[test]
fn change_directories_must_be_changes__name_outside_the_scopes() {
    let repo = Repo::from_fixture("lint");
    repo.write("openspec/changes/menu-fix/proposal.md", "# menu-fix\n");
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("menu-fix"));
    assert!(errs[0].contains("does not start with a scope"));
}

#[test]
fn change_directories_must_be_changes__stale_grandfather_entry() {
    let repo = Repo::from_fixture("lint");
    repo.write("openspec/changes/menu-fix/proposal.md", "# menu-fix\n");
    let toml = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    repo.write(
        "openspec/reviewer.toml",
        &toml.replace(
            "grandfathered   = []",
            "grandfathered   = [\"menu-fix\", \"long-gone\"]",
        ),
    );
    let out = lint_in(&repo, &[]);
    let errs = errors(&out);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("grandfathered change `long-gone` is gone"));
    assert!(errs[0].contains("remove it"));
}

#[test]
fn coverage_lists_citing_tests_per_requirement__one_uncited_requirement() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &["--coverage"]);
    let text = stdout(&out);
    assert!(
        text.contains("[0] Clients act on the outcome, never on URI matching  <- uncited"),
        "{text}"
    );
    assert!(
        text.contains("[0] Old epochs are purged immediately  <- uncited"),
        "{text}"
    );
    assert!(
        text.contains("[1] Rotation keeps a grace period"),
        "in-flight requirements are listed: {text}"
    );
    assert!(text.contains("coverage: 5/7"), "{text}");
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn configuration_lives_beside_the_specs__no_config() {
    let repo = Repo::from_fixture("lint");
    repo.remove("openspec/reviewer.toml");
    let out = lint_in(&repo, &[]);
    let err = stderr(&out);
    assert!(err.contains("openspec/reviewer.toml"), "{err}");
    assert!(
        err.contains("[lint]"),
        "the error shows a minimal example: {err}"
    );
    assert!(err.contains("source_roots"), "{err}");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn configuration_lives_beside_the_specs__bad_key() {
    let repo = Repo::from_fixture("lint");
    repo.write("openspec/reviewer.toml", "[lint]\nsource_root = \"apps\"\n");
    let out = lint_in(&repo, &[]);
    assert!(stderr(&out).contains("source_root"), "{}", stderr(&out));
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn configuration_lives_beside_the_specs__fields_left_out_are_named() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "openspec/reviewer.toml",
        "[lint]\nsource_roots = [\"apps\"]\n",
    );
    let out = lint_in(&repo, &[]);
    let line = stdout(&out)
        .lines()
        .find(|l| l.starts_with("lint:"))
        .unwrap()
        .to_string();
    assert!(
        line.contains(
            "not configured: path_prefixes, path_extensions, test_pattern, change_scopes"
        ),
        "{line}"
    );
}

#[test]
fn the_lint_is_a_subcommand_with_a_summary__clean_repository() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &[]);
    let line = stdout(&out)
        .lines()
        .find(|l| l.starts_with("lint:"))
        .unwrap()
        .to_string();
    assert_eq!(
        line,
        "lint: 2 specs, 1 changes, 1 paths, 1 tests, 0 hashes, 6 citations checked, 0 errors"
    );
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn the_lint_is_a_subcommand_with_a_summary__one_dangling_citation() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/stale.test.ts",
        "test(`spec:nowhere § Anything`, () => {})\n",
    );
    let out = lint_in(&repo, &[]);
    assert_eq!(errors(&out).len(), 1);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn the_lint_is_a_subcommand_with_a_summary__json() {
    let repo = Repo::from_fixture("lint");
    let out = lint_in(&repo, &["--format", "json", "--coverage"]);
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("json document");
    assert_eq!(doc["counts"]["citations"], 6);
    assert_eq!(doc["findings"][0]["severity"], "note");
    assert_eq!(doc["coverage"]["cited"], 5);
    assert!(stderr(&out).is_empty(), "{}", stderr(&out));
}

fn review_findings(repo: &Repo) -> Vec<openspec_reviewer::review::Finding> {
    let snapshot = openspec_reviewer::source::ChangeSource {
        root: repo.root().to_path_buf(),
        name: "rotation-grace-periods".to_string(),
    };
    use openspec_reviewer::source::Source;
    let review =
        openspec_reviewer::build::build_review(repo.root(), &snapshot.fetch().unwrap()).unwrap();
    review
        .pairings()
        .flat_map(|p| p.findings.iter().cloned())
        .collect()
}

#[test]
fn citation_findings_join_the_review__modified_requirement_with_citers() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/more.test.ts",
        "test(`spec:key-rotation § Members re-encrypt on their next write`, () => {})\n",
    );
    let out = run_in(repo.root(), &["change", "rotation-grace-periods"]);
    let text = stdout(&out);
    assert!(
        text.contains("note: modified while cited by 3 files outside the capability"),
        "{text}"
    );
    assert!(
        text.contains("        apps/web/test/more.test.ts"),
        "{text}"
    );
    assert!(
        text.contains("        packages/crypto/test/rotate.test.ts"),
        "{text}"
    );
    assert!(
        text.contains("        openspec/specs/keyring-tombstones/spec.md"),
        "{text}"
    );
    let json = run_in(
        repo.root(),
        &["--format", "json", "change", "rotation-grace-periods"],
    );
    assert!(
        stdout(&json).contains("\"kind\": \"modified_has_citers\""),
        "{}",
        stdout(&json)
    );
}

#[test]
fn citation_findings_join_the_review__removed_requirement_still_cited() {
    let repo = Repo::from_fixture("lint");
    repo.write(
        "apps/web/test/purge.test.ts",
        "test(`spec:key-rotation § Old epochs are purged immediately`, () => {})\n",
    );
    let findings = review_findings(&repo);
    let f = findings
        .iter()
        .find(|f| {
            matches!(
                f.kind,
                openspec_reviewer::review::FindingKind::RemovedStillCited { .. }
            )
        })
        .expect("removed still cited");
    assert_eq!(f.location.requirement, "Old epochs are purged immediately");
    assert_eq!(f.severity, openspec_reviewer::review::Severity::Error);
    let out = run_in(repo.root(), &["change", "rotation-grace-periods"]);
    assert!(
        stdout(&out).contains("- Old epochs are purged immediately !"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn citation_findings_join_the_review__dangling_citation_in_a_delta() {
    let repo = Repo::from_fixture("lint");
    repo.append(DELTA, "\nSee `spec:keyring-tombstones § No such rule`.\n");
    let findings = review_findings(&repo);
    let f = findings
        .iter()
        .find(|f| {
            matches!(
                f.kind,
                openspec_reviewer::review::FindingKind::CitationDangling { .. }
            )
        })
        .expect("dangling");
    assert_eq!(f.location.requirement, "Old epochs are purged immediately");
    assert!(f
        .message
        .contains("no requirement `No such rule` in keyring-tombstones"));
}

#[test]
fn citation_findings_join_the_review__no_config_no_citation_findings() {
    let repo = Repo::from_fixture("lint");
    repo.remove("openspec/reviewer.toml");
    let out = run_in(
        repo.root(),
        &["--findings-only", "change", "rotation-grace-periods"],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(!stdout(&out).contains("cited"), "{}", stdout(&out));
}

#[test]
fn lint_init_writes_a_starting_configuration__fresh_repository() {
    let repo = Repo::new();
    repo.write("crates/a/src/x.rs", "fn main() {}\n")
        .write("apps/web/y.tsx", "export {}\n")
        .write("apps/web/node_modules/dep/z.py", "pass\n");
    let out = lint_in(&repo, &["init"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out).trim(), "openspec/reviewer.toml");
    let text = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    let config = openspec_reviewer::citations::config::parse_config(
        std::path::Path::new("openspec/reviewer.toml"),
        &text,
    )
    .expect("written file parses");
    assert_eq!(
        config.lint.source_roots,
        Some(vec!["apps".into(), "crates".into()])
    );
    let exts = config.lint.path_extensions.unwrap();
    assert!(
        exts.contains(&"rs".to_string()) && exts.contains(&"tsx".to_string()),
        "{exts:?}"
    );
    assert!(
        !exts.contains(&"py".to_string()),
        "skipped directories are not surveyed: {exts:?}"
    );
    assert_eq!(config.lint.test_pattern.as_deref(), Some("bug__\\w+"));
    assert!(config.lint.change_scopes.is_none());
}

#[test]
fn lint_init_writes_a_starting_configuration__config_already_exists() {
    let repo = Repo::from_fixture("lint");
    let before = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    let out = lint_in(&repo, &["init"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).contains("openspec/reviewer.toml"),
        "{}",
        stderr(&out)
    );
    let after = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn lint_init_writes_a_starting_configuration__no_openspec_directory() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_in(dir.path(), &["lint", "init"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("openspec/"), "{}", stderr(&out));
    assert!(!dir.path().join("openspec/reviewer.toml").exists());
}

#[test]
fn lint_init_writes_a_starting_configuration__written_file_lints() {
    let repo = Repo::from_fixture("lint");
    repo.remove("openspec/reviewer.toml");
    assert_eq!(lint_in(&repo, &["init"]).status.code(), Some(0));
    let out = lint_in(&repo, &[]);
    assert!(!stderr(&out).contains("reviewer.toml"), "{}", stderr(&out));
    assert!(stdout(&out).contains("lint:"), "{}", stdout(&out));
}

#[test]
fn lint_init_writes_a_starting_configuration__example_matches_the_schema() {
    use openspec_reviewer::citations::{render, Survey};
    let text = render(&Survey::example());
    openspec_reviewer::citations::config::parse_config(std::path::Path::new("x"), &text)
        .expect("the refusal's example parses");
    let empty = render(&Survey::default());
    let config =
        openspec_reviewer::citations::config::parse_config(std::path::Path::new("x"), &empty)
            .expect("an empty survey still renders valid toml");
    assert_eq!(config.lint.source_roots, Some(vec![]));
    assert!(empty.contains("No source directory found"));
}

#[test]
fn lint_init_writes_a_starting_configuration__json_is_cited_but_not_scanned() {
    let repo = Repo::new();
    repo.write("src/a.rs", "fn main() {}\n")
        .write("src/fixtures/b.json", "{}\n");
    assert_eq!(lint_in(&repo, &["init"]).status.code(), Some(0));
    let text = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    let config = openspec_reviewer::citations::config::parse_config(
        std::path::Path::new("openspec/reviewer.toml"),
        &text,
    )
    .unwrap();
    assert!(config
        .lint
        .path_extensions
        .unwrap()
        .contains(&"json".to_string()));
    assert!(!config
        .lint
        .source_globs
        .unwrap()
        .contains(&"**/*.json".to_string()));
}

#[test]
fn lint_init_writes_a_starting_configuration__elixir_tests_are_scanned() {
    let repo = Repo::new();
    repo.write("test/keyring_test.exs", "# spec:x § Y\n")
        .write("test/_build/dev/z.py", "pass\n");
    assert_eq!(lint_in(&repo, &["init"]).status.code(), Some(0));
    let text = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    let config = openspec_reviewer::citations::config::parse_config(
        std::path::Path::new("openspec/reviewer.toml"),
        &text,
    )
    .unwrap();
    let globs = config.lint.source_globs.unwrap();
    assert!(globs.contains(&"**/*.exs".to_string()), "{globs:?}");
    assert!(
        !globs.contains(&"**/*.py".to_string()),
        "_build is skipped: {globs:?}"
    );
}
