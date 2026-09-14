//! `diff [<path>]`: a unified diff applied to the working tree.

use super::{FileChange, Snapshot, Source, SourceError};
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct DiffSource {
    pub root: PathBuf,
    /// `None` or `-` reads stdin.
    pub path: Option<PathBuf>,
}

impl Source for DiffSource {
    fn fetch(&self) -> Result<Snapshot, SourceError> {
        let text = match &self.path {
            Some(p) if p.as_os_str() != "-" => std::fs::read_to_string(p)
                .map_err(SourceError::io(format!("reading {}", p.display())))?,
            _ => {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .map_err(SourceError::io("reading stdin"))?;
                buf
            }
        };
        let origin = match &self.path {
            Some(p) if p.as_os_str() != "-" => format!("diff {}", p.display()),
            _ => "diff (stdin)".to_string(),
        };
        parse_diff(&self.root, &text, origin)
    }
}

/// One `diff --git` section of a multi-file patch.
struct FilePatch<'a> {
    text: &'a str,
    old: Option<String>,
    new: Option<String>,
}

fn header_path(line: &str, prefix: &str) -> Option<Option<String>> {
    let rest = line.strip_prefix(prefix)?;
    let name = rest.split('\t').next().unwrap_or(rest).trim();
    if name == "/dev/null" {
        return Some(None);
    }
    let name = name
        .strip_prefix("a/")
        .or_else(|| name.strip_prefix("b/"))
        .unwrap_or(name);
    Some(Some(name.to_string()))
}

fn split_files(text: &str) -> Vec<FilePatch<'_>> {
    let mut starts: Vec<usize> = text
        .match_indices("diff --git ")
        .filter(|(i, _)| *i == 0 || text.as_bytes()[i - 1] == b'\n')
        .map(|(i, _)| i)
        .collect();
    if starts.is_empty() {
        starts.push(0);
    }
    starts
        .iter()
        .enumerate()
        .map(|(n, &start)| {
            let end = starts.get(n + 1).copied().unwrap_or(text.len());
            let chunk = &text[start..end];
            let mut old = None;
            let mut new = None;
            for line in chunk.lines() {
                if line.starts_with("@@ ") {
                    break;
                }
                if let Some(p) = header_path(line, "--- ") {
                    old = Some(p);
                } else if let Some(p) = header_path(line, "+++ ") {
                    new = Some(p);
                }
            }
            FilePatch {
                text: chunk,
                old: old.flatten(),
                new: new.flatten(),
            }
        })
        .collect()
}

fn hunk_header(patch: &diffy::Patch<'_, str>, message: &str) -> String {
    let index: Option<usize> = message
        .rsplit('#')
        .next()
        .and_then(|n| n.trim().parse().ok());
    index
        .and_then(|i| patch.hunks().get(i.wrapping_sub(1)))
        .map(|h| format!("@@ -{} +{} @@", h.old_range(), h.new_range()))
        .unwrap_or_else(|| message.to_string())
}

pub fn parse_diff(root: &Path, text: &str, origin: String) -> Result<Snapshot, SourceError> {
    let files = split_files(text)
        .into_iter()
        .filter(|f| f.text.contains("\n@@ ") || f.text.starts_with("@@ "))
        .filter_map(|f| {
            let path = f.new.clone().or_else(|| f.old.clone())?;
            path.starts_with("openspec/").then_some((path, f))
        })
        .map(|(path, f)| apply_file(root, &path, &f))
        .collect::<Result<Vec<_>, _>>()?;
    if files.is_empty() {
        return Err(SourceError::NoOpenSpecContent);
    }
    Ok(Snapshot { files, origin })
}

fn apply_file(root: &Path, path: &str, f: &FilePatch<'_>) -> Result<FileChange, SourceError> {
    let patch = diffy::Patch::from_str(f.text).map_err(|e| SourceError::PatchParse {
        file: path.to_string(),
        message: e.to_string(),
    })?;
    let created = f.old.is_none();
    let deleted = f.new.is_none();
    let before = if created {
        None
    } else {
        Some(std::fs::read_to_string(root.join(path)).map_err(|_| {
            SourceError::MissingPreimage {
                file: path.to_string(),
            }
        })?)
    };
    let after = if deleted {
        None
    } else {
        let base = before.as_deref().unwrap_or("");
        Some(
            diffy::apply(base, &patch).map_err(|e| SourceError::HunkMismatch {
                file: path.to_string(),
                hunk: hunk_header(&patch, &e.to_string()),
            })?,
        )
    };
    Ok(FileChange {
        path: PathBuf::from(path),
        before,
        after,
    })
}
