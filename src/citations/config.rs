//! `openspec/reviewer.toml`: what one repository has to say about itself
//! before the lint knows where to look.

use crate::model::{Entry, Ignores, Scope, TermIgnore};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const CONFIG_PATH: &str = "openspec/reviewer.toml";

/// Directories `lint init` looks for, and the extensions it recognizes
/// under them. Anything outside these lists is not a citation target the
/// lint knows what to do with.
pub const CANDIDATE_ROOTS: &[&str] = &[
    "src", "lib", "apps", "packages", "crates", "services", "tests", "test", ".claude", ".agents",
];
pub const CANDIDATE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "mjs", "mts", "py", "go", "ex", "exs", "heex", "md", "json", "yaml",
    "yml", "toml", "css",
];
/// Formats with no comment syntax: nowhere to write a citation, so
/// `lint init` leaves them out of `source_globs`.
pub const NO_CITATIONS: &[&str] = &["json"];
pub const DEFAULT_SKIP_DIRS: &[&str] =
    &["node_modules", "target", "dist", "build", "_build", "deps"];

/// What `lint init` measured in the working tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Survey {
    pub roots: Vec<String>,
    pub extensions: BTreeSet<String>,
    pub has_docs: bool,
}

impl Survey {
    /// The shape of this repository, for the refusal's example.
    pub fn example() -> Survey {
        Survey {
            roots: vec!["src".into(), "tests".into()],
            extensions: ["rs", "md", "toml"].iter().map(|s| s.to_string()).collect(),
            has_docs: true,
        }
    }
}

fn toml_list(items: impl IntoIterator<Item = String>) -> String {
    let quoted: Vec<String> = items.into_iter().map(|s| format!("\"{s}\"")).collect();
    format!("[{}]", quoted.join(", "))
}

/// The configuration file as text. One renderer serves the refusal's
/// example and the file `lint init` writes, so the two cannot drift.
pub fn render(survey: &Survey) -> String {
    let roots_note = if survey.roots.is_empty() {
        "# No source directory found among the usual names; list yours here.\n"
    } else {
        ""
    };
    let mut path_extensions = survey.extensions.clone();
    path_extensions.insert("md".to_string());
    let mut prefixes = survey.roots.clone();
    if survey.has_docs {
        prefixes.push("docs".to_string());
    }
    format!(
        r#"[lint]
# Where tests and other citing source live, and which files to read.
{roots_note}source_roots    = {roots}
source_globs    = {globs}
skip_dirs       = {skip}

# Evidence cited from canon specs: paths, regression test names, commits.
path_prefixes   = {prefixes}
path_extensions = {path_extensions}
test_pattern    = "bug__\\w+"

# A call helper that builds `spec:` tags at runtime, if the repository has one.
# cite_helper   = "cite"

# Change names must start with one of these scopes and a hyphen.
# change_scopes = ["ui", "api"]
# grandfathered = ["legacy-change"]

[term_drift]
max_common = 5

# The glossary capability and how often an undefined span must recur.
# An empty capability switches the glossary checks off.
[definitions]
capability     = "definitions"
min_recurrence = 3

# Findings read and dismissed. Every entry says why, and the lint warns when
# one stops matching anything. `in` confines a term to whole capabilities or
# to single requirements; without it the term is ignored everywhere.
# [[definitions.ignore]]
# term   = "mountType"
# in     = ["citations", "glossary § A term lists the words that are acceptable for it"]
# reason = "a configuration key quoted in prose, not a concept"

# [[lint.ignore_uncited]]
# requirement = "citations § Coverage lists citing tests per requirement"
# reason      = "asserted by the ledger snapshot, which cannot cite itself"
"#,
        roots = toml_list(survey.roots.iter().cloned()),
        globs = toml_list(
            survey
                .extensions
                .iter()
                .filter(|e| !NO_CITATIONS.contains(&e.as_str()))
                .map(|e| format!("**/*.{e}"))
        ),
        skip = toml_list(DEFAULT_SKIP_DIRS.iter().map(|s| s.to_string())),
        prefixes = toml_list(prefixes),
        path_extensions = toml_list(path_extensions),
    )
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub lint: Lint,
    #[serde(default)]
    pub term_drift: TermDrift,
    #[serde(default)]
    pub definitions: Definitions,
}

impl Config {
    /// The requirements `[[lint.ignore_uncited]]` names, as register entries.
    /// An entry that names no requirement resolves against nothing, which is
    /// what the dangling check reports.
    pub fn ignored_requirements(&self) -> BTreeSet<Entry> {
        self.lint
            .ignore_uncited
            .iter()
            .filter_map(|e| Entry::parse(&e.requirement))
            .collect()
    }

    /// Every ignore entry says why it is there, and confines itself with an
    /// array. Checked once, at the edge, so the rest of the tool can read
    /// the lists as given.
    fn validate(&self, path: &Path) -> Result<(), ConfigError> {
        for entry in &self.definitions.ignore {
            if matches!(entry.within, Some(Scopes::Bare(_))) {
                return Err(ConfigError::ScopeNotAnArray {
                    path: path.to_path_buf(),
                    entry: entry.term.clone(),
                });
            }
            if entry.reason.is_none() {
                return Err(ConfigError::IgnoreWithoutReason {
                    path: path.to_path_buf(),
                    list: "[[definitions.ignore]]",
                    entry: entry.term.clone(),
                });
            }
        }
        for entry in &self.lint.ignore_uncited {
            if entry.reason.is_none() {
                return Err(ConfigError::IgnoreWithoutReason {
                    path: path.to_path_buf(),
                    list: "[[lint.ignore_uncited]]",
                    entry: entry.requirement.clone(),
                });
            }
        }
        Ok(())
    }
}

/// `[definitions]`: which capability is the glossary and how often a span
/// must recur before the lint suggests defining it. Absent means defaults,
/// never a refusal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definitions {
    #[serde(default = "default_capability")]
    pub capability: String,
    #[serde(default = "default_min_recurrence")]
    pub min_recurrence: usize,
    /// `[[definitions.ignore]]`: undefined-term findings read and dismissed.
    #[serde(default)]
    pub ignore: Vec<IgnoreTerm>,
}

/// One dismissed term. `parse_config` is the only way in, and it refuses an
/// entry without a reason and an `in` that is not an array, so a consumer
/// never sees either.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IgnoreTerm {
    pub term: String,
    #[serde(rename = "in")]
    pub within: Option<Scopes>,
    pub reason: Option<String>,
}

/// `in` is an array of scopes. A bare string parses so that the refusal can
/// name the entry rather than the shape.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Scopes {
    List(Vec<String>),
    Bare(String),
}

impl IgnoreTerm {
    /// The scopes the entry is confined to; empty is the whole repository.
    pub fn scopes(&self) -> Vec<Scope> {
        match &self.within {
            Some(Scopes::List(items)) => items.iter().map(|s| Scope::parse(s)).collect(),
            Some(Scopes::Bare(_)) | None => Vec::new(),
        }
    }
}

/// One requirement the coverage ledger must not mark uncited.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IgnoreUncited {
    pub requirement: String,
    pub reason: Option<String>,
}

impl Definitions {
    /// The dismissed terms as the checks read them.
    pub fn ignores(&self) -> Ignores {
        Ignores {
            terms: self
                .ignore
                .iter()
                .map(|e| TermIgnore {
                    term: e.term.clone(),
                    scopes: e.scopes(),
                })
                .collect(),
        }
    }
}

fn default_capability() -> String {
    crate::glossary::DEFAULT_CAPABILITY.to_string()
}

fn default_min_recurrence() -> usize {
    crate::glossary::DEFAULT_MIN_RECURRENCE
}

impl Default for Definitions {
    fn default() -> Definitions {
        Definitions {
            capability: default_capability(),
            min_recurrence: default_min_recurrence(),
            ignore: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lint {
    pub source_roots: Option<Vec<String>>,
    pub source_globs: Option<Vec<String>>,
    #[serde(default)]
    pub skip_dirs: Vec<String>,
    pub path_prefixes: Option<Vec<String>>,
    pub path_extensions: Option<Vec<String>>,
    pub test_pattern: Option<String>,
    pub cite_helper: Option<String>,
    pub change_scopes: Option<Vec<String>>,
    #[serde(default)]
    pub grandfathered: Vec<String>,
    /// `[[lint.ignore_uncited]]`: requirements the ledger must not mark.
    #[serde(default)]
    pub ignore_uncited: Vec<IgnoreUncited>,
}

impl Lint {
    /// Fields a present file left out, for the summary line.
    pub fn missing(&self) -> Vec<&'static str> {
        [
            ("source_roots", self.source_roots.is_none()),
            ("path_prefixes", self.path_prefixes.is_none()),
            ("path_extensions", self.path_extensions.is_none()),
            ("test_pattern", self.test_pattern.is_none()),
            ("change_scopes", self.change_scopes.is_none()),
        ]
        .into_iter()
        .filter_map(|(name, absent)| absent.then_some(name))
        .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermDrift {
    #[serde(default = "default_max_common")]
    pub max_common: usize,
}

fn default_max_common() -> usize {
    5
}

impl Default for TermDrift {
    fn default() -> TermDrift {
        TermDrift {
            max_common: default_max_common(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("no {path}; the lint needs to know where source lives. Run `openspec-reviewer lint init` to write one, or start from this minimal file:\n\n{}", render(&Survey::example()))]
    Missing { path: PathBuf },
    #[error("{path} already exists; edit it, or remove it and run `lint init` again")]
    Exists { path: PathBuf },
    #[error("no {dir} here; `lint init` writes its file next to an OpenSpec tree")]
    NoOpenSpec { dir: PathBuf },
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {message}")]
    Malformed { path: PathBuf, message: String },
    #[error("{path}: {list} entry `{entry}` has no `reason`; an ignore without a reason cannot be audited")]
    IgnoreWithoutReason {
        path: PathBuf,
        list: &'static str,
        entry: String,
    },
    #[error("{path}: [[definitions.ignore]] entry `{entry}`: `in` is an array of scopes, as in `in = [\"citations\"]`")]
    ScopeNotAnArray { path: PathBuf, entry: String },
}

/// Write the rendered survey to `openspec/reviewer.toml`. `create_new`
/// makes the existence check and the write one operation.
pub fn write_init(root: &Path, survey: &Survey) -> Result<PathBuf, ConfigError> {
    let dir = root.join("openspec");
    if !dir.is_dir() {
        return Err(ConfigError::NoOpenSpec {
            dir: PathBuf::from("openspec/"),
        });
    }
    let path = root.join(CONFIG_PATH);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => ConfigError::Exists {
                path: PathBuf::from(CONFIG_PATH),
            },
            _ => ConfigError::Io {
                path: path.clone(),
                source: e,
            },
        })?;
    std::io::Write::write_all(&mut file, render(survey).as_bytes()).map_err(|source| {
        ConfigError::Io {
            path: path.clone(),
            source,
        }
    })?;
    Ok(PathBuf::from(CONFIG_PATH))
}

pub fn parse_config(path: &Path, text: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(text).map_err(|e| ConfigError::Malformed {
        path: path.to_path_buf(),
        message: e.message().to_string(),
    })?;
    config.validate(path)?;
    Ok(config)
}

/// The config when the file exists, `Ok(None)` when it does not. The lint
/// turns `None` into a refusal; the review carries on without citations.
pub fn read_config(root: &Path) -> Result<Option<Config>, ConfigError> {
    let path = root.join(CONFIG_PATH);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_config(&path, &text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ConfigError::Io { path, source }),
    }
}

pub fn require_config(root: &Path) -> Result<Config, ConfigError> {
    read_config(root)?.ok_or_else(|| ConfigError::Missing {
        path: root.join(CONFIG_PATH),
    })
}
