//! What the lint reads from the repository: raw spec texts, open changes
//! with their delta texts, source files under the configured roots, and
//! two probes into the filesystem and git.

use super::canon::CanonError;
use super::load_canon;
use crate::citations::config::Lint;
use crate::citations::evidence::Probe;
use crate::citations::scan::{OpenChange, SourceFile, SpecFile};
use crate::citations::structure::ChangeDir;
use crate::model::{parse_delta_spec, Canon, DeltaSpec};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(transparent)]
    Canon(#[from] CanonError),
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("openspec/reviewer.toml: source_globs: {0}")]
    Glob(ignore::Error),
}

pub struct Workspace {
    pub root: PathBuf,
    pub canon: Canon,
    pub specs: Vec<SpecFile>,
    pub changes: Vec<OpenChange>,
    pub change_dirs: Vec<ChangeDir>,
    pub sources: Vec<SourceFile>,
}

fn read(path: &Path) -> Result<String, WorkspaceError> {
    std::fs::read_to_string(path).map_err(|source| WorkspaceError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn dirs_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn rel(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn spec_files(root: &Path, specs_dir: &Path) -> Result<Vec<SpecFile>, WorkspaceError> {
    let mut out = Vec::new();
    for dir in dirs_sorted(specs_dir) {
        let file = dir.join("spec.md");
        let Some(name) = dir.file_name() else {
            continue;
        };
        if !file.is_file() {
            continue;
        }
        out.push(SpecFile {
            capability: name.to_string_lossy().into_owned(),
            path: rel(root, &file),
            text: read(&file)?,
        });
    }
    Ok(out)
}

/// A change whose deltas do not parse is skipped here: it fails its own
/// review, and the lint should not double-report.
fn open_changes(root: &Path, changes_dir: &Path) -> Result<Vec<OpenChange>, WorkspaceError> {
    let mut out = Vec::new();
    for dir in dirs_sorted(changes_dir) {
        let Some(name) = dir.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        if name == "archive" {
            continue;
        }
        let delta_files = spec_files(root, &dir.join("specs"))?;
        let deltas: Result<Vec<DeltaSpec>, _> = delta_files
            .iter()
            .map(|f| {
                parse_delta_spec(&f.capability, &f.path.to_string_lossy(), &f.text)
                    .map(DeltaSpec::join_renames)
            })
            .collect();
        let Ok(deltas) = deltas else { continue };
        out.push(OpenChange {
            name,
            deltas,
            delta_files,
        });
    }
    Ok(out)
}

fn change_dirs(changes_dir: &Path) -> Vec<ChangeDir> {
    dirs_sorted(changes_dir)
        .into_iter()
        .filter_map(|dir| {
            let name = dir.file_name()?.to_string_lossy().into_owned();
            (name != "archive").then(|| ChangeDir {
                has_marker: dir.join("proposal.md").is_file()
                    || dir.join(".openspec.yaml").is_file(),
                name,
            })
        })
        .collect()
}

fn source_files(root: &Path, lint: &Lint) -> Result<Vec<SourceFile>, WorkspaceError> {
    let Some(roots) = &lint.source_roots else {
        return Ok(Vec::new());
    };
    let mut files = Vec::new();
    for source_root in roots {
        let base = root.join(source_root);
        if !base.is_dir() {
            continue;
        }
        let mut overrides = OverrideBuilder::new(&base);
        for glob in lint.source_globs.iter().flatten() {
            overrides.add(glob).map_err(WorkspaceError::Glob)?;
        }
        let overrides = overrides.build().map_err(WorkspaceError::Glob)?;
        let skip = lint.skip_dirs.clone();
        let mut builder = WalkBuilder::new(&base);
        builder
            .overrides(overrides)
            .hidden(false)
            .git_global(false)
            .git_exclude(false)
            .require_git(false)
            .sort_by_file_path(Ord::cmp)
            .filter_entry(move |e| {
                !(e.file_type().is_some_and(|t| t.is_dir())
                    && skip.iter().any(|s| e.file_name().to_string_lossy() == *s))
            });
        for entry in builder.build() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            files.push(SourceFile {
                path: rel(root, entry.path()),
                text,
            });
        }
    }
    Ok(files)
}

impl Workspace {
    pub fn load(root: &Path, lint: &Lint) -> Result<Workspace, WorkspaceError> {
        let openspec = root.join("openspec");
        Ok(Workspace {
            root: root.to_path_buf(),
            canon: load_canon(root)?,
            specs: spec_files(root, &openspec.join("specs"))?,
            changes: open_changes(root, &openspec.join("changes"))?,
            change_dirs: change_dirs(&openspec.join("changes")),
            sources: source_files(root, lint)?,
        })
    }

    pub fn input(&self) -> crate::citations::lint::Input<'_> {
        crate::citations::lint::Input {
            canon: &self.canon,
            specs: &self.specs,
            changes: &self.changes,
            change_dirs: &self.change_dirs,
            sources: &self.sources,
            probe: self,
        }
    }
}

impl Probe for Workspace {
    fn path_exists(&self, rel: &str) -> bool {
        self.root.join(rel).exists()
    }

    fn is_commit(&self, hash: &str) -> bool {
        Command::new("git")
            .args(["cat-file", "-e", &format!("{hash}^{{commit}}")])
            .current_dir(&self.root)
            .output()
            .is_ok_and(|o| o.status.success())
    }
}
