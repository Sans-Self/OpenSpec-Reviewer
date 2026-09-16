//! Skills an agent loads to work with the reviewer: six `SKILL.md`
//! bodies shipped in the binary, rendered with frontmatter, and the plan
//! for writing them into a repository. Reading and writing the files is
//! `source::skills`; this module only decides what each file should say.

use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CLAUDE_SKILLS: &str = ".claude/skills";
pub const CLAUDE_COMMANDS: &str = ".claude/commands/opsx-reviewer";
pub const AGENTS_SKILLS: &str = ".agents/skills";
pub const OVERRIDES: &str = "openspec/reviewer/skills";
/// The canon file and requirement whose presence means the target
/// repository is this tool's own, where the skills' citations resolve.
pub const HOME_MARKER: (&str, &str) = (
    "openspec/specs/glossary/spec.md",
    "### Requirement: A recurring undefined term is a note",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skill {
    /// The short name; the installed skill is `opsx-reviewer-<name>`.
    pub name: &'static str,
    pub description: &'static str,
    pub body: &'static str,
    /// Whether a user calls it, and so gets a `/opsx-reviewer:<name>` command.
    pub command: bool,
}

impl Skill {
    pub fn full_name(&self) -> String {
        format!("opsx-reviewer-{}", self.name)
    }
}

pub const SKILLS: &[Skill] = &[
    Skill {
        name: "workflow",
        description: "How to work with openspec-reviewer in a repository that has an openspec/ directory and the openspec-reviewer binary: when to run the review and the lint, what each finding kind means, how to write a spec: citation, and which opsx-reviewer skill handles a finding that needs judgment.",
        body: include_str!("workflow.md"),
        command: false,
    },
    Skill {
        name: "define",
        description: "Turn openspec-reviewer's recurring undefined terms into a drafted definitions delta in a new change. Writes under openspec/changes/ on confirmation.",
        body: include_str!("define.md"),
        command: true,
    },
    Skill {
        name: "discover",
        description: "Find the vocabulary a repository's specs rely on in plain prose, across capabilities, and hand the candidates to define. Writes nothing itself; define writes under openspec/changes/ on confirmation.",
        body: include_str!("discover.md"),
        command: true,
    },
    Skill {
        name: "cite",
        description: "Add spec: citations to the tests that exercise requirements openspec-reviewer's coverage ledger lists as uncited. Edits test titles on confirmation.",
        body: include_str!("cite.md"),
        command: true,
    },
    Skill {
        name: "crossref",
        description: "Judge the sibling requirements an OpenSpec change puts in question, quote archived decisions, and draft MODIFIED entries into the change on confirmation.",
        body: include_str!("crossref.md"),
        command: true,
    },
    Skill {
        name: "triage",
        description: "Walk an OpenSpec change's reviewer findings in severity order, propose the usual fix for mechanical ones, apply to the change's deltas on confirmation, route judgment findings to crossref.",
        body: include_str!("triage.md"),
        command: true,
    },
];

pub fn skill(name: &str) -> Option<&'static Skill> {
    SKILLS.iter().find(|s| s.name == name)
}

/// FNV-1a over the body, as sixteen hex digits. Stable across Rust
/// versions, unlike `DefaultHasher`, and no dependency.
pub fn checksum(body: &str) -> String {
    let hash = body.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{hash:016x}")
}

/// The frontmatter block and the body after it, when the text has both.
pub fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some((&rest[..end], &rest[end + 5..]))
}

pub fn recorded_checksum(text: &str) -> Option<&str> {
    let (front, _) = split(text)?;
    front
        .lines()
        .find_map(|l| l.trim_start().strip_prefix("checksum: "))
        .map(str::trim)
}

fn frontmatter(
    name: &str,
    description: &str,
    tools: bool,
    body: &str,
    override_path: Option<&str>,
) -> String {
    let mut out = format!("---\nname: {name}\ndescription: {description}\n");
    if tools {
        out.push_str("allowed-tools: Bash(openspec-reviewer:*), Bash(openspec:*)\n");
    }
    out.push_str(&format!(
        "metadata:\n  generatedBy: openspec-reviewer {VERSION}\n  checksum: {}\n",
        checksum(body)
    ));
    if let Some(p) = override_path {
        out.push_str(&format!("  override: {p}\n"));
    }
    out.push_str("---\n");
    out
}

/// A `spec:` citation inside backticks loses its prefix and keeps the
/// names: in a repository whose canon is not this tool's, the citations
/// are documentation.
pub fn plain_citations(body: &str) -> String {
    Regex::new(r"`spec:([a-z0-9-]+)\s*§\s*([^`]+)`")
        .expect("citation pattern compiles")
        .replace_all(body, "`$1 § $2`")
        .into_owned()
}

/// What the target repository has, measured by `source::skills`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Layout {
    pub claude: bool,
    pub agents: bool,
    /// The target is this tool's own repository.
    pub home: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Skill,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileState {
    NotInstalled,
    UpToDate,
    /// Written by an earlier version and untouched since.
    Outdated,
    /// The body no longer matches the checksum the tool recorded.
    Edited,
}

impl fmt::Display for FileState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FileState::NotInstalled => "not installed",
            FileState::UpToDate => "up to date",
            FileState::Outdated => "outdated",
            FileState::Edited => "edited",
        })
    }
}

impl FileState {
    pub fn of(existing: Option<&str>, fresh: &str) -> FileState {
        let Some(text) = existing else {
            return FileState::NotInstalled;
        };
        if text == fresh {
            return FileState::UpToDate;
        }
        match (recorded_checksum(text), split(text)) {
            (Some(recorded), Some((_, body))) if recorded == checksum(body) => FileState::Outdated,
            _ => FileState::Edited,
        }
    }
}

/// One file the install owns: where it goes, what it should say, and
/// what is there now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub skill: &'static Skill,
    pub kind: FileKind,
    pub path: String,
    pub content: String,
    pub state: FileState,
}

impl Planned {
    /// A hand-edited file is the one thing the install does not touch.
    pub fn writes(&self) -> bool {
        self.state != FileState::Edited
    }
}

/// The body a skill gets: the project's override when there is one.
fn body_for(
    skill: &Skill,
    layout: Layout,
    overrides: &BTreeMap<String, String>,
) -> (String, Option<String>) {
    match overrides.get(skill.name) {
        Some(text) => (text.clone(), Some(format!("{OVERRIDES}/{}.md", skill.name))),
        None if layout.home => (skill.body.to_string(), None),
        None => (plain_citations(skill.body), None),
    }
}

pub fn render_skill(skill: &Skill, layout: Layout, overrides: &BTreeMap<String, String>) -> String {
    let (body, override_path) = body_for(skill, layout, overrides);
    frontmatter(
        &skill.full_name(),
        skill.description,
        true,
        &body,
        override_path.as_deref(),
    ) + &body
}

pub fn render_command(skill: &Skill) -> String {
    let body = format!(
        "Load the `{}` skill and follow it. Arguments: $ARGUMENTS\n",
        skill.full_name()
    );
    frontmatter(
        &format!("\"opsx-reviewer: {}\"", skill.name),
        skill.description,
        false,
        &body,
        None,
    ) + &body
}

/// Every path the install would own under this layout, in write order.
pub fn paths(layout: Layout) -> Vec<(&'static Skill, FileKind, String)> {
    let mut out = Vec::new();
    for skill in SKILLS {
        out.push((
            skill,
            FileKind::Skill,
            format!("{CLAUDE_SKILLS}/{}/SKILL.md", skill.full_name()),
        ));
        if layout.agents {
            out.push((
                skill,
                FileKind::Skill,
                format!("{AGENTS_SKILLS}/{}/SKILL.md", skill.full_name()),
            ));
        }
        if skill.command {
            out.push((
                skill,
                FileKind::Command,
                format!("{CLAUDE_COMMANDS}/{}.md", skill.name),
            ));
        }
    }
    out
}

/// The install, decided: one entry per owned path, against what the
/// repository holds at those paths (`existing`) and the project's
/// override bodies keyed by skill name.
pub fn plan(
    layout: Layout,
    existing: &BTreeMap<String, String>,
    overrides: &BTreeMap<String, String>,
) -> Result<Vec<Planned>, SkillsError> {
    if !layout.claude {
        return Err(SkillsError::NoClaude);
    }
    Ok(paths(layout)
        .into_iter()
        .map(|(skill, kind, path)| {
            let content = match kind {
                FileKind::Skill => render_skill(skill, layout, overrides),
                FileKind::Command => render_command(skill),
            };
            let state = FileState::of(existing.get(&path).map(String::as_str), &content);
            Planned {
                skill,
                kind,
                path,
                content,
                state,
            }
        })
        .collect())
}

#[derive(Debug, Error)]
pub enum SkillsError {
    #[error("no .claude/ here; `skills install` writes into an agent's directory and does not create one")]
    NoClaude,
    #[error("cannot read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot write {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
