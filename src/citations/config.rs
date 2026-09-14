//! `openspec/reviewer.toml`: what one repository has to say about itself
//! before the lint knows where to look.

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const CONFIG_PATH: &str = "openspec/reviewer.toml";

/// Directories `lint init` looks for, and the extensions it recognizes
/// under them. Anything outside these lists is not a citation target the
/// lint knows what to do with.
pub const CANDIDATE_ROOTS: &[&str] = &[
    "src", "lib", "apps", "packages", "crates", "services", "tests", "test",
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
[definitions]
capability     = "definitions"
min_recurrence = 3
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
    toml::from_str(text).map_err(|e| ConfigError::Malformed {
        path: path.to_path_buf(),
        message: e.message().to_string(),
    })
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
