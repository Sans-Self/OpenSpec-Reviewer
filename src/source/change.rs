//! `change <name>`: the change as it is in the working tree. After side only.

use super::{FileChange, Snapshot, Source, SourceError};
use std::path::{Path, PathBuf};

pub struct ChangeSource {
    pub root: PathBuf,
    pub name: String,
}

fn list_changes(changes: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(changes)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "archive")
        .collect();
    names.sort();
    names
}

fn is_archived(changes: &Path, name: &str) -> bool {
    std::fs::read_dir(changes.join("archive"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .any(|dir| dir == name || dir.ends_with(&format!("-{name}")))
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

impl Source for ChangeSource {
    fn fetch(&self) -> Result<Snapshot, SourceError> {
        let changes = self.root.join("openspec").join("changes");
        let dir = changes.join(&self.name);
        if !dir.is_dir() {
            if is_archived(&changes, &self.name) {
                return Err(SourceError::ArchivedChange(self.name.clone()));
            }
            return Err(SourceError::UnknownChange {
                path: dir,
                found: list_changes(&changes),
            });
        }
        let mut paths = Vec::new();
        walk(&dir, &mut paths).map_err(SourceError::io(format!("reading {}", dir.display())))?;
        let files = paths
            .into_iter()
            .map(|abs| {
                let rel = abs.strip_prefix(&self.root).unwrap_or(&abs).to_path_buf();
                let text = std::fs::read_to_string(&abs)
                    .map_err(SourceError::io(format!("reading {}", abs.display())))?;
                Ok(FileChange {
                    path: rel,
                    before: None,
                    after: Some(text),
                })
            })
            .collect::<Result<Vec<_>, SourceError>>()?;
        Ok(Snapshot {
            files,
            origin: format!("change {}", self.name),
        })
    }
}
