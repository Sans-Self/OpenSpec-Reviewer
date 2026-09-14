//! `openspec/reviewer.toml`: what one repository has to say about itself
//! before the lint knows where to look.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const CONFIG_PATH: &str = "openspec/reviewer.toml";

pub const TEMPLATE: &str = r#"[lint]
# Where tests and other citing source live, and which files to read.
source_roots    = ["src", "tests"]
source_globs    = ["**/*.rs"]
skip_dirs       = ["target"]

# Evidence cited from canon specs: paths, regression test names, commits.
path_prefixes   = ["src", "tests", "docs"]
path_extensions = ["rs", "md", "toml"]
test_pattern    = "bug__\\w+"

# A call helper that builds `spec:` tags at runtime, if the repository has one.
# cite_helper   = "cite"

# Change names must start with one of these scopes and a hyphen.
# change_scopes = ["ui", "api"]
# grandfathered = ["legacy-change"]

[term_drift]
max_common = 5
"#;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub lint: Lint,
    #[serde(default)]
    pub term_drift: TermDrift,
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
    #[error("no {path}; the lint needs to know where source lives. A minimal file:\n\n{TEMPLATE}")]
    Missing { path: PathBuf },
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {message}")]
    Malformed { path: PathBuf, message: String },
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
