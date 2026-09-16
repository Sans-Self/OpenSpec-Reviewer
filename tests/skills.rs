//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::citations::Grammar;
use openspec_reviewer::review::FindingKind;
use openspec_reviewer::skills::{checksum, plain_citations, recorded_checksum, split, SKILLS};
use std::path::Path;

const TASK_SKILLS: [&str; 5] = ["define", "discover", "cite", "crossref", "triage"];

fn skills_in(repo: &Repo, args: &[&str]) -> std::process::Output {
    let mut all = vec!["skills"];
    all.extend_from_slice(args);
    run_in(repo.root(), &all)
}

fn claude_repo() -> Repo {
    let repo = Repo::new();
    std::fs::create_dir_all(repo.root().join(".claude")).unwrap();
    repo
}

fn installed(repo: &Repo, name: &str) -> String {
    std::fs::read_to_string(
        repo.root()
            .join(format!(".claude/skills/opsx-reviewer-{name}/SKILL.md")),
    )
    .unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn body_of(text: &str) -> &str {
    split(text).expect("frontmatter").1
}

/// This repository's canon, copied in, so the skills' citations resolve.
fn home_repo() -> Repo {
    let repo = claude_repo();
    let canon = Path::new(env!("CARGO_MANIFEST_DIR")).join("openspec/specs");
    for entry in std::fs::read_dir(canon).unwrap() {
        let entry = entry.unwrap();
        let spec = entry.path().join("spec.md");
        if spec.is_file() {
            repo.canon(
                entry.file_name().to_str().unwrap(),
                &std::fs::read_to_string(spec).unwrap(),
            );
        }
    }
    repo
}

#[test]
fn skills_install_into_the_repository__fresh_install() {
    let repo = claude_repo();
    let out = skills_in(&repo, &["install"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = installed(&repo, "workflow");
    let (front, body) = split(&text).unwrap();
    assert!(front.contains("name: opsx-reviewer-workflow"));
    assert!(front.contains("generatedBy: openspec-reviewer 0.1.0"));
    assert_eq!(recorded_checksum(&text), Some(checksum(body).as_str()));
    for name in TASK_SKILLS {
        installed(&repo, name);
    }
    assert!(!repo.root().join(".agents").exists());
    assert_eq!(
        stdout(&out).matches("wrote .claude/skills/").count(),
        6,
        "{}",
        stdout(&out)
    );
}

#[test]
fn skills_install_into_the_repository__other_agents_directory_present() {
    let repo = claude_repo();
    std::fs::create_dir_all(repo.root().join(".agents")).unwrap();
    skills_in(&repo, &["install"]);
    let agents = std::fs::read_to_string(
        repo.root()
            .join(".agents/skills/opsx-reviewer-workflow/SKILL.md"),
    )
    .unwrap();
    assert_eq!(body_of(&agents), body_of(&installed(&repo, "workflow")));
}

#[test]
fn skills_install_into_the_repository__hand_edited_skill_is_kept() {
    let repo = claude_repo();
    skills_in(&repo, &["install"]);
    let path = ".claude/skills/opsx-reviewer-cite/SKILL.md";
    let edited = installed(&repo, "cite") + "\nAlways ask twice.\n";
    repo.write(path, &edited);
    let out = skills_in(&repo, &["install"]);
    assert_eq!(installed(&repo, "cite"), edited);
    let line = stdout(&out)
        .lines()
        .find(|l| l.contains("opsx-reviewer-cite"))
        .map(str::to_string)
        .unwrap();
    assert!(line.starts_with("kept"), "{line}");
    assert!(line.contains("differs"), "{line}");
}

#[test]
fn skills_install_into_the_repository__no_claude_directory() {
    let repo = Repo::new();
    let out = skills_in(&repo, &["install"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains(".claude/"), "{}", stderr(&out));
}

#[test]
fn skills_install_into_the_repository__list_reports_state() {
    let repo = claude_repo();
    let before = stdout(&skills_in(&repo, &["list"]));
    assert_eq!(before.matches("not installed").count(), 11, "{before}");
    skills_in(&repo, &["install"]);
    let path = ".claude/skills/opsx-reviewer-cite/SKILL.md";
    repo.write(path, &(installed(&repo, "cite") + "edited\n"));
    let after = stdout(&skills_in(&repo, &["list"]));
    assert_eq!(after.matches("up to date").count(), 10, "{after}");
    assert!(
        after
            .lines()
            .any(|l| l.contains("edited") && l.contains("opsx-reviewer-cite")),
        "{after}"
    );
}

#[test]
fn skills_install_into_the_repository__outdated_file_is_overwritten() {
    let repo = claude_repo();
    skills_in(&repo, &["install"]);
    let path = ".claude/skills/opsx-reviewer-cite/SKILL.md";
    let older = "---\nname: opsx-reviewer-cite\nmetadata:\n  checksum: ".to_string()
        + &checksum("old body\n")
        + "\n---\nold body\n";
    repo.write(path, &older);
    assert!(stdout(&skills_in(&repo, &["list"])).contains("outdated"));
    skills_in(&repo, &["install"]);
    assert_eq!(
        body_of(&installed(&repo, "cite")),
        plain_citations(openspec_reviewer::skills::skill("cite").unwrap().body)
    );
}

#[test]
fn user_callable_skills_get_a_command_alias__five_commands_not_six() {
    let repo = claude_repo();
    skills_in(&repo, &["install"]);
    let dir = repo.root().join(".claude/commands/opsx-reviewer");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "cite.md",
            "crossref.md",
            "define.md",
            "discover.md",
            "triage.md"
        ]
    );
}

#[test]
fn user_callable_skills_get_a_command_alias__command_defers_to_the_skill() {
    let repo = claude_repo();
    skills_in(&repo, &["install"]);
    let text =
        std::fs::read_to_string(repo.root().join(".claude/commands/opsx-reviewer/triage.md"))
            .unwrap();
    let (front, body) = split(&text).unwrap();
    assert!(body.contains("`opsx-reviewer-triage`"), "{body}");
    assert!(body.contains("$ARGUMENTS"), "{body}");
    assert!(!body.contains("severity"), "{body}");
    assert_eq!(recorded_checksum(&text), Some(checksum(body).as_str()));
    assert!(front.contains("generatedBy"));
}

#[test]
fn the_workflow_skill_tells_an_agent_when_to_run_the_reviewer__every_finding_kind_is_explained() {
    let body = SKILLS[0].body;
    for kind in FindingKind::each() {
        let name = kind.name();
        let line = body
            .lines()
            .find(|l| l.contains(&format!("`{name}`")) && l.trim_start().starts_with("- "))
            .unwrap_or_else(|| panic!("workflow skill does not explain `{name}`"));
        assert!(line.contains(':'), "no meaning for `{name}`: {line}");
    }
    for judgment in [
        "sibling_uses_removed",
        "sibling_uses_old_name",
        "term_in_use",
        "modified_has_citers",
    ] {
        let table_row = body
            .lines()
            .find(|l| l.starts_with('|') && l.contains(judgment))
            .unwrap_or_else(|| panic!("{judgment} is not routed"));
        assert!(table_row.contains("opsx-reviewer-crossref"), "{table_row}");
    }
}

#[test]
fn the_workflow_skill_tells_an_agent_when_to_run_the_reviewer__agent_finishes_a_delta() {
    let skill = SKILLS[0];
    assert!(!skill.command);
    assert!(skill.description.contains("openspec/"));
    assert!(skill.description.contains("openspec-reviewer binary"));
    assert!(!skill.body.contains("disable-model-invocation"));
    assert!(skill
        .body
        .contains("openspec-reviewer change <name> --format json"));
    assert!(skill.body.contains("before you propose archiving"));
    assert!(skill.body.contains("openspec-reviewer lint"));
    assert!(skill
        .body
        .contains("`spec:<capability> § <requirement name>`"));
    assert!(skill.body.contains("edits `openspec/specs/`"));
}

#[test]
fn a_project_can_replace_a_skills_body__override_present() {
    let repo = claude_repo();
    repo.write(
        "openspec/reviewer/skills/triage.md",
        "Ask before every edit.\n",
    );
    skills_in(&repo, &["install"]);
    let text = installed(&repo, "triage");
    let (front, body) = split(&text).unwrap();
    assert_eq!(body, "Ask before every edit.\n");
    assert!(front.contains("name: opsx-reviewer-triage"));
    assert!(front.contains("override: openspec/reviewer/skills/triage.md"));
}

#[test]
fn skills_cite_the_requirements_they_depend_on__installed_here() {
    let repo = home_repo();
    repo.write(
        "openspec/reviewer.toml",
        "[lint]\nsource_roots = [\".claude\"]\nsource_globs = [\"**/*.md\"]\n",
    );
    repo.init_git("main");
    skills_in(&repo, &["install"]);
    for skill in SKILLS {
        assert!(
            installed(&repo, skill.name).contains("`spec:"),
            "{}",
            skill.name
        );
    }
    let out = run_in(repo.root(), &["lint"]);
    let err = stderr(&out);
    let in_skills: Vec<&str> = err.lines().filter(|l| l.contains(".claude/")).collect();
    assert!(in_skills.is_empty(), "{in_skills:?}");
    let summary = stdout(&out);
    let checked = summary
        .lines()
        .find(|l| l.starts_with("lint:"))
        .and_then(|l| {
            l.split(", ")
                .find_map(|f| f.strip_suffix(" citations checked"))
        })
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("{summary}"));
    assert!(checked >= 20, "{summary}");
}

#[test]
fn skills_cite_the_requirements_they_depend_on__installed_elsewhere() {
    let repo = claude_repo();
    repo.canon("alpha", ALPHA_CANON);
    skills_in(&repo, &["install"]);
    for skill in SKILLS {
        let text = installed(&repo, skill.name);
        assert!(
            Grammar::literal(body_of(&text)).is_empty(),
            "{}: {text}",
            skill.name
        );
        assert!(
            text.contains("## Depends on\n\n- `citations")
                || text.contains("- `review-findings")
                || text.contains("- `glossary")
                || text.contains("- `term-drift"),
            "{}",
            skill.name
        );
    }
}

#[test]
fn skills_cite_the_requirements_they_depend_on__rewrite_keeps_the_names() {
    assert_eq!(
        plain_citations("see `spec:alpha § Flat index of all routes and pages`."),
        "see `alpha § Flat index of all routes and pages`."
    );
}

#[test]
fn every_skill_writes_a_change_never_canon__shape_of_every_task_skill() {
    for name in TASK_SKILLS {
        let body = openspec_reviewer::skills::skill(name).unwrap().body;
        let first = body
            .split("\n\n")
            .next()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            first.contains("openspec/changes/") || first.contains("test files"),
            "{name}: {first}"
        );
        assert!(first.contains("never edits"), "{name}: {first}");
        assert!(first.contains("`openspec/specs/`"), "{name}: {first}");
        let steps = body.split("## Depends on").next().unwrap();
        let last_step_at = steps
            .match_indices('\n')
            .filter(|(i, _)| steps[i + 1..].starts_with(|c: char| c.is_ascii_digit()))
            .map(|(i, _)| i + 1)
            .next_back()
            .unwrap();
        let last_step = &steps[last_step_at..];
        assert!(
            last_step.contains("Run `openspec-reviewer")
                || last_step.contains("Run the review again"),
            "{name}: {last_step}"
        );
    }
}

#[test]
fn every_skill_writes_a_change_never_canon__shape_of_the_workflow_skill() {
    let first = SKILLS[0].body.split("\n\n").next().unwrap();
    assert!(first.contains("writes nothing itself"), "{first}");
    assert!(first.contains("`openspec/specs/`"), "{first}");
}

#[test]
fn every_skill_writes_a_change_never_canon__depends_on_list_closes_every_body() {
    for skill in SKILLS {
        let tail = skill.body.rsplit("## Depends on").next().unwrap();
        assert!(
            tail.lines().filter(|l| l.starts_with("- `spec:")).count() >= 3,
            "{}",
            skill.name
        );
    }
}

#[test]
fn the_define_skill_drafts_glossary_terms__three_candidates_one_kept() {
    let body = openspec_reviewer::skills::skill("define").unwrap().body;
    assert!(body.contains("openspec-reviewer lint --format json"));
    assert!(body.contains("recurring term without definition"));
    assert!(body.contains("field name"));
    assert!(body.contains("openspec/changes/<name>/specs/definitions/spec.md"));
    assert!(body.contains("## ADDED Requirements"));
    assert!(body.contains("### Requirement: <term>"));
    assert!(body.contains("A spec MUST use `<term>` to mean:"));
    assert!(body.contains("- **Deprecated:**"));
    assert!(body.contains("openspec-reviewer change <name>"));
}

#[test]
fn the_discover_skill_finds_the_vocabulary_the_spans_miss__prose_term_in_three_capabilities() {
    let body = openspec_reviewer::skills::skill("discover").unwrap().body;
    assert!(body.contains("openspec-reviewer lint --format json"));
    assert!(body.contains("recurring term without definition"));
    assert!(body.contains("`### Requirement:` heading"));
    assert!(body.contains("one to three"));
    assert!(body.contains("two or more capabilities"));
    assert!(body.contains("`capability § requirement`"));
    assert!(body.contains("`define`"));
    assert!(body.contains("openspec-reviewer change <name>"));
}

#[test]
fn the_discover_skill_finds_the_vocabulary_the_spans_miss__phrase_in_one_capability_only() {
    let body = openspec_reviewer::skills::skill("discover").unwrap().body;
    assert!(
        body.contains("recurrence inside one capability is\n   not a signal"),
        "{body}"
    );
    assert!(body.contains("Drop the rest"));
}

#[test]
fn the_discover_skill_finds_the_vocabulary_the_spans_miss__phrase_the_glossary_knows() {
    let body = openspec_reviewer::skills::skill("discover").unwrap().body;
    assert!(body.contains("openspec/specs/definitions/spec.md"));
    assert!(body.contains("- **Admitted:**"));
    assert!(body.contains("- **Deprecated:**"));
    assert!(body.contains("Ask how many"));
}

#[test]
fn the_cite_skill_adds_citations_to_uncited_requirements__two_uncited_one_test_found() {
    let body = openspec_reviewer::skills::skill("cite").unwrap().body;
    assert!(body.contains("openspec-reviewer lint --coverage --format json"));
    assert!(body.contains("`citing_files` of\n   `0`") || body.contains("`citing_files` of `0`"));
    assert!(body.contains("spec:<capability> § <requirement name>"));
    assert!(body.contains("Do not rename, move or edit the body of the test"));
    assert!(body.contains("no test found"));
    assert!(body.contains("before and after"));
}

#[test]
fn the_crossref_skill_judges_siblings__one_of_each_verdict() {
    let body = openspec_reviewer::skills::skill("crossref").unwrap().body;
    assert!(body.contains("openspec-reviewer change <change> --format json"));
    for kind in [
        "sibling_uses_removed",
        "sibling_uses_old_name",
        "term_in_use",
        "modified_has_citers",
    ] {
        assert!(body.contains(kind), "{kind}");
    }
    for verdict in ["`consistent`", "`contradicts`", "`stale term`"] {
        assert!(body.contains(verdict), "{verdict}");
    }
    assert!(body.contains("openspec/changes/archive/*/design.md"));
    assert!(
        body.contains("reverses a\n   recorded decision")
            || body.contains("reverses a recorded decision")
    );
    assert!(body.contains("## MODIFIED Requirements"));
    assert!(body.contains("only on confirmation"));
    assert!(body.contains("warning count before and after"));
}

#[test]
fn the_triage_skill_walks_findings_in_order__typo_in_a_citation() {
    let body = openspec_reviewer::skills::skill("triage").unwrap().body;
    assert!(body.contains("errors first"));
    let row = body
        .lines()
        .find(|l| l.contains("`citation_dangling`"))
        .unwrap();
    assert!(row.contains("nearest canon requirement name"), "{row}");
    assert!(
        body.contains("Apply only on\n   confirmation")
            || body.contains("Apply only on confirmation")
    );
    assert!(body.contains("only to files under `openspec/changes/<change>/`"));
}

#[test]
fn the_triage_skill_walks_findings_in_order__judgment_is_routed() {
    let body = openspec_reviewer::skills::skill("triage").unwrap().body;
    let routed = body.split("needs reading").next().unwrap();
    for kind in [
        "sibling_uses_removed",
        "sibling_uses_old_name",
        "term_in_use",
        "modified_has_citers",
    ] {
        assert!(routed.contains(kind), "{kind}");
    }
    assert!(
        body.contains("`opsx-reviewer-crossref` as the next\n   step")
            || body.contains("crossref` as the next step")
    );
    assert!(body.contains("Propose no edit for them"));
    assert!(body.contains("Do not dismiss any finding"));
}

#[test]
fn lint_init_writes_a_starting_configuration__agent_directories_are_roots() {
    let repo = claude_repo();
    repo.write(".claude/skills/x/SKILL.md", "hello\n");
    repo.write("src/lib.rs", "");
    let out = run_in(repo.root(), &["lint", "init"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let toml = std::fs::read_to_string(repo.root().join("openspec/reviewer.toml")).unwrap();
    let roots = toml
        .lines()
        .find(|l| l.starts_with("source_roots"))
        .unwrap();
    assert!(roots.contains("\".claude\""), "{roots}");
    assert!(!roots.contains("\".agents\""), "{roots}");
    let globs = toml
        .lines()
        .find(|l| l.starts_with("source_globs"))
        .unwrap();
    assert!(globs.contains("**/*.md"), "{globs}");
}

/// A markdown block a skill instructs an agent to write, with the
/// capability its delta belongs to.
struct Drafted {
    skill: &'static str,
    capability: String,
    delta: String,
}

/// Every fenced block in a skill body that is a delta spec, with the
/// angle-bracket placeholders filled in. A skill that starts instructing
/// another artefact shape is swept without touching this test.
fn drafted_deltas() -> Vec<Drafted> {
    let filled = |block: &str| {
        let sample = "the ledger gains an entry";
        block
            .replace("<term>", "ledger")
            .split_inclusive('\n')
            .map(|line| {
                let mut out = String::new();
                let mut rest = line;
                while let Some(open) = rest.find('<') {
                    match rest[open..].find('>') {
                        Some(close) => {
                            out.push_str(&rest[..open]);
                            out.push_str(sample);
                            rest = &rest[open + close + 1..];
                        }
                        None => break,
                    }
                }
                out.push_str(rest);
                out
            })
            .collect::<String>()
    };
    SKILLS
        .iter()
        .flat_map(|skill| {
            skill
                .body
                .split("```")
                .skip(1)
                .step_by(2)
                .filter(|block| block.contains("### Requirement:"))
                .map(|block| {
                    // The fence's language tag is the first line; the delta
                    // is indented to sit inside the numbered step.
                    let body: String = block
                        .lines()
                        .skip(1)
                        .map(|l| format!("{}\n", l.strip_prefix("   ").unwrap_or(l)))
                        .collect();
                    let capability = body
                        .lines()
                        .find_map(|l| l.strip_prefix("# ")?.strip_suffix(" (delta)"))
                        .unwrap_or("definitions")
                        .to_string();
                    Drafted {
                        skill: skill.name,
                        capability,
                        delta: filled(&body),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// `openspec` is on `PATH` in the nix devshell. A build that does not have
/// it — the flake's own package build, whose sandbox has no network for
/// the pinned dlx wrapper — cannot run the validator at all.
fn openspec_missing() -> bool {
    match std::process::Command::new("openspec")
        .arg("--version")
        .output()
    {
        Ok(out) => !out.status.success(),
        Err(_) => true,
    }
}

/// Writes `delta` as the one delta of a change and runs `openspec validate`
/// over it, returning the validator's combined output and whether it
/// reported the change valid.
fn validate_drafted(capability: &str, delta: &str) -> (bool, String) {
    let repo = Repo::new();
    let change = "drafted";
    repo.write(
        &format!("openspec/changes/{change}/.openspec.yaml"),
        "schema: spec-driven\ncreated: 2026-09-14\n",
    )
    .write(
        &format!("openspec/changes/{change}/proposal.md"),
        "## Why\n\nThe shape a skill instructs has to validate.\n\n## What Changes\n\n- One drafted entry.\n",
    )
    .write(
        &format!("openspec/changes/{change}/tasks.md"),
        "## 1. Draft\n\n- [ ] 1.1 Write the entry.\n",
    )
    .delta(change, capability, delta);
    let out = std::process::Command::new("openspec")
        .args(["validate", change])
        .current_dir(repo.root())
        .output()
        .expect("openspec runs");
    let text = format!("{}{}", stdout(&out), stderr(&out));
    (out.status.success() && !text.contains("[ERROR]"), text)
}

#[test]
fn a_skills_drafted_output_validates__drafted_term() {
    if openspec_missing() {
        eprintln!("skipped: openspec is not on PATH");
        return;
    }
    let drafted = drafted_deltas();
    let define = drafted
        .iter()
        .find(|d| d.skill == "define")
        .unwrap_or_else(|| panic!("the define skill instructs a delta shape"));
    assert_eq!(define.capability, "definitions");
    let (valid, text) = validate_drafted(&define.capability, &define.delta);
    assert!(valid, "{}\n{text}", define.delta);
}

#[test]
fn a_skills_drafted_output_validates__instructed_shape_stops_validating() {
    if openspec_missing() {
        eprintln!("skipped: openspec is not on PATH");
        return;
    }
    let drafted = drafted_deltas();
    assert!(!drafted.is_empty(), "no skill instructs a delta shape");
    for d in &drafted {
        let (valid, text) = validate_drafted(&d.capability, &d.delta);
        assert!(valid, "{}: {text}\n{}", d.skill, d.delta);
        let without = d.delta.replace("A spec MUST use `ledger` to mean:\n\n", "");
        if without != d.delta {
            let (still, _) = validate_drafted(&d.capability, &without);
            assert!(
                !still,
                "{}: the binding line is what makes the draft validate",
                d.skill
            );
        }
    }
}

/// `openspec validate` rejects every ADDED or MODIFIED requirement whose
/// body holds no SHALL or MUST, so the term the define skill drafted could
/// never be proposed as a change.
#[test]
fn bug__drafted_term_fails_validation() {
    if openspec_missing() {
        eprintln!("skipped: openspec is not on PATH");
        return;
    }
    let body = openspec_reviewer::skills::skill("define").unwrap().body;
    assert!(
        body.contains("A spec MUST use `<term>` to mean:"),
        "the skill drafts the binding line"
    );
    let drafted = drafted_deltas();
    let define = drafted.iter().find(|d| d.skill == "define").unwrap();
    let (valid, text) = validate_drafted(&define.capability, &define.delta);
    assert!(valid, "{text}");
}
