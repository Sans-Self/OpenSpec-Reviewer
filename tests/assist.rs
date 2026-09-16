//! Test titles quote the requirement they cover; a `__` suffix names
//! the scenario. No test here calls a real agent: every adapter runs
//! against a fake CLI from `tests/fixtures/assist/`.
#![allow(non_snake_case)]

mod common;

use common::*;
use openspec_reviewer::assist::{
    assistant, files, parse_hints, Agent, Assist, AssistError, Hint, HintKind, Session,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A search path holding one fake CLI under the name the adapter looks
/// for. The fake records its argv and stdin next to itself.
struct FakePath {
    dir: tempfile::TempDir,
    binary: String,
}

impl FakePath {
    fn with(binary: &str, fixture: &str) -> FakePath {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/assist")
            .join(fixture);
        std::fs::copy(&source, dir.path().join(binary)).expect("fake CLI copies");
        FakePath {
            dir,
            binary: binary.to_string(),
        }
    }

    fn path(&self) -> OsString {
        self.dir.path().as_os_str().to_os_string()
    }

    /// The argv the fake was called with, one entry per line, without the
    /// program name.
    fn argv(&self) -> Vec<String> {
        let path = self.dir.path().join(format!("{}.argv", self.binary));
        std::fs::read_to_string(path)
            .expect("the fake CLI ran")
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn stdin(&self) -> String {
        let path = self.dir.path().join(format!("{}.stdin", self.binary));
        std::fs::read_to_string(path).unwrap_or_default()
    }
}

fn assist(agent: Agent) -> Assist {
    Assist {
        agent,
        handoff_command: None,
        review_command: None,
    }
}

fn prompt_file(dir: &Path, text: &str) -> PathBuf {
    let path = dir.join("prompt.md");
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn an_assistant_is_an_agent_cli_behind_one_interface() {
    for (binary, agent) in [
        ("claude", Agent::Claude),
        ("codex", Agent::Codex),
        ("opencode", Agent::Opencode),
    ] {
        let fake = FakePath::with(binary, "agent-valid.sh");
        let prompt = prompt_file(fake.dir.path(), "review this\n");
        let adapter = assistant(Some(&assist(agent)), Some(&fake.path())).expect("adapter");
        let reply = adapter.review(&prompt).expect("the fake replies");
        assert_eq!(parse_hints(&reply).len(), 2, "{binary} reply parses");
    }
}

#[test]
fn an_assistant_is_an_agent_cli_behind_one_interface__two_agents_same_handoff() {
    let mut seen = Vec::new();
    for (binary, agent) in [("claude", Agent::Claude), ("codex", Agent::Codex)] {
        let fake = FakePath::with(binary, "agent-echo.sh");
        let prompt = prompt_file(fake.dir.path(), "the same prompt file\n");
        let adapter = assistant(Some(&assist(agent)), Some(&fake.path())).expect("adapter");
        adapter.handoff(&prompt).expect("the fake session exits");
        seen.push(fake.argv().join("\n"));
    }
    assert_eq!(seen[0], seen[1]);
    assert!(seen[0].contains("the same prompt file"));
}

#[test]
fn an_assistant_is_an_agent_cli_behind_one_interface__claude_flags() {
    let fake = FakePath::with("claude", "agent-valid.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Claude)), Some(&fake.path())).expect("adapter");
    adapter.review(&prompt).expect("reply");
    assert_eq!(fake.argv(), ["-p", "--output-format", "json"]);
    assert_eq!(fake.stdin(), "pairing prompt\n");
}

#[test]
fn an_assistant_is_an_agent_cli_behind_one_interface__codex_flags() {
    let fake = FakePath::with("codex", "agent-valid.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Codex)), Some(&fake.path())).expect("adapter");
    adapter.review(&prompt).expect("reply");
    assert_eq!(fake.argv(), ["exec", "--json"]);
    assert_eq!(fake.stdin(), "pairing prompt\n");
}

#[test]
fn an_assistant_is_an_agent_cli_behind_one_interface__opencode_flags() {
    let fake = FakePath::with("opencode", "agent-valid.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Opencode)), Some(&fake.path())).expect("adapter");
    adapter.review(&prompt).expect("reply");
    let argv = fake.argv();
    assert_eq!(argv[..3], ["run", "--format", "json"]);
    assert_eq!(argv[3], "pairing prompt");
}

#[test]
fn an_assistant_is_an_agent_cli_behind_one_interface__claude_unwraps_its_envelope() {
    let fake = FakePath::with("claude", "agent-envelope.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Claude)), Some(&fake.path())).expect("adapter");
    let hints = parse_hints(&adapter.review(&prompt).expect("reply"));
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].kind, HintKind::PlainLanguage);
}

#[test]
fn the_agent_is_chosen_in_configuration__not_configured() {
    let error = assistant(None, None).err().expect("no [assist] section");
    assert!(
        matches!(error, AssistError::NotConfigured),
        "unexpected: {error}"
    );
    assert!(error.to_string().contains("assist is not configured"));
}

#[test]
fn the_agent_is_chosen_in_configuration__agent_binary_missing() {
    let empty = tempfile::tempdir().unwrap();
    let path = empty.path().as_os_str().to_os_string();
    let error = assistant(Some(&assist(Agent::Codex)), Some(&path))
        .err()
        .expect("no codex on this path");
    assert_eq!(error.to_string(), "the `codex` CLI was not found");
}

#[test]
fn the_agent_is_chosen_in_configuration__custom_agent() {
    let dir = tempfile::tempdir().unwrap();
    let recorded = dir.path().join("recorded");
    let prompt = prompt_file(dir.path(), "custom prompt\n");
    let custom = Assist {
        agent: Agent::Custom,
        handoff_command: Some(format!("echo {} > {}", "{prompt}", recorded.display())),
        review_command: None,
    };
    let path = std::env::var_os("PATH");
    let adapter = assistant(Some(&custom), path.as_ref()).expect("sh is on the path");
    adapter.handoff(&prompt).expect("the template runs");
    let written = std::fs::read_to_string(&recorded).expect("the command ran");
    assert_eq!(written.trim(), prompt.display().to_string());
}

#[test]
fn the_agent_is_chosen_in_configuration__custom_review_command() {
    let dir = tempfile::tempdir().unwrap();
    let prompt = prompt_file(dir.path(), "custom prompt\n");
    let custom = Assist {
        agent: Agent::Custom,
        handoff_command: None,
        review_command: Some(
            "printf '[{\"kind\":\"term_misuse\",\"message\":\"module means a code module\"}]'"
                .to_string(),
        ),
    };
    let path = std::env::var_os("PATH");
    let adapter = assistant(Some(&custom), path.as_ref()).expect("sh is on the path");
    let hints = parse_hints(&adapter.review(&prompt).expect("reply"));
    assert_eq!(hints[0].kind, HintKind::TermMisuse);
}

#[test]
fn the_agent_is_chosen_in_configuration__parsed_from_reviewer_toml() {
    let config = openspec_reviewer::citations::config::parse_config(
        Path::new("openspec/reviewer.toml"),
        "[assist]\nagent = \"opencode\"\n",
    )
    .expect("the section parses");
    let assist = config.assist.expect("an [assist] section");
    assert_eq!(assist.agent, Agent::Opencode);
    assert_eq!(assist.handoff_command, None);
}

#[test]
fn batch_review_turns_the_agents_reply_into_hints__two_hints() {
    let fake = FakePath::with("claude", "agent-valid.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Claude)), Some(&fake.path())).expect("adapter");
    let hints = parse_hints(&adapter.review(&prompt).expect("reply"));
    assert_eq!(hints.len(), 2);
    assert_eq!(hints[0].kind, HintKind::CompoundCondition);
    assert_eq!(
        hints[0].message,
        "the WHEN joins mounting a page and rendering the row"
    );
    assert_eq!(
        hints[0].scenario.as_deref(),
        Some("Route-mounted page appears as a row")
    );
    assert_eq!(hints[1].kind, HintKind::UncoveredMust);
}

#[test]
fn batch_review_turns_the_agents_reply_into_hints__malformed_reply() {
    let fake = FakePath::with("claude", "agent-malformed.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Claude)), Some(&fake.path())).expect("adapter");
    let hints = parse_hints(&adapter.review(&prompt).expect("reply"));
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].kind, HintKind::Unparsed);
    assert_eq!(
        hints[0].message,
        "I read the requirement and it looks fine to me."
    );
}

#[test]
fn batch_review_turns_the_agents_reply_into_hints__agent_exits_non_zero() {
    let fake = FakePath::with("claude", "agent-fails.sh");
    let prompt = prompt_file(fake.dir.path(), "pairing prompt\n");
    let adapter = assistant(Some(&assist(Agent::Claude)), Some(&fake.path())).expect("adapter");
    let error = adapter.review(&prompt).expect_err("exit 3");
    assert!(
        error
            .to_string()
            .contains("not logged in; run the agent once by hand"),
        "unexpected: {error}"
    );
}

#[test]
fn batch_review_turns_the_agents_reply_into_hints__unknown_kind_is_unparsed() {
    let hints = parse_hints(r#"[{"kind":"vibes","message":"feels off"}]"#);
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].kind, HintKind::Unparsed);
    assert!(hints[0].message.contains("unknown hint kind `vibes`"));
}

#[test]
fn batch_review_turns_the_agents_reply_into_hints__empty_array_is_no_hints() {
    assert!(parse_hints("[]").is_empty());
    assert!(parse_hints("Here is my answer:\n\n```json\n[]\n```\n").is_empty());
}

#[test]
fn hint_round_trips_through_json() {
    let hint = Hint {
        kind: HintKind::SiblingInvalidated,
        message: "the sibling still assumes one route per page".to_string(),
        quote: Some("- **THEN** the index shows one row".to_string()),
        scenario: None,
        dismissed: true,
    };
    let text = serde_json::to_string(&hint).unwrap();
    assert!(text.contains("\"sibling_invalidated\""));
    assert!(text.contains("\"dismissed\":true"));
    assert_eq!(serde_json::from_str::<Hint>(&text).unwrap(), hint);
}

#[test]
fn prompt_templates_ship_and_can_be_overridden__export_defaults() {
    let repo = Repo::new();
    let export = files::export_prompts(repo.root()).expect("the export writes");
    assert_eq!(
        export.written,
        [
            "openspec/reviewer/prompts/pairing.md",
            "openspec/reviewer/prompts/change.md",
            "openspec/reviewer/prompts/hints.md",
        ]
    );
    assert!(export.skipped.is_empty());
    for name in ["pairing.md", "change.md", "hints.md"] {
        let path = repo.root().join("openspec/reviewer/prompts").join(name);
        assert!(path.is_file(), "{name} written");
    }
}

#[test]
fn prompt_templates_ship_and_can_be_overridden__export_does_not_overwrite() {
    let repo = Repo::new();
    repo.write("openspec/reviewer/prompts/pairing.md", "mine\n");
    let export = files::export_prompts(repo.root()).expect("the export writes");
    assert_eq!(export.skipped, ["openspec/reviewer/prompts/pairing.md"]);
    assert_eq!(export.written.len(), 2);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("openspec/reviewer/prompts/pairing.md")).unwrap(),
        "mine\n"
    );
}

#[test]
fn prompt_templates_ship_and_can_be_overridden__override() {
    let repo = Repo::new();
    assert_eq!(
        files::load_template(repo.root(), "pairing.md"),
        files::PAIRING
    );
    repo.write("openspec/reviewer/prompts/pairing.md", "read it my way\n");
    assert_eq!(
        files::load_template(repo.root(), "pairing.md"),
        "read it my way\n"
    );
}

#[test]
fn the_default_prompt_asks_for_judgment_not_repetition() {
    for template in [files::PAIRING, files::CHANGE] {
        for phrase in [
            "more than one condition",
            "no scenario exercises",
            "plain language",
            "glossary term used against",
            "sibling requirement",
            "archived change",
        ] {
            assert!(
                template.contains(phrase),
                "the template asks for `{phrase}`"
            );
        }
        assert!(
            template.contains("Do not repeat them"),
            "the template forbids restating known findings"
        );
    }
}

#[test]
fn the_default_prompt_asks_for_judgment_not_repetition__schema_lists_every_kind() {
    for kind in HintKind::ALL {
        if kind == HintKind::Unparsed {
            assert!(
                !files::HINTS.contains(kind.as_str()),
                "`unparsed` is the parser's, not the agent's"
            );
            continue;
        }
        assert!(
            files::HINTS.contains(kind.as_str()),
            "the schema lists `{kind}`"
        );
    }
}

const RULES_YAML: &str = "\
schema: spec-driven

context: |
  A test project.

rules:
  specs:
    - One condition per keyword line. A second condition goes on its own
      `- **AND**` line.
    - Name the thing: a path, a key, a flag.
  proposal:
    - Under 600 words.
";

/// A repository whose change drops terms a sibling still uses and whose
/// text says a glossary term, so one pairing carries a warning, a drift
/// hit and a term.
fn prompt_repo() -> Repo {
    let repo = Repo::new();
    repo.write("openspec/config.yaml", RULES_YAML)
        .canon("key-rotation", &fixture("drift", "key-rotation.md"))
        .canon(
            "keyring-tombstones",
            &fixture("drift", "keyring-tombstones.md"),
        )
        .canon("definitions", &fixture("glossary", "definitions.md"))
        .delta(
            "epoch-retire",
            "key-rotation",
            &fixture("drift", "delta.md"),
        )
        .write(
            "openspec/changes/epoch-retire/proposal.md",
            "# epoch-retire\n\nRetire the epoch.\n",
        );
    repo
}

fn review_of(repo: &Repo, change: &str) -> openspec_reviewer::review::Review {
    let source = openspec_reviewer::source::ChangeSource {
        root: repo.root().into(),
        name: change.into(),
    };
    let snapshot = openspec_reviewer::source::Source::fetch(&source).expect("the change reads");
    openspec_reviewer::build::build_review(repo.root(), &snapshot).expect("the review builds")
}

fn session_of(repo: &Repo) -> Session {
    Session::open(repo.root(), repo.state_home().join("prompts"))
}

#[test]
fn a_prompt_file_carries_the_pairings_full_context__pairing_prompt() {
    let repo = prompt_repo();
    let review = review_of(&repo, "epoch-retire");
    let session = session_of(&repo).with_glossary(review.glossary.clone());
    let pairing = review.pairings().next().expect("one pairing");
    assert!(
        pairing
            .findings
            .iter()
            .any(|f| f.severity == openspec_reviewer::review::Severity::Warning),
        "the fixture pairing has a warning: {:?}",
        pairing.findings
    );
    let text = session.pairing_text(pairing, false);

    assert!(text.contains("## Project rules"));
    assert!(text.contains("One condition per keyword line"));
    assert!(!text.contains("Under 600 words"), "only the spec rules");
    assert!(text.contains("## Before"));
    assert!(text.contains("## After"));
    assert!(text.contains("## Known findings"));
    assert!(
        text.contains("- warning  "),
        "the finding carries its severity"
    );
    assert!(text.contains("## Sibling texts"));
    assert!(text.contains("keyring-tombstones §"));
    assert!(text.contains("## Glossary"));
    assert!(text.contains("### ledger"));
    assert!(text.contains("Admitted: log"));

    let at = |heading: &str| text.find(heading).expect(heading);
    assert!(
        at("## Project rules") < at("## Pairing")
            && at("## Pairing") < at("## Known findings")
            && at("## Known findings") < at("## Sibling texts")
            && at("## Sibling texts") < at("## Glossary"),
        "the sections come in the order the requirement gives"
    );
}

#[test]
fn a_prompt_file_carries_the_pairings_full_context__no_glossary() {
    let repo = prompt_repo();
    repo.remove("openspec/specs/definitions");
    let review = review_of(&repo, "epoch-retire");
    let session = session_of(&repo).with_glossary(review.glossary.clone());
    let pairing = review.pairings().next().expect("one pairing");
    let text = session.pairing_text(pairing, false);
    assert!(
        !text.contains("## Glossary"),
        "no heading with nothing under it"
    );
    assert!(text.contains("## Sibling texts"), "the rest is unchanged");
    assert!(text.contains("## Project rules"));
}

#[test]
fn a_prompt_file_carries_the_pairings_full_context__reviewer_note() {
    let repo = prompt_repo();
    let review = review_of(&repo, "epoch-retire");
    let session = session_of(&repo);
    let mut pairing = review.pairings().next().expect("one pairing").clone();
    assert!(
        !session
            .pairing_text(&pairing, false)
            .contains("## Reviewer note"),
        "no note, no section"
    );
    pairing.state.note = Some("check the grace window".to_string());
    let text = session.pairing_text(&pairing, false);
    assert!(text.contains("## Reviewer note"));
    assert!(text.contains("check the grace window"));
}

#[test]
fn a_prompt_file_carries_the_pairings_full_context__whole_change() {
    let repo = prompt_repo();
    let review = review_of(&repo, "epoch-retire");
    let session = session_of(&repo).with_glossary(review.glossary.clone());
    let pairings: Vec<_> = review.pairings().collect();
    let text = session.change_text(
        "epoch-retire",
        Some("# epoch-retire\n\nRetire the epoch.\n"),
        &pairings,
        false,
    );
    assert!(text.contains("## Proposal"));
    assert!(text.contains("Retire the epoch."));
    for p in &pairings {
        assert!(
            text.contains(&format!("# {} § {}", p.capability, p.name)),
            "{} has a section",
            p.name
        );
    }
    assert_eq!(
        text.matches("## Project rules").count(),
        1,
        "the rules are quoted once"
    );
}

#[test]
fn a_prompt_file_carries_the_pairings_full_context__the_schema_is_batch_only() {
    let repo = prompt_repo();
    let review = review_of(&repo, "epoch-retire");
    let session = session_of(&repo);
    let pairing = review.pairings().next().expect("one pairing");
    assert!(!session.pairing_text(pairing, false).contains("## Reply"));
    assert!(session.pairing_text(pairing, true).contains("## Reply"));
}

#[test]
fn a_prompt_file_carries_the_pairings_full_context__rules_read_from_config_yaml() {
    let repo = Repo::new();
    repo.write("openspec/config.yaml", RULES_YAML);
    let rules = openspec_reviewer::assist::spec_rules(repo.root());
    assert_eq!(rules.len(), 2);
    assert!(rules[0].starts_with("One condition per keyword line."));
    assert!(rules[0].ends_with("`- **AND**` line."));
    assert_eq!(rules[1], "Name the thing: a path, a key, a flag.");
    assert!(
        openspec_reviewer::assist::spec_rules(Repo::new().root()).is_empty(),
        "no config.yaml, no rules"
    );
}

#[test]
fn prompt_templates_ship_and_can_be_overridden__the_prompts_subcommand() {
    let repo = Repo::new();
    let first = run_in(repo.root(), &["assist", "prompts"]);
    assert!(first.status.success());
    assert!(stdout(&first).contains("wrote openspec/reviewer/prompts/pairing.md"));
    let again = run_in(repo.root(), &["assist", "prompts"]);
    assert!(stdout(&again).contains("kept  openspec/reviewer/prompts/pairing.md"));
}
